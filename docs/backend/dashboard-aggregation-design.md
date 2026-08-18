# 仪表盘聚合数据对接设计

## 现状分析

| 层 | 现状 |
|---|---|
| **数据库** | `request_logs` 表已存储完整的请求记录（status_code, total_tokens, duration_ms, created_at 等） |
| **Repository** | `LogRepo` 仅有 `create` 和 `search` 方法，缺少聚合查询 |
| **Tauri Command** | ❌ 缺失——未暴露仪表盘聚合查询命令 |
| **前端 api.ts** | `fetchDashboardStats()` 返回硬编码 mock 数据 |
| **前端 DashboardPage** | 展示 mock 数据，包含 4 个统计卡片 + 2 个累计指标 + 服务健康状态 + 最近活动 |

## 需要实现的聚合指标

### 核心统计卡片（4 个）
| 指标 | 前端字段 | 数据来源 | SQL 逻辑 |
|------|---------|---------|---------|
| 今日请求数 | todayRequests | request_logs | `COUNT(*) WHERE created_at >= 今日零点` |
| 今日 Token 消耗 | todayTokens | request_logs | `SUM(total_tokens) WHERE created_at >= 今日零点` |
| 活跃渠道数 | activeChannels | channels | `COUNT(*) WHERE status = 1` |
| 平均延迟 | avgLatency | request_logs | `AVG(duration_ms) WHERE created_at >= 今日零点` |

### 累计指标（2 个）
| 指标 | 前端字段 | 数据来源 | SQL 逻辑 |
|------|---------|---------|---------|
| 累计请求数 | totalRequests | request_logs | `COUNT(*)` |
| 累计 Token 消耗 | totalTokens | request_logs | `SUM(total_tokens)` |

### 最近活动（5 条）
| 指标 | 数据来源 | SQL 逻辑 |
|------|---------|---------|
| 最近请求日志 | request_logs | `SELECT * ORDER BY created_at DESC LIMIT 5` |

### 服务健康状态（可选，暂用 mock）
| 指标 | 说明 |
|------|------|
| CPU 使用率 | 需要系统级 API，暂用 mock |
| 内存使用 | 需要系统级 API，暂用 mock |
| 运行时间 | 可从服务启动时间计算 |
| 错误率 | 可从 request_logs 计算：`COUNT(status_code >= 400) / COUNT(*) * 100` |

## 设计方案

### 1. 新增 DTO

**文件：`src-tauri/src/dto/dashboard_dto.rs`**

```rust
use serde::{Deserialize, Serialize};

/// 仪表盘统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    /// 今日请求数
    pub today_requests: i64,
    /// 今日 Token 消耗
    pub today_tokens: i64,
    /// 活跃渠道数
    pub active_channels: i64,
    /// 平均延迟（ms）
    pub avg_latency: i64,
    /// 累计请求数
    pub total_requests: i64,
    /// 累计 Token 消耗
    pub total_tokens: i64,
    /// 错误率（百分比）
    pub error_rate: f64,
}

/// 最近活动项（简化版日志）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentActivity {
    pub id: String,
    pub model: String,
    pub channel_name: Option<String>,
    pub status_code: i32,
    pub total_tokens: i32,
    pub duration_ms: i32,
    pub created_at: String,
}

/// 仪表盘完整响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardResponse {
    pub stats: DashboardStats,
    pub recent_activities: Vec<RecentActivity>,
}
```

### 2. 新增 Repository 方法

**文件：`src-tauri/src/db/repository/log_repo.rs`**

```rust
impl LogRepo {
    /// 获取仪表盘统计数据
    pub async fn get_dashboard_stats(pool: &SqlitePool) -> Result<DashboardStats, sqlx::Error> {
        let today_start = chrono::Local::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .to_string();

        // 今日统计
        let (today_requests, today_tokens, avg_latency): (i64, i64, f64) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(*) as cnt,
                COALESCE(SUM(total_tokens), 0) as tokens,
                COALESCE(AVG(duration_ms), 0.0) as avg_ms
            FROM request_logs 
            WHERE created_at >= ?
            "#,
        )
        .bind(&today_start)
        .fetch_one(pool)
        .await?;

        // 累计统计
        let (total_requests, total_tokens): (i64, i64) = sqlx::query_as(
            "SELECT COUNT(*), COALESCE(SUM(total_tokens), 0) FROM request_logs",
        )
        .fetch_one(pool)
        .await?;

        // 错误率
        let (total_count, error_count): (i64, i64) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(*),
                COUNT(CASE WHEN status_code >= 400 THEN 1 END)
            FROM request_logs
            "#,
        )
        .fetch_one(pool)
        .await?;
        let error_rate = if total_count > 0 {
            (error_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };

        Ok(DashboardStats {
            today_requests,
            today_tokens,
            active_channels: 0, // 由 Service 层填充
            avg_latency: avg_latency as i64,
            total_requests,
            total_tokens,
            error_rate,
        })
    }

    /// 获取最近活动（最近 N 条日志摘要）
    pub async fn get_recent_activities(
        pool: &SqlitePool,
        limit: i64,
    ) -> Result<Vec<RecentActivity>, sqlx::Error> {
        sqlx::query_as::<_, RecentActivity>(
            r#"
            SELECT id, model, channel_name, status_code, total_tokens, duration_ms, created_at
            FROM request_logs
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}
```

### 3. 新增 Service 方法

**文件：`src-tauri/src/services/dashboard_service.rs`**（新文件）

```rust
use crate::db::repository::channel_repo::ChannelRepo;
use crate::db::repository::log_repo::LogRepo;
use crate::dto::dashboard_dto::{DashboardResponse, DashboardStats};
use sqlx::SqlitePool;

pub struct DashboardService;

impl DashboardService {
    /// 获取仪表盘完整数据
    pub async fn get_dashboard(pool: &SqlitePool) -> Result<DashboardResponse, sqlx::Error> {
        let mut stats = LogRepo::get_dashboard_stats(pool).await?;
        
        // 填充活跃渠道数
        let active_channels = ChannelRepo::count_enabled(pool).await?;
        stats.active_channels = active_channels;

        // 获取最近活动
        let recent_activities = LogRepo::get_recent_activities(pool, 5).await?;

        Ok(DashboardResponse {
            stats,
            recent_activities,
        })
    }
}
```

### 4. 新增 ChannelRepo 方法

**文件：`src-tauri/src/db/repository/channel_repo.rs`**

```rust
impl ChannelRepo {
    /// 统计已启用的渠道数量
    pub async fn count_enabled(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COUNT(*) FROM channels WHERE status = 1")
            .fetch_one(pool)
            .await
    }
}
```

### 5. 新增 Tauri Command

**文件：`src-tauri/src/lib.rs`**

```rust
#[tauri::command]
async fn get_dashboard_stats(pool: State<'_, SqlitePool>) -> Result<DashboardResponse, String> {
    DashboardService::get_dashboard(&*pool)
        .await
        .map_err(|e| e.to_string())
}
```

注册到 `invoke_handler`。

### 6. 前端类型对齐

**文件：`src/types/index.ts`**

```typescript
export interface DashboardStats {
  todayRequests: number;
  todayTokens: number;
  activeChannels: number;
  avgLatency: number;
  totalRequests: number;
  totalTokens: number;
  errorRate: number;
}

export interface RecentActivity {
  id: string;
  model: string;
  channelName: string | null;
  statusCode: number;
  totalTokens: number;
  durationMs: number;
  createdAt: string;
}

export interface DashboardResponse {
  stats: DashboardStats;
  recentActivities: RecentActivity[];
}
```

### 7. 前端 api.ts 替换 mock

**文件：`src/lib/api.ts`**

```typescript
export async function fetchDashboardStats(): Promise<DashboardResponse> {
  return invoke<DashboardResponse>("get_dashboard_stats");
}
```

### 8. 前端 DashboardPage 适配

主要改动：
1. `stats` 从 `DashboardResponse.stats` 解构
2. `recentActivity` 从 `DashboardResponse.recentActivities` 映射
3. 错误率展示替换 mock 的 CPU/内存（或保留 mock 但标注）
4. 最近活动格式化：`{channelName} 渠道通过 {model} 完成请求`

## 改动文件清单

| 文件 | 改动 |
|------|------|
| `src-tauri/src/dto/dashboard_dto.rs` | 新建：DashboardStats, RecentActivity, DashboardResponse |
| `src-tauri/src/dto/mod.rs` | 添加 `pub mod dashboard_dto;` |
| `src-tauri/src/db/repository/log_repo.rs` | 新增 `get_dashboard_stats`, `get_recent_activities` |
| `src-tauri/src/db/repository/channel_repo.rs` | 新增 `count_enabled` |
| `src-tauri/src/services/dashboard_service.rs` | 新建：DashboardService |
| `src-tauri/src/services/mod.rs` | 添加 `pub mod dashboard_service;` |
| `src-tauri/src/lib.rs` | 新增 `get_dashboard_stats` Tauri command + 注册 |
| `src/types/index.ts` | 更新 DashboardStats，新增 RecentActivity, DashboardResponse |
| `src/lib/api.ts` | `fetchDashboardStats` 改为 invoke 真实接口 |
| `src/pages/DashboardPage.tsx` | 适配真实数据结构 + 最近活动动态渲染 |

## 不做的事

- 不做 CPU/内存监控（需要系统级依赖，非核心需求）
- 不做运行时间统计（可从服务启动时间计算，但暂不实现）
- 不做 Sparkline 真实数据（需要历史趋势数据，当前仅聚合统计）
- 不做自动刷新（用户手动刷新页面即可）

## SQL 性能考虑

- `created_at` 字段已有索引 `idx_logs_created`，今日统计查询高效
- 累计统计需要全表扫描，但 `request_logs` 数据量通常不会太大（万级以内）
- 如果未来数据量增长，可考虑增加定时聚合表或物化视图
