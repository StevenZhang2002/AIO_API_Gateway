import { useEffect, useState } from "react";
import { Zap, Activity, Globe, Timer, TrendingUp, ArrowUpRight, Clock } from "lucide-react";
import { fetchDashboardStats } from "../lib/api";
import type { DashboardStats } from "../types";

function formatNumber(n: number): string {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + "M";
  if (n >= 1000) return (n / 1000).toFixed(1) + "K";
  return n.toLocaleString();
}

function Sparkline({ data, color, height = 36 }: { data: number[]; color: string; height?: number }) {
  const width = 80;
  const max = Math.max(...data, 1);
  const min = Math.min(...data, 0);
  const range = max - min || 1;
  const points = data
    .map((v, i) => {
      const x = (i / (data.length - 1)) * width;
      const y = height - ((v - min) / range) * (height - 4) - 2;
      return `${x},${y}`;
    })
    .join(" ");
  const areaPoints = `0,${height} ${points} ${width},${height}`;

  return (
    <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} style={{ flexShrink: 0 }} aria-hidden="true">
      <defs>
        <linearGradient id={`sparkGrad-${color.replace("#", "")}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.25" />
          <stop offset="100%" stopColor={color} stopOpacity="0.02" />
        </linearGradient>
      </defs>
      <polygon points={areaPoints} fill={`url(#sparkGrad-${color.replace("#", "")})`} />
      <polyline points={points} fill="none" stroke={color} strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function genSparkData(seed: number, count = 10): number[] {
  const data: number[] = [];
  let v = 50 + (seed % 30);
  for (let i = 0; i < count; i++) {
    v = Math.max(5, Math.min(100, v + (Math.sin(i * 1.3 + seed) * 8 + (Math.random() - 0.5) * 12)));
    data.push(Math.round(v));
  }
  return data;
}

const statCards = [
  { key: "todayRequests", label: "今日请求数", icon: Zap, color: "#2563eb", cls: "stat-card-primary" },
  { key: "todayTokens", label: "Token 消耗", icon: Activity, color: "#10b981", cls: "stat-card-success" },
  { key: "activeChannels", label: "活跃渠道数", icon: Globe, color: "#6366f1", cls: "stat-card-info" },
  { key: "avgLatency", label: "平均延迟", icon: Timer, color: "#f59e0b", cls: "stat-card-warning" },
] as const;

const recentActivity = [
  { time: "2 分钟前", text: "OpenAI Primary 渠道通过 gpt-4o 完成请求", status: "success" },
  { time: "8 分钟前", text: "DeepSeek Main 渠道通过 deepseek-chat 完成请求", status: "success" },
  { time: "15 分钟前", text: "Claude Enterprise 连接超时，已重试成功", status: "warning" },
  { time: "28 分钟前", text: "新 API 密钥 Test Key 被创建", status: "info" },
  { time: "42 分钟前", text: "Gemini-Pro 渠道已被禁用", status: "warning" },
];

export default function DashboardPage() {
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchDashboardStats().then((data) => {
      setStats(data);
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

  const values: Record<string, string> = {
    todayRequests: formatNumber(stats!.todayRequests),
    todayTokens: formatNumber(stats!.todayTokens),
    activeChannels: stats!.activeChannels.toString(),
    avgLatency: stats!.avgLatency + "ms",
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
        {statCards.map(({ key, label, icon: Icon, color, cls }, idx) => (
          <div key={key} className={`stat-card ${cls} card-lift`}>
            <div className="flex items-start justify-between mb-4">
              <div
                className="icon-box icon-box-md"
                style={{ background: `linear-gradient(135deg, ${color}15, ${color}08)`, color }}
                aria-hidden="true"
              >
                <Icon size={22} aria-hidden="true" />
              </div>
              <Sparkline data={genSparkData(idx)} color={color} />
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
            <div className="flex items-center gap-1 mt-3" style={{ fontSize: 12, color: "var(--success)", fontWeight: 500 }}>
              <ArrowUpRight size={13} aria-hidden="true" />
              <span>+12.5%</span>
              <span style={{ color: "var(--ink-subtle)", fontWeight: 400, marginLeft: 4 }}>vs 昨日</span>
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
              {formatNumber(stats!.totalRequests)}
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
              {formatNumber(stats!.totalTokens)}
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
                { label: "CPU", value: "23%", color: "var(--success)" },
                { label: "内存", value: "1.2 GB", color: "var(--info)" },
                { label: "运行时间", value: "14d 6h", color: "var(--success)" },
                { label: "错误率", value: "0.12%", color: "var(--success)" },
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
            {recentActivity.map((item, i) => (
              <div
                key={i}
                className="activity-item"
                style={i === recentActivity.length - 1 ? { borderBottom: "none" } : undefined}
              >
                <div className="activity-dot-wrap">
                  <div className={`activity-dot activity-dot-${item.status}`} aria-hidden="true" />
                </div>
                <div style={{ minWidth: 0 }}>
                  <p className="activity-text">{item.text}</p>
                  <p className="activity-time">{item.time}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
