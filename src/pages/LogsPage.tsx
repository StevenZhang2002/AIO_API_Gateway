import { useEffect, useState, useCallback } from "react";
import { Search, X, Clock, AlertCircle, CheckCircle2, Eye, FileText, Filter, Hash, ChevronLeft, ChevronRight } from "lucide-react";
import { fetchLogs, fetchChannels } from "../lib/api";
import { CHANNEL_TYPES, ALL_MODELS } from "../lib/constants";
import type { RequestLog, LogFilter, PaginatedResult, Channel } from "../types";

function getLogStatus(statusCode: number): "success" | "error" {
  return statusCode >= 200 && statusCode < 300 ? "success" : "error";
}

const statusConfig = {
  success: { icon: CheckCircle2, badgeCls: "badge badge-success", label: "成功" },
  error:   { icon: AlertCircle,  badgeCls: "badge badge-danger",  label: "失败" },
};

export default function LogsPage() {
  const [logs, setLogs] = useState<RequestLog[]>([]);
  const [total, setTotal] = useState(0);
  const [totalPages, setTotalPages] = useState(0);
  const [loading, setLoading] = useState(true);
  const [channels, setChannels] = useState<Channel[]>([]);
  const [filter, setFilter] = useState<LogFilter>({
    keyword: "", channelId: "", model: "", statusCode: "", isStream: "", startTime: null, endTime: null, page: 1, pageSize: 10,
  });
  const [detailLog, setDetailLog] = useState<RequestLog | null>(null);

  // 加载渠道列表（用于筛选下拉）
  useEffect(() => {
    fetchChannels().then(setChannels).catch(() => {});
  }, []);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const result: PaginatedResult<RequestLog> = await fetchLogs(filter);
      setLogs(result.items);
      setTotal(result.total);
      setTotalPages(result.totalPages);
    } catch {
      setLogs([]);
      setTotal(0);
      setTotalPages(0);
    }
    setLoading(false);
  }, [filter]);

  useEffect(() => { load(); }, [load]);

  function updateFilter<K extends keyof LogFilter>(key: K, value: LogFilter[K]) {
    setFilter((f) => ({ ...f, [key]: value, page: 1 }));
  }

  function goToPage(page: number) {
    setFilter((f) => ({ ...f, page }));
  }

  function clearFilters() {
    setFilter({ keyword: "", channelId: "", model: "", statusCode: "", isStream: "", startTime: null, endTime: null, page: 1, pageSize: 10 });
  }

  const hasFilter = filter.keyword || filter.channelId || filter.model || filter.statusCode !== "" || filter.isStream !== "" || filter.startTime || filter.endTime;

  const activeTags: { label: string; onClear: () => void }[] = [];
  if (filter.keyword) activeTags.push({ label: `搜索: ${filter.keyword}`, onClear: () => updateFilter("keyword", "") });
  if (filter.channelId) {
    const ch = channels.find((c) => c.id === filter.channelId);
    activeTags.push({ label: `渠道: ${ch?.name ?? filter.channelId}`, onClear: () => updateFilter("channelId", "") });
  }
  if (filter.model) activeTags.push({ label: `模型: ${filter.model}`, onClear: () => updateFilter("model", "") });
  if (filter.statusCode !== "") activeTags.push({ label: `状态码: ${filter.statusCode}`, onClear: () => updateFilter("statusCode", "") });
  if (filter.isStream !== "") activeTags.push({ label: filter.isStream ? "流式" : "非流式", onClear: () => updateFilter("isStream", "") });

  return (
    <div className="page-wrap">
      <div className="page-header">
        <div>
          <h1>请求日志</h1>
          <p>查看所有 LLM API 请求记录 · 共 {total} 条</p>
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
              placeholder="搜索模型、密钥名、渠道名…"
              className="input"
              name="keyword"
              autoComplete="off"
              aria-label="搜索日志"
            />
          </div>

          <select
            value={filter.channelId}
            onChange={(e) => updateFilter("channelId", e.target.value)}
            className="select"
            name="channelId"
            aria-label="按渠道筛选"
          >
            <option value="">全部渠道</option>
            {channels.map((c) => (
              <option key={c.id} value={c.id}>{c.name} ({CHANNEL_TYPES.find((t) => t.value === c.type)?.label ?? c.type})</option>
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
            value={filter.statusCode === "" ? "" : String(filter.statusCode)}
            onChange={(e) => updateFilter("statusCode", e.target.value === "" ? "" : Number(e.target.value))}
            className="select"
            name="statusCode"
            aria-label="按状态码筛选"
          >
            <option value="">全部状态</option>
            <option value="200">200 成功</option>
            <option value="400">400 错误请求</option>
            <option value="403">403 禁止访问</option>
            <option value="429">429 限流</option>
            <option value="502">502 上游错误</option>
            <option value="503">503 不可用</option>
          </select>

          <select
            value={filter.isStream === "" ? "" : String(filter.isStream)}
            onChange={(e) => updateFilter("isStream", e.target.value === "" ? "" : e.target.value === "true")}
            className="select"
            name="isStream"
            aria-label="按流式筛选"
          >
            <option value="">全部模式</option>
            <option value="true">流式</option>
            <option value="false">非流式</option>
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
                    <th>渠道</th>
                    <th>模型</th>
                    <th>Token</th>
                    <th>延迟</th>
                    <th>状态</th>
                    <th>模式</th>
                    <th className="text-right">详情</th>
                  </tr>
                </thead>
                <tbody>
                  {logs.map((log) => {
                    const status = getLogStatus(log.statusCode);
                    const st = statusConfig[status];
                    const StatusIcon = st.icon;
                    return (
                      <tr key={log.id} className={`row-${status}`}>
                        <td>
                          <span className="font-mono text-sm text-ink-muted whitespace-nowrap">
                            {new Date(log.createdAt).toLocaleString("zh-CN", {
                              month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit",
                            })}
                          </span>
                        </td>
                        <td>
                          <div>
                            <p className="text-13 text-ink-muted font-medium">{log.channelName ?? "—"}</p>
                            <p className="text-sm text-ink-subtle">{log.channelName ? (CHANNEL_TYPES.find((t) => t.value === log.channelId?.replace(/-/g, ""))?.label ?? "") : ""}</p>
                          </div>
                        </td>
                        <td>
                          <span className="badge badge-muted font-mono" style={{ fontSize: 12 }}>{log.model}</span>
                        </td>
                        <td>
                          <span className="text-13 text-ink-muted font-medium tabular-nums">
                            {log.totalTokens > 0 ? log.totalTokens.toLocaleString() : "—"}
                          </span>
                        </td>
                        <td>
                          <span className={`text-13 font-medium tabular-nums ${
                            log.durationMs > 1000 ? "text-warning" : log.durationMs > 500 ? "text-ink-muted" : "text-success"
                          }`}>
                            {log.durationMs}ms
                          </span>
                        </td>
                        <td>
                          <span className={st.badgeCls} style={{ gap: 4 }}>
                            <StatusIcon size={12} aria-hidden="true" />
                            {st.label}
                          </span>
                        </td>
                        <td>
                          <div className="flex items-center gap-1">
                            <span className={`badge ${log.isStream ? "badge-info" : "badge-muted"}`} style={{ fontSize: 10, padding: "1px 6px" }}>
                              {log.isStream ? "SSE" : "STD"}
                            </span>
                            {log.isRetry && (
                              <span className="badge badge-warning" style={{ fontSize: 10, padding: "1px 6px" }}>重试</span>
                            )}
                          </div>
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

      {/* Pagination */}
      {!loading && logs.length > 0 && (
        <div className="flex items-center justify-between" style={{ marginTop: 16, padding: "0 4px" }}>
          <span className="text-13 text-ink-muted">
            第 {filter.page} / {totalPages} 页，共 {total} 条
          </span>
          <div className="flex items-center gap-2">
            <button
              onClick={() => goToPage(filter.page - 1)}
              disabled={filter.page <= 1}
              className="btn btn-icon btn-ghost"
              aria-label="上一页"
            >
              <ChevronLeft size={16} />
            </button>
            <button
              onClick={() => goToPage(filter.page + 1)}
              disabled={filter.page >= totalPages}
              className="btn btn-icon btn-ghost"
              aria-label="下一页"
            >
              <ChevronRight size={16} />
            </button>
          </div>
        </div>
      )}

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
                  <p className="detail-value mono">{new Date(detailLog.createdAt).toLocaleString("zh-CN")}</p>
                </div>
                <div>
                  <p className="detail-label">状态码</p>
                  <p className="detail-value">{detailLog.statusCode}</p>
                </div>
                <div>
                  <p className="detail-label">渠道</p>
                  <p className="detail-value">{detailLog.channelName ?? "—"}</p>
                </div>
                <div>
                  <p className="detail-label">模型</p>
                  <p className="detail-value mono">{detailLog.model}</p>
                </div>
                <div>
                  <p className="detail-label">上游模型</p>
                  <p className="detail-value mono">{detailLog.upstreamModel ?? "—"}</p>
                </div>
                <div>
                  <p className="detail-label">Token 消耗</p>
                  <p className="detail-value tabular-nums">
                    {detailLog.totalTokens > 0
                      ? `${detailLog.promptTokens.toLocaleString()} + ${detailLog.completionTokens.toLocaleString()} = ${detailLog.totalTokens.toLocaleString()}`
                      : "—"}
                  </p>
                </div>
                <div>
                  <p className="detail-label">延迟</p>
                  <p className="detail-value tabular-nums">{detailLog.durationMs}ms</p>
                </div>
                <div>
                  <p className="detail-label">API 密钥</p>
                  <p className="detail-value">{detailLog.apiKeyName ?? "—"}</p>
                </div>
                <div>
                  <p className="detail-label">模式</p>
                  <p className="detail-value">
                    {detailLog.isStream ? "流式 (SSE)" : "非流式"}
                    {detailLog.isRetry && " · 重试"}
                  </p>
                </div>
              </div>

              {detailLog.errorMessage && (
                <div>
                  <p className="detail-label mb-2"><Hash size={12} aria-hidden="true" /> 错误信息</p>
                  <pre className="pre-block pre-block-danger">{detailLog.errorMessage}</pre>
                </div>
              )}

              {detailLog.requestBody && (
                <div>
                  <p className="detail-label mb-2"><Hash size={12} aria-hidden="true" /> 请求体</p>
                  <pre className="pre-block">{(() => { try { return JSON.stringify(JSON.parse(detailLog.requestBody), null, 2); } catch { return detailLog.requestBody; } })()}</pre>
                </div>
              )}

              {detailLog.responseBody && (
                <div>
                  <p className="detail-label mb-2"><Hash size={12} aria-hidden="true" /> 响应体</p>
                  <pre className="pre-block">{(() => { try { return JSON.stringify(JSON.parse(detailLog.responseBody), null, 2); } catch { return detailLog.responseBody; } })()}</pre>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
