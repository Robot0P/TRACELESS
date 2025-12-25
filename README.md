# Traceless

[English](README.md) | [中文](README.zh-CN.md)

## Introduction

Traceless is a cross-platform privacy protection and system cleanup tool built with Tauri 2.x. It helps users securely clean up sensitive data, modify file timestamps, and protect privacy through various anti-forensics techniques.

## Features

### Free Features
- **System Scan** - Scan and identify privacy risks on your system
- **File Shredder** - Securely delete files with multiple overwrite passes
- **Timestamp Modifier** - Modify file creation/modification/access timestamps
- **Anti-Analysis Detection** - Detect virtual machines and debugging environments

### Pro Features
- **Memory Cleanup** - Clear sensitive data from system memory
- **Network Cleanup** - Clean network traces, DNS cache, and connection history
- **System Logs Cleanup** - Securely clean system and application logs
- **Registry Cleanup** (Windows) - Clean Windows registry traces
- **Disk Encryption** - Encrypt sensitive files and folders

## Screenshots

![Dashboard](public/app-icon.png)

## Installation

### Download Pre-built Binaries

Download the latest release for your platform from [Releases](https://github.com/Robot0P/TRACELESS/releases):

| Platform | Architecture | File |
|----------|-------------|------|
| macOS | ARM64 (M1/M2/M3/M4) | `Traceless_*_aarch64.dmg` |
| macOS | x64 (Intel) | `Traceless_*_x64.dmg` |
| Windows | x64 (64-bit) | `Traceless_*_x64-setup.exe` |
| Windows | x86 (32-bit) | `Traceless_*_x86-setup.exe` |
| Linux | x64 | `traceless_*_amd64.deb` / `.AppImage` |

### Build from Source

**Prerequisites:**
- Node.js 18+
- Rust 1.70+
- Platform-specific dependencies (see [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites))

```bash
# Clone the repository
git clone https://github.com/Robot0P/TRACELESS.git
cd TRACELESS

# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

## Tech Stack

- **Frontend:** React 19, TypeScript, Ant Design, Tailwind CSS
- **Backend:** Rust, Tauri 2.x
- **Internationalization:** i18next (Chinese & English)
- **License System:** Supabase

## License Activation

Traceless uses a license system for Pro features. To activate:

1. Obtain a license key from the license generator
2. Open Settings in the app
3. Enter your license key and click Activate

## Project Structure

```
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── pages/              # Page components
│   ├── contexts/           # React contexts
│   ├── i18n/               # Internationalization
│   └── utils/              # Utilities
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri commands
│   │   └── modules/        # Core modules
│   └── Cargo.toml
└── package.json
```

## Related Projects

- [Traceless License Generator](https://github.com/Robot0P/traceless-license-generator) - License key generator tool

## License

MIT License
