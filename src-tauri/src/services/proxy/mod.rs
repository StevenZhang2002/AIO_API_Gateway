pub mod auth;
pub mod desensitizer;
pub mod error;
pub mod scanner;

use std::sync::Arc;
use std::time::Instant;

use axum::body::Body;
use axum::http::Response;
use axum::response::IntoResponse;
use sqlx::SqlitePool;

use crate::adaptor::{AdaptorRegistry, ProxyRequest, TokenUsage, UsageReceiver};
use crate::db::models::{ApiKey, Channel};
use crate::db::repository::channel_repo::ChannelRepo;
use crate::db::repository::log_repo::LogRepo;
use crate::dto::log_dto::CreateLogDto;
use crate::services::dispatcher::Dispatcher;

use self::desensitizer::{Desensitizer, NoopDesensitizer};
use self::error::ProxyError;
use self::scanner::{PassThroughScanner, RequestScanner};

/// 代理请求上下文
pub struct ProxyContext {
    /// 请求 ID
    pub request_id: String,
    /// 请求开始时间
    pub started_at: Instant,
    /// 认证通过的 API Key
    pub api_key: ApiKey,
    /// 请求模型（外部名）
    pub model: String,
    /// 请求体（OpenAI 格式 JSON）
    pub body: serde_json::Value,
    /// 是否流式请求
    pub is_stream: bool,
}

/// 代理转发结果
pub enum ProxyResult {
    /// 非流式：状态码 + OpenAI 格式响应体 + Token 用量
    Standard {
        status: u16,
        body: serde_json::Value,
        usage: Option<TokenUsage>,
    },
    /// 流式：已转换为 OpenAI 格式的 SSE Body + 可选的流式用量接收器
    Stream {
        body: Body,
        usage_rx: Option<UsageReceiver>,
    },
}

/// 代理服务
pub struct ProxyService;

impl ProxyService {
    /// 处理代理请求
    ///
    /// 完整流程：
    /// 1. 鉴权
    /// 2. 解析请求（model + body）
    /// 3. 安全扫描请求体
    /// 4. 脱敏处理
    /// 5. 查询启用渠道
    /// 6. 构建故障转移队列
    /// 7. 循环转发（成功则返回，失败则继续下一个）
    pub async fn handle(
        pool: &SqlitePool,
        registry: &Arc<AdaptorRegistry>,
        auth_header: Option<&str>,
        body: serde_json::Value,
    ) -> Result<impl IntoResponse, ProxyError> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let started_at = Instant::now();

        // 1. 鉴权
        let api_key = auth::authenticate(pool, auth_header).await?;

        // 2. 解析请求
        let model = body
            .get("model")
            .and_then(|m| m.as_str())
            .ok_or_else(|| ProxyError::BadRequest("缺少 model 字段".to_string()))?
            .to_string();
        let is_stream = body.get("stream").and_then(|s| s.as_bool()).unwrap_or(false);

        let mut ctx = ProxyContext {
            request_id,
            started_at,
            api_key,
            model,
            body,
            is_stream,
        };

        // 2.5 检查模型权限
        let allowed_models: Vec<String> = serde_json::from_str(&ctx.api_key.allowed_models)
            .unwrap_or_default();
        if !allowed_models.is_empty() && !allowed_models.contains(&ctx.model) {
            Self::log_request(
                pool,
                &ctx,
                None,
                None,
                403,
                0,
                0,
                0,
                Some(&format!("模型 {} 不在允许列表中", ctx.model)),
                false,
                false,
            )
            .await;
            return Err(ProxyError::ModelNotAllowed(ctx.model.clone()));
        }

        // 3. 安全扫描请求体
        let scanner = PassThroughScanner;
        if let Some(reason) = scanner.scan_request(&ctx.body).await {
            Self::log_request(
                pool,
                &ctx,
                None,
                None,
                451,
                0,
                0,
                0,
                Some(&reason),
                false,
                false,
            )
            .await;
            return Err(ProxyError::Blocked(reason));
        }

        // 4. 脱敏处理
        let desensitizer = NoopDesensitizer;
        desensitizer.desensitize(&mut ctx.body).await;

        // 5. 查询启用渠道
        let enabled = ChannelRepo::find_enabled(pool)
            .await
            .map_err(|e| ProxyError::Internal(format!("查询渠道失败: {}", e)))?;

        // 5.5 按 allowed_channels 过滤渠道
        let allowed_channels: Vec<String> = serde_json::from_str(&ctx.api_key.allowed_channels)
            .unwrap_or_default();
        let enabled: Vec<Channel> = if allowed_channels.is_empty() {
            enabled
        } else {
            enabled
                .into_iter()
                .filter(|c| allowed_channels.contains(&c.r#type))
                .collect()
        };

        // 6. 构建故障转移队列
        let queue = Dispatcher::build_queue(&enabled, &ctx.model, &[]);
        if queue.is_empty() {
            Self::log_request(
                pool,
                &ctx,
                None,
                None,
                503,
                0,
                0,
                0,
                Some(&format!("无支持模型 {} 的渠道", ctx.model)),
                false,
                false,
            )
            .await;
            return Err(ProxyError::NoAvailableChannel(ctx.model.clone()));
        }

        // 7. 循环转发
        let max_attempts = queue.len().min(3); // 首版硬编码最大 3 次
        let mut failed_ids: Vec<String> = Vec::new();

        for (attempt, channel) in queue.into_iter().take(max_attempts).enumerate() {
            let is_retry = attempt > 0;

            match Self::forward_to_channel(&ctx, channel, registry).await {
                Ok(mut proxy_result) => {
                    // 响应安全扫描（仅非流式）
                    if let ProxyResult::Standard { ref body, .. } = proxy_result {
                        if let Some(reason) = scanner.scan_response(body).await {
                            Self::log_request(
                                pool,
                                &ctx,
                                Some(channel),
                                None,
                                451,
                                0,
                                0,
                                0,
                                Some(&reason),
                                ctx.is_stream,
                                is_retry,
                            )
                            .await;
                            return Err(ProxyError::Blocked(reason));
                        }
                    }

                    // 提取 token 用量
                    let (status, usage) = match &proxy_result {
                        ProxyResult::Standard { status, usage, .. } => (*status, usage.clone()),
                        ProxyResult::Stream { .. } => (200, None),
                    };

                    // 记录成功日志
                    let (prompt, completion, total) = usage
                        .as_ref()
                        .map(|u| {
                            (
                                u.prompt_tokens as i32,
                                u.completion_tokens as i32,
                                u.total_tokens as i32,
                            )
                        })
                        .unwrap_or((0, 0, 0));

                    Self::log_request(
                        pool,
                        &ctx,
                        Some(channel),
                        None,
                        status as i32,
                        prompt,
                        completion,
                        total,
                        None,
                        ctx.is_stream,
                        is_retry,
                    )
                    .await;

                    // 扣减配额
                    if let Some(ref u) = usage {
                        auth::deduct_quota(pool, &ctx.api_key.id, u.total_tokens as i64).await;
                    }

                    // 流式请求：启动后台任务等待用量并扣减配额
                    if let ProxyResult::Stream { ref mut usage_rx, .. } = proxy_result {
                        if let Some(rx) = usage_rx.take() {
                            let pool = pool.clone();
                            let api_key_id = ctx.api_key.id.clone();
                            tokio::spawn(async move {
                                if let Ok(usage) = rx.await {
                                    auth::deduct_quota(&pool, &api_key_id, usage.total_tokens as i64).await;
                                }
                            });
                        }
                    }

                    // 返回响应
                    return Ok(Self::build_response(proxy_result));
                }
                Err(e) => {
                    // 记录失败日志
                    Self::log_request(
                        pool,
                        &ctx,
                        Some(channel),
                        Some(&e.to_string()),
                        502,
                        0,
                        0,
                        0,
                        Some(&e.to_string()),
                        ctx.is_stream,
                        is_retry,
                    )
                    .await;
                    failed_ids.push(channel.id.clone());
                    // 继续下一个渠道
                }
            }
        }

        // 全部失败
        Err(ProxyError::AllChannelsFailed)
    }

    /// 转发请求到指定渠道
    async fn forward_to_channel(
        ctx: &ProxyContext,
        channel: &Channel,
        registry: &Arc<AdaptorRegistry>,
    ) -> Result<ProxyResult, ProxyError> {
        let config = Dispatcher::to_channel_config(channel);
        let adaptor = registry
            .get(&channel.r#type)
            .ok_or_else(|| ProxyError::UpstreamError(format!("未知渠道类型: {}", channel.r#type)))?;

        let proxy_request = ProxyRequest {
            model: ctx.model.clone(),
            body: ctx.body.clone(),
            stream: ctx.is_stream,
        };

        if ctx.is_stream {
            adaptor
                .forward_stream(&proxy_request, &config)
                .await
                .map(|(body, usage_rx)| ProxyResult::Stream { body, usage_rx })
                .map_err(|e| ProxyError::UpstreamError(e.to_string()))
        } else {
            adaptor
                .forward(&proxy_request, &config)
                .await
                .map(|(status, body, usage)| ProxyResult::Standard { status, body, usage })
                .map_err(|e| ProxyError::UpstreamError(e.to_string()))
        }
    }

    /// 构建 HTTP 响应
    fn build_response(result: ProxyResult) -> Response<Body> {
        match result {
            ProxyResult::Standard { status, body, .. } => {
                let http_status = axum::http::StatusCode::from_u16(status).unwrap_or(axum::http::StatusCode::OK);
                Response::builder()
                    .status(http_status)
                    .header("Content-Type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap()
            }
            ProxyResult::Stream { body, .. } => {
                // 流式响应：已转换为 OpenAI 格式的 SSE
                Response::builder()
                    .status(axum::http::StatusCode::OK)
                    .header("Content-Type", "text/event-stream")
                    .header("Cache-Control", "no-cache")
                    .header("Connection", "keep-alive")
                    .body(body)
                    .unwrap()
            }
        }
    }

    /// 记录请求日志
    async fn log_request(
        pool: &SqlitePool,
        ctx: &ProxyContext,
        channel: Option<&Channel>,
        upstream_model: Option<&str>,
        status_code: i32,
        prompt_tokens: i32,
        completion_tokens: i32,
        total_tokens: i32,
        error_message: Option<&str>,
        is_stream: bool,
        is_retry: bool,
    ) {
        let duration_ms = ctx.started_at.elapsed().as_millis() as i32;
        let dto = CreateLogDto {
            api_key_id: Some(ctx.api_key.id.clone()),
            api_key_name: Some(ctx.api_key.name.clone()),
            channel_id: channel.map(|c| c.id.clone()),
            channel_name: channel.map(|c| c.name.clone()),
            model: ctx.model.clone(),
            upstream_model: upstream_model.map(String::from),
            mode: "chat".to_string(),
            status_code,
            prompt_tokens,
            completion_tokens,
            total_tokens,
            duration_ms,
            error_message: error_message.map(String::from),
            is_stream,
            is_retry,
        };
        let _ = LogRepo::create(pool, dto).await;
    }
}
