import type {
  DashboardResponse,
  Channel,
  ChannelDefaults,
  ChannelFormData,
  ChannelTestResult,
  ApiKey,
  ApiKeyFormData,
  RequestLog,
  LogFilter,
  PaginatedResult,
  AppSettings,
  UsageOverview,
} from "../types";
import { invoke } from "@tauri-apps/api/core";

// ==================== Simulated delay ====================
function delay(ms = 300): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

// ==================== Dashboard ====================
export async function fetchDashboardStats(): Promise<DashboardResponse> {
  return invoke<DashboardResponse>("get_dashboard_stats");
}

// ==================== Channels ====================
export async function fetchChannels(): Promise<Channel[]> {
  return invoke<Channel[]>("get_all_channels");
}

export async function addChannel(data: ChannelFormData): Promise<Channel> {
  return invoke<Channel>("create_channel", { dto: data });
}

export async function updateChannel(id: string, data: Partial<ChannelFormData>): Promise<Channel> {
  return invoke<Channel>("update_channel", { dto: { id, ...data } });
}

export async function deleteChannel(id: string): Promise<void> {
  return invoke("delete_channel", { id });
}

export async function toggleChannel(id: string, currentStatus: number): Promise<Channel> {
  const newStatus = currentStatus === 1 ? 0 : 1;
  return invoke<Channel>("update_channel", { dto: { id, status: newStatus } });
}

/** 各渠道类型的默认配置（Base URL + 可选模型列表），供创建渠道表单自动填充 */
export async function fetchChannelDefaults(): Promise<ChannelDefaults[]> {
  return invoke<ChannelDefaults[]>("get_channel_defaults");
}

export async function testChannel(data: ChannelFormData): Promise<ChannelTestResult> {
  const result = await invoke<{ success: boolean; message: string; latency_ms: number; models: string[] }>(
    "test_channel",
    { dto: data },
  );
  return {
    success: result.success,
    latency: result.latency_ms,
    models: result.models,
  };
}

// ==================== API Keys ====================
export async function fetchApiKeys(): Promise<ApiKey[]> {
  return invoke<ApiKey[]>("get_all_api_keys");
}

export async function createApiKey(data: ApiKeyFormData): Promise<ApiKey> {
  return invoke<ApiKey>("create_api_key", {
    dto: {
      name: data.name,
      status: 1,
      allowed_models: JSON.stringify(data.allowedModels),
      allowed_channels: JSON.stringify(data.allowedChannels),
      quota_limit: data.quotaLimit,
      expires_at: data.expiresAt,
    },
  });
}

export async function deleteApiKey(id: string): Promise<void> {
  return invoke("delete_api_key", { id });
}

export async function toggleApiKeyStatus(id: string, currentStatus: number): Promise<ApiKey> {
  const newStatus = currentStatus === 1 ? 0 : 1;
  return invoke<ApiKey>("update_api_key", { dto: { id, status: newStatus } });
}

// ==================== Logs ====================
export async function fetchLogs(filter: LogFilter): Promise<PaginatedResult<RequestLog>> {
  return invoke<PaginatedResult<RequestLog>>("search_logs", {
    dto: {
      keyword: filter.keyword || null,
      channel_id: filter.channelId || null,
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

// ==================== Usage ====================
export async function fetchUsageOverview(): Promise<UsageOverview> {
  return invoke<UsageOverview>("get_usage_overview");
}

// ==================== Settings ====================
let settings: AppSettings = {
  service: { port: 8080, host: "0.0.0.0" },
  general: {
    siteName: "AIO Gateway",
    logRetentionDays: 30,
    maxConcurrentRequests: 100,
    defaultTimeout: 30000,
  },
  ui: {
    language: "zh-CN",
    theme: "system",
    sidebarCollapsed: false,
  },
  retry: {
    strategy: "exponential",
    maxRetries: 3,
    baseDelay: 1000,
    maxDelay: 30000,
  },
  security: {
    enabled: false,
    mode: "audit",
    scanRequest: true,
    scanResponse: false,
    scanUnicode: true,
    scanTools: true,
    scanNetwork: true,
    redactSecrets: true,
    blockOnCritical: true,
    maxScanBytes: 65536,
  },
};

export async function fetchSettings(): Promise<AppSettings> {
  await delay(300);
  return { ...settings };
}

export async function updateSettings(data: Partial<AppSettings>): Promise<AppSettings> {
  await delay(400);
  settings = { ...settings, ...data };
  return { ...settings };
}
