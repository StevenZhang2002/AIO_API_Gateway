import { useEffect, useState, useCallback, useRef } from "react";
import { Plus, Pencil, Trash2, FlaskConical, Server, Globe, Activity } from "lucide-react";
import {
  fetchChannels,
  fetchChannelDefaults,
  addChannel,
  updateChannel,
  deleteChannel,
  toggleChannel,
  testChannel,
} from "../lib/api";
import { CHANNEL_TYPES, CHANNEL_DEFAULTS } from "../lib/constants";
import type { Channel, ChannelDefaults, ChannelFormData, ChannelType, ChannelTestResult } from "../types";

const defaultForm: ChannelFormData = {
  name: "", type: "openai", baseUrl: "", apiKey: "",
  models: "[]", status: 1, priority: 0, weight: 1,
  config: "{}", modelMapping: "{}",
};

export default function ChannelsPage() {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [defaults, setDefaults] = useState<Record<string, ChannelDefaults>>({});
  const [loading, setLoading] = useState(true);
  const [modalOpen, setModalOpen] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [form, setForm] = useState<ChannelFormData>({ ...defaultForm });
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<ChannelTestResult | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const modalRef = useRef<HTMLDivElement>(null);

  const load = useCallback(async () => {
    const [data, defs] = await Promise.all([
      fetchChannels(),
      fetchChannelDefaults().catch(() => [] as ChannelDefaults[]),
    ]);
    setChannels(data);
    // 后端默认配置优先，本地 CHANNEL_DEFAULTS 兜底
    setDefaults(Object.fromEntries(defs.map((d) => [d.channelType, d])));
    setLoading(false);
  }, []);

  useEffect(() => { load(); }, [load]);

  // Modal keyboard handling
  useEffect(() => {
    if (!modalOpen) return;
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") setModalOpen(false);
    }
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [modalOpen]);

  /** 获取某渠道类型的默认配置：后端数据优先，本地 CHANNEL_DEFAULTS 兜底 */
  function getDefaults(type: ChannelType) {
    return defaults[type] ?? CHANNEL_DEFAULTS[type];
  }

  function openAdd() {
    setEditingId(null);
    setForm({ ...defaultForm, baseUrl: getDefaults("openai").baseUrl });
    setTestResult(null);
    setModalOpen(true);
  }

  function openEdit(ch: Channel) {
    setEditingId(ch.id);
    setForm({
      name: ch.name,
      type: ch.type,
      baseUrl: ch.baseUrl,
      apiKey: ch.apiKey,
      models: ch.models,
      status: ch.status,
      priority: ch.priority,
      weight: ch.weight,
      config: ch.config,
      modelMapping: ch.modelMapping,
    });
    setTestResult(null);
    setModalOpen(true);
  }

  function onTypeChange(type: ChannelType) {
    const d = getDefaults(type);
    setForm((f) => ({ ...f, type, baseUrl: d.baseUrl, models: JSON.stringify(d.models) }));
  }

  // 解析当前表单的 models
  function getFormModels(): string[] {
    try {
      return JSON.parse(form.models || "[]");
    } catch {
      return [];
    }
  }

  function toggleModel(model: string) {
    const currentModels = getFormModels();
    const newModels = currentModels.includes(model)
      ? currentModels.filter((m) => m !== model)
      : [...currentModels, model];
    setForm((f) => ({ ...f, models: JSON.stringify(newModels) }));
  }

  async function handleSave() {
    setSaving(true);
    try {
      if (editingId) {
        await updateChannel(editingId, form);
      } else {
        await addChannel(form);
      }
      setModalOpen(false);
      await load();
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(id: string) {
    if (deleteConfirmId === id) {
      await deleteChannel(id);
      setDeleteConfirmId(null);
      await load();
    } else {
      setDeleteConfirmId(id);
      setTimeout(() => setDeleteConfirmId(null), 3000);
    }
  }

  async function handleToggle(id: string, currentStatus: number) {
    await toggleChannel(id, currentStatus);
    await load();
  }

  async function handleTest() {
    setTesting(true);
    setTestResult(null);
    try {
      const result = await testChannel(form);
      setTestResult(result);
    } finally {
      setTesting(false);
    }
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
        <div className="card p-0">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="flex items-center gap-6 px-6 py-5 border-b border-divider" style={i === 3 ? { borderBottom: "none" } : undefined}>
              <div className="skeleton" style={{ height: 32, width: 128 }} />
              <div className="skeleton" style={{ height: 24, width: 80, borderRadius: 9999 }} />
              <div className="skeleton" style={{ height: 24, width: 160 }} />
              <div className="skeleton" style={{ height: 24, width: 48 }} />
              <div className="skeleton" style={{ height: 24, width: 64, borderRadius: 9999, marginLeft: "auto" }} />
            </div>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="page-wrap">
      {/* Page Header */}
      <div className="page-header">
        <div>
          <h1>渠道管理</h1>
          <p>管理 LLM API 接入渠道</p>
        </div>
        <button onClick={openAdd} className="btn btn-primary">
          <Plus size={17} aria-hidden="true" />
          添加渠道
        </button>
      </div>

      {/* Table */}
      <div className="table-container">
        <div className="table-scroll">
          <table>
            <thead>
              <tr>
                <th>渠道名称</th>
                <th>类型</th>
                <th>模型</th>
                <th>权重</th>
                <th>状态</th>
                <th className="text-right">操作</th>
              </tr>
            </thead>
            <tbody>
              {channels.length === 0 ? (
                <tr>
                  <td colSpan={6}>
                    <div className="empty-state">
                      <div className="empty-state-icon">
                        <Server size={22} />
                      </div>
                      <p className="empty-state-title">暂无渠道</p>
                      <p className="empty-state-desc">点击"添加渠道"按钮来接入你的第一个 LLM API</p>
                    </div>
                  </td>
                </tr>
              ) : (
                channels.map((ch) => (
                  <tr key={ch.id}>
                    <td>
                      <div className="flex items-center gap-3">
                        <div className={`icon-box icon-box-sm type-gradient-${ch.type}`} style={{ boxShadow: "var(--shadow-sm)" }} aria-hidden="true">
                          <Globe size={16} />
                        </div>
                        <div>
                          <p className="text-14 font-semibold text-ink">{ch.name}</p>
                          <p className="font-mono text-sm text-ink-soft truncate" style={{ maxWidth: 180 }}>{ch.baseUrl}</p>
                        </div>
                      </div>
                    </td>
                    <td>
                      <span className={`badge badge-bordered badge-${ch.type}`}>
                        {CHANNEL_TYPES.find((t) => t.value === ch.type)?.label}
                      </span>
                    </td>
                    <td>
                      <div className="flex flex-wrap gap-1">
                        {(() => {
                          const models: string[] = (() => {
                            try { return JSON.parse(ch.models || "[]"); } catch { return []; }
                          })();
                          return (
                            <>
                              {models.slice(0, 2).map((m) => (
                                <span key={m} className="badge badge-muted" style={{ fontSize: 11 }}>{m}</span>
                              ))}
                              {models.length > 2 && (
                                <span style={{ fontSize: 11, color: "var(--ink-subtle)", fontWeight: 500, marginTop: 2 }}>+{models.length - 2}</span>
                              )}
                            </>
                          );
                        })()}
                      </div>
                    </td>
                    <td>
                      <span className="text-14 font-semibold text-ink-muted tabular-nums">{ch.weight}</span>
                    </td>
                    <td>
                      <label className="toggle-switch" onClick={(e) => e.stopPropagation()}>
                        <input
                          type="checkbox"
                          checked={ch.status === 1}
                          onChange={() => handleToggle(ch.id, ch.status)}
                        />
                        <span className="toggle-track" />
                      </label>
                    </td>
                    <td>
                      <div className="flex items-center justify-end gap-1">
                        <button onClick={() => openEdit(ch)} className="btn btn-icon btn-ghost" title="编辑" aria-label={`编辑渠道 ${ch.name}`}>
                          <Pencil size={15} />
                        </button>
                        <button
                          onClick={() => handleDelete(ch.id)}
                          className={`btn btn-icon ${deleteConfirmId === ch.id ? "btn-danger" : "btn-ghost"}`}
                          title={deleteConfirmId === ch.id ? "再次点击确认删除" : "删除"}
                          aria-label={deleteConfirmId === ch.id ? "再次点击确认删除" : `删除渠道 ${ch.name}`}
                        >
                          <Trash2 size={15} />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Modal */}
      {modalOpen && (
        <div className="modal-overlay" onClick={() => setModalOpen(false)}>
          <div className="modal-content" style={{ maxWidth: 540 }} onClick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label={editingId ? "编辑渠道" : "添加渠道"} ref={modalRef}>
            <div className="modal-header">
              <h2>{editingId ? "编辑渠道" : "添加渠道"}</h2>
              <button onClick={() => setModalOpen(false)} className="btn btn-icon btn-ghost" aria-label="关闭对话框">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="M4 4L12 12M12 4L4 12" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round"/></svg>
              </button>
            </div>

            <div className="modal-body space-y-5">
              {/* Name + Type */}
              <div className="grid grid-2 grid-gap-sm">
                <div>
                  <label htmlFor="channel-name" className="text-13 font-semibold text-ink-muted" style={{ display: "block", marginBottom: 6 }}>渠道名称</label>
                  <input
                    id="channel-name"
                    name="name"
                    type="text" value={form.name}
                    onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
                    placeholder="我的 OpenAI 渠道"
                    className="input"
                    autoComplete="off"
                  />
                </div>
                <div>
                  <label htmlFor="channel-type" className="text-13 font-semibold text-ink-muted" style={{ display: "block", marginBottom: 6 }}>渠道类型</label>
                  <select
                    id="channel-type"
                    name="type"
                    value={form.type}
                    onChange={(e) => onTypeChange(e.target.value as ChannelType)}
                    className="select select-full"
                  >
                    {CHANNEL_TYPES.map((t) => (
                      <option key={t.value} value={t.value}>{t.label}</option>
                    ))}
                  </select>
                </div>
              </div>

              {/* Base URL */}
              <div>
                <label htmlFor="channel-base-url" className="text-13 font-semibold text-ink-muted" style={{ display: "block", marginBottom: 6 }}>API 地址</label>
                <input
                  id="channel-base-url"
                  name="baseUrl"
                  type="url" value={form.baseUrl}
                  onChange={(e) => setForm((f) => ({ ...f, baseUrl: e.target.value }))}
                  className="input input-mono"
                  autoComplete="off"
                />
              </div>

              {/* API Key */}
              <div>
                <label htmlFor="channel-api-key" className="text-13 font-semibold text-ink-muted" style={{ display: "block", marginBottom: 6 }}>API Key</label>
                <input
                  id="channel-api-key"
                  name="apiKey"
                  type="password" value={form.apiKey}
                  onChange={(e) => setForm((f) => ({ ...f, apiKey: e.target.value }))}
                  placeholder="sk-…"
                  className="input"
                  autoComplete="new-password"
                />
              </div>

              {/* Models */}
              <div>
                <label className="text-13 font-semibold text-ink-muted" style={{ display: "block", marginBottom: 8 }}>模型选择</label>
                <div className="flex flex-wrap gap-2">
                  {getDefaults(form.type).models.length === 0 ? (
                    <p className="text-13 text-ink-subtle">该类型无预置模型，可在渠道中手动配置</p>
                  ) : (
                    getDefaults(form.type).models.map((m) => (
                      <button
                        key={m}
                        type="button"
                        onClick={() => toggleModel(m)}
                        className={`model-btn ${getFormModels().includes(m) ? "active" : ""}`}
                      >
                        {m}
                      </button>
                    ))
                  )}
                </div>
              </div>

              {/* Grid: Weight, Priority */}
              <div className="grid grid-2 grid-gap-sm">
                <div>
                  <label htmlFor="channel-weight" className="text-13 font-semibold text-ink-muted" style={{ display: "block", marginBottom: 6 }}>
                    <Activity size={13} style={{ display: "inline", marginRight: 4 }} aria-hidden="true" />权重
                  </label>
                  <input
                    id="channel-weight"
                    name="weight"
                    type="number" value={form.weight ?? 1} min={1} max={100}
                    onChange={(e) => setForm((f) => ({ ...f, weight: Number(e.target.value) }))}
                    className="input"
                  />
                </div>
                <div>
                  <label htmlFor="channel-priority" className="text-13 font-semibold text-ink-muted" style={{ display: "block", marginBottom: 6 }}>优先级</label>
                  <input
                    id="channel-priority"
                    name="priority"
                    type="number" value={form.priority ?? 0} min={0} max={100}
                    onChange={(e) => setForm((f) => ({ ...f, priority: Number(e.target.value) }))}
                    className="input"
                  />
                </div>
              </div>

              {/* Test Result */}
              {testResult && (
                <div className={`test-result ${testResult.success ? "test-result-success" : "test-result-error"}`} role="status" aria-live="polite">
                  {testResult.success
                    ? `✓ 连接成功 · 延迟 ${testResult.latency}ms · 可用模型 ${testResult.models.length} 个`
                    : "✗ 连接失败，请检查 API Key 和地址是否正确"}
                </div>
              )}
            </div>

            <div className="modal-footer">
              <button
                onClick={handleTest}
                disabled={testing || !form.apiKey || !form.baseUrl}
                className="btn btn-secondary"
              >
                <FlaskConical size={15} className={testing ? "animate-pulse-dot" : ""} aria-hidden="true" />
                {testing ? "测试中\u2026" : "测试连接"}
              </button>
              <div className="flex items-center gap-3">
                <button onClick={() => setModalOpen(false)} className="btn btn-secondary">取消</button>
                <button
                  onClick={handleSave}
                  disabled={saving || !form.name}
                  className="btn btn-primary"
                >
                  {saving ? "保存中\u2026" : "保存"}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
