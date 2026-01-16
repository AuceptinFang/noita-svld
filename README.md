# SVLD - Save File Manager

一个基于 Tauri 和 Yew 构建的跨平台存档管理应用。

## 功能特性

- 🗂️ 存档文件管理
- 💾 自动备份功能
- 🔍 文件路径管理
- 🎨 现代化的用户界面
- 🚀 跨平台支持（Windows、macOS、Linux）

## 技术栈

- **前端**: Yew (Rust WebAssembly)
- **后端**: Tauri 2.0
- **数据库**: SQLite (sqlx)
- **构建工具**: Trunk

## 安装

从 [Releases](https://github.com/Auceptin/svld/releases) 页面下载适合你操作系统的安装包：

- **Windows**: `.msi` 或 `.exe`
- **macOS**: `.dmg` 或 `.app`
- **Linux**: `.deb` 或 `.AppImage`

## 开发

### 前置要求

- Rust (stable)
- Trunk: `cargo install trunk`
- Node.js (可选，用于前端工具)

### 运行开发环境

```bash
# 安装依赖
cargo build

# 运行开发服务器
cargo tauri dev
```

### 构建发布版本

```bash
cargo tauri build
```

## 许可证

MIT License

## 作者

Auceptin

开发使用 tauri + yew
