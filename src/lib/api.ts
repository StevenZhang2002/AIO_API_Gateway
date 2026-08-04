import type {
  DashboardStats,
  Channel,
  ChannelFormData,
  ChannelTestResult,
  ApiKey,
  ApiKeyFormData,
  RequestLog,
  LogFilter,
  AppSettings,
  UsageOverview,
  ChannelType,
} from "../types";
import { API_KEY_PREFIX } from "./constants";

// ==================== Simulated delay ====================
function delay(ms = 300): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

let uid = () => `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;

// ==================== Dashboard ====================
export async function fetchDashboardStats(): Promise<DashboardStats> {
  await delay();
  return {
    todayRequests: 2847,
    todayTokens: 1823400,
    activeChannels: 5,
    avgLatency: 320,
    totalRequests: 124800,
    totalTokens: 78450000,
  };
}

// ==================== Channels ====================
let channels: Channel[] = [
  {
    id: "ch-1", name: "OpenAI Primary", type: "openai",
    baseUrl: "https://api.openai.com/v1", apiKey: "sk-***",
    models: ["gpt-4o", "gpt-4o-mini"], isActive: true,
    weight: 10, maxRetries: 3, timeout: 30000,
    createdAt: "2026-06-01T08:00:00Z", updatedAt: "2026-07-28T10:00:00Z",
  },
  {
    id: "ch-2", name: "DeepSeek Main", type: "deepseek",
    baseUrl: "https://api.deepseek.com/v1", apiKey: "sk-***",
    models: ["deepseek-chat", "deepseek-reasoner"], isActive: true,
    weight: 8, maxRetries: 2, timeout: 60000,
    createdAt: "2026-06-15T12:00:00Z", updatedAt: "2026-08-01T09:00:00Z",
  },
  {
    id: "ch-3", name: "Claude Enterprise", type: "claude",
    baseUrl: "https://api.anthropic.com/v1", apiKey: "sk-ant-***",
    models: ["claude-3-5-sonnet", "claude-3-opus"], isActive: true,
    weight: 5, maxRetries: 2, timeout: 45000,
    createdAt: "2026-07-01T14:00:00Z", updatedAt: "2026-07-30T16:00:00Z",
  },
  {
    id: "ch-4", name: "Gemini-Pro", type: "gemini",
    baseUrl: "https://generativelanguage.googleapis.com/v1beta", apiKey: "AIza***",
    models: ["gemini-1.5-pro", "gemini-2.0-flash"], isActive: false,
    weight: 3, maxRetries: 1, timeout: 20000,
    createdAt: "2026-07-20T10:00:00Z", updatedAt: "2026-08-02T08:00:00Z",
  },
];

export async function fetchChannels(): Promise<Channel[]> {
  await delay();
  return [...channels];
}

export async function addChannel(data: ChannelFormData): Promise<Channel> {
  await delay(500);
  const now = new Date().toISOString();
  const ch: Channel = { id: uid(), ...data, isActive: true, createdAt: now, updatedAt: now };
  channels.push(ch);
  return ch;
}

export async function updateChannel(id: string, data: Partial<ChannelFormData>): Promise<Channel> {
  await delay(400);
  const idx = channels.findIndex((c) => c.id === id);
  if (idx === -1) throw new Error("Channel not found");
  channels[idx] = { ...channels[idx], ...data, updatedAt: new Date().toISOString() };
  return channels[idx];
}

export async function deleteChannel(id: string): Promise<void> {
  await delay(300);
  channels = channels.filter((c) => c.id !== id);
}

export async function toggleChannel(id: string): Promise<Channel> {
  await delay(200);
  const idx = channels.findIndex((c) => c.id === id);
  if (idx === -1) throw new Error("Channel not found");
  channels[idx] = { ...channels[idx], isActive: !channels[idx].isActive, updatedAt: new Date().toISOString() };
  return channels[idx];
}

export async function testChannel(data: ChannelFormData): Promise<ChannelTestResult> {
  await delay(1500);
  const success = Math.random() > 0.15;
  return {
    success,
    latency: Math.floor(Math.random() * 800) + 100,
    models: success ? data.models : [],
  };
}

// ==================== API Keys ====================
let apiKeys: ApiKey[] = [
  {
    id: "key-1", name: "Default Key", key: "sk-aio-d8f3a2b1c4e5f6a7b8c9d0e1",
    status: "active", quotaLimit: 1000000, quotaUsed: 458200,
    allowedModels: [], allowedChannels: [],
    createdAt: "2026-06-01T08:00:00Z", expiresAt: null,
  },
  {
    id: "key-2", name: "Dev Team Key", key: "sk-aio-1a2b3c4d5e6f7a8b9c0d1e2f",
    status: "active", quotaLimit: 500000, quotaUsed: 128000,
    allowedModels: ["gpt-4o-mini", "deepseek-chat"], allowedChannels: ["ch-1", "ch-2"],
    createdAt: "2026-07-10T09:00:00Z", expiresAt: "2026-12-31T23:59:59Z",
  },
  {
    id: "key-3", name: "Test Key", key: "sk-aio-f1e2d3c4b5a69788796a5b4c",
    status: "disabled", quotaLimit: 10000, quotaUsed: 12000,
    allowedModels: [], allowedChannels: [],
    createdAt: "2026-07-20T14:00:00Z", expiresAt: "2026-08-15T23:59:59Z",
  },
];

function generateApiKey(): string {
  const chars = "abcdef0123456789";
  let suffix = "";
  for (let i = 0; i < 32; i++) suffix += chars[Math.floor(Math.random() * chars.length)];
  return API_KEY_PREFIX + suffix;
}

export async function fetchApiKeys(): Promise<ApiKey[]> {
  await delay();
  return [...apiKeys];
}

export async function createApiKey(data: ApiKeyFormData): Promise<ApiKey> {
  await delay(400);
  const key: ApiKey = {
    id: uid(),
    name: data.name,
    key: generateApiKey(),
    status: "active",
    quotaLimit: data.quotaLimit,
    quotaUsed: 0,
    allowedModels: data.allowedModels,
    allowedChannels: data.allowedChannels,
    createdAt: new Date().toISOString(),
    expiresAt: data.expiresAt || null,
  };
  apiKeys.push(key);
  return key;
}

export async function deleteApiKey(id: string): Promise<void> {
  await delay(300);
  apiKeys = apiKeys.filter((k) => k.id !== id);
}

export async function toggleApiKeyStatus(id: string): Promise<ApiKey> {
  await delay(200);
  const idx = apiKeys.findIndex((k) => k.id === id);
  if (idx === -1) throw new Error("Key not found");
  const current = apiKeys[idx];
  const newStatus = current.status === "active" ? "disabled" : "active";
  apiKeys[idx] = { ...current, status: newStatus };
  return apiKeys[idx];
}

// ==================== Logs ====================
const channelNames = ["OpenAI Primary", "DeepSeek Main", "Claude Enterprise", "Gemini-Pro"];
const channelTypes: ChannelType[] = ["openai", "deepseek", "claude", "gemini"];
const models = ["gpt-4o", "gpt-4o-mini", "deepseek-chat", "claude-3-5-sonnet", "gemini-2.0-flash"];
const apiKeyNames = ["Default Key", "Dev Team Key", "Test Key"];
const paths = ["/v1/chat/completions", "/v1/embeddings", "/v1/completions"];
const methods = ["POST", "POST", "POST"];
const statuses: ("success" | "error" | "pending")[] = ["success", "success", "success", "success", "error", "pending"];

let logs: RequestLog[] = Array.from({ length: 50 }, (_, i) => {
  const status = statuses[Math.floor(Math.random() * statuses.length)];
  const channelIdx = Math.floor(Math.random() * channelNames.length);
  return {
    id: `log-${50 - i}`,
    timestamp: new Date(Date.now() - i * 5 * 60000 - Math.random() * 300000).toISOString(),
    method: methods[Math.floor(Math.random() * methods.length)],
    path: paths[Math.floor(Math.random() * paths.length)],
    status,
    statusCode: status === "success" ? 200 : status === "error" ? [400, 429, 500][Math.floor(Math.random() * 3)] : 102,
    latency: Math.floor(Math.random() * 2000) + 80,
    channelName: channelNames[channelIdx],
    channelType: channelTypes[channelIdx],
    model: models[Math.floor(Math.random() * models.length)],
    apiKeyName: apiKeyNames[Math.floor(Math.random() * apiKeyNames.length)],
    tokenUsage: Math.floor(Math.random() * 8000) + 100,
    requestBody: JSON.stringify({ model: "gpt-4o", messages: [{ role: "user", content: "Hello" }] }, null, 2),
    responseBody: status === "success"
      ? JSON.stringify({ choices: [{ message: { content: "Hi there!" } }], usage: { total_tokens: 150 } }, null, 2)
      : status === "error" ? "" : "",
    errorMessage: status === "error" ? ["Rate limit exceeded", "Invalid API key", "Model not found"][Math.floor(Math.random() * 3)] : undefined,
  };
});

export async function fetchLogs(filter?: LogFilter): Promise<RequestLog[]> {
  await delay(200);
  let result = [...logs];
  if (filter) {
    if (filter.keyword) {
      const kw = filter.keyword.toLowerCase();
      result = result.filter(
        (l) => l.path.toLowerCase().includes(kw) || l.model.toLowerCase().includes(kw) || l.apiKeyName.toLowerCase().includes(kw),
      );
    }
    if (filter.channelType) {
      result = result.filter((l) => l.channelType === filter.channelType);
    }
    if (filter.model) {
      result = result.filter((l) => l.model === filter.model);
    }
    if (filter.status) {
      result = result.filter((l) => l.status === filter.status);
    }
    if (filter.dateRange) {
      const [from, to] = filter.dateRange;
      result = result.filter((l) => l.timestamp >= from && l.timestamp <= to);
    }
  }
  return result;
}

// ==================== Usage ====================
export async function fetchUsageOverview(): Promise<UsageOverview> {
  await delay(500);
  const days = 30;
  const dailyData = Array.from({ length: days }, (_, i) => {
    const d = new Date();
    d.setDate(d.getDate() - (days - 1 - i));
    return {
      date: d.toISOString().slice(0, 10),
      requests: Math.floor(Math.random() * 3000) + 500,
      tokens: Math.floor(Math.random() * 2000000) + 200000,
    };
  });

  return {
    totalRequests: 124800,
    totalTokens: 78450000,
    dailyData,
    channelUsage: [
      { channelName: "OpenAI Primary", channelType: "openai", requests: 52000, tokens: 32000000, percentage: 41.7 },
      { channelName: "DeepSeek Main", channelType: "deepseek", requests: 38000, tokens: 26000000, percentage: 30.5 },
      { channelName: "Claude Enterprise", channelType: "claude", requests: 22000, tokens: 15000000, percentage: 17.6 },
      { channelName: "Gemini-Pro", channelType: "gemini", requests: 12800, tokens: 5450000, percentage: 10.2 },
    ],
    modelUsage: [
      { model: "gpt-4o", requests: 28000, tokens: 18000000, percentage: 22.4 },
      { model: "gpt-4o-mini", requests: 24000, tokens: 14000000, percentage: 19.2 },
      { model: "deepseek-chat", requests: 20000, tokens: 15000000, percentage: 16.0 },
      { model: "deepseek-reasoner", requests: 18000, tokens: 11000000, percentage: 14.4 },
      { model: "claude-3-5-sonnet", requests: 15000, tokens: 9500000, percentage: 12.0 },
      { model: "gemini-2.0-flash", requests: 10000, tokens: 3500000, percentage: 8.0 },
      { model: "claude-3-opus", requests: 7000, tokens: 5500000, percentage: 5.6 },
      { model: "gemini-1.5-pro", requests: 2800, tokens: 1950000, percentage: 2.4 },
    ],
  };
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
