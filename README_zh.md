# proxydoc 🩺

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![GitHub Stars](https://img.shields.io/github/stars/TrustDec/proxydoc.svg?style=social)](https://github.com/TrustDec/proxydoc)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)]()

**专为开发者打造的终端代理深度根因诊断、DNS 投毒检测与一键自动修复 CLI。**  
*一秒搞定系统与终端代理脱节、死端口导致的 Connection refused、以及 AI 接口断连。*

[English](README.md) | [简体中文](README_zh.md)

</div>

---

## ⚡ 解决的核心根因问题

绝大多数开发者的终端网络问题，本质上都是由于以下三类暗坑引起的：
1. **系统代理与终端脱节**：浏览器能访问是因为 macOS 系统设置里开了 Web Proxy，但终端 Shell 没有继承，导致 `git clone` 或终端工具一直超时挂起；
2. **退出代理软件后的死端口残留**：关了 Clash/Shadowrocket，但 Git 全局配置或终端还留着 `7890`/`1080`，狂报 `Connection refused`；
3. **DNS 投毒与污染**：`github.com`、`api.openai.com` 本地解析被劫持解析到了 `0.0.0.0` 或无效 IP。

**`proxydoc` 能够一键揪出深层根因，并通过 `proxydoc fix` 自动修复。**

---

## ✨ 核心特性

- 🩺 **深度环境与根因体检 (`proxydoc doctor`)**：
  - 自动对比 macOS 系统代理（`scutil --proxy`）与当前终端 Shell 状态，揪出“浏览器能上、终端上不了”的脱节问题；
  - 深度校验已配置端口是否真实存活，杜绝幽灵死代理；
  - 全面体检 Git 全局代理、NPM 代理（`proxy`/`https-proxy`）以及 Cargo 配置（`[http] proxy`）；
  - 检测 `github.com`、`raw.githubusercontent.com`、`api.openai.com`、`api.anthropic.com` 的本地 DNS 解析完整性；
  - 支持 `--json` 格式输出，方便集成 CI/CD、脚本与 IDE。
- 🩹 **一键自动修复 (`proxydoc fix`)**：
  - 自动清理 Git 和 NPM 残留的失效死代理；
  - 探测并预警 Cargo 配置中的失效代理；
  - 将系统有效代理一键生成同步注入到终端；
  - 自动刷新本地 DNS 解析缓存。
- ⚡ **AI 接口并发测速与真实出口探测 (`proxydoc test`)**：
  - 全异步并发秒级测试对 OpenAI、Anthropic、GitHub、Google、Cloudflare、HuggingFace 的真实延迟；
  - **出口 IP 与位置探测**：自动提取流量真实公网出口 IP、归属国家（如 US/SG/JP）与 Cloudflare 边缘节点，一眼看清节点是否生效；
  - 支持 `--json` 格式导出。
- 🔍 **本地代理客户端扫描 (`proxydoc scan`)**：智能探测 Shadowrocket、Clash、Sing-box、Surge、CPA 等端口（支持 `--json`）。
- 🟢 **极简切换 (`proxydoc on` / `proxydoc off`)**：一秒开闭终端代理，支持 `--git`、`--npm` 或 `-a, --all` 一键联动包管理器。

---

## 🚀 常用命令

```bash
# 1. 深度诊断
proxydoc doctor

# 2. 一键自动修复
proxydoc fix

# 3. 实测 AI 与外网接口连通性
proxydoc test
```

## 📦 安装方式

```bash
curl -fsSL https://raw.githubusercontent.com/TrustDec/proxydoc/main/install.sh | bash
# 或通过 Cargo
cargo install proxydoc
```

## 📄 开源协议

MIT © [TrustDec](https://github.com/TrustDec)
