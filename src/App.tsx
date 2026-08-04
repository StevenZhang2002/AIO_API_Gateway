import { Suspense, lazy } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import Layout from "./components/layout/Layout";

const DashboardPage = lazy(() => import("./pages/DashboardPage"));
const ChannelsPage = lazy(() => import("./pages/ChannelsPage"));
const ApiKeysPage = lazy(() => import("./pages/ApiKeysPage"));
const LogsPage = lazy(() => import("./pages/LogsPage"));
const UsagePage = lazy(() => import("./pages/UsagePage"));
const SettingsPage = lazy(() => import("./pages/SettingsPage"));

function PageFallback() {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%" }}>
      <div style={{ display: "flex", alignItems: "center", gap: 12, color: "var(--ink-soft)", fontSize: 15 }}>
        <div className="spinner" />
        <span>加载中\u2026</span>
      </div>
    </div>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <Suspense fallback={<PageFallback />}>
        <Routes>
          <Route element={<Layout />}>
            <Route index element={<DashboardPage />} />
            <Route path="usage" element={<UsagePage />} />
            <Route path="channels" element={<ChannelsPage />} />
            <Route path="keys" element={<ApiKeysPage />} />
            <Route path="logs" element={<LogsPage />} />
            <Route path="settings" element={<SettingsPage />} />
          </Route>
        </Routes>
      </Suspense>
    </BrowserRouter>
  );
}
