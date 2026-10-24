# proxydoc 🩺

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![GitHub Stars](https://img.shields.io/github/stars/TrustDec/proxydoc.svg?style=social)](https://github.com/TrustDec/proxydoc)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)]()

**Deep developer proxy diagnostics, DNS poisoning detector & 1-click auto-repair CLI.**  
*Fix broken terminal networks, dead proxy ports, and out-of-sync system proxies in 1 second.*

[English](README.md) | [简体中文](README_zh.md)

</div>

---

## ⚡ The Root Causes It Solves

Most developers suffer from network problems not because of complex networking, but due to **silent desync and dead ports**:
1. **System vs Terminal Desync**: Your browser works fine because macOS has Web Proxy enabled in GUI settings, but your terminal runs direct and hangs on `git clone` or AI APIs.
2. **Dead Ports After Quitting Proxy Apps**: You close Clash/Shadowrocket, but your Shell or `git config --global http.proxy` still points to port `7890`/`1080`, throwing `curl: (7) Connection refused`.
3. **DNS Poisoning**: `github.com` or `api.openai.com` silently resolves to `0.0.0.0` or bogus IPs due to polluted local DNS.

**`proxydoc` analyzes the root cause, checks DNS integrity, and repairs broken configs with `proxydoc fix`.**

---

## ✨ Features

- 🩺 **Deep Diagnostics (`proxydoc doctor`)**:
  - Compares macOS GUI System Proxy (`scutil --proxy`) vs Terminal Shell environment.
  - Verifies whether configured proxy ports are actually ALIVE or DEAD.
  - Inspects Git, SSH (`~/.ssh/config` ProxyCommand), NPM, **Yarn** (`~/.yarnrc` / `~/.yarnrc.yml`), Cargo (`[http] proxy`), Pip (`global.proxy`), **Conda** (`~/.condarc` `proxy_servers`), cURL (`~/.curlrc`), Wget (`~/.wgetrc`), Docker (`~/.docker/config.json`), **Gradle** (`~/.gradle/gradle.properties`), **Maven** (`~/.m2/settings.xml`), and **Go Modules** (`GOPROXY`).
  - Diagnoses **`/etc/hosts` pinned overrides**: detects if developer domains (`github.com`, `api.openai.com`, `huggingface.co`, etc.) are mapped to dead/stale IPs or blocked to `0.0.0.0` overriding DNS.
  - Diagnoses **environment variable casing conflicts** (`http_proxy != HTTP_PROXY`) where tools like cURL (lowercase only) and Go/Python behave inconsistently.
  - Diagnoses **AI Developer Gateways & Mirrors** (`OPENAI_BASE_URL`, `ANTHROPIC_BASE_URL`, `DEEPSEEK_BASE_URL`, `OLLAMA_HOST`, `HF_ENDPOINT`) for dead or unreachable local endpoints.
  - Detects **WSL2 Linux** environment and resolves Windows Host gateway IP to prevent `127.0.0.1` routing traps.
  - Detects **SOCKS5 vs SOCKS5h DNS leak risk** (`socks5h://` tunnels DNS through the daemon to prevent local DNS poisoning).
  - Verifies `NO_PROXY` localhost bypass list to prevent local dev servers (Vite, Next.js, Docker) from failing.
  - Detects DNS poisoning on `github.com`, `raw.githubusercontent.com`, `api.openai.com`, `api.anthropic.com`, `api.deepseek.com`, `proxy.golang.org`, and `registry.yarnpkg.com`.
  - Supports `--json` for machine-readable CI/CD and IDE diagnostics.
- 🩹 **1-Click Auto-Repair (`proxydoc fix`)**:
  - Clears dead Git, Cargo (`~/.cargo/config.toml`), NPM, **Yarn** (`~/.yarnrc` / `~/.yarnrc.yml`), Pip, **Conda** (`~/.condarc`), Docker (`~/.docker/config.json`), **Gradle** (`~/.gradle/gradle.properties`), and **Maven** (`~/.m2/settings.xml`) global proxies.
  - Automatically disables dead proxies in **cURL** (`~/.curlrc`) and **Wget** (`~/.wgetrc`) with automatic `.bak` backups.
  - Diagnoses dead SSH config ProxyCommand targets preventing Git SSH hangs.
  - Generates sync injection from system proxy to terminal with proper `NO_PROXY` bypass.
  - Flushes local DNS cache.
- ⚡ **AI & Dev Latency Checker (`proxydoc test`)**:
  - Fully concurrent testing to DeepSeek, OpenAI, Anthropic, GitHub, Google, Cloudflare, HuggingFace, **Docker Hub**, **crates.io**, **npm Registry**, **Yarn Registry**, and **Go Proxy**.
  - **Comparative Benchmark (`proxydoc test --compare` / `-c`)**: Tests Direct vs Proxy side-by-side, comparing latency, reachability, and egress IPs.
  - **Custom Target Benchmark (`proxydoc test -t <URL>`)**: Benchmarks custom API endpoints or internal corporate gateways.
  - **Configurable Timeout (`--timeout <SECS>`)**: Fine-tune probe deadlines for slow or satellite connections.
  - **Forced Direct Mode (`proxydoc test --direct` / `-d`)**: Tests connection directly bypassing any active proxy.
  - **Outbound Egress IP & Location Inspector**: instantly verifies your actual exit IP, country, and Cloudflare data center.
  - Supports `--json` for automation.
- 🔍 **Active Client Scanner (`proxydoc scan`)**: Detects Clash, Clash Verge, Shadowrocket, Sing-box, V2Ray, Surge, and CPA.
- 🟢 **Multi-Shell Switcher (`proxydoc on` / `proxydoc off`)**:
  - Toggle proxies in current shell session with syntax tailored for **Bash/Zsh**, **Fish**, and **PowerShell** (`--shell fish`, `--shell pwsh`).
  - Configure package managers & dev tools on demand: `--git`, `--cargo`, `--npm`, `--yarn`, `--pip`, `--conda`, `--docker`, `--gradle`, `--maven`, or `--all` (`-a`). Sets standard `no_proxy` bypass automatically.

---

## 🚀 Quick Start

### 1. Run Root-Cause Diagnostic
```bash
proxydoc doctor
```
Output:
```text
🩺 ==================== DEEP PROXY DOCTOR ==================== 🩺
   Running comprehensive root-cause network & environment diagnostics...

+---------------+-----------------------------+--------------------+------------------------------------------------+
| Category      | Inspection Target           | Current Value      | Health Diagnosis                               |
+---------------+-----------------------------+--------------------+------------------------------------------------+
| System Proxy  | macOS Web Proxy (scutil)    | 127.0.0.1:1082     | ● ACTIVE in OS (Port Open)                     |
| Sync Conflict | Terminal vs OS              | OS=ON | Term=OFF   | ▲ Out of sync! (Browser works, Terminal hangs) |
| Shell Env     | Terminal Proxy              | (not set)          | ○ Direct (Inherits nothing)                    |
| Git           | git config --global *.proxy | (not set)          | ○ Clean (Inherits Shell or Direct)             |
| DNS Check     | github.com                  | 198.18.0.39        | ● Normal resolution                            |
| DNS Check     | api.openai.com              | 198.18.0.96        | ● Normal resolution                            |
| Local Daemon  | Port 1082                   | Shadowrocket Mac   | ● OPEN & READY                                 |
+---------------+-----------------------------+--------------------+------------------------------------------------+

🚨 Detected Root-Cause Issues:
  1. macOS GUI proxy is active (127.0.0.1:1082), but your Terminal is DIRECT! Run `proxydoc fix` to sync.
```

### 2. Auto-Repair Broken Settings
```bash
proxydoc fix
```

### 3. Test Connectivity & Compare Direct vs Proxy
```bash
# Test active network connection (OpenAI, Anthropic, DeepSeek, GitHub, Google)
proxydoc test

# Compare Direct vs Proxy performance side-by-side
proxydoc test --compare

# Benchmark a custom endpoint with custom timeout
proxydoc test -t https://api.deepseek.com/models --timeout 8
```

### 4. Turn Proxy On / Off Across Shell & Dev Tools
```bash
# Sync terminal shell and all dev tools (Git, NPM, Pip, Docker)
eval $(proxydoc on -a)

# Turn off proxies across terminal and dev tools
eval $(proxydoc off -a)

# Fish shell users:
eval (proxydoc on --shell fish)
```

---

## 📦 Installation

```bash
curl -fsSL https://raw.githubusercontent.com/TrustDec/proxydoc/main/install.sh | bash
# Or with Cargo:
cargo install proxydoc
```

## 📄 License

MIT © [TrustDec](https://github.com/TrustDec)
