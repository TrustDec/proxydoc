use anyhow::Result;
use colored::*;
use std::process::Command;
use tabled::{Table, Tabled};

use crate::scanner;

#[derive(Tabled)]
struct DiagItem {
    #[tabled(rename = "Category")]
    category: &'static str,
    #[tabled(rename = "Target / Config")]
    target: String,
    #[tabled(rename = "Configured Value")]
    value: String,
    #[tabled(rename = "Health Status")]
    status: String,
}

pub async fn run(_verbose: bool) -> Result<()> {
    println!("\n{}", "🩺 ==================== PROXY DOCTOR ==================== 🩺".bold().cyan());
    println!("{}", "   Running full development environment diagnostics...\n".dimmed());

    let mut items = Vec::new();

    // 1. Check Shell Env Vars
    let env_vars = [
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "no_proxy",
        "NO_PROXY",
    ];

    let mut shell_proxy_set = false;
    for var in env_vars {
        if let Ok(val) = std::env::var(var) {
            shell_proxy_set = true;
            let healthy = verify_proxy_url(&val).await;
            items.push(DiagItem {
                category: "Shell Env",
                target: var.to_string(),
                value: val,
                status: if healthy {
                    "● Configured & Reachable".green().to_string()
                } else {
                    "▲ Server Unreachable!".red().bold().to_string()
                },
            });
        }
    }

    if !shell_proxy_set {
        items.push(DiagItem {
            category: "Shell Env",
            target: "Terminal Proxy".to_string(),
            value: "(not set)".dimmed().to_string(),
            status: "○ Direct (Normal)".dimmed().to_string(),
        });
    }

    // 2. Check Git Config
    let git_http = get_git_config("http.proxy");
    let git_https = get_git_config("https.proxy");

    if git_http.is_none() && git_https.is_none() {
        items.push(DiagItem {
            category: "Git",
            target: "git config --global *.proxy".to_string(),
            value: "(not set)".dimmed().to_string(),
            status: "○ Inherits Shell or Direct".dimmed().to_string(),
        });
    } else {
        if let Some(val) = git_http {
            let ok = verify_proxy_url(&val).await;
            items.push(DiagItem {
                category: "Git",
                target: "http.proxy".to_string(),
                value: val,
                status: if ok {
                    "● Reachable".green().to_string()
                } else {
                    "▲ Dead Proxy Port!".red().bold().to_string()
                },
            });
        }
        if let Some(val) = git_https {
            let ok = verify_proxy_url(&val).await;
            items.push(DiagItem {
                category: "Git",
                target: "https.proxy".to_string(),
                value: val,
                status: if ok {
                    "● Reachable".green().to_string()
                } else {
                    "▲ Dead Proxy Port!".red().bold().to_string()
                },
            });
        }
    }

    // 3. Check NPM / Yarn
    let npm_proxy = get_npm_config("proxy");
    let npm_https = get_npm_config("https-proxy");
    if npm_proxy.is_none() && npm_https.is_none() {
        items.push(DiagItem {
            category: "NPM",
            target: "npm config proxy".to_string(),
            value: "(not set)".dimmed().to_string(),
            status: "○ Direct".dimmed().to_string(),
        });
    } else {
        if let Some(val) = npm_proxy {
            let ok = verify_proxy_url(&val).await;
            items.push(DiagItem {
                category: "NPM",
                target: "proxy".to_string(),
                value: val,
                status: if ok { "● Reachable".green().to_string() } else { "▲ Dead Port!".red().to_string() },
            });
        }
    }

    // 4. Check Homebrew Env
    let brew_proxy = std::env::var("HOMEBREW_BOTTLE_DOMAIN")
        .ok()
        .or_else(|| std::env::var("HOMEBREW_API_DOMAIN").ok());
    if let Some(val) = brew_proxy {
        items.push(DiagItem {
            category: "Homebrew",
            target: "Mirror Domain".to_string(),
            value: val,
            status: "● Mirror Active".green().to_string(),
        });
    } else {
        items.push(DiagItem {
            category: "Homebrew",
            target: "Mirror Domain".to_string(),
            value: "Official (default)".dimmed().to_string(),
            status: "○ Uses Shell Proxy".dimmed().to_string(),
        });
    }

    // 5. Check Local Client Ports
    let active_client = scanner::find_first_active_proxy().await;
    match active_client {
        Some(p) => {
            items.push(DiagItem {
                category: "Local Daemon",
                target: format!("Port {}", p.port),
                value: p.client_hint.to_string(),
                status: "● OPEN & LISTENING".green().bold().to_string(),
            });
        }
        None => {
            items.push(DiagItem {
                category: "Local Daemon",
                target: "Common Ports".to_string(),
                value: "7890, 7897, 1080, 1082...".dimmed().to_string(),
                status: "▲ No proxy software detected".yellow().to_string(),
            });
        }
    }

    let table = Table::new(items).to_string();
    println!("{}\n", table);

    // Tips and Actions
    println!("{}", "💡 Recommendations & Quick Actions:".bold());
    println!("  • To enable proxy in current terminal:  {}", "proxydoc on".cyan().bold());
    println!("  • To test latency to AI & GitHub:       {}", "proxydoc test".cyan().bold());
    println!("  • To scan all listening proxy ports:    {}", "proxydoc scan".cyan().bold());
    println!("  • To clean all proxy settings:          {}", "proxydoc off".cyan().bold());
    println!();

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

fn get_npm_config(key: &str) -> Option<String> {
    let out = Command::new("npm").args(["config", "get", key]).output().ok()?;
    if out.status.success() {
        let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !val.is_empty() && val != "null" && val != "undefined" {
            return Some(val);
        }
    }
    None
}

async fn verify_proxy_url(val: &str) -> bool {
    // extract port from url
    if let Ok(url) = url::Url::parse(val) {
        if let Some(port) = url.port() {
            return scanner::check_port(port).await;
        }
    }
    // simple fallback: search for :PORT
    if let Some(idx) = val.rfind(':') {
        let port_part: String = val[idx + 1..].chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(p) = port_part.parse::<u16>() {
            return scanner::check_port(p).await;
        }
    }
    false
}
