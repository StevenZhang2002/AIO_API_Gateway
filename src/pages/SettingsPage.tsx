import { useEffect, useState } from "react";
import { Save, Server, Globe, Monitor, RefreshCw, Check, Shield } from "lucide-react";
import { fetchSettings, updateSettings } from "../lib/api";
import { RETRY_STRATEGIES } from "../lib/constants";
import type { AppSettings, RetryStrategy } from "../types";

type TabKey = "service" | "general" | "ui" | "retry";

const tabs: { key: TabKey; label: string; icon: React.FC<{ size?: number }>; desc: string }[] = [
  { key: "service", label: "服务配置", icon: Server, desc: "网关监听地址与端口" },
  { key: "general", label: "通用设置", icon: Globe, desc: "常规运行参数" },
  { key: "ui", label: "界面设置", icon: Monitor, desc: "语言、主题与布局" },
  { key: "retry", label: "重试策略", icon: RefreshCw, desc: "请求失败重试配置" },
];

const themeLabels: Record<string, string> = {
  light: "浅色",
  dark: "深色",
  system: "跟随系统",
};

export default function SettingsPage() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [activeTab, setActiveTab] = useState<TabKey>("service");

  useEffect(() => {
    fetchSettings().then((data) => {
      setSettings(data);
      setLoading(false);
    });
  }, []);

  async function handleSave() {
    if (!settings) return;
    setSaving(true);
    setSaved(false);
    await updateSettings(settings);
    setSaving(false);
    setSaved(true);
    setTimeout(() => setSaved(false), 2500);
  }

  function update<K extends keyof AppSettings>(section: K, value: Partial<AppSettings[K]>) {
    if (!settings) return;
    setSettings({ ...settings, [section]: { ...settings[section], ...value } });
  }

  if (loading) {
    return (
      <div className="page-wrap">
        <div className="page-header">
          <div>
            <div className="skeleton" style={{ height: 32, width: 128, marginBottom: 8 }} />
            <div className="skeleton" style={{ height: 16, width: 192 }} />
          </div>
        </div>
        <div className="flex gap-5">
          <div style={{ width: 200 }}>
            {Array.from({ length: 4 }).map((_, i) => (
              <div key={i} className="skeleton" style={{ height: 40, width: "100%", borderRadius: 8, marginBottom: 8 }} />
            ))}
          </div>
          <div className="card card-body" style={{ flex: 1 }}>
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="skeleton" style={{ height: 64, width: "100%", borderRadius: 8, marginBottom: 16 }} />
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="page-wrap">
      {/* Page Header */}
      <div className="page-header">
        <div>
          <h1>设置中心</h1>
          <p>配置网关服务参数与偏好</p>
        </div>
        <button
          onClick={handleSave}
          disabled={saving}
          className={`btn btn-primary ${saved ? "btn-success" : ""}`}
          style={{ transition: "all 0.3s" }}
        >
          {saved ? (
            <>
              <Check size={17} className="animate-scale-in" aria-hidden="true" />
              已保存
            </>
          ) : saving ? (
            <>
              <div className="spinner spinner-sm spinner-white" />
              保存中…
            </>
          ) : (
            <>
              <Save size={17} aria-hidden="true" />
              保存设置
            </>
          )}
        </button>
      </div>

      {/* Layout: Side tabs + Content */}
      <div className="settings-layout">
        {/* Side Tabs */}
        <div className="settings-tabs">
          <div className="card settings-tab-card">
            {tabs.map(({ key, label, icon: Icon, desc }) => {
              const isActive = activeTab === key;
              return (
                <button
                  key={key}
                  onClick={() => setActiveTab(key)}
                  className={`settings-tab-btn ${isActive ? "active" : ""}`}
                >
                  <div className="tab-icon" aria-hidden="true">
                    <Icon size={17} />
                  </div>
                  <div style={{ minWidth: 0 }}>
                    <p className={`tab-label ${isActive ? "active" : ""}`}>
                      {label}
                    </p>
                    <p className="tab-desc">{desc}</p>
                  </div>
                </button>
              );
            })}
          </div>

          {/* Status indicator */}
          <div className="card card-body" style={{ marginTop: 16, display: "flex", alignItems: "center", gap: 12 }}>
            <div className="notification-dot notification-dot-active" aria-hidden="true" />
            <div>
              <p className="text-13 font-semibold text-ink">运行中</p>
              <p style={{ fontSize: 11, color: "var(--ink-subtle)" }}>
                {settings!.service.host}:{settings!.service.port}
              </p>
            </div>
          </div>
        </div>

        {/* Content */}
        <div className="settings-content">
          <div className="card card-body">
            {activeTab === "service" && (
              <div className="settings-content-inner space-y-5">
                <h3>
                  <Server size={18} style={{ color: "var(--primary)" }} aria-hidden="true" />
                  服务配置
                </h3>
                <p className="subtitle">修改后需重启服务生效</p>

                <div className="space-y-4">
                  <div className="settings-field">
                    <label htmlFor="service-host">监听地址 <span>当前: {settings!.service.host}:{settings!.service.port}</span></label>
                    <input
                      id="service-host"
                      name="host"
                      type="text" value={settings!.service.host}
                      onChange={(e) => update("service", { host: e.target.value })}
                      className="input"
                      placeholder="0.0.0.0"
                    />
                  </div>
                  <div className="settings-field">
                    <label htmlFor="service-port">监听端口</label>
                    <input
                      id="service-port"
                      name="port"
                      type="number" value={settings!.service.port}
                      onChange={(e) => update("service", { port: Number(e.target.value) })}
                      className="input"
                    />
                  </div>
                </div>
              </div>
            )}

            {activeTab === "general" && (
              <div className="settings-content-inner space-y-5">
                <h3>
                  <Globe size={18} style={{ color: "var(--primary)" }} aria-hidden="true" />
                  通用设置
                </h3>

                <div className="space-y-4">
                  <div className="settings-field">
                    <label htmlFor="general-site-name">站点名称</label>
                    <input
                      id="general-site-name"
                      name="siteName"
                      type="text" value={settings!.general.siteName}
                      onChange={(e) => update("general", { siteName: e.target.value })}
                      className="input"
                    />
                  </div>
                  <div className="settings-field">
                    <label htmlFor="general-log-retention">日志保留天数</label>
                    <input
                      id="general-log-retention"
                      name="logRetentionDays"
                      type="number" value={settings!.general.logRetentionDays}
                      onChange={(e) => update("general", { logRetentionDays: Number(e.target.value) })}
                      className="input"
                    />
                  </div>
                  <div className="settings-field">
                    <label htmlFor="general-max-concurrent">最大并发请求数</label>
                    <input
                      id="general-max-concurrent"
                      name="maxConcurrentRequests"
                      type="number" value={settings!.general.maxConcurrentRequests}
                      onChange={(e) => update("general", { maxConcurrentRequests: Number(e.target.value) })}
                      className="input"
                    />
                  </div>
                  <div className="settings-field">
                    <label htmlFor="general-timeout">默认超时 (ms)</label>
                    <input
                      id="general-timeout"
                      name="defaultTimeout"
                      type="number" value={settings!.general.defaultTimeout} step={1000}
                      onChange={(e) => update("general", { defaultTimeout: Number(e.target.value) })}
                      className="input"
                    />
                  </div>
                </div>
              </div>
            )}

            {activeTab === "ui" && (
              <div className="settings-content-inner space-y-5">
                <h3>
                  <Monitor size={18} style={{ color: "var(--primary)" }} aria-hidden="true" />
                  界面设置
                </h3>

                <div className="space-y-4">
                  <div className="settings-field">
                    <label htmlFor="ui-language">语言</label>
                    <select
                      id="ui-language"
                      name="language"
                      value={settings!.ui.language}
                      onChange={(e) => update("ui", { language: e.target.value })}
                      className="select select-full"
                    >
                      <option value="zh-CN">简体中文</option>
                      <option value="en-US">English</option>
                    </select>
                  </div>

                  <div className="settings-field">
                    <label>主题</label>
                    <div className="flex gap-2">
                      {(["light", "dark", "system"] as const).map((t) => (
                        <button
                          key={t}
                          onClick={() => update("ui", { theme: t })}
                          className={`model-btn ${settings!.ui.theme === t ? "active" : ""}`}
                        >
                          {themeLabels[t]}
                        </button>
                      ))}
                    </div>
                  </div>

                  <div className="settings-field">
                    <label>侧边栏</label>
                    <div className="flex items-center">
                      <label className="toggle-switch">
                        <input
                          type="checkbox"
                          checked={settings!.ui.sidebarCollapsed}
                          onChange={(e) => update("ui", { sidebarCollapsed: e.target.checked })}
                        />
                        <span className="toggle-track" />
                      </label>
                      <span role="status" aria-live="polite" style={{ marginLeft: 12, fontSize: 13, color: "var(--ink-soft)" }}>
                        {settings!.ui.sidebarCollapsed ? "默认折叠" : "默认展开"}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {activeTab === "retry" && (
              <div className="settings-content-inner space-y-5">
                <h3>
                  <Shield size={18} style={{ color: "var(--primary)" }} aria-hidden="true" />
                  重试策略
                </h3>

                <div className="space-y-4">
                  <div className="settings-field">
                    <label>重试策略</label>
                    <div className="flex gap-2">
                      {RETRY_STRATEGIES.map((s) => (
                        <button
                          key={s.value}
                          onClick={() => update("retry", { strategy: s.value as RetryStrategy })}
                          className={`model-btn ${settings!.retry.strategy === s.value ? "active" : ""}`}
                        >
                          {s.label}
                        </button>
                      ))}
                    </div>
                  </div>

                  <div className="settings-field">
                    <label htmlFor="retry-max-retries">最大重试次数</label>
                    <input
                      id="retry-max-retries"
                      name="maxRetries"
                      type="number" value={settings!.retry.maxRetries} min={0} max={10}
                      onChange={(e) => update("retry", { maxRetries: Number(e.target.value) })}
                      className="input"
                    />
                  </div>
                  <div className="settings-field">
                    <label htmlFor="retry-base-delay">基础延迟 (ms)</label>
                    <input
                      id="retry-base-delay"
                      name="baseDelay"
                      type="number" value={settings!.retry.baseDelay} step={500}
                      onChange={(e) => update("retry", { baseDelay: Number(e.target.value) })}
                      className="input"
                    />
                  </div>
                  <div className="settings-field">
                    <label htmlFor="retry-max-delay">最大延迟 (ms)</label>
                    <input
                      id="retry-max-delay"
                      name="maxDelay"
                      type="number" value={settings!.retry.maxDelay} step={1000}
                      onChange={(e) => update("retry", { maxDelay: Number(e.target.value) })}
                      className="input"
                    />
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
