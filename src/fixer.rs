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
    get_bundler_proxy,
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
                "  [1/48] Found dead Git http.proxy (`{}`), cleaning...",
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
                "  [1/48] Found dead Git https.proxy (`{}`), cleaning...",
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
            println!("  [1/48] Found broken Git http.sslCAInfo pointing to missing file (`{}`), cleaning...", val.yellow());
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
        println!("  [1/48] Git global proxy: {}", "Clean or healthy".dimmed());
    }

    // 2. Clean dead NPM proxy
    let npm_http = get_npm_config("proxy");
    let npm_https = get_npm_config("https-proxy");
    let mut npm_cleaned = false;

    if let Some(val) = npm_http {
        if !verify_proxy_url(&val).await {
            println!(
                "  [2/48] Found dead NPM proxy (`{}`), cleaning...",
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
                "  [2/48] Found dead NPM https-proxy (`{}`), cleaning...",
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
        println!("  [2/48] NPM proxy: {}", "Clean or healthy".dimmed());
    }

    // 3. Clean dead Yarn proxy (~/.yarnrc / ~/.yarnrc.yml)
    if let Some(cfg) = get_yarn_proxy() {
        if !verify_proxy_url(&cfg.proxy).await {
            println!(
                "  [3/48] Found dead Yarn proxy (`{}` in {}), cleaning...",
                cfg.proxy.yellow(),
                cfg.source
            );
            if let Ok(true) = unset_yarn_proxy() {
                println!("  {} Cleared broken Yarn global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [3/48] Yarn proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [3/48] Yarn proxy: {}", "Clean or not set".dimmed());
    }

    // 4. Clean dead pnpm proxy (~/.pnpmrc)
    if let Some(val) = get_pnpm_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [4/48] Found dead pnpm proxy (`{}` in ~/.pnpmrc), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_pnpm_proxy() {
                println!("  {} Cleared broken pnpm proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [4/48] pnpm proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [4/48] pnpm proxy: {}", "Clean or not set".dimmed());
    }

    // 5. Clean dead Bun proxy (~/.bunfig.toml)
    if let Some(val) = get_bun_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [5/48] Found dead Bun proxy (`{}` in ~/.bunfig.toml), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_bun_proxy() {
                println!("  {} Cleared broken Bun proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [5/48] Bun proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [5/48] Bun proxy: {}", "Clean or not set".dimmed());
    }

    // 6. Clean dead Pip proxy
    if let Some(val) = get_pip_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [6/48] Found dead Pip global.proxy (`{}`), cleaning...",
                val.yellow()
            );
            let _ = Command::new("pip")
                .args(["config", "unset", "global.proxy"])
                .status();
            println!("  {} Cleared broken Pip global proxy.", "✔".green());
            fixed_count += 1;
        } else {
            println!("  [6/48] Pip global proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [6/48] Pip global proxy: {}", "Clean or not set".dimmed());
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
            println!("  [7/48] Found dead Conda proxy in ~/.condarc, cleaning...");
            if let Ok(true) = unset_conda_proxy() {
                println!(
                    "  {} Cleared broken Conda proxy in ~/.condarc.",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [7/48] Conda proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [7/48] Conda proxy: {}", "Clean or not set".dimmed());
    }

    // 8. Clean dead Docker client proxy (~/.docker/config.json)
    if let Some(val) = get_docker_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [8/48] Found dead Docker proxy (`{}`), cleaning ~/.docker/config.json...",
                val.yellow()
            );
            if let Ok(true) = unset_docker_proxy() {
                println!("  {} Cleared broken Docker client proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [8/48] Docker client proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [8/48] Docker client proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 9. Clean dead Gradle proxy (~/.gradle/gradle.properties)
    if let Some(cfg) = get_gradle_proxy() {
        if !scanner::check_host_port(&cfg.host, cfg.port).await {
            println!(
                "  [9/48] Found dead Gradle proxy (`{}:{}`), removing from ~/.gradle/gradle.properties...",
                cfg.host.yellow(),
                cfg.port
            );
            if let Ok(true) = unset_gradle_proxy() {
                println!("  {} Cleared broken Gradle global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [9/48] Gradle global proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [9/48] Gradle global proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 10. Clean dead Maven proxy (~/.m2/settings.xml)
    if let Some(cfg) = get_maven_proxy() {
        if !scanner::check_host_port(&cfg.host, cfg.port).await {
            println!(
                "  [10/48] Found dead Maven proxy (`{}:{}`), deactivating in ~/.m2/settings.xml...",
                cfg.host.yellow(),
                cfg.port
            );
            if let Ok(true) = unset_maven_proxy() {
                println!("  {} Deactivated broken Maven global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [10/48] Maven global proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [10/48] Maven global proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 11. Clean dead NuGet (.NET) proxy
    if let Some((val, path)) = get_nuget_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [11/48] Found dead NuGet proxy (`{}` in {}), cleaning...",
                val.yellow(),
                path.display()
            );
            if let Ok(true) = unset_nuget_proxy() {
                println!("  {} Cleared broken NuGet global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!(
                "  [11/48] NuGet (.NET) global proxy: {}",
                "Reachable".green()
            );
        }
    } else {
        println!(
            "  [11/48] NuGet (.NET) global proxy: {}",
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
                    "  [12/48] Found dead {} proxy (`{}`), cleaning {}...",
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
            "  [12/48] VS Code & Cursor IDE settings: {}",
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
                "  [13/48] Found dead {} proxy in {}, deactivating...",
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
            "  [13/48] JetBrains & Android Studio settings: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 13. Clean dead GitHub CLI (gh) proxy
    if let Some(val) = get_gh_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [14/48] Found dead GitHub CLI (gh) proxy (`{}`), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_gh_proxy() {
                println!("  {} Cleared broken GitHub CLI (gh) proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [14/48] GitHub CLI (gh) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [14/48] GitHub CLI (gh) proxy: {}",
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
                    println!("  [15/48] Found dead proxy `{}` in ~/.curlrc, commenting out with backup...", val.yellow());
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
                    println!("  [15/48] Found dead proxy `{}` in ~/.wgetrc, commenting out with backup...", val.yellow());
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
                "  [15/48] Found dead Cargo proxy (`{}`), cleaning ~/.cargo/config.toml...",
                val.yellow()
            );
            if let Ok(true) = crate::manager::unset_cargo_proxy() {
                println!("  {} Cleared broken Cargo global proxy.", "✔".green());
                static_fixed = true;
                fixed_count += 1;
            } else {
                println!(
                    "  [15/48] {} Cargo config has dead proxy `{}`! Please remove `proxy` in ~/.cargo/config.toml",
                    "▲".yellow(),
                    val.yellow()
                );
                static_fixed = true;
            }
        }
    }

    if !static_fixed {
        println!(
            "  [15/48] cURL, Wget, and Cargo configs: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 16. Clean dead Gem (~/.gemrc) proxy
    if let Some(val) = get_gem_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [16/48] Found dead Gem proxy (`{}` in ~/.gemrc), cleaning with backup...",
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
            println!("  [16/48] Gem (Ruby) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [16/48] Gem (Ruby) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 17. Clean dead Subversion (~/.subversion/servers) proxy
    if let Some(val) = get_svn_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [17/48] Found dead Subversion proxy (`{}` in servers), cleaning with backup...",
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
            println!("  [17/48] Subversion (SVN) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [17/48] Subversion (SVN) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 18. Clean dead Mercurial (~/.hgrc / Mercurial.ini) proxy
    if let Some(val) = get_hg_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [18/48] Found dead Mercurial proxy (`{}` in ~/.hgrc), cleaning with backup...",
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
            println!("  [18/48] Mercurial (Hg) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [18/48] Mercurial (Hg) proxy: {}",
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
            "  [19/48] Node / JS Registry mirrors: {}",
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
                    "  [20/48] {} ~/.ssh/config Host `{}` has dead ProxyCommand target `{}`! (Git clone over SSH will hang)",
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
            "  [20/48] SSH ProxyCommand configs: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 21. Clean dead Astral uv (~/.config/uv/uv.toml) proxy
    if let Some(val) = get_uv_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [21/48] Found dead Astral uv proxy (`{}` in uv.toml), cleaning with backup...",
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
            println!("  [21/48] Astral uv proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [21/48] Astral uv proxy: {}", "Clean or not set".dimmed());
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
                "  [22/48] Found dead GOPROXY mirror (`{}`), cleaning / resetting to default...",
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
            println!("  [22/48] Go GOPROXY: {}", "Reachable or direct".green());
        }
    } else {
        println!("  [22/48] Go GOPROXY: {}", "Clean or default".dimmed());
    }

    // 23. Clean dead Bazel (~/.bazelrc) action_env proxy
    if let Some((val, path)) = get_bazel_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [23/48] Found dead Bazel action_env proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [23/48] Bazel proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [23/48] Bazel proxy: {}", "Clean or not set".dimmed());
    }

    // 24. Clean dead PHP Composer (config.json) proxy
    if let Some((val, path)) = get_composer_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [24/48] Found dead PHP Composer proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [24/48] PHP Composer proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [24/48] PHP Composer proxy: {}", "Clean or not set".dimmed());
    }

    // 25. Clean dead Conan C/C++ (global.conf / conan.conf) proxy
    if let Some((val, path)) = get_conan_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [25/48] Found dead Conan C/C++ proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [25/48] Conan C/C++ proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [25/48] Conan C/C++ proxy: {}", "Clean or not set".dimmed());
    }

    // 26. Clean dead Scala Build Tool (~/.sbt/sbtopts) proxy
    if let Some((val, path)) = get_sbt_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [26/48] Found dead Scala (sbt) proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [26/48] Scala (sbt) proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [26/48] Scala (sbt) proxy: {}", "Clean or not set".dimmed());
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
                    "  [27/48] Found dead cluster proxy-url in {}, cleaning with backup...",
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
                println!("  [27/48] Kubernetes (kubectl) proxy-url: {}", "Reachable".green());
            } else {
                println!("  [27/48] Kubernetes (kubectl) proxy-url: {}", "Clean or not set".dimmed());
            }
        }
    } else {
        println!("  [27/48] Kubernetes (kubectl) proxy-url: {}", "Clean or not set".dimmed());
    }

    // 28. Clean dead Julia startup (~/.julia/config/startup.jl) proxy
    if let Some((val, path)) = get_julia_startup_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [28/48] Found dead Julia startup proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [28/48] Julia startup proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [28/48] Julia startup proxy: {}", "Clean or not set".dimmed());
    }

    // 29. Clean dead GitLab CLI (~/.config/glab-cli/config.yml) proxy
    if let Some(val) = get_glab_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [29/48] Found dead GitLab CLI (glab) proxy (`{}`), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_glab_proxy() {
                println!("  {} Cleared broken GitLab CLI (glab) proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [29/48] GitLab CLI (glab) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [29/48] GitLab CLI (glab) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 30. Clean dead Nix (~/.config/nix/nix.conf) http-proxy
    if let Some((val, path)) = get_nix_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [30/48] Found dead Nix http-proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [30/48] Nix (nix.conf) http-proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [30/48] Nix (nix.conf) http-proxy: {}", "Clean or not set".dimmed());
    }

    // 31. Clean dead Haskell Cabal (~/.cabal/config) proxy
    if let Some((val, path)) = get_cabal_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [31/48] Found dead Haskell Cabal proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [31/48] Haskell Cabal proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [31/48] Haskell Cabal proxy: {}", "Clean or not set".dimmed());
    }

    // 32. Clean dead Haskell Stack (~/.stack/config.yaml) proxy
    if let Some((val, path)) = get_stack_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [32/48] Found dead Haskell Stack proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [32/48] Haskell Stack proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [32/48] Haskell Stack proxy: {}", "Clean or not set".dimmed());
    }

    // 33. Clean dead Elixir / Erlang / Gleam Hex (~/.hex/hex.config) proxy
    if let Some((val, path)) = get_hex_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [33/48] Found dead Hex package proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [33/48] Hex (Elixir/Gleam) proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [33/48] Hex (Elixir/Gleam) proxy: {}", "Clean or not set".dimmed());
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
                    "  [34/48] Found {} dead SwiftPM mirror(s) in {}, cleaning with backup...",
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
                println!("  [34/48] SwiftPM mirrors: {}", "Reachable".green());
            } else {
                println!("  [34/48] SwiftPM mirrors: {}", "Clean or empty".dimmed());
            }
        }
    } else {
        println!("  [34/48] SwiftPM mirrors: {}", "Clean or not set".dimmed());
    }

    // 35. Clean dead R package manager (~/.Renviron) proxy
    if let Some((val, path)) = get_renviron_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [35/48] Found dead R proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [35/48] R (~/.Renviron) proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [35/48] R (~/.Renviron) proxy: {}", "Clean or not set".dimmed());
    }

    // 36. Clean dead OCaml OPAM (~/.opam/config) proxy
    if let Some((val, path)) = get_opam_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [36/48] Found dead OCaml OPAM proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [36/48] OCaml OPAM proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [36/48] OCaml OPAM proxy: {}", "Clean or not set".dimmed());
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
                        "  [37/48] Found {} dead Terraform/OpenTofu mirror(s) in {}, cleaning with backup...",
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
                    println!("  [37/48] Terraform/OpenTofu network mirrors: {}", "Reachable".green());
                } else {
                    println!("  [37/48] Terraform/OpenTofu network mirrors: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [37/48] Terraform/OpenTofu network mirrors: {}", "Clean or not set".dimmed());
    }

    // 38. Clean dead Clojure Leiningen (~/.lein/profiles.clj) proxy
    if let Some((val, path)) = get_lein_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [38/48] Found dead Clojure Leiningen proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [38/48] Clojure Leiningen proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [38/48] Clojure Leiningen proxy: {}", "Clean or not set".dimmed());
    }

    // 39. Clean dead Perl CPAN (~/.cpan/CPAN/MyConfig.pm) proxy
    if let Some((val, path)) = get_cpan_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [39/48] Found dead Perl CPAN proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [39/48] Perl CPAN proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [39/48] Perl CPAN proxy: {}", "Clean or not set".dimmed());
    }

    // 40. Clean dead LuaRocks (~/.luarocks/config.lua) proxy
    if let Some((val, path)) = get_luarocks_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [40/48] Found dead LuaRocks proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [40/48] LuaRocks proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [40/48] LuaRocks proxy: {}", "Clean or not set".dimmed());
    }

    // 41. Clean dead Nim (Nimble, ~/.config/nimble/nimble.ini) proxy
    if let Some((val, path)) = get_nimble_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [41/48] Found dead Nimble proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [41/48] Nimble proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [41/48] Nimble proxy: {}", "Clean or not set".dimmed());
    }

    // 42. Clean dead Erlang (Rebar3, ~/.config/rebar3/rebar.config) proxy
    if let Some((val, path)) = get_rebar3_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [42/48] Found dead Rebar3 proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [42/48] Rebar3 proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [42/48] Rebar3 proxy: {}", "Clean or not set".dimmed());
    }

    // 43. Clean dead Scoop (~/.config/scoop/config.json) proxy
    if let Some((val, path)) = get_scoop_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [43/48] Found dead Scoop proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [43/48] Scoop proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [43/48] Scoop proxy: {}", "Clean or not set".dimmed());
    }

    // 44. Clean dead Ruby Bundler (~/.bundle/config) proxy
    if let Some((val, path)) = get_bundler_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [44/48] Found dead Bundler proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [44/48] Bundler proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [44/48] Bundler proxy: {}", "Clean or not set".dimmed());
    }

    // 45. Clean dead vcpkg registries/mirrors in vcpkg-configuration.json
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
                        "  [45/48] Found {} dead vcpkg registry mirror(s) in {}, cleaning with backup...",
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
                    println!("  [45/48] vcpkg configuration registries: {}", "Reachable".green());
                } else {
                    println!("  [45/48] vcpkg configuration registries: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [45/48] vcpkg configuration registries: {}", "Clean or not set".dimmed());
    }

    // 46. Clean dead Kubernetes Helm chart repositories in repositories.yaml
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
                        "  [46/48] Found {} dead Helm chart repository mirror(s) in {}, cleaning with backup...",
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
                    println!("  [46/48] Kubernetes Helm chart repositories: {}", "Reachable".green());
                } else {
                    println!("  [46/48] Kubernetes Helm chart repositories: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [46/48] Kubernetes Helm chart repositories: {}", "Clean or not set".dimmed());
    }

    // 47. Sync active system/daemon proxy to terminal with proper NO_PROXY bypass
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
            "  [47/48] Detected healthy active proxy on port {}. Generating Shell sync...",
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
            "  [47/48] No active local proxy client found. Ensure your proxy software is open."
        );
    }

    // 48. Flush local DNS cache
    println!("  [48/48] Flushing local DNS cache to fix resolution stalls...");
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
