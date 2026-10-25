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
  - 全面体检 Git、SSH（`~/.ssh/config` ProxyCommand）、**GitHub CLI**（`~/.config/gh/config.yml` `http_proxy`）、**GitLab CLI (glab)**（`~/.config/glab-cli/config.yml` `glab_proxy`）、NPM、**Yarn**（`~/.yarnrc` / `~/.yarnrc.yml`）、**pnpm**（`~/.pnpmrc`）、**Bun**（`~/.bunfig.toml` `[install] proxy`）、Cargo（`[http] proxy`）、Pip（`global.proxy`）、**Astral `uv`**（`~/.config/uv/uv.toml` `http-proxy`）、**Conda 与 Mamba**（`~/.condarc` / `~/.mambarc` `proxy_servers`）、**NuGet (.NET)**（`~/.nuget/NuGet/NuGet.Config`）、**Gem (Ruby / CocoaPods)**（`~/.gemrc`）、**Subversion (SVN)**（`~/.subversion/servers`）、**Mercurial (Hg)**（`~/.hgrc`）、cURL（`~/.curlrc`）、Wget（`~/.wgetrc`）、Docker（`~/.docker/config.json`）、**Docker Daemon 引擎**（`/etc/docker/daemon.json` 与 `~/.docker/daemon.json`）、**Containerd 与 CRI-O (Kubernetes / nerdctl / 容器运行时守护进程)**（`/etc/systemd/system/containerd.service.d/http-proxy.conf`、`/etc/default/containerd`、`/etc/sysconfig/containerd`）、**Flatpak (OSTree / Linux 桌面应用管理器)**（`~/.local/share/flatpak/repo/config` 与 `/var/lib/flatpak/repo/config` `proxy=`）、**Snap (Canonical Snapcraft / Linux 应用包管理器)**（`/etc/systemd/system/snapd.service.d/http-proxy.conf`、`/var/lib/snapd/environment`、`snap set system proxy.http`）、**Ollama (本地大模型运行服务守护进程)**（`/etc/systemd/system/ollama.service.d/http-proxy.conf`、`~/.ollama/environment`、macOS LaunchAgent plist）、**Alpine Linux (apk)**（`/etc/apk/repositories`）、**Podman**（`~/.config/containers/containers.conf` `[engine] env`）、**Vagrant**（`~/.vagrant.d/Vagrantfile`）、**Chocolatey (Windows)**（`chocolatey.config` `proxy`）、**Colima (macOS 与 Linux)**（`~/.colima/default/colima.yaml` `env`）、**Minikube**（`~/.minikube/config/config.json` `http_proxy`）、**Spack (超算/科研科学计算)**（`~/.spack/config.yaml` `proxy:`）、**MacPorts (macOS 包管理器)**（`~/.macports/macports.conf` `proxy_http`）、**APT (Debian/Ubuntu/WSL 包管理器)**（`/etc/apt/apt.conf.d/99proxydoc.conf` / `~/.apt/apt.conf`）、**DNF 与 YUM (Fedora/RHEL/CentOS/Rocky/Alma)**（`/etc/dnf/dnf.conf` `proxy=`）、**Zypper (openSUSE/SLES)**（`/etc/zypp/zypp.conf` `proxy =`）、**Pacman (Arch Linux/Manjaro/SteamOS/MSYS2)**（`/etc/pacman.conf` `XferCommand`）、**FreeBSD pkg**（`/usr/local/etc/pkg.conf` `http_proxy`）、**Gentoo Portage**（`/etc/portage/make.conf` `http_proxy`）、**Gradle**（`~/.gradle/gradle.properties`）、**Maven**（`~/.m2/settings.xml`）、**Go 模块**（`go/env` 与 `GOPROXY`）、**Bazel**（`~/.bazelrc` `--action_env`）、**PHP Composer**（`config.json` `http-proxy`）、**Conan C/C++**（`~/.conan2/global.conf` 与 `~/.conan/conan.conf`）、**Scala (sbt)**（`~/.sbt/sbtopts`）、**Kubernetes (kubectl)**（`~/.kube/config`）、**Julia**（`~/.julia/config/startup.jl`）、**Nix**（`~/.config/nix/nix.conf` `http-proxy`）、**Haskell Cabal**（`~/.cabal/config` `proxy:`）、**Haskell Stack**（`~/.stack/config.yaml` `proxy:`）、**Hex (Elixir/Erlang/Gleam)**（`~/.hex/hex.config` `http_proxy`）、**R 语言包管理器**（`~/.Renviron` `http_proxy`）、**OCaml OPAM**（`~/.opam/config` `http-proxy:`）、**Clojure (Leiningen)**（`~/.lein/profiles.clj` `:http-proxy`）、**Perl (CPAN)**（`~/.cpan/CPAN/MyConfig.pm` `http_proxy`）、**Lua (LuaRocks)**（`~/.luarocks/config.lua` `http_proxy`）、**Nim (Nimble)**（`~/.config/nimble/nimble.ini` `http_proxy`）、**Erlang (Rebar3)**（`~/.config/rebar3/rebar.config` `http_proxy`）、**Scoop (Windows)**（`~/.config/scoop/config.json` `proxy`）、**Ruby (Bundler)**（`~/.bundle/config` `BUNDLE_HTTP_PROXY`）、**Google Cloud SDK (gcloud)**（`~/.config/gcloud/configurations/config_*` `[proxy]`）、**AWS CLI**（`~/.aws/config` `proxy`）及 **Microsoft Azure CLI (az)**（`~/.azure/config` `[proxy]`）代理配置；
  - 智能诊断 **Containerd 与 CRI-O (Kubernetes / nerdctl / 容器运行时守护进程) 代理**（`/etc/systemd/system/containerd.service.d/http-proxy.conf`、`/etc/default/containerd`、`/etc/sysconfig/containerd`），排查 `crictl pull` 超时与 Kubernetes 容器组 `ErrImagePull` / `ImagePullBackOff` 故障；
  - 智能诊断 **Ollama (本地 AI 模型服务守护进程) 代理**（`/etc/systemd/system/ollama.service.d/http-proxy.conf`、`~/.ollama/environment`、`~/Library/LaunchAgents/com.ollama.ollama.plist`），排查 `ollama pull` 与 `ollama run` 因守护进程死代理超时卡死；
  - 智能诊断 **Alpine Linux apk 包管理器仓库与镜像源**（`/etc/apk/repositories`），排查 `apk add` 与 `apk update` 因死镜像源阻塞；
  - 智能诊断 **Flatpak (Linux 桌面应用包管理器与运行环境) 仓库代理**（`~/.local/share/flatpak/repo/config`、`/var/lib/flatpak/repo/config`、`/etc/flatpak/remotes.d/`），排查 `flatpak install` 与应用更新因死代理失败；
  - 智能诊断 **Snap (Ubuntu / Debian / Fedora Linux 应用包管理器) 代理**（`/etc/systemd/system/snapd.service.d/http-proxy.conf`、`/var/lib/snapd/environment`、`snap set system proxy.http`），排查 `snap install` 与 `snap refresh` 因死代理卡死；
  - 智能诊断 **Portage (Gentoo、Funtoo、ChromeOS 开发环境) 代理**（`/etc/portage/make.conf`、`~/.config/portage/make.conf`、`PORTAGE_MAKE_CONF`），排查 `emerge --sync` 与源码下载因死代理失败；
  - 智能诊断 **Pacman (Arch Linux、Manjaro、SteamOS、MSYS2) 代理**（`/etc/pacman.conf`、`~/.config/pacman/pacman.conf`、`PACMAN_CONF`），排查 `pacman -Syu` 因死代理阻塞；
  - 智能诊断 **FreeBSD pkg 包管理器代理**（`/usr/local/etc/pkg.conf`、`/etc/pkg.conf`、`PKG_CONF`），排查 `pkg install` 与索引拉取因死代理失败；
  - 智能诊断 **Docker Daemon 引擎代理**（`/etc/docker/daemon.json`、`~/.docker/daemon.json`、`DOCKER_DAEMON_JSON`），排查 `docker pull` 与镜像构建因守护进程死代理超时；
  - 智能诊断 **DNF 与 YUM (Fedora/RHEL/CentOS/Rocky/Alma/Amazon Linux) 代理**（`/etc/dnf/dnf.conf`、`/etc/yum.conf`、`~/.config/dnf/dnf.conf`、`DNF_CONF`），排查 `dnf update` 与 `dnf install` 因死代理阻塞；
  - 智能诊断 **Zypper (openSUSE 与 SUSE Linux Enterprise) 代理**（`/etc/zypp/zypp.conf`、`~/.zypp/zypp.conf`、`ZYPP_CONF`），排查 `zypper refresh` 与 `zypper install` 失败；
  - 智能诊断 **MacPorts 包管理器代理**（`~/.macports/macports.conf`、`/opt/local/etc/macports/macports.conf`、`MACPORTS_CONF`），排查 `port install` 与同步因死代理失败；
  - 智能诊断 **APT (Debian/Ubuntu/WSL) 包管理器代理**（`/etc/apt/apt.conf.d/*`、`~/.apt/apt.conf`、`APT_CONFIG`），排查 `apt update` 与 `apt install` 因死代理阻塞；
  - 智能诊断 **Minikube 本地 Kubernetes 集群代理**（`~/.minikube/config/config.json`、`MINIKUBE_HOME`），排查 `minikube start`、镜像拉取与插件加载因死代理失败；
  - 智能诊断 **Spack 科学计算包管理器代理**（`~/.spack/config.yaml`、`SPACK_USER_CONFIG_PATH`），排查 `spack install` 与源码下载因死代理崩溃；
  - 智能诊断 **Chocolatey (Windows 包管理器) 代理**（`chocolatey.config`、`ChocolateyInstall`），排查 `choco install` 与 `choco upgrade` 失败；
  - 智能诊断 **Colima 容器运行时代理**（`~/.colima/default/colima.yaml`、`COLIMA_HOME`），排查 `colima start`、镜像拉取与容器网络超时；
  - 智能诊断 **Podman 容器引擎代理**（`~/.config/containers/containers.conf`、`CONTAINERS_CONF`），排查 `podman pull`、`podman build` 与容器网络因失效代理崩溃；
  - 智能诊断 **Vagrant 开发环境代理**（`~/.vagrant.d/Vagrantfile`、`VAGRANT_HOME`），排查 `vagrant box add` 与 `vagrant up` 虚拟机镜像下载超时；
  - 智能诊断 **Microsoft Azure CLI (az) 代理**（`~/.azure/config`、`AZURE_CONFIG_DIR`），排查 `az login`、`az account` 与 Azure 管理 SDK 因死代理崩溃；
  - 智能诊断 **Google Cloud SDK (gcloud) 代理**（`~/.config/gcloud/configurations/config_*`、`CLOUDSDK_CONFIG`），排查 `gcloud` 命令行、鉴权与 SDK 调用因死代理崩溃；
  - 智能诊断 **AWS CLI 代理**（`~/.aws/config`、`AWS_CONFIG_FILE`），排查 `aws s3`、`aws sts` 与 API 交互挂起；
  - 智能诊断 **Mamba 与 Micromamba 代理**（`~/.mambarc`、`MAMBARC`），排查 `mamba install` 失败；
  - 智能诊断 **Scoop (Windows 包管理器) 代理**（`~/.config/scoop/config.json`、`SCOOP_CONFIG`），排查 `scoop install` 与 `scoop update` 失败；
  - 智能诊断 **Ruby (Bundler) 包管理器代理**（`~/.bundle/config`、`BUNDLE_USER_CONFIG`），排查 `bundle install` 与 gem 模块下载失败；
  - 智能诊断 **Nim (Nimble) 包管理器代理**（`~/.config/nimble/nimble.ini`、`NIMBLE_DIR`），排查 `nimble install` 与依赖解析失败；
  - 智能诊断 **Erlang (Rebar3) 包管理器代理**（`~/.config/rebar3/rebar.config`、`REBAR_GLOBAL_CONFIG_DIR`），排查 `rebar3 compile`、`rebar3 update` 与 Hex 包拉取超时挂起；
  - 智能诊断 **Kubernetes Helm Chart 仓库镜像源**（`~/.config/helm/repositories.yaml`、`HELM_CONFIG_HOME`），排查 `helm repo update` 因仓库不可达导致的更新中断；
  - 智能诊断 **Lua (LuaRocks) 包管理器代理**（`~/.luarocks/config.lua`、`LUAROCKS_CONFIG`），排查 `luarocks install` 与 rockspec 模块构建失败；
  - 智能诊断 **C/C++ vcpkg 配置注册表与镜像源**（`~/.vcpkg/vcpkg-configuration.json`、`vcpkg-configuration.json`），排查 `vcpkg install` 因私有/失效注册表导致的构建失败；
  - 智能诊断 **Clojure Leiningen 代理**（`~/.lein/profiles.clj`、`LEIN_HOME`），排查 `lein deps` 与 `lein new` 依赖拉取失败；
  - 智能诊断 **Perl CPAN 包管理器代理**（`~/.cpan/CPAN/MyConfig.pm`、`PERL_CPAN_DIR`），排查 `cpan` 与 `cpanm` 模块安装超时；
  - 智能诊断 **C/C++ vcpkg 资产缓存与二进制镜像源**（`X_VCPKG_ASSET_SOURCES`、`VCPKG_BINARY_SOURCES`），排查 `vcpkg install` 资产拉取超时挂死；
  - 智能诊断 **R 语言包管理器代理与 CRAN 镜像**（`~/.Renviron`、`R_ENVIRON_USER`、`CRAN_MIRROR`），排查 `install.packages()`、`devtools` 与 `remotes` 下载安装中断；
  - 智能诊断 **OCaml OPAM 代理**（`~/.opam/config`、`OPAMROOT`），排查死代理导致的 `opam update` 与 `opam install` 超时挂起；
  - 智能诊断 **Terraform 与 OpenTofu Provider 网络镜像源**（`~/.terraformrc`、`~/.tofurc`、`TF_CLI_CONFIG_FILE`），排查 `terraform init` 因镜像失效导致的模块拉取中断；
  - 智能诊断 **Haskell Stack 代理**（`~/.stack/config.yaml` 的 `proxy:`），排查 GHC 编译器安装（`stack setup`）与依赖构建卡死；
  - 智能诊断 **Hex (Elixir/Erlang/Gleam) 代理**（`~/.hex/hex.config`、`HEX_HTTP_PROXY`），排查 `mix deps.get`、`rebar3` 与 `gleam deps download` 依赖下载失败；
  - 智能诊断 **Nix 包管理器代理与二进制缓存**（`~/.config/nix/nix.conf`、`/etc/nix/nix.conf` 的 `http-proxy` 与 `substituters`），杜绝因死代理或不可达镜像源导致 `nix-shell` 与 `nix build` 卡死挂起；
  - 智能诊断 **Haskell Cabal 代理**（`~/.cabal/config` 的 `proxy:`），排查死代理导致的 `cabal update` 与包构建失败；
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
  - 诊断 **AI 开发者网关与镜像**（`OPENAI_BASE_URL`、`OPENAI_API_BASE`、`ANTHROPIC_BASE_URL`、`GEMINI_BASE_URL`、`GOOGLE_AI_BASE_URL`、`DEEPSEEK_BASE_URL`、`DEEPSEEK_API_BASE`、`GROQ_BASE_URL`、`OPENROUTER_BASE_URL`、`MISTRAL_BASE_URL`、`PERPLEXITY_BASE_URL`、`COHERE_BASE_URL`、`XAI_BASE_URL`、`SILICONFLOW_BASE_URL`、`TOGETHER_BASE_URL`、`MOONSHOT_BASE_URL`、`DASHSCOPE_BASE_URL`、`DEEPINFRA_BASE_URL`、`CEREBRAS_BASE_URL`、`FIREWORKS_BASE_URL`、`ZHIPU_BASE_URL`、`MINIMAX_BASE_URL`、`STEPFUN_BASE_URL`、`LINGYIWANWU_BASE_URL`、`VOYAGE_BASE_URL`、`NOVITA_BASE_URL`、`HYPERBOLIC_BASE_URL`、`LAMBDA_BASE_URL`、`UPSTAGE_BASE_URL`、`AI21_BASE_URL`、`BAICHUAN_BASE_URL`、`SENSENOVA_BASE_URL`、`INTERNLM_BASE_URL`、`HUNYUAN_BASE_URL`、`BASETEN_BASE_URL`、`FRIENDLI_BASE_URL`、`LEPTON_BASE_URL`、`COPILOT_PROXY`、`SAMBANOVA_BASE_URL`、`REPLICATE_API_BASE`、`QIANFAN_BASE_URL`、`DEEPL_BASE_URL`、`ELEVENLABS_BASE_URL`、`PORTKEY_BASE_URL`、`LANGFUSE_BASE_URL`、`CARTESIA_BASE_URL`、`OLLAMA_HOST`、`HF_ENDPOINT`）是否指向不可达的幽灵端口；
  - 自动识别 **WSL2 Linux** 环境，定位 Windows 主机网关 IP，避免使用 `127.0.0.1` 导致的网络隔离死锁；
  - 检测 **SOCKS5 vs SOCKS5h DNS 泄漏风险**（`socks5h://` 走远端代理隧道解析 DNS，避免本地 DNS 污染与解析失败）；
  - 自动校验 `NO_PROXY` 本地回环绕行白名单，杜绝开启代理后本地开发服务（Vite、Next.js、Docker）卡死；
  - 检测 `github.com`、`raw.githubusercontent.com`、`api.openai.com`、`api.anthropic.com`、`api.deepseek.com`、`proxy.golang.org` 与 `registry.yarnpkg.com` 的本地 DNS 解析完整性；
  - 支持 `--json` 格式输出，方便集成 CI/CD、脚本与 IDE。
- 🩹 **一键自动修复 (`proxydoc fix`)**：
  - 自动清理 Git、**GitHub CLI**（`~/.config/gh/config.yml`）、**GitLab CLI**（`~/.config/glab-cli/config.yml`）、Cargo（`~/.cargo/config.toml`）、NPM、**Yarn**（`~/.yarnrc` / `~/.yarnrc.yml`）、**pnpm**（`~/.pnpmrc`）、**Bun**（`~/.bunfig.toml`）、Pip、**Astral `uv`**（`uv.toml`）、**Go 模块**（`go/env` / `GOPROXY`）、**Bazel**（`~/.bazelrc`）、**PHP Composer**（`config.json`）、**Conan C/C++**（`global.conf` / `conan.conf`）、**Scala (sbt)**（`~/.sbt/sbtopts`）、**Kubernetes (kubectl)**（`~/.kube/config`）、**Julia**（`~/.julia/config/startup.jl`）、**Nix**（`~/.config/nix/nix.conf`）、**Haskell Cabal**（`~/.cabal/config`）、**Haskell Stack**（`~/.stack/config.yaml`）、**Hex (Elixir/Gleam)**（`~/.hex/hex.config`）、**R (~/.Renviron)**、**OCaml OPAM (~/.opam/config)**、**Clojure Leiningen (~/.lein/profiles.clj)**、**Perl CPAN (~/.cpan/CPAN/MyConfig.pm)**、**LuaRocks (~/.luarocks/config.lua)**、**Nimble (~/.config/nimble/nimble.ini)**、**Rebar3 (~/.config/rebar3/rebar.config)**、**Scoop (~/.config/scoop/config.json)**、**Ruby Bundler (~/.bundle/config)**、**Google Cloud SDK (gcloud)**（`~/.config/gcloud/configurations/config_*`）、**AWS CLI**（`~/.aws/config`）、**Microsoft Azure CLI (az)**（`~/.azure/config`）、**Conda 与 Mamba**（`~/.condarc` / `~/.mambarc`）、**Podman**（`~/.config/containers/containers.conf`）、**Vagrant**（`~/.vagrant.d/Vagrantfile`）、**Chocolatey**（`chocolatey.config`）、**Colima**（`~/.colima/default/colima.yaml`）、**Minikube**（`~/.minikube/config/config.json`）、**Spack**（`~/.spack/config.yaml`）、**MacPorts**（`~/.macports/macports.conf`）、**APT**（`/etc/apt/apt.conf.d/99proxydoc.conf` / `~/.apt/apt.conf`）、**DNF 与 YUM**（`/etc/dnf/dnf.conf` / `~/.config/dnf/dnf.conf`）、**Zypper**（`/etc/zypp/zypp.conf` / `~/.zypp/zypp.conf`）、**Pacman**（`/etc/pacman.conf`）、**FreeBSD pkg**（`/usr/local/etc/pkg.conf`）、**Gentoo Portage**（`/etc/portage/make.conf`）、**Docker Daemon**（`/etc/docker/daemon.json`）、**Containerd 与 CRI-O**（`/etc/systemd/system/containerd.service.d/http-proxy.conf` / `/etc/default/containerd`）、**Flatpak**（`~/.local/share/flatpak/repo/config`）、**Snap**（`/etc/systemd/system/snapd.service.d/http-proxy.conf` / `snap set system`）、**Ollama 守护进程**（`/etc/systemd/system/ollama.service.d/http-proxy.conf` / `~/.ollama/environment`）、**NuGet (.NET)**（`NuGet.Config`）、**Gem (Ruby)**（`~/.gemrc`）、**Subversion (SVN)**（`servers`）、**Mercurial (Hg)**（`~/.hgrc`）、Docker（`~/.docker/config.json`）、Gradle（`~/.gradle/gradle.properties`）、Maven、**JetBrains / Android Studio**（`proxy.settings.xml`）及 **VS Code / Cursor IDE**（`settings.json`）残留的失效死代理（涵盖 71 个自动化修复步骤）；
  - 自动注释并禁用 `/etc/apk/repositories` 中失效不可达的 Alpine Linux apk 镜像源，自动生成 `.bak` 安全备份；
  - 自动注释并禁用 `repositories.yaml` 中失效不可达的 Kubernetes Helm Chart 仓库镜像源，自动生成 `.bak` 安全备份；
  - 自动清理 `vcpkg-configuration.json` 中失效不可达的 registry 镜像源并恢复官方默认源，自动生成 `.bak` 安全备份；
  - 自动注释并禁用 `~/.terraformrc` 与 `~/.tofurc` 中失效不可达的 Provider 网络镜像源，自动生成 `.bak` 安全备份；
  - 自动检测并清理 Swift Package Manager（`~/.swiftpm/configuration/mirrors.json`）中失效卡死的包镜像源，自动生成 `.bak` 备份；
  - 自动检测并一键迁移 `~/.npmrc`、`~/.yarnrc`、`~/.yarnrc.yml`、`~/.pnpmrc` 中已废弃停服的 `registry.npm.taobao.org` -> 官方可用 `https://registry.npmmirror.com`，自动备份安全无忧；
  - 自动清理 Git 中指向不存在证书路径的失效 `http.sslCAInfo` 配置；
  - 自动备份（`.bak`）并注释禁用 **cURL**（`~/.curlrc`）与 **Wget**（`~/.wgetrc`）中的死代理配置；
  - 检测 SSH 配置中导致 `git clone git@...` 挂起的失效 ProxyCommand；
  - 将系统有效代理一键生成同步注入到终端，并自动配置标准 `NO_PROXY` 绕行；
  - 跨平台刷新本地 DNS 解析缓存（支持 **macOS**、**Windows**、**Linux**）。
- ⚡ **AI 接口并发测速与双向对照 (`proxydoc test`)**：
  - 全异步并发秒级测试对 DeepSeek、OpenAI、Anthropic、**Google Gemini**、**xAI (Grok)**、**SiliconFlow (硅基流动)**、**Together AI**、**Moonshot (月之暗面 Kimi)**、**Aliyun DashScope (阿里通义千问)**、**DeepInfra**、**Cerebras**、**Fireworks AI**、**Zhipu AI (智谱清言 GLM)**、**MiniMax**、**StepFun (阶跃星辰 Step-2)**、**01.AI (零一万物 Yi)**、**Voyage AI**、**Novita AI**、**Hyperbolic API**、**Lambda Labs API**、**Upstage (Solar) API**、**Ai21 Labs (Jamba) API**、**Baichuan AI (百川智能)**、**SenseNova (商汤日日新)**、**Tencent Hunyuan (腾讯混元)**、**Baseten API**、**FriendliAI API**、**Lepton AI API**、**Nebius AI Studio API**、**Chutes AI API**、**Infini-AI API**、**AIML API**、**Crusoe Cloud API**、**Modal**、**NVIDIA NIM API**、**ByteDance Doubao (字节跳动火山引擎豆包 Ark) API**、**Fal.ai**、**RunPod API**、**Helicone AI Gateway**、**Kling AI (快手可灵) API**、**ElevenLabs API**、**Cartesia Voice AI API**、**Portkey AI Gateway**、**Langfuse API**、**LiveKit Cloud**、**Deepgram Speech AI API**、**Weights & Biases (W&B) API**、**Pinecone Vector DB**、**Tavily AI Search API**、**Exa AI Search API**、**Qdrant Cloud**、**AssemblyAI Speech API**、**Jina AI API**、**Firecrawl API**、**Weaviate Cloud**、**Chroma Vector DB**、**GitHub Copilot API**、**SambaNova API**、**Replicate API**、**Baidu Qianfan (百度千帆/文心一言)**、**DeepL API**、**Ollama**、**OpenRouter**、**Groq**、**Mistral API**、**Perplexity API**、**Cohere API**、GitHub、Google、Cloudflare、HuggingFace、**Docker Hub**、**crates.io**、**npm Registry**、**Yarn Registry**、**Go Proxy**、**Stackage (Haskell)**、**Hex.pm Registry**、**CRAN (R Archive)**、**OPAM Registry (OCaml)**、**Clojars (Clojure)**、**MetaCPAN (Perl)**、**LuaRocks Registry**、**Nimble Directory**、**Hex PM (Erlang/Hex)**、**RubyGems Registry**、**Cursor AI API**、**LanceDB Cloud**、**Maven Central**、**NuGet Gallery**、**Packagist**、**Vapi AI API**、**Braintrust API**、**LlamaIndex Cloud**、**Mem0 AI API**、**Supabase API**、**Dify Cloud API**、**Unstructured.io API**、**Upstash API**、**Vercel Cloud API**、**Resend API**、**E2B Sandbox API**、**PostHog Analytics API**、**Pydantic Logfire API**、**Railway Cloud API**、**Sentry API**、**Neon Database API**、**Render Cloud API**、**Fly.io Machines API**、**Turso Database API**、**Clerk Auth API**、**Convex Cloud**、**Inngest API**、**Trigger.dev Cloud**、**Civitai API**、**Axiom API**、**Stripe API**、**Cloudflare API**、**Dub.co API**、**Koyeb Cloud API**、**Novu API**、**Linear API**、**Codeium API**、**Datadog API**、**Sanity API**、**Meilisearch Cloud**、**PlanetScale API**、**Better Stack API**、**Mintlify API**、**Val Town API**、**Hetzner Cloud API**、**GitLab API**、**DigitalOcean API**、**Netlify API**、**Scaleway Cloud API**、**Linode (Akamai) API**、**Bitbucket API**、**Vultr Cloud API**、**Fastly API**、**Gitee API**、**Deno Deploy API**、**Ollama Registry**、**GHCR (GitHub Packages)**、**GitHub Releases (Objects)**、**Tailscale API**、**Replit API**、**Quay.io (红帽容器镜像仓库)**、**Terraform Registry**、**Ngrok API**、**Grafana Cloud API** 与 **Postman API** 的真实延迟；
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
  - 支持 `--git`、`--cargo`、`--npm`、`--yarn`、`--pnpm`、`--bun`、`--pip`、`--uv`、`--go`、`--bazel`、`--composer`、`--conan`、`--sbt`、`--kube`、`--glab`、`--julia`、`--nix`、`--cabal`、`--stack`、`--hex`、`--r`、`--opam`、`--lein`、`--cpan`、`--luarocks`、`--nimble`、`--rebar3`、`--scoop`、`--bundle`、`--gcloud`、`--aws`、`--azure`（`--az`）、`--mamba`（`--micromamba`）、`--podman`、`--vagrant`、`--choco`（`--chocolatey`）、`--colima`、`--minikube`、`--spack`、`--macports`、`--apt`、`--dnf`、`--zypper`、`--pacman`、`--pkg`、`--portage`（`--emerge`）、`--docker-daemon`（`--daemon`）、`--flatpak`（`--ostree`）、`--snap`（`--snapd`）、`--ollama`（`--ollamad`）、`--containerd`（`--crictl`、`--cri`）、`--conda`、`--docker`、`--gradle`、`--maven`、`--vscode`、`--gh`、`--jetbrains`、`--nuget`、`--gem`、`--svn`、`--hg`、`--curl`、`--wget` 或 `-a, --all` 一键联动包管理器、开发工具与容器并注入 `no_proxy`。

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
