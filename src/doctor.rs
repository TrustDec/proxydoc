use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tabled::{Table, Tabled};
use tokio::net::lookup_host;

use crate::scanner;

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct DiagItem {
    #[tabled(rename = "Category")]
    pub category: String,
    #[tabled(rename = "Inspection Target")]
    pub target: String,
    #[tabled(rename = "Current Value")]
    pub value: String,
    #[tabled(rename = "Health Diagnosis")]
    pub status: String,
}

impl DiagItem {
    pub fn new(category: &str, target: impl Into<String>, value: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            category: category.to_string(),
            target: target.into(),
            value: value.into(),
            status: status.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoctorReport {
    pub healthy: bool,
    pub items: Vec<DiagItem>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemProxyState {
    pub http_enabled: bool,
    pub http_host: String,
    pub http_port: u16,
    pub https_enabled: bool,
    pub https_host: String,
    pub https_port: u16,
    pub socks_enabled: bool,
    pub socks_host: String,
    pub socks_port: u16,
}

pub fn parse_scutil_proxy(s: &str) -> Option<SystemProxyState> {
    let extract_bool = |key: &str| -> bool {
        for line in s.lines() {
            let trim = line.trim();
            if let Some((k, v)) = trim.split_once(':') {
                if k.trim() == key {
                    return v.trim() == "1";
                }
            }
        }
        false
    };

    let extract_field = |key: &str| -> Option<String> {
        for line in s.lines() {
            let trim = line.trim();
            if let Some((k, v)) = trim.split_once(':') {
                if k.trim() == key {
                    let val = v.trim();
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
        None
    };

    let http_enabled = extract_bool("HTTPEnable");
    let https_enabled = extract_bool("HTTPSEnable");
    let socks_enabled = extract_bool("SOCKSEnable");

    let http_host = extract_field("HTTPProxy").unwrap_or_else(|| "127.0.0.1".to_string());
    let http_port = extract_field("HTTPPort").and_then(|p| p.parse().ok()).unwrap_or(0);

    let https_host = extract_field("HTTPSProxy").unwrap_or_else(|| "127.0.0.1".to_string());
    let https_port = extract_field("HTTPSPort").and_then(|p| p.parse().ok()).unwrap_or(0);

    let socks_host = extract_field("SOCKSProxy").unwrap_or_else(|| "127.0.0.1".to_string());
    let socks_port = extract_field("SOCKSPort").and_then(|p| p.parse().ok()).unwrap_or(0);

    Some(SystemProxyState {
        http_enabled,
        http_host,
        http_port,
        https_enabled,
        https_host,
        https_port,
        socks_enabled,
        socks_host,
        socks_port,
    })
}

pub fn get_macos_system_proxy() -> Option<SystemProxyState> {
    #[cfg(target_os = "macos")]
    {
        let out = Command::new("scutil").arg("--proxy").output().ok()?;
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            return parse_scutil_proxy(&s);
        }
    }
    None
}

pub fn get_npm_config(key: &str) -> Option<String> {
    let out = Command::new("npm").args(["config", "get", key]).output().ok()?;
    if out.status.success() {
        let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !val.is_empty() && val != "null" && val != "undefined" {
            return Some(val);
        }
    }
    None
}

pub fn parse_cargo_proxy_toml(content: &str) -> Option<String> {
    let mut in_http = false;
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if trim.starts_with('[') {
            in_http = trim == "[http]";
            continue;
        }
        if in_http {
            if let Some((k, v)) = trim.split_once('=') {
                if k.trim() == "proxy" {
                    let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }
    None
}

pub fn get_cargo_proxy() -> Option<String> {
    if let Ok(val) = std::env::var("CARGO_HTTP_PROXY") {
        if !val.trim().is_empty() {
            return Some(val.trim().to_string());
        }
    }
    if let Some(home) = dirs::home_dir() {
        let paths = [
            home.join(".cargo").join("config.toml"),
            home.join(".cargo").join("config"),
        ];
        for path in paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Some(proxy) = parse_cargo_proxy_toml(&content) {
                    return Some(proxy);
                }
            }
        }
    }
    None
}

pub async fn run(_verbose: bool, json_output: bool) -> Result<()> {
    if !json_output {
        println!("\n{}", "🩺 ==================== DEEP PROXY DOCTOR ==================== 🩺".bold().cyan());
        println!("{}", "   Running comprehensive root-cause network & environment diagnostics...\n".dimmed());
    }

    let mut items = Vec::new();
    let mut root_issues: Vec<String> = Vec::new();

    // 1. macOS System Proxy vs Current Terminal Sync
    if let Some(sys) = get_macos_system_proxy() {
        if sys.http_enabled && sys.http_port > 0 {
            let active = scanner::check_host_port(&sys.http_host, sys.http_port).await;
            items.push(DiagItem::new(
                "System Proxy",
                "macOS Web Proxy (scutil)",
                format!("{}:{}", sys.http_host, sys.http_port),
                if active {
                    "● ACTIVE in OS (Port Open)".green().to_string()
                } else {
                    root_issues.push(format!("System proxy is pointing to dead port {}", sys.http_port));
                    "▲ OS Proxy enabled, but port is DEAD!".red().bold().to_string()
                },
            ));

            // Check if terminal inherited this
            let term_set = std::env::var("http_proxy").or_else(|_| std::env::var("HTTP_PROXY")).is_ok();
            if !term_set {
                root_issues.push(format!(
                    "macOS GUI proxy is active ({}:{}), but your Terminal is DIRECT! Run `proxydoc fix` to sync.",
                    sys.http_host, sys.http_port
                ));
                items.push(DiagItem::new(
                    "Sync Conflict",
                    "Terminal vs OS",
                    "OS=ON | Terminal=OFF".yellow().to_string(),
                    "▲ Out of sync! (Browser works, Terminal hangs)".yellow().bold().to_string(),
                ));
            }
        } else {
            items.push(DiagItem::new(
                "System Proxy",
                "macOS Web Proxy (scutil)",
                "(disabled)".dimmed().to_string(),
                "○ Direct in OS".dimmed().to_string(),
            ));
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
            items.push(DiagItem::new(
                "Shell Env",
                var,
                val.clone(),
                if healthy {
                    "● Reachable Port".green().to_string()
                } else {
                    "▲ DEAD PROXY PORT (Causes curl error 7)".red().bold().to_string()
                },
            ));
        }
    }

    if !shell_proxy_set {
        items.push(DiagItem::new(
            "Shell Env",
            "Terminal Proxy",
            "(not set)".dimmed().to_string(),
            "○ Direct (Inherits nothing)".dimmed().to_string(),
        ));
    }

    // 3. Git Config
    let git_http = get_git_config("http.proxy");
    let git_https = get_git_config("https.proxy");
    if git_http.is_none() && git_https.is_none() {
        items.push(DiagItem::new(
            "Git",
            "git config --global *.proxy",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    } else {
        if let Some(val) = git_http {
            let ok = verify_proxy_url(&val).await;
            if !ok {
                root_issues.push(format!("git config --global http.proxy is pointing to dead port `{}`", val));
            }
            items.push(DiagItem::new(
                "Git",
                "http.proxy",
                val,
                if ok { "● Reachable".green().to_string() } else { "▲ DEAD PORT! (Git clone will fail)".red().bold().to_string() },
            ));
        }
        if let Some(val) = git_https {
            let ok = verify_proxy_url(&val).await;
            if !ok {
                root_issues.push(format!("git config --global https.proxy is pointing to dead port `{}`", val));
            }
            items.push(DiagItem::new(
                "Git",
                "https.proxy",
                val,
                if ok { "● Reachable".green().to_string() } else { "▲ DEAD PORT!".red().bold().to_string() },
            ));
        }
    }

    // 4. NPM Config
    let npm_http = get_npm_config("proxy");
    let npm_https = get_npm_config("https-proxy");
    if npm_http.is_none() && npm_https.is_none() {
        items.push(DiagItem::new(
            "NPM",
            "npm config (proxy/https-proxy)",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    } else {
        if let Some(val) = npm_http {
            let ok = verify_proxy_url(&val).await;
            if !ok {
                root_issues.push(format!("npm config proxy points to dead port `{}` (npm install will fail)", val));
            }
            items.push(DiagItem::new(
                "NPM",
                "proxy",
                val,
                if ok { "● Reachable".green().to_string() } else { "▲ DEAD PORT! (npm install will fail)".red().bold().to_string() },
            ));
        }
        if let Some(val) = npm_https {
            let ok = verify_proxy_url(&val).await;
            if !ok {
                root_issues.push(format!("npm config https-proxy points to dead port `{}`", val));
            }
            items.push(DiagItem::new(
                "NPM",
                "https-proxy",
                val,
                if ok { "● Reachable".green().to_string() } else { "▲ DEAD PORT!".red().bold().to_string() },
            ));
        }
    }

    // 5. Cargo Config
    if let Some(val) = get_cargo_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!("Cargo [http] proxy points to dead port `{}` (cargo fetch will fail)", val));
        }
        items.push(DiagItem::new(
            "Cargo",
            "cargo config [http] proxy",
            val,
            if ok { "● Reachable".green().to_string() } else { "▲ DEAD PORT! (cargo will fail)".red().bold().to_string() },
        ));
    } else {
        items.push(DiagItem::new(
            "Cargo",
            "cargo config [http] proxy",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // 6. DNS Poisoning & Resolution Check
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
                        items.push(DiagItem::new(
                            "DNS / Hosts",
                            domain,
                            ip.to_string(),
                            "▲ POISONED / BLOCKED!".red().bold().to_string(),
                        ));
                    } else {
                        items.push(DiagItem::new(
                            "DNS Check",
                            domain,
                            ip.to_string(),
                            "● Normal resolution".green().to_string(),
                        ));
                    }
                }
            }
            Err(e) => {
                root_issues.push(format!("Failed to resolve DNS for {}: {}", domain, e));
                items.push(DiagItem::new(
                    "DNS Check",
                    domain,
                    "Resolution failed",
                    "▲ DNS Resolution Failed".yellow().to_string(),
                ));
            }
        }
    }

    // 7. Active Daemons
    let active_client = scanner::find_first_active_proxy().await;
    match active_client {
        Some(p) => {
            items.push(DiagItem::new(
                "Local Daemon",
                format!("Port {}", p.port),
                p.client_hint,
                "● OPEN & READY".green().bold().to_string(),
            ));
        }
        None => {
            items.push(DiagItem::new(
                "Local Daemon",
                "Common Ports",
                "7890, 7897, 1080, 1082...".dimmed().to_string(),
                "▲ No local proxy client found".yellow().to_string(),
            ));
        }
    }

    if json_output {
        let report = DoctorReport {
            healthy: root_issues.is_empty(),
            items,
            issues: root_issues,
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    let table = Table::new(&items).to_string();
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
            let host = url.host_str().unwrap_or("127.0.0.1");
            return scanner::check_host_port(host, port).await;
        }
    }
    if let Some(idx) = val.rfind(':') {
        let host_part = &val[..idx];
        let port_part: String = val[idx + 1..].chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(p) = port_part.parse::<u16>() {
            let host = if host_part.is_empty() || host_part == "localhost" {
                "127.0.0.1"
            } else {
                host_part.trim_start_matches("http://").trim_start_matches("https://").trim_start_matches("socks5://")
            };
            return scanner::check_host_port(host, p).await;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scutil_proxy() {
        let raw = r#"<dictionary> {
  HTTPEnable : 1
  HTTPPort : 1082
  HTTPProxy : 127.0.0.1
  HTTPSEnable : 1
  HTTPSPort : 1082
  HTTPSProxy : 127.0.0.1
  SOCKSEnable : 0
}"#;
        let sys = parse_scutil_proxy(raw).unwrap();
        assert!(sys.http_enabled);
        assert_eq!(sys.http_port, 1082);
        assert_eq!(sys.http_host, "127.0.0.1");
        assert!(sys.https_enabled);
        assert_eq!(sys.https_port, 1082);
        assert!(!sys.socks_enabled);
    }

    #[test]
    fn test_parse_scutil_proxy_disabled() {
        let raw = r#"<dictionary> {
  HTTPEnable : 0
  HTTPSEnable : 0
  SOCKSEnable : 0
}"#;
        let sys = parse_scutil_proxy(raw).unwrap();
        assert!(!sys.http_enabled);
        assert!(!sys.https_enabled);
        assert!(!sys.socks_enabled);
    }

    #[test]
    fn test_parse_cargo_proxy_toml() {
        let content = r#"
[http]
proxy = "http://127.0.0.1:7890"

[build]
jobs = 8
"#;
        assert_eq!(parse_cargo_proxy_toml(content), Some("http://127.0.0.1:7890".to_string()));

        let empty = r#"
[build]
jobs = 4
"#;
        assert_eq!(parse_cargo_proxy_toml(empty), None);
    }

    #[tokio::test]
    async fn test_verify_proxy_url_with_active_listener() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Should return true for active port
        assert!(verify_proxy_url(&format!("http://127.0.0.1:{}", port)).await);
        assert!(verify_proxy_url(&format!("127.0.0.1:{}", port)).await);

        // Drop the listener to make port dead
        drop(listener);
        assert!(!verify_proxy_url("invalid_url_without_port").await);
    }

    #[tokio::test]
    async fn test_verify_proxy_url_invalid() {
        assert!(!verify_proxy_url("").await);
        assert!(!verify_proxy_url("not_a_proxy").await);
    }
}
