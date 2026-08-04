import { useEffect, useState, useCallback } from "react";
import { Search, X, Clock, AlertCircle, CheckCircle2, Loader2, Eye, FileText, Filter, Hash } from "lucide-react";
import { fetchLogs } from "../lib/api";
import { CHANNEL_TYPES, ALL_MODELS } from "../lib/constants";
import type { RequestLog, LogFilter, LogStatus, ChannelType } from "../types";

const methodBadgeCls: Record<string, string> = {
  POST: "badge badge-bordered badge-post",
  GET: "badge badge-bordered badge-get",
  PUT: "badge badge-bordered badge-put",
  DELETE: "badge badge-bordered badge-delete",
};

const statusConfig: Record<LogStatus, {
  icon: React.ComponentType<{ size?: number; className?: string }>;
  badgeCls: string;
  label: string;
}> = {
  success: { icon: CheckCircle2, badgeCls: "badge badge-success", label: "成功" },
  error:   { icon: AlertCircle,  badgeCls: "badge badge-danger",  label: "失败" },
  pending: { icon: Loader2,     badgeCls: "badge badge-info",    label: "处理中" },
};

export default function LogsPage() {
  const [logs, setLogs] = useState<RequestLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<LogFilter>({
    keyword: "", channelType: "", model: "", status: "", dateRange: null,
  });
  const [detailLog, setDetailLog] = useState<RequestLog | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    const data = await fetchLogs(filter);
    setLogs(data);
    setLoading(false);
  }, [filter]);

  useEffect(() => { load(); }, [load]);

  function updateFilter<K extends keyof LogFilter>(key: K, value: LogFilter[K]) {
    setFilter((f) => ({ ...f, [key]: value }));
  }

  function clearFilters() {
    setFilter({ keyword: "", channelType: "", model: "", status: "", dateRange: null });
  }

  const hasFilter = filter.keyword || filter.channelType || filter.model || filter.status || filter.dateRange;

  const activeTags: { label: string; onClear: () => void }[] = [];
  if (filter.keyword) activeTags.push({ label: `搜索: ${filter.keyword}`, onClear: () => updateFilter("keyword", "") });
  if (filter.channelType) {
    const ct = CHANNEL_TYPES.find((t) => t.value === filter.channelType);
    activeTags.push({ label: `渠道: ${ct?.label ?? filter.channelType}`, onClear: () => updateFilter("channelType", "") });
  }
  if (filter.model) activeTags.push({ label: `模型: ${filter.model}`, onClear: () => updateFilter("model", "") });
  if (filter.status) activeTags.push({ label: `状态: ${statusConfig[filter.status as LogStatus]?.label ?? filter.status}`, onClear: () => updateFilter("status", "") });

  return (
    <div className="page-wrap">
      <div className="page-header">
        <div>
          <h1>请求日志</h1>
          <p>查看所有 LLM API 请求记录</p>
        </div>
      </div>

      {/* Filter Bar */}
      <div className="card card-body mb-5">
        <div className="flex flex-wrap items-center gap-3">
          <div className="input-with-icon" style={{ flex: "1 1 180px" }}>
            <Search size={15} className="input-icon" aria-hidden="true" />
            <input
              type="search"
              value={filter.keyword}
              onChange={(e) => updateFilter("keyword", e.target.value)}
              placeholder="搜索路径、模型、密钥…"
              className="input"
              name="keyword"
              autoComplete="off"
              aria-label="搜索日志"
            />
          </div>

          <select
            value={filter.channelType}
            onChange={(e) => updateFilter("channelType", e.target.value as ChannelType | "")}
            className="select"
            name="channelType"
            aria-label="按渠道筛选"
          >
            <option value="">全部渠道</option>
            {CHANNEL_TYPES.map((t) => (
              <option key={t.value} value={t.value}>{t.label}</option>
            ))}
          </select>

          <select
            value={filter.model}
            onChange={(e) => updateFilter("model", e.target.value)}
            className="select"
            name="model"
            aria-label="按模型筛选"
          >
            <option value="">全部模型</option>
            {ALL_MODELS.map((m) => (
              <option key={m} value={m}>{m}</option>
            ))}
          </select>

          <select
            value={filter.status}
            onChange={(e) => updateFilter("status", e.target.value as LogStatus | "")}
            className="select"
            name="status"
            aria-label="按状态筛选"
          >
            <option value="">全部状态</option>
            <option value="success">成功</option>
            <option value="error">失败</option>
            <option value="pending">处理中</option>
          </select>

          {hasFilter && (
            <button onClick={clearFilters} className="btn btn-ghost" style={{ fontSize: 13 }} aria-label="清除全部筛选">
              <X size={14} aria-hidden="true" />
              清除全部
            </button>
          )}
        </div>

        {activeTags.length > 0 && (
          <div className="flex flex-wrap items-center gap-2" style={{ marginTop: 12, paddingTop: 12, borderTop: "1px solid var(--divider)" }}>
            <Filter size={13} style={{ color: "var(--ink-subtle)" }} aria-hidden="true" />
            {activeTags.map((tag) => (
              <button
                key={tag.label}
                onClick={tag.onClear}
                className="filter-tag"
                aria-label={`清除筛选: ${tag.label}`}
              >
                {tag.label}
                <X size={12} aria-hidden="true" />
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Logs Container */}
      <div className="card p-0 overflow-hidden">
        {loading && (
          <div className="empty-state">
            <div className="spinner" style={{ marginBottom: 16 }} />
            <p className="empty-state-title">加载日志…</p>
          </div>
        )}

        {!loading && logs.length === 0 && (
          <div className="empty-state">
            <div className="empty-state-icon">
              <FileText size={22} aria-hidden="true" />
            </div>
            <p className="empty-state-title">暂无日志记录</p>
            <p className="empty-state-desc">当 API 请求发生时，日志将在此处显示</p>
          </div>
        )}

        {!loading && logs.length > 0 && (
          <div className="table-container" style={{ border: "none", borderRadius: 0 }}>
            <div className="table-scroll">
              <table>
                <thead>
                  <tr>
                    <th>
                      <Clock size={13} style={{ display: "inline", marginRight: 6 }} aria-hidden="true" />
                      时间
                    </th>
                    <th>路径</th>
                    <th>渠道</th>
                    <th>模型</th>
                    <th>Token</th>
                    <th>延迟</th>
                    <th>状态</th>
                    <th className="text-right">详情</th>
                  </tr>
                </thead>
                <tbody>
                  {logs.map((log) => {
                    const st = statusConfig[log.status];
                    const StatusIcon = st.icon;
                    return (
                      <tr
                        key={log.id}
                        className={`row-${log.status}`}
                      >
                        <td>
                          <span className="font-mono text-sm text-ink-muted whitespace-nowrap">
                            {new Date(log.timestamp).toLocaleString("zh-CN", {
                              month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit",
                            })}
                          </span>
                        </td>
                        <td>
                          <div className="flex items-center gap-2">
                            <span className={`${methodBadgeCls[log.method] ?? "badge badge-muted"}`} style={{ fontSize: 10, padding: "1px 6px" }}>
                              {log.method}
                            </span>
                            <span className="text-13 font-mono text-ink truncate" style={{ maxWidth: 160 }}>{log.path}</span>
                          </div>
                        </td>
                        <td>
                          <div>
                            <p className="text-13 text-ink-muted font-medium">{log.channelName}</p>
                            <p className="text-sm text-ink-subtle">{CHANNEL_TYPES.find((t) => t.value === log.channelType)?.label ?? log.channelType}</p>
                          </div>
                        </td>
                        <td>
                          <span className="badge badge-muted font-mono" style={{ fontSize: 12 }}>{log.model}</span>
                        </td>
                        <td>
                          <span className="text-13 text-ink-muted font-medium tabular-nums">{log.tokenUsage.toLocaleString()}</span>
                        </td>
                        <td>
                          <span className={`text-13 font-medium tabular-nums ${
                            log.latency > 1000 ? "text-warning" : log.latency > 500 ? "text-ink-muted" : "text-success"
                          }`}>
                            {log.latency}ms
                          </span>
                        </td>
                        <td>
                          <span className={st.badgeCls} style={{ gap: 4 }}>
                            <StatusIcon size={12} className={log.status === "pending" ? "animate-spin" : ""} aria-hidden="true" />
                            {st.label}
                          </span>
                        </td>
                        <td>
                          <div className="flex items-center justify-end">
                            <button
                              onClick={() => setDetailLog(log)}
                              className="btn btn-icon btn-ghost"
                              title="查看详情"
                              aria-label={`查看请求详情: ${log.model}`}
                            >
                              <Eye size={15} aria-hidden="true" />
                            </button>
                          </div>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>

      {/* Detail Modal */}
      {detailLog && (
        <div className="modal-overlay" onClick={() => setDetailLog(null)}>
          <div className="modal-content" style={{ maxWidth: 680 }} onClick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label="请求详情">
            <div className="modal-header">
              <div className="flex items-center gap-3">
                <h2>请求详情</h2>
                {detailLog.errorMessage && (
                  <span className="badge badge-danger">含错误</span>
                )}
              </div>
              <button onClick={() => setDetailLog(null)} className="btn btn-icon btn-ghost" aria-label="关闭对话框">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="M4 4L12 12M12 4L4 12" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round"/></svg>
              </button>
            </div>

            <div className="modal-body space-y-5">
              <div className="detail-grid">
                <div>
                  <p className="detail-label">时间</p>
                  <p className="detail-value mono">{new Date(detailLog.timestamp).toLocaleString("zh-CN")}</p>
                </div>
                <div>
                  <p className="detail-label">状态码</p>
                  <p className="detail-value">{detailLog.statusCode}</p>
                </div>
                <div>
                  <p className="detail-label">渠道</p>
                  <p className="detail-value">{detailLog.channelName} · {CHANNEL_TYPES.find((t) => t.value === detailLog.channelType)?.label ?? detailLog.channelType}</p>
                </div>
                <div>
                  <p className="detail-label">模型</p>
                  <p className="detail-value mono">{detailLog.model}</p>
                </div>
                <div>
                  <p className="detail-label">Token 消耗</p>
                  <p className="detail-value tabular-nums">{detailLog.tokenUsage.toLocaleString()}</p>
                </div>
                <div>
                  <p className="detail-label">延迟</p>
                  <p className="detail-value tabular-nums">{detailLog.latency}ms</p>
                </div>
                <div>
                  <p className="detail-label">API 密钥</p>
                  <p className="detail-value">{detailLog.apiKeyName}</p>
                </div>
                <div>
                  <p className="detail-label">方法 · 路径</p>
                  <p className="detail-value">
                    <span className="flex items-center gap-1">
                      <span className={`${methodBadgeCls[detailLog.method] ?? "badge badge-muted"}`} style={{ fontSize: 10, padding: "1px 6px" }}>
                        {detailLog.method}
                      </span>
                      <span className="font-mono">{detailLog.path}</span>
                    </span>
                  </p>
                </div>
              </div>

              {detailLog.errorMessage && (
                <div>
                  <p className="detail-label mb-2"><Hash size={12} aria-hidden="true" /> 错误信息</p>
                  <pre className="pre-block pre-block-danger">{detailLog.errorMessage}</pre>
                </div>
              )}

              <div>
                <p className="detail-label mb-2"><Hash size={12} aria-hidden="true" /> 请求体</p>
                <pre className="pre-block">{detailLog.requestBody}</pre>
              </div>

              {detailLog.responseBody && (
                <div>
                  <p className="detail-label mb-2"><Hash size={12} aria-hidden="true" /> 响应体</p>
                  <pre className="pre-block">{detailLog.responseBody}</pre>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
