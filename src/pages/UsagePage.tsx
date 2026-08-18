import { useEffect, useState } from "react";
import { TrendingUp, BarChart3, PieChart, Calendar } from "lucide-react";
import { fetchUsageOverview } from "../lib/api";
import type { UsageOverview } from "../types";

function formatNumber(n: number): string {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + "M";
  if (n >= 1000) return (n / 1000).toFixed(1) + "K";
  return n.toLocaleString();
}

function AreaChart({ data }: { data: { date: string; requests: number }[] }) {
  const w = 600;
  const h = 180;
  const pad = { top: 10, right: 8, bottom: 22, left: 48 };
  const chartW = w - pad.left - pad.right;
  const chartH = h - pad.top - pad.bottom;

  const maxVal = Math.max(...data.map((d) => d.requests), 1);
  const points = data
    .map((d, i) => {
      const x = pad.left + (i / (data.length - 1)) * chartW;
      const y = pad.top + chartH - (d.requests / maxVal) * chartH;
      return `${x},${y}`;
    })
    .join(" ");

  const areaD = `M${pad.left},${pad.top + chartH} ${points} L${pad.left + chartW},${pad.top + chartH} Z`;

  const labelIndices = data
    .map((_d, i) => (i % 7 === 0 ? i : -1))
    .filter((i) => i >= 0);

  return (
    <svg viewBox={`0 0 ${w} ${h}`} style={{ width: "100%", height: "auto" }} preserveAspectRatio="xMidYMid meet" role="img" aria-label="每日请求趋势图表">
      <defs>
        <linearGradient id="areaGrad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#2563eb" stopOpacity="0.2" />
          <stop offset="100%" stopColor="#2563eb" stopOpacity="0.02" />
        </linearGradient>
      </defs>
      {[0, 0.25, 0.5, 0.75, 1].map((pct) => {
        const y = pad.top + chartH - pct * chartH;
        return (
          <g key={pct}>
            <line x1={pad.left} y1={y} x2={pad.left + chartW} y2={y} stroke="#f1f5f9" strokeWidth="1" />
            {pct > 0 && (
              <text x={pad.left - 4} y={y + 4} textAnchor="end" fill="#94a3b8" fontSize="10">
                {Math.round(maxVal * pct)}
              </text>
            )}
          </g>
        );
      })}
      <path d={areaD} fill="url(#areaGrad)" />
      <polyline points={points} fill="none" stroke="#2563eb" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
      {data.map((d, i) => {
        const x = pad.left + (i / (data.length - 1)) * chartW;
        const y = pad.top + chartH - (d.requests / maxVal) * chartH;
        return (
          <circle
            key={i}
            cx={x}
            cy={y}
            r="8"
            fill="transparent"
            style={{ cursor: "pointer" }}
          >
            <title>{`${d.date}: ${d.requests.toLocaleString()} 请求`}</title>
          </circle>
        );
      })}
      {labelIndices.map((i) => {
        const x = pad.left + (i / (data.length - 1)) * chartW;
        return (
          <text
            key={i}
            x={x}
            y={h - 4}
            textAnchor="middle"
            fill="#94a3b8"
            fontSize="10"
            fontFamily="system-ui, sans-serif"
          >
            {data[i].date.slice(5)}
          </text>
        );
      })}
    </svg>
  );
}

const barColors = ["#2563eb", "#6366f1", "#10b981", "#f59e0b", "#ef4444", "#8b5cf6", "#06b6d4", "#f97316"];

export default function UsagePage() {
  const [data, setData] = useState<UsageOverview | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchUsageOverview().then((d) => {
      setData(d);
      setLoading(false);
    });
  }, []);

  if (loading) {
    return (
      <div className="page-wrap">
        <div className="page-header">
          <div>
            <div className="skeleton" style={{ height: 32, width: 128, marginBottom: 8 }} />
            <div className="skeleton" style={{ height: 16, width: 208 }} />
          </div>
        </div>
        <div className="grid grid-2 grid-gap-sm">
          <div className="skeleton" style={{ height: 112, borderRadius: 12 }} />
          <div className="skeleton" style={{ height: 112, borderRadius: 12 }} />
        </div>
        <div className="skeleton" style={{ height: 240, borderRadius: 12, marginTop: 20 }} />
      </div>
    );
  }

  return (
    <div className="page-wrap">
      <div className="page-header">
        <div>
          <h1>用量统计</h1>
          <p>过去 30 天的请求与 Token 使用趋势</p>
        </div>
        <div className="page-header-badge">
          <Calendar size={14} aria-hidden="true" />
          <span>近 30 天</span>
        </div>
      </div>

      {/* Overview Cards */}
      <div className="grid grid-responsive-2 grid-gap-sm mb-6">
        <div className="stat-card stat-card-primary">
          <div className="flex items-center gap-3 mb-4">
            <div className="icon-box icon-box-md" style={{ background: "linear-gradient(135deg, #2563eb15, #2563eb08)", color: "#2563eb" }} aria-hidden="true">
              <TrendingUp size={20} />
            </div>
            <span className="text-14 text-ink-soft font-medium">总请求数</span>
          </div>
          <p className="text-34 font-bold text-ink tracking-tight tabular-nums">{formatNumber(data!.totalRequests)}</p>
        </div>
        <div className="stat-card stat-card-success">
          <div className="flex items-center gap-3 mb-4">
            <div className="icon-box icon-box-md" style={{ background: "linear-gradient(135deg, #10b98115, #10b98108)", color: "#10b981" }} aria-hidden="true">
              <BarChart3 size={20} />
            </div>
            <span className="text-14 text-ink-soft font-medium">总 Token 消耗</span>
          </div>
          <p className="text-34 font-bold text-ink tracking-tight tabular-nums">{formatNumber(data!.totalTokens)}</p>
        </div>
      </div>

      {/* Daily Chart */}
      <div className="card card-body mb-6 card-hover">
        <h3 className="section-title mb-2">
          <TrendingUp size={14} aria-hidden="true" />
          每日请求趋势
        </h3>
        <AreaChart data={data!.dailyData} />
      </div>

      {/* Breakdown */}
      <div className="grid grid-responsive-2 grid-gap-lg">
        {/* Channel Usage */}
        <div className="card card-body card-hover">
          <h3 className="section-title">
            <PieChart size={14} aria-hidden="true" />
            渠道用量分布
          </h3>
          <div className="space-y-5">
            {data!.channelUsage.map((c, i) => (
              <div key={c.channelName ?? c.channelType}>
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <div
                      style={{ width: 10, height: 10, borderRadius: "50%", background: barColors[i % barColors.length], flexShrink: 0 }}
                    />
                    <span className="text-13 font-medium text-ink-muted">{c.channelName ?? "未知渠道"}</span>
                  </div>
                  <span style={{ fontSize: 12, fontWeight: 600, color: "var(--ink-soft)" }} className="tabular-nums">{c.percentage.toFixed(1)}%</span>
                </div>
                <div className="progress-bar progress-bar-lg">
                  <div
                    className="progress-fill"
                    style={{
                      width: `${c.percentage}%`,
                      background: `linear-gradient(90deg, ${barColors[i % barColors.length]}dd, ${barColors[i % barColors.length]})`,
                      height: 12,
                      borderRadius: 9999,
                    }}
                  />
                </div>
                <p style={{ fontSize: 11, color: "var(--ink-subtle)", marginTop: 6 }}>
                  {formatNumber(c.requests)} 请求 · {formatNumber(c.tokens)} Token
                </p>
              </div>
            ))}
          </div>
        </div>

        {/* Model Usage */}
        <div className="card card-body card-hover">
          <h3 className="section-title">
            <PieChart size={14} aria-hidden="true" />
            模型用量分布
          </h3>
          <div className="space-y-5">
            {data!.modelUsage.map((m, i) => (
              <div key={m.model}>
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <div
                      style={{ width: 10, height: 10, borderRadius: "50%", background: barColors[(i + 2) % barColors.length], flexShrink: 0 }}
                    />
                    <span className="text-13 font-medium text-ink-muted font-mono">{m.model}</span>
                  </div>
                  <span style={{ fontSize: 12, fontWeight: 600, color: "var(--ink-soft)" }} className="tabular-nums">{m.percentage.toFixed(1)}%</span>
                </div>
                <div className="progress-bar progress-bar-lg">
                  <div
                    className="progress-fill"
                    style={{
                      width: `${m.percentage}%`,
                      background: `linear-gradient(90deg, ${barColors[(i + 2) % barColors.length]}dd, ${barColors[(i + 2) % barColors.length]})`,
                      height: 12,
                      borderRadius: 9999,
                    }}
                  />
                </div>
                <p style={{ fontSize: 11, color: "var(--ink-subtle)", marginTop: 6 }}>
                  {formatNumber(m.requests)} 请求 · {formatNumber(m.tokens)} Token
                </p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
