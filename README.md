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
  - **Cross-Platform OS System Proxy Detection**: Compares **macOS** Web Proxy (`scutil --proxy`), **Windows** Internet Settings (Registry), and **Linux** Desktop (`gsettings`) vs Terminal Shell environment.
  - Verifies whether configured proxy ports are actually ALIVE or DEAD.
  - Inspects Git, SSH (`~/.ssh/config` ProxyCommand), **GitHub CLI** (`~/.config/gh/config.yml` `http_proxy`), **GitLab CLI (glab)** (`~/.config/glab-cli/config.yml` `glab_proxy`), NPM, **Yarn** (`~/.yarnrc` / `~/.yarnrc.yml`), **pnpm** (`~/.pnpmrc`), **Bun** (`~/.bunfig.toml` `[install] proxy`), Cargo (`[http] proxy`), Pip (`global.proxy`), **Astral `uv`** (`~/.config/uv/uv.toml` `http-proxy`), **Conda & Mamba** (`~/.condarc` / `~/.mambarc` `proxy_servers`), **NuGet (.NET)** (`~/.nuget/NuGet/NuGet.Config`), **Gem (Ruby / CocoaPods)** (`~/.gemrc`), **Subversion (SVN)** (`~/.subversion/servers`), **Mercurial (Hg)** (`~/.hgrc`), cURL (`~/.curlrc`), Wget (`~/.wgetrc`), Docker (`~/.docker/config.json`), **Docker Daemon Engine** (`/etc/docker/daemon.json` & `~/.docker/daemon.json`), **Flatpak (OSTree / Linux Desktop Application Manager)** (`~/.local/share/flatpak/repo/config` & `/var/lib/flatpak/repo/config` `proxy=`), **Snap (Canonical Snapcraft / Linux Application Package Manager)** (`/etc/systemd/system/snapd.service.d/http-proxy.conf`, `/var/lib/snapd/environment`, `snap set system proxy.http`), **Ollama (Local AI Runner Daemon)** (`/etc/systemd/system/ollama.service.d/http-proxy.conf`, `~/.ollama/environment`, launchd plist), **Alpine Linux (apk)** (`/etc/apk/repositories`), **Podman** (`~/.config/containers/containers.conf` `[engine] env`), **Vagrant** (`~/.vagrant.d/Vagrantfile`), **Chocolatey (Windows)** (`chocolatey.config` `proxy`), **Colima (macOS & Linux)** (`~/.colima/default/colima.yaml` `env`), **Minikube** (`~/.minikube/config/config.json` `http_proxy`), **Spack (HPC/Scientific)** (`~/.spack/config.yaml` `proxy:`), **MacPorts (macOS)** (`~/.macports/macports.conf` `proxy_http`), **APT (Debian/Ubuntu/WSL)** (`/etc/apt/apt.conf.d/99proxydoc.conf` / `~/.apt/apt.conf`), **DNF & YUM (Fedora/RHEL/CentOS/Rocky/Alma)** (`/etc/dnf/dnf.conf` `proxy=`), **Zypper (openSUSE/SLES)** (`/etc/zypp/zypp.conf` `proxy =`), **Pacman (Arch Linux/Manjaro/SteamOS/MSYS2)** (`/etc/pacman.conf` `XferCommand`), **FreeBSD pkg** (`/usr/local/etc/pkg.conf` `http_proxy`), **Gentoo Portage** (`/etc/portage/make.conf` `http_proxy`), **Gradle** (`~/.gradle/gradle.properties`), **Maven** (`~/.m2/settings.xml`), **Go Modules** (`go/env` & `GOPROXY`), **Bazel** (`~/.bazelrc` `--action_env`), **PHP Composer** (`~/.composer/config.json` `http-proxy`), **Conan C/C++** (`~/.conan2/global.conf` & `~/.conan/conan.conf`), **Scala Build Tool (sbt)** (`~/.sbt/sbtopts`), **Kubernetes (kubectl)** (`~/.kube/config`), **Julia** (`~/.julia/config/startup.jl`), **Nix** (`~/.config/nix/nix.conf` `http-proxy`), **Haskell Cabal** (`~/.cabal/config` `proxy:`), **Haskell Stack** (`~/.stack/config.yaml` `proxy:`), **Hex (Elixir/Erlang/Gleam)** (`~/.hex/hex.config` `http_proxy`), **R Package Manager** (`~/.Renviron` `http_proxy`), **OCaml OPAM** (`~/.opam/config` `http-proxy:`), **Clojure (Leiningen)** (`~/.lein/profiles.clj` `:http-proxy`), **Perl (CPAN)** (`~/.cpan/CPAN/MyConfig.pm` `http_proxy`), **Lua (LuaRocks)** (`~/.luarocks/config.lua` `http_proxy`), **Nim (Nimble)** (`~/.config/nimble/nimble.ini` `http_proxy`), **Erlang (Rebar3)** (`~/.config/rebar3/rebar.config` `http_proxy`), **Scoop (Windows)** (`~/.config/scoop/config.json` `proxy`), **Ruby (Bundler)** (`~/.bundle/config` `BUNDLE_HTTP_PROXY`), **Google Cloud SDK (gcloud)** (`~/.config/gcloud/configurations/config_*` `[proxy]`), **AWS CLI** (`~/.aws/config` `proxy`), and **Microsoft Azure CLI (az)** (`~/.azure/config` `[proxy]`).
  - Diagnoses **Ollama (Local AI Model Runner Daemon) Proxy** (`/etc/systemd/system/ollama.service.d/http-proxy.conf`, `~/.ollama/environment`, `~/Library/LaunchAgents/com.ollama.ollama.plist`) for dead proxy settings causing `ollama pull` and `ollama run` timeout failures.
  - Diagnoses **Alpine Linux apk Package Manager Repositories & Mirrors** (`/etc/apk/repositories`) for unreachable mirrors causing `apk add` and `apk update` stalls.
  - Diagnoses **Flatpak (Linux Desktop Application Package Manager & Runtime)** repository proxy (`~/.local/share/flatpak/repo/config`, `/var/lib/flatpak/repo/config`, `/etc/flatpak/remotes.d/`) for dead proxy settings in `proxy`/`http-proxy` breaking `flatpak install` and `flatpak update`.
  - Diagnoses **Snap (Ubuntu / Debian / Fedora Application Package Manager)** proxy (`/etc/systemd/system/snapd.service.d/http-proxy.conf`, `/var/lib/snapd/environment`, `snap set system proxy.http`) for dead proxy settings breaking `snap install` and `snap refresh`.
  - Diagnoses **Portage (Gentoo, Funtoo, ChromeOS Developer) Proxy** (`/etc/portage/make.conf`, `~/.config/portage/make.conf`, `PORTAGE_MAKE_CONF`) for dead proxy settings in `http_proxy`/`https_proxy` breaking `emerge --sync` and source fetch.
  - Diagnoses **Pacman (Arch Linux, Manjaro, SteamOS, MSYS2) Proxy** (`/etc/pacman.conf`, `~/.config/pacman/pacman.conf`, `PACMAN_CONF`) for dead proxy settings in `XferCommand` breaking `pacman -Syu` and database sync.
  - Diagnoses **FreeBSD pkg Package Manager Proxy** (`/usr/local/etc/pkg.conf`, `/etc/pkg.conf`, `PKG_CONF`) for dead proxy settings breaking `pkg install` and repository catalog updates.
  - Diagnoses **Docker Daemon Engine Proxy** (`/etc/docker/daemon.json`, `~/.docker/daemon.json`, `DOCKER_DAEMON_JSON`) for dead proxy settings causing `docker pull` and image build timeouts.
  - Diagnoses **DNF & YUM (Fedora, RHEL, CentOS, Rocky Linux, AlmaLinux, Amazon Linux) Proxy** (`/etc/dnf/dnf.conf`, `/etc/yum.conf`, `~/.config/dnf/dnf.conf`, `DNF_CONF`) for dead proxy settings breaking `dnf update`, `dnf install`, and repo metadata sync.
  - Diagnoses **Zypper (openSUSE & SUSE Linux Enterprise) Proxy** (`/etc/zypp/zypp.conf`, `~/.zypp/zypp.conf`, `ZYPP_CONF`) for dead proxy settings breaking `zypper refresh` and `zypper install`.
  - Diagnoses **MacPorts (macOS Package Manager) Proxy** (`~/.macports/macports.conf`, `/opt/local/etc/macports/macports.conf`, `MACPORTS_CONF`) for dead proxy settings breaking `port install` and port tree sync.
  - Diagnoses **APT (Debian, Ubuntu, WSL & Containers Package Manager) Proxy** (`/etc/apt/apt.conf.d/*`, `~/.apt/apt.conf`, `APT_CONFIG`) for dead proxy settings breaking `apt update` and `apt install`.
  - Diagnoses **Minikube Local Kubernetes Cluster Proxy** (`~/.minikube/config/config.json`, `MINIKUBE_HOME`) for dead proxy settings breaking `minikube start`, `minikube addons`, and image pulls.
  - Diagnoses **Spack HPC & Scientific Package Manager Proxy** (`~/.spack/config.yaml`, `SPACK_USER_CONFIG_PATH`) for dead proxy settings breaking `spack install`, `spack fetch`, and package downloads.
  - Diagnoses **Chocolatey (Windows Package Manager) Proxy** (`chocolatey.config`, `ChocolateyInstall`) for dead proxy settings breaking `choco install` and `choco upgrade`.
  - Diagnoses **Colima Container Runtime Proxy** (`~/.colima/default/colima.yaml`, `COLIMA_HOME`) for dead proxy environment variables breaking `colima start`, container builds, and image downloads.
  - Diagnoses **Podman Container Engine Proxy** (`~/.config/containers/containers.conf`, `CONTAINERS_CONF`) for dead proxy environment variables breaking `podman pull`, `podman build`, and container execution.
  - Diagnoses **Vagrant Development Environments Proxy** (`~/.vagrant.d/Vagrantfile`, `VAGRANT_HOME`) for dead proxy settings breaking `vagrant box add` and `vagrant up`.
  - Diagnoses **Microsoft Azure CLI (az) Proxy** (`~/.azure/config`, `AZURE_CONFIG_DIR`) for dead proxy settings breaking `az login`, `az account`, and Azure management SDK calls.
  - Diagnoses **Google Cloud SDK (gcloud) Proxy** (`~/.config/gcloud/configurations/config_*`, `CLOUDSDK_CONFIG`) for dead proxy settings breaking `gcloud` commands, authentication, and SDK calls.
  - Diagnoses **AWS CLI Proxy** (`~/.aws/config`, `AWS_CONFIG_FILE`) for dead proxy settings breaking `aws s3`, `aws sts`, and API interactions.
  - Diagnoses **Mamba & Micromamba Proxy** (`~/.mambarc`, `MAMBARC`) for dead proxy servers breaking `mamba install`.
  - Diagnoses **Scoop (Windows Package Manager) Proxy** (`~/.config/scoop/config.json`, `SCOOP_CONFIG`) for dead proxy settings breaking `scoop install` and `scoop update`.
  - Diagnoses **Ruby (Bundler) Package Manager Proxy** (`~/.bundle/config`, `BUNDLE_USER_CONFIG`) for dead proxy settings breaking `bundle install`, `bundle update`, and gem downloads.
  - Diagnoses **Nim (Nimble) Package Manager Proxy** (`~/.config/nimble/nimble.ini`, `NIMBLE_DIR`) for dead proxy settings breaking `nimble install` and package resolution.
  - Diagnoses **Erlang (Rebar3) Package Manager Proxy** (`~/.config/rebar3/rebar.config`, `REBAR_GLOBAL_CONFIG_DIR`) for dead proxy settings breaking `rebar3 compile`, `rebar3 update`, and Hex package downloads.
  - Diagnoses **Kubernetes Helm Chart Repositories** (`~/.config/helm/repositories.yaml`, `HELM_CONFIG_HOME`) for unreachable repository mirrors breaking `helm repo update`.
  - Diagnoses **Lua (LuaRocks) Package Manager Proxy** (`~/.luarocks/config.lua`, `LUAROCKS_CONFIG`) for dead proxy settings breaking `luarocks install` and rockspec builds.
  - Diagnoses **C/C++ vcpkg Configuration Registries & Mirrors** (`~/.vcpkg/vcpkg-configuration.json`, `vcpkg-configuration.json`) for unreachable registry mirrors breaking `vcpkg install`.
  - Diagnoses **Clojure Leiningen Proxy** (`~/.lein/profiles.clj`, `LEIN_HOME`) for dead proxy settings breaking `lein deps`, `lein new`, and Clojars/Maven dependency resolution.
  - Diagnoses **Perl CPAN Package Manager Proxy** (`~/.cpan/CPAN/MyConfig.pm`, `PERL_CPAN_DIR`) for dead proxy ports breaking `cpan` and `cpanm` package installations.
  - Diagnoses **C/C++ vcpkg Asset & Binary Cache Mirrors** (`X_VCPKG_ASSET_SOURCES`, `VCPKG_BINARY_SOURCES`) for unreachable mirror sources causing `vcpkg install` asset fetch failures.
  - Diagnoses **R Package Manager Proxy & CRAN Mirrors** (`~/.Renviron`, `R_ENVIRON_USER`, `CRAN_MIRROR`) for dead proxy ports and unreachable mirrors causing `install.packages()`, `devtools`, and `remotes` build failures.
  - Diagnoses **OCaml OPAM Proxy** (`~/.opam/config`, `OPAMROOT`) for dead proxy ports causing `opam update` and `opam install` hangs.
  - Diagnoses **Terraform & OpenTofu Provider Network Mirrors** (`~/.terraformrc`, `~/.tofurc`, `TF_CLI_CONFIG_FILE`) for unreachable network mirrors breaking `terraform init` and provider downloads.
  - Diagnoses **Haskell Stack Proxy** (`~/.stack/config.yaml` `proxy:`) for dead proxy ports causing GHC compiler setup (`stack setup`) and package build stalls.
  - Diagnoses **Hex (Elixir/Erlang/Gleam) Proxy** (`~/.hex/hex.config`, `HEX_HTTP_PROXY`) for dead proxy ports breaking `mix deps.get`, `rebar3`, and `gleam deps download`.
  - Diagnoses **Nix Package Manager Proxy & Substituters** (`~/.config/nix/nix.conf`, `/etc/nix/nix.conf` `http-proxy` and `substituters`) for dead proxy ports and unreachable binary cache mirrors causing `nix-shell` and `nix build` stalls.
  - Diagnoses **Haskell Cabal Proxy** (`~/.cabal/config` `proxy:`) for dead proxy ports causing `cabal update` and package build failures.
  - Diagnoses **Julia Startup Script Proxy** (`~/.julia/config/startup.jl` `ENV["HTTP_PROXY"]`, `ENV["HTTPS_PROXY"]`, `ENV["all_proxy"]`) to prevent package operations (`Pkg.add` / `Pkg.update`) from hanging.
  - Diagnoses **GitLab CLI (glab) Proxy** (`~/.config/glab-cli/config.yml` `glab_proxy`) for dead proxy ports causing merge request, issue, and pipeline API stalls.
  - Diagnoses **Nix Binary Cache Substituters** (`~/.config/nix/nix.conf`, `/etc/nix/nix.conf` `substituters`) for unreachable mirror caches causing `nix-shell` and `nix build` stalls.
  - Diagnoses **Swift Package Manager & CocoaPods Mirrors** (`~/.swiftpm/configuration/mirrors.json`, `CP_REPO_DIR_URL`, `SWIFT_PACKAGE_REGISTRY_URL`, `SWIFT_PACKAGE_COLLECTION_URL`) for unreachable mirrors causing `pod install` or `swift package resolve` timeouts.
  - Diagnoses **Scala Build Tool (sbt) Proxy** (`~/.sbt/sbtopts` `-Dhttp.proxyHost`, `-Dhttp.proxyPort`, `-Dhttps.proxyHost`, `-Dhttps.proxyPort`) for dead JVM proxy settings breaking sbt build and plugin resolution.
  - Diagnoses **Kubernetes (kubectl) Cluster Proxy** (`~/.kube/config` `clusters[].cluster.proxy-url`) for dead proxy ports causing `kubectl`, `k9s`, and cluster operations to hang.
  - Diagnoses **Conan C/C++ Package Manager Proxy** (`~/.conan2/global.conf` `core.net.http:proxies`, `~/.conan/conan.conf` `[proxies]`) for dead proxy ports breaking C/C++ dependency downloads.
  - Diagnoses **PHP Composer Proxy & Packagist Mirrors** (`~/.composer/config.json`, `~/.config/composer/config.json`) for dead proxy ports and unreachable repository mirrors causing package install failures.
  - Diagnoses **Bazel Proxy Configuration** (`~/.bazelrc` `--action_env=HTTP_PROXY`) to prevent external repository build failures from dead proxy ports.
  - Diagnoses **Swift Package Manager & CocoaPods Mirrors** (`CP_REPO_DIR_URL`, `SWIFT_PACKAGE_REGISTRY_URL`, `SWIFT_PACKAGE_COLLECTION_URL`) for unreachable mirrors causing `pod install` or `swift package resolve` timeouts.
  - Diagnoses **Flutter / Dart Environment & Mirrors** (`PUB_HOSTED_URL`, `FLUTTER_STORAGE_BASE_URL`) for unreachable mirrors causing `flutter pub get` or SDK engine download timeouts.
  - Diagnoses **Rustup / Cargo Toolchain Mirrors** (`RUSTUP_DIST_SERVER`, `RUSTUP_UPDATE_ROOT`) for dead/unreachable download servers causing `rustup toolchain install` or `rustup update` failures.
  - Diagnoses **Julia (Pkg) Package Server Mirror** (`JULIA_PKG_SERVER`) for unreachable mirrors causing Julia package operations to fail.
  - Diagnoses **Deno Proxy Environment** (`DENO_HTTP_PROXY`, `DENO_HTTPS_PROXY`) for dead proxy configurations.
  - Diagnoses **Node / JS Ecosystem Registry Mirrors** (`~/.npmrc`, `~/.yarnrc`, `~/.yarnrc.yml`, `~/.pnpmrc`, `NPM_CONFIG_REGISTRY`): detects unreachable registry mirrors and alerts on deprecated, shut-down domains (`registry.npm.taobao.org`).
  - Diagnoses **Homebrew Mirror Reachability** (`HOMEBREW_BOTTLE_DOMAIN`, `HOMEBREW_API_DOMAIN`, `HOMEBREW_BREW_GIT_REMOTE`, `HOMEBREW_CORE_GIT_REMOTE`) to prevent `brew install` / `brew update` hangs from stale/dead mirrors.
  - Diagnoses **JetBrains & Android Studio IDE family** (`options/proxy.settings.xml` in IntelliJ IDEA, PyCharm, WebStorm, Android Studio, CLion, RustRover, GoLand): detects dead proxy ports and unreachable PAC URLs breaking plugin downloads, AI Assistant, Gradle sync, and SDK tools.
  - Diagnoses **VS Code, Cursor, Windsurf & VSCodium IDE settings** (`settings.json` `http.proxy`, `http.proxyStrictSSL`, `http.proxySupport`): detects dead proxy ports and disabled strict SSL causing GitHub Copilot, Cursor AI, and extensions marketplace connection failures.
  - Diagnoses **Git SSL & Certificate Integrity**: warns when `http.sslVerify` is globally disabled (MITM security risk) and detects broken `http.sslCAInfo` bundle paths that cause Git fatal errors.
  - Diagnoses **Node.js TLS security** (`NODE_TLS_REJECT_UNAUTHORIZED=0`) and CA bundle overrides (`SSL_CERT_FILE`, `REQUESTS_CA_BUNDLE`, `CURL_CA_BUNDLE`).
  - Diagnoses **Cargo Crates.io mirror replacements** (`[source.crates-io] replace-with`) and **Rustup download mirrors** (`RUSTUP_DIST_SERVER`, `RUSTUP_UPDATE_ROOT`) for dead/unreachable servers.
  - Diagnoses **Python / Astral `uv` index mirrors & proxy** (`UV_INDEX_URL`, `UV_EXTRA_INDEX_URL`, `PIP_INDEX_URL`, `UV_HTTP_PROXY`, `uv.toml`) for unreachable mirrors or dead proxy ports causing package install timeouts.
  - Diagnoses **`/etc/hosts` pinned overrides**: detects if developer domains (`github.com`, `api.openai.com`, `huggingface.co`, etc.) are mapped to dead/stale IPs or blocked to `0.0.0.0` overriding DNS.
  - Diagnoses **environment variable casing conflicts** (`http_proxy != HTTP_PROXY`) where tools like cURL (lowercase only) and Go/Python behave inconsistently.
  - Diagnoses **AI Developer Gateways & Mirrors** (`OPENAI_BASE_URL`, `OPENAI_API_BASE`, `ANTHROPIC_BASE_URL`, `GEMINI_BASE_URL`, `GOOGLE_AI_BASE_URL`, `DEEPSEEK_BASE_URL`, `DEEPSEEK_API_BASE`, `GROQ_BASE_URL`, `OPENROUTER_BASE_URL`, `MISTRAL_BASE_URL`, `PERPLEXITY_BASE_URL`, `COHERE_BASE_URL`, `XAI_BASE_URL`, `SILICONFLOW_BASE_URL`, `TOGETHER_BASE_URL`, `MOONSHOT_BASE_URL`, `DASHSCOPE_BASE_URL`, `DEEPINFRA_BASE_URL`, `CEREBRAS_BASE_URL`, `FIREWORKS_BASE_URL`, `ZHIPU_BASE_URL`, `MINIMAX_BASE_URL`, `STEPFUN_BASE_URL`, `LINGYIWANWU_BASE_URL`, `VOYAGE_BASE_URL`, `NOVITA_BASE_URL`, `HYPERBOLIC_BASE_URL`, `LAMBDA_BASE_URL`, `UPSTAGE_BASE_URL`, `AI21_BASE_URL`, `BAICHUAN_BASE_URL`, `SENSENOVA_BASE_URL`, `INTERNLM_BASE_URL`, `HUNYUAN_BASE_URL`, `BASETEN_BASE_URL`, `FRIENDLI_BASE_URL`, `LEPTON_BASE_URL`, `COPILOT_PROXY`, `SAMBANOVA_BASE_URL`, `REPLICATE_API_BASE`, `QIANFAN_BASE_URL`, `DEEPL_BASE_URL`, `ELEVENLABS_BASE_URL`, `PORTKEY_BASE_URL`, `LANGFUSE_BASE_URL`, `CARTESIA_BASE_URL`, `OLLAMA_HOST`, `HF_ENDPOINT`) for dead or unreachable local endpoints.
  - Detects **WSL2 Linux** environment and resolves Windows Host gateway IP to prevent `127.0.0.1` routing traps.
  - Detects **SOCKS5 vs SOCKS5h DNS leak risk** (`socks5h://` tunnels DNS through the daemon to prevent local DNS poisoning).
  - Verifies `NO_PROXY` localhost bypass list to prevent local dev servers (Vite, Next.js, Docker) from failing.
  - Detects DNS poisoning on `github.com`, `raw.githubusercontent.com`, `api.openai.com`, `api.anthropic.com`, `api.deepseek.com`, `proxy.golang.org`, and `registry.yarnpkg.com`.
  - Supports `--json` for machine-readable CI/CD and IDE diagnostics.
- 🩹 **1-Click Auto-Repair (`proxydoc fix`)**:
  - Clears dead Git, **GitHub CLI** (`~/.config/gh/config.yml`), **GitLab CLI** (`~/.config/glab-cli/config.yml`), Cargo (`~/.cargo/config.toml`), NPM, **Yarn** (`~/.yarnrc` / `~/.yarnrc.yml`), **pnpm** (`~/.pnpmrc`), **Bun** (`~/.bunfig.toml`), Pip, **Astral `uv`** (`uv.toml`), **Go Modules** (`go/env` / `GOPROXY`), **Bazel** (`~/.bazelrc`), **PHP Composer** (`config.json`), **Conan C/C++** (`global.conf` / `conan.conf`), **Scala Build Tool (sbt)** (`~/.sbt/sbtopts`), **Kubernetes (kubectl)** (`~/.kube/config`), **Julia** (`~/.julia/config/startup.jl`), **Nix** (`~/.config/nix/nix.conf`), **Haskell Cabal** (`~/.cabal/config`), **Haskell Stack** (`~/.stack/config.yaml`), **Hex (Elixir/Gleam)** (`~/.hex/hex.config`), **R (~/.Renviron)**, **OCaml OPAM (~/.opam/config)**, **Clojure Leiningen (~/.lein/profiles.clj)**, **Perl CPAN (~/.cpan/CPAN/MyConfig.pm)**, **LuaRocks (~/.luarocks/config.lua)**, **Nimble (~/.config/nimble/nimble.ini)**, **Rebar3 (~/.config/rebar3/rebar.config)**, **Scoop (~/.config/scoop/config.json)**, **Ruby Bundler (~/.bundle/config)**, **Google Cloud SDK (gcloud)** (`~/.config/gcloud/configurations/config_*`), **AWS CLI** (`~/.aws/config`), **Microsoft Azure CLI (az)** (`~/.azure/config`), **Conda & Mamba** (`~/.condarc` / `~/.mambarc`), **Podman** (`~/.config/containers/containers.conf`), **Vagrant** (`~/.vagrant.d/Vagrantfile`), **Chocolatey** (`chocolatey.config`), **Colima** (`~/.colima/default/colima.yaml`), **Minikube** (`~/.minikube/config/config.json`), **Spack** (`~/.spack/config.yaml`), **MacPorts** (`~/.macports/macports.conf`), **APT** (`/etc/apt/apt.conf.d/99proxydoc.conf` / `~/.apt/apt.conf`), **DNF & YUM** (`/etc/dnf/dnf.conf` / `~/.config/dnf/dnf.conf`), **Zypper** (`/etc/zypp/zypp.conf` / `~/.zypp/zypp.conf`), **Pacman** (`/etc/pacman.conf`), **FreeBSD pkg** (`/usr/local/etc/pkg.conf`), **Gentoo Portage** (`/etc/portage/make.conf`), **Docker Daemon** (`/etc/docker/daemon.json`), **Flatpak** (`~/.local/share/flatpak/repo/config`), **Snap** (`/etc/systemd/system/snapd.service.d/http-proxy.conf` / `snap set system`), **Ollama Daemon** (`/etc/systemd/system/ollama.service.d/http-proxy.conf` / `~/.ollama/environment`), **NuGet (.NET)** (`NuGet.Config`), **Gem (Ruby)** (`~/.gemrc`), **Subversion (SVN)** (`servers`), **Mercurial (Hg)** (`~/.hgrc`), Docker (`~/.docker/config.json`), **Gradle** (`~/.gradle/gradle.properties`), **Maven** (`~/.m2/settings.xml`), **JetBrains / Android Studio** (`proxy.settings.xml`), and **VS Code / Cursor IDE** (`settings.json`) global proxies across 70 automated remediation steps.
  - Automatically disables unreachable Alpine Linux apk repository mirrors in `/etc/apk/repositories` with `.bak` safety backups.
  - Automatically disables unreachable Kubernetes Helm chart repositories in `repositories.yaml` with `.bak` safety backups.
  - Automatically disables dead vcpkg registry mirrors in `vcpkg-configuration.json` and restores official sources with `.bak` safety backups.
  - Automatically disables dead Terraform & OpenTofu provider network mirrors in `~/.terraformrc` and `~/.tofurc` with `.bak` safety backups.
  - Automatically prunes dead Swift Package Manager package mirrors in `~/.swiftpm/configuration/mirrors.json` with `.bak` safety backups.
  - Automatically migrates deprecated & shut-down `registry.npm.taobao.org` to official `https://registry.npmmirror.com` across `~/.npmrc`, `~/.yarnrc`, `~/.yarnrc.yml`, and `~/.pnpmrc` with `.bak` safety backups.
  - Automatically unsets broken Git `http.sslCAInfo` paths pointing to missing certificate bundle files.
  - Automatically disables dead proxies in **cURL** (`~/.curlrc`) and **Wget** (`~/.wgetrc`) with automatic `.bak` backups.
  - Diagnoses dead SSH config ProxyCommand targets preventing Git SSH hangs.
  - Generates sync injection from system proxy to terminal with proper `NO_PROXY` bypass.
  - Cross-platform DNS cache flush (**macOS**, **Windows**, **Linux**).
- ⚡ **AI & Dev Latency Checker (`proxydoc test`)**:
  - Fully concurrent testing to DeepSeek, OpenAI, Anthropic, **Google Gemini**, **xAI (Grok)**, **SiliconFlow**, **Together AI**, **Moonshot (Kimi)**, **Aliyun DashScope (Qwen)**, **DeepInfra**, **Cerebras**, **Fireworks AI**, **Zhipu AI (GLM)**, **MiniMax**, **StepFun (Step-2)**, **01.AI (Yi)**, **Voyage AI**, **Novita AI**, **Hyperbolic API**, **Lambda Labs API**, **Upstage (Solar) API**, **Ai21 Labs (Jamba) API**, **Baichuan AI API**, **SenseNova (SenseTime) API**, **Tencent Hunyuan API**, **Baseten API**, **FriendliAI API**, **Lepton AI API**, **Nebius AI Studio API**, **Chutes AI API**, **Infini-AI API**, **AIML API**, **Crusoe Cloud API**, **Modal**, **NVIDIA NIM API**, **ByteDance Doubao (Ark) API**, **Fal.ai**, **RunPod API**, **Helicone AI Gateway**, **Kling AI API**, **ElevenLabs API**, **Cartesia Voice AI API**, **Portkey AI Gateway**, **Langfuse API**, **LiveKit Cloud**, **Deepgram Speech AI API**, **Weights & Biases (W&B) API**, **Pinecone Vector DB**, **Tavily AI Search API**, **Exa AI Search API**, **Qdrant Cloud**, **AssemblyAI Speech API**, **Jina AI API**, **Firecrawl API**, **Weaviate Cloud**, **Chroma Vector DB**, **GitHub Copilot API**, **SambaNova API**, **Replicate API**, **Baidu Qianfan (ERNIE)**, **DeepL API**, **Ollama**, **OpenRouter**, **Groq**, **Mistral API**, **Perplexity API**, **Cohere API**, GitHub, Google, Cloudflare, HuggingFace, **Docker Hub**, **crates.io**, **npm Registry**, **Yarn Registry**, **Go Proxy**, **Stackage (Haskell)**, **Hex.pm Registry**, **CRAN (R Archive)**, **OPAM Registry (OCaml)**, **Clojars (Clojure)**, **MetaCPAN (Perl)**, **LuaRocks Registry**, **Nimble Directory**, **Hex PM (Erlang/Hex)**, **RubyGems Registry**, **Cursor AI API**, **LanceDB Cloud**, **Maven Central**, **NuGet Gallery**, **Packagist**, **Vapi AI API**, **Braintrust API**, **LlamaIndex Cloud**, **Mem0 AI API**, **Supabase API**, **Dify Cloud API**, **Unstructured.io API**, **Upstash API**, **Vercel Cloud API**, **Resend API**, **E2B Sandbox API**, **PostHog Analytics API**, **Pydantic Logfire API**, **Railway Cloud API**, **Sentry API**, **Neon Database API**, **Render Cloud API**, **Fly.io Machines API**, **Turso Database API**, **Clerk Auth API**, **Convex Cloud**, **Inngest API**, **Trigger.dev Cloud**, **Civitai API**, **Axiom API**, **Stripe API**, **Cloudflare API**, **Dub.co API**, **Koyeb Cloud API**, **Novu API**, **Linear API**, **Codeium API**, **Datadog API**, **Sanity API**, **Meilisearch Cloud**, **PlanetScale API**, **Better Stack API**, **Mintlify API**, **Val Town API**, **Hetzner Cloud API**, **GitLab API**, **DigitalOcean API**, **Netlify API**, **Scaleway Cloud API**, **Linode (Akamai) API**, **Bitbucket API**, **Vultr Cloud API**, **Fastly API**, **Gitee API**, **Deno Deploy API**, **Ollama Registry**, **GHCR (GitHub Packages)**, **GitHub Releases (Objects)**, **Tailscale API**, and **Replit API**.
  - **TLS/SSL Interception Detection**: Distinguishes corporate proxy MITM certificate errors and untrusted CAs from basic connection timeouts.
  - **Comparative Benchmark (`proxydoc test --compare` / `-c`)**: Tests Direct vs Proxy side-by-side, comparing latency, reachability, and egress IPs.
  - **Custom Target Benchmark (`proxydoc test -t <URL>`)**: Benchmarks custom API endpoints or internal corporate gateways.
  - **Configurable Timeout (`--timeout <SECS>`)**: Fine-tune probe deadlines for slow or satellite connections.
  - **Forced Direct Mode (`proxydoc test --direct` / `-d`)**: Tests connection directly bypassing any active proxy.
  - **Outbound Egress IP & Location Inspector**: instantly verifies your actual exit IP, country, and Cloudflare data center.
  - Supports `--json` for automation.
- 🔍 **Active Client Scanner (`proxydoc scan`)**: Detects Clash, Clash Verge, Shadowrocket, Sing-box, V2Ray, Surge, and CPA.
- 🟢 **Multi-Shell Switcher (`proxydoc on` / `proxydoc off`)**:
  - Toggle proxies in current shell session with syntax tailored for **Bash/Zsh**, **Fish**, and **PowerShell** (`--shell fish`, `--shell pwsh`).
  - Configure package managers & dev tools on demand: `--git`, `--cargo`, `--npm`, `--yarn`, `--pnpm`, `--bun`, `--pip`, `--uv`, `--go`, `--bazel`, `--composer`, `--conan`, `--sbt`, `--kube`, `--glab`, `--julia`, `--nix`, `--cabal`, `--stack`, `--hex`, `--r`, `--opam`, `--lein`, `--cpan`, `--luarocks`, `--nimble`, `--rebar3`, `--scoop`, `--bundle`, `--gcloud`, `--aws`, `--azure` (`--az`), `--mamba` (`--micromamba`), `--podman`, `--vagrant`, `--choco` (`--chocolatey`), `--colima`, `--minikube`, `--spack`, `--macports`, `--apt`, `--dnf`, `--zypper`, `--pacman`, `--pkg`, `--portage` (`--emerge`), `--docker-daemon` (`--daemon`), `--flatpak` (`--ostree`), `--snap` (`--snapd`), `--ollama` (`--ollamad`), `--conda`, `--docker`, `--gradle`, `--maven`, `--vscode`, `--gh`, `--jetbrains`, `--nuget`, `--gem`, `--svn`, `--hg`, `--curl`, `--wget`, or `--all` (`-a`). Sets standard `no_proxy` bypass automatically.

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
