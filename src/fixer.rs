use anyhow::Result;
use colored::*;
use std::process::Command;

use crate::doctor::{get_cargo_proxy, get_curlrc_proxy, get_macos_system_proxy, get_npm_config, get_pip_proxy, verify_proxy_url};
use crate::scanner;

pub async fn run() -> Result<()> {
    println!("\n{}", "🩹 ==================== PROXY AUTO-REPAIR ==================== 🩹".bold().cyan());
    println!("{}", "   Detecting anomalies and automatically applying fixes...\n".dimmed());

    let mut fixed_count = 0;

    // 1. Clean dead Git proxy
    let git_http = get_git_config("http.proxy");
    let git_https = get_git_config("https.proxy");
    let mut git_cleaned = false;

    if let Some(val) = git_http {
        if !verify_proxy_url(&val).await {
            println!("  [1/6] Found dead Git http.proxy (`{}`), cleaning...", val.yellow());
            let _ = Command::new("git").args(["config", "--global", "--unset", "http.proxy"]).status();
            git_cleaned = true;
            fixed_count += 1;
        }
    }
    if let Some(val) = git_https {
        if !verify_proxy_url(&val).await {
            println!("  [1/6] Found dead Git https.proxy (`{}`), cleaning...", val.yellow());
            let _ = Command::new("git").args(["config", "--global", "--unset", "https.proxy"]).status();
            git_cleaned = true;
            fixed_count += 1;
        }
    }

    if git_cleaned {
        println!("  {} Cleared broken Git global proxy.", "✔".green());
    } else {
        println!("  [1/6] Git global proxy: {}", "Clean or healthy".dimmed());
    }

    // 2. Clean dead NPM proxy
    let npm_http = get_npm_config("proxy");
    let npm_https = get_npm_config("https-proxy");
    let mut npm_cleaned = false;

    if let Some(val) = npm_http {
        if !verify_proxy_url(&val).await {
            println!("  [2/6] Found dead NPM proxy (`{}`), cleaning...", val.yellow());
            let _ = Command::new("npm").args(["config", "delete", "proxy"]).status();
            npm_cleaned = true;
            fixed_count += 1;
        }
    }
    if let Some(val) = npm_https {
        if !verify_proxy_url(&val).await {
            println!("  [2/6] Found dead NPM https-proxy (`{}`), cleaning...", val.yellow());
            let _ = Command::new("npm").args(["config", "delete", "https-proxy"]).status();
            npm_cleaned = true;
            fixed_count += 1;
        }
    }

    if npm_cleaned {
        println!("  {} Cleared broken NPM proxy.", "✔".green());
    } else {
        println!("  [2/6] NPM proxy: {}", "Clean or healthy".dimmed());
    }

    // 3. Clean dead Pip proxy
    if let Some(val) = get_pip_proxy() {
        if !verify_proxy_url(&val).await {
            println!("  [3/6] Found dead Pip global.proxy (`{}`), cleaning...", val.yellow());
            let _ = Command::new("pip").args(["config", "unset", "global.proxy"]).status();
            println!("  {} Cleared broken Pip global proxy.", "✔".green());
            fixed_count += 1;
        } else {
            println!("  [3/6] Pip global proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [3/6] Pip global proxy: {}", "Clean or not set".dimmed());
    }

    // 4. Inspect Cargo and cURL config
    let mut config_issue = false;
    if let Some(val) = get_cargo_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [4/6] {} Cargo config has dead proxy `{}`! Please remove `proxy` in ~/.cargo/config.toml",
                "▲".yellow(),
                val.yellow()
            );
            config_issue = true;
        }
    }
    if let Some(val) = get_curlrc_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [4/6] {} ~/.curlrc has dead proxy `{}`! Please edit ~/.curlrc",
                "▲".yellow(),
                val.yellow()
            );
            config_issue = true;
        }
    }
    if !config_issue {
        println!("  [4/6] Cargo & cURL static configs: {}", "Clean or healthy".dimmed());
    }

    // 5. Sync active system/daemon proxy to terminal with proper NO_PROXY bypass
    let active_port = match get_macos_system_proxy() {
        Some(sys) if sys.http_enabled && sys.http_port > 0 => {
            if scanner::check_host_port(&sys.http_host, sys.http_port).await {
                Some(sys.http_port)
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
            "  [5/6] Detected healthy active proxy on port {}. Generating Shell sync...",
            port.to_string().green().bold()
        );
        let proxy_url = format!("http://127.0.0.1:{}", port);
        println!("  {} Recommended shell injection (run `eval $(proxydoc on)`):", "✔".green());
        println!("\n    {}", format!("export http_proxy=\"{}\"", proxy_url).cyan());
        println!("    {}", format!("export https_proxy=\"{}\"", proxy_url).cyan());
        println!("    {}", format!("export all_proxy=\"{}\"", proxy_url).cyan());
        println!("    {}", "export no_proxy=\"localhost,127.0.0.1,::1,localaddress,.local\"".cyan());
        println!();
        fixed_count += 1;
    } else {
        println!("  [5/6] No active local proxy client found. Ensure your proxy software is open.");
    }

    // 6. Flush macOS DNS cache
    println!("  [6/6] Flushing local DNS cache to fix resolution stalls...");
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("dscacheutil").arg("-flushcache").status();
        let _ = Command::new("killall").args(["-HUP", "mDNSResponder"]).status();
        println!("  {} Local DNS cache flushed.", "✔".green());
        fixed_count += 1;
    }

    println!("\n{}", format!("🎉 Auto-repair complete! ({} action(s) taken)", fixed_count).green().bold());
    println!("Run `proxydoc doctor` to verify the new state.\n");

    Ok(())
}

fn get_git_config(key: &str) -> Option<String> {
    let out = Command::new("git").args(["config", "--global", "--get", key]).output().ok()?;
    if out.status.success() {
        let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !val.is_empty() {
            return Some(val);
        }
    }
    None
}
