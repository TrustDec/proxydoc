use anyhow::Result;
use colored::*;
use std::process::Command;

use crate::doctor::{get_macos_system_proxy, verify_proxy_url};
use crate::scanner;

pub async fn run() -> Result<()> {
    println!("\n{}", "🩹 ==================== PROXY AUTO-REPAIR ==================== 🩹".bold().cyan());
    println!("{}", "   Detecting anomalies and automatically applying fixes...\n".dimmed());

    let mut fixed_count = 0;

    // 1. Clean dead Git proxy
    let git_http = get_git_config("http.proxy");
    if let Some(val) = git_http {
        if !verify_proxy_url(&val).await {
            println!("  [1/3] Found dead Git http.proxy (`{}`), cleaning...", val.yellow());
            let _ = Command::new("git").args(["config", "--global", "--unset", "http.proxy"]).status();
            let _ = Command::new("git").args(["config", "--global", "--unset", "https.proxy"]).status();
            println!("  {} Cleared broken Git global proxy.", "✔".green());
            fixed_count += 1;
        }
    }

    // 2. Sync active system/daemon proxy to terminal
    let active_port = match get_macos_system_proxy() {
        Some(sys) if sys.http_enabled && sys.http_port > 0 => {
            if scanner::check_port(sys.http_port).await {
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
            "  [2/3] Detected healthy active proxy on port {}. Generating Shell injection...",
            port.to_string().green().bold()
        );
        let proxy_url = format!("http://127.0.0.1:{}", port);
        println!("  {} Recommended shell injection:", "✔".green());
        println!("\n    {}", format!("export http_proxy=\"{}\"", proxy_url).cyan());
        println!("    {}", format!("export https_proxy=\"{}\"", proxy_url).cyan());
        println!("    {}", format!("export all_proxy=\"{}\"", proxy_url).cyan());
        println!();
        fixed_count += 1;
    } else {
        println!("  [2/3] No active local proxy client found. Ensure your proxy software is open.");
    }

    // 3. Flush macOS DNS cache
    println!("  [3/3] Flushing local DNS cache to fix resolution stalls...");
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("dscacheutil").arg("-flushcache").status();
        println!("  {} Local DNS cache flushed.", "✔".green());
        fixed_count += 1;
    }

    println!("\n{}", format!("🎉 Auto-repair complete! ({} issues resolved)", fixed_count).green().bold());
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
