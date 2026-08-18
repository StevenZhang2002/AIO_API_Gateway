# 请求日志前端对接后端设计

## 现状分析

| 层 | 现状 |
|---|---|
| **数据库** | `request_logs` 表已建，含完整字段（api_key_id/name, channel_id/name, model, upstream_model, status_code, tokens, duration_ms, error_message, is_stream, is_retry 等） |
| **Repository** | `LogRepo::create` 已实现（ProxyService 每次转发后调用写入）；`LogRepo::search` 已实现（支持多条件动态查询 + 分页） |
| **DTO** | `SearchLogDto` 已定义（api_key_id, channel_id, model, status_code, is_stream, is_retry, start_time, end_time, page, page_size）；`PaginatedResult<T>` 已定义 |
| **Tauri Command** | ❌ 缺失——未暴露 `search_logs` 命令 |
| **前端 api.ts** | `fetchLogs()` 返回 50 条随机 mock 数据，字段与后端 `RequestLog` model 不匹配 |
| **前端 LogsPage** | 展示 mock 数据，有 keyword / channelType / model / status / dateRange 筛选，无分页 |

## 核心问题：前后端字段映射

### 前端当前 RequestLog 类型（mock 字段）
```
id, timestamp, method, path, status, statusCode, latency,
channelName, channelType, model, apiKeyName, tokenUsage,
requestBody, responseBody, errorMessage
```

### 后端 RequestLog model（真实字段）
```
id, apiKeyId, apiKeyName, channelId, channelName,
model, upstreamModel, mode, statusCode,
promptTokens, completionTokens, totalTokens,
durationMs, errorMessage, isStream, isRetry, createdAt
```

### 映射关系
| 前端字段 | 后端字段 | 转换说明 |
|---------|---------|---------|
| timestamp | createdAt | 直接映射 |
| statusCode | statusCode | 直接映射 |
| latency | durationMs | 直接映射 |
| channelName | channelName | 直接映射 |
| channelType | — | 后端无此字段，需前端通过 channelId 查渠道表 or 后端返回时附带 |
| model | model | 直接映射 |
| apiKeyName | apiKeyName | 直接映射 |
| tokenUsage | totalTokens | 直接映射 |
| status | statusCode | 前端根据 statusCode 推导：2xx=success, 其他=error |
| method | — | 后端无此字段，当前仅 POST /v1/chat/completions，可前端硬编码 |
| path | — | 后端无此字段，同上可硬编码 |
| requestBody / responseBody | — | 后端未存储请求/响应体，详情弹窗中移除 |

## 改动方案

### 1. 后端：新增 Tauri Command

**文件：`lib.rs`**

```rust
#[tauri::command]
async fn search_logs(
    pool: State<'_, SqlitePool>,
    dto: SearchLogDto,
) -> Result<PaginatedResult<RequestLog>, String> {
    LogRepo::search(&*pool, dto)
        .await
        .map_err(|e| e.to_string())
}
```

注册到 `invoke_handler`。

### 2. 后端：SearchLogDto 增加 keyword 模糊搜索

当前 `SearchLogDto` 没有 keyword 字段，前端有搜索框。两种方案：

- **方案 A（推荐）**：后端 `SearchLogDto` 新增 `keyword: Option<String>`，`LogRepo::search` 中增加对 model / api_key_name / channel_name 的 LIKE 模糊匹配
- **方案 B**：前端拆 keyword 为具体字段传给后端（体验差，不推荐）

选方案 A。

### 3. 前端：类型对齐

**`types/index.ts` 修改 `RequestLog`：**

```typescript
export interface RequestLog {
  id: string;
  apiKeyId: string | null;
  apiKeyName: string | null;
  channelId: string | null;
  channelName: string | null;
  model: string;
  upstreamModel: string | null;
  mode: string;          // "chat"
  statusCode: number;
  promptTokens: number;
  completionTokens: number;
  totalTokens: number;
  durationMs: number;
  errorMessage: string | null;
  isStream: boolean;
  isRetry: boolean;
  createdAt: string;
}
```

**新增分页结果类型：**

```typescript
export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}
```

**`LogFilter` 调整：**

```typescript
export interface LogFilter {
  keyword: string;        // → 后端 keyword
  model: string;          // → 后端 model
  statusCode: number | ""; // → 后端 status_code
  isStream: boolean | "";  // → 后端 is_stream
  startTime: string | null; // → 后端 start_time
  endTime: string | null;   // → 后端 end_time
  page: number;
  pageSize: number;
}
```

> 移除 `channelType` 和 `status`（前端 mock 概念），改为与后端对齐的字段。

### 4. 前端：api.ts 替换 mock

```typescript
export async function fetchLogs(filter: LogFilter): Promise<PaginatedResult<RequestLog>> {
  return invoke<PaginatedResult<RequestLog>>("search_logs", {
    dto: {
      keyword: filter.keyword || null,
      model: filter.model || null,
      status_code: filter.statusCode || null,
      is_stream: filter.isStream === "" ? null : filter.isStream,
      start_time: filter.startTime,
      end_time: filter.endTime,
      page: filter.page,
      page_size: filter.pageSize,
    },
  });
}
```

### 5. 前端：LogsPage 适配

主要改动：
1. **状态推导**：`status` 由 `statusCode` 推导（2xx → success，其他 → error），不再有 pending
2. **移除 channelType 筛选**：后端按 channel_id 过滤，前端改为下拉显示渠道名称列表（从 channels API 获取）
3. **新增分页组件**：底部分页栏，显示总数/页码
4. **详情弹窗精简**：移除 requestBody / responseBody（后端未存储），增加 isStream / isRetry / upstreamModel 等信息展示
5. **Token 列**：显示 `totalTokens`（流式为 0 时显示 "—"）
6. **延迟列**：使用 `durationMs`

### 6. 前端渠道筛选改造

当前按 `channelType`（openai/claude 等）筛选，后端按 `channel_id` 筛选。

方案：页面加载时调用 `fetchChannels()` 获取渠道列表，筛选下拉框显示渠道名称，选中后传 `channel_id` 给后端。

## 改动文件清单

| 文件 | 改动 |
|------|------|
| `src-tauri/src/dto/log_dto.rs` | `SearchLogDto` 增加 `keyword: Option<String>` |
| `src-tauri/src/db/repository/log_repo.rs` | `search` 方法增加 keyword 模糊匹配逻辑 |
| `src-tauri/src/lib.rs` | 新增 `search_logs` Tauri command + 注册 |
| `src/types/index.ts` | `RequestLog` 对齐后端字段；新增 `PaginatedResult`；调整 `LogFilter` |
| `src/lib/api.ts` | `fetchLogs` 改为 invoke 真实接口 |
| `src/pages/LogsPage.tsx` | 适配真实数据结构 + 分页 + 渠道筛选改造 |

## 不做的事

- 不存储请求/响应体（体积大，非核心需求）
- 不做日志导出功能
- 不做日志自动清理（保留天数设置属于 Settings 持久化范畴）
