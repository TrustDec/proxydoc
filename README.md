# proxydoc 🩺

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/proxydoc.svg)](https://crates.io/crates/proxydoc)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![GitHub Stars](https://img.shields.io/github/stars/TrustDec/proxydoc.svg?style=social)](https://github.com/TrustDec/proxydoc)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)]()

**All-in-one developer proxy environment diagnostics, latency tester & manager.**  
*Fix broken terminal networks, dead proxy ports, and disconnected AI APIs in 1 second.*

[English](README.md) | [简体中文](README_zh.md)

</div>

---

## ⚡ The Pain It Solves

If you are a developer in a restricted network region, you must have suffered from:
- ❌ Browser proxy works, but `git clone` or `brew update` hangs forever.
- ❌ Terminal throws `curl: (7) Failed to connect to 127.0.0.1 port 7890: Connection refused` because you forgot your proxy app was closed or changed ports.
- ❌ Shell env (`http_proxy`), Git global config, NPM/Yarn config, Docker daemon, and AI coding tools (Claude Code, Cursor, Codex) each require separate proxy configurations.

**`proxydoc` gives you a 1-second full diagnostic, automated port detection, and instantaneous on/off toggles across your whole toolchain.**

---

## ✨ Features

- 🩺 **Full System Diagnostics (`proxydoc doctor`)**: Checks Shell variables, Git global configs, NPM, Homebrew, and listening proxy ports in a neat terminal table.
- 🔍 **Active Client Scanner (`proxydoc scan`)**: Automatically scans and identifies Clash, Clash Verge, Shadowrocket, Sing-box, V2Ray, Surge, and CLI-Proxy-API daemon ports.
- ⚡ **AI & Dev Latency Checker (`proxydoc test`)**: Tests real-time connectivity and latency to **GitHub, OpenAI, Anthropic, Google, Cloudflare, and HuggingFace**.
- 🟢 **Instant Shell Switcher (`proxydoc on` / `proxydoc off`)**: Sets or clears proxy across current shell session, Git, and NPM cleanly.
- 🚀 **Zero Dependencies**: Pure Rust single static binary, blazing fast (< 10ms execution time).

---

## 📦 Installation

### Pre-built Binaries (macOS / Linux)
```bash
curl -fsSL https://raw.githubusercontent.com/TrustDec/proxydoc/main/install.sh | bash
```

### With Cargo
```bash
cargo install proxydoc
```

---

## 🚀 Quick Start

### 1. Run Doctor Diagnostic
```bash
proxydoc doctor
```
Output:
```text
🩺 ==================== PROXY DOCTOR ==================== 🩺
   Running full development environment diagnostics...

+--------------+-----------------------------+--------------------------------------+----------------------------+
| Category     | Target / Config             | Configured Value                     | Health Status              |
+--------------+-----------------------------+--------------------------------------+----------------------------+
| Shell Env    | Terminal Proxy              | (not set)                            | ○ Direct (Normal)          |
| Git          | git config --global *.proxy | (not set)                            | ○ Inherits Shell or Direct |
| NPM          | npm config proxy            | (not set)                            | ○ Direct                   |
| Homebrew     | Mirror Domain               | Official (default)                   | ○ Uses Shell Proxy         |
| Local Daemon | Port 7890                   | Clash / Clash Verge (Mixed/HTTP)     | ● OPEN & LISTENING         |
+--------------+-----------------------------+--------------------------------------+----------------------------+
```

### 2. Test Connection & Latency to AI APIs
```bash
# Test through current active shell proxy
proxydoc test

# Or test through a specific proxy
proxydoc test --proxy socks5://127.0.0.1:1080
```

### 3. Scan Local Proxy Ports
```bash
proxydoc scan
```

### 4. Turn On / Off Proxy in Terminal
Add this to your `~/.zshrc` or `~/.bashrc`:
```bash
eval "$(proxydoc init)"
```

Then you get super-short aliases:
```bash
pon          # Enable terminal proxy (auto selects active port)
pon 7890     # Enable with custom port
pon --git    # Also set git global proxy
poff         # Turn off terminal proxy
poff --git   # Clean git proxy as well
```

---

## 🤝 Contributing

PRs, bug reports, and suggestions are warmly welcomed! Please star ⭐ this repository if you find it helpful.

## 📄 License

MIT © [TrustDec](https://github.com/TrustDec)
