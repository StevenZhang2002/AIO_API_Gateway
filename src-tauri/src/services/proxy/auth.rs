use sqlx::SqlitePool;

use crate::db::models::ApiKey;
use crate::db::repository::api_key_repo::ApiKeyRepo;

use super::error::ProxyError;

/// 从请求头提取并验证 API Key
///
/// 流程：
/// 1. 从 Authorization header 提取 Bearer token
/// 2. 查询 api_keys 表验证 key 是否存在且启用
/// 3. 检查是否过期
/// 4. 检查配额是否用尽
pub async fn authenticate(
    pool: &SqlitePool,
    auth_header: Option<&str>,
) -> Result<ApiKey, ProxyError> {
    // 1. 提取 Bearer token
    let key = extract_bearer_token(auth_header).ok_or(ProxyError::MissingApiKey)?;

    // 2. 查询已启用的 API Key
    let api_key = ApiKeyRepo::find_by_key(pool, key)
        .await
        .map_err(|e| ProxyError::Internal(format!("数据库查询失败: {}", e)))?
        .ok_or(ProxyError::InvalidApiKey)?;

    // 3. 检查过期时间
    if let Some(ref expires_at) = api_key.expires_at {
        let expires = chrono::DateTime::parse_from_rfc3339(expires_at)
            .map(|dt| dt.with_timezone(&chrono::Utc));
        if let Ok(expires) = expires {
            if chrono::Utc::now() > expires {
                return Err(ProxyError::ExpiredApiKey);
            }
        }
    }

    // 4. 检查配额
    if api_key.quota_limit >= 0 && api_key.quota_used >= api_key.quota_limit {
        return Err(ProxyError::QuotaExhausted);
    }

    Ok(api_key)
}

/// 从 Authorization header 提取 Bearer token
fn extract_bearer_token(header: Option<&str>) -> Option<&str> {
    let header = header?;
    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() == 2 && parts[0].eq_ignore_ascii_case("Bearer") {
        Some(parts[1])
    } else {
        None
    }
}

/// 原子扣减配额（仅成功转发后调用）
pub async fn deduct_quota(pool: &SqlitePool, api_key_id: &str, tokens: i64) {
    if tokens <= 0 {
        return;
    }
    let _ = sqlx::query("UPDATE api_keys SET quota_used = quota_used + ? WHERE id = ?")
        .bind(tokens)
        .bind(api_key_id)
        .execute(pool)
        .await;
}
