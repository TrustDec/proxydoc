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

pub fn parse_npmrc_proxy(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            if k.trim().eq_ignore_ascii_case(key) {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() && val != "null" && val != "undefined" {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

pub fn get_npm_config(key: &str) -> Option<String> {
    if let Ok(out) = Command::new("npm").args(["config", "get", key]).output() {
        if out.status.success() {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !val.is_empty() && val != "null" && val != "undefined" {
                return Some(val);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let npmrc = home.join(".npmrc");
        if let Ok(content) = std::fs::read_to_string(npmrc) {
            if let Some(val) = parse_npmrc_proxy(&content, key) {
                return Some(val);
            }
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

pub fn parse_curlrc_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if let Some(rest) = trim.strip_prefix("proxy") {
            let val = rest.trim_start_matches(|c: char| c == '=' || c.is_whitespace());
            let val = val.trim_matches('"').trim_matches('\'').trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        } else if let Some(rest) = trim.strip_prefix("-x") {
            let val = rest.trim().trim_matches('"').trim_matches('\'').trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        } else if let Some(rest) = trim.strip_prefix("--proxy") {
            let val = rest.trim_start_matches(|c: char| c == '=' || c.is_whitespace());
            let val = val.trim_matches('"').trim_matches('\'').trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

pub fn get_curlrc_proxy() -> Option<String> {
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".curlrc");
        if let Ok(content) = std::fs::read_to_string(path) {
            return parse_curlrc_proxy(&content);
        }
    }
    None
}

pub fn parse_pip_config_proxy(content: &str) -> Option<String> {
    let mut in_global = false;
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if trim.starts_with('[') {
            let sec = trim.trim_matches(|c| c == '[' || c == ']').trim();
            in_global = sec.eq_ignore_ascii_case("global");
            continue;
        }
        if in_global {
            if let Some((k, v)) = trim.split_once('=') {
                if k.trim().eq_ignore_ascii_case("proxy") {
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

pub fn get_pip_proxy() -> Option<String> {
    if let Ok(val) = std::env::var("PIP_PROXY") {
        if !val.trim().is_empty() {
            return Some(val.trim().to_string());
        }
    }
    if let Ok(out) = Command::new("pip").args(["config", "get", "global.proxy"]).output() {
        if out.status.success() {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !val.is_empty() && val != "null" && val != "undefined" {
                return Some(val);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let paths = [
            home.join(".config").join("pip").join("pip.conf"),
            home.join(".pip").join("pip.conf"),
        ];
        for path in paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Some(p) = parse_pip_config_proxy(&content) {
                    return Some(p);
                }
            }
        }
    }
    None
}

pub fn check_socks5_dns_leak(val: &str) -> bool {
    let lower = val.to_ascii_lowercase();
    lower.starts_with("socks5://") && !lower.starts_with("socks5h://")
}

pub fn parse_docker_config_proxy(content: &str) -> Option<String> {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(http) = v.pointer("/proxies/default/httpProxy").and_then(|s| s.as_str()) {
            let trim = http.trim();
            if !trim.is_empty() {
                return Some(trim.to_string());
            }
        }
        if let Some(https) = v.pointer("/proxies/default/httpsProxy").and_then(|s| s.as_str()) {
            let trim = https.trim();
            if !trim.is_empty() {
                return Some(trim.to_string());
            }
        }
    }
    None
}

pub fn get_docker_proxy() -> Option<String> {
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".docker").join("config.json");
        if let Ok(content) = std::fs::read_to_string(path) {
            return parse_docker_config_proxy(&content);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GradleProxyConfig {
    pub host: String,
    pub port: u16,
}

pub fn parse_gradle_properties_proxy(content: &str) -> Option<GradleProxyConfig> {
    let mut host = None;
    let mut port = None;
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with('!') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            let key = k.trim();
            let val = v.trim().trim_matches('"').trim_matches('\'').trim();
            if key == "systemProp.http.proxyHost" || key == "systemProp.https.proxyHost" {
                if !val.is_empty() {
                    host = Some(val.to_string());
                }
            } else if key == "systemProp.http.proxyPort" || key == "systemProp.https.proxyPort" {
                if let Ok(p) = val.parse::<u16>() {
                    port = Some(p);
                }
            }
        }
    }
    match (host, port) {
        (Some(h), Some(p)) => Some(GradleProxyConfig { host: h, port: p }),
        _ => None,
    }
}

pub fn get_gradle_proxy() -> Option<GradleProxyConfig> {
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".gradle").join("gradle.properties");
        if let Ok(content) = std::fs::read_to_string(path) {
            return parse_gradle_properties_proxy(&content);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MavenProxyConfig {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub active: bool,
    pub protocol: String,
}

pub fn strip_xml_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut remaining = content;
    while let Some(start) = remaining.find("<!--") {
        result.push_str(&remaining[..start]);
        if let Some(end) = remaining[start + 4..].find("-->") {
            remaining = &remaining[start + 4 + end + 3..];
        } else {
            return result;
        }
    }
    result.push_str(remaining);
    result
}

pub fn parse_maven_settings_proxy(content: &str) -> Option<MavenProxyConfig> {
    let clean = strip_xml_comments(content);
    let mut cursor = clean.as_str();

    while let Some(start_idx) = cursor.find("<proxy>") {
        let after_start = &cursor[start_idx + 7..];
        let end_idx = match after_start.find("</proxy>") {
            Some(i) => i,
            None => break,
        };
        let proxy_block = &after_start[..end_idx];
        cursor = &after_start[end_idx + 8..];

        let extract_tag = |tag: &str| -> Option<String> {
            let open = format!("<{}>", tag);
            let close = format!("</{}>", tag);
            let s = proxy_block.find(&open)?;
            let rest = &proxy_block[s + open.len()..];
            let e = rest.find(&close)?;
            let val = rest[..e].trim();
            if !val.is_empty() {
                Some(val.to_string())
            } else {
                None
            }
        };

        let is_active = extract_tag("active")
            .map(|a| a.eq_ignore_ascii_case("true"))
            .unwrap_or(true);

        if is_active {
            let host = extract_tag("host");
            let port = extract_tag("port").and_then(|p| p.parse::<u16>().ok());
            let id = extract_tag("id").unwrap_or_else(|| "proxy".to_string());
            let protocol = extract_tag("protocol").unwrap_or_else(|| "http".to_string());

            if let (Some(h), Some(p)) = (host, port) {
                return Some(MavenProxyConfig {
                    id,
                    host: h,
                    port: p,
                    active: true,
                    protocol,
                });
            }
        }
    }
    None
}

pub fn get_maven_proxy() -> Option<MavenProxyConfig> {
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".m2").join("settings.xml");
        if let Ok(content) = std::fs::read_to_string(path) {
            return parse_maven_settings_proxy(&content);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitRewrite {
    pub replacement: String,
    pub original: String,
}

pub fn get_git_insteadof_rewrites() -> Vec<GitRewrite> {
    let mut rewrites = Vec::new();
    let out = Command::new("git")
        .args(["config", "--global", "--get-regexp", r"^url\..*\.insteadof"])
        .output();

    if let Ok(output) = out {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            for line in s.lines() {
                let trim = line.trim();
                if let Some((k, v)) = trim.split_once(' ') {
                    let key = k.trim();
                    let orig = v.trim();
                    if let Some(rest) = key.strip_prefix("url.") {
                        let lower = rest.to_ascii_lowercase();
                        if let Some(idx) = lower.rfind(".insteadof") {
                            let repl = &rest[..idx];
                            rewrites.push(GitRewrite {
                                replacement: repl.to_string(),
                                original: orig.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    rewrites
}

pub fn parse_goproxy(val: &str) -> Vec<String> {
    val.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "direct" && s != "off")
        .collect()
}

pub fn get_goproxy() -> Option<String> {
    if let Ok(val) = std::env::var("GOPROXY") {
        let trim = val.trim();
        if !trim.is_empty() {
            return Some(trim.to_string());
        }
    }
    if let Ok(out) = Command::new("go").args(["env", "GOPROXY"]).output() {
        if out.status.success() {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}
pub fn parse_wgetrc_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            let key = k.trim();
            let val = v.trim().trim_matches('"').trim_matches('\'').trim();
            if (key.eq_ignore_ascii_case("http_proxy") || key.eq_ignore_ascii_case("https_proxy")) && !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

pub fn get_wgetrc_proxy() -> Option<String> {
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".wgetrc");
        if let Ok(content) = std::fs::read_to_string(path) {
            return parse_wgetrc_proxy(&content);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshProxyCommand {
    pub host_pattern: String,
    pub command: String,
    pub target: Option<String>,
}

pub fn parse_ssh_config_proxy(content: &str) -> Vec<SshProxyCommand> {
    let mut results = Vec::new();
    let mut current_host = "*".to_string();

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trim.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        if parts[0].eq_ignore_ascii_case("Host") && parts.len() > 1 {
            current_host = parts[1..].join(" ");
        } else if parts[0].eq_ignore_ascii_case("ProxyCommand") {
            let command = trim["ProxyCommand".len()..].trim().to_string();
            let mut target = None;
            for (i, &token) in parts.iter().enumerate() {
                if (token == "-x" || token == "-S" || token == "-H") && i + 1 < parts.len() {
                    target = Some(parts[i + 1].to_string());
                    break;
                }
                if token.contains(':') && !token.starts_with('%') {
                    if let Some((h, p)) = token.split_once(':') {
                        if p.parse::<u16>().is_ok() {
                            target = Some(format!("{}:{}", h, p));
                            break;
                        }
                    }
                }
            }
            results.push(SshProxyCommand {
                host_pattern: current_host.clone(),
                command,
                target,
            });
        }
    }

    results
}

pub fn get_ssh_config_proxies() -> Vec<SshProxyCommand> {
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".ssh").join("config");
        if let Ok(content) = std::fs::read_to_string(path) {
            return parse_ssh_config_proxy(&content);
        }
    }
    Vec::new()
}

pub fn detect_wsl2() -> Option<String> {
    if cfg!(target_os = "linux") {
        let is_wsl = std::env::var("WSL_DISTRO_NAME").is_ok()
            || std::env::var("WSL_INTEROP").is_ok()
            || std::fs::read_to_string("/proc/version")
                .map(|v| v.to_ascii_lowercase().contains("microsoft"))
                .unwrap_or(false);

        if is_wsl {
            if let Ok(resolv) = std::fs::read_to_string("/etc/resolv.conf") {
                for line in resolv.lines() {
                    let trim = line.trim();
                    if let Some(rest) = trim.strip_prefix("nameserver") {
                        let ip = rest.trim();
                        if !ip.is_empty() && ip != "127.0.0.1" {
                            return Some(ip.to_string());
                        }
                    }
                }
            }
            return Some("Windows Host Gateway".to_string());
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
            let dns_leak = check_socks5_dns_leak(&val);

            if !healthy {
                root_issues.push(format!("Terminal env `{}` points to an unreachable port `{}`", var, val));
            } else if dns_leak {
                root_issues.push(format!(
                    "Terminal env `{}` uses socks5:// instead of socks5h://! DNS queries will resolve locally and may leak or be poisoned.",
                    var
                ));
            }

            let status = if !healthy {
                "▲ DEAD PROXY PORT (Causes curl error 7)".red().bold().to_string()
            } else if dns_leak {
                "▲ Reachable, but socks5:// leaks local DNS (use socks5h://)".yellow().to_string()
            } else {
                "● Reachable Port".green().to_string()
            };

            items.push(DiagItem::new(
                "Shell Env",
                var,
                val.clone(),
                status,
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
    } else {
        // Inspect no_proxy / NO_PROXY to guarantee localhost / 127.0.0.1 bypass
        let no_proxy = std::env::var("no_proxy").or_else(|_| std::env::var("NO_PROXY")).ok();
        match no_proxy {
            Some(ref val) => {
                let has_localhost = val.contains("localhost") && (val.contains("127.0.0.1") || val.contains("::1"));
                if !has_localhost {
                    root_issues.push(format!(
                        "NO_PROXY is set (`{}`) but misses `localhost` or `127.0.0.1` (local dev servers like Vite/Docker will route to proxy and fail)",
                        val
                    ));
                    items.push(DiagItem::new(
                        "Shell Env",
                        "no_proxy / NO_PROXY",
                        val.clone(),
                        "▲ Missing localhost bypass".yellow().to_string(),
                    ));
                } else {
                    items.push(DiagItem::new(
                        "Shell Env",
                        "no_proxy / NO_PROXY",
                        val.clone(),
                        "● Localhost bypass configured".green().to_string(),
                    ));
                }
            }
            None => {
                root_issues.push("Terminal proxy is set, but `no_proxy` is MISSING! (Local dev servers like Vite/Next.js/Docker will fail)".to_string());
                items.push(DiagItem::new(
                    "Shell Env",
                    "no_proxy / NO_PROXY",
                    "(not set)".yellow().to_string(),
                    "▲ Missing (Local servers might hang)".yellow().bold().to_string(),
                ));
            }
        }
    }

    // WSL2 Environment Check
    if let Some(host_ip) = detect_wsl2() {
        let term_proxy = std::env::var("http_proxy").or_else(|_| std::env::var("all_proxy")).unwrap_or_default();
        if term_proxy.contains("127.0.0.1") || term_proxy.contains("localhost") {
            root_issues.push(format!(
                "WSL2 detected! `127.0.0.1` routes inside Linux VM. Use Windows host IP `{}` or mirrored networking mode.",
                host_ip
            ));
            items.push(DiagItem::new(
                "WSL2 Host",
                "WSL2 Windows Gateway",
                host_ip,
                "▲ 127.0.0.1 routes inside VM (use Host IP)".yellow().bold().to_string(),
            ));
        } else {
            items.push(DiagItem::new(
                "WSL2 Host",
                "WSL2 Windows Gateway",
                host_ip,
                "● WSL2 Environment Active".dimmed().to_string(),
            ));
        }
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

    // Git URL Rewrites (insteadOf)
    let rewrites = get_git_insteadof_rewrites();
    for rw in rewrites {
        if let Ok(parsed_url) = url::Url::parse(&rw.replacement) {
            if let Some(host) = parsed_url.host_str() {
                let host_resolves = lookup_host(format!("{}:443", host)).await.is_ok();
                if !host_resolves {
                    root_issues.push(format!(
                        "Git URL rewrite `url.{}.insteadOf {}` has unreachable mirror `{}`! (Git clone will fail with 'Could not resolve host')",
                        rw.replacement, rw.original, host
                    ));
                    items.push(DiagItem::new(
                        "Git Rewrite",
                        format!("insteadOf {}", rw.original),
                        rw.replacement.clone(),
                        "▲ UNRESOLVABLE MIRROR!".red().bold().to_string(),
                    ));
                } else {
                    items.push(DiagItem::new(
                        "Git Rewrite",
                        format!("insteadOf {}", rw.original),
                        rw.replacement.clone(),
                        "● Resolvable Mirror".green().to_string(),
                    ));
                }
            }
        }
    }

    // 4. SSH Config ProxyCommand (~/.ssh/config)
    let ssh_proxies = get_ssh_config_proxies();
    if ssh_proxies.is_empty() {
        items.push(DiagItem::new(
            "SSH Config",
            "~/.ssh/config ProxyCommand",
            "(not set)".dimmed().to_string(),
            "○ Clean (Direct SSH connection)".dimmed().to_string(),
        ));
    } else {
        for cmd in ssh_proxies {
            if let Some(ref target) = cmd.target {
                let ok = verify_proxy_url(target).await;
                if !ok {
                    root_issues.push(format!(
                        "~/.ssh/config for Host `{}` has dead ProxyCommand target `{}` (Git SSH clones will hang!)",
                        cmd.host_pattern, target
                    ));
                }
                items.push(DiagItem::new(
                    "SSH Config",
                    format!("Host {} (ProxyCommand)", cmd.host_pattern),
                    format!("{} [{}]", cmd.command, target),
                    if ok {
                        "● Reachable Port".green().to_string()
                    } else {
                        "▲ DEAD PORT! (Git SSH will hang)".red().bold().to_string()
                    },
                ));
            } else {
                items.push(DiagItem::new(
                    "SSH Config",
                    format!("Host {} (ProxyCommand)", cmd.host_pattern),
                    cmd.command,
                    "● Configured".dimmed().to_string(),
                ));
            }
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

    // 6. Pip Config
    if let Some(val) = get_pip_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!("Pip proxy points to dead port `{}` (pip install will fail)", val));
        }
        items.push(DiagItem::new(
            "Pip",
            "pip global.proxy",
            val,
            if ok { "● Reachable".green().to_string() } else { "▲ DEAD PORT! (pip install will fail)".red().bold().to_string() },
        ));
    } else {
        items.push(DiagItem::new(
            "Pip",
            "pip global.proxy",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // 7. cURL Config (~/.curlrc)
    if let Some(val) = get_curlrc_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!("~/.curlrc points to dead proxy `{}` (curl commands & bash installers will fail)", val));
        }
        items.push(DiagItem::new(
            "cURL",
            "~/.curlrc proxy",
            val,
            if ok { "● Reachable".green().to_string() } else { "▲ DEAD PORT! (curl commands fail)".red().bold().to_string() },
        ));
    } else {
        items.push(DiagItem::new(
            "cURL",
            "~/.curlrc proxy",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // 8. Wget Config (~/.wgetrc)
    if let Some(val) = get_wgetrc_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!("~/.wgetrc points to dead proxy `{}` (wget commands will fail)", val));
        }
        items.push(DiagItem::new(
            "Wget",
            "~/.wgetrc proxy",
            val,
            if ok { "● Reachable".green().to_string() } else { "▲ DEAD PORT! (wget will fail)".red().bold().to_string() },
        ));
    } else {
        items.push(DiagItem::new(
            "Wget",
            "~/.wgetrc proxy",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // 9. Docker Client Config (~/.docker/config.json)
    if let Some(val) = get_docker_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!(
                "Docker config (~/.docker/config.json) points to dead proxy `{}` (docker build & pull will fail)",
                val
            ));
        }
        items.push(DiagItem::new(
            "Docker",
            "~/.docker/config.json",
            val,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (docker build/pull will fail)".red().bold().to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Docker",
            "~/.docker/config.json",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // 10. Gradle Config (~/.gradle/gradle.properties)
    if let Some(cfg) = get_gradle_proxy() {
        let ok = scanner::check_host_port(&cfg.host, cfg.port).await;
        if !ok {
            root_issues.push(format!(
                "Gradle config (~/.gradle/gradle.properties) points to dead proxy `{}:{}` (Gradle/Android builds will fail)",
                cfg.host, cfg.port
            ));
        }
        items.push(DiagItem::new(
            "Gradle",
            "~/.gradle/gradle.properties",
            format!("{}:{}", cfg.host, cfg.port),
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (Gradle builds fail)".red().bold().to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Gradle",
            "~/.gradle/gradle.properties",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // 11. Maven Config (~/.m2/settings.xml)
    if let Some(cfg) = get_maven_proxy() {
        let ok = scanner::check_host_port(&cfg.host, cfg.port).await;
        if !ok {
            root_issues.push(format!(
                "Maven config (~/.m2/settings.xml) points to dead proxy `{}:{}` (Maven/Java builds will fail)",
                cfg.host, cfg.port
            ));
        }
        items.push(DiagItem::new(
            "Maven",
            "~/.m2/settings.xml",
            format!("{}:{}", cfg.host, cfg.port),
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (Maven builds fail)".red().bold().to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Maven",
            "~/.m2/settings.xml",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // 12. Go Proxy (GOPROXY)
    if let Some(gp) = get_goproxy() {
        let mirrors = parse_goproxy(&gp);
        let mut any_dead = false;
        for mirror in &mirrors {
            let ok = verify_proxy_url(mirror).await;
            if !ok {
                root_issues.push(format!("GOPROXY mirror `{}` is unreachable! (go get / go build will fail)", mirror));
                any_dead = true;
            }
        }
        let status = if mirrors.is_empty() {
            "○ Direct (direct / off)".dimmed().to_string()
        } else if any_dead {
            "▲ DEAD GOPROXY MIRROR! (go get fails)".red().bold().to_string()
        } else {
            "● Reachable GOPROXY".green().to_string()
        };
        items.push(DiagItem::new(
            "Go Modules",
            "GOPROXY",
            gp,
            status,
        ));
    } else {
        items.push(DiagItem::new(
            "Go Modules",
            "GOPROXY",
            "(default proxy.golang.org,direct)".dimmed().to_string(),
            "○ Default Official Proxy".dimmed().to_string(),
        ));
    }

    // 13. AI Developer Endpoints (Base URLs)
    let ai_vars = [
        "OPENAI_BASE_URL",
        "ANTHROPIC_BASE_URL",
        "DEEPSEEK_BASE_URL",
        "OLLAMA_HOST",
    ];
    let mut ai_found = false;
    for var in ai_vars {
        if let Ok(val) = std::env::var(var) {
            let trim = val.trim();
            if !trim.is_empty() {
                ai_found = true;
                let ok = verify_proxy_url(trim).await;
                if !ok {
                    root_issues.push(format!(
                        "AI gateway `{}` is set to `{}` but target is unreachable! (AI CLI agents will fail)",
                        var, trim
                    ));
                }
                items.push(DiagItem::new(
                    "AI Gateway",
                    var,
                    trim.to_string(),
                    if ok {
                        "● Reachable EndPoint".green().to_string()
                    } else {
                        "▲ UNREACHABLE! (AI agents fail)".red().bold().to_string()
                    },
                ));
            }
        }
    }
    if !ai_found {
        items.push(DiagItem::new(
            "AI Gateway",
            "AI Base URLs (OpenAI/Anthropic/DeepSeek/Ollama)",
            "(default public endpoints)".dimmed().to_string(),
            "○ Direct Official Endpoints".dimmed().to_string(),
        ));
    }

    // 13. Homebrew Environment & Mirrors
    let brew_vars = ["HOMEBREW_BOTTLE_DOMAIN", "HOMEBREW_API_DOMAIN", "HOMEBREW_BREW_GIT_REMOTE"];
    for var in brew_vars {
        if let Ok(val) = std::env::var(var) {
            let trim = val.trim();
            if !trim.is_empty() {
                items.push(DiagItem::new(
                    "Homebrew",
                    var,
                    trim.to_string(),
                    "● Configured Mirror".cyan().to_string(),
                ));
            }
        }
    }

    // 14. CA Certificate Bundles Inspection
    let ca_vars = [
        ("SSL_CERT_FILE", "OpenSSL / Python / Rust CA bundle"),
        ("REQUESTS_CA_BUNDLE", "Python Requests CA bundle"),
        ("CURL_CA_BUNDLE", "cURL CA bundle"),
        ("NODE_EXTRA_CA_CERTS", "Node.js extra CA certs"),
    ];
    for (var, desc) in ca_vars {
        if let Ok(val) = std::env::var(var) {
            let trim = val.trim();
            if !trim.is_empty() {
                let path = std::path::Path::new(trim);
                if path.exists() {
                    items.push(DiagItem::new(
                        "CA Certificate",
                        format!("{} ({})", var, desc),
                        trim.to_string(),
                        "● File exists & valid".green().to_string(),
                    ));
                } else {
                    root_issues.push(format!(
                        "CA certificate variable `{}` points to non-existent file `{}`! (Dev tools like Node/Python/cURL will crash with TLS/SSL errors)",
                        var, trim
                    ));
                    items.push(DiagItem::new(
                        "CA Certificate",
                        format!("{} ({})", var, desc),
                        trim.to_string(),
                        "▲ MISSING FILE! (SSL crashes)".red().bold().to_string(),
                    ));
                }
            }
        }
    }

    // 15. DNS Poisoning & Resolution Check
    let dns_targets = [
        ("github.com", 443),
        ("raw.githubusercontent.com", 443),
        ("api.openai.com", 443),
        ("api.anthropic.com", 443),
        ("api.deepseek.com", 443),
        ("pypi.org", 443),
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

    // 10. Active Daemons
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
        let port = url.port().unwrap_or(match url.scheme() {
            "https" => 443,
            "http" => 80,
            _ => 80,
        });
        let host = url.host_str().unwrap_or("127.0.0.1");
        return scanner::check_host_port(host, port).await;
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

    #[test]
    fn test_parse_curlrc_proxy() {
        let c1 = "proxy = \"http://127.0.0.1:7890\"\n";
        assert_eq!(parse_curlrc_proxy(c1), Some("http://127.0.0.1:7890".to_string()));

        let c2 = "# some comment\n-x 127.0.0.1:1080\n";
        assert_eq!(parse_curlrc_proxy(c2), Some("127.0.0.1:1080".to_string()));

        let c3 = "--proxy socks5://127.0.0.1:1082\n";
        assert_eq!(parse_curlrc_proxy(c3), Some("socks5://127.0.0.1:1082".to_string()));

        let c4 = "# no proxy here\nsilent\n";
        assert_eq!(parse_curlrc_proxy(c4), None);
    }

    #[test]
    fn test_parse_pip_config_proxy() {
        let c1 = r#"
[global]
index-url = https://pypi.org/simple
proxy = http://127.0.0.1:7890
"#;
        assert_eq!(parse_pip_config_proxy(c1), Some("http://127.0.0.1:7890".to_string()));

        let c2 = r#"
[install]
trusted-host = pypi.org
"#;
        assert_eq!(parse_pip_config_proxy(c2), None);
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

    #[test]
    fn test_parse_docker_config_proxy() {
        let json_with_proxy = r#"{
            "auths": {
                "https://index.docker.io/v1/": {}
            },
            "proxies": {
                "default": {
                    "httpProxy": "http://127.0.0.1:7890",
                    "httpsProxy": "http://127.0.0.1:7890",
                    "noProxy": "localhost,127.0.0.1"
                }
            }
        }"#;
        assert_eq!(parse_docker_config_proxy(json_with_proxy), Some("http://127.0.0.1:7890".to_string()));

        let json_without_proxy = r#"{
            "auths": {}
        }"#;
        assert_eq!(parse_docker_config_proxy(json_without_proxy), None);

        let invalid_json = "not json";
        assert_eq!(parse_docker_config_proxy(invalid_json), None);
    }

    #[test]
    fn test_check_socks5_dns_leak() {
        assert!(check_socks5_dns_leak("socks5://127.0.0.1:1080"));
        assert!(check_socks5_dns_leak("SOCKS5://127.0.0.1:1080"));
        assert!(!check_socks5_dns_leak("socks5h://127.0.0.1:1080"));
        assert!(!check_socks5_dns_leak("http://127.0.0.1:7890"));
    }

    #[test]
    fn test_parse_gradle_properties_proxy() {
        let raw = r#"
org.gradle.jvmargs=-Xmx2048m
systemProp.http.proxyHost=127.0.0.1
systemProp.http.proxyPort=7890
systemProp.https.proxyHost=127.0.0.1
systemProp.https.proxyPort=7890
"#;
        let cfg = parse_gradle_properties_proxy(raw).unwrap();
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 7890);

        let empty = "org.gradle.jvmargs=-Xmx2048m\n";
        assert_eq!(parse_gradle_properties_proxy(empty), None);
    }

    #[test]
    fn test_parse_wgetrc_proxy() {
        let raw = r#"
# Wget configuration
http_proxy = http://127.0.0.1:7890/
use_proxy = on
"#;
        assert_eq!(parse_wgetrc_proxy(raw), Some("http://127.0.0.1:7890/".to_string()));

        let empty = "verbose = off\n";
        assert_eq!(parse_wgetrc_proxy(empty), None);
    }

    #[test]
    fn test_parse_ssh_config_proxy() {
        let raw = r#"
Host github.com
    User git
    ProxyCommand nc -X 5 -x 127.0.0.1:7890 %h %p

Host *
    ServerAliveInterval 60
"#;
        let list = parse_ssh_config_proxy(raw);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].host_pattern, "github.com");
        assert_eq!(list[0].target, Some("127.0.0.1:7890".to_string()));
    }

    #[test]
    fn test_parse_npmrc_proxy() {
        let content = r#"
registry=https://registry.npmjs.org/
proxy=http://127.0.0.1:7890
https-proxy="http://127.0.0.1:7890"
# comment=true
"#;
        assert_eq!(parse_npmrc_proxy(content, "proxy"), Some("http://127.0.0.1:7890".to_string()));
        assert_eq!(parse_npmrc_proxy(content, "https-proxy"), Some("http://127.0.0.1:7890".to_string()));
        assert_eq!(parse_npmrc_proxy(content, "non_existent"), None);
    }

    #[test]
    fn test_strip_xml_comments() {
        let xml = "<root><!-- comment --><child>hello</child></root>";
        assert_eq!(strip_xml_comments(xml), "<root><child>hello</child></root>");

        let multi = "<settings>\n<!--\n<proxy>\n  <port>8888</port>\n</proxy>\n-->\n<proxy><port>7890</port></proxy></settings>";
        assert!(!strip_xml_comments(multi).contains("8888"));
        assert!(strip_xml_comments(multi).contains("7890"));
    }

    #[test]
    fn test_parse_maven_settings_proxy() {
        let xml = r#"
<settings>
  <proxies>
    <proxy>
      <id>my-proxy</id>
      <active>true</active>
      <protocol>http</protocol>
      <host>127.0.0.1</host>
      <port>7890</port>
      <nonProxyHosts>localhost|127.0.0.1</nonProxyHosts>
    </proxy>
  </proxies>
</settings>
"#;
        let cfg = parse_maven_settings_proxy(xml).unwrap();
        assert_eq!(cfg.id, "my-proxy");
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 7890);
        assert!(cfg.active);

        // Inactive proxy
        let inactive = r#"
<settings>
  <proxies>
    <proxy>
      <id>old</id>
      <active>false</active>
      <host>127.0.0.1</host>
      <port>1080</port>
    </proxy>
  </proxies>
</settings>
"#;
        assert_eq!(parse_maven_settings_proxy(inactive), None);

        // Commented out proxy
        let commented = r#"
<settings>
  <!--
  <proxies>
    <proxy>
      <id>dead</id>
      <active>true</active>
      <host>127.0.0.1</host>
      <port>9999</port>
    </proxy>
  </proxies>
  -->
</settings>
"#;
        assert_eq!(parse_maven_settings_proxy(commented), None);
    }

    #[test]
    fn test_parse_goproxy() {
        let val = "https://goproxy.cn,https://proxy.golang.org,direct";
        let parsed = parse_goproxy(val);
        assert_eq!(parsed, vec!["https://goproxy.cn", "https://proxy.golang.org"]);

        let direct_only = "direct";
        assert!(parse_goproxy(direct_only).is_empty());
    }
}
