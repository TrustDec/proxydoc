use anyhow::Result;
use colored::*;
use std::process::Command;
use tabled::{Table, Tabled};
use tokio::net::lookup_host;

use crate::scanner;

#[derive(Tabled)]
struct DiagItem {
    #[tabled(rename = "Category")]
    category: &'static str,
    #[tabled(rename = "Inspection Target")]
    target: String,
    #[tabled(rename = "Current Value")]
    value: String,
    #[tabled(rename = "Health Diagnosis")]
    status: String,
}

#[allow(dead_code)]
pub struct SystemProxyState {
    pub http_enabled: bool,
    pub http_host: String,
    pub http_port: u16,
    pub socks_enabled: bool,
    pub socks_host: String,
    pub socks_port: u16,
}

pub fn get_macos_system_proxy() -> Option<SystemProxyState> {
    #[cfg(target_os = "macos")]
    {
        let out = Command::new("scutil").arg("--proxy").output().ok()?;
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            let http_enabled = s.contains("HTTPEnable : 1");
            let socks_enabled = s.contains("SOCKSEnable : 1");

            let extract_field = |prefix: &str| -> Option<String> {
                for line in s.lines() {
                    let trim = line.trim();
                    if trim.starts_with(prefix) {
                        if let Some(pos) = trim.find(':') {
                            return Some(trim[pos + 1..].trim().to_string());
                        }
                    }
                }
                None
            };

            let http_host = extract_field("HTTPProxy").unwrap_or_else(|| "127.0.0.1".to_string());
            let http_port = extract_field("HTTPPort")
                .and_then(|p| p.parse().ok())
                .unwrap_or(0);

            let socks_host = extract_field("SOCKSProxy").unwrap_or_else(|| "127.0.0.1".to_string());
            let socks_port = extract_field("SOCKSPort")
                .and_then(|p| p.parse().ok())
                .unwrap_or(0);

            return Some(SystemProxyState {
                http_enabled,
                http_host,
                http_port,
                socks_enabled,
                socks_host,
                socks_port,
            });
        }
    }
    None
}

pub async fn run(_verbose: bool) -> Result<()> {
    println!("\n{}", "🩺 ==================== DEEP PROXY DOCTOR ==================== 🩺".bold().cyan());
    println!("{}", "   Running comprehensive root-cause network & environment diagnostics...\n".dimmed());

    let mut items = Vec::new();
    let mut root_issues: Vec<String> = Vec::new();

    // 1. macOS System Proxy vs Current Terminal Sync
    if let Some(sys) = get_macos_system_proxy() {
        if sys.http_enabled && sys.http_port > 0 {
            let active = scanner::check_port(sys.http_port).await;
            items.push(DiagItem {
                category: "System Proxy",
                target: "macOS Web Proxy (scutil)".to_string(),
                value: format!("{}:{}", sys.http_host, sys.http_port),
                status: if active {
                    "● ACTIVE in OS (Port Open)".green().to_string()
                } else {
                    root_issues.push(format!("System proxy is pointing to dead port {}", sys.http_port));
                    "▲ OS Proxy enabled, but port is DEAD!".red().bold().to_string()
                },
            });

            // Check if terminal inherited this
            let term_set = std::env::var("http_proxy").or_else(|_| std::env::var("HTTP_PROXY")).is_ok();
            if !term_set {
                root_issues.push(format!(
                    "macOS GUI proxy is active ({}:{}), but your Terminal is DIRECT! Run `proxydoc fix` to sync.",
                    sys.http_host, sys.http_port
                ));
                items.push(DiagItem {
                    category: "Sync Conflict",
                    target: "Terminal vs OS".to_string(),
                    value: "OS=ON | Terminal=OFF".yellow().to_string(),
                    status: "▲ Out of sync! (Browser works, Terminal hangs)".yellow().bold().to_string(),
                });
            }
        } else {
            items.push(DiagItem {
                category: "System Proxy",
                target: "macOS Web Proxy (scutil)".to_string(),
                value: "(disabled)".dimmed().to_string(),
                status: "○ Direct in OS".dimmed().to_string(),
            });
        }
    }

    // 2. Terminal Shell Env Vars
    let env_vars = ["http_proxy", "https_proxy", "all_proxy", "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY"];
    let mut shell_proxy_set = false;
    for var in env_vars {
        if let Ok(val) = std::env::var(var) {
            shell_proxy_set = true;
            let healthy = verify_proxy_url(&val).await;
            if !healthy {
                root_issues.push(format!("Terminal env `{}` points to an unreachable port `{}`", var, val));
            }
            items.push(DiagItem {
                category: "Shell Env",
                target: var.to_string(),
                value: val.clone(),
                status: if healthy {
                    "● Reachable Port".green().to_string()
                } else {
                    "▲ DEAD PROXY PORT (Causes curl error 7)".red().bold().to_string()
                },
            });
        }
    }

    if !shell_proxy_set {
        items.push(DiagItem {
            category: "Shell Env",
            target: "Terminal Proxy".to_string(),
            value: "(not set)".dimmed().to_string(),
            status: "○ Direct (Inherits nothing)".dimmed().to_string(),
        });
    }

    // 3. Git Config
    let git_http = get_git_config("http.proxy");
    let git_https = get_git_config("https.proxy");
    if git_http.is_none() && git_https.is_none() {
        items.push(DiagItem {
            category: "Git",
            target: "git config --global *.proxy".to_string(),
            value: "(not set)".dimmed().to_string(),
            status: "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        });
    } else {
        if let Some(val) = git_http {
            let ok = verify_proxy_url(&val).await;
            if !ok {
                root_issues.push(format!("git config --global http.proxy is pointing to dead port `{}`", val));
            }
            items.push(DiagItem {
                category: "Git",
                target: "http.proxy".to_string(),
                value: val,
                status: if ok { "● Reachable".green().to_string() } else { "▲ DEAD PORT! (Git clone will fail)".red().bold().to_string() },
            });
        }
        if let Some(val) = git_https {
            let ok = verify_proxy_url(&val).await;
            if !ok {
                root_issues.push(format!("git config --global https.proxy is pointing to dead port `{}`", val));
            }
            items.push(DiagItem {
                category: "Git",
                target: "https.proxy".to_string(),
                value: val,
                status: if ok { "● Reachable".green().to_string() } else { "▲ DEAD PORT!".red().bold().to_string() },
            });
        }
    }

    // 4. DNS Poisoning & Resolution Check
    let dns_targets = [
        ("github.com", 443),
        ("raw.githubusercontent.com", 443),
        ("api.openai.com", 443),
        ("api.anthropic.com", 443),
    ];

    for (domain, port) in dns_targets {
        let lookup_res = lookup_host(format!("{}:{}", domain, port)).await;
        match lookup_res {
            Ok(mut addrs) => {
                if let Some(first) = addrs.next() {
                    let ip = first.ip();
                    let is_poisoned = ip.is_loopback() || ip.is_unspecified() || ip.to_string() == "0.0.0.0";
                    if is_poisoned {
                        root_issues.push(format!("DNS for {} is poisoned/blocked (Resolved to {})", domain, ip));
                        items.push(DiagItem {
                            category: "DNS / Hosts",
                            target: domain.to_string(),
                            value: ip.to_string(),
                            status: "▲ POISONED / BLOCKED!".red().bold().to_string(),
                        });
                    } else {
                        items.push(DiagItem {
                            category: "DNS Check",
                            target: domain.to_string(),
                            value: ip.to_string(),
                            status: "● Normal resolution".green().to_string(),
                        });
                    }
                }
            }
            Err(e) => {
                root_issues.push(format!("Failed to resolve DNS for {}: {}", domain, e));
                items.push(DiagItem {
                    category: "DNS Check",
                    target: domain.to_string(),
                    value: "Resolution failed".to_string(),
                    status: "▲ DNS Resolution Failed".yellow().to_string(),
                });
            }
        }
    }

    // 5. Active Daemons
    let active_client = scanner::find_first_active_proxy().await;
    match active_client {
        Some(p) => {
            items.push(DiagItem {
                category: "Local Daemon",
                target: format!("Port {}", p.port),
                value: p.client_hint.to_string(),
                status: "● OPEN & READY".green().bold().to_string(),
            });
        }
        None => {
            items.push(DiagItem {
                category: "Local Daemon",
                target: "Common Ports".to_string(),
                value: "7890, 7897, 1080, 1082...".dimmed().to_string(),
                status: "▲ No local proxy client found".yellow().to_string(),
            });
        }
    }

    let table = Table::new(items).to_string();
    println!("{}\n", table);

    // Root-Cause Analysis Summary
    if !root_issues.is_empty() {
        println!("{}", "🚨 Detected Root-Cause Issues:".red().bold());
        for (i, issue) in root_issues.iter().enumerate() {
            println!("  {}. {}", (i + 1).to_string().yellow(), issue);
        }
        println!(
            "\n{}",
            "👉 Run `proxydoc fix` to automatically repair and synchronize these settings!"
                .green()
                .bold()
        );
    } else {
        println!("{}", "✨ All configurations healthy and aligned! No dead ports detected.".green().bold());
    }

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

pub async fn verify_proxy_url(val: &str) -> bool {
    if let Ok(url) = url::Url::parse(val) {
        if let Some(port) = url.port() {
            return scanner::check_port(port).await;
        }
    }
    if let Some(idx) = val.rfind(':') {
        let port_part: String = val[idx + 1..].chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(p) = port_part.parse::<u16>() {
            return scanner::check_port(p).await;
        }
    }
    false
}
