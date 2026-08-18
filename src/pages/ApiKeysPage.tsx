import { useEffect, useState, useCallback, useRef } from "react";
import { Plus, Trash2, Copy, Check, Key, Shield, Infinity } from "lucide-react";
import { fetchApiKeys, createApiKey, deleteApiKey, toggleApiKeyStatus } from "../lib/api";
import { ALL_MODELS } from "../lib/constants";
import type { ApiKey, ApiKeyFormData } from "../types";

const defaultForm: ApiKeyFormData = {
  name: "", quotaLimit: -1, allowedModels: [], allowedChannels: [], expiresAt: null,
};

function maskKey(key: string): string {
  if (key.length <= 18) return key;
  return key.slice(0, 14) + "\u2026" + key.slice(-4);
}

export default function ApiKeysPage() {
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [modalOpen, setModalOpen] = useState(false);
  const [form, setForm] = useState<ApiKeyFormData>({ ...defaultForm });
  const [saving, setSaving] = useState(false);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const modalRef = useRef<HTMLDivElement>(null);

  // Modal keyboard handling
  useEffect(() => {
    if (!modalOpen) return;
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") setModalOpen(false);
    }
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [modalOpen]);

  const load = useCallback(async () => {
    const data = await fetchApiKeys();
    setKeys(data);
    setLoading(false);
  }, []);

  useEffect(() => { load(); }, [load]);

  async function copyKey(key: string, id: string) {
    await navigator.clipboard.writeText(key);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  }

  async function handleCreate() {
    setSaving(true);
    try {
      await createApiKey(form);
      setModalOpen(false);
      setForm({ ...defaultForm });
      await load();
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(id: string) {
    if (deleteConfirmId === id) {
      await deleteApiKey(id);
      setDeleteConfirmId(null);
      await load();
    } else {
      setDeleteConfirmId(id);
      setTimeout(() => setDeleteConfirmId(null), 3000);
    }
  }

  async function handleToggle(id: string, currentStatus: number) {
    await toggleApiKeyStatus(id, currentStatus);
    await load();
  }

  function toggleModel(m: string) {
    setForm((f) => ({
      ...f,
      allowedModels: f.allowedModels.includes(m)
        ? f.allowedModels.filter((x) => x !== m)
        : [...f.allowedModels, m],
    }));
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
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="flex items-center gap-6 px-6 py-5 border-b border-divider" style={i === 2 ? { borderBottom: "none" } : undefined}>
              <div className="skeleton" style={{ height: 24, width: 96 }} />
              <div className="skeleton" style={{ height: 24, width: 192 }} />
              <div className="skeleton" style={{ height: 16, width: 128 }} />
              <div className="skeleton" style={{ height: 24, width: 64, borderRadius: 9999, marginLeft: "auto" }} />
            </div>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="page-wrap">
      <div className="page-header">
        <div>
          <h1>密钥管理</h1>
          <p>管理 API 访问密钥 · 创建 sk-aio-* 格式密钥</p>
        </div>
        <button
          onClick={() => { setForm({ ...defaultForm }); setModalOpen(true); }}
          className="btn btn-primary"
        >
          <Plus size={17} aria-hidden="true" />
          创建密钥
        </button>
      </div>

      {/* Table */}
      <div className="table-container">
        <div className="table-scroll">
          <table>
            <thead>
              <tr>
                <th>名称</th>
                <th>密钥</th>
                <th>配额使用</th>
                <th>状态</th>
                <th>过期时间</th>
                <th className="text-right">操作</th>
              </tr>
            </thead>
            <tbody>
              {keys.length === 0 ? (
                <tr>
                  <td colSpan={6}>
                    <div className="empty-state">
                      <div className="empty-state-icon">
                        <Key size={22} />
                      </div>
                      <p className="empty-state-title">暂无密钥</p>
                      <p className="empty-state-desc">创建你的第一个 API 密钥以开始使用 AIO Gateway</p>
                    </div>
                  </td>
                </tr>
              ) : (
                keys.map((k) => {
                  const quotaPercent = k.quotaLimit > 0 ? Math.round((k.quotaUsed / k.quotaLimit) * 100) : 0;
                  const quotaExceeded = quotaPercent >= 80;
                  return (
                    <tr key={k.id}>
                      <td>
                        <div className="flex items-center gap-2">
                          <div className="icon-box icon-box-sm" style={{ background: "linear-gradient(135deg, rgba(37,99,235,0.06), rgba(99,102,241,0.06))", color: "var(--primary)" }} aria-hidden="true">
                            <Shield size={16} />
                          </div>
                          <span className="text-14 font-semibold text-ink">{k.name}</span>
                        </div>
                      </td>
                      <td>
                        <div className="flex items-center gap-2">
                          <code className="badge badge-muted font-mono" style={{ fontSize: 12, padding: "2px 12px" }}>
                            {maskKey(k.key)}
                          </code>
                          <button
                            onClick={() => copyKey(k.key, k.id)}
                            className={`copy-btn ${copiedId === k.id ? "copied" : ""}`}
                            title="复制"
                            aria-label={`复制密钥 ${k.name}`}
                          >
                            {copiedId === k.id ? <Check size={14} aria-hidden="true" /> : <Copy size={14} aria-hidden="true" />}
                          </button>
                        </div>
                      </td>
                      <td>
                        {k.quotaLimit > 0 ? (
                          <div className="flex items-center gap-3" style={{ minWidth: 140 }}>
                            <div className="progress-bar" style={{ flex: 1 }}>
                              <div
                                className={`progress-fill ${quotaExceeded ? "progress-fill-danger" : "progress-fill-primary"}`}
                                style={{ width: `${Math.min(quotaPercent, 100)}%` }}
                              />
                            </div>
                            <span className="text-13 font-medium tabular-nums">{(k.quotaUsed / 1000).toFixed(0)}K / {(k.quotaLimit / 1000).toFixed(0)}K</span>
                          </div>
                        ) : (
                          <div className="quota-unlimited">
                            <Infinity size={14} aria-hidden="true" />
                            <span>无限制</span>
                          </div>
                        )}
                      </td>
                      <td>
                        <label className="toggle-switch" onClick={(e) => e.stopPropagation()}>
                          <input
                            type="checkbox"
                            checked={k.status === 1}
                            onChange={() => handleToggle(k.id, k.status)}
                          />
                          <span className="toggle-track" />
                        </label>
                      </td>
                      <td>
                        <span className={`text-13 font-medium tabular-nums ${
                          k.expiresAt && new Date(k.expiresAt) < new Date()
                            ? "text-danger"
                            : "text-ink-soft"
                        }`}>
                          {k.expiresAt
                            ? new Date(k.expiresAt).toLocaleDateString("zh-CN", { month: "short", day: "numeric" })
                            : "永久"}
                        </span>
                      </td>
                      <td>
                        <div className="flex items-center justify-end">
                          <button
                            onClick={() => handleDelete(k.id)}
                            className={`btn btn-icon ${deleteConfirmId === k.id ? "btn-danger" : "btn-ghost"}`}
                            title={deleteConfirmId === k.id ? "再次点击确认删除" : "删除"}
                            aria-label={deleteConfirmId === k.id ? "再次点击确认删除" : `删除密钥 ${k.name}`}
                          >
                            <Trash2 size={15} />
                          </button>
                        </div>
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Create Modal */}
      {modalOpen && (
        <div className="modal-overlay" onClick={() => setModalOpen(false)}>
          <div className="modal-content" style={{ maxWidth: 480 }} onClick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label="创建密钥" ref={modalRef}>
            <div className="modal-header">
              <h2>创建密钥</h2>
              <button onClick={() => setModalOpen(false)} className="btn btn-icon btn-ghost" aria-label="关闭对话框">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="M4 4L12 12M12 4L4 12" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round"/></svg>
              </button>
            </div>

            <div className="modal-body space-y-5">
              <div>
                <label htmlFor="key-name" className="text-13 font-semibold text-ink-muted" style={{ display: "block", marginBottom: 6 }}>密钥名称</label>
                <input
                  id="key-name"
                  name="name"
                  type="text" value={form.name}
                  onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
                  placeholder="我的 API Key"
                  className="input"
                  autoComplete="off"
                />
              </div>

              <div>
                <label htmlFor="key-quota" className="text-13 font-semibold text-ink-muted" style={{ display: "block", marginBottom: 6 }}>
                  配额限制 <span style={{ color: "var(--ink-subtle)", fontWeight: 400 }}>( -1 = 无限制 )</span>
                </label>
                <input
                  id="key-quota"
                  name="quotaLimit"
                  type="number" value={form.quotaLimit}
                  onChange={(e) => setForm((f) => ({ ...f, quotaLimit: Number(e.target.value) }))}
                  className="input"
                />
              </div>

              <div>
                <label className="text-13 font-semibold text-ink-muted" style={{ display: "block", marginBottom: 8 }}>
                  允许模型 <span style={{ color: "var(--ink-subtle)", fontWeight: 400 }}>(留空 = 全部允许)</span>
                </label>
                <div className="flex flex-wrap gap-2" style={{ maxHeight: 140, overflowY: "auto" }}>
                  {ALL_MODELS.map((m) => (
                    <button
                      key={m}
                      type="button"
                      onClick={() => toggleModel(m)}
                      className={`model-btn ${form.allowedModels.includes(m) ? "active" : ""}`}
                    >
                      {m}
                    </button>
                  ))}
                </div>
              </div>

              <div>
                <label htmlFor="key-expires" className="text-13 font-semibold text-ink-muted" style={{ display: "block", marginBottom: 6 }}>过期时间</label>
                <input
                  id="key-expires"
                  name="expiresAt"
                  type="date" value={form.expiresAt?.slice(0, 10) || ""}
                  onChange={(e) => setForm((f) => ({ ...f, expiresAt: e.target.value ? e.target.value + "T23:59:59Z" : null }))}
                  className="input"
                />
              </div>
            </div>

            <div className="modal-footer modal-footer-right">
              <div className="flex items-center gap-3">
                <button onClick={() => setModalOpen(false)} className="btn btn-secondary">取消</button>
                <button
                  onClick={handleCreate}
                  disabled={saving || !form.name}
                  className="btn btn-primary"
                >
                  {saving ? "创建中\u2026" : "创建"}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
