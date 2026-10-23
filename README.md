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
  - Detects DNS poisoning on `github.com`, `raw.githubusercontent.com`, `api.openai.com`, and `api.anthropic.com`.
- 🩹 **1-Click Auto-Repair (`proxydoc fix`)**:
  - Clears dead Git global proxies.
  - Generates sync injection from system proxy to terminal.
  - Flushes local DNS cache.
- ⚡ **AI & Dev Latency Checker (`proxydoc test`)**: Concurrently tests real latency to OpenAI, Anthropic, GitHub, Google, and HuggingFace.
- 🔍 **Active Client Scanner (`proxydoc scan`)**: Detects Clash, Clash Verge, Shadowrocket, Sing-box, V2Ray, Surge, and CPA.
- 🟢 **Instant Shell Switcher (`proxydoc on` / `proxydoc off`)**: Toggle proxies in current shell session cleanly.

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

### 3. Test Connectivity to AI APIs
```bash
proxydoc test
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
