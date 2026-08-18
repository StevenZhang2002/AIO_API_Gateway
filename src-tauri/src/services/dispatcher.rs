use std::collections::HashMap;

use rand::Rng;
use thiserror::Error;

use crate::adaptor::ChannelConfig;
use crate::db::models::Channel;

/// 分发错误
#[derive(Debug, Error)]
pub enum DispatchError {
    #[error("无可用渠道: {0}")]
    NoAvailable(String),
}

/// 渠道分发器：纯内存、无状态
///
/// 负载均衡策略：「优先级分组 + 组内权重随机」
/// - 按 priority 分组（数值越大优先级越高），只从最高优先级非空组中选择
/// - 组内按 weight 加权随机（概率 ∝ weight）；weight <= 0 不参与
/// - 跨组降级由调用方重试驱动：失败渠道进入 exclude 后，最高组耗尽自然落入次高组
pub struct Dispatcher;

impl Dispatcher {
    /// 从候选渠道中选择一个目标渠道
    ///
    /// - `candidates`：候选渠道列表（通常来自 ChannelRepo::find_enabled）
    /// - `model`：请求的模型名（外部名，与 channels.models 中存的一致）
    /// - `exclude`：本次请求已失败的渠道 id 列表（重试时排除，首轮为空）
    pub fn select<'a>(
        candidates: &'a [Channel],
        model: &str,
        exclude: &[&str],
    ) -> Result<&'a Channel, DispatchError> {
        // 1. 过滤：启用 + 未失败 + 模型匹配
        let mut filtered: Vec<&Channel> = candidates
            .iter()
            .filter(|c| c.status == 1)
            .filter(|c| !exclude.contains(&c.id.as_str()))
            .filter(|c| channel_supports_model(c, model))
            .collect();

        if filtered.is_empty() {
            return Err(diagnose_no_available(candidates, model, exclude));
        }

        // 2. 按 priority 分组，取最高优先级组
        let mut groups: HashMap<i32, Vec<&Channel>> = HashMap::new();
        for c in filtered.drain(..) {
            groups.entry(c.priority).or_default().push(c);
        }
        let top_group = groups
            .into_iter()
            .max_by_key(|(priority, _)| *priority)
            .map(|(_, group)| group)
            .expect("filtered 非空则分组必非空");

        // 3. 组内权重随机
        Ok(weighted_random(&top_group))
    }

    /// 将数据库 Channel 模型转换为适配器所需的 ChannelConfig
    ///
    /// JSON 字符串字段（models / config / model_mapping）解析失败时安全兜底：
    /// models → 空列表（通配），config / model_mapping → 空对象
    pub fn to_channel_config(channel: &Channel) -> ChannelConfig {
        ChannelConfig {
            base_url: channel.base_url.clone(),
            api_key: channel.api_key.clone(),
            models: serde_json::from_str(&channel.models).unwrap_or_default(),
            model_mapping: serde_json::from_str(&channel.model_mapping)
                .unwrap_or_else(|_| serde_json::json!({})),
            extra: serde_json::from_str(&channel.config).unwrap_or_else(|_| serde_json::json!({})),
        }
    }

    /// 构建故障转移队列：按优先级分组，组内按权重排序，返回有序的渠道列表
    ///
    /// 与 select() 的区别：
    /// - select()：只返回一个（加权随机）
    /// - build_queue()：返回完整的有序列表，供 ProxyService 依次尝试
    ///
    /// 队列构建规则：
    /// 1. 过滤：status=1 + 支持 model + 未在 exclude 中
    /// 2. 按 priority 降序排列组
    /// 3. 每组内按 weight 降序排列（权重大的排前面）
    /// 4. 同 weight 的渠道随机排列（避免固定顺序）
    pub fn build_queue<'a>(
        candidates: &'a [Channel],
        model: &str,
        exclude: &[&str],
    ) -> Vec<&'a Channel> {
        // 1. 过滤
        let filtered: Vec<&Channel> = candidates
            .iter()
            .filter(|c| c.status == 1)
            .filter(|c| !exclude.contains(&c.id.as_str()))
            .filter(|c| channel_supports_model(c, model))
            .collect();

        if filtered.is_empty() {
            return Vec::new();
        }

        // 2. 按 priority 分组
        let mut groups: HashMap<i32, Vec<&Channel>> = HashMap::new();
        for c in filtered {
            groups.entry(c.priority).or_default().push(c);
        }

        // 3. 按 priority 降序排列组，每组内按权重排序（同权重随机）
        let mut priorities: Vec<i32> = groups.keys().copied().collect();
        priorities.sort_by(|a, b| b.cmp(a)); // 降序

        let mut queue = Vec::new();
        for priority in priorities {
            let mut group = groups.remove(&priority).unwrap();
            // 组内按 weight 降序，同 weight 随机
            group.sort_by(|a, b| {
                b.weight.cmp(&a.weight).then_with(|| {
                    // 同 weight 时随机排序
                    let mut rng = rand::rng();
                    rng.random_range(0..2).cmp(&0)
                })
            });
            queue.extend(group);
        }

        queue
    }
}

/// 候选为空时区分失败原因，便于排查与展示
fn diagnose_no_available(
    candidates: &[Channel],
    model: &str,
    exclude: &[&str],
) -> DispatchError {
    let enabled: Vec<&Channel> = candidates.iter().filter(|c| c.status == 1).collect();
    if enabled.is_empty() {
        return DispatchError::NoAvailable("无启用渠道".to_string());
    }
    let not_excluded = enabled
        .iter()
        .filter(|c| !exclude.contains(&c.id.as_str()))
        .count();
    if not_excluded == 0 {
        return DispatchError::NoAvailable("可用渠道均已尝试失败".to_string());
    }
    DispatchError::NoAvailable(format!("无支持模型 {} 的渠道", model))
}

/// 模型匹配：models 为空数组（或非法 JSON）视为通配，支持所有模型
fn channel_supports_model(channel: &Channel, model: &str) -> bool {
    let models: Vec<String> = serde_json::from_str(&channel.models).unwrap_or_default();
    models.is_empty() || models.iter().any(|m| m == model)
}

/// 组内加权随机（累计权重法）
///
/// weight <= 0 的渠道不参与；若组内全部 weight <= 0，退化为均匀随机
fn weighted_random<'a>(group: &[&'a Channel]) -> &'a Channel {
    let mut rng = rand::rng();
    let valid: Vec<&Channel> = group.iter().copied().filter(|c| c.weight > 0).collect();

    if valid.is_empty() {
        let idx = rng.random_range(0..group.len());
        return group[idx];
    }

    let total: i32 = valid.iter().map(|c| c.weight).sum();
    let mut r = rng.random_range(0..total);
    for c in valid {
        if r < c.weight {
            return c;
        }
        r -= c.weight;
    }
    unreachable!("r < total，累计权重遍历必然命中")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channel(id: &str, status: i32, priority: i32, weight: i32, models: &str) -> Channel {
        Channel {
            id: id.to_string(),
            name: id.to_string(),
            r#type: "openai".to_string(),
            base_url: "https://example.com".to_string(),
            api_key: "sk-test".to_string(),
            models: models.to_string(),
            status,
            priority,
            weight,
            config: "{}".to_string(),
            model_mapping: "{}".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            last_test_at: None,
            last_test_ok: None,
        }
    }

    /// 权重 3:2:1 的分布应接近 50% / 33.3% / 16.7%（60000 次采样，容差 ±1000，约 10σ）
    #[test]
    fn weight_distribution() {
        let candidates = vec![
            channel("A", 1, 0, 3, r#"["gpt-4o"]"#),
            channel("B", 1, 0, 2, r#"["gpt-4o"]"#),
            channel("C", 1, 0, 1, r#"["gpt-4o"]"#),
        ];
        let mut counts = [0usize; 3];
        for _ in 0..60_000 {
            let picked = Dispatcher::select(&candidates, "gpt-4o", &[]).unwrap().id.clone();
            let idx = ["A", "B", "C"].iter().position(|x| *x == picked).unwrap();
            counts[idx] += 1;
        }
        assert!((counts[0] as i64 - 30_000).abs() < 1000, "A 命中 {}", counts[0]);
        assert!((counts[1] as i64 - 20_000).abs() < 1000, "B 命中 {}", counts[1]);
        assert!((counts[2] as i64 - 10_000).abs() < 1000, "C 命中 {}", counts[2]);
    }

    /// 高优先级组非空时，低优先级组永远不被选中
    #[test]
    fn top_priority_group_wins() {
        let candidates = vec![
            channel("high", 1, 10, 1, "[]"),
            channel("low", 1, 5, 100, "[]"),
        ];
        for _ in 0..1000 {
            let picked = Dispatcher::select(&candidates, "gpt-4o", &[]).unwrap();
            assert_eq!(picked.id, "high");
        }
    }

    /// 最高组全部排除后，落入次高优先级组
    #[test]
    fn exclude_falls_to_next_group() {
        let candidates = vec![
            channel("high", 1, 10, 1, "[]"),
            channel("low", 1, 5, 1, "[]"),
        ];
        let picked = Dispatcher::select(&candidates, "gpt-4o", &["high"]).unwrap();
        assert_eq!(picked.id, "low");
    }

    /// weight <= 0 不参与随机；全部 <= 0 时退化为均匀随机
    #[test]
    fn zero_weight_handling() {
        // 仅 A 权重 > 0，恒选 A
        let candidates = vec![
            channel("A", 1, 0, 3, "[]"),
            channel("B", 1, 0, 0, "[]"),
            channel("C", 1, 0, -1, "[]"),
        ];
        for _ in 0..1000 {
            assert_eq!(Dispatcher::select(&candidates, "m", &[]).unwrap().id, "A");
        }

        // 全部 weight <= 0，均匀随机：两者都应出现
        let all_zero = vec![
            channel("X", 1, 0, 0, "[]"),
            channel("Y", 1, 0, -2, "[]"),
        ];
        let mut seen = std::collections::HashSet::new();
        for _ in 0..1000 {
            seen.insert(Dispatcher::select(&all_zero, "m", &[]).unwrap().id.clone());
        }
        assert!(seen.contains("X") && seen.contains("Y"));
    }

    /// 模型过滤：匹配 / 不匹配 / 空数组通配 / 非法 JSON 通配
    #[test]
    fn model_filter() {
        let candidates = vec![
            channel("A", 1, 0, 1, r#"["gpt-4o", "gpt-4o-mini"]"#),
            channel("B", 1, 0, 1, r#"["claude-3-5-sonnet"]"#),
            channel("C", 1, 0, 1, "[]"),          // 空数组 = 通配
            channel("D", 1, 0, 1, "not-json"),    // 非法 JSON = 通配
        ];

        // 命中 A（C/D 也可能被选中，但绝不会选 B）
        for _ in 0..200 {
            let picked = Dispatcher::select(&candidates, "gpt-4o", &[]).unwrap();
            assert_ne!(picked.id, "B", "B 不支持 gpt-4o");
        }
        // 只有 C/D 支持 unknown-model
        for _ in 0..200 {
            let picked = Dispatcher::select(&candidates, "unknown-model", &[]).unwrap();
            assert!(picked.id == "C" || picked.id == "D");
        }
    }

    /// 无候选 / 全部禁用 / 全部排除 / 模型无匹配 → 各类 NoAvailable
    #[test]
    fn no_available_cases() {
        // 空列表
        let err = Dispatcher::select(&[], "m", &[]).unwrap_err().to_string();
        assert!(err.contains("无启用渠道"), "{}", err);

        // 全部禁用
        let disabled = vec![channel("A", 0, 0, 1, "[]")];
        let err = Dispatcher::select(&disabled, "m", &[]).unwrap_err().to_string();
        assert!(err.contains("无启用渠道"), "{}", err);

        // 全部被排除
        let candidates = vec![channel("A", 1, 0, 1, "[]"), channel("B", 1, 0, 1, "[]")];
        let err = Dispatcher::select(&candidates, "m", &["A", "B"]).unwrap_err().to_string();
        assert!(err.contains("均已尝试失败"), "{}", err);

        // 启用但无模型匹配
        let candidates = vec![channel("A", 1, 0, 1, r#"["gpt-4o"]"#)];
        let err = Dispatcher::select(&candidates, "claude-x", &[]).unwrap_err().to_string();
        assert!(err.contains("无支持模型 claude-x"), "{}", err);
    }

    /// 单渠道（无论 priority/weight）直接命中
    #[test]
    fn single_channel() {
        let candidates = vec![channel("only", 1, -5, 0, "[]")];
        for _ in 0..100 {
            assert_eq!(Dispatcher::select(&candidates, "m", &[]).unwrap().id, "only");
        }
    }

    /// Channel → ChannelConfig 字段映射
    #[test]
    fn to_channel_config_mapping() {
        let ch = Channel {
            id: "ch-1".to_string(),
            name: "测试渠道".to_string(),
            r#type: "openai".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: "sk-123".to_string(),
            models: r#"["gpt-4o","gpt-4o-mini"]"#.to_string(),
            status: 1,
            priority: 0,
            weight: 1,
            config: r#"{"temperature":0.7,"max_tokens":4096}"#.to_string(),
            model_mapping: r#"{"gpt-4o":"gpt-4o-2024-08-06"}"#.to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            last_test_at: None,
            last_test_ok: None,
        };
        let cfg = Dispatcher::to_channel_config(&ch);
        assert_eq!(cfg.base_url, "https://api.openai.com/v1");
        assert_eq!(cfg.api_key, "sk-123");
        assert_eq!(cfg.models, vec!["gpt-4o", "gpt-4o-mini"]);
        assert_eq!(cfg.model_mapping["gpt-4o"], "gpt-4o-2024-08-06");
        assert_eq!(cfg.extra["temperature"], 0.7);
        assert_eq!(cfg.extra["max_tokens"], 4096);
    }

    /// 非法 JSON 字段安全兜底：models → 空列表，config / model_mapping → 空对象
    #[test]
    fn to_channel_config_invalid_json_fallback() {
        let ch = Channel {
            id: "ch-2".to_string(),
            name: "坏数据渠道".to_string(),
            r#type: "custom".to_string(),
            base_url: "https://example.com/v1".to_string(),
            api_key: "sk-456".to_string(),
            models: "not-json".to_string(),
            status: 1,
            priority: 0,
            weight: 1,
            config: "not-json".to_string(),
            model_mapping: "not-json".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            last_test_at: None,
            last_test_ok: None,
        };
        let cfg = Dispatcher::to_channel_config(&ch);
        assert!(cfg.models.is_empty(), "非法 models 应兜底为空列表");
        assert_eq!(cfg.model_mapping, serde_json::json!({}));
        assert_eq!(cfg.extra, serde_json::json!({}));
    }
}
