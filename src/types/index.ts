// ==================== Dashboard ====================
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

// ==================== Channel ====================
export type ChannelType = "openai" | "deepseek" | "claude" | "gemini" | "dashscope" | "custom";

/** 渠道类型默认配置——后端 get_channel_defaults 返回，供创建渠道表单自动填充 */
export interface ChannelDefaults {
  channelType: ChannelType;
  baseUrl: string;
  models: string[];
}

export interface Channel {
  id: string;
  name: string;
  type: ChannelType;
  baseUrl: string;
  apiKey: string;
  models: string; // JSON string
  status: number; // 1=enabled, 0=disabled
  priority: number;
  weight: number;
  config: string; // JSON string
  modelMapping: string; // JSON string
  createdAt: string;
  updatedAt: string;
  lastTestAt?: string;
  lastTestOk?: boolean;
}

export interface ChannelFormData {
  name: string;
  type: ChannelType;
  baseUrl: string;
  apiKey: string;
  models?: string;
  status?: number;
  priority?: number;
  weight?: number;
  config?: string;
  modelMapping?: string;
}

export interface ChannelTestResult {
  success: boolean;
  latency: number; // ms
  models: string[];
}

// ==================== API Key ====================
export interface ApiKey {
  id: string;
  name: string;
  key: string; // sk-aio-*
  status: number; // 1=active, 0=disabled
  quotaLimit: number; // -1 means unlimited
  quotaUsed: number;
  allowedModels: string; // JSON string
  allowedChannels: string; // JSON string
  createdAt: string;
  expiresAt: string | null;
  updatedAt: string;
}

export interface ApiKeyFormData {
  name: string;
  quotaLimit: number;
  allowedModels: string[];
  allowedChannels: string[];
  expiresAt: string | null;
}

// ==================== Log ====================

export interface RequestLog {
  id: string;
  apiKeyId: string | null;
  apiKeyName: string | null;
  channelId: string | null;
  channelName: string | null;
  model: string;
  upstreamModel: string | null;
  mode: string;
  statusCode: number;
  promptTokens: number;
  completionTokens: number;
  totalTokens: number;
  durationMs: number;
  errorMessage: string | null;
  isStream: boolean;
  isRetry: boolean;
  createdAt: string;
  requestBody: string | null;
  responseBody: string | null;
}

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

export interface LogFilter {
  keyword: string;
  channelId: string;
  model: string;
  statusCode: number | "";
  isStream: boolean | "";
  startTime: string | null;
  endTime: string | null;
  page: number;
  pageSize: number;
}

// ==================== Settings ====================
export type RetryStrategy = "fixed" | "exponential";

export interface ServiceConfig {
  port: number;
  host: string;
}

export interface GeneralSettings {
  siteName: string;
  logRetentionDays: number;
  maxConcurrentRequests: number;
  defaultTimeout: number;
}

export interface UISettings {
  language: string;
  theme: "light" | "dark" | "system";
  sidebarCollapsed: boolean;
}

export interface RetryPolicy {
  strategy: RetryStrategy;
  maxRetries: number;
  baseDelay: number;
  maxDelay: number;
}

export type SecurityMode = "audit" | "warn" | "redact" | "confirm" | "block";

export interface SecuritySettings {
  enabled: boolean;
  mode: SecurityMode;
  scanRequest: boolean;
  scanResponse: boolean;
  scanUnicode: boolean;
  scanTools: boolean;
  scanNetwork: boolean;
  redactSecrets: boolean;
  blockOnCritical: boolean;
  maxScanBytes: number;
}

export interface AppSettings {
  service: ServiceConfig;
  general: GeneralSettings;
  ui: UISettings;
  retry: RetryPolicy;
  security: SecuritySettings;
}

// ==================== Usage ====================
export interface UsageDataPoint {
  date: string;
  requests: number;
  tokens: number;
}

export interface ChannelUsage {
  channelName: string | null;
  channelType: string;
  requests: number;
  tokens: number;
  percentage: number;
}

export interface ModelUsage {
  model: string;
  requests: number;
  tokens: number;
  percentage: number;
}

export interface UsageOverview {
  totalRequests: number;
  totalTokens: number;
  dailyData: UsageDataPoint[];
  channelUsage: ChannelUsage[];
  modelUsage: ModelUsage[];
}

// ==================== Navigation ====================
export interface NavItem {
  label: string;
  path: string;
  icon: string; // lucide icon name
}
