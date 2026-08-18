import type { NavItem, ChannelType } from "../types";

export const NAV_ITEMS: NavItem[] = [
  { label: "仪表盘", path: "/", icon: "LayoutDashboard" },
  { label: "用量统计", path: "/usage", icon: "BarChart3" },
  { label: "渠道管理", path: "/channels", icon: "Route" },
  { label: "密钥管理", path: "/keys", icon: "Key" },
  { label: "请求日志", path: "/logs", icon: "ScrollText" },
  // 设置中心暂时隐藏，恢复时取消注释
  // { label: "设置中心", path: "/settings", icon: "Settings" },
];

export const CHANNEL_TYPES: { value: ChannelType; label: string }[] = [
  { value: "openai", label: "OpenAI" },
  { value: "deepseek", label: "DeepSeek" },
  { value: "dashscope", label: "阿里云百炼" },
  { value: "claude", label: "Claude" },
  { value: "gemini", label: "Gemini" },
  { value: "custom", label: "自定义" },
];

export const CHANNEL_DEFAULTS: Record<ChannelType, { baseUrl: string; models: string[] }> = {
  openai: {
    baseUrl: "https://api.openai.com/v1",
    models: ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-3.5-turbo"],
  },
  deepseek: {
    baseUrl: "https://api.deepseek.com/v1",
    models: ["deepseek-v4-flash", "deepseek-v4-pro"],
  },
  dashscope: {
    baseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    models: ["qwen3.8-max", "qwen3.7-plus", "qwen3.7-flash", "qwen3.5-omni-plus"],
  },
  claude: {
    baseUrl: "https://api.anthropic.com/v1",
    models: ["claude-3-5-sonnet", "claude-3-opus", "claude-3-haiku"],
  },
  gemini: {
    baseUrl: "https://generativelanguage.googleapis.com/v1beta",
    models: ["gemini-1.5-pro", "gemini-1.5-flash", "gemini-2.0-flash"],
  },
  custom: {
    baseUrl: "",
    models: [],
  },
};

export const ALL_MODELS: string[] = Object.values(CHANNEL_DEFAULTS).flatMap((d) => d.models);

export const API_KEY_PREFIX = "sk-aio-";

export const RETRY_STRATEGIES: { value: string; label: string }[] = [
  { value: "fixed", label: "固定间隔" },
  { value: "exponential", label: "指数退避" },
];
