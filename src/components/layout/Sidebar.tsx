import { useState } from "react";
import { NavLink, useLocation } from "react-router-dom";
import {
  LayoutDashboard,
  BarChart3,
  Route,
  Key,
  ScrollText,
  Settings,
  ChevronLeft,
  ChevronRight,
  Zap,
} from "lucide-react";
import { NAV_ITEMS } from "../../lib/constants";

const iconMap: Record<string, React.FC<{ size?: number; className?: string }>> = {
  LayoutDashboard,
  BarChart3,
  Route,
  Key,
  ScrollText,
  Settings,
};

export default function Sidebar() {
  const location = useLocation();
  const [collapsed, setCollapsed] = useState(false);

  return (
    <aside className={`sidebar ${collapsed ? "collapsed" : ""}`}>
      {/* Logo */}
      {!collapsed ? (
        <div className="sidebar-logo">
          <div className="sidebar-logo-icon">
            <Zap size={18} color="#fff" aria-hidden="true" />
            <div className="sidebar-logo-dot" aria-hidden="true" />
          </div>
          <div className="sidebar-logo-text">
            <h1>AIO Gateway</h1>
            <p>LLM API Hub</p>
          </div>
        </div>
      ) : (
        <div className="sidebar-logo-centered">
          <div className="sidebar-logo-icon">
            <Zap size={18} color="#fff" aria-hidden="true" />
            <div className="sidebar-logo-dot" aria-hidden="true" />
          </div>
        </div>
      )}

      {/* Navigation */}
      <nav className="sidebar-nav">
        {NAV_ITEMS.map((item) => {
          const isActive = location.pathname === item.path;
          const Icon = iconMap[item.icon];
          return (
            <NavLink
              key={item.path}
              to={item.path}
              className={`sidebar-nav-item ${isActive ? "active" : ""} ${collapsed ? "centered" : ""}`}
            >
              {isActive && !collapsed && <div className="sidebar-active-bar" />}
              <Icon size={20} className="nav-icon" aria-hidden="true" />
              {!collapsed && <span className="nav-label">{item.label}</span>}
              {isActive && collapsed && <div className="sidebar-nav-dot" />}
            </NavLink>
          );
        })}
      </nav>

      {/* Collapse Toggle */}
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="sidebar-collapse-btn"
        aria-label={collapsed ? "展开侧边栏" : "折叠侧边栏"}
      >
        {collapsed ? <ChevronRight size={16} aria-hidden="true" /> : <ChevronLeft size={16} aria-hidden="true" />}
        {!collapsed && <span>收起菜单</span>}
      </button>

      {/* Status & Version */}
      <div className={`sidebar-footer ${collapsed ? "collapsed" : ""}`}>
        {!collapsed && (
          <div className="sidebar-footer-status">
            <span className="notification-dot notification-dot-active" aria-hidden="true" />
            <span>服务运行中</span>
          </div>
        )}
        {collapsed && (
          <span className="notification-dot notification-dot-active" style={{ marginBottom: 8 }} aria-hidden="true" />
        )}
        <p className={`sidebar-footer-version ${collapsed ? "centered" : ""}`}>
          v0.1.0
        </p>
      </div>
    </aside>
  );
}
