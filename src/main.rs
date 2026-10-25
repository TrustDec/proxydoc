use anyhow::Result;
use clap::{Parser, Subcommand};

mod checker;
mod doctor;
mod fixer;
mod manager;
mod scanner;

#[derive(Parser)]
#[command(
    name = "proxydoc",
    author = "TrustDec <https://github.com/TrustDec>",
    version = clap::crate_version!(),
    about = "🩺 Deep developer proxy diagnostics, DNS poisoning checker & one-click auto-repair CLI.",
    long_about = "proxydoc is a modern, lightweight CLI tool to diagnose root causes of broken developer network environments, detect DNS poisoning on GitHub/AI APIs, sync system proxies to terminal, and repair broken configurations automatically."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 🩺 Deep diagnostics (System vs Terminal sync, DNS poisoning, Git/NPM/Cargo/Pip/cURL/Docker dead ports)
    Doctor {
        /// Show verbose diagnostic details
        #[arg(short, long)]
        verbose: bool,
        /// Output diagnostics in JSON format for automation / IDEs
        #[arg(long)]
        json: bool,
    },
    /// 🩹 One-click automatic repair for detected network and proxy issues
    Fix,
    /// ⚡ Concurrently test latency to GitHub, OpenAI, Anthropic, DeepSeek, Google & inspect Egress IP
    Test {
        /// Custom proxy URL to test (e.g. http://127.0.0.1:7890 or socks5://127.0.0.1:1082)
        #[arg(short, long)]
        proxy: Option<String>,
        /// Force direct connection ignoring any active proxy
        #[arg(short, long)]
        direct: bool,
        /// Benchmark and compare Direct vs Proxy side-by-side
        #[arg(short, long)]
        compare: bool,
        /// Output test results and egress info in JSON format
        #[arg(long)]
        json: bool,
        /// Additional custom endpoint URL to benchmark (e.g. https://api.deepseek.com or internal gateway)
        #[arg(short, long)]
        target: Option<String>,
        /// Timeout in seconds per endpoint request (default: 5)
        #[arg(long, default_value_t = 5)]
        timeout: u64,
    },
    /// 🔍 Scan active local proxy daemon ports (Clash, Shadowrocket, Sing-box, V2Ray, CPA)
    Scan {
        /// Output scan results in JSON format
        #[arg(long)]
        json: bool,
    },
    /// 🟢 Output shell export commands to turn ON proxy for current session
    On {
        /// Proxy port or URL (defaults to auto-detected system or daemon port)
        #[arg(default_value = "auto")]
        target: String,
        /// Also configure git global proxy
        #[arg(long)]
        git: bool,
        /// Also configure cargo proxy (~/.cargo/config.toml)
        #[arg(long)]
        cargo: bool,
        /// Also configure npm proxy
        #[arg(long)]
        npm: bool,
        /// Also configure Yarn proxy (~/.yarnrc / ~/.yarnrc.yml)
        #[arg(long)]
        yarn: bool,
        /// Also configure pnpm proxy (~/.pnpmrc)
        #[arg(long)]
        pnpm: bool,
        /// Also configure Bun proxy (~/.bunfig.toml)
        #[arg(long)]
        bun: bool,
        /// Also configure pip global proxy
        #[arg(long)]
        pip: bool,
        /// Also configure Conda proxy (~/.condarc)
        #[arg(long)]
        conda: bool,
        /// Also configure Docker client proxy (~/.docker/config.json)
        #[arg(long)]
        docker: bool,
        /// Also configure Gradle global proxy (~/.gradle/gradle.properties)
        #[arg(long)]
        gradle: bool,
        /// Also configure Maven global proxy (~/.m2/settings.xml)
        #[arg(long)]
        maven: bool,
        /// Also configure VS Code and Cursor IDE proxy (settings.json)
        #[arg(long)]
        vscode: bool,
        /// Also configure GitHub CLI (gh) proxy (~/.config/gh/config.yml)
        #[arg(long)]
        gh: bool,
        /// Also configure GitLab CLI (glab) proxy (~/.config/glab-cli/config.yml)
        #[arg(long)]
        glab: bool,
        /// Also configure JetBrains & Android Studio IDE proxy (proxy.settings.xml)
        #[arg(long)]
        jetbrains: bool,
        /// Also configure NuGet (.NET) proxy (NuGet.Config)
        #[arg(long)]
        nuget: bool,
        /// Also configure cURL proxy (~/.curlrc)
        #[arg(long)]
        curl: bool,
        /// Also configure Wget proxy (~/.wgetrc)
        #[arg(long)]
        wget: bool,
        /// Also configure Gem (Ruby) proxy (~/.gemrc)
        #[arg(long)]
        gem: bool,
        /// Also configure Subversion (SVN) proxy (~/.subversion/servers)
        #[arg(long)]
        svn: bool,
        /// Also configure Mercurial (Hg) proxy (~/.hgrc)
        #[arg(long)]
        hg: bool,
        /// Also configure Astral uv proxy (~/.config/uv/uv.toml)
        #[arg(long)]
        uv: bool,
        /// Also configure Go modules proxy (go/env & GOPROXY)
        #[arg(long)]
        go: bool,
        /// Also configure Bazel proxy (~/.bazelrc action_env)
        #[arg(long)]
        bazel: bool,
        /// Also configure PHP Composer proxy (config.json)
        #[arg(long)]
        composer: bool,
        /// Also configure Conan C/C++ proxy (global.conf / conan.conf)
        #[arg(long)]
        conan: bool,
        /// Also configure Scala Build Tool (sbt) proxy (~/.sbt/sbtopts)
        #[arg(long)]
        sbt: bool,
        /// Also configure Kubernetes (kubectl) cluster proxy (~/.kube/config proxy-url)
        #[arg(long)]
        kube: bool,
        /// Also configure Julia proxy (~/.julia/config/startup.jl)
        #[arg(long)]
        julia: bool,
        /// Also configure Nix package manager proxy (~/.config/nix/nix.conf)
        #[arg(long)]
        nix: bool,
        /// Also configure Haskell Cabal proxy (~/.cabal/config)
        #[arg(long)]
        cabal: bool,
        /// Also configure Haskell Stack proxy (~/.stack/config.yaml)
        #[arg(long)]
        stack: bool,
        /// Also configure Elixir / Erlang / Gleam Hex proxy (~/.hex/hex.config)
        #[arg(long)]
        hex: bool,
        /// Also configure R package manager proxy (~/.Renviron)
        #[arg(long)]
        r: bool,
        /// Also configure OCaml OPAM proxy (~/.opam/config)
        #[arg(long)]
        opam: bool,
        /// Also configure Clojure Leiningen proxy (~/.lein/profiles.clj)
        #[arg(long)]
        lein: bool,
        /// Also configure Perl CPAN package manager proxy (~/.cpan/CPAN/MyConfig.pm)
        #[arg(long)]
        cpan: bool,
        /// Also configure Lua (LuaRocks) package manager proxy (~/.luarocks/config.lua)
        #[arg(long)]
        luarocks: bool,
        /// Also configure Nim (Nimble) package manager proxy (~/.config/nimble/nimble.ini)
        #[arg(long)]
        nimble: bool,
        /// Also configure Erlang (Rebar3) package manager proxy (~/.config/rebar3/rebar.config)
        #[arg(long)]
        rebar3: bool,
        /// Also configure Scoop (Windows Package Manager) proxy (~/.config/scoop/config.json)
        #[arg(long)]
        scoop: bool,
        /// Also configure Ruby Bundler proxy (~/.bundle/config)
        #[arg(long, visible_alias = "bundler")]
        bundle: bool,
        /// Also configure Google Cloud SDK (gcloud) proxy (~/.config/gcloud/configurations/config_*)
        #[arg(long)]
        gcloud: bool,
        /// Also configure AWS CLI proxy (~/.aws/config)
        #[arg(long)]
        aws: bool,
        /// Also configure Microsoft Azure CLI (az) proxy (~/.azure/config)
        #[arg(long, visible_alias = "az")]
        azure: bool,
        /// Also configure Mamba / Micromamba proxy (~/.mambarc)
        #[arg(long, visible_alias = "micromamba")]
        mamba: bool,
        /// Also configure Podman container engine proxy (~/.config/containers/containers.conf)
        #[arg(long)]
        podman: bool,
        /// Also configure Vagrant proxy (~/.vagrant.d/Vagrantfile)
        #[arg(long)]
        vagrant: bool,
        /// Also configure Chocolatey package manager proxy (chocolatey.config)
        #[arg(long, visible_alias = "chocolatey")]
        choco: bool,
        /// Also configure Colima container runtime proxy (~/.colima/default/colima.yaml)
        #[arg(long)]
        colima: bool,
        /// Also configure Minikube cluster proxy (~/.minikube/config/config.json)
        #[arg(long)]
        minikube: bool,
        /// Also configure Spack scientific package manager proxy (~/.spack/config.yaml)
        #[arg(long)]
        spack: bool,
        /// Also configure MacPorts package manager proxy (~/.macports/macports.conf)
        #[arg(long)]
        macports: bool,
        /// Also configure APT package manager proxy (/etc/apt/apt.conf.d/99proxydoc.conf)
        #[arg(long)]
        apt: bool,
        /// Also configure DNF & YUM package manager proxy (/etc/dnf/dnf.conf)
        #[arg(long)]
        dnf: bool,
        /// Also configure Zypper package manager proxy (/etc/zypp/zypp.conf)
        #[arg(long)]
        zypper: bool,
        /// Also configure Pacman package manager proxy (/etc/pacman.conf)
        #[arg(long)]
        pacman: bool,
        /// Also configure FreeBSD pkg package manager proxy (/usr/local/etc/pkg.conf)
        #[arg(long)]
        pkg: bool,
        /// Also configure Portage package manager proxy (/etc/portage/make.conf)
        #[arg(long, visible_alias = "emerge")]
        portage: bool,
        /// Also configure Docker Daemon engine proxy (/etc/docker/daemon.json)
        #[arg(long, visible_alias = "daemon")]
        docker_daemon: bool,
        /// Also configure Flatpak package manager proxy (~/.local/share/flatpak/repo/config / /var/lib/flatpak/repo/config)
        #[arg(long, visible_alias = "ostree")]
        flatpak: bool,
        /// Also configure Snap package manager proxy (/etc/systemd/system/snapd.service.d/http-proxy.conf / snap set system proxy.http)
        #[arg(long, visible_alias = "snapd")]
        snap: bool,
        /// Also configure Ollama local AI runner daemon proxy (/etc/systemd/system/ollama.service.d/http-proxy.conf / ~/.ollama/environment)
        #[arg(long, visible_alias = "ollamad")]
        ollama: bool,
        /// Also configure git, cargo, npm, yarn, pnpm, bun, pip, conda, docker, gradle, maven, vscode, gh, glab, jetbrains, nuget, gem, svn, hg, uv, go, bazel, composer, conan, sbt, kube, julia, nix, cabal, stack, hex, r, opam, lein, cpan, luarocks, nimble, rebar3, scoop, bundle, gcloud, aws, azure, mamba, podman, vagrant, choco, colima, minikube, spack, macports, apt, dnf, zypper, pacman, pkg, portage, docker daemon, flatpak, snap, and ollama proxy
        #[arg(short, long)]
        all: bool,
        /// Shell format for environment variables: bash, zsh, fish, pwsh (auto-detected if omitted)
        #[arg(short, long)]
        shell: Option<String>,
    },
    /// 🔴 Output shell unset commands to turn OFF proxy for current session
    Off {
        /// Also unset git global proxy
        #[arg(long)]
        git: bool,
        /// Also unset cargo proxy (~/.cargo/config.toml)
        #[arg(long)]
        cargo: bool,
        /// Also unset npm proxy
        #[arg(long)]
        npm: bool,
        /// Also unset Yarn proxy (~/.yarnrc / ~/.yarnrc.yml)
        #[arg(long)]
        yarn: bool,
        /// Also unset pnpm proxy (~/.pnpmrc)
        #[arg(long)]
        pnpm: bool,
        /// Also unset Bun proxy (~/.bunfig.toml)
        #[arg(long)]
        bun: bool,
        /// Also unset pip global proxy
        #[arg(long)]
        pip: bool,
        /// Also unset Conda proxy (~/.condarc)
        #[arg(long)]
        conda: bool,
        /// Also unset Docker client proxy (~/.docker/config.json)
        #[arg(long)]
        docker: bool,
        /// Also unset Gradle global proxy (~/.gradle/gradle.properties)
        #[arg(long)]
        gradle: bool,
        /// Also unset Maven global proxy (~/.m2/settings.xml)
        #[arg(long)]
        maven: bool,
        /// Also unset VS Code and Cursor IDE proxy (settings.json)
        #[arg(long)]
        vscode: bool,
        /// Also unset GitHub CLI (gh) proxy (~/.config/gh/config.yml)
        #[arg(long)]
        gh: bool,
        /// Also unset GitLab CLI (glab) proxy (~/.config/glab-cli/config.yml)
        #[arg(long)]
        glab: bool,
        /// Also unset JetBrains & Android Studio IDE proxy (proxy.settings.xml)
        #[arg(long)]
        jetbrains: bool,
        /// Also unset NuGet (.NET) proxy (NuGet.Config)
        #[arg(long)]
        nuget: bool,
        /// Also unset cURL proxy (~/.curlrc)
        #[arg(long)]
        curl: bool,
        /// Also unset Wget proxy (~/.wgetrc)
        #[arg(long)]
        wget: bool,
        /// Also unset Gem (Ruby) proxy (~/.gemrc)
        #[arg(long)]
        gem: bool,
        /// Also unset Subversion (SVN) proxy (~/.subversion/servers)
        #[arg(long)]
        svn: bool,
        /// Also unset Mercurial (Hg) proxy (~/.hgrc)
        #[arg(long)]
        hg: bool,
        /// Also unset Astral uv proxy (~/.config/uv/uv.toml)
        #[arg(long)]
        uv: bool,
        /// Also unset Go modules proxy (go/env & GOPROXY)
        #[arg(long)]
        go: bool,
        /// Also unset Bazel proxy (~/.bazelrc action_env)
        #[arg(long)]
        bazel: bool,
        /// Also unset PHP Composer proxy (config.json)
        #[arg(long)]
        composer: bool,
        /// Also unset Conan C/C++ proxy (global.conf / conan.conf)
        #[arg(long)]
        conan: bool,
        /// Also unset Scala Build Tool (sbt) proxy (~/.sbt/sbtopts)
        #[arg(long)]
        sbt: bool,
        /// Also unset Kubernetes (kubectl) cluster proxy (~/.kube/config proxy-url)
        #[arg(long)]
        kube: bool,
        /// Also unset Julia proxy (~/.julia/config/startup.jl)
        #[arg(long)]
        julia: bool,
        /// Also unset Nix package manager proxy (~/.config/nix/nix.conf)
        #[arg(long)]
        nix: bool,
        /// Also unset Haskell Cabal proxy (~/.cabal/config)
        #[arg(long)]
        cabal: bool,
        /// Also unset Haskell Stack proxy (~/.stack/config.yaml)
        #[arg(long)]
        stack: bool,
        /// Also unset Elixir / Erlang / Gleam Hex proxy (~/.hex/hex.config)
        #[arg(long)]
        hex: bool,
        /// Also unset R package manager proxy (~/.Renviron)
        #[arg(long)]
        r: bool,
        /// Also unset OCaml OPAM proxy (~/.opam/config)
        #[arg(long)]
        opam: bool,
        /// Also unset Clojure Leiningen proxy (~/.lein/profiles.clj)
        #[arg(long)]
        lein: bool,
        /// Also unset Perl CPAN package manager proxy (~/.cpan/CPAN/MyConfig.pm)
        #[arg(long)]
        cpan: bool,
        /// Also unset Lua (LuaRocks) package manager proxy (~/.luarocks/config.lua)
        #[arg(long)]
        luarocks: bool,
        /// Also unset Nim (Nimble) package manager proxy (~/.config/nimble/nimble.ini)
        #[arg(long)]
        nimble: bool,
        /// Also unset Erlang (Rebar3) package manager proxy (~/.config/rebar3/rebar.config)
        #[arg(long)]
        rebar3: bool,
        /// Also unset Scoop (Windows Package Manager) proxy (~/.config/scoop/config.json)
        #[arg(long)]
        scoop: bool,
        /// Also unset Ruby Bundler proxy (~/.bundle/config)
        #[arg(long, visible_alias = "bundler")]
        bundle: bool,
        /// Also unset Google Cloud SDK (gcloud) proxy (~/.config/gcloud/configurations/config_*)
        #[arg(long)]
        gcloud: bool,
        /// Also unset AWS CLI proxy (~/.aws/config)
        #[arg(long)]
        aws: bool,
        /// Also unset Microsoft Azure CLI (az) proxy (~/.azure/config)
        #[arg(long, visible_alias = "az")]
        azure: bool,
        /// Also unset Mamba / Micromamba proxy (~/.mambarc)
        #[arg(long, visible_alias = "micromamba")]
        mamba: bool,
        /// Also unset Podman container engine proxy (~/.config/containers/containers.conf)
        #[arg(long)]
        podman: bool,
        /// Also unset Vagrant proxy (~/.vagrant.d/Vagrantfile)
        #[arg(long)]
        vagrant: bool,
        /// Also unset Chocolatey package manager proxy (chocolatey.config)
        #[arg(long, visible_alias = "chocolatey")]
        choco: bool,
        /// Also unset Colima container runtime proxy (~/.colima/default/colima.yaml)
        #[arg(long)]
        colima: bool,
        /// Also unset Minikube cluster proxy (~/.minikube/config/config.json)
        #[arg(long)]
        minikube: bool,
        /// Also unset Spack scientific package manager proxy (~/.spack/config.yaml)
        #[arg(long)]
        spack: bool,
        /// Also unset MacPorts package manager proxy (~/.macports/macports.conf)
        #[arg(long)]
        macports: bool,
        /// Also unset APT package manager proxy (/etc/apt/apt.conf.d/99proxydoc.conf)
        #[arg(long)]
        apt: bool,
        /// Also unset DNF & YUM package manager proxy (/etc/dnf/dnf.conf)
        #[arg(long)]
        dnf: bool,
        /// Also unset Zypper package manager proxy (/etc/zypp/zypp.conf)
        #[arg(long)]
        zypper: bool,
        /// Also unset Pacman package manager proxy (/etc/pacman.conf)
        #[arg(long)]
        pacman: bool,
        /// Also unset FreeBSD pkg package manager proxy (/usr/local/etc/pkg.conf)
        #[arg(long)]
        pkg: bool,
        /// Also unset Portage package manager proxy (/etc/portage/make.conf)
        #[arg(long, visible_alias = "emerge")]
        portage: bool,
        /// Also unset Docker Daemon engine proxy (/etc/docker/daemon.json)
        #[arg(long, visible_alias = "daemon")]
        docker_daemon: bool,
        /// Also unset Flatpak package manager proxy (~/.local/share/flatpak/repo/config / /var/lib/flatpak/repo/config)
        #[arg(long, visible_alias = "ostree")]
        flatpak: bool,
        /// Also unset Snap package manager proxy (/etc/systemd/system/snapd.service.d/http-proxy.conf / snap set system proxy.http)
        #[arg(long, visible_alias = "snapd")]
        snap: bool,
        /// Also unset Ollama local AI runner daemon proxy (/etc/systemd/system/ollama.service.d/http-proxy.conf / ~/.ollama/environment)
        #[arg(long, visible_alias = "ollamad")]
        ollama: bool,
        /// Also unset git, cargo, npm, yarn, pnpm, bun, pip, conda, docker, gradle, maven, vscode, gh, glab, jetbrains, nuget, gem, svn, hg, uv, go, bazel, composer, conan, sbt, kube, julia, nix, cabal, stack, hex, r, opam, lein, cpan, luarocks, nimble, rebar3, scoop, bundle, gcloud, aws, azure, mamba, podman, vagrant, choco, colima, minikube, spack, macports, apt, dnf, zypper, pacman, pkg, portage, docker daemon, flatpak, snap, and ollama proxy
        #[arg(short, long)]
        all: bool,
        /// Shell format for environment variables: bash, zsh, fish, pwsh (auto-detected if omitted)
        #[arg(short, long)]
        shell: Option<String>,
    },
    /// 🐚 Print shell integration helper script for eval: `eval "$(proxydoc init)"`
    Init {
        /// Target shell syntax: bash, zsh, fish, pwsh (auto-detected if omitted)
        #[arg(short, long)]
        shell: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            doctor::run(false, false).await?;
        }
        Some(Commands::Doctor { verbose, json }) => {
            doctor::run(verbose, json).await?;
        }
        Some(Commands::Fix) => {
            fixer::run().await?;
        }
        Some(Commands::Test {
            proxy,
            direct,
            compare,
            json,
            target,
            timeout,
        }) => {
            checker::run(proxy, direct, compare, json, target, timeout).await?;
        }
        Some(Commands::Scan { json }) => {
            scanner::run(json).await?;
        }
        Some(Commands::On {
            target,
            git,
            cargo,
            npm,
            yarn,
            pnpm,
            bun,
            pip,
            conda,
            docker,
            gradle,
            maven,
            vscode,
            gh,
            glab,
            jetbrains,
            nuget,
            curl,
            wget,
            gem,
            svn,
            hg,
            uv,
            go,
            bazel,
            composer,
            conan,
            sbt,
            kube,
            julia,
            nix,
            cabal,
            stack,
            hex,
            r,
            opam,
            lein,
            cpan,
            luarocks,
            nimble,
            rebar3,
            scoop,
            bundle,
            gcloud,
            aws,
            azure,
            mamba,
            podman,
            vagrant,
            choco,
            colima,
            minikube,
            spack,
            macports,
            apt,
            dnf,
            zypper,
            pacman,
            pkg,
            portage,
            docker_daemon,
            flatpak,
            snap,
            ollama,
            all,
            shell,
        }) => {
            manager::turn_on(
                &target,
                git,
                cargo,
                npm,
                yarn,
                pnpm,
                bun,
                pip,
                conda,
                docker,
                gradle,
                maven,
                vscode,
                gh,
                glab,
                jetbrains,
                nuget,
                curl,
                wget,
                gem,
                svn,
                hg,
                uv,
                go,
                bazel,
                composer,
                conan,
                sbt,
                kube,
                julia,
                nix,
                cabal,
                stack,
                hex,
                r,
                opam,
                lein,
                cpan,
                luarocks,
                nimble,
                rebar3,
                scoop,
                bundle,
                gcloud,
                aws,
                azure,
                mamba,
                podman,
                vagrant,
                choco,
                colima,
                minikube,
                spack,
                macports,
                apt,
                dnf,
                zypper,
                pacman,
                pkg,
                portage,
                docker_daemon,
                flatpak,
                snap,
                ollama,
                all,
                shell.as_deref(),
            )
            .await?;
        }
        Some(Commands::Off {
            git,
            cargo,
            npm,
            yarn,
            pnpm,
            bun,
            pip,
            conda,
            docker,
            gradle,
            maven,
            vscode,
            gh,
            glab,
            jetbrains,
            nuget,
            curl,
            wget,
            gem,
            svn,
            hg,
            uv,
            go,
            bazel,
            composer,
            conan,
            sbt,
            kube,
            julia,
            nix,
            cabal,
            stack,
            hex,
            r,
            opam,
            lein,
            cpan,
            luarocks,
            nimble,
            rebar3,
            scoop,
            bundle,
            gcloud,
            aws,
            azure,
            mamba,
            podman,
            vagrant,
            choco,
            colima,
            minikube,
            spack,
            macports,
            apt,
            dnf,
            zypper,
            pacman,
            pkg,
            portage,
            docker_daemon,
            flatpak,
            snap,
            ollama,
            all,
            shell,
        }) => {
            manager::turn_off(
                git,
                cargo,
                npm,
                yarn,
                pnpm,
                bun,
                pip,
                conda,
                docker,
                gradle,
                maven,
                vscode,
                gh,
                glab,
                jetbrains,
                nuget,
                curl,
                wget,
                gem,
                svn,
                hg,
                uv,
                go,
                bazel,
                composer,
                conan,
                sbt,
                kube,
                julia,
                nix,
                cabal,
                stack,
                hex,
                r,
                opam,
                lein,
                cpan,
                luarocks,
                nimble,
                rebar3,
                scoop,
                bundle,
                gcloud,
                aws,
                azure,
                mamba,
                podman,
                vagrant,
                choco,
                colima,
                minikube,
                spack,
                macports,
                apt,
                dnf,
                zypper,
                pacman,
                pkg,
                portage,
                docker_daemon,
                flatpak,
                snap,
                ollama,
                all,
                shell.as_deref(),
            )?;
        }
        Some(Commands::Init { shell }) => {
            manager::print_init_script(shell.as_deref());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_doctor_json() {
        let cli = Cli::try_parse_from(["proxydoc", "doctor", "--json"]).unwrap();
        match cli.command {
            Some(Commands::Doctor { verbose, json }) => {
                assert!(!verbose);
                assert!(json);
            }
            _ => panic!("Expected Doctor command"),
        }
    }

    #[test]
    fn test_cli_parse_test_options() {
        let cli = Cli::try_parse_from([
            "proxydoc",
            "test",
            "--proxy",
            "http://127.0.0.1:7890",
            "--json",
        ])
        .unwrap();
        match cli.command {
            Some(Commands::Test {
                proxy,
                direct,
                compare,
                json,
                ..
            }) => {
                assert_eq!(proxy, Some("http://127.0.0.1:7890".to_string()));
                assert!(!direct);
                assert!(!compare);
                assert!(json);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_test_compare() {
        let cli = Cli::try_parse_from(["proxydoc", "test", "--compare"]).unwrap();
        match cli.command {
            Some(Commands::Test {
                compare, direct, ..
            }) => {
                assert!(compare);
                assert!(!direct);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_test_direct() {
        let cli = Cli::try_parse_from(["proxydoc", "test", "-d"]).unwrap();
        match cli.command {
            Some(Commands::Test {
                direct, compare, ..
            }) => {
                assert!(direct);
                assert!(!compare);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_on_all() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "7890", "--all"]).unwrap();
        match cli.command {
            Some(Commands::On {
                target, all, pip, ..
            }) => {
                assert_eq!(target, "7890");
                assert!(all);
                assert!(!pip);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pip() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pip"]).unwrap();
        match cli.command {
            Some(Commands::On { pip, .. }) => {
                assert!(pip);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_test_target_and_timeout() {
        let cli = Cli::try_parse_from([
            "proxydoc",
            "test",
            "-t",
            "https://api.deepseek.com/models",
            "--timeout",
            "10",
        ])
        .unwrap();
        match cli.command {
            Some(Commands::Test {
                target, timeout, ..
            }) => {
                assert_eq!(target, Some("https://api.deepseek.com/models".to_string()));
                assert_eq!(timeout, 10);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_on_docker_and_shell() {
        let cli =
            Cli::try_parse_from(["proxydoc", "on", "7897", "--docker", "--shell", "fish"]).unwrap();
        match cli.command {
            Some(Commands::On {
                target,
                docker,
                shell,
                ..
            }) => {
                assert_eq!(target, "7897");
                assert!(docker);
                assert_eq!(shell, Some("fish".to_string()));
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_docker() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--docker"]).unwrap();
        match cli.command {
            Some(Commands::Off { docker, .. }) => {
                assert!(docker);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_init_shell() {
        let cli = Cli::try_parse_from(["proxydoc", "init", "--shell", "pwsh"]).unwrap();
        match cli.command {
            Some(Commands::Init { shell }) => {
                assert_eq!(shell, Some("pwsh".to_string()));
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_cli_parse_off_all() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "-a"]).unwrap();
        match cli.command {
            Some(Commands::Off { all, .. }) => {
                assert!(all);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_gradle() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "7890", "--gradle"]).unwrap();
        match cli.command {
            Some(Commands::On { target, gradle, .. }) => {
                assert_eq!(target, "7890");
                assert!(gradle);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_on_cargo() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "7890", "--cargo"]).unwrap();
        match cli.command {
            Some(Commands::On { target, cargo, .. }) => {
                assert_eq!(target, "7890");
                assert!(cargo);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_cargo() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--cargo"]).unwrap();
        match cli.command {
            Some(Commands::Off { cargo, .. }) => {
                assert!(cargo);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_off_gradle() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--gradle"]).unwrap();
        match cli.command {
            Some(Commands::Off { gradle, .. }) => {
                assert!(gradle);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_yarn() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--yarn"]).unwrap();
        match cli.command {
            Some(Commands::On { yarn, .. }) => {
                assert!(yarn);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_yarn() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--yarn"]).unwrap();
        match cli.command {
            Some(Commands::Off { yarn, .. }) => {
                assert!(yarn);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_conda() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--conda"]).unwrap();
        match cli.command {
            Some(Commands::On { conda, .. }) => {
                assert!(conda);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_conda() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--conda"]).unwrap();
        match cli.command {
            Some(Commands::Off { conda, .. }) => {
                assert!(conda);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pnpm() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pnpm"]).unwrap();
        match cli.command {
            Some(Commands::On { pnpm, .. }) => {
                assert!(pnpm);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pnpm() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pnpm"]).unwrap();
        match cli.command {
            Some(Commands::Off { pnpm, .. }) => {
                assert!(pnpm);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_bun() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--bun"]).unwrap();
        match cli.command {
            Some(Commands::On { bun, .. }) => {
                assert!(bun);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_bun() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--bun"]).unwrap();
        match cli.command {
            Some(Commands::Off { bun, .. }) => {
                assert!(bun);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_vscode() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--vscode"]).unwrap();
        match cli.command {
            Some(Commands::On { vscode, .. }) => {
                assert!(vscode);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_vscode() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--vscode"]).unwrap();
        match cli.command {
            Some(Commands::Off { vscode, .. }) => {
                assert!(vscode);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_gh() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--gh"]).unwrap();
        match cli.command {
            Some(Commands::On { gh, .. }) => {
                assert!(gh);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_gh() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--gh"]).unwrap();
        match cli.command {
            Some(Commands::Off { gh, .. }) => {
                assert!(gh);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_jetbrains() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--jetbrains"]).unwrap();
        match cli.command {
            Some(Commands::On { jetbrains, .. }) => {
                assert!(jetbrains);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_jetbrains() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--jetbrains"]).unwrap();
        match cli.command {
            Some(Commands::Off { jetbrains, .. }) => {
                assert!(jetbrains);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_nuget() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--nuget"]).unwrap();
        match cli.command {
            Some(Commands::On { nuget, .. }) => {
                assert!(nuget);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_nuget() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--nuget"]).unwrap();
        match cli.command {
            Some(Commands::Off { nuget, .. }) => {
                assert!(nuget);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_curl() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--curl"]).unwrap();
        match cli.command {
            Some(Commands::On { curl, .. }) => {
                assert!(curl);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_curl() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--curl"]).unwrap();
        match cli.command {
            Some(Commands::Off { curl, .. }) => {
                assert!(curl);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_wget() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--wget"]).unwrap();
        match cli.command {
            Some(Commands::On { wget, .. }) => {
                assert!(wget);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_wget() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--wget"]).unwrap();
        match cli.command {
            Some(Commands::Off { wget, .. }) => {
                assert!(wget);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_gem() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--gem"]).unwrap();
        match cli.command {
            Some(Commands::On { gem, .. }) => {
                assert!(gem);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_gem() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--gem"]).unwrap();
        match cli.command {
            Some(Commands::Off { gem, .. }) => {
                assert!(gem);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_svn() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--svn"]).unwrap();
        match cli.command {
            Some(Commands::On { svn, .. }) => {
                assert!(svn);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_svn() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--svn"]).unwrap();
        match cli.command {
            Some(Commands::Off { svn, .. }) => {
                assert!(svn);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_hg() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--hg"]).unwrap();
        match cli.command {
            Some(Commands::On { hg, .. }) => {
                assert!(hg);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_hg() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--hg"]).unwrap();
        match cli.command {
            Some(Commands::Off { hg, .. }) => {
                assert!(hg);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_uv() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--uv"]).unwrap();
        match cli.command {
            Some(Commands::On { uv, .. }) => {
                assert!(uv);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_uv() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--uv"]).unwrap();
        match cli.command {
            Some(Commands::Off { uv, .. }) => {
                assert!(uv);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_go() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--go"]).unwrap();
        match cli.command {
            Some(Commands::On { go, .. }) => {
                assert!(go);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_go() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--go"]).unwrap();
        match cli.command {
            Some(Commands::Off { go, .. }) => {
                assert!(go);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_bazel() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--bazel"]).unwrap();
        match cli.command {
            Some(Commands::On { bazel, .. }) => {
                assert!(bazel);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_bazel() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--bazel"]).unwrap();
        match cli.command {
            Some(Commands::Off { bazel, .. }) => {
                assert!(bazel);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_composer() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--composer"]).unwrap();
        match cli.command {
            Some(Commands::On { composer, .. }) => {
                assert!(composer);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_composer() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--composer"]).unwrap();
        match cli.command {
            Some(Commands::Off { composer, .. }) => {
                assert!(composer);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_conan() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--conan"]).unwrap();
        match cli.command {
            Some(Commands::On { conan, .. }) => {
                assert!(conan);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_conan() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--conan"]).unwrap();
        match cli.command {
            Some(Commands::Off { conan, .. }) => {
                assert!(conan);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_sbt() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--sbt"]).unwrap();
        match cli.command {
            Some(Commands::On { sbt, .. }) => {
                assert!(sbt);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_sbt() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--sbt"]).unwrap();
        match cli.command {
            Some(Commands::Off { sbt, .. }) => {
                assert!(sbt);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_kube() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--kube"]).unwrap();
        match cli.command {
            Some(Commands::On { kube, .. }) => {
                assert!(kube);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_kube() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--kube"]).unwrap();
        match cli.command {
            Some(Commands::Off { kube, .. }) => {
                assert!(kube);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_glab() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--glab"]).unwrap();
        match cli.command {
            Some(Commands::On { glab, .. }) => {
                assert!(glab);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_glab() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--glab"]).unwrap();
        match cli.command {
            Some(Commands::Off { glab, .. }) => {
                assert!(glab);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_julia() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--julia"]).unwrap();
        match cli.command {
            Some(Commands::On { julia, .. }) => {
                assert!(julia);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_julia() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--julia"]).unwrap();
        match cli.command {
            Some(Commands::Off { julia, .. }) => {
                assert!(julia);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_nix() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--nix"]).unwrap();
        match cli.command {
            Some(Commands::On { nix, .. }) => {
                assert!(nix);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_nix() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--nix"]).unwrap();
        match cli.command {
            Some(Commands::Off { nix, .. }) => {
                assert!(nix);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_cabal() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--cabal"]).unwrap();
        match cli.command {
            Some(Commands::On { cabal, .. }) => {
                assert!(cabal);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_cabal() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--cabal"]).unwrap();
        match cli.command {
            Some(Commands::Off { cabal, .. }) => {
                assert!(cabal);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_stack() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--stack"]).unwrap();
        match cli.command {
            Some(Commands::On { stack, .. }) => {
                assert!(stack);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_stack() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--stack"]).unwrap();
        match cli.command {
            Some(Commands::Off { stack, .. }) => {
                assert!(stack);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_hex() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--hex"]).unwrap();
        match cli.command {
            Some(Commands::On { hex, .. }) => {
                assert!(hex);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_hex() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--hex"]).unwrap();
        match cli.command {
            Some(Commands::Off { hex, .. }) => {
                assert!(hex);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_r() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--r"]).unwrap();
        match cli.command {
            Some(Commands::On { r, .. }) => {
                assert!(r);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_r() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--r"]).unwrap();
        match cli.command {
            Some(Commands::Off { r, .. }) => {
                assert!(r);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_opam() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--opam"]).unwrap();
        match cli.command {
            Some(Commands::On { opam, .. }) => {
                assert!(opam);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_opam() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--opam"]).unwrap();
        match cli.command {
            Some(Commands::Off { opam, .. }) => {
                assert!(opam);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_lein() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--lein"]).unwrap();
        match cli.command {
            Some(Commands::On { lein, .. }) => {
                assert!(lein);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_lein() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--lein"]).unwrap();
        match cli.command {
            Some(Commands::Off { lein, .. }) => {
                assert!(lein);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_cpan() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--cpan"]).unwrap();
        match cli.command {
            Some(Commands::On { cpan, .. }) => {
                assert!(cpan);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_cpan() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--cpan"]).unwrap();
        match cli.command {
            Some(Commands::Off { cpan, .. }) => {
                assert!(cpan);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_luarocks() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--luarocks"]).unwrap();
        match cli.command {
            Some(Commands::On { luarocks, .. }) => {
                assert!(luarocks);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_luarocks() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--luarocks"]).unwrap();
        match cli.command {
            Some(Commands::Off { luarocks, .. }) => {
                assert!(luarocks);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_nimble() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--nimble"]).unwrap();
        match cli.command {
            Some(Commands::On { nimble, .. }) => {
                assert!(nimble);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_nimble() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--nimble"]).unwrap();
        match cli.command {
            Some(Commands::Off { nimble, .. }) => {
                assert!(nimble);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_rebar3() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--rebar3"]).unwrap();
        match cli.command {
            Some(Commands::On { rebar3, .. }) => {
                assert!(rebar3);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_rebar3() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--rebar3"]).unwrap();
        match cli.command {
            Some(Commands::Off { rebar3, .. }) => {
                assert!(rebar3);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_scoop() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--scoop"]).unwrap();
        match cli.command {
            Some(Commands::On { scoop, .. }) => {
                assert!(scoop);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_scoop() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--scoop"]).unwrap();
        match cli.command {
            Some(Commands::Off { scoop, .. }) => {
                assert!(scoop);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_bundle() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--bundle"]).unwrap();
        match cli.command {
            Some(Commands::On { bundle, .. }) => {
                assert!(bundle);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_bundle() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--bundle"]).unwrap();
        match cli.command {
            Some(Commands::Off { bundle, .. }) => {
                assert!(bundle);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_gcloud() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--gcloud"]).unwrap();
        match cli.command {
            Some(Commands::On { gcloud, .. }) => {
                assert!(gcloud);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_gcloud() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--gcloud"]).unwrap();
        match cli.command {
            Some(Commands::Off { gcloud, .. }) => {
                assert!(gcloud);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_aws() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--aws"]).unwrap();
        match cli.command {
            Some(Commands::On { aws, .. }) => {
                assert!(aws);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_aws() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--aws"]).unwrap();
        match cli.command {
            Some(Commands::Off { aws, .. }) => {
                assert!(aws);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_azure() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--azure"]).unwrap();
        match cli.command {
            Some(Commands::On { azure, .. }) => {
                assert!(azure);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--az"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { azure, .. }) => {
                assert!(azure);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_azure() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--azure"]).unwrap();
        match cli.command {
            Some(Commands::Off { azure, .. }) => {
                assert!(azure);
            }
            _ => panic!("Expected Off command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "off", "--az"]).unwrap();
        match cli_alias.command {
            Some(Commands::Off { azure, .. }) => {
                assert!(azure);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_mamba() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--mamba"]).unwrap();
        match cli.command {
            Some(Commands::On { mamba, .. }) => {
                assert!(mamba);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--micromamba"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { mamba, .. }) => {
                assert!(mamba);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_mamba() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--mamba"]).unwrap();
        match cli.command {
            Some(Commands::Off { mamba, .. }) => {
                assert!(mamba);
            }
            _ => panic!("Expected Off command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "off", "--micromamba"]).unwrap();
        match cli_alias.command {
            Some(Commands::Off { mamba, .. }) => {
                assert!(mamba);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_podman() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--podman"]).unwrap();
        match cli.command {
            Some(Commands::On { podman, .. }) => {
                assert!(podman);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_podman() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--podman"]).unwrap();
        match cli.command {
            Some(Commands::Off { podman, .. }) => {
                assert!(podman);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_vagrant() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--vagrant"]).unwrap();
        match cli.command {
            Some(Commands::On { vagrant, .. }) => {
                assert!(vagrant);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_vagrant() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--vagrant"]).unwrap();
        match cli.command {
            Some(Commands::Off { vagrant, .. }) => {
                assert!(vagrant);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_choco() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--choco"]).unwrap();
        match cli.command {
            Some(Commands::On { choco, .. }) => {
                assert!(choco);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--chocolatey"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { choco, .. }) => {
                assert!(choco);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_choco() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--choco"]).unwrap();
        match cli.command {
            Some(Commands::Off { choco, .. }) => {
                assert!(choco);
            }
            _ => panic!("Expected Off command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "off", "--chocolatey"]).unwrap();
        match cli_alias.command {
            Some(Commands::Off { choco, .. }) => {
                assert!(choco);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_colima() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--colima"]).unwrap();
        match cli.command {
            Some(Commands::On { colima, .. }) => {
                assert!(colima);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_colima() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--colima"]).unwrap();
        match cli.command {
            Some(Commands::Off { colima, .. }) => {
                assert!(colima);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_minikube() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--minikube"]).unwrap();
        match cli.command {
            Some(Commands::On { minikube, .. }) => {
                assert!(minikube);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_minikube() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--minikube"]).unwrap();
        match cli.command {
            Some(Commands::Off { minikube, .. }) => {
                assert!(minikube);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_spack() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--spack"]).unwrap();
        match cli.command {
            Some(Commands::On { spack, .. }) => {
                assert!(spack);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_spack() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--spack"]).unwrap();
        match cli.command {
            Some(Commands::Off { spack, .. }) => {
                assert!(spack);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_macports() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--macports"]).unwrap();
        match cli.command {
            Some(Commands::On { macports, .. }) => {
                assert!(macports);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_macports() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--macports"]).unwrap();
        match cli.command {
            Some(Commands::Off { macports, .. }) => {
                assert!(macports);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_apt() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--apt"]).unwrap();
        match cli.command {
            Some(Commands::On { apt, .. }) => {
                assert!(apt);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_apt() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--apt"]).unwrap();
        match cli.command {
            Some(Commands::Off { apt, .. }) => {
                assert!(apt);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_dnf() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--dnf"]).unwrap();
        match cli.command {
            Some(Commands::On { dnf, .. }) => {
                assert!(dnf);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_dnf() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--dnf"]).unwrap();
        match cli.command {
            Some(Commands::Off { dnf, .. }) => {
                assert!(dnf);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_zypper() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--zypper"]).unwrap();
        match cli.command {
            Some(Commands::On { zypper, .. }) => {
                assert!(zypper);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_zypper() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--zypper"]).unwrap();
        match cli.command {
            Some(Commands::Off { zypper, .. }) => {
                assert!(zypper);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pacman() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pacman"]).unwrap();
        match cli.command {
            Some(Commands::On { pacman, .. }) => {
                assert!(pacman);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pacman() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pacman"]).unwrap();
        match cli.command {
            Some(Commands::Off { pacman, .. }) => {
                assert!(pacman);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pkg() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pkg"]).unwrap();
        match cli.command {
            Some(Commands::On { pkg, .. }) => {
                assert!(pkg);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pkg() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pkg"]).unwrap();
        match cli.command {
            Some(Commands::Off { pkg, .. }) => {
                assert!(pkg);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_portage() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--portage"]).unwrap();
        match cli.command {
            Some(Commands::On { portage, .. }) => {
                assert!(portage);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_portage() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--emerge"]).unwrap();
        match cli.command {
            Some(Commands::Off { portage, .. }) => {
                assert!(portage);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_docker_daemon() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--docker-daemon"]).unwrap();
        match cli.command {
            Some(Commands::On { docker_daemon, .. }) => {
                assert!(docker_daemon);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_docker_daemon() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--daemon"]).unwrap();
        match cli.command {
            Some(Commands::Off { docker_daemon, .. }) => {
                assert!(docker_daemon);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_flatpak() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--flatpak"]).unwrap();
        match cli.command {
            Some(Commands::On { flatpak, .. }) => {
                assert!(flatpak);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_flatpak() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--ostree"]).unwrap();
        match cli.command {
            Some(Commands::Off { flatpak, .. }) => {
                assert!(flatpak);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_snap() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--snap"]).unwrap();
        match cli.command {
            Some(Commands::On { snap, .. }) => {
                assert!(snap);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_snap() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--snapd"]).unwrap();
        match cli.command {
            Some(Commands::Off { snap, .. }) => {
                assert!(snap);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_ollama() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--ollama"]).unwrap();
        match cli.command {
            Some(Commands::On { ollama, .. }) => {
                assert!(ollama);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_ollama() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--ollamad"]).unwrap();
        match cli.command {
            Some(Commands::Off { ollama, .. }) => {
                assert!(ollama);
            }
            _ => panic!("Expected Off command"),
        }
    }
}
