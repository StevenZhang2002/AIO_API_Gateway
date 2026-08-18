# AIO Gateway

AIO Gateway 是一个本地运行的 LLM API 统一网关桌面应用。它聚合 OpenAI、DeepSeek、阿里云百炼、Claude、Gemini 等多家模型供应商，通过统一的 OpenAI 兼容接口对外提供服务，内置渠道调度、密钥治理、请求日志与用量统计等能力，所有数据均存储在本地 SQLite 数据库中。

> 当前版本：v0.1.0（基础 API 版）

## 核心功能

| 功能 | 说明 |
|------|------|
| 🚀 统一代理端点 | 提供 OpenAI 兼容的 `/v1/chat/completions` 接口，支持流式 SSE 输出 |
| 🔀 多渠道接入 | 内置 OpenAI / DeepSeek / 阿里云百炼 / Claude / Gemini / 自定义 六类渠道适配器 |
| 🔑 密钥管理 | 生成 `sk-aio-` 前缀的网关密钥，支持启用/停用、删除与配额治理 |
| 📊 仪表盘 | 请求量、Token 用量、成功率等实时统计与最近活动 |
| 📈 用量统计 | 30 天请求/Token 趋势、渠道与模型维度用量分布 |
| 📜 请求日志 | 完整记录请求与响应体，支持多维条件检索 |
| 🛡️ 安全审计引擎（开发中） | 风险分级（Info~Critical）与处置行为（Allow/Warn/Redact/Confirm/Block）基础类型与决策矩阵已就绪 |

## 技术栈

**桌面框架**

- Tauri 2 + Rust（后端核心）
- React 19 + TypeScript + Vite（前端界面）
- Zustand（状态管理）+ TanStack Query（数据请求）
- Lucide React（图标）

**后端**

- Axum（内置 HTTP 代理服务器，与 Tauri 共享 Tokio 运行时）
- SQLx（异步 SQLite 访问 + 版本化迁移）
- 分层架构：`Command → Service → Repository → SQLite`

## 架构概览

```
┌─────────────────────────────────────────────────────┐
│                    前端 (React)                      │
│   仪表盘 / 用量 / 渠道 / 密钥 / 日志 / 设置          │
└──────────────┬──────────────────┬───────────────────┘
               │ Tauri Command    │ HTTP (OpenAI 兼容)
               ▼                  ▼
┌──────────────────────────┐  ┌───────────────────────┐
│    Tauri Rust 核心        │  │  内置代理服务器 (Axum) │
│   Service / Repository    │  │  /v1/chat/completions │
│   AdaptorRegistry 适配器  │  │  SSE 流式转发          │
└──────────────┬───────────┘  └───────────┬───────────┘
               ▼                          ▼
        ┌──────────────────────────────────────┐
        │        SQLite (本地 aio_gateway.db)   │
        └──────────────────────────────────────┘
```

外部 LLM 客户端只需将 Base URL 指向本应用暴露的代理端点，即可无感切换底层供应商。

## 环境要求

- Node.js ≥ 20
- Rust（stable，含 `cargo`）
- Tauri CLI（`npm install` 后通过 `npx tauri` 使用）
- macOS / Windows / Linux（Tauri 2 跨平台，当前主要为 macOS 打包）

## 快速开始（开发模式）

```bash
# 1. 安装前端依赖
npm install

# 2. 启动开发模式（自动拉起前端 Vite + Rust 应用）
npm run tauri dev
```

## 构建与打包

```bash
# 仅构建前端产物
npm run build

# 构建并打包桌面应用（生成 .app / .dmg / .deb 等安装包）
npm run tauri build
```

打包产物输出在 `src-tauri/target/release/bundle/` 目录下：

- **macOS**：`bundle/macos/aio-gateway.app`（直接可运行的 .app）
- **macOS 安装包**：`bundle/dmg/aio-gateway_0.1.0_<arch>.dmg`
- **Windows**：`bundle/nsis/` 与 `bundle/msi/`
- **Linux**：`bundle/deb/` 与 `bundle/appimage/`

> 提示：`.app` 可直接拖入「应用程序」使用；`.dmg` 适合分发安装。

## 数据存储

- 数据库文件位于系统应用数据目录：`~/Library/Application Support/com.bowen.aio-gateway/aio_gateway.db`
- **首次启动自动初始化数据库**：应用启动时自动执行 `src-tauri/migrations/` 下的版本化迁移（建表、索引、日志字段扩展），无需手动建库
- 数据库连接串使用 `mode=rwc`，目录不存在时自动创建

## 使用说明

1. 启动应用后，在「渠道管理」中添加并启用至少一个上游渠道（配置 Base URL 与 API Key）
2. 在「密钥管理」中创建网关 API Key（前缀 `sk-aio-`）
3. 将你的 LLM 客户端指向代理端点：

```bash
curl http://localhost:8080/v1/chat/completions \
  -H "Authorization: Bearer sk-aio-xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "你好"}],
    "stream": true
  }'
```

代理服务信息（应用启动时打印在终端）：

- 代理端点：`http://localhost:8080/v1/chat/completions`
- 健康检查：`http://localhost:8080/health`

## 目录结构

```
├── src/                      # 前端 (React + TS)
│   ├── pages/                # 页面：仪表盘/用量/渠道/密钥/日志/设置
│   ├── components/           # 布局与通用组件
│   ├── lib/                  # API 封装（Tauri invoke）与常量
│   └── types/                # 类型定义（与后端 DTO 对齐）
├── src-tauri/                # 后端 (Rust)
│   ├── src/
│   │   ├── server/           # 内置 Axum 代理服务器
│   │   ├── services/         # 业务层（含 proxy / audit 引擎）
│   │   ├── adaptor/          # 各供应商适配器
│   │   ├── db/               # SQLite 连接池 / Repository / 模型
│   │   ├── dto/              # 数据传输对象
│   │   └── lib.rs            # Tauri 入口与 Command 注册
│   ├── migrations/           # 数据库版本化迁移脚本
│   └── tauri.conf.json       # 应用打包配置
└── docs/                     # 设计文档（代理/调度/聚合等）
```

## 文档

- [数据库 Schema 设计](docs/backend/database-schema.md)
- [代理服务设计](docs/backend/proxy-service-design.md)
- [渠道调度设计](docs/backend/dispatcher-design.md)
- [仪表盘聚合设计](docs/backend/dashboard-aggregation-design.md)
- [日志前后端对接](docs/backend/logs-frontend-integration.md)

## License

Private / Internal use.
