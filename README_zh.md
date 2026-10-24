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
  - 全面体检 Git、SSH（`~/.ssh/config` ProxyCommand）、NPM、**Yarn**（`~/.yarnrc` / `~/.yarnrc.yml`）、Cargo（`[http] proxy`）、Pip（`global.proxy`）、cURL（`~/.curlrc`）、Wget（`~/.wgetrc`）、Docker（`~/.docker/config.json`）、**Gradle**（`~/.gradle/gradle.properties`）、**Maven**（`~/.m2/settings.xml`）及 **Go 模块**（`GOPROXY`）代理配置；
  - 智能诊断 **环境变量大小写冲突**（`http_proxy != HTTP_PROXY`），杜绝因 cURL（仅支持小写）与 Go/Python 读取习惯不同导致的玄学报错；
  - 诊断 **AI 开发者网关与镜像**（`OPENAI_BASE_URL`、`ANTHROPIC_BASE_URL`、`DEEPSEEK_BASE_URL`、`OLLAMA_HOST`、`HF_ENDPOINT`）是否指向不可达的幽灵端口；
  - 自动识别 **WSL2 Linux** 环境，定位 Windows 主机网关 IP，避免使用 `127.0.0.1` 导致的网络隔离死锁；
  - 检测 **SOCKS5 vs SOCKS5h DNS 泄漏风险**（`socks5h://` 走远端代理隧道解析 DNS，避免本地 DNS 污染与解析失败）；
  - 自动校验 `NO_PROXY` 本地回环绕行白名单，杜绝开启代理后本地开发服务（Vite、Next.js、Docker）卡死；
  - 检测 `github.com`、`raw.githubusercontent.com`、`api.openai.com`、`api.anthropic.com`、`api.deepseek.com`、`proxy.golang.org` 与 `registry.yarnpkg.com` 的本地 DNS 解析完整性；
  - 支持 `--json` 格式输出，方便集成 CI/CD、脚本与 IDE。
- 🩹 **一键自动修复 (`proxydoc fix`)**：
  - 自动清理 Git、Cargo（`~/.cargo/config.toml`）、NPM、**Yarn**（`~/.yarnrc` / `~/.yarnrc.yml`）、Pip、Docker（`~/.docker/config.json`）、Gradle（`~/.gradle/gradle.properties`）及 Maven 残留的失效死代理；
  - 自动备份（`.bak`）并注释禁用 **cURL**（`~/.curlrc`）与 **Wget**（`~/.wgetrc`）中的死代理配置；
  - 检测 SSH 配置中导致 `git clone git@...` 挂起的失效 ProxyCommand；
  - 将系统有效代理一键生成同步注入到终端，并自动配置标准 `NO_PROXY` 绕行；
  - 自动刷新本地 DNS 解析缓存。
- ⚡ **AI 接口并发测速与双向对照 (`proxydoc test`)**：
  - 全异步并发秒级测试对 DeepSeek、OpenAI、Anthropic、GitHub、Google、Cloudflare、HuggingFace、**crates.io**、**npm Registry**、**Yarn Registry** 与 **Go Proxy** 的真实延迟；
  - **直连与代理双向对照 (`proxydoc test --compare` / `-c`)**：同屏并发对比 Direct 与 Proxy 的延迟差与联通状态，直观看出代理加速倍率；
  - **自定义接口测速 (`proxydoc test -t <URL>`)**：一键测试自定义 API、自建网关或私有服务在代理环境下的连通与延迟；
  - **自定义超时时间 (`--timeout <SECS>`)**：弱网或卫星网络下灵活配置超时阈值；
  - **强制直连测试 (`proxydoc test --direct` / `-d`)**：忽略当前环境代理，测试裸连状态；
  - **出口 IP 与位置探测**：自动提取流量真实公网出口 IP、归属国家（如 US/SG/JP）与 Cloudflare 边缘节点，一眼看清节点是否生效；
  - 支持 `--json` 格式导出。
- 🔍 **本地代理客户端扫描 (`proxydoc scan`)**：智能探测 Shadowrocket、Clash、Sing-box、Surge、CPA 等端口（支持 `--json`）。
- 🟢 **多 Shell 极简切换 (`proxydoc on` / `proxydoc off`)**：
  - 支持 **Bash/Zsh**、**Fish** 与 **PowerShell** 专属语法导出（`--shell fish`、`--shell pwsh`）；
  - 支持 `--git`、`--cargo`、`--npm`、`--yarn`、`--pip`、`--docker`、`--gradle`、`--maven` 或 `-a, --all` 一键联动包管理器与容器并注入 `no_proxy`。

---

## 🚀 常用命令

```bash
# 1. 深度诊断
proxydoc doctor

# 2. 一键自动修复死代理并同步系统配置
proxydoc fix

# 3. 实测 AI 与外网接口连通性（DeepSeek、OpenAI、Anthropic 等）
proxydoc test

# 4. 直连与代理双向对照测速
proxydoc test --compare

# 5. 指定自定义接口测速与超时
proxydoc test -t https://api.deepseek.com/models --timeout 8

# 6. 一键联动终端与包管理器（Git, NPM, Pip, Docker）
eval $(proxydoc on -a)

# 7. 一键关闭终端与所有包管理器代理
eval $(proxydoc off -a)

# 8. Fish Shell 用户
eval (proxydoc on --shell fish)
```

## 📦 安装方式

```bash
curl -fsSL https://raw.githubusercontent.com/TrustDec/proxydoc/main/install.sh | bash
# 或通过 Cargo
cargo install proxydoc
```

## 📄 开源协议

MIT © [TrustDec](https://github.com/TrustDec)
