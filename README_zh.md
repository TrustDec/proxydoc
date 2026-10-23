# proxydoc 🩺

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![GitHub Stars](https://img.shields.io/github/stars/TrustDec/proxydoc.svg?style=social)](https://github.com/TrustDec/proxydoc)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)]()

**专为开发者打造的全终端代理环境体检、延迟测速与快捷管理 CLI。**  
*一秒解决终端报 Connection refused、Git 卡死、AI 接口连不上的网络玄学。*

[English](README.md) | [简体中文](README_zh.md)

</div>

---

## ⚡ 解决的核心痛点

很多开发者都遇到过以下抓狂场景：
- ❌ 浏览器能正常上网，但终端里 `git clone` 或 `brew update` 一直卡死。
- ❌ 关了代理客户端后，终端一直报错 `curl: (7) Failed to connect to 127.0.0.1 port 7890: Connection refused`，不知道哪个配置文件没删干净。
- ❌ 环境变量（`http_proxy`）、Git 全局配置、NPM 配置、Docker 以及各 AI Coding 客户端（Claude Code、Cursor、Codex 等）各自为战。

**`proxydoc` 提供 1 秒全系统环境诊断、本地代理软件端口自动探测、AI 接口延迟实测，以及全工具链一键开/关。**

---

## ✨ 核心特性

- 🩺 **全链路环境体检 (`proxydoc doctor`)**：一键检测 Shell 环境变量、Git 全局配置、NPM 配置、Homebrew 镜像源及本地代理端口健康度。
- 🔍 **本地代理客户端扫描 (`proxydoc scan`)**：智能识别 Clash、Clash Verge、Shadowrocket、Sing-box、V2Ray、Surge、CLI-Proxy-API 等常见监听端口。
- ⚡ **AI 接口与外网连通性测速 (`proxydoc test`)**：多线程秒级测试对 **GitHub、OpenAI、Anthropic、Google、Cloudflare、HuggingFace** 的真实连通性与毫秒延迟。
- 🟢 **一键开/关 (`proxydoc on` / `proxydoc off`)**：一秒设置/清空当前终端、Git、NPM 的代理配置。
- 🚀 **单二进制极简交付**：基于 Rust 编写，无任何运行时依赖，运行极快（毫秒级响应）。

---

## 📦 安装方式

### 极速一键安装 (macOS / Linux)
```bash
curl -fsSL https://raw.githubusercontent.com/TrustDec/proxydoc/main/install.sh | bash
```

### 通过 Cargo 源码安装
```bash
cargo install proxydoc
```

---

## 🚀 常用命令

### 1. 运行全面体检
```bash
proxydoc doctor
```

### 2. 测试 AI 接口与外网延迟
```bash
# 测试当前终端代理环境
proxydoc test

# 指定特定代理节点测试
proxydoc test --proxy socks5://127.0.0.1:1082
```

### 3. 扫描本机代理端口
```bash
proxydoc scan
```

### 4. 终端会话极速集成
将以下内容加入 `~/.zshrc` 或 `~/.bashrc`：
```bash
eval "$(proxydoc init)"
```

之后即可使用超短别名：
```bash
pon          # 开启当前终端代理（自动识别本机可用端口）
pon 7890     # 指定端口开启
pon --git    # 同时配置 git 全局代理
poff         # 关闭当前终端代理
poff --git   # 同时清空 git 代理
```

---

## 📄 开源协议

MIT © [TrustDec](https://github.com/TrustDec)
