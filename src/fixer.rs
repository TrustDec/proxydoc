use anyhow::Result;
use colored::*;
use std::process::Command;

use crate::doctor::{
    clean_jsonc, get_bun_proxy, get_cargo_proxy, get_conda_proxy, get_curlrc_proxy,
    get_docker_proxy, get_gem_proxy, get_gemrc_path, get_gh_proxy, get_goproxy, get_gradle_proxy,
    get_hg_proxy, get_hgrc_path, get_ide_proxies, get_jetbrains_proxies, get_maven_proxy,
    get_npm_config, get_npm_registries, get_nuget_proxy, get_pip_proxy, get_pnpm_proxy,
    get_ssh_config_proxies, get_svn_proxy, get_svn_servers_path, get_system_proxy,
    get_uv_config_path, get_uv_proxy, get_wgetrc_proxy, get_yarn_proxy, verify_proxy_url,
};
use crate::manager::{
    deactivate_jetbrains_proxy_xml, remove_vscode_proxy_json, unset_bun_proxy, unset_conda_proxy,
    unset_docker_proxy, unset_gem_proxy, unset_gh_proxy, unset_go_proxy, unset_gradle_proxy,
    unset_hg_proxy, unset_maven_proxy, unset_nuget_proxy, unset_pnpm_proxy, unset_svn_proxy,
    unset_uv_proxy, unset_yarn_proxy,
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
                "  [1/24] Found dead Git http.proxy (`{}`), cleaning...",
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
                "  [1/24] Found dead Git https.proxy (`{}`), cleaning...",
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
            println!("  [1/24] Found broken Git http.sslCAInfo pointing to missing file (`{}`), cleaning...", val.yellow());
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
        println!("  [1/24] Git global proxy: {}", "Clean or healthy".dimmed());
    }

    // 2. Clean dead NPM proxy
    let npm_http = get_npm_config("proxy");
    let npm_https = get_npm_config("https-proxy");
    let mut npm_cleaned = false;

    if let Some(val) = npm_http {
        if !verify_proxy_url(&val).await {
            println!(
                "  [2/24] Found dead NPM proxy (`{}`), cleaning...",
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
                "  [2/24] Found dead NPM https-proxy (`{}`), cleaning...",
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
        println!("  [2/24] NPM proxy: {}", "Clean or healthy".dimmed());
    }

    // 3. Clean dead Yarn proxy (~/.yarnrc / ~/.yarnrc.yml)
    if let Some(cfg) = get_yarn_proxy() {
        if !verify_proxy_url(&cfg.proxy).await {
            println!(
                "  [3/24] Found dead Yarn proxy (`{}` in {}), cleaning...",
                cfg.proxy.yellow(),
                cfg.source
            );
            if let Ok(true) = unset_yarn_proxy() {
                println!("  {} Cleared broken Yarn global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [3/24] Yarn proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [3/24] Yarn proxy: {}", "Clean or not set".dimmed());
    }

    // 4. Clean dead pnpm proxy (~/.pnpmrc)
    if let Some(val) = get_pnpm_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [4/24] Found dead pnpm proxy (`{}` in ~/.pnpmrc), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_pnpm_proxy() {
                println!("  {} Cleared broken pnpm proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [4/24] pnpm proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [4/24] pnpm proxy: {}", "Clean or not set".dimmed());
    }

    // 5. Clean dead Bun proxy (~/.bunfig.toml)
    if let Some(val) = get_bun_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [5/24] Found dead Bun proxy (`{}` in ~/.bunfig.toml), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_bun_proxy() {
                println!("  {} Cleared broken Bun proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [5/24] Bun proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [5/24] Bun proxy: {}", "Clean or not set".dimmed());
    }

    // 6. Clean dead Pip proxy
    if let Some(val) = get_pip_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [6/24] Found dead Pip global.proxy (`{}`), cleaning...",
                val.yellow()
            );
            let _ = Command::new("pip")
                .args(["config", "unset", "global.proxy"])
                .status();
            println!("  {} Cleared broken Pip global proxy.", "✔".green());
            fixed_count += 1;
        } else {
            println!("  [6/24] Pip global proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [6/24] Pip global proxy: {}", "Clean or not set".dimmed());
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
            println!("  [7/24] Found dead Conda proxy in ~/.condarc, cleaning...");
            if let Ok(true) = unset_conda_proxy() {
                println!(
                    "  {} Cleared broken Conda proxy in ~/.condarc.",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [7/24] Conda proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [7/24] Conda proxy: {}", "Clean or not set".dimmed());
    }

    // 8. Clean dead Docker client proxy (~/.docker/config.json)
    if let Some(val) = get_docker_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [8/24] Found dead Docker proxy (`{}`), cleaning ~/.docker/config.json...",
                val.yellow()
            );
            if let Ok(true) = unset_docker_proxy() {
                println!("  {} Cleared broken Docker client proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [8/24] Docker client proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [8/24] Docker client proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 9. Clean dead Gradle proxy (~/.gradle/gradle.properties)
    if let Some(cfg) = get_gradle_proxy() {
        if !scanner::check_host_port(&cfg.host, cfg.port).await {
            println!(
                "  [9/24] Found dead Gradle proxy (`{}:{}`), removing from ~/.gradle/gradle.properties...",
                cfg.host.yellow(),
                cfg.port
            );
            if let Ok(true) = unset_gradle_proxy() {
                println!("  {} Cleared broken Gradle global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [9/24] Gradle global proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [9/24] Gradle global proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 10. Clean dead Maven proxy (~/.m2/settings.xml)
    if let Some(cfg) = get_maven_proxy() {
        if !scanner::check_host_port(&cfg.host, cfg.port).await {
            println!(
                "  [10/24] Found dead Maven proxy (`{}:{}`), deactivating in ~/.m2/settings.xml...",
                cfg.host.yellow(),
                cfg.port
            );
            if let Ok(true) = unset_maven_proxy() {
                println!("  {} Deactivated broken Maven global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [10/24] Maven global proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [10/24] Maven global proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 11. Clean dead NuGet (.NET) proxy
    if let Some((val, path)) = get_nuget_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [11/24] Found dead NuGet proxy (`{}` in {}), cleaning...",
                val.yellow(),
                path.display()
            );
            if let Ok(true) = unset_nuget_proxy() {
                println!("  {} Cleared broken NuGet global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!(
                "  [11/24] NuGet (.NET) global proxy: {}",
                "Reachable".green()
            );
        }
    } else {
        println!(
            "  [11/24] NuGet (.NET) global proxy: {}",
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
                    "  [12/24] Found dead {} proxy (`{}`), cleaning {}...",
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
            "  [12/24] VS Code & Cursor IDE settings: {}",
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
                "  [13/24] Found dead {} proxy in {}, deactivating...",
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
            "  [13/24] JetBrains & Android Studio settings: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 13. Clean dead GitHub CLI (gh) proxy
    if let Some(val) = get_gh_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [14/24] Found dead GitHub CLI (gh) proxy (`{}`), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_gh_proxy() {
                println!("  {} Cleared broken GitHub CLI (gh) proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [14/24] GitHub CLI (gh) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [14/24] GitHub CLI (gh) proxy: {}",
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
                    println!("  [15/24] Found dead proxy `{}` in ~/.curlrc, commenting out with backup...", val.yellow());
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
                    println!("  [15/24] Found dead proxy `{}` in ~/.wgetrc, commenting out with backup...", val.yellow());
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
                "  [15/24] Found dead Cargo proxy (`{}`), cleaning ~/.cargo/config.toml...",
                val.yellow()
            );
            if let Ok(true) = crate::manager::unset_cargo_proxy() {
                println!("  {} Cleared broken Cargo global proxy.", "✔".green());
                static_fixed = true;
                fixed_count += 1;
            } else {
                println!(
                    "  [15/24] {} Cargo config has dead proxy `{}`! Please remove `proxy` in ~/.cargo/config.toml",
                    "▲".yellow(),
                    val.yellow()
                );
                static_fixed = true;
            }
        }
    }

    if !static_fixed {
        println!(
            "  [15/24] cURL, Wget, and Cargo configs: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 16. Clean dead Gem (~/.gemrc) proxy
    if let Some(val) = get_gem_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [16/24] Found dead Gem proxy (`{}` in ~/.gemrc), cleaning with backup...",
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
            println!("  [16/24] Gem (Ruby) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [16/24] Gem (Ruby) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 17. Clean dead Subversion (~/.subversion/servers) proxy
    if let Some(val) = get_svn_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [17/24] Found dead Subversion proxy (`{}` in servers), cleaning with backup...",
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
            println!("  [17/24] Subversion (SVN) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [17/24] Subversion (SVN) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 18. Clean dead Mercurial (~/.hgrc / Mercurial.ini) proxy
    if let Some(val) = get_hg_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [18/24] Found dead Mercurial proxy (`{}` in ~/.hgrc), cleaning with backup...",
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
            println!("  [18/24] Mercurial (Hg) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [18/24] Mercurial (Hg) proxy: {}",
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
            "  [19/24] Node / JS Registry mirrors: {}",
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
                    "  [20/24] {} ~/.ssh/config Host `{}` has dead ProxyCommand target `{}`! (Git clone over SSH will hang)",
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
            "  [20/24] SSH ProxyCommand configs: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 21. Clean dead Astral uv (~/.config/uv/uv.toml) proxy
    if let Some(val) = get_uv_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [21/24] Found dead Astral uv proxy (`{}` in uv.toml), cleaning with backup...",
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
            println!("  [21/24] Astral uv proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [21/24] Astral uv proxy: {}", "Clean or not set".dimmed());
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
                "  [22/24] Found dead GOPROXY mirror (`{}`), cleaning / resetting to default...",
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
            println!("  [22/24] Go GOPROXY: {}", "Reachable or direct".green());
        }
    } else {
        println!("  [22/24] Go GOPROXY: {}", "Clean or default".dimmed());
    }

    // 23. Sync active system/daemon proxy to terminal with proper NO_PROXY bypass
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
            "  [23/24] Detected healthy active proxy on port {}. Generating Shell sync...",
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
            "  [23/24] No active local proxy client found. Ensure your proxy software is open."
        );
    }

    // 24. Flush local DNS cache
    println!("  [24/24] Flushing local DNS cache to fix resolution stalls...");
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
