import { Outlet } from "react-router-dom";
import Sidebar from "./Sidebar";

export default function Layout() {
  return (
    <div className="app-layout">
      <a href="#main-content" className="skip-link" style={{
        position: "absolute",
        left: "-9999px",
        top: "auto",
        width: "1px",
        height: "1px",
        overflow: "hidden",
        zIndex: 100,
        padding: "8px 16px",
        background: "var(--primary)",
        color: "#fff",
        fontSize: "14px",
        fontWeight: 500,
        borderRadius: "0 0 8px 0",
        textDecoration: "none",
      }}
      onFocus={(e) => { e.currentTarget.style.left = "0"; e.currentTarget.style.width = "auto"; e.currentTarget.style.height = "auto"; }}
      onBlur={(e) => { e.currentTarget.style.left = "-9999px"; e.currentTarget.style.width = "1px"; e.currentTarget.style.height = "1px"; }}
      >
        跳转到主内容
      </a>
      <Sidebar />
      <main className="main-content" id="main-content">
        <Outlet />
      </main>
    </div>
  );
}
