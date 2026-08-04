// ==================== Dashboard ====================
export interface DashboardStats {
  todayRequests: number;
  todayTokens: number;
  activeChannels: number;
  avgLatency: number; // ms
  totalRequests: number;
  totalTokens: number;
}

// ==================== Channel ====================
export type ChannelType = "openai" | "deepseek" | "claude" | "gemini";

export interface Channel {
  id: string;
  name: string;
  type: ChannelType;
  baseUrl: string;
  apiKey: string;
  models: string[];
  isActive: boolean;
  weight: number;
  maxRetries: number;
  timeout: number; // ms
  createdAt: string;
  updatedAt: string;
}

export interface ChannelFormData {
  name: string;
  type: ChannelType;
  baseUrl: string;
  apiKey: string;
  models: string[];
  weight: number;
  maxRetries: number;
  timeout: number;
}

export interface ChannelTestResult {
  success: boolean;
  latency: number; // ms
  models: string[];
}

// ==================== API Key ====================
export type ApiKeyStatus = "active" | "disabled" | "expired";

export interface ApiKey {
  id: string;
  name: string;
  key: string; // sk-aio-*
  status: ApiKeyStatus;
  quotaLimit: number; // -1 means unlimited
  quotaUsed: number;
  allowedModels: string[];
  allowedChannels: string[];
  createdAt: string;
  expiresAt: string | null;
}

export interface ApiKeyFormData {
  name: string;
  quotaLimit: number;
  allowedModels: string[];
  allowedChannels: string[];
  expiresAt: string | null;
}

// ==================== Log ====================
export type LogStatus = "success" | "error" | "pending";

export interface RequestLog {
  id: string;
  timestamp: string;
  method: string;
  path: string;
  status: LogStatus;
  statusCode: number;
  latency: number; // ms
  channelName: string;
  channelType: ChannelType;
  model: string;
  apiKeyName: string;
  tokenUsage: number;
  requestBody: string;
  responseBody: string;
  errorMessage?: string;
}

export interface LogFilter {
  keyword: string;
  channelType: ChannelType | "";
  model: string;
  status: LogStatus | "";
  dateRange: [string, string] | null;
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

export interface AppSettings {
  service: ServiceConfig;
  general: GeneralSettings;
  ui: UISettings;
  retry: RetryPolicy;
}

// ==================== Usage ====================
export interface UsageDataPoint {
  date: string;
  requests: number;
  tokens: number;
}

export interface ChannelUsage {
  channelName: string;
  channelType: ChannelType;
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
