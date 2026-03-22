# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Startup Manager — 一个本地项目启动管理器，用于集中管理和启动/停止多个开发项目进程。基于 Tauri v2 + 原生 HTML/JS 前端（无框架），Windows 桌面应用。

## Build & Dev Commands

```bash
npm run dev          # 启动开发模式（Tauri dev server）
npm run build        # 构建生产版本
npm run tauri -- build   # 等价于 npm run build
cargo build          # 仅编译 Rust 后端（不含前端）
cargo check          # 快速类型检查
```

无测试、无 lint 配置。

## Architecture

**单文件 Rust 后端** (`main.rs`)：所有后端逻辑在一个文件中，包括：
- 数据模型（`Project`, `ProjectStatus`）
- 应用状态（`AppState`）— 用 `Mutex<HashMap>` 管理子进程和日志
- Tauri commands：`get_projects`, `add_project`, `update_project`, `delete_project`, `start_project`, `stop_project`, `get_status`, `get_all_statuses`, `get_project_logs`, `open_directory`
- 进程管理：Windows 上通过 `cmd /C` 启动，`taskkill /T /F` 终止进程树
- stdout/stderr 在后台线程中捕获，日志上限 100KB 自动截断

**单文件前端** (`dist/index.html`)：纯 HTML/CSS/JS，通过 `window.__TAURI__.core.invoke` 调用后端命令。每 3 秒轮询状态。

**数据持久化**：项目配置存储在 `%APPDATA%/startup-manager/projects.json`。

**系统托盘**：关闭窗口时隐藏到托盘而非退出，双击托盘图标恢复窗口。

## Key Conventions

- 前端不使用构建工具，直接编辑 `dist/index.html`
- Tauri 配置在 `tauri.conf.json`，权限在 `capabilities/default.json`
- `build.rs` 仅调用 `tauri_build::build()`
- UI 语言为中文
