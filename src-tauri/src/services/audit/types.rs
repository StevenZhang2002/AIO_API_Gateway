use serde::{Deserialize, Serialize};

/// 风险等级（从低到高）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 信息级：仅记录，无实际风险
    Info,
    /// 低风险：轻微异常，建议关注
    Low,
    /// 中风险：可疑行为，需要处理
    Medium,
    /// 高风险：明确威胁，应立即处置
    High,
    /// 严重：紧急威胁，必须阻断
    Critical,
}

impl RiskLevel {
    /// 数字权重，用于比较和排序
    pub fn weight(&self) -> u8 {
        match self {
            RiskLevel::Info => 0,
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        }
    }

    /// 取两个风险等级中较高的一个
    pub fn max(self, other: Self) -> Self {
        if self.weight() >= other.weight() {
            self
        } else {
            other
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Info => write!(f, "INFO"),
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
            RiskLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// 对请求的处置行为
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// 放行：请求正常通过
    Allow,
    /// 警告：放行但记录告警日志
    Warn,
    /// 脱敏：放行但自动遮蔽敏感内容
    Redact,
    /// 需确认：暂停请求，等待管理员确认后再决定是否放行
    NeedConfirm,
    /// 阻断：直接拒绝请求
    Block,
}

impl Action {
    /// 是否允许请求继续执行（Allow / Warn / Redact 均放行）
    pub fn is_allowed(&self) -> bool {
        matches!(self, Action::Allow | Action::Warn | Action::Redact)
    }

    /// 是否需要人工介入
    pub fn requires_human(&self) -> bool {
        matches!(self, Action::NeedConfirm | Action::Block)
    }

    /// 是否需要脱敏处理
    pub fn needs_redaction(&self) -> bool {
        matches!(self, Action::Redact)
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Allow => write!(f, "ALLOW"),
            Action::Warn => write!(f, "WARN"),
            Action::Redact => write!(f, "REDACT"),
            Action::NeedConfirm => write!(f, "NEED_CONFIRM"),
            Action::Block => write!(f, "BLOCK"),
        }
    }
}

/// SecurityAction 类型别名（与 Action 等价）
pub type SecurityAction = Action;

/// 安全审计模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityMode {
    /// 仅审计记录，不拦截
    Audit,
    /// 发现风险时发出警告
    Warn,
    /// 自动脱敏敏感内容
    Redact,
    /// 需要管理员确认后放行
    Confirm,
    /// 直接阻断高风险请求
    Block,
}

impl SecurityMode {
    /// 根据运行模式和风险等级，解析应执行的处置行为
    ///
    /// 决策矩阵：
    ///
    /// | mode   | Clean/Low | Medium | High   | Critical          |
    /// |--------|-----------|--------|--------|-------------------|
    /// | audit  | Allow     | Allow  | Allow  | Allow / Block*    |
    /// | warn   | Allow     | Warn   | Warn   | Warn / Block*     |
    /// | redact | Allow     | Allow  | Redact | Redact / Block*   |
    /// | block  | Allow     | Allow  | Block  | Block             |
    ///
    /// * 当 `block_on_critical = true` 时，Critical 级别一律 Block
    pub fn resolve_action(&self, risk: &RiskLevel, block_on_critical: bool) -> Action {
        // Critical 级别：block_on_critical 优先
        if *risk == RiskLevel::Critical && block_on_critical {
            return Action::Block;
        }

        match self {
            SecurityMode::Audit => Action::Allow,

            SecurityMode::Warn => match risk {
                RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical => Action::Warn,
                _ => Action::Allow,
            },

            SecurityMode::Redact => match risk {
                RiskLevel::High | RiskLevel::Critical => Action::Redact,
                _ => Action::Allow,
            },

            SecurityMode::Block => match risk {
                RiskLevel::High | RiskLevel::Critical => Action::Block,
                _ => Action::Allow,
            },

            SecurityMode::Confirm => match risk {
                RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical => Action::NeedConfirm,
                _ => Action::Allow,
            },
        }
    }
}

impl std::fmt::Display for SecurityMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityMode::Audit => write!(f, "AUDIT"),
            SecurityMode::Warn => write!(f, "WARN"),
            SecurityMode::Redact => write!(f, "REDACT"),
            SecurityMode::Confirm => write!(f, "CONFIRM"),
            SecurityMode::Block => write!(f, "BLOCK"),
        }
    }
}

/// 安全审计引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SecuritySettings {
    /// 总开关
    pub enabled: bool,
    /// 运行模式
    pub mode: SecurityMode,
    /// 是否扫描请求
    pub scan_request: bool,
    /// 是否扫描响应
    pub scan_response: bool,
    /// 是否启用 Unicode 隐写检测
    pub scan_unicode: bool,
    /// 是否启用工具/命令风险检测
    pub scan_tools: bool,
    /// 是否启用网络风险检测
    pub scan_network: bool,
    /// 是否强制脱敏敏感信息
    pub redact_secrets: bool,
    /// Critical 级别是否强制阻断（无视 mode）
    pub block_on_critical: bool,
    /// 单字段最大扫描字节数（性能保护）
    pub max_scan_bytes: usize,
}

/// 单条风险发现
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SecurityFinding {
    /// 扫描阶段：request 或 response
    pub phase: String,
    /// 风险分类：credential | file | unicode | network | tool | prompt
    pub category: String,
    /// 规则 ID，如 credential.secret_token
    pub rule_id: String,
    /// 风险等级
    pub severity: RiskLevel,
    /// 风险标题
    pub title: String,
    /// 风险描述
    pub description: String,
    /// JSON 路径，如 $.messages[0].content
    pub location: String,
    /// 打码证据，如 sk-abc****wxyz
    pub evidence_masked: String,
}

/// 一次扫描的完整结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SecurityScanResult {
    /// 综合风险等级
    pub risk_level: RiskLevel,
    /// 综合风险评分（0-100）
    pub risk_score: i32,
    /// 处置行为
    pub action: SecurityAction,
    /// 是否已脱敏
    pub sanitized: bool,
    /// 阻断原因（仅当 action=Block 时有值）
    pub blocked_reason: Option<String>,
    /// 扫描摘要
    pub summary: String,
    /// 风险发现列表
    pub findings: Vec<SecurityFinding>,
}

/// 风险等级 → 默认处置行为的映射
impl RiskLevel {
    /// 获取该风险等级对应的默认处置行为
    pub fn default_action(&self) -> Action {
        match self {
            RiskLevel::Info => Action::Allow,
            RiskLevel::Low => Action::Warn,
            RiskLevel::Medium => Action::NeedConfirm,
            RiskLevel::High => Action::Block,
            RiskLevel::Critical => Action::Block,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Info < RiskLevel::Low);
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn test_max() {
        assert_eq!(RiskLevel::Low.max(RiskLevel::High), RiskLevel::High);
        assert_eq!(RiskLevel::Critical.max(RiskLevel::Info), RiskLevel::Critical);
    }

    #[test]
    fn test_default_action() {
        assert_eq!(RiskLevel::Info.default_action(), Action::Allow);
        assert_eq!(RiskLevel::Low.default_action(), Action::Warn);
        assert_eq!(RiskLevel::Medium.default_action(), Action::NeedConfirm);
        assert_eq!(RiskLevel::High.default_action(), Action::Block);
        assert_eq!(RiskLevel::Critical.default_action(), Action::Block);
    }

    #[test]
    fn test_action_flags() {
        assert!(Action::Allow.is_allowed());
        assert!(Action::Warn.is_allowed());
        assert!(Action::Redact.is_allowed());
        assert!(!Action::NeedConfirm.is_allowed());
        assert!(!Action::Block.is_allowed());

        assert!(!Action::Allow.requires_human());
        assert!(!Action::Warn.requires_human());
        assert!(!Action::Redact.requires_human());
        assert!(Action::NeedConfirm.requires_human());
        assert!(Action::Block.requires_human());

        assert!(!Action::Allow.needs_redaction());
        assert!(!Action::Warn.needs_redaction());
        assert!(Action::Redact.needs_redaction());
        assert!(!Action::NeedConfirm.needs_redaction());
        assert!(!Action::Block.needs_redaction());
    }

    #[test]
    fn test_resolve_action_audit() {
        // audit 模式：全部 Allow（block_on_critical 除外）
        assert_eq!(SecurityMode::Audit.resolve_action(&RiskLevel::Info, false), Action::Allow);
        assert_eq!(SecurityMode::Audit.resolve_action(&RiskLevel::Low, false), Action::Allow);
        assert_eq!(SecurityMode::Audit.resolve_action(&RiskLevel::Medium, false), Action::Allow);
        assert_eq!(SecurityMode::Audit.resolve_action(&RiskLevel::High, false), Action::Allow);
        assert_eq!(SecurityMode::Audit.resolve_action(&RiskLevel::Critical, false), Action::Allow);
        // block_on_critical 时 Critical → Block
        assert_eq!(SecurityMode::Audit.resolve_action(&RiskLevel::Critical, true), Action::Block);
    }

    #[test]
    fn test_resolve_action_warn() {
        assert_eq!(SecurityMode::Warn.resolve_action(&RiskLevel::Info, false), Action::Allow);
        assert_eq!(SecurityMode::Warn.resolve_action(&RiskLevel::Low, false), Action::Allow);
        assert_eq!(SecurityMode::Warn.resolve_action(&RiskLevel::Medium, false), Action::Warn);
        assert_eq!(SecurityMode::Warn.resolve_action(&RiskLevel::High, false), Action::Warn);
        assert_eq!(SecurityMode::Warn.resolve_action(&RiskLevel::Critical, false), Action::Warn);
        assert_eq!(SecurityMode::Warn.resolve_action(&RiskLevel::Critical, true), Action::Block);
    }

    #[test]
    fn test_resolve_action_redact() {
        assert_eq!(SecurityMode::Redact.resolve_action(&RiskLevel::Info, false), Action::Allow);
        assert_eq!(SecurityMode::Redact.resolve_action(&RiskLevel::Low, false), Action::Allow);
        assert_eq!(SecurityMode::Redact.resolve_action(&RiskLevel::Medium, false), Action::Allow);
        assert_eq!(SecurityMode::Redact.resolve_action(&RiskLevel::High, false), Action::Redact);
        assert_eq!(SecurityMode::Redact.resolve_action(&RiskLevel::Critical, false), Action::Redact);
        assert_eq!(SecurityMode::Redact.resolve_action(&RiskLevel::Critical, true), Action::Block);
    }

    #[test]
    fn test_resolve_action_block() {
        assert_eq!(SecurityMode::Block.resolve_action(&RiskLevel::Info, false), Action::Allow);
        assert_eq!(SecurityMode::Block.resolve_action(&RiskLevel::Low, false), Action::Allow);
        assert_eq!(SecurityMode::Block.resolve_action(&RiskLevel::Medium, false), Action::Allow);
        assert_eq!(SecurityMode::Block.resolve_action(&RiskLevel::High, false), Action::Block);
        assert_eq!(SecurityMode::Block.resolve_action(&RiskLevel::Critical, false), Action::Block);
        // block 模式本身已经 Block，block_on_critical 不影响
        assert_eq!(SecurityMode::Block.resolve_action(&RiskLevel::Critical, true), Action::Block);
    }

    #[test]
    fn test_resolve_action_confirm() {
        assert_eq!(SecurityMode::Confirm.resolve_action(&RiskLevel::Info, false), Action::Allow);
        assert_eq!(SecurityMode::Confirm.resolve_action(&RiskLevel::Low, false), Action::Allow);
        assert_eq!(SecurityMode::Confirm.resolve_action(&RiskLevel::Medium, false), Action::NeedConfirm);
        assert_eq!(SecurityMode::Confirm.resolve_action(&RiskLevel::High, false), Action::NeedConfirm);
        assert_eq!(SecurityMode::Confirm.resolve_action(&RiskLevel::Critical, false), Action::NeedConfirm);
        assert_eq!(SecurityMode::Confirm.resolve_action(&RiskLevel::Critical, true), Action::Block);
    }

    #[test]
    fn test_security_finding_creation() {
        let finding = SecurityFinding {
            phase: "request".to_string(),
            category: "credential".to_string(),
            rule_id: "credential.secret_token".to_string(),
            severity: RiskLevel::High,
            title: "检测到密钥泄露".to_string(),
            description: "请求体中包含疑似 API 密钥".to_string(),
            location: "$.messages[0].content".to_string(),
            evidence_masked: "sk-abc****wxyz".to_string(),
        };
        assert_eq!(finding.severity, RiskLevel::High);
        assert_eq!(finding.phase, "request");
    }

    #[test]
    fn test_security_scan_result_creation() {
        let result = SecurityScanResult {
            risk_level: RiskLevel::Medium,
            risk_score: 65,
            action: Action::NeedConfirm,
            sanitized: true,
            blocked_reason: None,
            summary: "发现中风险凭证泄露".to_string(),
            findings: vec![],
        };
        assert_eq!(result.risk_level, RiskLevel::Medium);
        assert_eq!(result.action, Action::NeedConfirm);
        assert!(result.action.requires_human());
    }
}
