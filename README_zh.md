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
  - **跨平台系统代理检测**：自动对比 **macOS** 系统代理（`scutil --proxy`）、**Windows** 注册表代理与 **Linux** 桌面（`gsettings`）与当前终端 Shell 状态，揪出“浏览器能上、终端上不了”的脱节问题；
  - 深度校验已配置端口是否真实存活，杜绝幽灵死代理；
  - 全面体检 Git、SSH（`~/.ssh/config` ProxyCommand）、**GitHub CLI**（`~/.config/gh/config.yml` `http_proxy`）、**GitLab CLI (glab)**（`~/.config/glab-cli/config.yml` `glab_proxy`）、NPM、**Yarn**（`~/.yarnrc` / `~/.yarnrc.yml`）、**pnpm**（`~/.pnpmrc`）、**Bun**（`~/.bunfig.toml` `[install] proxy`）、Cargo（`[http] proxy`）、Pip（`global.proxy`）、**Astral `uv`**（`~/.config/uv/uv.toml` `http-proxy`）、**Conda**（`~/.condarc` `proxy_servers`）、**NuGet (.NET)**（`~/.nuget/NuGet/NuGet.Config`）、**Gem (Ruby / CocoaPods)**（`~/.gemrc`）、**Subversion (SVN)**（`~/.subversion/servers`）、**Mercurial (Hg)**（`~/.hgrc`）、cURL（`~/.curlrc`）、Wget（`~/.wgetrc`）、Docker（`~/.docker/config.json`）、**Gradle**（`~/.gradle/gradle.properties`）、**Maven**（`~/.m2/settings.xml`）、**Go 模块**（`go/env` 与 `GOPROXY`）、**Bazel**（`~/.bazelrc` `--action_env`）、**PHP Composer**（`config.json` `http-proxy`）、**Conan C/C++**（`~/.conan2/global.conf` 与 `~/.conan/conan.conf`）、**Scala (sbt)**（`~/.sbt/sbtopts`）、**Kubernetes (kubectl)**（`~/.kube/config`）及 **Julia**（`~/.julia/config/startup.jl`）代理配置；
  - 智能诊断 **Julia 启动脚本代理**（`~/.julia/config/startup.jl` 的 `ENV["HTTP_PROXY"]`、`ENV["HTTPS_PROXY"]` 与 `ENV["all_proxy"]`），杜绝因死代理导致科学计算包安装（`Pkg.add` / `Pkg.update`）卡死挂起；
  - 智能诊断 **GitLab CLI (glab) 代理**（`~/.config/glab-cli/config.yml` 的 `glab_proxy`），排查 Merge Request、流水线与 API 交互超时；
  - 智能诊断 **Nix 二进制缓存镜像源**（`~/.config/nix/nix.conf`、`/etc/nix/nix.conf` 的 `substituters`），防止 `nix-shell` 与 `nix build` 因镜像不可达超时；
  - 智能诊断 **Swift Package Manager 镜像配置**（`~/.swiftpm/configuration/mirrors.json`）与 CocoaPods 镜像源，排查 iOS/macOS 依赖解析失败；
  - 智能诊断 **Scala Build Tool (sbt) 代理**（`~/.sbt/sbtopts` 的 `-Dhttp.proxyHost`、`-Dhttp.proxyPort`、`-Dhttps.proxyHost`、`-Dhttps.proxyPort`），排查 JVM 死代理导致的 sbt 构建与插件解析卡死；
  - 智能诊断 **Kubernetes (kubectl) 集群代理配置**（`~/.kube/config` 的 `clusters[].cluster.proxy-url`），杜绝因死代理端口导致 `kubectl` 与 `k9s` 全面挂起；
  - 智能诊断 **Conan C/C++ 包管理器代理**（`~/.conan2/global.conf` 的 `core.net.http:proxies`、`~/.conan/conan.conf` 的 `[proxies]`），排查死代理导致的 C/C++ 第三方库下载卡死与编译中断；
  - 智能诊断 **PHP Composer 代理与 Packagist 镜像源**（`~/.composer/config.json`、`~/.config/composer/config.json`）是否存活或因死代理与失效镜像源导致安装报错；
  - 智能诊断 **Bazel 代理配置**（`~/.bazelrc` 的 `--action_env=HTTP_PROXY`），避免因死代理导致构建依赖拉取卡死；
  - 智能诊断 **Swift Package Manager 与 CocoaPods 镜像源**（`CP_REPO_DIR_URL`、`SWIFT_PACKAGE_REGISTRY_URL`、`SWIFT_PACKAGE_COLLECTION_URL`），防止 iOS/macOS 依赖拉取因失效镜像超时中断；
  - 智能诊断 **Flutter / Dart 环境与镜像源**（`PUB_HOSTED_URL`、`FLUTTER_STORAGE_BASE_URL`），防止依赖包下载与引擎拉取因失效镜像超时中断；
  - 智能诊断 **Rustup 下载镜像**（`RUSTUP_DIST_SERVER`、`RUSTUP_UPDATE_ROOT`）连通性，防止 Rust 工具链更新卡死；
  - 智能诊断 **Julia (Pkg) 镜像源**（`JULIA_PKG_SERVER`），排查科学计算与 AI 依赖安装超时；
  - 智能诊断 **Deno 代理环境变量**（`DENO_HTTP_PROXY`、`DENO_HTTPS_PROXY`）连通性；
  - 智能诊断 **Node / JS 生态 Registry 镜像源**（`~/.npmrc`、`~/.yarnrc`、`~/.yarnrc.yml`、`~/.pnpmrc`、`NPM_CONFIG_REGISTRY`）：检测不可达镜像源并深度预警已停服下线的废弃域名（`registry.npm.taobao.org`）；
  - 智能诊断 **Homebrew 镜像源连通性**（`HOMEBREW_BOTTLE_DOMAIN`、`HOMEBREW_API_DOMAIN`、`HOMEBREW_BREW_GIT_REMOTE`、`HOMEBREW_CORE_GIT_REMOTE`），防止 `brew install` / `brew update` 因失效镜像卡死；
  - 智能诊断 **JetBrains 与 Android Studio IDE 全家桶**（IntelliJ IDEA、PyCharm、WebStorm、Android Studio、CLion、RustRover、GoLand 的 `options/proxy.settings.xml`）：检测死代理端口与失效 PAC URL 导致的插件市场超时、AI 助手连接失败与 Gradle/SDK 同步故障；
  - 智能诊断 **VS Code、Cursor、Windsurf 与 VSCodium IDE 配置**（`settings.json` 的 `http.proxy`、`http.proxyStrictSSL`、`http.proxySupport`）：精准排查死代理端口与禁用严格 SSL 导致的 GitHub Copilot、Cursor AI 与插件市场连接超时或崩溃；
  - 智能诊断 **Git SSL 与证书完整性**：预警 `http.sslVerify false`（全局禁用证书验证的中间人攻击风险），检测 `http.sslCAInfo` 是否指向不存在的文件导致 Git 崩溃；
  - 智能诊断 **Node.js TLS 安全配置**（`NODE_TLS_REJECT_UNAUTHORIZED='0'`）与 CA 证书环境变量覆盖；
  - 智能诊断 **Cargo Crates.io 替换镜像源**（`[source.crates-io] replace-with`）与 **Rustup 下载镜像**（`RUSTUP_DIST_SERVER`、`RUSTUP_UPDATE_ROOT`）是否存活或死链；
  - 智能诊断 **Python / Astral `uv` 镜像源与代理**（`UV_INDEX_URL`、`UV_EXTRA_INDEX_URL`、`PIP_INDEX_URL`、`UV_HTTP_PROXY`、`uv.toml`）连通性，防止包安装因镜像失效或死代理卡死；
  - 智能排查 **`/etc/hosts` 本地静态劫持与失效死 IP**：精准探测开发域名（`github.com`、`api.openai.com`、`huggingface.co` 等）是否被绑定到已下线死 IP 或误配置为 `0.0.0.0` 导致网络挂死；
  - 智能诊断 **环境变量大小写冲突**（`http_proxy != HTTP_PROXY`），杜绝因 cURL（仅支持小写）与 Go/Python 读取习惯不同导致的玄学报错；
  - 诊断 **AI 开发者网关与镜像**（`OPENAI_BASE_URL`、`OPENAI_API_BASE`、`ANTHROPIC_BASE_URL`、`GEMINI_BASE_URL`、`GOOGLE_AI_BASE_URL`、`DEEPSEEK_BASE_URL`、`DEEPSEEK_API_BASE`、`GROQ_BASE_URL`、`OPENROUTER_BASE_URL`、`MISTRAL_BASE_URL`、`PERPLEXITY_BASE_URL`、`COHERE_BASE_URL`、`XAI_BASE_URL`、`SILICONFLOW_BASE_URL`、`TOGETHER_BASE_URL`、`MOONSHOT_BASE_URL`、`DASHSCOPE_BASE_URL`、`DEEPINFRA_BASE_URL`、`CEREBRAS_BASE_URL`、`FIREWORKS_BASE_URL`、`ZHIPU_BASE_URL`、`MINIMAX_BASE_URL`、`STEPFUN_BASE_URL`、`LINGYIWANWU_BASE_URL`、`VOYAGE_BASE_URL`、`NOVITA_BASE_URL`、`HYPERBOLIC_BASE_URL`、`LAMBDA_BASE_URL`、`UPSTAGE_BASE_URL`、`AI21_BASE_URL`、`BAICHUAN_BASE_URL`、`SENSENOVA_BASE_URL`、`INTERNLM_BASE_URL`、`HUNYUAN_BASE_URL`、`BASETEN_BASE_URL`、`FRIENDLI_BASE_URL`、`LEPTON_BASE_URL`、`COPILOT_PROXY`、`SAMBANOVA_BASE_URL`、`REPLICATE_API_BASE`、`QIANFAN_BASE_URL`、`DEEPL_BASE_URL`、`OLLAMA_HOST`、`HF_ENDPOINT`）是否指向不可达的幽灵端口；
  - 自动识别 **WSL2 Linux** 环境，定位 Windows 主机网关 IP，避免使用 `127.0.0.1` 导致的网络隔离死锁；
  - 检测 **SOCKS5 vs SOCKS5h DNS 泄漏风险**（`socks5h://` 走远端代理隧道解析 DNS，避免本地 DNS 污染与解析失败）；
  - 自动校验 `NO_PROXY` 本地回环绕行白名单，杜绝开启代理后本地开发服务（Vite、Next.js、Docker）卡死；
  - 检测 `github.com`、`raw.githubusercontent.com`、`api.openai.com`、`api.anthropic.com`、`api.deepseek.com`、`proxy.golang.org` 与 `registry.yarnpkg.com` 的本地 DNS 解析完整性；
  - 支持 `--json` 格式输出，方便集成 CI/CD、脚本与 IDE。
- 🩹 **一键自动修复 (`proxydoc fix`)**：
  - 自动清理 Git、**GitHub CLI**（`~/.config/gh/config.yml`）、**GitLab CLI**（`~/.config/glab-cli/config.yml`）、Cargo（`~/.cargo/config.toml`）、NPM、**Yarn**（`~/.yarnrc` / `~/.yarnrc.yml`）、**pnpm**（`~/.pnpmrc`）、**Bun**（`~/.bunfig.toml`）、Pip、**Astral `uv`**（`uv.toml`）、**Go 模块**（`go/env` / `GOPROXY`）、**Bazel**（`~/.bazelrc`）、**PHP Composer**（`config.json`）、**Conan C/C++**（`global.conf` / `conan.conf`）、**Scala (sbt)**（`~/.sbt/sbtopts`）、**Kubernetes (kubectl)**（`~/.kube/config`）、**Julia**（`~/.julia/config/startup.jl`）、**Conda**（`~/.condarc`）、**NuGet (.NET)**（`NuGet.Config`）、**Gem (Ruby)**（`~/.gemrc`）、**Subversion (SVN)**（`servers`）、**Mercurial (Hg)**（`~/.hgrc`）、Docker（`~/.docker/config.json`）、Gradle（`~/.gradle/gradle.properties`）、Maven、**JetBrains / Android Studio**（`proxy.settings.xml`）及 **VS Code / Cursor IDE**（`settings.json`）残留的失效死代理（涵盖 31 个自动化修复步骤）；
  - 自动检测并一键迁移 `~/.npmrc`、`~/.yarnrc`、`~/.yarnrc.yml`、`~/.pnpmrc` 中已废弃停服的 `registry.npm.taobao.org` -> 官方可用 `https://registry.npmmirror.com`，自动备份安全无忧；
  - 自动清理 Git 中指向不存在证书路径的失效 `http.sslCAInfo` 配置；
  - 自动备份（`.bak`）并注释禁用 **cURL**（`~/.curlrc`）与 **Wget**（`~/.wgetrc`）中的死代理配置；
  - 检测 SSH 配置中导致 `git clone git@...` 挂起的失效 ProxyCommand；
  - 将系统有效代理一键生成同步注入到终端，并自动配置标准 `NO_PROXY` 绕行；
  - 跨平台刷新本地 DNS 解析缓存（支持 **macOS**、**Windows**、**Linux**）。
- ⚡ **AI 接口并发测速与双向对照 (`proxydoc test`)**：
  - 全异步并发秒级测试对 DeepSeek、OpenAI、Anthropic、**Google Gemini**、**xAI (Grok)**、**SiliconFlow (硅基流动)**、**Together AI**、**Moonshot (月之暗面 Kimi)**、**Aliyun DashScope (阿里通义千问)**、**DeepInfra**、**Cerebras**、**Fireworks AI**、**Zhipu AI (智谱清言 GLM)**、**MiniMax**、**StepFun (阶跃星辰 Step-2)**、**01.AI (零一万物 Yi)**、**Voyage AI**、**Novita AI**、**Hyperbolic API**、**Lambda Labs API**、**Upstage (Solar) API**、**Ai21 Labs (Jamba) API**、**Baichuan AI (百川智能)**、**SenseNova (商汤日日新)**、**Tencent Hunyuan (腾讯混元)**、**Baseten API**、**FriendliAI API**、**Lepton AI API**、**Nebius AI Studio API**、**Chutes AI API**、**Infini-AI API**、**AIML API**、**Crusoe Cloud API**、**Modal**、**GitHub Copilot API**、**SambaNova API**、**Replicate API**、**Baidu Qianfan (百度千帆/文心一言)**、**DeepL API**、**Ollama**、**OpenRouter**、**Groq**、**Mistral API**、**Perplexity API**、**Cohere API**、GitHub、Google、Cloudflare、HuggingFace、**Docker Hub**、**crates.io**、**npm Registry**、**Yarn Registry** 与 **Go Proxy** 的真实延迟；
  - **TLS/SSL 拦截排查**：精准识别公司内网代理 MITM 证书劫持及自签名未授信根证书错误；
  - **直连与代理双向对照 (`proxydoc test --compare` / `-c`)**：同屏并发对比 Direct 与 Proxy 的延迟差与联通状态，直观看出代理加速倍率；
  - **自定义接口测速 (`proxydoc test -t <URL>`)**：一键测试自定义 API、自建网关或私有服务在代理环境下的连通与延迟；
  - **自定义超时时间 (`--timeout <SECS>`)**：弱网或卫星网络下灵活配置超时阈值；
  - **强制直连测试 (`proxydoc test --direct` / `-d`)**：忽略当前环境代理，测试裸连状态；
  - **出口 IP 与位置探测**：自动提取流量真实公网出口 IP、归属国家（如 US/SG/JP）与 Cloudflare 边缘节点，一眼看清节点是否生效；
  - 支持 `--json` 格式导出。
- 🔍 **本地代理客户端扫描 (`proxydoc scan`)**：智能探测 Shadowrocket、Clash、Sing-box、Surge、CPA 等端口（支持 `--json`）。
- 🟢 **多 Shell 极简切换 (`proxydoc on` / `proxydoc off`)**：
  - 支持 **Bash/Zsh**、**Fish** 与 **PowerShell** 专属语法导出（`--shell fish`、`--shell pwsh`）；
  - 支持 `--git`、`--cargo`、`--npm`、`--yarn`、`--pnpm`、`--bun`、`--pip`、`--uv`、`--go`、`--bazel`、`--composer`、`--conan`、`--sbt`、`--kube`、`--glab`、`--julia`、`--conda`、`--docker`、`--gradle`、`--maven`、`--vscode`、`--gh`、`--jetbrains`、`--nuget`、`--gem`、`--svn`、`--hg`、`--curl`、`--wget` 或 `-a, --all` 一键联动包管理器、开发工具与容器并注入 `no_proxy`。

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
