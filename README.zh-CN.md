# Traceless

[English](README.md) | [中文](README.zh-CN.md)

## 简介

Traceless 是一款基于 Tauri 2.x 构建的跨平台隐私保护和系统清理工具。它可以帮助用户安全地清理敏感数据、修改文件时间戳，并通过各种反取证技术保护隐私。

## 功能特性

### 免费功能
- **系统扫描** - 扫描并识别系统中的隐私风险
- **文件粉碎** - 使用多次覆写安全删除文件
- **时间戳修改** - 修改文件的创建/修改/访问时间戳
- **反分析检测** - 检测虚拟机和调试环境

### 专业版功能
- **内存清理** - 清除系统内存中的敏感数据
- **网络清理** - 清理网络痕迹、DNS 缓存和连接历史
- **系统日志清理** - 安全清理系统和应用程序日志
- **注册表清理** (Windows) - 清理 Windows 注册表痕迹
- **磁盘加密** - 加密敏感文件和文件夹

## 截图

![仪表盘](public/app-icon.png)

## 安装

### 下载预编译版本

从 [Releases](https://github.com/Robot0P/TRACELESS/releases) 下载适合您平台的最新版本：

| 平台 | 架构 | 文件 |
|------|------|------|
| macOS | ARM64 (M1/M2/M3/M4) | `Traceless_*_aarch64.dmg` |
| macOS | x64 (Intel) | `Traceless_*_x64.dmg` |
| Windows | x64 (64位) | `Traceless_*_x64-setup.exe` |
| Windows | x86 (32位) | `Traceless_*_x86-setup.exe` |
| Linux | x64 | `traceless_*_amd64.deb` / `.AppImage` |

### 从源码构建

**前置要求：**
- Node.js 18+
- Rust 1.70+
- 平台特定依赖（参见 [Tauri 前置要求](https://tauri.app/v1/guides/getting-started/prerequisites)）

```bash
# 克隆仓库
git clone https://github.com/Robot0P/TRACELESS.git
cd TRACELESS

# 安装依赖
npm install

# 开发模式运行
npm run tauri:dev

# 构建生产版本
npm run tauri:build
```

## 技术栈

- **前端：** React 19, TypeScript, Ant Design, Tailwind CSS
- **后端：** Rust, Tauri 2.x
- **国际化：** i18next（中文和英文）
- **许可证系统：** Supabase

## 许可证激活

Traceless 的专业版功能需要许可证。激活步骤：

1. 从许可证生成器获取许可证密钥
2. 在应用中打开设置
3. 输入许可证密钥并点击激活

## 项目结构

```
├── src/                    # React 前端
│   ├── components/         # UI 组件
│   ├── pages/              # 页面组件
│   ├── contexts/           # React 上下文
│   ├── i18n/               # 国际化
│   └── utils/              # 工具函数
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── commands/       # Tauri 命令
│   │   └── modules/        # 核心模块
│   └── Cargo.toml
└── package.json
```

## 相关项目

- [Traceless 许可证生成器](https://github.com/Robot0P/traceless-license-generator) - 许可证密钥生成工具

## 许可证

MIT License
