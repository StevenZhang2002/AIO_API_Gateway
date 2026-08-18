import { useEffect, useState } from "react";
import { Zap, Activity, Globe, Timer, TrendingUp, Clock } from "lucide-react";
import { fetchDashboardStats } from "../lib/api";
import type { DashboardResponse, RecentActivity } from "../types";

function formatNumber(n: number): string {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + "M";
  if (n >= 1000) return (n / 1000).toFixed(1) + "K";
  return n.toLocaleString();
}

function formatRelativeTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60000);
  if (diffMin < 1) return "刚刚";
  if (diffMin < 60) return `${diffMin} 分钟前`;
  const diffHour = Math.floor(diffMin / 60);
  if (diffHour < 24) return `${diffHour} 小时前`;
  const diffDay = Math.floor(diffHour / 24);
  return `${diffDay} 天前`;
}

function getActivityText(item: RecentActivity): string {
  const channel = item.channelName || "未知渠道";
  const isSuccess = item.statusCode >= 200 && item.statusCode < 400;
  const action = isSuccess ? "完成请求" : "请求失败";
  return `${channel} 渠道通过 ${item.model} ${action}`;
}

const statCards = [
  { key: "todayRequests", label: "今日请求数", icon: Zap, color: "#2563eb", cls: "stat-card-primary" },
  { key: "todayTokens", label: "Token 消耗", icon: Activity, color: "#10b981", cls: "stat-card-success" },
  { key: "activeChannels", label: "活跃渠道数", icon: Globe, color: "#6366f1", cls: "stat-card-info" },
  { key: "avgLatency", label: "平均延迟", icon: Timer, color: "#f59e0b", cls: "stat-card-warning" },
] as const;

export default function DashboardPage() {
  const [data, setData] = useState<DashboardResponse | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchDashboardStats().then((res) => {
      setData(res);
      setLoading(false);
    });
  }, []);

  if (loading) {
    return (
      <div className="page-wrap">
        <div style={{ marginBottom: 32 }}>
          <div className="skeleton" style={{ height: 32, width: 144, marginBottom: 8 }} />
          <div className="skeleton" style={{ height: 16, width: 224 }} />
        </div>
        <div className="grid grid-responsive-4 grid-gap-sm">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="card card-body">
              <div className="skeleton" style={{ height: 40, width: 40, borderRadius: 12, marginBottom: 16 }} />
              <div className="skeleton" style={{ height: 32, width: 80, marginBottom: 8 }} />
              <div className="skeleton" style={{ height: 12, width: 64 }} />
            </div>
          ))}
        </div>
      </div>
    );
  }

  const stats = data!.stats;
  const recentActivities = data!.recentActivities;

  const values: Record<string, string> = {
    todayRequests: formatNumber(stats.todayRequests),
    todayTokens: formatNumber(stats.todayTokens),
    activeChannels: stats.activeChannels.toString(),
    avgLatency: stats.avgLatency + "ms",
  };

  return (
    <div className="page-wrap">
      {/* Page Header */}
      <div className="page-header">
        <div>
          <h1>仪表盘</h1>
          <p>LLM API 网关运行概览</p>
        </div>
        <div className="page-header-badge">
          <Clock size={14} aria-hidden="true" />
          <span>实时更新中</span>
          <span className="notification-dot notification-dot-active" style={{ marginLeft: 4 }} aria-hidden="true" />
        </div>
      </div>

      {/* Stat Cards */}
      <div className="grid grid-responsive-4 grid-gap-sm mb-6 stagger-children">
        {statCards.map(({ key, label, icon: Icon, color, cls }) => (
          <div key={key} className={`stat-card ${cls} card-lift`}>
            <div className="flex items-start justify-between mb-4">
              <div
                className="icon-box icon-box-md"
                style={{ background: `linear-gradient(135deg, ${color}15, ${color}08)`, color }}
                aria-hidden="true"
              >
                <Icon size={22} aria-hidden="true" />
              </div>
            </div>
            <div>
              <p className="text-13 text-ink-soft font-medium mb-1">{label}</p>
              <div className="flex items-baseline gap-2">
                <p className="text-32 font-bold text-ink tracking-tight leading-none tabular-nums">
                  {values[key]}
                </p>
                {key === "avgLatency" && (
                  <span style={{ fontSize: 12, color: "var(--ink-subtle)", fontWeight: 500 }}>P50</span>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Bottom Row */}
      <div className="grid grid-3-to-2 grid-gap-sm">
        {/* Cumulative Stats */}
        <div className="grid grid-2 grid-gap-sm" style={{ gridColumn: "span 2" }}>
          <div className="card card-body card-hover">
            <div className="flex items-center gap-3 mb-4">
              <div className="icon-box icon-box-sm" style={{ background: "linear-gradient(135deg, #2563eb15, #2563eb08)", color: "#2563eb" }} aria-hidden="true">
                <TrendingUp size={18} />
              </div>
              <h3 className="text-15 font-semibold text-ink">累计请求数</h3>
            </div>
            <p className="text-36 font-bold text-ink tracking-tight leading-none mb-1 tabular-nums">
              {formatNumber(stats.totalRequests)}
            </p>
            <p className="text-13 text-ink-soft">自服务启动以来</p>
          </div>
          <div className="card card-body card-hover">
            <div className="flex items-center gap-3 mb-4">
              <div className="icon-box icon-box-sm" style={{ background: "linear-gradient(135deg, #10b98115, #10b98108)", color: "#10b981" }} aria-hidden="true">
                <Activity size={18} />
              </div>
              <h3 className="text-15 font-semibold text-ink">累计 Token 消耗</h3>
            </div>
            <p className="text-36 font-bold text-ink tracking-tight leading-none mb-1 tabular-nums">
              {formatNumber(stats.totalTokens)}
            </p>
            <p className="text-13 text-ink-soft">自服务启动以来</p>
          </div>

          {/* Health Check */}
          <div className="card card-body card-hover" style={{ gridColumn: "span 2" }}>
            <h3 className="section-title mb-4">
              <Activity size={14} aria-hidden="true" />
              服务健康状态
            </h3>
            <div className="dashboard-health-grid">
              {[
                { label: "错误率", value: `${stats.errorRate.toFixed(2)}%`, color: stats.errorRate < 1 ? "var(--success)" : "var(--danger)" },
                { label: "今日请求", value: stats.todayRequests.toString(), color: "var(--info)" },
                { label: "今日 Token", value: formatNumber(stats.todayTokens), color: "var(--success)" },
                { label: "活跃渠道", value: stats.activeChannels.toString(), color: "var(--info)" },
              ].map((item) => (
                <div key={item.label} className="dashboard-health-item">
                  <div className="dashboard-health-dot" style={{ background: item.color }} aria-hidden="true" />
                  <div>
                    <p className="dashboard-health-label">{item.label}</p>
                    <p className="dashboard-health-value">{item.value}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Recent Activity */}
        <div className="card card-body card-hover">
          <h3 className="section-title mb-4">
            <Clock size={14} aria-hidden="true" />
            最近活动
          </h3>
          <div>
            {recentActivities.length === 0 ? (
              <p className="text-13 text-ink-soft" style={{ textAlign: "center", padding: "24px 0" }}>暂无活动记录</p>
            ) : (
              recentActivities.map((item, i) => {
                const isSuccess = item.statusCode >= 200 && item.statusCode < 400;
                return (
                  <div
                    key={item.id}
                    className="activity-item"
                    style={i === recentActivities.length - 1 ? { borderBottom: "none" } : undefined}
                  >
                    <div className="activity-dot-wrap">
                      <div className={`activity-dot activity-dot-${isSuccess ? "success" : "error"}`} aria-hidden="true" />
                    </div>
                    <div style={{ minWidth: 0 }}>
                      <p className="activity-text">{getActivityText(item)}</p>
                      <p className="activity-time">{formatRelativeTime(item.createdAt)}</p>
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
