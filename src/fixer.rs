use anyhow::Result;
use colored::*;
use std::process::Command;

use crate::doctor::{
    clean_helm_repositories, clean_jsonc, clean_swiftpm_mirrors, clean_terraform_network_mirrors,
    clean_vcpkg_configuration_registries, get_bazel_proxy, get_bun_proxy, get_cabal_proxy,
    get_cargo_proxy, get_composer_proxy, get_conan_proxy, get_conda_proxy, get_cpan_proxy,
    get_curlrc_proxy, get_docker_proxy, get_gem_proxy, get_gemrc_path, get_gh_proxy,
    get_glab_proxy, get_goproxy, get_gradle_proxy, get_helm_repositories_paths, get_hex_proxy,
    get_hg_proxy, get_hgrc_path, get_ide_proxies, get_jetbrains_proxies, get_julia_startup_proxy,
    get_kubeconfig_path, get_lein_proxy, get_luarocks_proxy, get_maven_proxy, get_nimble_proxy,
    get_nix_proxy, get_npm_config, get_npm_registries, get_nuget_proxy, get_opam_proxy,
    get_pip_proxy, get_pnpm_proxy, get_rebar3_proxy, get_renviron_proxy, get_sbt_proxy,
    get_scoop_proxy, get_ssh_config_proxies, get_stack_proxy, get_svn_proxy,
    get_svn_servers_path, get_swiftpm_mirrors_path, get_system_proxy, get_terraform_config_paths,
    get_uv_config_path, get_uv_proxy, get_vcpkg_configuration_paths, get_wgetrc_proxy,
    get_yarn_proxy, parse_helm_repositories, parse_kubeconfig_proxies, parse_swiftpm_mirrors,
    parse_terraform_network_mirrors, parse_vcpkg_configuration_registries, verify_proxy_url,
    get_bundler_proxy, get_gcloud_proxy, get_aws_proxy, get_azure_proxy, get_podman_proxy,
    get_vagrant_proxy, get_choco_proxy, get_colima_proxy, get_minikube_proxy, get_spack_proxy,
    get_macports_proxy, get_apt_proxy, get_dnf_proxy, get_zypper_proxy,
    get_pacman_proxy, get_pkg_proxy, get_portage_proxy, get_docker_daemon_proxy,
    get_flatpak_proxy, get_snap_proxy, get_ollama_proxy, get_containerd_proxy, get_buildkit_proxy, get_k3s_proxy, get_apk_repositories_paths,
    parse_apk_repositories, clean_apk_repositories,
};
use crate::manager::{
    deactivate_jetbrains_proxy_xml, remove_vscode_proxy_json, unset_bazel_proxy, unset_bun_proxy,
    unset_cabal_proxy, unset_composer_proxy, unset_conan_proxy, unset_conda_proxy,
    unset_cpan_proxy, unset_docker_proxy, unset_gem_proxy, unset_gh_proxy, unset_glab_proxy,
    unset_go_proxy, unset_gradle_proxy, unset_hex_proxy, unset_hg_proxy, unset_julia_proxy,
    unset_kube_proxy, unset_lein_proxy, unset_luarocks_proxy, unset_maven_proxy,
    unset_nimble_proxy, unset_nix_proxy, unset_nuget_proxy, unset_opam_proxy, unset_pnpm_proxy,
    unset_rebar3_proxy, unset_renviron_proxy, unset_sbt_proxy, unset_scoop_proxy,
    unset_stack_proxy, unset_svn_proxy, unset_uv_proxy, unset_yarn_proxy, unset_bundler_proxy,
    unset_gcloud_proxy, unset_aws_proxy, unset_azure_proxy, unset_mamba_proxy, unset_podman_proxy,
    unset_vagrant_proxy, unset_choco_proxy, unset_colima_proxy, unset_minikube_proxy,
    unset_spack_proxy, unset_macports_proxy, unset_apt_proxy, unset_dnf_proxy, unset_zypper_proxy,
    unset_pacman_proxy, unset_pkg_proxy, unset_portage_proxy, unset_docker_daemon_proxy,
    unset_flatpak_proxy, unset_snap_proxy, unset_ollama_proxy, unset_containerd_proxy,
    unset_buildkit_proxy, unset_k3s_proxy,
};
use crate::scanner;

pub fn comment_out_dead_lines(
    content: &str,
    is_dead_proxy_line: impl Fn(&str) -> bool,
) -> (String, bool) {
    let mut modified = false;
    let mut out_lines = Vec::new();

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && is_dead_proxy_line(trim) {
            out_lines.push(format!("# [proxydoc disabled dead proxy]: {}", line));
            modified = true;
        } else {
            out_lines.push(line.to_string());
        }
    }

    (out_lines.join("\n") + "\n", modified)
}

pub async fn run() -> Result<()> {
    println!(
        "\n{}",
        "🩹 ==================== PROXY AUTO-REPAIR ==================== 🩹"
            .bold()
            .cyan()
    );
    println!(
        "{}",
        "   Detecting anomalies and automatically applying fixes...\n".dimmed()
    );

    let mut fixed_count = 0;

    // 1. Clean dead Git proxy
    let git_http = get_git_config("http.proxy");
    let git_https = get_git_config("https.proxy");
    let mut git_cleaned = false;

    if let Some(val) = git_http {
        if !verify_proxy_url(&val).await {
            println!(
                "  [1/73] Found dead Git http.proxy (`{}`), cleaning...",
                val.yellow()
            );
            let _ = Command::new("git")
                .args(["config", "--global", "--unset", "http.proxy"])
                .status();
            git_cleaned = true;
            fixed_count += 1;
        }
    }
    if let Some(val) = git_https {
        if !verify_proxy_url(&val).await {
            println!(
                "  [1/73] Found dead Git https.proxy (`{}`), cleaning...",
                val.yellow()
            );
            let _ = Command::new("git")
                .args(["config", "--global", "--unset", "https.proxy"])
                .status();
            git_cleaned = true;
            fixed_count += 1;
        }
    }

    if let Some(val) = get_git_config("http.sslCAInfo") {
        if !std::path::Path::new(&val).exists() {
            println!("  [1/73] Found broken Git http.sslCAInfo pointing to missing file (`{}`), cleaning...", val.yellow());
            let _ = Command::new("git")
                .args(["config", "--global", "--unset", "http.sslCAInfo"])
                .status();
            git_cleaned = true;
            fixed_count += 1;
        }
    }

    if git_cleaned {
        println!("  {} Cleared broken Git global proxy.", "✔".green());
    } else {
        println!("  [1/73] Git global proxy: {}", "Clean or healthy".dimmed());
    }

    // 2. Clean dead NPM proxy
    let npm_http = get_npm_config("proxy");
    let npm_https = get_npm_config("https-proxy");
    let mut npm_cleaned = false;

    if let Some(val) = npm_http {
        if !verify_proxy_url(&val).await {
            println!(
                "  [2/73] Found dead NPM proxy (`{}`), cleaning...",
                val.yellow()
            );
            let _ = Command::new("npm")
                .args(["config", "delete", "proxy"])
                .status();
            npm_cleaned = true;
            fixed_count += 1;
        }
    }
    if let Some(val) = npm_https {
        if !verify_proxy_url(&val).await {
            println!(
                "  [2/73] Found dead NPM https-proxy (`{}`), cleaning...",
                val.yellow()
            );
            let _ = Command::new("npm")
                .args(["config", "delete", "https-proxy"])
                .status();
            npm_cleaned = true;
            fixed_count += 1;
        }
    }

    if npm_cleaned {
        println!("  {} Cleared broken NPM proxy.", "✔".green());
    } else {
        println!("  [2/73] NPM proxy: {}", "Clean or healthy".dimmed());
    }

    // 3. Clean dead Yarn proxy (~/.yarnrc / ~/.yarnrc.yml)
    if let Some(cfg) = get_yarn_proxy() {
        if !verify_proxy_url(&cfg.proxy).await {
            println!(
                "  [3/73] Found dead Yarn proxy (`{}` in {}), cleaning...",
                cfg.proxy.yellow(),
                cfg.source
            );
            if let Ok(true) = unset_yarn_proxy() {
                println!("  {} Cleared broken Yarn global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [3/73] Yarn proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [3/73] Yarn proxy: {}", "Clean or not set".dimmed());
    }

    // 4. Clean dead pnpm proxy (~/.pnpmrc)
    if let Some(val) = get_pnpm_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [4/73] Found dead pnpm proxy (`{}` in ~/.pnpmrc), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_pnpm_proxy() {
                println!("  {} Cleared broken pnpm proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [4/73] pnpm proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [4/73] pnpm proxy: {}", "Clean or not set".dimmed());
    }

    // 5. Clean dead Bun proxy (~/.bunfig.toml)
    if let Some(val) = get_bun_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [5/73] Found dead Bun proxy (`{}` in ~/.bunfig.toml), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_bun_proxy() {
                println!("  {} Cleared broken Bun proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [5/73] Bun proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [5/73] Bun proxy: {}", "Clean or not set".dimmed());
    }

    // 6. Clean dead Pip proxy
    if let Some(val) = get_pip_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [6/73] Found dead Pip global.proxy (`{}`), cleaning...",
                val.yellow()
            );
            let _ = Command::new("pip")
                .args(["config", "unset", "global.proxy"])
                .status();
            println!("  {} Cleared broken Pip global proxy.", "✔".green());
            fixed_count += 1;
        } else {
            println!("  [6/73] Pip global proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [6/73] Pip global proxy: {}", "Clean or not set".dimmed());
    }

    // 7. Clean dead Conda proxy (~/.condarc)
    if let Some(cfg) = get_conda_proxy() {
        let mut dead = false;
        if let Some(ref http) = cfg.http {
            if !verify_proxy_url(http).await {
                dead = true;
            }
        }
        if let Some(ref https) = cfg.https {
            if !verify_proxy_url(https).await {
                dead = true;
            }
        }

        if dead {
            println!("  [7/73] Found dead Conda proxy in ~/.condarc, cleaning...");
            if let Ok(true) = unset_conda_proxy() {
                println!(
                    "  {} Cleared broken Conda proxy in ~/.condarc.",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [7/73] Conda proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [7/73] Conda proxy: {}", "Clean or not set".dimmed());
    }

    // 8. Clean dead Docker client proxy (~/.docker/config.json)
    if let Some(val) = get_docker_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [8/73] Found dead Docker proxy (`{}`), cleaning ~/.docker/config.json...",
                val.yellow()
            );
            if let Ok(true) = unset_docker_proxy() {
                println!("  {} Cleared broken Docker client proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [8/73] Docker client proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [8/73] Docker client proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 9. Clean dead Gradle proxy (~/.gradle/gradle.properties)
    if let Some(cfg) = get_gradle_proxy() {
        if !scanner::check_host_port(&cfg.host, cfg.port).await {
            println!(
                "  [9/73] Found dead Gradle proxy (`{}:{}`), removing from ~/.gradle/gradle.properties...",
                cfg.host.yellow(),
                cfg.port
            );
            if let Ok(true) = unset_gradle_proxy() {
                println!("  {} Cleared broken Gradle global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [9/73] Gradle global proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [9/73] Gradle global proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 10. Clean dead Maven proxy (~/.m2/settings.xml)
    if let Some(cfg) = get_maven_proxy() {
        if !scanner::check_host_port(&cfg.host, cfg.port).await {
            println!(
                "  [10/73] Found dead Maven proxy (`{}:{}`), deactivating in ~/.m2/settings.xml...",
                cfg.host.yellow(),
                cfg.port
            );
            if let Ok(true) = unset_maven_proxy() {
                println!("  {} Deactivated broken Maven global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [10/73] Maven global proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [10/73] Maven global proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 11. Clean dead NuGet (.NET) proxy
    if let Some((val, path)) = get_nuget_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [11/73] Found dead NuGet proxy (`{}` in {}), cleaning...",
                val.yellow(),
                path.display()
            );
            if let Ok(true) = unset_nuget_proxy() {
                println!("  {} Cleared broken NuGet global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!(
                "  [11/73] NuGet (.NET) global proxy: {}",
                "Reachable".green()
            );
        }
    } else {
        println!(
            "  [11/73] NuGet (.NET) global proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 12. Clean dead VS Code / Cursor IDE proxies
    let ide_proxies = get_ide_proxies();
    let mut ide_cleaned = false;
    for info in ide_proxies {
        if let Some(ref val) = info.proxy {
            if !verify_proxy_url(val).await {
                println!(
                    "  [12/73] Found dead {} proxy (`{}`), cleaning {}...",
                    info.ide_name,
                    val.yellow(),
                    info.path.display()
                );
                let bak = info.path.with_extension("json.bak");
                let _ = std::fs::copy(&info.path, bak);

                if let Ok(content) = std::fs::read_to_string(&info.path) {
                    let cleaned = clean_jsonc(&content);
                    if let Ok(mut root) = serde_json::from_str::<serde_json::Value>(&cleaned) {
                        if remove_vscode_proxy_json(&mut root) {
                            if let Ok(out) = serde_json::to_string_pretty(&root) {
                                let _ = std::fs::write(&info.path, out);
                                ide_cleaned = true;
                                fixed_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    if ide_cleaned {
        println!("  {} Cleared broken IDE proxy settings.", "✔".green());
    } else {
        println!(
            "  [12/73] VS Code & Cursor IDE settings: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 12. Clean dead JetBrains / Android Studio IDE proxies
    let jb_proxies = get_jetbrains_proxies();
    let mut jb_cleaned = false;
    for info in jb_proxies {
        let is_dead = if info.use_proxy {
            if let (Some(ref host), Some(port)) = (&info.host, info.port) {
                !scanner::check_host_port(host, port).await
            } else {
                false
            }
        } else if info.use_pac {
            if let Some(ref pac) = info.pac_url {
                !verify_proxy_url(pac).await
            } else {
                false
            }
        } else {
            false
        };

        if is_dead {
            println!(
                "  [13/73] Found dead {} proxy in {}, deactivating...",
                info.ide_name,
                info.path.display()
            );
            let bak = info.path.with_extension("xml.bak");
            let _ = std::fs::copy(&info.path, bak);

            if let Ok(content) = std::fs::read_to_string(&info.path) {
                let (new_c, changed) = deactivate_jetbrains_proxy_xml(&content);
                if changed {
                    let _ = std::fs::write(&info.path, new_c);
                    jb_cleaned = true;
                    fixed_count += 1;
                }
            }
        }
    }

    if jb_cleaned {
        println!(
            "  {} Deactivated broken JetBrains / Android Studio proxy settings.",
            "✔".green()
        );
    } else {
        println!(
            "  [13/73] JetBrains & Android Studio settings: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 13. Clean dead GitHub CLI (gh) proxy
    if let Some(val) = get_gh_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [14/73] Found dead GitHub CLI (gh) proxy (`{}`), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_gh_proxy() {
                println!("  {} Cleared broken GitHub CLI (gh) proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [14/73] GitHub CLI (gh) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [14/73] GitHub CLI (gh) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 14. Clean dead cURL (~/.curlrc), Wget (~/.wgetrc) & Cargo proxy
    let mut static_fixed = false;
    if let Some(home) = dirs::home_dir() {
        let curlrc_path = home.join(".curlrc");
        if let Some(val) = get_curlrc_proxy() {
            if !verify_proxy_url(&val).await {
                if let Ok(content) = std::fs::read_to_string(&curlrc_path) {
                    println!("  [15/73] Found dead proxy `{}` in ~/.curlrc, commenting out with backup...", val.yellow());
                    let (new_c, changed) = comment_out_dead_lines(&content, |l| {
                        l.starts_with("proxy") || l.starts_with("-x") || l.starts_with("--proxy")
                    });
                    if changed {
                        let _ = std::fs::copy(&curlrc_path, home.join(".curlrc.bak"));
                        let _ = std::fs::write(&curlrc_path, new_c);
                        println!(
                            "  {} Disabled dead proxy in ~/.curlrc (backup saved to ~/.curlrc.bak)",
                            "✔".green()
                        );
                        static_fixed = true;
                        fixed_count += 1;
                    }
                }
            }
        }

        let wgetrc_path = home.join(".wgetrc");
        if let Some(val) = get_wgetrc_proxy() {
            if !verify_proxy_url(&val).await {
                if let Ok(content) = std::fs::read_to_string(&wgetrc_path) {
                    println!("  [15/73] Found dead proxy `{}` in ~/.wgetrc, commenting out with backup...", val.yellow());
                    let (new_c, changed) = comment_out_dead_lines(&content, |l| {
                        l.to_ascii_lowercase().starts_with("http_proxy")
                            || l.to_ascii_lowercase().starts_with("https_proxy")
                    });
                    if changed {
                        let _ = std::fs::copy(&wgetrc_path, home.join(".wgetrc.bak"));
                        let _ = std::fs::write(&wgetrc_path, new_c);
                        println!(
                            "  {} Disabled dead proxy in ~/.wgetrc (backup saved to ~/.wgetrc.bak)",
                            "✔".green()
                        );
                        static_fixed = true;
                        fixed_count += 1;
                    }
                }
            }
        }
    }

    if let Some(val) = get_cargo_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [15/73] Found dead Cargo proxy (`{}`), cleaning ~/.cargo/config.toml...",
                val.yellow()
            );
            if let Ok(true) = crate::manager::unset_cargo_proxy() {
                println!("  {} Cleared broken Cargo global proxy.", "✔".green());
                static_fixed = true;
                fixed_count += 1;
            } else {
                println!(
                    "  [15/73] {} Cargo config has dead proxy `{}`! Please remove `proxy` in ~/.cargo/config.toml",
                    "▲".yellow(),
                    val.yellow()
                );
                static_fixed = true;
            }
        }
    }

    if !static_fixed {
        println!(
            "  [15/73] cURL, Wget, and Cargo configs: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 16. Clean dead Gem (~/.gemrc) proxy
    if let Some(val) = get_gem_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [16/73] Found dead Gem proxy (`{}` in ~/.gemrc), cleaning with backup...",
                val.yellow()
            );
            let path = get_gemrc_path();
            if path.exists() {
                let _ = std::fs::copy(&path, path.with_extension("gemrc.bak"));
            }
            if let Ok(true) = unset_gem_proxy() {
                println!(
                    "  {} Cleared broken Gem proxy in ~/.gemrc (backup saved to ~/.gemrc.bak)",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [16/73] Gem (Ruby) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [16/73] Gem (Ruby) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 17. Clean dead Subversion (~/.subversion/servers) proxy
    if let Some(val) = get_svn_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [17/73] Found dead Subversion proxy (`{}` in servers), cleaning with backup...",
                val.yellow()
            );
            let path = get_svn_servers_path();
            if path.exists() {
                let _ = std::fs::copy(&path, path.with_extension("servers.bak"));
            }
            if let Ok(true) = unset_svn_proxy() {
                println!(
                    "  {} Cleared broken Subversion proxy in servers config (backup saved)",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [17/73] Subversion (SVN) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [17/73] Subversion (SVN) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 18. Clean dead Mercurial (~/.hgrc / Mercurial.ini) proxy
    if let Some(val) = get_hg_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [18/73] Found dead Mercurial proxy (`{}` in ~/.hgrc), cleaning with backup...",
                val.yellow()
            );
            let path = get_hgrc_path();
            if path.exists() {
                let _ = std::fs::copy(&path, path.with_extension("hgrc.bak"));
            }
            if let Ok(true) = unset_hg_proxy() {
                println!(
                    "  {} Cleared broken Mercurial proxy in ~/.hgrc (backup saved)",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [18/73] Mercurial (Hg) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [18/73] Mercurial (Hg) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 19. Migrate deprecated registry.npm.taobao.org in NPM / Yarn / pnpm configs
    let node_registries = get_npm_registries();
    let mut taobao_migrated = false;
    for reg in node_registries {
        if reg.url.contains("registry.npm.taobao.org") {
            if let Some(ref path) = reg.path {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if content.contains("registry.npm.taobao.org") {
                        let backup = path.with_extension("registry.bak");
                        let _ = std::fs::write(&backup, &content);
                        let updated =
                            content.replace("registry.npm.taobao.org", "registry.npmmirror.com");
                        if std::fs::write(path, updated).is_ok() {
                            println!(
                                "  {} Migrated deprecated 'registry.npm.taobao.org' -> 'https://registry.npmmirror.com' in {}",
                                "✔".green(),
                                path.display()
                            );
                            taobao_migrated = true;
                            fixed_count += 1;
                        }
                    }
                }
            }
        }
    }
    if !taobao_migrated {
        println!(
            "  [19/73] Node / JS Registry mirrors: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 20. Inspect SSH config (~/.ssh/config) ProxyCommands
    let ssh_proxies = get_ssh_config_proxies();
    let mut ssh_has_dead = false;
    for cmd in &ssh_proxies {
        if let Some(ref target) = cmd.target {
            if !verify_proxy_url(target).await {
                println!(
                    "  [20/73] {} ~/.ssh/config Host `{}` has dead ProxyCommand target `{}`! (Git clone over SSH will hang)",
                    "▲".red().bold(),
                    cmd.host_pattern.yellow(),
                    target.yellow()
                );
                ssh_has_dead = true;
            }
        }
    }
    if !ssh_has_dead {
        println!(
            "  [20/73] SSH ProxyCommand configs: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 21. Clean dead Astral uv (~/.config/uv/uv.toml) proxy
    if let Some(val) = get_uv_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [21/73] Found dead Astral uv proxy (`{}` in uv.toml), cleaning with backup...",
                val.yellow()
            );
            let path = get_uv_config_path();
            if path.exists() {
                let _ = std::fs::copy(&path, path.with_extension("toml.bak"));
            }
            if let Ok(true) = unset_uv_proxy() {
                println!(
                    "  {} Cleared broken Astral uv proxy in uv.toml (backup saved)",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [21/73] Astral uv proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [21/73] Astral uv proxy: {}", "Clean or not set".dimmed());
    }

    // 22. Clean dead Go GOPROXY mirror
    if let Some(gp) = get_goproxy() {
        let mirrors = crate::doctor::parse_goproxy(&gp);
        let mut dead_found = false;
        for m in &mirrors {
            if !verify_proxy_url(m).await {
                dead_found = true;
                break;
            }
        }
        if dead_found {
            println!(
                "  [22/73] Found dead GOPROXY mirror (`{}`), cleaning / resetting to default...",
                gp.yellow()
            );
            if let Ok(true) = unset_go_proxy() {
                println!(
                    "  {} Reset broken GOPROXY to default (https://proxy.golang.org,direct)",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [22/73] Go GOPROXY: {}", "Reachable or direct".green());
        }
    } else {
        println!("  [22/73] Go GOPROXY: {}", "Clean or default".dimmed());
    }

    // 23. Clean dead Bazel (~/.bazelrc) action_env proxy
    if let Some((val, path)) = get_bazel_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [23/73] Found dead Bazel action_env proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bazelrc.bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_bazel_proxy() {
                println!(
                    "  {} Cleared broken Bazel action_env proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [23/73] Bazel proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [23/73] Bazel proxy: {}", "Clean or not set".dimmed());
    }

    // 24. Clean dead PHP Composer (config.json) proxy
    if let Some((val, path)) = get_composer_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [24/73] Found dead PHP Composer proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("composer.bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_composer_proxy() {
                println!(
                    "  {} Cleared broken PHP Composer proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [24/73] PHP Composer proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [24/73] PHP Composer proxy: {}", "Clean or not set".dimmed());
    }

    // 25. Clean dead Conan C/C++ (global.conf / conan.conf) proxy
    if let Some((val, path)) = get_conan_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [25/73] Found dead Conan C/C++ proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("conan.bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_conan_proxy() {
                println!(
                    "  {} Cleared broken Conan C/C++ proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [25/73] Conan C/C++ proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [25/73] Conan C/C++ proxy: {}", "Clean or not set".dimmed());
    }

    // 26. Clean dead Scala Build Tool (~/.sbt/sbtopts) proxy
    if let Some((val, path)) = get_sbt_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [26/73] Found dead Scala (sbt) proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("sbtopts.bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_sbt_proxy() {
                println!(
                    "  {} Cleared broken Scala (sbt) proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [26/73] Scala (sbt) proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [26/73] Scala (sbt) proxy: {}", "Clean or not set".dimmed());
    }

    // 27. Clean dead Kubernetes (~/.kube/config) cluster proxy-url
    if let Some(kpath) = get_kubeconfig_path() {
        if let Ok(content) = std::fs::read_to_string(&kpath) {
            let kube_proxies = parse_kubeconfig_proxies(&content);
            let mut dead_cluster = false;
            for (_cluster, purl) in &kube_proxies {
                if !verify_proxy_url(purl).await {
                    dead_cluster = true;
                    break;
                }
            }
            if dead_cluster {
                println!(
                    "  [27/73] Found dead cluster proxy-url in {}, cleaning with backup...",
                    kpath.display()
                );
                let backup = kpath.with_extension("bak");
                let _ = std::fs::copy(&kpath, &backup);
                if let Ok(true) = unset_kube_proxy() {
                    println!(
                        "  {} Cleared broken Kubernetes proxy-url in {} (backup saved)",
                        "✔".green(),
                        kpath.display()
                    );
                    fixed_count += 1;
                }
            } else if !kube_proxies.is_empty() {
                println!("  [27/73] Kubernetes (kubectl) proxy-url: {}", "Reachable".green());
            } else {
                println!("  [27/73] Kubernetes (kubectl) proxy-url: {}", "Clean or not set".dimmed());
            }
        }
    } else {
        println!("  [27/73] Kubernetes (kubectl) proxy-url: {}", "Clean or not set".dimmed());
    }

    // 28. Clean dead Julia startup (~/.julia/config/startup.jl) proxy
    if let Some((val, path)) = get_julia_startup_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [28/73] Found dead Julia startup proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("startup.bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_julia_proxy() {
                println!(
                    "  {} Cleared broken Julia startup proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [28/73] Julia startup proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [28/73] Julia startup proxy: {}", "Clean or not set".dimmed());
    }

    // 29. Clean dead GitLab CLI (~/.config/glab-cli/config.yml) proxy
    if let Some(val) = get_glab_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [29/73] Found dead GitLab CLI (glab) proxy (`{}`), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_glab_proxy() {
                println!("  {} Cleared broken GitLab CLI (glab) proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [29/73] GitLab CLI (glab) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [29/73] GitLab CLI (glab) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 30. Clean dead Nix (~/.config/nix/nix.conf) http-proxy
    if let Some((val, path)) = get_nix_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [30/73] Found dead Nix http-proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_nix_proxy() {
                println!(
                    "  {} Cleared broken Nix http-proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [30/73] Nix (nix.conf) http-proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [30/73] Nix (nix.conf) http-proxy: {}", "Clean or not set".dimmed());
    }

    // 31. Clean dead Haskell Cabal (~/.cabal/config) proxy
    if let Some((val, path)) = get_cabal_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [31/73] Found dead Haskell Cabal proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_cabal_proxy() {
                println!(
                    "  {} Cleared broken Haskell Cabal proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [31/73] Haskell Cabal proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [31/73] Haskell Cabal proxy: {}", "Clean or not set".dimmed());
    }

    // 32. Clean dead Haskell Stack (~/.stack/config.yaml) proxy
    if let Some((val, path)) = get_stack_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [32/73] Found dead Haskell Stack proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_stack_proxy() {
                println!(
                    "  {} Cleared broken Haskell Stack proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [32/73] Haskell Stack proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [32/73] Haskell Stack proxy: {}", "Clean or not set".dimmed());
    }

    // 33. Clean dead Elixir / Erlang / Gleam Hex (~/.hex/hex.config) proxy
    if let Some((val, path)) = get_hex_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [33/73] Found dead Hex package proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_hex_proxy() {
                println!(
                    "  {} Cleared broken Hex proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [33/73] Hex (Elixir/Gleam) proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [33/73] Hex (Elixir/Gleam) proxy: {}", "Clean or not set".dimmed());
    }

    // 34. Clean dead SwiftPM (~/.swiftpm/configuration/mirrors.json) package mirrors
    if let Some(mpath) = get_swiftpm_mirrors_path() {
        if let Ok(content) = std::fs::read_to_string(&mpath) {
            let mirrors = parse_swiftpm_mirrors(&content);
            let mut dead_mirrors = Vec::new();
            for m in &mirrors {
                if !verify_proxy_url(m).await {
                    dead_mirrors.push(m.as_str());
                }
            }
            if !dead_mirrors.is_empty() {
                println!(
                    "  [34/73] Found {} dead SwiftPM mirror(s) in {}, cleaning with backup...",
                    dead_mirrors.len().to_string().yellow(),
                    mpath.display()
                );
                let (new_json, modified) = clean_swiftpm_mirrors(&content, &dead_mirrors);
                if modified {
                    let backup = mpath.with_extension("bak");
                    let _ = std::fs::copy(&mpath, &backup);
                    let _ = std::fs::write(&mpath, new_json);
                    println!(
                        "  {} Pruned dead SwiftPM mirror(s) from {} (backup saved)",
                        "✔".green(),
                        mpath.display()
                    );
                    fixed_count += 1;
                }
            } else if !mirrors.is_empty() {
                println!("  [34/73] SwiftPM mirrors: {}", "Reachable".green());
            } else {
                println!("  [34/73] SwiftPM mirrors: {}", "Clean or empty".dimmed());
            }
        }
    } else {
        println!("  [34/73] SwiftPM mirrors: {}", "Clean or not set".dimmed());
    }

    // 35. Clean dead R package manager (~/.Renviron) proxy
    if let Some((val, path)) = get_renviron_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [35/73] Found dead R proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_renviron_proxy() {
                println!(
                    "  {} Cleared broken R proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [35/73] R (~/.Renviron) proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [35/73] R (~/.Renviron) proxy: {}", "Clean or not set".dimmed());
    }

    // 36. Clean dead OCaml OPAM (~/.opam/config) proxy
    if let Some((val, path)) = get_opam_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [36/73] Found dead OCaml OPAM proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_opam_proxy() {
                println!(
                    "  {} Cleared broken OCaml OPAM proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [36/73] OCaml OPAM proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [36/73] OCaml OPAM proxy: {}", "Clean or not set".dimmed());
    }

    // 37. Clean dead Terraform / OpenTofu (~/.terraformrc / ~/.tofurc) network mirrors
    let tf_paths = get_terraform_config_paths();
    if !tf_paths.is_empty() {
        for tpath in tf_paths {
            if let Ok(content) = std::fs::read_to_string(&tpath) {
                let mirrors = parse_terraform_network_mirrors(&content);
                let mut dead_mirrors = Vec::new();
                for mirror in &mirrors {
                    if !verify_proxy_url(mirror).await {
                        dead_mirrors.push(mirror.as_str());
                    }
                }
                if !dead_mirrors.is_empty() {
                    println!(
                        "  [37/73] Found {} dead Terraform/OpenTofu mirror(s) in {}, cleaning with backup...",
                        dead_mirrors.len().to_string().yellow(),
                        tpath.display()
                    );
                    let (new_content, modified) =
                        clean_terraform_network_mirrors(&content, &dead_mirrors);
                    if modified {
                        let backup = tpath.with_extension("bak");
                        let _ = std::fs::copy(&tpath, &backup);
                        let _ = std::fs::write(&tpath, new_content);
                        println!(
                            "  {} Disabled dead Terraform/OpenTofu mirror(s) in {} (backup saved)",
                            "✔".green(),
                            tpath.display()
                        );
                        fixed_count += 1;
                    }
                } else if !mirrors.is_empty() {
                    println!("  [37/73] Terraform/OpenTofu network mirrors: {}", "Reachable".green());
                } else {
                    println!("  [37/73] Terraform/OpenTofu network mirrors: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [37/73] Terraform/OpenTofu network mirrors: {}", "Clean or not set".dimmed());
    }

    // 38. Clean dead Clojure Leiningen (~/.lein/profiles.clj) proxy
    if let Some((val, path)) = get_lein_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [38/73] Found dead Clojure Leiningen proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_lein_proxy() {
                println!(
                    "  {} Cleared broken Clojure Leiningen proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [38/73] Clojure Leiningen proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [38/73] Clojure Leiningen proxy: {}", "Clean or not set".dimmed());
    }

    // 39. Clean dead Perl CPAN (~/.cpan/CPAN/MyConfig.pm) proxy
    if let Some((val, path)) = get_cpan_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [39/73] Found dead Perl CPAN proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_cpan_proxy() {
                println!(
                    "  {} Cleared broken Perl CPAN proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [39/73] Perl CPAN proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [39/73] Perl CPAN proxy: {}", "Clean or not set".dimmed());
    }

    // 40. Clean dead LuaRocks (~/.luarocks/config.lua) proxy
    if let Some((val, path)) = get_luarocks_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [40/73] Found dead LuaRocks proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_luarocks_proxy() {
                println!(
                    "  {} Cleared broken LuaRocks proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [40/73] LuaRocks proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [40/73] LuaRocks proxy: {}", "Clean or not set".dimmed());
    }

    // 41. Clean dead Nim (Nimble, ~/.config/nimble/nimble.ini) proxy
    if let Some((val, path)) = get_nimble_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [41/73] Found dead Nimble proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_nimble_proxy() {
                println!(
                    "  {} Cleared broken Nimble proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [41/73] Nimble proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [41/73] Nimble proxy: {}", "Clean or not set".dimmed());
    }

    // 42. Clean dead Erlang (Rebar3, ~/.config/rebar3/rebar.config) proxy
    if let Some((val, path)) = get_rebar3_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [42/73] Found dead Rebar3 proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_rebar3_proxy() {
                println!(
                    "  {} Cleared broken Rebar3 proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [42/73] Rebar3 proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [42/73] Rebar3 proxy: {}", "Clean or not set".dimmed());
    }

    // 43. Clean dead Scoop (~/.config/scoop/config.json) proxy
    if let Some((val, path)) = get_scoop_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [43/73] Found dead Scoop proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_scoop_proxy() {
                println!(
                    "  {} Cleared broken Scoop proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [43/73] Scoop proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [43/73] Scoop proxy: {}", "Clean or not set".dimmed());
    }

    // 44. Clean dead Ruby Bundler (~/.bundle/config) proxy
    if let Some((val, path)) = get_bundler_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [44/73] Found dead Bundler proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_bundler_proxy() {
                println!(
                    "  {} Cleared broken Bundler proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [44/73] Bundler proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [44/73] Bundler proxy: {}", "Clean or not set".dimmed());
    }

    // 45. Clean dead Google Cloud SDK (gcloud) proxy (~/.config/gcloud/configurations/config_*)
    if let Some((val, path)) = get_gcloud_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [45/73] Found dead Google Cloud SDK (gcloud) proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_gcloud_proxy() {
                println!(
                    "  {} Cleared broken Google Cloud SDK (gcloud) proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [45/73] Google Cloud SDK (gcloud) proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [45/73] Google Cloud SDK (gcloud) proxy: {}", "Clean or not set".dimmed());
    }

    // 46. Clean dead AWS CLI proxy (~/.aws/config)
    if let Some((val, path)) = get_aws_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [46/73] Found dead AWS CLI proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_aws_proxy() {
                println!(
                    "  {} Cleared broken AWS CLI proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [46/73] AWS CLI proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [46/73] AWS CLI proxy: {}", "Clean or not set".dimmed());
    }

    // 47. Clean dead Microsoft Azure CLI proxy (~/.azure/config)
    if let Some((val, path)) = get_azure_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [47/73] Found dead Azure CLI proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_azure_proxy() {
                println!(
                    "  {} Cleared broken Azure CLI proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [47/73] Azure CLI proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [47/73] Azure CLI proxy: {}", "Clean or not set".dimmed());
    }

    // 48. Clean dead Mamba / Micromamba proxy (~/.mambarc)
    if let Some(mamba_path) = crate::doctor::get_mambarc_path() {
        if mamba_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&mamba_path) {
                if let Some(cfg) = crate::doctor::parse_condarc_proxy(&content) {
                    let mut dead = false;
                    if let Some(ref http) = cfg.http {
                        if !verify_proxy_url(http).await {
                            dead = true;
                        }
                    }
                    if let Some(ref https) = cfg.https {
                        if !verify_proxy_url(https).await {
                            dead = true;
                        }
                    }
                    if dead {
                        println!(
                            "  [48/73] Found dead Mamba proxy in {}, cleaning with backup...",
                            mamba_path.display()
                        );
                        let backup = mamba_path.with_extension("bak");
                        let _ = std::fs::copy(&mamba_path, &backup);
                        if let Ok(true) = unset_mamba_proxy() {
                            println!(
                                "  {} Cleared broken Mamba proxy in {} (backup saved)",
                                "✔".green(),
                                mamba_path.display()
                            );
                            fixed_count += 1;
                        }
                    } else {
                        println!("  [48/73] Mamba proxy: {}", "Reachable".green());
                    }
                } else {
                    println!("  [48/73] Mamba proxy: {}", "Clean or not set".dimmed());
                }
            } else {
                println!("  [48/73] Mamba proxy: {}", "Clean or not set".dimmed());
            }
        } else {
            println!("  [48/73] Mamba proxy: {}", "Clean or not set".dimmed());
        }
    } else {
        println!("  [48/73] Mamba proxy: {}", "Clean or not set".dimmed());
    }

    // 49. Clean dead Podman proxy (~/.config/containers/containers.conf)
    if let Some((podproxy, podpath)) = get_podman_proxy() {
        if !verify_proxy_url(&podproxy).await {
            println!(
                "  [49/73] Found dead Podman proxy (`{}` in {}), cleaning with backup...",
                podproxy.yellow(),
                podpath.display()
            );
            let backup = podpath.with_extension("bak");
            let _ = std::fs::copy(&podpath, &backup);
            if let Ok(true) = unset_podman_proxy() {
                println!(
                    "  {} Cleared broken Podman proxy in {} (backup saved)",
                    "✔".green(),
                    podpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [49/73] Podman proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [49/73] Podman proxy: {}", "Clean or not set".dimmed());
    }

    // 50. Clean dead Vagrant proxy (~/.vagrant.d/Vagrantfile)
    if let Some((vagproxy, vagpath)) = get_vagrant_proxy() {
        if !verify_proxy_url(&vagproxy).await {
            println!(
                "  [50/73] Found dead Vagrant proxy (`{}` in {}), cleaning with backup...",
                vagproxy.yellow(),
                vagpath.display()
            );
            let backup = vagpath.with_extension("bak");
            let _ = std::fs::copy(&vagpath, &backup);
            if let Ok(true) = unset_vagrant_proxy() {
                println!(
                    "  {} Cleared broken Vagrant proxy in {} (backup saved)",
                    "✔".green(),
                    vagpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [50/73] Vagrant proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [50/73] Vagrant proxy: {}", "Clean or not set".dimmed());
    }

    // 51. Clean dead Chocolatey proxy (chocolatey.config)
    if let Some((chocoproxy, chocopath)) = get_choco_proxy() {
        if !verify_proxy_url(&chocoproxy).await {
            println!(
                "  [51/73] Found dead Chocolatey proxy (`{}` in {}), cleaning with backup...",
                chocoproxy.yellow(),
                chocopath.display()
            );
            let backup = chocopath.with_extension("bak");
            let _ = std::fs::copy(&chocopath, &backup);
            if let Ok(true) = unset_choco_proxy() {
                println!(
                    "  {} Cleared broken Chocolatey proxy in {} (backup saved)",
                    "✔".green(),
                    chocopath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [51/73] Chocolatey proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [51/73] Chocolatey proxy: {}", "Clean or not set".dimmed());
    }

    // 52. Clean dead Colima proxy (~/.colima/default/colima.yaml)
    if let Some((colimaproxy, colimapath)) = get_colima_proxy() {
        if !verify_proxy_url(&colimaproxy).await {
            println!(
                "  [52/73] Found dead Colima proxy (`{}` in {}), cleaning with backup...",
                colimaproxy.yellow(),
                colimapath.display()
            );
            let backup = colimapath.with_extension("bak");
            let _ = std::fs::copy(&colimapath, &backup);
            if let Ok(true) = unset_colima_proxy() {
                println!(
                    "  {} Cleared broken Colima proxy in {} (backup saved)",
                    "✔".green(),
                    colimapath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [52/73] Colima proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [52/73] Colima proxy: {}", "Clean or not set".dimmed());
    }

    // 53. Clean dead Minikube proxy (~/.minikube/config/config.json)
    if let Some((miniproxy, minipath)) = get_minikube_proxy() {
        if !verify_proxy_url(&miniproxy).await {
            println!(
                "  [53/73] Found dead Minikube proxy (`{}` in {}), cleaning with backup...",
                miniproxy.yellow(),
                minipath.display()
            );
            let backup = minipath.with_extension("bak");
            let _ = std::fs::copy(&minipath, &backup);
            if let Ok(true) = unset_minikube_proxy() {
                println!(
                    "  {} Cleared broken Minikube proxy in {} (backup saved)",
                    "✔".green(),
                    minipath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [53/73] Minikube proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [53/73] Minikube proxy: {}", "Clean or not set".dimmed());
    }

    // 54. Clean dead Spack proxy (~/.spack/config.yaml)
    if let Some((spackproxy, spackpath)) = get_spack_proxy() {
        if !verify_proxy_url(&spackproxy).await {
            println!(
                "  [54/73] Found dead Spack proxy (`{}` in {}), cleaning with backup...",
                spackproxy.yellow(),
                spackpath.display()
            );
            let backup = spackpath.with_extension("bak");
            let _ = std::fs::copy(&spackpath, &backup);
            if let Ok(true) = unset_spack_proxy() {
                println!(
                    "  {} Cleared broken Spack proxy in {} (backup saved)",
                    "✔".green(),
                    spackpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [54/73] Spack proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [54/73] Spack proxy: {}", "Clean or not set".dimmed());
    }

    // 55. Clean dead MacPorts proxy (~/.macports/macports.conf / /opt/local/etc/macports/macports.conf)
    if let Some((mpproxy, mppath)) = get_macports_proxy() {
        if !verify_proxy_url(&mpproxy).await {
            println!(
                "  [55/73] Found dead MacPorts proxy (`{}` in {}), cleaning with backup...",
                mpproxy.yellow(),
                mppath.display()
            );
            let backup = mppath.with_extension("bak");
            let _ = std::fs::copy(&mppath, &backup);
            if let Ok(true) = unset_macports_proxy() {
                println!(
                    "  {} Cleared broken MacPorts proxy in {} (backup saved)",
                    "✔".green(),
                    mppath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [55/73] MacPorts proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [55/73] MacPorts proxy: {}", "Clean or not set".dimmed());
    }

    // 56. Clean dead APT proxy (/etc/apt/apt.conf.d/* / ~/.apt/apt.conf)
    if let Some((aptproxy, aptpath)) = get_apt_proxy() {
        if !verify_proxy_url(&aptproxy).await {
            println!(
                "  [56/73] Found dead APT proxy (`{}` in {}), cleaning with backup...",
                aptproxy.yellow(),
                aptpath.display()
            );
            let backup = aptpath.with_extension("bak");
            let _ = std::fs::copy(&aptpath, &backup);
            if let Ok(true) = unset_apt_proxy() {
                println!(
                    "  {} Cleared broken APT proxy in {} (backup saved)",
                    "✔".green(),
                    aptpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [56/73] APT proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [56/73] APT proxy: {}", "Clean or not set".dimmed());
    }

    // 57. Clean dead DNF & YUM proxy (/etc/dnf/dnf.conf / ~/.config/dnf/dnf.conf / /etc/yum.conf)
    if let Some((dnfproxy, dnfpath)) = get_dnf_proxy() {
        if !verify_proxy_url(&dnfproxy).await {
            println!(
                "  [57/73] Found dead DNF/YUM proxy (`{}` in {}), cleaning with backup...",
                dnfproxy.yellow(),
                dnfpath.display()
            );
            let backup = dnfpath.with_extension("bak");
            let _ = std::fs::copy(&dnfpath, &backup);
            if let Ok(true) = unset_dnf_proxy() {
                println!(
                    "  {} Cleared broken DNF/YUM proxy in {} (backup saved)",
                    "✔".green(),
                    dnfpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [57/73] DNF/YUM proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [57/73] DNF/YUM proxy: {}", "Clean or not set".dimmed());
    }

    // 58. Clean dead Zypper proxy (/etc/zypp/zypp.conf / ~/.zypp/zypp.conf)
    if let Some((zyppproxy, zypppath)) = get_zypper_proxy() {
        if !verify_proxy_url(&zyppproxy).await {
            println!(
                "  [58/73] Found dead Zypper proxy (`{}` in {}), cleaning with backup...",
                zyppproxy.yellow(),
                zypppath.display()
            );
            let backup = zypppath.with_extension("bak");
            let _ = std::fs::copy(&zypppath, &backup);
            if let Ok(true) = unset_zypper_proxy() {
                println!(
                    "  {} Cleared broken Zypper proxy in {} (backup saved)",
                    "✔".green(),
                    zypppath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [58/73] Zypper proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [58/73] Zypper proxy: {}", "Clean or not set".dimmed());
    }

    // 59. Clean dead Pacman package manager proxy in /etc/pacman.conf
    if let Some((pacproxy, pacpath)) = get_pacman_proxy() {
        if !verify_proxy_url(&pacproxy).await {
            println!(
                "  [59/73] Found dead Pacman proxy (`{}` in {}), cleaning with backup...",
                pacproxy.yellow(),
                pacpath.display()
            );
            let backup = pacpath.with_extension("bak");
            let _ = std::fs::copy(&pacpath, &backup);
            if let Ok(true) = unset_pacman_proxy() {
                println!(
                    "  {} Cleared broken Pacman proxy in {} (backup saved)",
                    "✔".green(),
                    pacpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [59/73] Pacman proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [59/73] Pacman proxy: {}", "Clean or not set".dimmed());
    }

    // 60. Clean dead FreeBSD pkg package manager proxy in /usr/local/etc/pkg.conf
    if let Some((pkgproxy, pkgpath)) = get_pkg_proxy() {
        if !verify_proxy_url(&pkgproxy).await {
            println!(
                "  [60/73] Found dead FreeBSD pkg proxy (`{}` in {}), cleaning with backup...",
                pkgproxy.yellow(),
                pkgpath.display()
            );
            let backup = pkgpath.with_extension("bak");
            let _ = std::fs::copy(&pkgpath, &backup);
            if let Ok(true) = unset_pkg_proxy() {
                println!(
                    "  {} Cleared broken FreeBSD pkg proxy in {} (backup saved)",
                    "✔".green(),
                    pkgpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [60/73] FreeBSD pkg proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [60/73] FreeBSD pkg proxy: {}", "Clean or not set".dimmed());
    }

    // 61. Clean dead Portage package manager proxy in make.conf
    if let Some((portproxy, portpath)) = get_portage_proxy() {
        if !verify_proxy_url(&portproxy).await {
            println!(
                "  [61/73] Found dead Portage proxy (`{}` in {}), cleaning with backup...",
                portproxy.yellow(),
                portpath.display()
            );
            let backup = portpath.with_extension("bak");
            let _ = std::fs::copy(&portpath, &backup);
            if let Ok(true) = unset_portage_proxy() {
                println!(
                    "  {} Cleared broken Portage proxy in {} (backup saved)",
                    "✔".green(),
                    portpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [61/73] Portage proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [61/73] Portage proxy: {}", "Clean or not set".dimmed());
    }

    // 62. Clean dead Docker Daemon engine proxy in daemon.json
    if let Some((daemon_proxy, daemon_path)) = get_docker_daemon_proxy() {
        if !verify_proxy_url(&daemon_proxy).await {
            println!(
                "  [62/73] Found dead Docker Daemon proxy (`{}` in {}), cleaning with backup...",
                daemon_proxy.yellow(),
                daemon_path.display()
            );
            let backup = daemon_path.with_extension("bak");
            let _ = std::fs::copy(&daemon_path, &backup);
            if let Ok(true) = unset_docker_daemon_proxy() {
                println!(
                    "  {} Cleared broken Docker Daemon proxy in {} (backup saved)",
                    "✔".green(),
                    daemon_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [62/73] Docker Daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [62/73] Docker Daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 63. Clean dead Flatpak package manager proxy in repo/config
    if let Some((flatpak_proxy, flatpak_path)) = get_flatpak_proxy() {
        if !verify_proxy_url(&flatpak_proxy).await {
            println!(
                "  [63/73] Found dead Flatpak proxy (`{}` in {}), cleaning with backup...",
                flatpak_proxy.yellow(),
                flatpak_path.display()
            );
            let backup = flatpak_path.with_extension("bak");
            let _ = std::fs::copy(&flatpak_path, &backup);
            if let Ok(true) = unset_flatpak_proxy() {
                println!(
                    "  {} Cleared broken Flatpak proxy in {} (backup saved)",
                    "✔".green(),
                    flatpak_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [63/73] Flatpak proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [63/73] Flatpak proxy: {}", "Clean or not set".dimmed());
    }

    // 64. Clean dead Snap package manager proxy in snapd config / system settings
    if let Some((snap_proxy, snap_path)) = get_snap_proxy() {
        if !verify_proxy_url(&snap_proxy).await {
            println!(
                "  [64/73] Found dead Snap proxy (`{}` in {}), cleaning with backup...",
                snap_proxy.yellow(),
                snap_path.display()
            );
            if snap_path.exists() {
                let backup = snap_path.with_extension("bak");
                let _ = std::fs::copy(&snap_path, &backup);
            }
            if let Ok(true) = unset_snap_proxy() {
                println!(
                    "  {} Cleared broken Snap proxy in {} (backup saved)",
                    "✔".green(),
                    snap_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [64/73] Snap proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [64/73] Snap proxy: {}", "Clean or not set".dimmed());
    }

    // 65. Clean dead Ollama local AI runner daemon proxy in systemd / config
    if let Some((ollama_proxy, ollama_path)) = get_ollama_proxy() {
        if !verify_proxy_url(&ollama_proxy).await {
            println!(
                "  [65/73] Found dead Ollama daemon proxy (`{}` in {}), cleaning with backup...",
                ollama_proxy.yellow(),
                ollama_path.display()
            );
            if ollama_path.exists() {
                let backup = ollama_path.with_extension("bak");
                let _ = std::fs::copy(&ollama_path, &backup);
            }
            if let Ok(true) = unset_ollama_proxy() {
                println!(
                    "  {} Cleared broken Ollama proxy in {} (backup saved)",
                    "✔".green(),
                    ollama_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [65/73] Ollama daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [65/73] Ollama daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 66. Clean dead Containerd (Kubernetes / CRI) container runtime daemon proxy in systemd / config
    if let Some((containerd_proxy, containerd_path)) = get_containerd_proxy() {
        if !verify_proxy_url(&containerd_proxy).await {
            println!(
                "  [66/73] Found dead Containerd daemon proxy (`{}` in {}), cleaning with backup...",
                containerd_proxy.yellow(),
                containerd_path.display()
            );
            if containerd_path.exists() {
                let backup = containerd_path.with_extension("bak");
                let _ = std::fs::copy(&containerd_path, &backup);
            }
            if let Ok(true) = unset_containerd_proxy() {
                println!(
                    "  {} Cleared broken Containerd proxy in {} (backup saved)",
                    "✔".green(),
                    containerd_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [66/73] Containerd daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [66/73] Containerd daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 67. Clean dead BuildKit (Docker Buildx / buildctl) container builder daemon proxy in systemd / config
    if let Some((buildkit_proxy, buildkit_path)) = get_buildkit_proxy() {
        if !verify_proxy_url(&buildkit_proxy).await {
            println!(
                "  [67/73] Found dead BuildKit daemon proxy (`{}` in {}), cleaning with backup...",
                buildkit_proxy.yellow(),
                buildkit_path.display()
            );
            if buildkit_path.exists() {
                let backup = buildkit_path.with_extension("bak");
                let _ = std::fs::copy(&buildkit_path, &backup);
            }
            if let Ok(true) = unset_buildkit_proxy() {
                println!(
                    "  {} Cleared broken BuildKit proxy in {} (backup saved)",
                    "✔".green(),
                    buildkit_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [67/73] BuildKit daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [67/73] BuildKit daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 68. Clean dead K3s (Lightweight Kubernetes / k3s-agent) node daemon proxy in systemd / env
    if let Some((k3s_proxy, k3s_path)) = get_k3s_proxy() {
        if !verify_proxy_url(&k3s_proxy).await {
            println!(
                "  [68/73] Found dead K3s daemon proxy (`{}` in {}), cleaning with backup...",
                k3s_proxy.yellow(),
                k3s_path.display()
            );
            if k3s_path.exists() {
                let backup = k3s_path.with_extension("bak");
                let _ = std::fs::copy(&k3s_path, &backup);
            }
            if let Ok(true) = unset_k3s_proxy() {
                println!(
                    "  {} Cleared broken K3s proxy in {} (backup saved)",
                    "✔".green(),
                    k3s_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [68/73] K3s daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [68/73] K3s daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 69. Clean dead Alpine Linux apk package manager repositories in /etc/apk/repositories
    let apk_paths = get_apk_repositories_paths();
    if !apk_paths.is_empty() {
        for apath in apk_paths {
            if let Ok(content) = std::fs::read_to_string(&apath) {
                let repos = parse_apk_repositories(&content);
                let mut dead_repos = Vec::new();
                for repo in &repos {
                    if !verify_proxy_url(repo).await {
                        dead_repos.push(repo.as_str());
                    }
                }
                if !dead_repos.is_empty() {
                    println!(
                        "  [69/73] Found {} dead Alpine apk repository mirror(s) in {}, cleaning with backup...",
                        dead_repos.len().to_string().yellow(),
                        apath.display()
                    );
                    let (new_content, modified) = clean_apk_repositories(&content, &dead_repos);
                    if modified {
                        let backup = apath.with_extension("bak");
                        let _ = std::fs::copy(&apath, &backup);
                        let _ = std::fs::write(&apath, new_content);
                        println!(
                            "  {} Cleaned dead Alpine apk repository mirror(s) in {} (backup saved)",
                            "✔".green(),
                            apath.display()
                        );
                        fixed_count += 1;
                    }
                } else if !repos.is_empty() {
                    println!("  [69/73] Alpine Linux apk repositories: {}", "Reachable".green());
                } else {
                    println!("  [69/73] Alpine Linux apk repositories: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [69/73] Alpine Linux apk repositories: {}", "Clean or not set".dimmed());
    }

    // 70. Clean dead vcpkg registries/mirrors in vcpkg-configuration.json
    let vcpkg_paths = get_vcpkg_configuration_paths();
    if !vcpkg_paths.is_empty() {
        for vpath in vcpkg_paths {
            if let Ok(content) = std::fs::read_to_string(&vpath) {
                let regs = parse_vcpkg_configuration_registries(&content);
                let mut dead_regs = Vec::new();
                for reg in &regs {
                    if !verify_proxy_url(reg).await {
                        dead_regs.push(reg.as_str());
                    }
                }
                if !dead_regs.is_empty() {
                    println!(
                        "  [70/73] Found {} dead vcpkg registry mirror(s) in {}, cleaning with backup...",
                        dead_regs.len().to_string().yellow(),
                        vpath.display()
                    );
                    let (new_content, modified) =
                        clean_vcpkg_configuration_registries(&content, &dead_regs);
                    if modified {
                        let backup = vpath.with_extension("bak");
                        let _ = std::fs::copy(&vpath, &backup);
                        let _ = std::fs::write(&vpath, new_content);
                        println!(
                            "  {} Cleaned dead vcpkg registry mirror(s) in {} (backup saved)",
                            "✔".green(),
                            vpath.display()
                        );
                        fixed_count += 1;
                    }
                } else if !regs.is_empty() {
                    println!("  [70/73] vcpkg configuration registries: {}", "Reachable".green());
                } else {
                    println!("  [70/73] vcpkg configuration registries: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [70/73] vcpkg configuration registries: {}", "Clean or not set".dimmed());
    }

    // 71. Clean dead Kubernetes Helm chart repositories in repositories.yaml
    let helm_paths = get_helm_repositories_paths();
    if !helm_paths.is_empty() {
        for hpath in helm_paths {
            if let Ok(content) = std::fs::read_to_string(&hpath) {
                let repos = parse_helm_repositories(&content);
                let mut dead_repos = Vec::new();
                for repo in &repos {
                    if !verify_proxy_url(repo).await {
                        dead_repos.push(repo.as_str());
                    }
                }
                if !dead_repos.is_empty() {
                    println!(
                        "  [71/73] Found {} dead Helm chart repository mirror(s) in {}, cleaning with backup...",
                        dead_repos.len().to_string().yellow(),
                        hpath.display()
                    );
                    let (new_content, modified) =
                        clean_helm_repositories(&content, &dead_repos);
                    if modified {
                        let backup = hpath.with_extension("bak");
                        let _ = std::fs::copy(&hpath, &backup);
                        let _ = std::fs::write(&hpath, new_content);
                        println!(
                            "  {} Cleaned dead Helm chart repository mirror(s) in {} (backup saved)",
                            "✔".green(),
                            hpath.display()
                        );
                        fixed_count += 1;
                    }
                } else if !repos.is_empty() {
                    println!("  [71/73] Kubernetes Helm chart repositories: {}", "Reachable".green());
                } else {
                    println!("  [71/73] Kubernetes Helm chart repositories: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [71/73] Kubernetes Helm chart repositories: {}", "Clean or not set".dimmed());
    }

    // 72. Sync active system/daemon proxy to terminal with proper NO_PROXY bypass
    let active_port = match get_system_proxy() {
        Some(sys_info) if sys_info.state.http_enabled && sys_info.state.http_port > 0 => {
            if scanner::check_host_port(&sys_info.state.http_host, sys_info.state.http_port).await {
                Some(sys_info.state.http_port)
            } else {
                None
            }
        }
        _ => None,
    };

    let target_port = match active_port {
        Some(p) => Some(p),
        None => scanner::find_first_active_proxy().await.map(|p| p.port),
    };

    if let Some(port) = target_port {
        println!(
            "  [72/73] Detected healthy active proxy on port {}. Generating Shell sync...",
            port.to_string().green().bold()
        );
        let proxy_url = format!("http://127.0.0.1:{}", port);
        println!(
            "  {} Recommended shell injection (run `eval $(proxydoc on)`):",
            "✔".green()
        );
        println!(
            "\n    {}",
            format!("export http_proxy=\"{}\"", proxy_url).cyan()
        );
        println!(
            "    {}",
            format!("export https_proxy=\"{}\"", proxy_url).cyan()
        );
        println!(
            "    {}",
            format!("export all_proxy=\"{}\"", proxy_url).cyan()
        );
        println!(
            "    {}",
            "export no_proxy=\"localhost,127.0.0.1,::1,localaddress,.local\"".cyan()
        );
        println!();
        fixed_count += 1;
    } else {
        println!(
            "  [72/73] No active local proxy client found. Ensure your proxy software is open."
        );
    }

    // 73. Flush local DNS cache
    println!("  [73/73] Flushing local DNS cache to fix resolution stalls...");
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("dscacheutil").arg("-flushcache").status();
        let _ = Command::new("killall")
            .args(["-HUP", "mDNSResponder"])
            .status();
        println!("  {} macOS DNS cache flushed.", "✔".green());
        fixed_count += 1;
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("ipconfig").arg("/flushdns").status();
        println!("  {} Windows DNS cache flushed.", "✔".green());
        fixed_count += 1;
    }
    #[cfg(target_os = "linux")]
    {
        let flushed = Command::new("resolvectl")
            .args(["flush-caches"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
            || Command::new("systemd-resolve")
                .args(["--flush-caches"])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
        if flushed {
            println!("  {} Linux DNS cache flushed.", "✔".green());
            fixed_count += 1;
        } else {
            println!(
                "  {} Linux DNS cache: No systemd-resolved detected or flush skipped.",
                "○".dimmed()
            );
        }
    }

    println!(
        "\n{}",
        format!("🎉 Auto-repair complete! ({} action(s) taken)", fixed_count)
            .green()
            .bold()
    );
    println!("Run `proxydoc doctor` to verify the new state.\n");

    Ok(())
}

fn get_git_config(key: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["config", "--global", "--get", key])
        .output()
        .ok()?;
    if out.status.success() {
        let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !val.is_empty() {
            return Some(val);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_out_dead_lines() {
        let content = "verbose\nproxy = http://127.0.0.1:9999\nsilent\n";
        let (res, modified) = comment_out_dead_lines(content, |l| l.starts_with("proxy"));
        assert!(modified);
        assert!(res.contains("# [proxydoc disabled dead proxy]: proxy = http://127.0.0.1:9999"));
        assert!(res.contains("verbose\n"));
        assert!(res.contains("silent\n"));

        let clean_content = "verbose\n# already commented\n";
        let (_, clean_mod) = comment_out_dead_lines(clean_content, |l| l.starts_with("proxy"));
        assert!(!clean_mod);
    }
}
