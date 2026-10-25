use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
    pub fn new(
        category: &str,
        target: impl Into<String>,
        value: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
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
    let http_port = extract_field("HTTPPort")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);

    let https_host = extract_field("HTTPSProxy").unwrap_or_else(|| "127.0.0.1".to_string());
    let https_port = extract_field("HTTPSPort")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);

    let socks_host = extract_field("SOCKSProxy").unwrap_or_else(|| "127.0.0.1".to_string());
    let socks_port = extract_field("SOCKSPort")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemProxyInfo {
    pub source: String,
    pub state: SystemProxyState,
}

#[allow(dead_code)]
pub fn parse_windows_reg_proxy(enable_out: &str, server_out: &str) -> Option<SystemProxyState> {
    let mut enabled = false;
    for line in enable_out.lines() {
        let trim = line.trim();
        if trim.contains("ProxyEnable") {
            if trim.contains("0x1") || trim.ends_with(" 1") {
                enabled = true;
            }
        }
    }

    let mut server_val = None;
    for line in server_out.lines() {
        let trim = line.trim();
        if trim.contains("ProxyServer") {
            if let Some(pos) = trim.find("REG_SZ") {
                let rest = trim[pos + 6..].trim();
                if !rest.is_empty() {
                    server_val = Some(rest.to_string());
                }
            }
        }
    }

    let server_str = server_val.unwrap_or_default();
    let mut http_host = "127.0.0.1".to_string();
    let mut http_port = 0u16;
    let mut https_host = "127.0.0.1".to_string();
    let mut https_port = 0u16;
    let mut socks_host = "127.0.0.1".to_string();
    let mut socks_port = 0u16;

    let parse_host_port = |val: &str| -> (String, u16) {
        let clean = val
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .trim_start_matches("socks://")
            .trim_start_matches("socks5://");
        if let Some((h, p)) = clean.rsplit_once(':') {
            let host = if h.is_empty() || h == "localhost" {
                "127.0.0.1".to_string()
            } else {
                h.to_string()
            };
            let port = p.parse().unwrap_or(0);
            (host, port)
        } else {
            ("127.0.0.1".to_string(), 0)
        }
    };

    if server_str.contains('=') {
        for part in server_str.split(';') {
            let part = part.trim();
            if let Some((proto, target)) = part.split_once('=') {
                let (h, p) = parse_host_port(target);
                match proto.trim().to_ascii_lowercase().as_str() {
                    "http" => {
                        http_host = h;
                        http_port = p;
                    }
                    "https" => {
                        https_host = h;
                        https_port = p;
                    }
                    "socks" => {
                        socks_host = h;
                        socks_port = p;
                    }
                    _ => {}
                }
            }
        }
    } else if !server_str.is_empty() {
        let (h, p) = parse_host_port(&server_str);
        http_host = h.clone();
        http_port = p;
        https_host = h;
        https_port = p;
    }

    Some(SystemProxyState {
        http_enabled: enabled && http_port > 0,
        http_host,
        http_port,
        https_enabled: enabled && https_port > 0,
        https_host,
        https_port,
        socks_enabled: enabled && socks_port > 0,
        socks_host,
        socks_port,
    })
}

#[allow(dead_code)]
pub fn parse_gnome_proxy(
    mode_out: &str,
    http_host: &str,
    http_port: &str,
    https_host: &str,
    https_port: &str,
    socks_host: &str,
    socks_port: &str,
) -> Option<SystemProxyState> {
    let mode = mode_out.trim().trim_matches('\'').trim_matches('"');
    let is_manual = mode.eq_ignore_ascii_case("manual");

    let clean = |val: &str| val.trim().trim_matches('\'').trim_matches('"').to_string();
    let parse_port = |val: &str| clean(val).parse::<u16>().unwrap_or(0);

    let h_host = clean(http_host);
    let h_port = parse_port(http_port);
    let hs_host = clean(https_host);
    let hs_port = parse_port(https_port);
    let s_host = clean(socks_host);
    let s_port = parse_port(socks_port);

    Some(SystemProxyState {
        http_enabled: is_manual && h_port > 0,
        http_host: if h_host.is_empty() {
            "127.0.0.1".to_string()
        } else {
            h_host
        },
        http_port: h_port,
        https_enabled: is_manual && hs_port > 0,
        https_host: if hs_host.is_empty() {
            "127.0.0.1".to_string()
        } else {
            hs_host
        },
        https_port: hs_port,
        socks_enabled: is_manual && s_port > 0,
        socks_host: if s_host.is_empty() {
            "127.0.0.1".to_string()
        } else {
            s_host
        },
        socks_port: s_port,
    })
}

#[allow(dead_code)]
pub fn get_windows_system_proxy() -> Option<SystemProxyState> {
    #[cfg(target_os = "windows")]
    {
        let reg_key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings";
        let enable_out = Command::new("reg")
            .args(["query", reg_key, "/v", "ProxyEnable"])
            .output()
            .ok()?;
        let server_out = Command::new("reg")
            .args(["query", reg_key, "/v", "ProxyServer"])
            .output()
            .ok()?;

        let enable_s = String::from_utf8_lossy(&enable_out.stdout);
        let server_s = String::from_utf8_lossy(&server_out.stdout);
        return parse_windows_reg_proxy(&enable_s, &server_s);
    }
    #[allow(unreachable_code)]
    None
}

#[allow(dead_code)]
pub fn get_linux_system_proxy() -> Option<SystemProxyState> {
    #[cfg(target_os = "linux")]
    {
        let run_gsettings = |schema: &str, key: &str| -> String {
            Command::new("gsettings")
                .args(["get", schema, key])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default()
        };

        let mode = run_gsettings("org.gnome.system.proxy", "mode");
        if !mode.is_empty() {
            let h_host = run_gsettings("org.gnome.system.proxy.http", "host");
            let h_port = run_gsettings("org.gnome.system.proxy.http", "port");
            let hs_host = run_gsettings("org.gnome.system.proxy.https", "host");
            let hs_port = run_gsettings("org.gnome.system.proxy.https", "port");
            let s_host = run_gsettings("org.gnome.system.proxy.socks", "host");
            let s_port = run_gsettings("org.gnome.system.proxy.socks", "port");

            return parse_gnome_proxy(
                &mode, &h_host, &h_port, &hs_host, &hs_port, &s_host, &s_port,
            );
        }
    }
    #[allow(unreachable_code)]
    None
}

pub fn get_system_proxy() -> Option<SystemProxyInfo> {
    #[cfg(target_os = "macos")]
    {
        if let Some(state) = get_macos_system_proxy() {
            return Some(SystemProxyInfo {
                source: "macOS Web Proxy (scutil)".to_string(),
                state,
            });
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(state) = get_windows_system_proxy() {
            return Some(SystemProxyInfo {
                source: "Windows Internet Settings (Registry)".to_string(),
                state,
            });
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(state) = get_linux_system_proxy() {
            return Some(SystemProxyInfo {
                source: "Linux Desktop Proxy (gsettings)".to_string(),
                state,
            });
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
    if let Ok(out) = Command::new("pip")
        .args(["config", "get", "global.proxy"])
        .output()
    {
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
        if let Some(http) = v
            .pointer("/proxies/default/httpProxy")
            .and_then(|s| s.as_str())
        {
            let trim = http.trim();
            if !trim.is_empty() {
                return Some(trim.to_string());
            }
        }
        if let Some(https) = v
            .pointer("/proxies/default/httpsProxy")
            .and_then(|s| s.as_str())
        {
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

pub fn parse_go_env_content(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            if k.trim() == "GOPROXY" {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

pub fn get_go_env_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        let p = config_dir.join("go").join("env");
        if p.exists() {
            return p;
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".config").join("go").join("env");
        if p.exists() {
            return p;
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("go").join("env");
    }
    PathBuf::from("go/env")
}

pub fn get_goproxy() -> Option<String> {
    if let Ok(val) = std::env::var("GOPROXY") {
        let trim = val.trim();
        if !trim.is_empty() {
            return Some(trim.to_string());
        }
    }
    let go_env_path = get_go_env_path();
    if let Ok(content) = std::fs::read_to_string(&go_env_path) {
        if let Some(val) = parse_go_env_content(&content) {
            return Some(val);
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
            if (key.eq_ignore_ascii_case("http_proxy") || key.eq_ignore_ascii_case("https_proxy"))
                && !val.is_empty()
            {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YarnProxyConfig {
    pub proxy: String,
    pub source: String,
}

pub fn parse_yarnrc_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.is_empty() {
            continue;
        }
        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("proxy") || lower.starts_with("https-proxy") {
            let rest = if let Some(stripped) = lower.strip_prefix("https-proxy") {
                &trim[trim.len() - stripped.len()..]
            } else if let Some(stripped) = lower.strip_prefix("proxy") {
                &trim[trim.len() - stripped.len()..]
            } else {
                continue;
            };
            let rest_trim = rest.trim_start_matches(|c: char| c.is_whitespace() || c == '=');
            let val = rest_trim.trim().trim_matches('"').trim_matches('\'').trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

pub fn parse_yarnrc_yml_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.is_empty() {
            continue;
        }
        if let Some((k, v)) = trim.split_once(':') {
            let k_trim = k.trim();
            if k_trim == "httpProxy" || k_trim == "httpsProxy" {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

pub fn parse_yarnrc_registry(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.is_empty() {
            continue;
        }
        if trim.starts_with("registry ") {
            let val = trim["registry ".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

pub fn parse_yarnrc_yml_registry(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.is_empty() {
            continue;
        }
        if let Some((k, v)) = trim.split_once(':') {
            if k.trim() == "npmRegistryServer" {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

pub fn get_yarn_proxy() -> Option<YarnProxyConfig> {
    if let Some(home) = dirs::home_dir() {
        let yml_path = home.join(".yarnrc.yml");
        if let Ok(content) = std::fs::read_to_string(yml_path) {
            if let Some(proxy) = parse_yarnrc_yml_proxy(&content) {
                return Some(YarnProxyConfig {
                    proxy,
                    source: "~/.yarnrc.yml".to_string(),
                });
            }
        }

        let rc_path = home.join(".yarnrc");
        if let Ok(content) = std::fs::read_to_string(rc_path) {
            if let Some(proxy) = parse_yarnrc_proxy(&content) {
                return Some(YarnProxyConfig {
                    proxy,
                    source: "~/.yarnrc".to_string(),
                });
            }
        }
    }
    None
}

pub fn check_case_mismatch(val1: &str, val2: &str) -> bool {
    let t1 = val1.trim().trim_end_matches('/');
    let t2 = val2.trim().trim_end_matches('/');
    !t1.is_empty() && !t2.is_empty() && t1 != t2
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CondaProxyConfig {
    pub http: Option<String>,
    pub https: Option<String>,
}

pub fn parse_condarc_proxy(content: &str) -> Option<CondaProxyConfig> {
    let mut in_proxy_servers = false;
    let mut http = None;
    let mut https = None;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.is_empty() {
            continue;
        }

        if !line.starts_with(' ') && !line.starts_with('\t') {
            if trim.starts_with("proxy_servers:") {
                in_proxy_servers = true;
                continue;
            } else if in_proxy_servers {
                break;
            }
        }

        if in_proxy_servers {
            if let Some((k, v)) = trim.split_once(':') {
                let key = k.trim().to_ascii_lowercase();
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() {
                    if key == "https" {
                        https = Some(val.to_string());
                    } else if key == "http" {
                        http = Some(val.to_string());
                    }
                }
            }
        }
    }

    if http.is_some() || https.is_some() {
        Some(CondaProxyConfig { http, https })
    } else {
        None
    }
}

pub fn get_condarc_path() -> Option<PathBuf> {
    if let Ok(condarc_env) = std::env::var("CONDARC") {
        let p = PathBuf::from(condarc_env.trim());
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".condarc");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_mambarc_path() -> Option<PathBuf> {
    if let Ok(mambarc_env) = std::env::var("MAMBARC") {
        let p = PathBuf::from(mambarc_env.trim());
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".mambarc");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_mamba_write_path() -> PathBuf {
    if let Some(p) = get_mambarc_path() {
        return p;
    }
    if let Ok(mambarc_env) = std::env::var("MAMBARC") {
        return PathBuf::from(mambarc_env.trim());
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".mambarc");
    }
    PathBuf::from(".mambarc")
}

pub fn get_conda_proxy() -> Option<CondaProxyConfig> {
    if let Some(path) = get_condarc_path() {
        if let Ok(content) = std::fs::read_to_string(path) {
            return parse_condarc_proxy(&content);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostsEntry {
    pub ip: String,
    pub domain: String,
}

pub fn parse_hosts_file(content: &str) -> Vec<HostsEntry> {
    let mut entries = Vec::new();
    for line in content.lines() {
        let mut clean = line;
        if let Some(idx) = clean.find('#') {
            clean = &clean[..idx];
        }
        let parts: Vec<&str> = clean.split_whitespace().collect();
        if parts.len() >= 2 {
            let ip = parts[0];
            if ip.parse::<std::net::IpAddr>().is_ok() {
                for &domain in &parts[1..] {
                    let d = domain.trim().to_ascii_lowercase();
                    if !d.is_empty() {
                        entries.push(HostsEntry {
                            ip: ip.to_string(),
                            domain: d,
                        });
                    }
                }
            }
        }
    }
    entries
}

pub fn get_hosts_file_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(sysroot) = std::env::var("SystemRoot") {
            return PathBuf::from(sysroot)
                .join("System32")
                .join("drivers")
                .join("etc")
                .join("hosts");
        }
        PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
    }
    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/etc/hosts")
    }
}

pub fn get_hosts_entries() -> Vec<HostsEntry> {
    let path = get_hosts_file_path();
    if let Ok(content) = std::fs::read_to_string(path) {
        return parse_hosts_file(&content);
    }
    Vec::new()
}

pub fn parse_bunfig_proxy(content: &str) -> Option<String> {
    let mut current_section = "";
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if trim.starts_with('[') && trim.ends_with(']') {
            current_section = trim.trim_matches(|c| c == '[' || c == ']').trim();
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "proxy" || key == "http_proxy" || key == "https_proxy" {
                if current_section == "install" || current_section.is_empty() {
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

pub fn get_bun_proxy() -> Option<String> {
    if let Some(home) = dirs::home_dir() {
        let paths = [
            home.join(".bunfig.toml"),
            home.join(".bun").join("bunfig.toml"),
        ];
        for path in paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Some(p) = parse_bunfig_proxy(&content) {
                    return Some(p);
                }
            }
        }
    }
    if let Ok(content) = std::fs::read_to_string("bunfig.toml") {
        if let Some(p) = parse_bunfig_proxy(&content) {
            return Some(p);
        }
    }
    None
}

pub fn parse_pnpmrc_proxy(content: &str, key: &str) -> Option<String> {
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

pub fn get_pnpm_proxy() -> Option<String> {
    if let Ok(out) = Command::new("pnpm")
        .args(["config", "get", "proxy"])
        .output()
    {
        if out.status.success() {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !val.is_empty() && val != "null" && val != "undefined" {
                return Some(val);
            }
        }
    }
    if let Ok(out) = Command::new("pnpm")
        .args(["config", "get", "https-proxy"])
        .output()
    {
        if out.status.success() {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !val.is_empty() && val != "null" && val != "undefined" {
                return Some(val);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let pnpmrc = home.join(".pnpmrc");
        if let Ok(content) = std::fs::read_to_string(pnpmrc) {
            if let Some(val) = parse_pnpmrc_proxy(&content, "proxy") {
                return Some(val);
            }
            if let Some(val) = parse_pnpmrc_proxy(&content, "https-proxy") {
                return Some(val);
            }
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoRegistryReplacement {
    pub source: String,
    pub replace_with: String,
    pub registry_url: Option<String>,
}

pub fn parse_cargo_registry_replacements(content: &str) -> Vec<CargoRegistryReplacement> {
    use std::collections::HashMap;

    let mut current_section = String::new();
    let mut replace_with_map: HashMap<String, String> = HashMap::new();
    let mut source_registries: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if trim.starts_with('[') && trim.ends_with(']') {
            current_section = trim
                .trim_matches(|c| c == '[' || c == ']')
                .trim()
                .to_string();
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            let key = k.trim();
            let val = v.trim().trim_matches('"').trim_matches('\'').trim();
            if key == "replace-with" {
                if let Some(src) = current_section.strip_prefix("source.") {
                    replace_with_map.insert(src.to_string(), val.to_string());
                }
            } else if key == "registry" {
                if let Some(src) = current_section.strip_prefix("source.") {
                    source_registries.insert(src.to_string(), val.to_string());
                }
            }
        }
    }

    let mut results = Vec::new();
    for (src, target) in replace_with_map {
        let registry_url = source_registries.get(&target).cloned();
        results.push(CargoRegistryReplacement {
            source: src,
            replace_with: target,
            registry_url,
        });
    }
    results
}

pub fn get_cargo_registry_replacements() -> Vec<CargoRegistryReplacement> {
    if let Some(home) = dirs::home_dir() {
        let paths = [
            home.join(".cargo").join("config.toml"),
            home.join(".cargo").join("config"),
        ];
        for path in paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                let replacements = parse_cargo_registry_replacements(&content);
                if !replacements.is_empty() {
                    return replacements;
                }
            }
        }
    }
    Vec::new()
}

pub fn clean_jsonc(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;
    let mut escape = false;

    while i < len {
        let ch = chars[i];
        if in_string {
            out.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            i += 1;
        } else {
            if ch == '"' {
                in_string = true;
                out.push(ch);
                i += 1;
            } else if ch == '/' && i + 1 < len && chars[i + 1] == '/' {
                i += 2;
                while i < len && chars[i] != '\n' {
                    i += 1;
                }
            } else if ch == '/' && i + 1 < len && chars[i + 1] == '*' {
                i += 2;
                while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                if i + 1 < len {
                    i += 2;
                } else {
                    i = len;
                }
            } else {
                out.push(ch);
                i += 1;
            }
        }
    }

    let out_chars: Vec<char> = out.chars().collect();
    let out_len = out_chars.len();
    let mut final_out = String::with_capacity(out_len);
    let mut in_str = false;
    let mut esc = false;

    for idx in 0..out_len {
        let c = out_chars[idx];
        if in_str {
            final_out.push(c);
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
        } else {
            if c == '"' {
                in_str = true;
                final_out.push(c);
            } else if c == ',' {
                let mut is_trailing = false;
                for lookahead in (idx + 1)..out_len {
                    let next_c = out_chars[lookahead];
                    if next_c.is_whitespace() {
                        continue;
                    }
                    if next_c == '}' || next_c == ']' {
                        is_trailing = true;
                    }
                    break;
                }
                if !is_trailing {
                    final_out.push(c);
                }
            } else {
                final_out.push(c);
            }
        }
    }

    final_out
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdeProxyConfig {
    pub ide_name: String,
    pub path: PathBuf,
    pub proxy: Option<String>,
    pub strict_ssl: Option<bool>,
    pub proxy_support: Option<String>,
}

pub const KNOWN_IDES: &[(&str, &str)] = &[
    ("VS Code", "Code"),
    ("Cursor", "Cursor"),
    ("Windsurf", "Windsurf"),
    ("VSCodium", "VSCodium"),
];

pub fn parse_ide_settings(
    ide_name: &str,
    path: &std::path::Path,
    content: &str,
) -> Option<IdeProxyConfig> {
    let cleaned = clean_jsonc(content);
    let val: serde_json::Value = serde_json::from_str(&cleaned).ok()?;
    let obj = val.as_object()?;

    let proxy = obj
        .get("http.proxy")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let strict_ssl = obj.get("http.proxyStrictSSL").and_then(|v| v.as_bool());

    let proxy_support = obj
        .get("http.proxySupport")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string());

    Some(IdeProxyConfig {
        ide_name: ide_name.to_string(),
        path: path.to_path_buf(),
        proxy,
        strict_ssl,
        proxy_support,
    })
}

pub fn get_ide_settings_paths() -> Vec<(&'static str, PathBuf)> {
    let mut list = Vec::new();
    if let Some(config_dir) = dirs::config_dir() {
        for &(name, folder) in KNOWN_IDES {
            let path = config_dir.join(folder).join("User").join("settings.json");
            list.push((name, path));
        }
    }
    list
}

pub fn get_ide_proxies() -> Vec<IdeProxyConfig> {
    let mut results = Vec::new();
    for (name, path) in get_ide_settings_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Some(info) = parse_ide_settings(name, &path, &content) {
                    results.push(info);
                }
            }
        }
    }
    results
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JetBrainsProxyConfig {
    pub ide_name: String,
    pub path: PathBuf,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub is_socks: bool,
    pub use_proxy: bool,
    pub pac_url: Option<String>,
    pub use_pac: bool,
}

pub fn parse_jetbrains_proxy_xml(
    ide_name: &str,
    path: &std::path::Path,
    content: &str,
) -> Option<JetBrainsProxyConfig> {
    let clean = strip_xml_comments(content);
    let mut use_proxy = false;
    let mut use_pac = false;
    let mut host = None;
    let mut port = None;
    let mut is_socks = false;
    let mut pac_url = None;

    let mut cursor = clean.as_str();
    while let Some(idx) = cursor.find("<option") {
        let rest = &cursor[idx + 7..];
        let end_idx = match rest.find('>') {
            Some(e) => e,
            None => break,
        };
        let tag_content = &rest[..end_idx];
        cursor = &rest[end_idx + 1..];

        let extract_attr = |attr: &str| -> Option<String> {
            let key = format!("{}=\"", attr);
            if let Some(pos) = tag_content.find(&key) {
                let v_rest = &tag_content[pos + key.len()..];
                if let Some(q_end) = v_rest.find('"') {
                    return Some(v_rest[..q_end].to_string());
                }
            }
            None
        };

        if let (Some(name), Some(val)) = (extract_attr("name"), extract_attr("value")) {
            match name.as_str() {
                "USE_HTTP_PROXY" => {
                    use_proxy = val.eq_ignore_ascii_case("true");
                }
                "USE_PROXY_PAC" => {
                    use_pac = val.eq_ignore_ascii_case("true");
                }
                "PROXY_HOST" => {
                    let trim = val.trim();
                    if !trim.is_empty() {
                        host = Some(trim.to_string());
                    }
                }
                "PROXY_PORT" => {
                    if let Ok(p) = val.trim().parse::<u16>() {
                        port = Some(p);
                    }
                }
                "PROXY_TYPE_NUM" => {
                    if val.trim() == "2" {
                        is_socks = true;
                    }
                }
                "PAC_URL" => {
                    let trim = val.trim();
                    if !trim.is_empty() {
                        pac_url = Some(trim.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    if use_proxy || use_pac || host.is_some() || pac_url.is_some() {
        Some(JetBrainsProxyConfig {
            ide_name: ide_name.to_string(),
            path: path.to_path_buf(),
            host,
            port,
            is_socks,
            use_proxy,
            pac_url,
            use_pac,
        })
    } else {
        None
    }
}

pub fn get_jetbrains_search_dirs() -> Vec<PathBuf> {
    let mut dirs_to_check = Vec::new();
    if let Some(config_dir) = dirs::config_dir() {
        dirs_to_check.push(config_dir.join("JetBrains"));
        dirs_to_check.push(config_dir.join("Google"));
    }
    if let Some(home) = dirs::home_dir() {
        let dot_config = home.join(".config");
        dirs_to_check.push(dot_config.join("JetBrains"));
        dirs_to_check.push(dot_config.join("Google"));
        #[cfg(target_os = "macos")]
        {
            let app_sup = home.join("Library").join("Application Support");
            dirs_to_check.push(app_sup.join("JetBrains"));
            dirs_to_check.push(app_sup.join("Google"));
        }
    }
    let mut unique = Vec::new();
    for d in dirs_to_check {
        if !unique.contains(&d) {
            unique.push(d);
        }
    }
    unique
}

pub fn get_jetbrains_proxies() -> Vec<JetBrainsProxyConfig> {
    let mut results = Vec::new();
    for base_dir in get_jetbrains_search_dirs() {
        if let Ok(entries) = std::fs::read_dir(base_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let settings_file = p.join("options").join("proxy.settings.xml");
                    if settings_file.exists()
                        && !results
                            .iter()
                            .any(|r: &JetBrainsProxyConfig| r.path == settings_file)
                    {
                        if let Ok(content) = std::fs::read_to_string(&settings_file) {
                            let ide_name = entry.file_name().to_string_lossy().to_string();
                            if let Some(cfg) =
                                parse_jetbrains_proxy_xml(&ide_name, &settings_file, &content)
                            {
                                results.push(cfg);
                            }
                        }
                    }
                }
            }
        }
    }
    results
}

pub fn parse_gh_config_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trim.split_once(':') {
            if k.trim() == "http_proxy" {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

pub fn get_gh_config_path() -> PathBuf {
    if let Ok(dir) = std::env::var("GH_CONFIG_DIR") {
        let p = PathBuf::from(dir).join("config.yml");
        if p.exists() {
            return p;
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p1 = config_dir.join("gh").join("config.yml");
        if p1.exists() {
            return p1;
        }
        let p2 = config_dir.join("GitHub CLI").join("config.yml");
        if p2.exists() {
            return p2;
        }
        p1
    } else if let Some(home) = dirs::home_dir() {
        home.join(".config").join("gh").join("config.yml")
    } else {
        PathBuf::from(".config/gh/config.yml")
    }
}

pub fn get_gh_proxy() -> Option<String> {
    let path = get_gh_config_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            return parse_gh_config_proxy(&content);
        }
    }
    None
}

pub fn parse_glab_config_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trim.split_once(':') {
            let key = k.trim();
            if key == "glab_proxy" || key == "http_proxy" {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

pub fn get_glab_config_path() -> PathBuf {
    if let Ok(dir) = std::env::var("GLAB_CONFIG_DIR") {
        let p = PathBuf::from(dir).join("config.yml");
        if p.exists() {
            return p;
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p1 = config_dir.join("glab-cli").join("config.yml");
        if p1.exists() {
            return p1;
        }
        let p2 = config_dir.join("glab").join("config.yml");
        if p2.exists() {
            return p2;
        }
        p1
    } else if let Some(home) = dirs::home_dir() {
        home.join(".config").join("glab-cli").join("config.yml")
    } else {
        PathBuf::from(".config/glab-cli/config.yml")
    }
}

pub fn get_glab_proxy() -> Option<String> {
    let path = get_glab_config_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            return parse_glab_config_proxy(&content);
        }
    }
    None
}

pub fn extract_xml_attr(tag_content: &str, attr_name: &str) -> Option<String> {
    let lower = tag_content.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(pos) = lower[search_from..].find(attr_name) {
        let abs_pos = search_from + pos;
        let after_name = &tag_content[abs_pos + attr_name.len()..];
        let trimmed_after = after_name.trim_start();
        if trimmed_after.starts_with('=') {
            let after_eq = trimmed_after[1..].trim_start();
            if let Some(quote) = after_eq.chars().next() {
                if quote == '"' || quote == '\'' {
                    let val_str = &after_eq[quote.len_utf8()..];
                    if let Some(end_q) = val_str.find(quote) {
                        return Some(val_str[..end_q].trim().to_string());
                    }
                }
            }
        }
        search_from = abs_pos + attr_name.len();
    }
    None
}

pub fn parse_nuget_config_proxy(content: &str) -> Option<String> {
    let clean = strip_xml_comments(content);
    let mut cursor = clean.as_str();

    while let Some(idx) = cursor.find("<add") {
        let rest = &cursor[idx + 4..];
        let end_idx = match rest.find('>') {
            Some(e) => e,
            None => break,
        };
        let tag_content = &rest[..end_idx];
        cursor = &rest[end_idx + 1..];

        if let (Some(key), Some(val)) = (
            extract_xml_attr(tag_content, "key"),
            extract_xml_attr(tag_content, "value"),
        ) {
            let k_lower = key.to_ascii_lowercase();
            if (k_lower == "http_proxy" || k_lower == "https_proxy") && !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}

pub fn get_nuget_config_path() -> PathBuf {
    #[cfg(windows)]
    if let Some(appdata) = dirs::config_dir() {
        return appdata.join("NuGet").join("NuGet.Config");
    }
    if let Some(home) = dirs::home_dir() {
        let p1 = home.join(".nuget").join("NuGet").join("NuGet.Config");
        if p1.exists() {
            return p1;
        }
        let p2 = home.join(".config").join("NuGet").join("NuGet.Config");
        if p2.exists() {
            return p2;
        }
        return p1;
    }
    PathBuf::from("NuGet.Config")
}

pub fn get_nuget_proxy() -> Option<(String, PathBuf)> {
    let mut candidates = Vec::new();
    #[cfg(windows)]
    if let Some(appdata) = dirs::config_dir() {
        candidates.push(appdata.join("NuGet").join("NuGet.Config"));
    }
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join(".nuget").join("NuGet").join("NuGet.Config"));
        candidates.push(home.join(".config").join("NuGet").join("NuGet.Config"));
    }
    for p in candidates {
        if p.exists() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                if let Some(proxy) = parse_nuget_config_proxy(&content) {
                    return Some((proxy, p));
                }
            }
        }
    }
    None
}

pub fn parse_gemrc_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        let stripped = if let Some(rest) = trim.strip_prefix(":http_proxy:") {
            Some(rest)
        } else if let Some(rest) = trim.strip_prefix("http_proxy:") {
            Some(rest)
        } else {
            None
        };
        if let Some(rest) = stripped {
            let val = rest.trim().trim_matches('"').trim_matches('\'').trim();
            if !val.is_empty() && val != ":system" && val != "null" && val != "~" {
                return Some(val.to_string());
            }
        }
    }
    None
}

pub fn get_gemrc_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".gemrc")
    } else {
        PathBuf::from(".gemrc")
    }
}

pub fn get_gem_proxy() -> Option<String> {
    let path = get_gemrc_path();
    if let Ok(content) = std::fs::read_to_string(path) {
        return parse_gemrc_proxy(&content);
    }
    None
}

pub fn parse_svn_servers_proxy(content: &str) -> Option<String> {
    let mut in_global = false;
    let mut host: Option<String> = None;
    let mut port: Option<u16> = None;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = &trim[1..trim.len() - 1].trim();
            in_global = section.eq_ignore_ascii_case("global");
            continue;
        }
        if in_global {
            if let Some((k, v)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if key == "http-proxy-host" && !val.is_empty() {
                    host = Some(val.to_string());
                } else if key == "http-proxy-port" && !val.is_empty() {
                    port = val.parse().ok();
                }
            }
        }
    }

    if let Some(h) = host {
        if !h.is_empty() {
            let p = port.unwrap_or(80);
            return Some(format!("http://{}:{}", h, p));
        }
    }
    None
}

pub fn get_svn_servers_path() -> PathBuf {
    #[cfg(windows)]
    if let Some(appdata) = dirs::config_dir() {
        let p = appdata.join("Subversion").join("servers");
        if p.exists() {
            return p;
        }
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".subversion").join("servers");
    }
    PathBuf::from(".subversion/servers")
}

pub fn get_svn_proxy() -> Option<String> {
    let path = get_svn_servers_path();
    if let Ok(content) = std::fs::read_to_string(path) {
        return parse_svn_servers_proxy(&content);
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NpmRegistryConfig {
    pub source: String,
    pub path: Option<PathBuf>,
    pub url: String,
}

pub fn get_npm_registries() -> Vec<NpmRegistryConfig> {
    let mut list = Vec::new();

    if let Ok(val) = std::env::var("NPM_CONFIG_REGISTRY") {
        let trim = val.trim();
        if !trim.is_empty() {
            list.push(NpmRegistryConfig {
                source: "NPM_CONFIG_REGISTRY env".to_string(),
                path: None,
                url: trim.to_string(),
            });
        }
    }

    if let Some(home) = dirs::home_dir() {
        let npmrc = home.join(".npmrc");
        if let Ok(content) = std::fs::read_to_string(&npmrc) {
            if let Some(url) = parse_npmrc_proxy(&content, "registry") {
                list.push(NpmRegistryConfig {
                    source: "~/.npmrc registry".to_string(),
                    path: Some(npmrc),
                    url,
                });
            }
        }

        let pnpmrc = home.join(".pnpmrc");
        if let Ok(content) = std::fs::read_to_string(&pnpmrc) {
            if let Some(url) = parse_pnpmrc_proxy(&content, "registry") {
                list.push(NpmRegistryConfig {
                    source: "~/.pnpmrc registry".to_string(),
                    path: Some(pnpmrc),
                    url,
                });
            }
        }

        let yarnrc = home.join(".yarnrc");
        if let Ok(content) = std::fs::read_to_string(&yarnrc) {
            if let Some(url) = parse_yarnrc_registry(&content) {
                list.push(NpmRegistryConfig {
                    source: "~/.yarnrc registry".to_string(),
                    path: Some(yarnrc),
                    url,
                });
            }
        }

        let yarnrc_yml = home.join(".yarnrc.yml");
        if let Ok(content) = std::fs::read_to_string(&yarnrc_yml) {
            if let Some(url) = parse_yarnrc_yml_registry(&content) {
                list.push(NpmRegistryConfig {
                    source: "~/.yarnrc.yml registry".to_string(),
                    path: Some(yarnrc_yml),
                    url,
                });
            }
        }
    }

    list
}

pub fn parse_hgrc_proxy(content: &str) -> Option<String> {
    let mut in_http_proxy = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = &trim[1..trim.len() - 1].trim();
            in_http_proxy = section.eq_ignore_ascii_case("http_proxy");
            continue;
        }
        if in_http_proxy {
            if let Some((k, v)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if key == "host" && !val.is_empty() {
                    if val.starts_with("http://")
                        || val.starts_with("https://")
                        || val.starts_with("socks5://")
                    {
                        return Some(val.to_string());
                    } else {
                        return Some(format!("http://{}", val));
                    }
                }
            }
        }
    }
    None
}

pub fn get_hgrc_path() -> PathBuf {
    #[cfg(windows)]
    if let Some(home) = dirs::home_dir() {
        let ini = home.join("Mercurial.ini");
        if ini.exists() {
            return ini;
        }
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".hgrc");
    }
    PathBuf::from(".hgrc")
}

pub fn get_hg_proxy() -> Option<String> {
    let path = get_hgrc_path();
    if let Ok(content) = std::fs::read_to_string(path) {
        return parse_hgrc_proxy(&content);
    }
    None
}

pub fn parse_uv_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http-proxy" || key == "http_proxy" || key == "proxy" {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

pub fn get_uv_config_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        let xdg_p = home.join(".config").join("uv").join("uv.toml");
        if xdg_p.exists() {
            return xdg_p;
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p = config_dir.join("uv").join("uv.toml");
        if p.exists() {
            return p;
        }
    }
    if let Some(home) = dirs::home_dir() {
        let dot_uv = home.join(".uv").join("uv.toml");
        if dot_uv.exists() {
            return dot_uv;
        }
        return home.join(".config").join("uv").join("uv.toml");
    }
    PathBuf::from("uv.toml")
}

pub fn get_uv_proxy() -> Option<String> {
    let path = get_uv_config_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Some(p) = parse_uv_proxy(&content) {
            return Some(p);
        }
    }
    if let Ok(content) = std::fs::read_to_string("uv.toml") {
        if let Some(p) = parse_uv_proxy(&content) {
            return Some(p);
        }
    }
    None
}

pub fn parse_composer_packagist_repo(content: &str) -> Option<String> {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(url) = val
            .get("repositories")
            .and_then(|r| r.get("packagist"))
            .and_then(|p| p.get("url"))
            .and_then(|u| u.as_str())
        {
            let trim = url.trim();
            if !trim.is_empty() {
                return Some(trim.to_string());
            }
        }
    }
    None
}

pub fn get_composer_config_path() -> Option<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        let p1 = home.join(".composer").join("config.json");
        if p1.exists() {
            return Some(p1);
        }
        let p2 = home.join(".config").join("composer").join("config.json");
        if p2.exists() {
            return Some(p2);
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p3 = config_dir.join("composer").join("config.json");
        if p3.exists() {
            return Some(p3);
        }
    }
    None
}

pub fn parse_composer_proxy(content: &str) -> Option<String> {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(cfg) = val.get("config") {
            if let Some(p) = cfg.get("https-proxy").and_then(|v| v.as_str()) {
                let trim = p.trim();
                if !trim.is_empty() {
                    return Some(trim.to_string());
                }
            }
            if let Some(p) = cfg.get("http-proxy").and_then(|v| v.as_str()) {
                let trim = p.trim();
                if !trim.is_empty() {
                    return Some(trim.to_string());
                }
            }
        }
    }
    None
}

pub fn get_composer_proxy() -> Option<(String, PathBuf)> {
    if let Some(path) = get_composer_config_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Some(proxy) = parse_composer_proxy(&content) {
                return Some((proxy, path));
            }
        }
    }
    None
}

pub fn get_composer_config_write_path() -> PathBuf {
    if let Some(p) = get_composer_config_path() {
        return p;
    }
    if let Some(home) = dirs::home_dir() {
        let p1 = home.join(".composer");
        if p1.is_dir() {
            return p1.join("config.json");
        }
        let p2 = home.join(".config").join("composer");
        if p2.is_dir() {
            return p2.join("config.json");
        }
        return home.join(".composer").join("config.json");
    }
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("composer").join("config.json");
    }
    PathBuf::from("config.json")
}

pub fn parse_bazelrc_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if let Some(pos) = trim.find("--action_env=") {
            let part = &trim[pos + "--action_env=".len()..];
            if let Some((k, v)) = part.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "http_proxy" || key == "https_proxy" || key == "all_proxy" {
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

pub fn get_bazelrc_path() -> Option<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".bazelrc");
        if p.exists() {
            return Some(p);
        }
    }
    let local = PathBuf::from(".bazelrc");
    if local.exists() {
        return Some(local);
    }
    None
}

pub fn get_bazel_proxy() -> Option<(String, PathBuf)> {
    if let Some(path) = get_bazelrc_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Some(proxy) = parse_bazelrc_proxy(&content) {
                return Some((proxy, path));
            }
        }
    }
    None
}

pub fn get_bazelrc_write_path() -> PathBuf {
    if let Some(p) = get_bazelrc_path() {
        return p;
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".bazelrc");
    }
    PathBuf::from(".bazelrc")
}

pub fn parse_conan2_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.is_empty() {
            continue;
        }
        if trim.starts_with("core.net.http:proxies") {
            if let Some((_, rest)) = trim.split_once('=') {
                let rest = rest.trim();
                for key in ["https", "http"] {
                    for quote in ['"', '\''] {
                        let search = format!("{}{}{}", quote, key, quote);
                        if let Some(idx) = rest.find(&search) {
                            let after = &rest[idx + search.len()..];
                            if let Some(colon_idx) = after.find(':') {
                                let val_part = after[colon_idx + 1..].trim();
                                if let Some(first_char) = val_part.chars().next() {
                                    if first_char == '"' || first_char == '\'' {
                                        let inside = &val_part[1..];
                                        if let Some(end_idx) = inside.find(first_char) {
                                            let proxy = inside[..end_idx].trim();
                                            if !proxy.is_empty() {
                                                return Some(proxy.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn parse_conan1_proxy(content: &str) -> Option<String> {
    let mut in_proxies_section = false;
    let mut http_proxy = None;
    let mut https_proxy = None;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') || trim.is_empty() {
            continue;
        }
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim();
            in_proxies_section = section.eq_ignore_ascii_case("proxies");
            continue;
        }
        if in_proxies_section {
            if let Some((k, v)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if key == "https" && !val.is_empty() {
                    https_proxy = Some(val.to_string());
                } else if key == "http" && !val.is_empty() {
                    http_proxy = Some(val.to_string());
                }
            }
        }
    }
    https_proxy.or(http_proxy)
}

pub fn get_conan2_config_path() -> Option<PathBuf> {
    if let Ok(val) = std::env::var("CONAN_HOME") {
        let p = PathBuf::from(val.trim()).join("global.conf");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".conan2").join("global.conf");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_conan1_config_path() -> Option<PathBuf> {
    if let Ok(val) = std::env::var("CONAN_USER_HOME") {
        let p = PathBuf::from(val.trim()).join(".conan").join("conan.conf");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".conan").join("conan.conf");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_conan_proxy() -> Option<(String, PathBuf)> {
    if let Some(p) = get_conan2_config_path() {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(proxy) = parse_conan2_proxy(&content) {
                return Some((proxy, p));
            }
        }
    }
    if let Some(p) = get_conan1_config_path() {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(proxy) = parse_conan1_proxy(&content) {
                return Some((proxy, p));
            }
        }
    }
    None
}

pub fn get_conan_write_path() -> PathBuf {
    if let Some(p) = get_conan2_config_path() {
        return p;
    }
    if let Some(p) = get_conan1_config_path() {
        return p;
    }
    if let Ok(val) = std::env::var("CONAN_HOME") {
        return PathBuf::from(val.trim()).join("global.conf");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".conan2").join("global.conf");
    }
    PathBuf::from("global.conf")
}

pub fn parse_kubeconfig_proxies(content: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut in_clusters = false;
    let mut current_name = "default".to_string();
    let mut current_proxy = None;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.is_empty() {
            continue;
        }

        if !line.starts_with(' ') && !line.starts_with('\t') && !line.starts_with('-') {
            if trim.starts_with("clusters:") {
                in_clusters = true;
            } else if in_clusters {
                if let Some(p) = current_proxy.take() {
                    results.push((current_name.clone(), p));
                }
                in_clusters = false;
            }
            continue;
        }

        if in_clusters {
            if trim.starts_with("- cluster") || trim.starts_with("- name:") {
                if let Some(p) = current_proxy.take() {
                    results.push((current_name.clone(), p));
                }
                current_name = "unnamed".to_string();
                if trim.starts_with("- name:") {
                    let name = trim["- name:".len()..]
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim();
                    if !name.is_empty() {
                        current_name = name.to_string();
                    }
                }
            } else if trim.starts_with("name:") {
                let name = trim["name:".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim();
                if !name.is_empty() {
                    current_name = name.to_string();
                }
            } else if trim.starts_with("proxy-url:") {
                let purl = trim["proxy-url:".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim();
                if !purl.is_empty() {
                    current_proxy = Some(purl.to_string());
                }
            }
        }
    }

    if in_clusters {
        if let Some(p) = current_proxy {
            results.push((current_name, p));
        }
    }

    results
}

pub fn get_kubeconfig_path() -> Option<PathBuf> {
    if let Ok(val) = std::env::var("KUBECONFIG") {
        for part in val.split(':') {
            let p = PathBuf::from(part.trim());
            if p.exists() {
                return Some(p);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".kube").join("config");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_kubeconfig_write_path() -> PathBuf {
    if let Ok(val) = std::env::var("KUBECONFIG") {
        if let Some(first) = val.split(':').next() {
            let p = PathBuf::from(first.trim());
            if !first.trim().is_empty() {
                return p;
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".kube").join("config");
    }
    PathBuf::from("config")
}

pub fn parse_sbtopts_proxy(content: &str) -> Option<String> {
    let mut host = None;
    let mut port = None;
    let mut scheme = "http";

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        for token in trim.split_whitespace() {
            if let Some(rest) = token.strip_prefix("-Dhttp.proxyHost=") {
                let h = rest.trim_matches('"').trim_matches('\'').trim();
                if !h.is_empty() {
                    host = Some(h.to_string());
                }
            } else if let Some(rest) = token.strip_prefix("-Dhttps.proxyHost=") {
                let h = rest.trim_matches('"').trim_matches('\'').trim();
                if !h.is_empty() && host.is_none() {
                    host = Some(h.to_string());
                }
            } else if let Some(rest) = token.strip_prefix("-DsocksProxyHost=") {
                let h = rest.trim_matches('"').trim_matches('\'').trim();
                if !h.is_empty() {
                    host = Some(h.to_string());
                    scheme = "socks5";
                }
            } else if let Some(rest) = token.strip_prefix("-Dhttp.proxyPort=") {
                let p = rest.trim_matches('"').trim_matches('\'').trim();
                if !p.is_empty() {
                    port = Some(p.to_string());
                }
            } else if let Some(rest) = token.strip_prefix("-Dhttps.proxyPort=") {
                let p = rest.trim_matches('"').trim_matches('\'').trim();
                if !p.is_empty() && port.is_none() {
                    port = Some(p.to_string());
                }
            } else if let Some(rest) = token.strip_prefix("-DsocksProxyPort=") {
                let p = rest.trim_matches('"').trim_matches('\'').trim();
                if !p.is_empty() {
                    port = Some(p.to_string());
                }
            }
        }
    }

    if let Some(h) = host {
        if let Some(p) = port {
            return Some(format!("{}://{}:{}", scheme, h, p));
        } else {
            return Some(format!("{}://{}", scheme, h));
        }
    }
    None
}

pub fn get_sbtopts_path() -> Option<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".sbt").join("sbtopts");
        if p.exists() {
            return Some(p);
        }
    }
    let local = PathBuf::from(".sbtopts");
    if local.exists() {
        return Some(local);
    }
    None
}

pub fn get_sbt_proxy() -> Option<(String, PathBuf)> {
    if let Some(path) = get_sbtopts_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Some(proxy) = parse_sbtopts_proxy(&content) {
                return Some((proxy, path));
            }
        }
    }
    None
}

pub fn get_sbtopts_write_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        return home.join(".sbt").join("sbtopts");
    }
    PathBuf::from(".sbtopts")
}

pub fn parse_julia_startup_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        let upper = trim.to_ascii_uppercase();
        if upper.starts_with("ENV[") {
            if upper.contains("HTTP_PROXY") || upper.contains("HTTPS_PROXY") || upper.contains("ALL_PROXY") {
                if let Some((_, right)) = trim.split_once('=') {
                    let val = right.trim().trim_matches(';').trim().trim_matches('"').trim_matches('\'').trim();
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }
    None
}

pub fn get_julia_startup_path() -> Option<PathBuf> {
    if let Ok(depot) = std::env::var("JULIA_DEPOT_PATH") {
        if let Some(first) = depot.split(':').next() {
            let p = PathBuf::from(first.trim()).join("config").join("startup.jl");
            if p.exists() {
                return Some(p);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".julia").join("config").join("startup.jl");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_julia_startup_proxy() -> Option<(String, PathBuf)> {
    if let Some(path) = get_julia_startup_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Some(proxy) = parse_julia_startup_proxy(&content) {
                return Some((proxy, path));
            }
        }
    }
    None
}

pub fn get_julia_startup_write_path() -> PathBuf {
    if let Ok(depot) = std::env::var("JULIA_DEPOT_PATH") {
        if let Some(first) = depot.split(':').next() {
            if !first.trim().is_empty() {
                return PathBuf::from(first.trim()).join("config").join("startup.jl");
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".julia").join("config").join("startup.jl");
    }
    PathBuf::from("startup.jl")
}

pub fn parse_nix_substituters(content: &str) -> Vec<String> {
    let mut substituters = Vec::new();
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            let key = k.trim();
            if key == "substituters" || key == "extra-substituters" || key == "trusted-substituters" {
                for token in v.split_whitespace() {
                    let t = token.trim_matches('"').trim_matches('\'').trim();
                    if t.starts_with("http://") || t.starts_with("https://") {
                        if !substituters.contains(&t.to_string()) {
                            substituters.push(t.to_string());
                        }
                    }
                }
            }
        }
    }
    substituters
}

pub fn get_nix_config_path() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("NIX_CONFIG_DIR") {
        let p = PathBuf::from(dir).join("nix.conf");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p = config_dir.join("nix").join("nix.conf");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".config").join("nix").join("nix.conf");
        if p.exists() {
            return Some(p);
        }
    }
    let system_nix = PathBuf::from("/etc/nix/nix.conf");
    if system_nix.exists() {
        return Some(system_nix);
    }
    None
}

pub fn parse_nix_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            if k.trim() == "http-proxy" {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

pub fn get_nix_write_path() -> PathBuf {
    if let Some(p) = get_nix_config_path() {
        if p.starts_with("/etc") {
            if let Some(home) = dirs::home_dir() {
                return home.join(".config").join("nix").join("nix.conf");
            }
        }
        return p;
    }
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("nix").join("nix.conf");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".config").join("nix").join("nix.conf");
    }
    PathBuf::from("nix.conf")
}

pub fn get_nix_proxy() -> Option<(String, PathBuf)> {
    let p = get_nix_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_nix_proxy(&content)?;
    Some((proxy, p))
}

pub fn parse_cabal_config_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with("--") || trim.starts_with('#') {
            continue;
        }
        if let Some(rest) = trim.strip_prefix("proxy:") {
            let val = rest.trim().trim_matches('"').trim_matches('\'').trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

pub fn get_cabal_config_path() -> Option<PathBuf> {
    if let Ok(cabal_dir) = std::env::var("CABAL_DIR") {
        let p = PathBuf::from(cabal_dir).join("config");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".cabal").join("config");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p = config_dir.join("cabal").join("config");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_cabal_write_path() -> PathBuf {
    if let Some(p) = get_cabal_config_path() {
        return p;
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".cabal").join("config");
    }
    PathBuf::from("config")
}

pub fn get_cabal_proxy() -> Option<(String, PathBuf)> {
    let p = get_cabal_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_cabal_config_proxy(&content)?;
    Some((proxy, p))
}

pub fn get_swiftpm_mirrors_path() -> Option<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".swiftpm").join("configuration").join("mirrors.json");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn parse_swiftpm_mirrors(content: &str) -> Vec<String> {
    let mut urls = Vec::new();
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(mirrors_map) = val.get("mirrors").and_then(|m| m.as_object()) {
            for (_orig, mirror_dest) in mirrors_map {
                if let Some(s) = mirror_dest.as_str() {
                    urls.push(s.to_string());
                }
            }
        } else if let Some(object_list) = val.get("object").and_then(|o| o.as_array()) {
            for item in object_list {
                if let Some(mirror_url) = item.get("mirror").and_then(|m| m.as_str()) {
                    urls.push(mirror_url.to_string());
                }
            }
        }
    }
    urls
}

pub fn clean_swiftpm_mirrors(content: &str, dead_mirrors: &[&str]) -> (String, bool) {
    if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(content) {
        let mut changed = false;
        if let Some(mirrors_map) = val.get_mut("mirrors").and_then(|m| m.as_object_mut()) {
            let initial_len = mirrors_map.len();
            mirrors_map.retain(|_orig, dest| {
                if let Some(s) = dest.as_str() {
                    !dead_mirrors.iter().any(|d| *d == s)
                } else {
                    true
                }
            });
            if mirrors_map.len() != initial_len {
                changed = true;
            }
        } else if let Some(object_list) = val.get_mut("object").and_then(|o| o.as_array_mut()) {
            let initial_len = object_list.len();
            object_list.retain(|item| {
                if let Some(mirror_url) = item.get("mirror").and_then(|m| m.as_str()) {
                    !dead_mirrors.iter().any(|d| *d == mirror_url)
                } else {
                    true
                }
            });
            if object_list.len() != initial_len {
                changed = true;
            }
        }
        if changed {
            if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                return (pretty + "\n", true);
            }
        }
    }
    (content.to_string(), false)
}

pub fn parse_stack_config_proxy(content: &str) -> Option<String> {
    let mut in_proxy_block = false;
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with("//") {
            continue;
        }
        if trim.starts_with("proxy:") {
            let after = trim.strip_prefix("proxy:").unwrap_or("").trim();
            if after.starts_with('{') {
                for part in after.trim_matches('{').trim_matches('}').split(',') {
                    let ptrim = part.trim();
                    if ptrim.starts_with("http:") || ptrim.starts_with("\"http\":") || ptrim.starts_with("'http':") {
                        if let Some((_, v)) = ptrim.split_once(':') {
                            let url = v.trim().trim_matches('"').trim_matches('\'').trim();
                            if !url.is_empty() {
                                return Some(url.to_string());
                            }
                        }
                    } else if ptrim.starts_with("https:") || ptrim.starts_with("\"https\":") || ptrim.starts_with("'https':") {
                        if let Some((_, v)) = ptrim.split_once(':') {
                            let url = v.trim().trim_matches('"').trim_matches('\'').trim();
                            if !url.is_empty() {
                                return Some(url.to_string());
                            }
                        }
                    }
                }
            } else if !after.is_empty() && (after.starts_with("http://") || after.starts_with("https://") || after.starts_with("socks")) {
                return Some(after.trim_matches('"').trim_matches('\'').to_string());
            } else {
                in_proxy_block = true;
            }
            continue;
        }
        if in_proxy_block {
            if !line.starts_with(' ') && !line.starts_with('\t') && !trim.is_empty() {
                in_proxy_block = false;
            } else if trim.starts_with("http:") || trim.starts_with("https:") {
                let rest = if let Some(r) = trim.strip_prefix("http:") {
                    r
                } else if let Some(r) = trim.strip_prefix("https:") {
                    r
                } else {
                    ""
                };
                let val = rest.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

pub fn get_stack_config_path() -> Option<PathBuf> {
    if let Ok(stack_root) = std::env::var("STACK_ROOT") {
        let p = PathBuf::from(stack_root).join("config.yaml");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".stack").join("config.yaml");
        if p.exists() {
            return Some(p);
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let p1 = PathBuf::from(&appdata).join("stack").join("config.yaml");
            if p1.exists() {
                return Some(p1);
            }
            let p2 = PathBuf::from(&appdata).join("local").join("bin").join("config.yaml");
            if p2.exists() {
                return Some(p2);
            }
        }
    }
    None
}

pub fn get_stack_write_path() -> PathBuf {
    if let Some(p) = get_stack_config_path() {
        return p;
    }
    if let Ok(stack_root) = std::env::var("STACK_ROOT") {
        return PathBuf::from(stack_root).join("config.yaml");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".stack").join("config.yaml");
    }
    PathBuf::from("config.yaml")
}

pub fn get_stack_proxy() -> Option<(String, PathBuf)> {
    let p = get_stack_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_stack_config_proxy(&content)?;
    Some((proxy, p))
}

pub fn parse_hex_config_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('%') || trim.starts_with('#') {
            continue;
        }
        if trim.contains("http_proxy,") || trim.contains("https_proxy,") {
            if let Some((_, rest)) = trim.split_once(',') {
                let clean = rest
                    .trim()
                    .trim_end_matches('.')
                    .trim_end_matches('}')
                    .trim()
                    .trim_start_matches("<<")
                    .trim_end_matches(">>")
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim();
                if !clean.is_empty() && (clean.starts_with("http://") || clean.starts_with("https://") || clean.starts_with("socks")) {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

pub fn get_hex_config_path() -> Option<PathBuf> {
    if let Ok(hex_home) = std::env::var("HEX_HOME") {
        let p = PathBuf::from(hex_home).join("hex.config");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".hex").join("hex.config");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_hex_write_path() -> PathBuf {
    if let Some(p) = get_hex_config_path() {
        return p;
    }
    if let Ok(hex_home) = std::env::var("HEX_HOME") {
        return PathBuf::from(hex_home).join("hex.config");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".hex").join("hex.config");
    }
    PathBuf::from("hex.config")
}

pub fn get_hex_proxy() -> Option<(String, PathBuf)> {
    if let Ok(p) = std::env::var("HEX_HTTP_PROXY").or_else(|_| std::env::var("HEX_HTTPS_PROXY")) {
        let trim = p.trim().to_string();
        if !trim.is_empty() {
            return Some((trim, PathBuf::from("ENV:HEX_HTTP_PROXY")));
        }
    }
    let p = get_hex_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_hex_config_proxy(&content)?;
    Some((proxy, p))
}

pub fn parse_renviron_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            let key = k.trim().to_lowercase();
            if key == "http_proxy" || key == "https_proxy" || key == "all_proxy" {
                let clean = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !clean.is_empty()
                    && (clean.starts_with("http://")
                        || clean.starts_with("https://")
                        || clean.starts_with("socks"))
                {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

pub fn get_renviron_config_path() -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("R_ENVIRON_USER") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".Renviron");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_renviron_write_path() -> PathBuf {
    if let Some(p) = get_renviron_config_path() {
        return p;
    }
    if let Ok(env_path) = std::env::var("R_ENVIRON_USER") {
        return PathBuf::from(env_path);
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".Renviron");
    }
    PathBuf::from(".Renviron")
}

pub fn get_renviron_proxy() -> Option<(String, PathBuf)> {
    let p = get_renviron_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_renviron_proxy(&content)?;
    Some((proxy, p))
}

pub fn parse_opam_config_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trim.split_once(':') {
            let key = k.trim().to_lowercase();
            if key == "http-proxy" || key == "https-proxy" || key == "http_proxy" || key == "https_proxy" {
                let clean = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !clean.is_empty()
                    && (clean.starts_with("http://")
                        || clean.starts_with("https://")
                        || clean.starts_with("socks"))
                {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

pub fn get_opam_config_path() -> Option<PathBuf> {
    if let Ok(opamroot) = std::env::var("OPAMROOT") {
        let p = PathBuf::from(opamroot).join("config");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".opam").join("config");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_opam_write_path() -> PathBuf {
    if let Some(p) = get_opam_config_path() {
        return p;
    }
    if let Ok(opamroot) = std::env::var("OPAMROOT") {
        return PathBuf::from(opamroot).join("config");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".opam").join("config");
    }
    PathBuf::from("config")
}

pub fn get_opam_proxy() -> Option<(String, PathBuf)> {
    let p = get_opam_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_opam_config_proxy(&content)?;
    Some((proxy, p))
}

pub fn parse_terraform_network_mirrors(content: &str) -> Vec<String> {
    let mut mirrors = Vec::new();
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with("//") {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            if k.trim() == "url" {
                let clean = v.trim().trim_matches('"').trim_matches('\'').trim();
                if clean.starts_with("http://") || clean.starts_with("https://") {
                    mirrors.push(clean.to_string());
                }
            }
        }
    }
    mirrors
}

pub fn clean_terraform_network_mirrors(content: &str, dead_mirrors: &[&str]) -> (String, bool) {
    if dead_mirrors.is_empty() {
        return (content.to_string(), false);
    }
    let mut out_lines = Vec::new();
    let mut changed = false;
    let mut in_mirror_block = false;
    let mut block_lines: Vec<String> = Vec::new();
    let mut block_has_dead = false;

    for line in content.lines() {
        let trim = line.trim();
        if !in_mirror_block && (trim.starts_with("network_mirror") || trim.contains("network_mirror {")) {
            in_mirror_block = true;
            block_lines.clear();
            block_has_dead = false;
            block_lines.push(line.to_string());
            continue;
        }

        if in_mirror_block {
            block_lines.push(line.to_string());
            for dead in dead_mirrors {
                if trim.contains(dead) {
                    block_has_dead = true;
                }
            }
            if trim == "}" || trim.ends_with('}') {
                in_mirror_block = false;
                if block_has_dead {
                    out_lines.push("  # [proxydoc disabled dead mirror(s)]".to_string());
                    for bl in &block_lines {
                        out_lines.push(format!("  # {}", bl));
                    }
                    changed = true;
                } else {
                    out_lines.extend(block_lines.drain(..));
                }
            }
            continue;
        }

        let mut single_dead = false;
        for dead in dead_mirrors {
            if trim.contains(dead) && (trim.starts_with("url") || trim.contains("url =") || trim.contains("url=")) {
                single_dead = true;
                break;
            }
        }
        if single_dead {
            out_lines.push(format!("# [proxydoc disabled dead mirror] # {}", line));
            changed = true;
        } else {
            out_lines.push(line.to_string());
        }
    }

    if in_mirror_block {
        if block_has_dead {
            out_lines.push("  # [proxydoc disabled dead mirror(s)]".to_string());
            for bl in &block_lines {
                out_lines.push(format!("  # {}", bl));
            }
            changed = true;
        } else {
            out_lines.extend(block_lines);
        }
    }

    let mut res = out_lines.join("\n");
    if content.ends_with('\n') {
        res.push('\n');
    }
    (res, changed)
}

pub fn parse_lein_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with(';') {
            continue;
        }
        if let Some(pos) = trim.find(":http-proxy") {
            let rest = trim[pos + 11..].trim();
            let clean = rest
                .trim_start_matches(|c| c == ' ' || c == '\t' || c == ':')
                .trim_matches('"')
                .trim_matches('\'')
                .trim();
            let val = clean
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches('"')
                .trim_matches('\'');
            if val.starts_with("http://") || val.starts_with("https://") || val.starts_with("socks") {
                return Some(val.to_string());
            }
        }
        if let Some(pos) = trim.find(":https-proxy") {
            let rest = trim[pos + 12..].trim();
            let clean = rest
                .trim_start_matches(|c| c == ' ' || c == '\t' || c == ':')
                .trim_matches('"')
                .trim_matches('\'')
                .trim();
            let val = clean
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches('"')
                .trim_matches('\'');
            if val.starts_with("http://") || val.starts_with("https://") || val.starts_with("socks") {
                return Some(val.to_string());
            }
        }
    }

    let mut host = None;
    let mut port = None;
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with(';') {
            continue;
        }
        if let Some(idx) = trim.find("-Dhttp.proxyHost=") {
            let h = &trim[idx + 17..];
            let end = h
                .find(|c| c == '"' || c == '\'' || c == ' ' || c == ']')
                .unwrap_or(h.len());
            host = Some(h[..end].to_string());
        }
        if let Some(idx) = trim.find("-Dhttp.proxyPort=") {
            let p = &trim[idx + 17..];
            let end = p
                .find(|c| c == '"' || c == '\'' || c == ' ' || c == ']')
                .unwrap_or(p.len());
            port = Some(p[..end].to_string());
        }
    }
    if let (Some(h), Some(p)) = (host, port) {
        if !h.is_empty() && !p.is_empty() {
            return Some(format!("http://{}:{}", h, p));
        }
    }
    None
}

pub fn get_lein_profiles_path() -> Option<PathBuf> {
    if let Ok(lein_home) = std::env::var("LEIN_HOME") {
        let p = PathBuf::from(lein_home).join("profiles.clj");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".lein").join("profiles.clj");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_lein_write_path() -> PathBuf {
    if let Some(p) = get_lein_profiles_path() {
        return p;
    }
    if let Ok(lein_home) = std::env::var("LEIN_HOME") {
        return PathBuf::from(lein_home).join("profiles.clj");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".lein").join("profiles.clj");
    }
    PathBuf::from("profiles.clj")
}

pub fn get_lein_proxy() -> Option<(String, PathBuf)> {
    let p = get_lein_profiles_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_lein_proxy(&content)?;
    Some((proxy, p))
}

pub fn parse_cpan_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if trim.contains("'http_proxy'")
            || trim.contains("\"http_proxy\"")
            || trim.contains("http_proxy =>")
        {
            if let Some((_, v)) = trim.split_once("=>") {
                let clean = v
                    .trim()
                    .trim_matches(',')
                    .trim_matches(';')
                    .trim();
                let extracted = if clean.starts_with("q[") && clean.ends_with(']') {
                    &clean[2..clean.len() - 1]
                } else if clean.starts_with("q(") && clean.ends_with(')') {
                    &clean[2..clean.len() - 1]
                } else {
                    clean.trim_matches('\'').trim_matches('"')
                }
                .trim();

                if !extracted.is_empty()
                    && (extracted.starts_with("http://")
                        || extracted.starts_with("https://")
                        || extracted.starts_with("socks"))
                {
                    return Some(extracted.to_string());
                }
            }
        }
    }
    None
}

pub fn get_cpan_config_path() -> Option<PathBuf> {
    if let Ok(cpan_home) = std::env::var("PERL_CPAN_DIR") {
        let p = PathBuf::from(cpan_home).join("CPAN").join("MyConfig.pm");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".cpan").join("CPAN").join("MyConfig.pm");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_cpan_write_path() -> PathBuf {
    if let Some(p) = get_cpan_config_path() {
        return p;
    }
    if let Ok(cpan_home) = std::env::var("PERL_CPAN_DIR") {
        return PathBuf::from(cpan_home).join("CPAN").join("MyConfig.pm");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".cpan").join("CPAN").join("MyConfig.pm");
    }
    PathBuf::from("MyConfig.pm")
}

pub fn get_cpan_proxy() -> Option<(String, PathBuf)> {
    let p = get_cpan_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_cpan_proxy(&content)?;
    Some((proxy, p))
}

pub fn parse_luarocks_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with("--") {
            continue;
        }
        let lower = trim.to_lowercase();
        if lower.contains("http_proxy") || lower.contains("https_proxy") {
            if let Some((_, v)) = trim.split_once('=') {
                let clean = v.trim().trim_matches(',').trim_matches(';').trim();
                let clean = clean.trim_matches('"').trim_matches('\'').trim();
                if !clean.is_empty()
                    && (clean.starts_with("http://")
                        || clean.starts_with("https://")
                        || clean.starts_with("socks"))
                {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

pub fn get_luarocks_config_path() -> Option<PathBuf> {
    if let Ok(path_str) = std::env::var("LUAROCKS_CONFIG") {
        let p = PathBuf::from(path_str);
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".luarocks").join("config.lua"),
            home.join(".luarocks").join("config-5.4.lua"),
            home.join(".luarocks").join("config-5.3.lua"),
            home.join(".luarocks").join("config-5.1.lua"),
            home.join(".config").join("luarocks").join("config.lua"),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("luarocks").join("config.lua");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_luarocks_write_path() -> PathBuf {
    if let Some(p) = get_luarocks_config_path() {
        return p;
    }
    if let Ok(path_str) = std::env::var("LUAROCKS_CONFIG") {
        return PathBuf::from(path_str);
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".luarocks").join("config.lua");
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        return PathBuf::from(appdata).join("luarocks").join("config.lua");
    }
    PathBuf::from(".luarocks").join("config.lua")
}

pub fn get_luarocks_proxy() -> Option<(String, PathBuf)> {
    let p = get_luarocks_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_luarocks_proxy(&content)?;
    Some((proxy, p))
}

pub fn parse_vcpkg_asset_url(val: &str) -> Option<String> {
    for part in val.split(|c| c == ';' || c == ',') {
        let trim = part.trim();
        if trim.starts_with("http://") || trim.starts_with("https://") {
            return Some(trim.to_string());
        }
    }
    None
}

pub fn get_terraform_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(tf_cli) = std::env::var("TF_CLI_CONFIG_FILE") {
        let p = PathBuf::from(tf_cli);
        if p.exists() {
            paths.push(p);
        }
    }
    if let Ok(tofu_cli) = std::env::var("TOFU_CLI_CONFIG_FILE") {
        let p = PathBuf::from(tofu_cli);
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".terraformrc"),
            home.join(".terraform.d").join("config.tfrc"),
            home.join(".tofurc"),
            home.join(".tofu.d").join("config.tfrc"),
        ];
        for c in candidates {
            if c.exists() && !paths.contains(&c) {
                paths.push(c);
            }
        }
    }
    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("terraform.rc");
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths
}

pub fn get_vcpkg_configuration_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(vcpkg_conf) = std::env::var("VCPKG_CONFIGURATION_FILE") {
        let p = PathBuf::from(vcpkg_conf);
        if p.exists() {
            paths.push(p);
        }
    }
    let cwd_conf = PathBuf::from("vcpkg-configuration.json");
    if cwd_conf.exists() && !paths.contains(&cwd_conf) {
        paths.push(cwd_conf);
    }
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".vcpkg").join("vcpkg-configuration.json"),
            home.join(".config").join("vcpkg").join("vcpkg-configuration.json"),
        ];
        for c in candidates {
            if c.exists() && !paths.contains(&c) {
                paths.push(c);
            }
        }
    }
    if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(localappdata).join("vcpkg").join("vcpkg-configuration.json");
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths
}

pub fn parse_vcpkg_configuration_registries(content: &str) -> Vec<String> {
    let mut results = Vec::new();
    let Ok(val) = serde_json::from_str::<serde_json::Value>(content) else {
        return results;
    };

    if let Some(def_reg) = val.get("default-registry") {
        for key in ["repository", "location", "baseUrl"] {
            if let Some(url) = def_reg.get(key).and_then(|u| u.as_str()) {
                if url.starts_with("http://") || url.starts_with("https://") {
                    if !results.contains(&url.to_string()) {
                        results.push(url.to_string());
                    }
                }
            }
        }
    }

    if let Some(registries) = val.get("registries").and_then(|r| r.as_array()) {
        for reg in registries {
            for key in ["repository", "location", "baseUrl"] {
                if let Some(url) = reg.get(key).and_then(|u| u.as_str()) {
                    if url.starts_with("http://") || url.starts_with("https://") {
                        if !results.contains(&url.to_string()) {
                            results.push(url.to_string());
                        }
                    }
                }
            }
        }
    }

    results
}

pub fn clean_vcpkg_configuration_registries(
    content: &str,
    dead_registries: &[&str],
) -> (String, bool) {
    let Ok(mut val) = serde_json::from_str::<serde_json::Value>(content) else {
        return (content.to_string(), false);
    };

    let mut modified = false;

    if let Some(def_reg) = val.get_mut("default-registry").and_then(|d| d.as_object_mut()) {
        for key in ["repository", "location", "baseUrl"] {
            if let Some(url_val) = def_reg.get(key).and_then(|u| u.as_str()) {
                if dead_registries.contains(&url_val) {
                    if url_val != "https://github.com/microsoft/vcpkg" {
                        def_reg.insert(
                            key.to_string(),
                            serde_json::Value::String("https://github.com/microsoft/vcpkg".to_string()),
                        );
                        modified = true;
                    }
                }
            }
        }
    }

    if let Some(regs) = val.get_mut("registries").and_then(|r| r.as_array_mut()) {
        let orig_len = regs.len();
        regs.retain(|reg| {
            for key in ["repository", "location", "baseUrl"] {
                if let Some(u) = reg.get(key).and_then(|v| v.as_str()) {
                    if dead_registries.contains(&u) {
                        return false;
                    }
                }
            }
            true
        });
        if regs.len() != orig_len {
            modified = true;
        }
    }

    if modified {
        if let Ok(formatted) = serde_json::to_string_pretty(&val) {
            return (formatted + "\n", true);
        }
    }

    (content.to_string(), false)
}

pub fn parse_nimble_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        let lower = trim.to_lowercase();
        if lower.starts_with("http_proxy") || lower.starts_with("https_proxy") {
            if let Some((_, v)) = trim.split_once('=') {
                let clean = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !clean.is_empty()
                    && (clean.starts_with("http://")
                        || clean.starts_with("https://")
                        || clean.starts_with("socks"))
                {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

pub fn get_nimble_config_path() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("NIMBLE_DIR") {
        let p = PathBuf::from(dir).join("nimble.ini");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p = config_dir.join("nimble").join("nimble.ini");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".config").join("nimble").join("nimble.ini"),
            home.join(".nimble").join("nimble.ini"),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("nimble").join("nimble.ini");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_nimble_write_path() -> PathBuf {
    if let Some(p) = get_nimble_config_path() {
        return p;
    }
    if let Ok(dir) = std::env::var("NIMBLE_DIR") {
        return PathBuf::from(dir).join("nimble.ini");
    }
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("nimble").join("nimble.ini");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".config").join("nimble").join("nimble.ini");
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        return PathBuf::from(appdata).join("nimble").join("nimble.ini");
    }
    PathBuf::from("nimble.ini")
}

pub fn get_nimble_proxy() -> Option<(String, PathBuf)> {
    let p = get_nimble_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_nimble_proxy(&content)?;
    Some((proxy, p))
}

pub fn parse_rebar3_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('%') || trim.starts_with('#') {
            continue;
        }
        if trim.contains("http_proxy,") || trim.contains("https_proxy,") {
            if let Some((_, rest)) = trim.split_once(',') {
                let clean = rest
                    .trim()
                    .trim_end_matches('.')
                    .trim_end_matches('}')
                    .trim()
                    .trim_start_matches("<<")
                    .trim_end_matches(">>")
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim();
                if !clean.is_empty()
                    && (clean.starts_with("http://")
                        || clean.starts_with("https://")
                        || clean.starts_with("socks"))
                {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

pub fn get_rebar3_config_path() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("REBAR_GLOBAL_CONFIG_DIR") {
        let p = PathBuf::from(dir).join("rebar.config");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p = config_dir.join("rebar3").join("rebar.config");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".config").join("rebar3").join("rebar.config"),
            home.join(".rebar3").join("rebar.config"),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("rebar3").join("rebar.config");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_rebar3_write_path() -> PathBuf {
    if let Some(p) = get_rebar3_config_path() {
        return p;
    }
    if let Ok(dir) = std::env::var("REBAR_GLOBAL_CONFIG_DIR") {
        return PathBuf::from(dir).join("rebar.config");
    }
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("rebar3").join("rebar.config");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".config").join("rebar3").join("rebar.config");
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        return PathBuf::from(appdata).join("rebar3").join("rebar.config");
    }
    PathBuf::from("rebar.config")
}

pub fn get_rebar3_proxy() -> Option<(String, PathBuf)> {
    let p = get_rebar3_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_rebar3_proxy(&content)?;
    Some((proxy, p))
}

pub fn parse_scoop_proxy(content: &str) -> Option<String> {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(p) = val.get("proxy").and_then(|v| v.as_str()) {
            let clean = p.trim().trim_matches('"').trim_matches('\'').trim();
            if !clean.is_empty() && clean != "none" {
                return Some(clean.to_string());
            }
        }
    }
    None
}

pub fn get_scoop_config_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("SCOOP_CONFIG") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".config").join("scoop").join("config.json"),
            home.join(".scoop").join("config.json"),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p = config_dir.join("scoop").join("config.json");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("scoop").join("config.json");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_scoop_write_path() -> PathBuf {
    if let Some(p) = get_scoop_config_path() {
        return p;
    }
    if let Ok(path) = std::env::var("SCOOP_CONFIG") {
        return PathBuf::from(path);
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".config").join("scoop").join("config.json");
    }
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("scoop").join("config.json");
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        return PathBuf::from(appdata).join("scoop").join("config.json");
    }
    PathBuf::from("config.json")
}

pub fn get_scoop_proxy() -> Option<(String, PathBuf)> {
    let p = get_scoop_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_scoop_proxy(&content)?;
    Some((proxy, p))
}

pub fn parse_bundler_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim == "---" {
            continue;
        }
        let lower = trim.to_lowercase();
        if lower.starts_with("bundle_http_proxy")
            || lower.starts_with("bundle_https_proxy")
            || lower.starts_with("http_proxy:")
            || lower.starts_with("https_proxy:")
        {
            if let Some((_, v)) = trim.split_once(':') {
                let clean = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !clean.is_empty()
                    && (clean.starts_with("http://")
                        || clean.starts_with("https://")
                        || clean.starts_with("socks")
                        || clean.contains(':'))
                {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

pub fn get_bundler_config_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("BUNDLE_USER_CONFIG") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(home) = std::env::var("BUNDLE_USER_HOME") {
        let p = PathBuf::from(home).join("config");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".bundle").join("config"),
            home.join(".config").join("bundle").join("config"),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p = config_dir.join("bundle").join("config");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_bundler_write_path() -> PathBuf {
    if let Some(p) = get_bundler_config_path() {
        return p;
    }
    if let Ok(path) = std::env::var("BUNDLE_USER_CONFIG") {
        return PathBuf::from(path);
    }
    if let Ok(home) = std::env::var("BUNDLE_USER_HOME") {
        return PathBuf::from(home).join("config");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".bundle").join("config");
    }
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("bundle").join("config");
    }
    PathBuf::from("config")
}

pub fn get_bundler_proxy() -> Option<(String, PathBuf)> {
    let p = get_bundler_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_bundler_proxy(&content)?;
    Some((proxy, p))
}

pub fn parse_gcloud_config_proxy(content: &str) -> Option<String> {
    let mut in_proxy = false;
    let mut address = String::new();
    let mut port = String::new();
    let mut proxy_type = "http".to_string();

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim();
            in_proxy = section.eq_ignore_ascii_case("proxy");
            continue;
        }
        if in_proxy {
            if let Some((k, v)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                match key.as_str() {
                    "address" | "host" => address = val.to_string(),
                    "port" => port = val.to_string(),
                    "type" => proxy_type = val.to_ascii_lowercase(),
                    _ => {}
                }
            }
        }
    }

    if address.is_empty() {
        return None;
    }

    if address.starts_with("http://") || address.starts_with("https://") || address.starts_with("socks") {
        if !port.is_empty() && !address.contains(':') {
            return Some(format!("{}:{}", address, port));
        }
        return Some(address);
    }

    let scheme = match proxy_type.as_str() {
        "socks5" | "socks5h" => "socks5",
        "socks4" => "socks4",
        _ => "http",
    };

    if !port.is_empty() {
        Some(format!("{}://{}:{}", scheme, address, port))
    } else {
        Some(format!("{}://{}", scheme, address))
    }
}

pub fn get_gcloud_config_dir() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CLOUDSDK_CONFIG") {
        let p = PathBuf::from(path.trim());
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".config").join("gcloud");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p = config_dir.join("gcloud");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("gcloud");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_gcloud_config_path() -> Option<PathBuf> {
    let root = get_gcloud_config_dir()?;
    let active_name = if let Ok(act) = std::fs::read_to_string(root.join("active_config")) {
        let t = act.trim().to_string();
        if !t.is_empty() {
            t
        } else {
            "default".to_string()
        }
    } else {
        "default".to_string()
    };

    let p = root.join("configurations").join(format!("config_{}", active_name));
    if p.exists() {
        return Some(p);
    }
    let p_default = root.join("configurations").join("config_default");
    if p_default.exists() {
        return Some(p_default);
    }
    None
}

pub fn get_gcloud_write_path() -> PathBuf {
    if let Some(p) = get_gcloud_config_path() {
        return p;
    }
    let root = if let Ok(path) = std::env::var("CLOUDSDK_CONFIG") {
        PathBuf::from(path.trim())
    } else if let Some(home) = dirs::home_dir() {
        home.join(".config").join("gcloud")
    } else if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("gcloud")
    } else if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("gcloud")
    } else {
        PathBuf::from(".config").join("gcloud")
    };

    let active_name = if let Ok(act) = std::fs::read_to_string(root.join("active_config")) {
        let t = act.trim().to_string();
        if !t.is_empty() {
            t
        } else {
            "default".to_string()
        }
    } else {
        "default".to_string()
    };
    root.join("configurations").join(format!("config_{}", active_name))
}

pub fn get_gcloud_proxy() -> Option<(String, PathBuf)> {
    let p = get_gcloud_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_gcloud_config_proxy(&content)?;
    Some((proxy, p))
}

pub fn parse_aws_config_proxy(content: &str, profile: Option<&str>) -> Option<String> {
    let env_profile = std::env::var("AWS_PROFILE").unwrap_or_else(|_| "default".to_string());
    let target_profile = profile.unwrap_or(&env_profile);
    let mut in_target_section = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim();
            if target_profile == "default" {
                in_target_section = section.eq_ignore_ascii_case("default")
                    || section.eq_ignore_ascii_case("profile default");
            } else {
                in_target_section = section.eq_ignore_ascii_case(target_profile)
                    || section.eq_ignore_ascii_case(&format!("profile {}", target_profile));
            }
            continue;
        }
        if in_target_section {
            if let Some((k, v)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if (key == "proxy" || key == "http_proxy" || key == "https_proxy") && !val.is_empty() {
                    if val.starts_with("http://")
                        || val.starts_with("https://")
                        || val.starts_with("socks")
                    {
                        return Some(val.to_string());
                    } else {
                        return Some(format!("http://{}", val));
                    }
                }
            }
        }
    }
    None
}

pub fn get_aws_config_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("AWS_CONFIG_FILE") {
        let p = PathBuf::from(path.trim());
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".aws").join("config");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_aws_write_path() -> PathBuf {
    if let Some(p) = get_aws_config_path() {
        return p;
    }
    if let Ok(path) = std::env::var("AWS_CONFIG_FILE") {
        return PathBuf::from(path.trim());
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".aws").join("config");
    }
    PathBuf::from(".aws").join("config")
}

pub fn get_aws_proxy() -> Option<(String, PathBuf)> {
    let p = get_aws_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_aws_config_proxy(&content, None)?;
    Some((proxy, p))
}

pub fn parse_azure_config_proxy(content: &str) -> Option<String> {
    let mut in_proxy = false;
    let mut http_proxy = String::new();
    let mut https_proxy = String::new();
    let mut proxy_url = String::new();
    let mut mode_disabled = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim();
            in_proxy = section.eq_ignore_ascii_case("proxy");
            continue;
        }
        if in_proxy {
            if let Some((k, v)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                match key.as_str() {
                    "mode" => {
                        let m = val.to_ascii_lowercase();
                        if m == "none" || m == "disabled" || m == "off" || m == "direct" {
                            mode_disabled = true;
                        }
                    }
                    "https" | "https_proxy" => https_proxy = val.to_string(),
                    "http" | "http_proxy" => http_proxy = val.to_string(),
                    "proxy_url" | "url" => proxy_url = val.to_string(),
                    _ => {}
                }
            }
        }
    }

    if mode_disabled {
        return None;
    }

    let candidate = if !https_proxy.is_empty() {
        https_proxy
    } else if !http_proxy.is_empty() {
        http_proxy
    } else if !proxy_url.is_empty() {
        proxy_url
    } else {
        return None;
    };

    if candidate.starts_with("http://")
        || candidate.starts_with("https://")
        || candidate.starts_with("socks")
    {
        Some(candidate)
    } else {
        Some(format!("http://{}", candidate))
    }
}

pub fn get_azure_config_dir() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("AZURE_CONFIG_DIR") {
        let p = PathBuf::from(path.trim());
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".azure");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(userprofile).join(".azure");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_azure_config_path() -> Option<PathBuf> {
    if let Some(dir) = get_azure_config_dir() {
        let p = dir.join("config");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(path) = std::env::var("AZURE_CONFIG_DIR") {
        let p = PathBuf::from(path.trim()).join("config");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".azure").join("config");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_azure_write_path() -> PathBuf {
    if let Some(p) = get_azure_config_path() {
        return p;
    }
    if let Ok(path) = std::env::var("AZURE_CONFIG_DIR") {
        return PathBuf::from(path.trim()).join("config");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".azure").join("config");
    }
    PathBuf::from(".azure").join("config")
}

pub fn get_azure_proxy() -> Option<(String, PathBuf)> {
    let p = get_azure_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_azure_config_proxy(&content)?;
    Some((proxy, p))
}

pub fn get_containers_conf_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CONTAINERS_CONF") {
        let p = PathBuf::from(path.trim());
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(cfg) = dirs::config_dir() {
        let p = cfg.join("containers").join("containers.conf");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".config").join("containers").join("containers.conf");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("containers").join("containers.conf");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_containers_conf_write_path() -> PathBuf {
    if let Some(p) = get_containers_conf_path() {
        return p;
    }
    if let Ok(path) = std::env::var("CONTAINERS_CONF") {
        return PathBuf::from(path.trim());
    }
    if let Some(cfg) = dirs::config_dir() {
        return cfg.join("containers").join("containers.conf");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".config").join("containers").join("containers.conf");
    }
    PathBuf::from(".config").join("containers").join("containers.conf")
}

pub fn parse_podman_proxy(content: &str) -> Option<String> {
    let mut current_section = String::new();
    let mut in_env_array = false;
    let mut candidate = None;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if trim.starts_with('[') && trim.ends_with(']') {
            current_section = trim[1..trim.len() - 1].trim().to_ascii_lowercase();
            in_env_array = false;
            continue;
        }

        if current_section == "engine" || current_section == "engine.env" {
            if trim.starts_with("env") && trim.contains('=') && trim.contains('[') {
                in_env_array = true;
            }

            if in_env_array {
                let check_str = trim.trim_matches(|c| c == '[' || c == ']' || c == ',');
                for item in check_str.split(',') {
                    let item_clean = item.trim().trim_matches('"').trim_matches('\'').trim();
                    if let Some((k, v)) = item_clean.split_once('=') {
                        let k_lower = k.trim().to_ascii_lowercase();
                        if k_lower == "https_proxy" || k_lower == "http_proxy" || k_lower == "all_proxy" {
                            let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                            if !val.is_empty() {
                                candidate = Some(val.to_string());
                            }
                        }
                    }
                }
                if trim.ends_with(']') || trim.contains(']') {
                    in_env_array = false;
                }
            } else if let Some((k, v)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if key == "https_proxy" || key == "http_proxy" || key == "all_proxy" {
                    if !val.is_empty() {
                        candidate = Some(val.to_string());
                    }
                }
            }
        }
    }

    let cand = candidate?;
    if cand.starts_with("http://") || cand.starts_with("https://") || cand.starts_with("socks") {
        Some(cand)
    } else {
        Some(format!("http://{}", cand))
    }
}

pub fn get_podman_proxy() -> Option<(String, PathBuf)> {
    let p = get_containers_conf_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_podman_proxy(&content)?;
    Some((proxy, p))
}

pub fn get_vagrant_config_path() -> Option<PathBuf> {
    if let Ok(vhome) = std::env::var("VAGRANT_HOME") {
        let p = PathBuf::from(vhome.trim()).join("Vagrantfile");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".vagrant.d").join("Vagrantfile");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(userprofile).join(".vagrant.d").join("Vagrantfile");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_vagrant_write_path() -> PathBuf {
    if let Some(p) = get_vagrant_config_path() {
        return p;
    }
    if let Ok(vhome) = std::env::var("VAGRANT_HOME") {
        return PathBuf::from(vhome.trim()).join("Vagrantfile");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".vagrant.d").join("Vagrantfile");
    }
    PathBuf::from(".vagrant.d").join("Vagrantfile")
}

pub fn parse_vagrant_proxy(content: &str) -> Option<String> {
    let mut candidate = None;
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "config.proxy.http"
                || key == "config.proxy.https"
                || key == "config.proxy.http_proxy"
                || key == "config.proxy.https_proxy"
            {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() {
                    candidate = Some(val.to_string());
                }
            }
        }
    }

    let cand = candidate?;
    if cand.starts_with("http://") || cand.starts_with("https://") || cand.starts_with("socks") {
        Some(cand)
    } else {
        Some(format!("http://{}", cand))
    }
}

pub fn get_vagrant_proxy() -> Option<(String, PathBuf)> {
    let p = get_vagrant_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_vagrant_proxy(&content)?;
    Some((proxy, p))
}

pub fn get_choco_config_path() -> Option<PathBuf> {
    if let Ok(chocohome) = std::env::var("ChocolateyInstall") {
        let p = PathBuf::from(chocohome.trim()).join("config").join("chocolatey.config");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(chocohome) = std::env::var("CHOCOLATEY_INSTALL") {
        let p = PathBuf::from(chocohome.trim()).join("config").join("chocolatey.config");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(progdata) = std::env::var("ProgramData") {
        let p = PathBuf::from(progdata.trim()).join("chocolatey").join("config").join("chocolatey.config");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(allusers) = std::env::var("ALLUSERSPROFILE") {
        let p = PathBuf::from(allusers.trim()).join("chocolatey").join("config").join("chocolatey.config");
        if p.exists() {
            return Some(p);
        }
    }
    let default_win = PathBuf::from("C:\\ProgramData\\chocolatey\\config\\chocolatey.config");
    if default_win.exists() {
        return Some(default_win);
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p = config_dir.join("chocolatey").join("chocolatey.config");
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".config").join("chocolatey").join("chocolatey.config");
        if p.exists() {
            return Some(p);
        }
        let p2 = home.join(".chocolatey").join("chocolatey.config");
        if p2.exists() {
            return Some(p2);
        }
    }
    None
}

pub fn get_choco_write_path() -> PathBuf {
    if let Some(p) = get_choco_config_path() {
        return p;
    }
    if let Ok(chocohome) = std::env::var("ChocolateyInstall") {
        return PathBuf::from(chocohome.trim()).join("config").join("chocolatey.config");
    }
    if let Ok(chocohome) = std::env::var("CHOCOLATEY_INSTALL") {
        return PathBuf::from(chocohome.trim()).join("config").join("chocolatey.config");
    }
    if let Ok(progdata) = std::env::var("ProgramData") {
        return PathBuf::from(progdata.trim()).join("chocolatey").join("config").join("chocolatey.config");
    }
    if cfg!(windows) {
        return PathBuf::from("C:\\ProgramData\\chocolatey\\config\\chocolatey.config");
    }
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("chocolatey").join("chocolatey.config");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".config").join("chocolatey").join("chocolatey.config");
    }
    PathBuf::from(".config").join("chocolatey").join("chocolatey.config")
}

pub fn parse_choco_proxy(content: &str) -> Option<String> {
    let clean = strip_xml_comments(content);
    for line in clean.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();
        if lower.contains("<add ") && (lower.contains("key=\"proxy\"") || lower.contains("key='proxy'")) {
            if let Some(val) = extract_xml_attr(trim, "value") {
                let cleaned = val.trim();
                if !cleaned.is_empty() {
                    if cleaned.starts_with("http://") || cleaned.starts_with("https://") || cleaned.starts_with("socks") {
                        return Some(cleaned.to_string());
                    } else {
                        return Some(format!("http://{}", cleaned));
                    }
                }
            }
        }
    }
    None
}

pub fn get_choco_proxy() -> Option<(String, PathBuf)> {
    let p = get_choco_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_choco_proxy(&content)?;
    Some((proxy, p))
}

pub fn get_colima_config_path() -> Option<PathBuf> {
    if let Ok(colima_home) = std::env::var("COLIMA_HOME") {
        let candidates = [
            PathBuf::from(colima_home.trim()).join("default").join("colima.yaml"),
            PathBuf::from(colima_home.trim()).join("_current").join("colima.yaml"),
            PathBuf::from(colima_home.trim()).join("colima.yaml"),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".colima").join("default").join("colima.yaml"),
            home.join(".colima").join("_current").join("colima.yaml"),
            home.join(".colima").join("colima.yaml"),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }
    None
}

pub fn get_colima_write_path() -> PathBuf {
    if let Some(p) = get_colima_config_path() {
        return p;
    }
    if let Ok(colima_home) = std::env::var("COLIMA_HOME") {
        return PathBuf::from(colima_home.trim()).join("default").join("colima.yaml");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".colima").join("default").join("colima.yaml");
    }
    PathBuf::from(".colima").join("default").join("colima.yaml")
}

pub fn parse_colima_proxy(content: &str) -> Option<String> {
    let mut in_env = false;
    let mut candidate = None;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }

        if trim == "env:" || trim.starts_with("env:") {
            in_env = true;
            continue;
        }

        if in_env {
            if !line.starts_with(' ') && !line.starts_with('\t') && trim.ends_with(':') {
                in_env = false;
                continue;
            }

            if let Some((k, v)) = trim.split_once(':') {
                let key = k.trim().to_ascii_lowercase();
                if key == "http_proxy" || key == "https_proxy" || key == "all_proxy" {
                    let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                    if !val.is_empty() {
                        candidate = Some(val.to_string());
                    }
                }
            }
        }
    }

    let cand = candidate?;
    if cand.starts_with("http://") || cand.starts_with("https://") || cand.starts_with("socks") {
        Some(cand)
    } else {
        Some(format!("http://{}", cand))
    }
}

pub fn get_colima_proxy() -> Option<(String, PathBuf)> {
    let p = get_colima_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_colima_proxy(&content)?;
    Some((proxy, p))
}

pub fn get_minikube_config_path() -> Option<PathBuf> {
    if let Ok(minikube_home) = std::env::var("MINIKUBE_HOME") {
        let candidates = [
            PathBuf::from(minikube_home.trim()).join("config").join("config.json"),
            PathBuf::from(minikube_home.trim()).join(".minikube").join("config").join("config.json"),
            PathBuf::from(minikube_home.trim()).join("config.json"),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".minikube").join("config").join("config.json"),
            home.join(".minikube").join("config.json"),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }
    None
}

pub fn get_minikube_write_path() -> PathBuf {
    if let Some(p) = get_minikube_config_path() {
        return p;
    }
    if let Ok(minikube_home) = std::env::var("MINIKUBE_HOME") {
        return PathBuf::from(minikube_home.trim()).join("config").join("config.json");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".minikube").join("config").join("config.json");
    }
    PathBuf::from(".minikube").join("config").join("config.json")
}

pub fn parse_minikube_proxy(content: &str) -> Option<String> {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(obj) = v.as_object() {
            for key in ["http_proxy", "https_proxy", "HTTP_PROXY", "HTTPS_PROXY"] {
                if let Some(val) = obj.get(key).and_then(|x| x.as_str()) {
                    let trim = val.trim();
                    if !trim.is_empty() && trim != "none" {
                        if trim.starts_with("http://") || trim.starts_with("https://") || trim.starts_with("socks") {
                            return Some(trim.to_string());
                        } else {
                            return Some(format!("http://{}", trim));
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn get_minikube_proxy() -> Option<(String, PathBuf)> {
    let p = get_minikube_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_minikube_proxy(&content)?;
    Some((proxy, p))
}

pub fn get_spack_config_path() -> Option<PathBuf> {
    if let Ok(spack_path) = std::env::var("SPACK_USER_CONFIG_PATH") {
        let p = PathBuf::from(spack_path.trim());
        if p.is_file() && p.exists() {
            return Some(p);
        }
        let p_cfg = p.join("config.yaml");
        if p_cfg.exists() {
            return Some(p_cfg);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".spack").join("config.yaml"),
            home.join(".config").join("spack").join("config.yaml"),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p = config_dir.join("spack").join("config.yaml");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata.trim()).join("spack").join("config.yaml");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_spack_write_path() -> PathBuf {
    if let Some(p) = get_spack_config_path() {
        return p;
    }
    if let Ok(spack_path) = std::env::var("SPACK_USER_CONFIG_PATH") {
        let p = PathBuf::from(spack_path.trim());
        if p.ends_with("config.yaml") {
            return p;
        }
        return p.join("config.yaml");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".spack").join("config.yaml");
    }
    PathBuf::from(".spack").join("config.yaml")
}

pub fn parse_spack_proxy(content: &str) -> Option<String> {
    let mut candidate = None;
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trim.split_once(':') {
            let key = k.trim().to_ascii_lowercase();
            if key == "proxy" || key == "http_proxy" || key == "https_proxy" || key == "all_proxy" {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() && val != "none" {
                    candidate = Some(val.to_string());
                }
            }
        }
    }
    let cand = candidate?;
    if cand.starts_with("http://") || cand.starts_with("https://") || cand.starts_with("socks") {
        Some(cand)
    } else {
        Some(format!("http://{}", cand))
    }
}

pub fn get_spack_proxy() -> Option<(String, PathBuf)> {
    let p = get_spack_config_path()?;
    let content = std::fs::read_to_string(&p).ok()?;
    let proxy = parse_spack_proxy(&content)?;
    Some((proxy, p))
}

pub fn get_macports_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(mp_conf) = std::env::var("MACPORTS_CONF") {
        let p = PathBuf::from(mp_conf.trim());
        if p.exists() {
            paths.push(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let user_conf = home.join(".macports").join("macports.conf");
        if user_conf.exists() && !paths.contains(&user_conf) {
            paths.push(user_conf);
        }
    }
    let system_paths = [
        PathBuf::from("/opt/local/etc/macports/macports.conf"),
        PathBuf::from("/opt/macports/etc/macports/macports.conf"),
    ];
    for p in system_paths {
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths
}

pub fn get_macports_config_path() -> Option<PathBuf> {
    let paths = get_macports_config_paths();
    paths.into_iter().next()
}

pub fn get_macports_write_path() -> PathBuf {
    if let Some(p) = get_macports_config_path() {
        return p;
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".macports").join("macports.conf");
    }
    PathBuf::from("/opt/local/etc/macports/macports.conf")
}

pub fn parse_macports_proxy(content: &str) -> Option<String> {
    let mut candidate = None;
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trim.split_whitespace().collect();
        if parts.len() >= 2 {
            let key = parts[0].to_ascii_lowercase();
            if key == "proxy_http" || key == "proxy_https" || key == "proxy_ftp" {
                let val = parts[1].trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() && val != "none" {
                    candidate = Some(val.to_string());
                }
            }
        }
    }
    let cand = candidate?;
    if cand.starts_with("http://") || cand.starts_with("https://") || cand.starts_with("socks") {
        Some(cand)
    } else {
        Some(format!("http://{}", cand))
    }
}

pub fn get_macports_proxy() -> Option<(String, PathBuf)> {
    let paths = get_macports_config_paths();
    for p in paths {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(proxy) = parse_macports_proxy(&content) {
                return Some((proxy, p));
            }
        }
    }
    None
}

pub fn get_apt_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(apt_conf) = std::env::var("APT_CONFIG") {
        let p = PathBuf::from(apt_conf.trim());
        if p.exists() {
            paths.push(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let user_conf = home.join(".apt").join("apt.conf");
        if user_conf.exists() && !paths.contains(&user_conf) {
            paths.push(user_conf);
        }
    }
    let main_conf = PathBuf::from("/etc/apt/apt.conf");
    if main_conf.exists() && !paths.contains(&main_conf) {
        paths.push(main_conf);
    }
    let conf_d = PathBuf::from("/etc/apt/apt.conf.d");
    if conf_d.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&conf_d) {
            let mut sub_files: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_file())
                .collect();
            sub_files.sort();
            for p in sub_files {
                if !paths.contains(&p) {
                    paths.push(p);
                }
            }
        }
    }
    paths
}

pub fn get_apt_config_path() -> Option<PathBuf> {
    let paths = get_apt_config_paths();
    for p in &paths {
        if let Ok(content) = std::fs::read_to_string(p) {
            if parse_apt_proxy(&content).is_some() {
                return Some(p.clone());
            }
        }
    }
    paths.into_iter().next()
}

pub fn get_apt_write_path() -> PathBuf {
    if let Some(p) = get_apt_config_path() {
        return p;
    }
    let conf_d = PathBuf::from("/etc/apt/apt.conf.d");
    if conf_d.is_dir() {
        let default_target = conf_d.join("99proxydoc.conf");
        if let Ok(f) = std::fs::OpenOptions::new().write(true).create(true).open(&default_target) {
            drop(f);
            return default_target;
        }
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".apt").join("apt.conf");
    }
    PathBuf::from("/etc/apt/apt.conf.d/99proxydoc.conf")
}

pub fn parse_apt_proxy(content: &str) -> Option<String> {
    let mut candidate = None;
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with("//") || trim.starts_with('#') || trim.starts_with("/*") {
            continue;
        }
        let lower = trim.to_ascii_lowercase();
        if lower.contains("acquire::http::proxy")
            || lower.contains("acquire::https::proxy")
            || lower.contains("acquire::socks::proxy")
        {
            let val_part = if let Some((_, v)) = trim.split_once(' ') {
                v
            } else if let Some((_, v)) = trim.split_once('\t') {
                v
            } else if let Some((_, v)) = trim.split_once('=') {
                v
            } else {
                continue;
            };
            let cleaned = val_part
                .trim()
                .trim_end_matches(';')
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim();
            if !cleaned.is_empty()
                && cleaned.to_ascii_lowercase() != "direct"
                && cleaned.to_ascii_lowercase() != "false"
            {
                candidate = Some(cleaned.to_string());
            }
        }
    }
    let cand = candidate?;
    if cand.starts_with("http://") || cand.starts_with("https://") || cand.starts_with("socks") {
        Some(cand)
    } else {
        Some(format!("http://{}", cand))
    }
}

pub fn get_apt_proxy() -> Option<(String, PathBuf)> {
    let paths = get_apt_config_paths();
    for p in paths {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(proxy) = parse_apt_proxy(&content) {
                return Some((proxy, p));
            }
        }
    }
    None
}

pub fn get_dnf_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(dnf_conf) = std::env::var("DNF_CONF") {
        let p = PathBuf::from(dnf_conf.trim());
        if p.exists() {
            paths.push(p);
        }
    }
    if let Ok(yum_conf) = std::env::var("YUM_CONF") {
        let p = PathBuf::from(yum_conf.trim());
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let user_dnf = home.join(".config").join("dnf").join("dnf.conf");
        if user_dnf.exists() && !paths.contains(&user_dnf) {
            paths.push(user_dnf);
        }
        let user_yum = home.join(".config").join("yum").join("yum.conf");
        if user_yum.exists() && !paths.contains(&user_yum) {
            paths.push(user_yum);
        }
    }
    let system_paths = [
        PathBuf::from("/etc/dnf/dnf.conf"),
        PathBuf::from("/etc/yum.conf"),
        PathBuf::from("/etc/yum/yum.conf"),
    ];
    for p in system_paths {
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    let conf_d = PathBuf::from("/etc/dnf/dnf.conf.d");
    if conf_d.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&conf_d) {
            let mut sub_files: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_file() && p.extension().map_or(false, |ext| ext == "conf"))
                .collect();
            sub_files.sort();
            for p in sub_files {
                if !paths.contains(&p) {
                    paths.push(p);
                }
            }
        }
    }
    paths
}

pub fn get_dnf_config_path() -> Option<PathBuf> {
    let paths = get_dnf_config_paths();
    for p in &paths {
        if let Ok(content) = std::fs::read_to_string(p) {
            if parse_dnf_proxy(&content).is_some() {
                return Some(p.clone());
            }
        }
    }
    paths.into_iter().next()
}

pub fn get_dnf_write_path() -> PathBuf {
    if let Some(p) = get_dnf_config_path() {
        return p;
    }
    let dnf_etc = PathBuf::from("/etc/dnf");
    if dnf_etc.is_dir() {
        let default_target = dnf_etc.join("dnf.conf");
        if let Ok(f) = std::fs::OpenOptions::new().write(true).create(true).open(&default_target) {
            drop(f);
            return default_target;
        }
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".config").join("dnf").join("dnf.conf");
    }
    PathBuf::from("/etc/dnf/dnf.conf")
}

pub fn parse_dnf_proxy(content: &str) -> Option<String> {
    let mut candidate = None;
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            if k.trim().eq_ignore_ascii_case("proxy") {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty()
                    && !val.eq_ignore_ascii_case("none")
                    && !val.eq_ignore_ascii_case("off")
                    && !val.eq_ignore_ascii_case("_none_")
                    && val != "0"
                {
                    candidate = Some(val.to_string());
                }
            }
        }
    }
    let cand = candidate?;
    if cand.starts_with("http://") || cand.starts_with("https://") || cand.starts_with("socks") {
        Some(cand)
    } else {
        Some(format!("http://{}", cand))
    }
}

pub fn get_dnf_proxy() -> Option<(String, PathBuf)> {
    let paths = get_dnf_config_paths();
    for p in paths {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(proxy) = parse_dnf_proxy(&content) {
                return Some((proxy, p));
            }
        }
    }
    None
}

pub fn get_zypper_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(zypp_conf) = std::env::var("ZYPP_CONF") {
        let p = PathBuf::from(zypp_conf.trim());
        if p.exists() {
            paths.push(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let user_zypp = home.join(".zypp").join("zypp.conf");
        if user_zypp.exists() && !paths.contains(&user_zypp) {
            paths.push(user_zypp);
        }
        let user_cfg = home.join(".config").join("zypp").join("zypp.conf");
        if user_cfg.exists() && !paths.contains(&user_cfg) {
            paths.push(user_cfg);
        }
    }
    let system_conf = PathBuf::from("/etc/zypp/zypp.conf");
    if system_conf.exists() && !paths.contains(&system_conf) {
        paths.push(system_conf);
    }
    paths
}

pub fn get_zypper_config_path() -> Option<PathBuf> {
    let paths = get_zypper_config_paths();
    for p in &paths {
        if let Ok(content) = std::fs::read_to_string(p) {
            if parse_zypper_proxy(&content).is_some() {
                return Some(p.clone());
            }
        }
    }
    paths.into_iter().next()
}

pub fn get_zypper_write_path() -> PathBuf {
    if let Some(p) = get_zypper_config_path() {
        return p;
    }
    let zypp_etc = PathBuf::from("/etc/zypp");
    if zypp_etc.is_dir() {
        let default_target = zypp_etc.join("zypp.conf");
        if let Ok(f) = std::fs::OpenOptions::new().write(true).create(true).open(&default_target) {
            drop(f);
            return default_target;
        }
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".zypp").join("zypp.conf");
    }
    PathBuf::from("/etc/zypp/zypp.conf")
}

pub fn parse_zypper_proxy(content: &str) -> Option<String> {
    let mut candidate = None;
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "proxy"
                || key == "proxy.http"
                || key == "proxy.https"
                || key == "http_proxy"
                || key == "https_proxy"
            {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty()
                    && !val.eq_ignore_ascii_case("none")
                    && !val.eq_ignore_ascii_case("off")
                    && !val.eq_ignore_ascii_case("_none_")
                    && val != "0"
                {
                    candidate = Some(val.to_string());
                }
            }
        }
    }
    let cand = candidate?;
    if cand.starts_with("http://") || cand.starts_with("https://") || cand.starts_with("socks") {
        Some(cand)
    } else {
        Some(format!("http://{}", cand))
    }
}

pub fn get_zypper_proxy() -> Option<(String, PathBuf)> {
    let paths = get_zypper_config_paths();
    for p in paths {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(proxy) = parse_zypper_proxy(&content) {
                return Some((proxy, p));
            }
        }
    }
    None
}

pub fn get_helm_repositories_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(helm_home) = std::env::var("HELM_CONFIG_HOME") {
        let p = PathBuf::from(helm_home).join("repositories.yaml");
        if p.exists() {
            paths.push(p);
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        let p = config_dir.join("helm").join("repositories.yaml");
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".config").join("helm").join("repositories.yaml"),
            home.join("Library")
                .join("Preferences")
                .join("helm")
                .join("repositories.yaml"),
        ];
        for c in candidates {
            if c.exists() && !paths.contains(&c) {
                paths.push(c);
            }
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("helm").join("repositories.yaml");
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths
}

pub fn parse_helm_repositories(content: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut in_repos = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            continue;
        }
        if trim == "repositories:" {
            in_repos = true;
            continue;
        }
        if in_repos && !line.starts_with(' ') && !line.starts_with('\t') && trim.ends_with(':') {
            in_repos = false;
        }

        if in_repos || line.contains("url:") {
            if let Some((k, v)) = trim.split_once(':') {
                let k_trim = k.trim().trim_start_matches('-').trim();
                if k_trim == "url" {
                    let clean = v.trim().trim_matches('"').trim_matches('\'').trim();
                    if !clean.is_empty()
                        && (clean.starts_with("http://") || clean.starts_with("https://"))
                        && !urls.contains(&clean.to_string())
                    {
                        urls.push(clean.to_string());
                    }
                }
            }
        }
    }
    urls
}

pub fn clean_helm_repositories(content: &str, dead_urls: &[&str]) -> (String, bool) {
    if dead_urls.is_empty() {
        return (content.to_string(), false);
    }

    let mut out_lines = Vec::new();
    let mut current_block: Vec<String> = Vec::new();
    let mut block_has_dead = false;
    let mut in_repos = false;
    let mut modified = false;

    let flush_block = |out: &mut Vec<String>,
                       block: &mut Vec<String>,
                       has_dead: bool,
                       mod_flag: &mut bool| {
        if block.is_empty() {
            return;
        }
        if has_dead {
            out.push("  # [proxydoc disabled unreachable helm repository]".to_string());
            for l in block.drain(..) {
                out.push(format!("  # {}", l));
            }
            *mod_flag = true;
        } else {
            out.extend(block.drain(..));
        }
    };

    for line in content.lines() {
        let trim = line.trim();
        if trim == "repositories:" {
            flush_block(&mut out_lines, &mut current_block, block_has_dead, &mut modified);
            block_has_dead = false;
            in_repos = true;
            out_lines.push(line.to_string());
            continue;
        }

        if in_repos {
            if trim.starts_with("- ") {
                flush_block(&mut out_lines, &mut current_block, block_has_dead, &mut modified);
                block_has_dead = false;
                for dead in dead_urls {
                    if line.contains(dead) {
                        block_has_dead = true;
                    }
                }
                current_block.push(line.to_string());
                continue;
            } else if !line.starts_with(' ') && !line.starts_with('\t') && trim.ends_with(':') {
                flush_block(&mut out_lines, &mut current_block, block_has_dead, &mut modified);
                block_has_dead = false;
                in_repos = false;
                out_lines.push(line.to_string());
                continue;
            } else {
                for dead in dead_urls {
                    if line.contains(dead) {
                        block_has_dead = true;
                    }
                }
                current_block.push(line.to_string());
                continue;
            }
        }

        out_lines.push(line.to_string());
    }

    flush_block(&mut out_lines, &mut current_block, block_has_dead, &mut modified);

    let mut res = out_lines.join("\n");
    if content.ends_with('\n') {
        res.push('\n');
    }
    (res, modified)
}

pub fn get_pacman_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(pacman_conf) = std::env::var("PACMAN_CONF") {
        let p = PathBuf::from(pacman_conf.trim());
        if p.exists() {
            paths.push(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let user_pacman = home.join(".config").join("pacman").join("pacman.conf");
        if user_pacman.exists() && !paths.contains(&user_pacman) {
            paths.push(user_pacman);
        }
    }
    let system_paths = [
        PathBuf::from("/etc/pacman.conf"),
        PathBuf::from("C:\\msys64\\etc\\pacman.conf"),
        PathBuf::from("C:\\tools\\msys64\\etc\\pacman.conf"),
        PathBuf::from("C:\\msys2\\etc\\pacman.conf"),
    ];
    for p in system_paths {
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths
}

pub fn get_pacman_config_path() -> Option<PathBuf> {
    let paths = get_pacman_config_paths();
    for p in &paths {
        if let Ok(content) = std::fs::read_to_string(p) {
            if parse_pacman_proxy(&content).is_some() {
                return Some(p.clone());
            }
        }
    }
    paths.into_iter().next()
}

pub fn get_pacman_write_path() -> PathBuf {
    if let Some(p) = get_pacman_config_path() {
        return p;
    }
    let etc = PathBuf::from("/etc");
    if etc.is_dir() {
        let default_target = etc.join("pacman.conf");
        if let Ok(f) = std::fs::OpenOptions::new().write(true).create(true).open(&default_target) {
            drop(f);
            return default_target;
        }
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".config").join("pacman").join("pacman.conf");
    }
    PathBuf::from("/etc/pacman.conf")
}

pub fn parse_pacman_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            if k.trim().eq_ignore_ascii_case("xfercommand") {
                let cmd = v.trim();
                let tokens: Vec<&str> = cmd.split_whitespace().collect();
                for i in 0..tokens.len() {
                    let tok = tokens[i];
                    if tok.starts_with("--proxy=") {
                        let val = tok.trim_start_matches("--proxy=").trim_matches('"').trim_matches('\'').trim();
                        if !val.is_empty() {
                            return Some(if val.starts_with("http://") || val.starts_with("https://") || val.starts_with("socks") {
                                val.to_string()
                            } else {
                                format!("http://{}", val)
                            });
                        }
                    } else if tok == "--proxy" && i + 1 < tokens.len() {
                        let val = tokens[i + 1].trim_matches('"').trim_matches('\'').trim();
                        if !val.is_empty() {
                            return Some(if val.starts_with("http://") || val.starts_with("https://") || val.starts_with("socks") {
                                val.to_string()
                            } else {
                                format!("http://{}", val)
                            });
                        }
                    } else if tok.starts_with("-x=") {
                        let val = tok.trim_start_matches("-x=").trim_matches('"').trim_matches('\'').trim();
                        if !val.is_empty() {
                            return Some(if val.starts_with("http://") || val.starts_with("https://") || val.starts_with("socks") {
                                val.to_string()
                            } else {
                                format!("http://{}", val)
                            });
                        }
                    } else if tok == "-x" && i + 1 < tokens.len() {
                        let val = tokens[i + 1].trim_matches('"').trim_matches('\'').trim();
                        if !val.is_empty() {
                            return Some(if val.starts_with("http://") || val.starts_with("https://") || val.starts_with("socks") {
                                val.to_string()
                            } else {
                                format!("http://{}", val)
                            });
                        }
                    } else if tok.contains("http_proxy=") {
                        if let Some((_, val)) = tok.split_once("http_proxy=") {
                            let clean = val.trim_matches('"').trim_matches('\'').trim();
                            if !clean.is_empty() {
                                return Some(if clean.starts_with("http://") || clean.starts_with("https://") || clean.starts_with("socks") {
                                    clean.to_string()
                                } else {
                                    format!("http://{}", clean)
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn get_pacman_proxy() -> Option<(String, PathBuf)> {
    let paths = get_pacman_config_paths();
    for p in paths {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(proxy) = parse_pacman_proxy(&content) {
                return Some((proxy, p));
            }
        }
    }
    None
}

pub fn get_pkg_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(pkg_conf) = std::env::var("PKG_CONF") {
        let p = PathBuf::from(pkg_conf.trim());
        if p.exists() {
            paths.push(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let user_pkg = home.join(".config").join("pkg").join("pkg.conf");
        if user_pkg.exists() && !paths.contains(&user_pkg) {
            paths.push(user_pkg);
        }
    }
    let system_paths = [
        PathBuf::from("/usr/local/etc/pkg.conf"),
        PathBuf::from("/etc/pkg.conf"),
    ];
    for p in system_paths {
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths
}

pub fn get_pkg_config_path() -> Option<PathBuf> {
    let paths = get_pkg_config_paths();
    for p in &paths {
        if let Ok(content) = std::fs::read_to_string(p) {
            if parse_pkg_proxy(&content).is_some() {
                return Some(p.clone());
            }
        }
    }
    paths.into_iter().next()
}

pub fn get_pkg_write_path() -> PathBuf {
    if let Some(p) = get_pkg_config_path() {
        return p;
    }
    let local_etc = PathBuf::from("/usr/local/etc");
    if local_etc.is_dir() {
        let default_target = local_etc.join("pkg.conf");
        if let Ok(f) = std::fs::OpenOptions::new().write(true).create(true).open(&default_target) {
            drop(f);
            return default_target;
        }
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".config").join("pkg").join("pkg.conf");
    }
    PathBuf::from("/usr/local/etc/pkg.conf")
}

pub fn parse_pkg_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        let clean = trim.trim_end_matches(';').trim();
        let split_opt = if let Some((k, v)) = clean.split_once('=') {
            if k.trim().chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                Some((k, v))
            } else {
                clean.split_once(':')
            }
        } else {
            clean.split_once(':')
        };
        if let Some((k, v)) = split_opt {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy" || key == "https_proxy" || key == "pkg_proxy" || key == "proxy" {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty()
                    && !val.eq_ignore_ascii_case("none")
                    && !val.eq_ignore_ascii_case("off")
                    && val != "0"
                {
                    return Some(if val.starts_with("http://") || val.starts_with("https://") || val.starts_with("socks") {
                        val.to_string()
                    } else {
                        format!("http://{}", val)
                    });
                }
            }
        }
    }
    None
}

pub fn get_pkg_proxy() -> Option<(String, PathBuf)> {
    let paths = get_pkg_config_paths();
    for p in paths {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(proxy) = parse_pkg_proxy(&content) {
                return Some((proxy, p));
            }
        }
    }
    None
}

pub fn get_docker_daemon_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(docker_daemon) = std::env::var("DOCKER_DAEMON_JSON") {
        let p = PathBuf::from(docker_daemon.trim());
        if p.exists() {
            paths.push(p);
        }
    }
    let system_paths = [
        PathBuf::from("/etc/docker/daemon.json"),
        PathBuf::from("C:\\ProgramData\\Docker\\config\\daemon.json"),
        PathBuf::from("C:\\ProgramData\\docker\\config\\daemon.json"),
    ];
    for p in system_paths {
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let user_daemon = home.join(".docker").join("daemon.json");
        if user_daemon.exists() && !paths.contains(&user_daemon) {
            paths.push(user_daemon);
        }
        let rootless_daemon = home.join(".config").join("docker").join("daemon.json");
        if rootless_daemon.exists() && !paths.contains(&rootless_daemon) {
            paths.push(rootless_daemon);
        }
    }
    paths
}

pub fn parse_docker_daemon_proxy(content: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(content).ok()?;
    if let Some(proxies) = v.get("proxies").and_then(|p| p.as_object()) {
        for key in ["http-proxy", "https-proxy", "httpProxy", "httpsProxy"] {
            if let Some(val) = proxies.get(key).and_then(|s| s.as_str()) {
                let trim = val.trim();
                if !trim.is_empty() && !trim.eq_ignore_ascii_case("none") && !trim.eq_ignore_ascii_case("off") {
                    return Some(if trim.starts_with("http://") || trim.starts_with("https://") || trim.starts_with("socks") {
                        trim.to_string()
                    } else {
                        format!("http://{}", trim)
                    });
                }
            }
        }
    }
    for key in ["http-proxy", "https-proxy", "httpProxy", "httpsProxy"] {
        if let Some(val) = v.get(key).and_then(|s| s.as_str()) {
            let trim = val.trim();
            if !trim.is_empty() && !trim.eq_ignore_ascii_case("none") && !trim.eq_ignore_ascii_case("off") {
                return Some(if trim.starts_with("http://") || trim.starts_with("https://") || trim.starts_with("socks") {
                    trim.to_string()
                } else {
                    format!("http://{}", trim)
                });
            }
        }
    }
    None
}

pub fn get_docker_daemon_proxy() -> Option<(String, PathBuf)> {
    let paths = get_docker_daemon_config_paths();
    for p in paths {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(proxy) = parse_docker_daemon_proxy(&content) {
                return Some((proxy, p));
            }
        }
    }
    None
}

pub fn get_docker_daemon_write_path() -> PathBuf {
    if let Some((_, p)) = get_docker_daemon_proxy() {
        return p;
    }
    let paths = get_docker_daemon_config_paths();
    for p in paths {
        if p.exists() {
            return p;
        }
    }
    let etc_docker = PathBuf::from("/etc/docker");
    if etc_docker.is_dir() {
        let default_target = etc_docker.join("daemon.json");
        if let Ok(f) = std::fs::OpenOptions::new().write(true).create(true).open(&default_target) {
            drop(f);
            return default_target;
        }
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".docker").join("daemon.json");
    }
    PathBuf::from("/etc/docker/daemon.json")
}

pub fn get_portage_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(conf) = std::env::var("PORTAGE_MAKE_CONF") {
        let p = PathBuf::from(conf.trim());
        if p.exists() {
            paths.push(p);
        }
    }
    if let Ok(root) = std::env::var("PORTAGE_CONFIGROOT") {
        let p = PathBuf::from(root.trim()).join("etc/portage/make.conf");
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let user_conf = home.join(".config").join("portage").join("make.conf");
        if user_conf.exists() && !paths.contains(&user_conf) {
            paths.push(user_conf);
        }
    }
    let system_paths = [
        PathBuf::from("/etc/portage/make.conf"),
        PathBuf::from("/etc/make.conf"),
    ];
    for p in system_paths {
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths
}

pub fn get_portage_config_path() -> Option<PathBuf> {
    let paths = get_portage_config_paths();
    for p in &paths {
        if let Ok(content) = std::fs::read_to_string(p) {
            if parse_portage_proxy(&content).is_some() {
                return Some(p.clone());
            }
        }
    }
    paths.into_iter().next()
}

pub fn get_portage_write_path() -> PathBuf {
    if let Some(p) = get_portage_config_path() {
        return p;
    }
    let etc_portage = PathBuf::from("/etc/portage");
    if etc_portage.is_dir() {
        let default_target = etc_portage.join("make.conf");
        if let Ok(f) = std::fs::OpenOptions::new().write(true).create(true).open(&default_target) {
            drop(f);
            return default_target;
        }
    }
    let etc = PathBuf::from("/etc");
    if etc.is_dir() {
        let default_target = etc.join("make.conf");
        if let Ok(f) = std::fs::OpenOptions::new().write(true).create(true).open(&default_target) {
            drop(f);
            return default_target;
        }
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".config").join("portage").join("make.conf");
    }
    PathBuf::from("/etc/portage/make.conf")
}

pub fn parse_portage_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy" || key == "https_proxy" || key == "all_proxy" || key == "ftp_proxy" {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty()
                    && !val.eq_ignore_ascii_case("none")
                    && !val.eq_ignore_ascii_case("off")
                    && val != "0"
                {
                    return Some(if val.starts_with("http://") || val.starts_with("https://") || val.starts_with("socks") {
                        val.to_string()
                    } else {
                        format!("http://{}", val)
                    });
                }
            }
        }
    }
    None
}

pub fn get_portage_proxy() -> Option<(String, PathBuf)> {
    let paths = get_portage_config_paths();
    for p in paths {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(proxy) = parse_portage_proxy(&content) {
                return Some((proxy, p));
            }
        }
    }
    None
}

pub fn get_flatpak_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(dir) = std::env::var("FLATPAK_USER_DIR") {
        let p = PathBuf::from(dir.trim()).join("repo/config");
        if p.exists() {
            paths.push(p);
        }
    }
    if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        let p = PathBuf::from(xdg_data.trim()).join("flatpak/repo/config");
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let user_flatpak = home.join(".local").join("share").join("flatpak").join("repo").join("config");
        if user_flatpak.exists() && !paths.contains(&user_flatpak) {
            paths.push(user_flatpak);
        }
    }
    let system_paths = [
        PathBuf::from("/var/lib/flatpak/repo/config"),
        PathBuf::from("/etc/flatpak/repo/config"),
    ];
    for p in system_paths {
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    let remotes_d = PathBuf::from("/etc/flatpak/remotes.d");
    if remotes_d.is_dir() {
        if let Ok(entries) = std::fs::read_dir(remotes_d) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().map_or(false, |ext| ext == "conf") && !paths.contains(&p) {
                    paths.push(p);
                }
            }
        }
    }
    paths
}

pub fn get_flatpak_config_path() -> Option<PathBuf> {
    let paths = get_flatpak_config_paths();
    for p in &paths {
        if let Ok(content) = std::fs::read_to_string(p) {
            if parse_flatpak_proxy(&content).is_some() {
                return Some(p.clone());
            }
        }
    }
    paths.into_iter().next()
}

pub fn get_flatpak_write_path() -> PathBuf {
    if let Some(p) = get_flatpak_config_path() {
        return p;
    }
    if let Ok(dir) = std::env::var("FLATPAK_USER_DIR") {
        return PathBuf::from(dir.trim()).join("repo/config");
    }
    if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data.trim()).join("flatpak/repo/config");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".local").join("share").join("flatpak").join("repo").join("config");
    }
    PathBuf::from("/var/lib/flatpak/repo/config")
}

pub fn parse_flatpak_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }
        if let Some((k, v)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "proxy" || key == "http-proxy" || key == "http_proxy" {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty()
                    && !val.eq_ignore_ascii_case("none")
                    && !val.eq_ignore_ascii_case("off")
                    && val != "0"
                {
                    return Some(if val.starts_with("http://") || val.starts_with("https://") || val.starts_with("socks") {
                        val.to_string()
                    } else {
                        format!("http://{}", val)
                    });
                }
            }
        }
    }
    None
}

pub fn get_flatpak_proxy() -> Option<(String, PathBuf)> {
    let paths = get_flatpak_config_paths();
    for p in paths {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(proxy) = parse_flatpak_proxy(&content) {
                return Some((proxy, p));
            }
        }
    }
    None
}

pub fn get_snap_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(dir) = std::env::var("SNAP_CONF_DIR") {
        let p = PathBuf::from(dir.trim()).join("snap.conf");
        if p.exists() {
            paths.push(p);
        }
    }
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg_config.trim()).join("snap/snap.conf");
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let user_snap = home.join(".snap").join("snap.conf");
        if user_snap.exists() && !paths.contains(&user_snap) {
            paths.push(user_snap);
        }
        let user_cfg = home.join(".config").join("snap").join("snap.conf");
        if user_cfg.exists() && !paths.contains(&user_cfg) {
            paths.push(user_cfg);
        }
    }
    let system_paths = [
        PathBuf::from("/etc/systemd/system/snapd.service.d/http-proxy.conf"),
        PathBuf::from("/etc/systemd/system/snapd.service.d/snap_proxy.conf"),
        PathBuf::from("/etc/systemd/system/snapd.service.d/override.conf"),
        PathBuf::from("/var/lib/snapd/environment"),
        PathBuf::from("/etc/snapd/snapd.conf"),
        PathBuf::from("/etc/environment.d/snap.conf"),
    ];
    for p in system_paths {
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths
}

pub fn get_snap_config_path() -> Option<PathBuf> {
    let paths = get_snap_config_paths();
    for p in &paths {
        if let Ok(content) = std::fs::read_to_string(p) {
            if parse_snap_proxy(&content).is_some() {
                return Some(p.clone());
            }
        }
    }
    paths.into_iter().next()
}

pub fn get_snap_write_path() -> PathBuf {
    if let Some(p) = get_snap_config_path() {
        return p;
    }
    let systemd_dir = PathBuf::from("/etc/systemd/system/snapd.service.d");
    if systemd_dir.is_dir() {
        let candidate = systemd_dir.join("http-proxy.conf");
        if let Ok(f) = std::fs::OpenOptions::new().write(true).create(true).open(&candidate) {
            drop(f);
            return candidate;
        }
    }
    let var_lib_snapd = PathBuf::from("/var/lib/snapd");
    if var_lib_snapd.is_dir() {
        let env_path = var_lib_snapd.join("environment");
        if let Ok(f) = std::fs::OpenOptions::new().write(true).create(true).open(&env_path) {
            drop(f);
            return env_path;
        }
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".config").join("snap").join("snap.conf");
    }
    PathBuf::from("/etc/systemd/system/snapd.service.d/http-proxy.conf")
}

pub fn parse_snap_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }

        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..].trim().trim_matches('"').trim_matches('\'');
            for item in env_part.split_whitespace() {
                if let Some((k, v)) = item.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" || k_low == "https_proxy" || k_low == "all_proxy" {
                        let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                        if !val.is_empty()
                            && !val.eq_ignore_ascii_case("none")
                            && !val.eq_ignore_ascii_case("off")
                            && val != "0"
                        {
                            return Some(if val.starts_with("http://") || val.starts_with("https://") || val.starts_with("socks") {
                                val.to_string()
                            } else {
                                format!("http://{}", val)
                            });
                        }
                    }
                }
            }
        }

        if let Some((k, v)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "proxy.http"
                || key == "proxy.https"
                || key == "proxy_http"
                || key == "proxy_https"
                || key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "proxy"
            {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty()
                    && !val.eq_ignore_ascii_case("none")
                    && !val.eq_ignore_ascii_case("off")
                    && val != "0"
                {
                    return Some(if val.starts_with("http://") || val.starts_with("https://") || val.starts_with("socks") {
                        val.to_string()
                    } else {
                        format!("http://{}", val)
                    });
                }
            }
        }
    }
    None
}

pub fn get_snap_proxy() -> Option<(String, PathBuf)> {
    let paths = get_snap_config_paths();
    for p in paths {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(proxy) = parse_snap_proxy(&content) {
                return Some((proxy, p));
            }
        }
    }
    if let Ok(out) = std::process::Command::new("snap")
        .args(["get", "system", "proxy.http"])
        .output()
    {
        if out.status.success() {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !val.is_empty()
                && !val.eq_ignore_ascii_case("none")
                && !val.eq_ignore_ascii_case("off")
                && val != "0"
            {
                let proxy = if val.starts_with("http://") || val.starts_with("https://") || val.starts_with("socks") {
                    val
                } else {
                    format!("http://{}", val)
                };
                return Some((proxy, PathBuf::from("snap system proxy.http")));
            }
        }
    }
    None
}

pub fn get_ollama_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(dir) = std::env::var("OLLAMA_CONF_DIR") {
        let p = PathBuf::from(dir.trim()).join("environment");
        if p.exists() {
            paths.push(p);
        }
    }
    if let Ok(file) = std::env::var("OLLAMA_CONFIG_PATH") {
        let p = PathBuf::from(file.trim());
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let user_env = home.join(".ollama").join("environment");
        if user_env.exists() && !paths.contains(&user_env) {
            paths.push(user_env);
        }
        let user_cfg = home.join(".ollama").join("config");
        if user_cfg.exists() && !paths.contains(&user_cfg) {
            paths.push(user_cfg);
        }
        let xdg_env = home.join(".config").join("ollama").join("environment");
        if xdg_env.exists() && !paths.contains(&xdg_env) {
            paths.push(xdg_env);
        }
        let user_systemd = home
            .join(".config")
            .join("systemd")
            .join("user")
            .join("ollama.service.d")
            .join("http-proxy.conf");
        if user_systemd.exists() && !paths.contains(&user_systemd) {
            paths.push(user_systemd);
        }
        let mac_plist = home
            .join("Library")
            .join("LaunchAgents")
            .join("com.ollama.ollama.plist");
        if mac_plist.exists() && !paths.contains(&mac_plist) {
            paths.push(mac_plist);
        }
    }
    let system_paths = [
        PathBuf::from("/etc/systemd/system/ollama.service.d/http-proxy.conf"),
        PathBuf::from("/etc/systemd/system/ollama.service.d/override.conf"),
        PathBuf::from("/etc/systemd/system/ollama.service.d/proxy.conf"),
        PathBuf::from("/etc/systemd/system/ollama.service"),
        PathBuf::from("/etc/default/ollama"),
        PathBuf::from("/etc/ollama/ollama.conf"),
        PathBuf::from("/etc/sysconfig/ollama"),
        PathBuf::from("/Library/LaunchAgents/com.ollama.ollama.plist"),
    ];
    for p in system_paths {
        if p.exists() && !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths
}

pub fn get_ollama_config_path() -> Option<PathBuf> {
    let paths = get_ollama_config_paths();
    for p in &paths {
        if let Ok(content) = std::fs::read_to_string(p) {
            if parse_ollama_proxy(&content).is_some() {
                return Some(p.clone());
            }
        }
    }
    paths.into_iter().next()
}

pub fn get_ollama_write_path() -> PathBuf {
    if let Some(p) = get_ollama_config_path() {
        return p;
    }
    let systemd_dir = PathBuf::from("/etc/systemd/system/ollama.service.d");
    if systemd_dir.is_dir() {
        let candidate = systemd_dir.join("http-proxy.conf");
        if let Ok(f) = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&candidate)
        {
            drop(f);
            return candidate;
        }
    }
    if let Some(home) = dirs::home_dir() {
        let mac_agent_dir = home.join("Library").join("LaunchAgents");
        if mac_agent_dir.is_dir() {
            let mac_plist = mac_agent_dir.join("com.ollama.ollama.plist");
            if mac_plist.exists() {
                return mac_plist;
            }
        }
        let user_ollama = home.join(".ollama");
        if user_ollama.is_dir() {
            return user_ollama.join("environment");
        }
        let user_systemd_dir = home
            .join(".config")
            .join("systemd")
            .join("user")
            .join("ollama.service.d");
        if user_systemd_dir.is_dir() {
            return user_systemd_dir.join("http-proxy.conf");
        }
        return home.join(".ollama").join("environment");
    }
    PathBuf::from("/etc/systemd/system/ollama.service.d/http-proxy.conf")
}

pub fn parse_ollama_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            continue;
        }

        let lower = trim.to_ascii_lowercase();

        // Systemd drop-in format: Environment="HTTP_PROXY=..."
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            for item in env_part.split_whitespace() {
                if let Some((k, v)) = item.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy"
                        || k_low == "https_proxy"
                        || k_low == "all_proxy"
                        || k_low == "ollama_proxy"
                    {
                        let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                        if !val.is_empty()
                            && !val.eq_ignore_ascii_case("none")
                            && !val.eq_ignore_ascii_case("off")
                            && val != "0"
                        {
                            return Some(if val.starts_with("http://")
                                || val.starts_with("https://")
                                || val.starts_with("socks")
                            {
                                val.to_string()
                            } else {
                                format!("http://{}", val)
                            });
                        }
                    }
                }
            }
        }

        // Key-value / Environment format: HTTP_PROXY=... or OLLAMA_PROXY=...
        if let Some((k, v)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "ollama_proxy"
                || key == "proxy"
            {
                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty()
                    && !val.eq_ignore_ascii_case("none")
                    && !val.eq_ignore_ascii_case("off")
                    && val != "0"
                {
                    return Some(if val.starts_with("http://")
                        || val.starts_with("https://")
                        || val.starts_with("socks")
                    {
                        val.to_string()
                    } else {
                        format!("http://{}", val)
                    });
                }
            }
        }

        // macOS LaunchAgent plist format inline: <key>HTTP_PROXY</key><string>...</string>
        if lower.contains("<key>http_proxy</key>")
            || lower.contains("<key>https_proxy</key>")
            || lower.contains("<key>ollama_proxy</key>")
        {
            if let Some(pos) = lower.find("<string>") {
                let rest = &trim[pos + 8..];
                if let Some(end) = rest.to_ascii_lowercase().find("</string>") {
                    let val = rest[..end].trim();
                    if !val.is_empty() {
                        return Some(if val.starts_with("http://")
                            || val.starts_with("https://")
                            || val.starts_with("socks")
                        {
                            val.to_string()
                        } else {
                            format!("http://{}", val)
                        });
                    }
                }
            }
        }
    }

    // Multi-line plist XML: <key>HTTP_PROXY</key>\n<string>...</string>
    if content.contains("<key>HTTP_PROXY</key>")
        || content.contains("<key>http_proxy</key>")
        || content.contains("<key>HTTPS_PROXY</key>")
        || content.contains("<key>https_proxy</key>")
        || content.contains("<key>OLLAMA_PROXY</key>")
    {
        let mut key_matched = false;
        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();
            if lower == "<key>http_proxy</key>"
                || lower == "<key>https_proxy</key>"
                || lower == "<key>ollama_proxy</key>"
            {
                key_matched = true;
                continue;
            }
            if key_matched && lower.starts_with("<string>") && lower.ends_with("</string>") {
                let val = trim[8..trim.len() - 9].trim();
                if !val.is_empty() {
                    return Some(if val.starts_with("http://")
                        || val.starts_with("https://")
                        || val.starts_with("socks")
                    {
                        val.to_string()
                    } else {
                        format!("http://{}", val)
                    });
                }
            }
            if key_matched && trim.starts_with('<') {
                key_matched = false;
            }
        }
    }

    None
}

pub fn get_ollama_proxy() -> Option<(String, PathBuf)> {
    let paths = get_ollama_config_paths();
    for p in paths {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(proxy) = parse_ollama_proxy(&content) {
                return Some((proxy, p));
            }
        }
    }
    if let Ok(val) = std::env::var("OLLAMA_PROXY") {
        let trim = val.trim();
        if !trim.is_empty() {
            let proxy = if trim.starts_with("http://")
                || trim.starts_with("https://")
                || trim.starts_with("socks")
            {
                trim.to_string()
            } else {
                format!("http://{}", trim)
            };
            return Some((proxy, PathBuf::from("env:OLLAMA_PROXY")));
        }
    }
    None
}

pub fn get_apk_repositories_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(p_str) = std::env::var("APK_REPOSITORIES_PATH") {
        let p = PathBuf::from(p_str.trim());
        if p.exists() {
            paths.push(p);
        }
    }
    let default_path = PathBuf::from("/etc/apk/repositories");
    if default_path.exists() && !paths.contains(&default_path) {
        paths.push(default_path);
    }
    let d_dir = PathBuf::from("/etc/apk/repositories.d");
    if d_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(d_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().map(|e| e == "list").unwrap_or(false) && !paths.contains(&p) {
                    paths.push(p);
                }
            }
        }
    }
    paths
}

pub fn parse_apk_repositories(content: &str) -> Vec<String> {
    let mut repos = Vec::new();
    for line in content.lines() {
        let trim = line.trim();
        if trim.is_empty() || trim.starts_with('#') {
            continue;
        }
        let url_part = if trim.starts_with('@') {
            trim.split_whitespace().nth(1).unwrap_or(trim)
        } else {
            trim
        };
        if url_part.starts_with("http://") || url_part.starts_with("https://") {
            repos.push(url_part.to_string());
        }
    }
    repos
}

pub fn clean_apk_repositories(content: &str, dead_urls: &[&str]) -> (String, bool) {
    let mut out_lines = Vec::new();
    let mut modified = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.is_empty() || trim.starts_with('#') {
            out_lines.push(line.to_string());
            continue;
        }

        let is_dead = dead_urls.iter().any(|d| trim.contains(d));
        if is_dead {
            out_lines.push(format!("# [proxydoc disabled dead repository] {}", line));
            modified = true;
        } else {
            out_lines.push(line.to_string());
        }
    }

    (out_lines.join("\n") + "\n", modified)
}

pub async fn run(_verbose: bool, json_output: bool) -> Result<()> {
    if !json_output {
        println!(
            "\n{}",
            "🩺 ==================== DEEP PROXY DOCTOR ==================== 🩺"
                .bold()
                .cyan()
        );
        println!(
            "{}",
            "   Running comprehensive root-cause network & environment diagnostics...\n".dimmed()
        );
    }

    let mut items = Vec::new();
    let mut root_issues: Vec<String> = Vec::new();

    // 1. OS System Proxy vs Current Terminal Sync
    if let Some(sys_info) = get_system_proxy() {
        let sys = &sys_info.state;
        let source_name = &sys_info.source;
        if sys.http_enabled && sys.http_port > 0 {
            let active = scanner::check_host_port(&sys.http_host, sys.http_port).await;
            items.push(DiagItem::new(
                "System Proxy",
                source_name,
                format!("{}:{}", sys.http_host, sys.http_port),
                if active {
                    "● ACTIVE in OS (Port Open)".green().to_string()
                } else {
                    root_issues.push(format!(
                        "System proxy is pointing to dead port {}",
                        sys.http_port
                    ));
                    "▲ OS Proxy enabled, but port is DEAD!"
                        .red()
                        .bold()
                        .to_string()
                },
            ));

            // Check if terminal inherited this
            let term_set = std::env::var("http_proxy")
                .or_else(|_| std::env::var("HTTP_PROXY"))
                .is_ok();
            if !term_set {
                root_issues.push(format!(
                    "{} is active ({}:{}), but your Terminal is DIRECT! Run `proxydoc fix` to sync.",
                    source_name, sys.http_host, sys.http_port
                ));
                items.push(DiagItem::new(
                    "Sync Conflict",
                    "Terminal vs OS",
                    "OS=ON | Terminal=OFF".yellow().to_string(),
                    "▲ Out of sync! (Browser works, Terminal hangs)"
                        .yellow()
                        .bold()
                        .to_string(),
                ));
            }
        } else {
            items.push(DiagItem::new(
                "System Proxy",
                source_name,
                "(disabled)".dimmed().to_string(),
                "○ Direct in OS".dimmed().to_string(),
            ));
        }
    }

    // 2. Terminal Shell Env Vars
    let env_vars = [
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
    ];
    let mut shell_proxy_set = false;
    for var in env_vars {
        if let Ok(val) = std::env::var(var) {
            shell_proxy_set = true;
            let healthy = verify_proxy_url(&val).await;
            let dns_leak = check_socks5_dns_leak(&val);

            if !healthy {
                root_issues.push(format!(
                    "Terminal env `{}` points to an unreachable port `{}`",
                    var, val
                ));
            } else if dns_leak {
                root_issues.push(format!(
                    "Terminal env `{}` uses socks5:// instead of socks5h://! DNS queries will resolve locally and may leak or be poisoned.",
                    var
                ));
            }

            let status = if !healthy {
                "▲ DEAD PROXY PORT (Causes curl error 7)"
                    .red()
                    .bold()
                    .to_string()
            } else if dns_leak {
                "▲ Reachable, but socks5:// leaks local DNS (use socks5h://)"
                    .yellow()
                    .to_string()
            } else {
                "● Reachable Port".green().to_string()
            };

            items.push(DiagItem::new("Shell Env", var, val.clone(), status));
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
        let no_proxy = std::env::var("no_proxy")
            .or_else(|_| std::env::var("NO_PROXY"))
            .ok();
        match no_proxy {
            Some(ref val) => {
                let has_localhost =
                    val.contains("localhost") && (val.contains("127.0.0.1") || val.contains("::1"));
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
                    "▲ Missing (Local servers might hang)"
                        .yellow()
                        .bold()
                        .to_string(),
                ));
            }
        }
    }

    // Check for casing mismatch conflicts between lowercase and uppercase env vars
    let case_pairs = [
        ("http_proxy", "HTTP_PROXY"),
        ("https_proxy", "HTTPS_PROXY"),
        ("all_proxy", "ALL_PROXY"),
    ];
    for (lower_var, upper_var) in case_pairs {
        if let (Ok(lower_val), Ok(upper_val)) = (std::env::var(lower_var), std::env::var(upper_var))
        {
            if check_case_mismatch(&lower_val, &upper_val) {
                root_issues.push(format!(
                    "Proxy environment casing conflict! `{}` (`{}`) != `{}` (`{}`). Tools like cURL, Go, and Python will behave inconsistently.",
                    lower_var, lower_val, upper_var, upper_val
                ));
                items.push(DiagItem::new(
                    "Env Conflict",
                    format!("{} vs {}", lower_var, upper_var),
                    format!("{} != {}", lower_val, upper_val),
                    "▲ Inconsistent Values! (Dev tools disagree)"
                        .yellow()
                        .bold()
                        .to_string(),
                ));
            }
        }
    }

    // WSL2 Environment Check
    if let Some(host_ip) = detect_wsl2() {
        let term_proxy = std::env::var("http_proxy")
            .or_else(|_| std::env::var("all_proxy"))
            .unwrap_or_default();
        if term_proxy.contains("127.0.0.1") || term_proxy.contains("localhost") {
            root_issues.push(format!(
                "WSL2 detected! `127.0.0.1` routes inside Linux VM. Use Windows host IP `{}` or mirrored networking mode.",
                host_ip
            ));
            items.push(DiagItem::new(
                "WSL2 Host",
                "WSL2 Windows Gateway",
                host_ip,
                "▲ 127.0.0.1 routes inside VM (use Host IP)"
                    .yellow()
                    .bold()
                    .to_string(),
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
                root_issues.push(format!(
                    "git config --global http.proxy is pointing to dead port `{}`",
                    val
                ));
            }
            items.push(DiagItem::new(
                "Git",
                "http.proxy",
                val,
                if ok {
                    "● Reachable".green().to_string()
                } else {
                    "▲ DEAD PORT! (Git clone will fail)"
                        .red()
                        .bold()
                        .to_string()
                },
            ));
        }
        if let Some(val) = git_https {
            let ok = verify_proxy_url(&val).await;
            if !ok {
                root_issues.push(format!(
                    "git config --global https.proxy is pointing to dead port `{}`",
                    val
                ));
            }
            items.push(DiagItem::new(
                "Git",
                "https.proxy",
                val,
                if ok {
                    "● Reachable".green().to_string()
                } else {
                    "▲ DEAD PORT!".red().bold().to_string()
                },
            ));
        }
    }

    // Git SSL Verification & CA Certificate checks
    if let Some(val) = get_git_config("http.sslVerify") {
        if val.eq_ignore_ascii_case("false") {
            root_issues.push("`git config --global http.sslVerify false` is set! Certificate validation is globally DISABLED, exposing git traffic to MITM interception.".to_string());
            items.push(DiagItem::new(
                "Git SSL",
                "http.sslVerify",
                "false".yellow().to_string(),
                "▲ DISABLED! (Security risk / MITM vulnerable)"
                    .yellow()
                    .bold()
                    .to_string(),
            ));
        } else {
            items.push(DiagItem::new(
                "Git SSL",
                "http.sslVerify",
                val,
                "● Enabled".green().to_string(),
            ));
        }
    }

    if let Some(val) = get_git_config("http.sslCAInfo") {
        let p = std::path::Path::new(&val);
        if p.exists() {
            items.push(DiagItem::new(
                "Git SSL",
                "http.sslCAInfo",
                val,
                "● Certificate bundle exists".green().to_string(),
            ));
        } else {
            root_issues.push(format!(
                "Git config `http.sslCAInfo` points to missing certificate file `{}`! (git clone/push will fail with 'unable to load SSL CA info')",
                val
            ));
            items.push(DiagItem::new(
                "Git SSL",
                "http.sslCAInfo",
                val,
                "▲ MISSING FILE! (Git network operations will fail)"
                    .red()
                    .bold()
                    .to_string(),
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

    // GitHub CLI (gh) Config (~/.config/gh/config.yml)
    if let Some(val) = get_gh_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!(
                "GitHub CLI (gh) config points to dead proxy `{}`! (gh repo/pr, git operations & Copilot CLI will fail)",
                val
            ));
        }
        items.push(DiagItem::new(
            "GitHub CLI",
            "gh config (http_proxy)",
            val,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (gh repo/pr & Copilot fail)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "GitHub CLI",
            "gh config (http_proxy)",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // GitLab CLI (glab) Config (~/.config/glab-cli/config.yml)
    if let Some(val) = get_glab_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!(
                "GitLab CLI (glab) config points to dead proxy `{}`! (glab repo/mr & API operations will fail)",
                val
            ));
        }
        items.push(DiagItem::new(
            "GitLab CLI",
            "glab config (glab_proxy)",
            val,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (glab repo/mr fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "GitLab CLI",
            "glab config (glab_proxy)",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
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
                root_issues.push(format!(
                    "npm config proxy points to dead port `{}` (npm install will fail)",
                    val
                ));
            }
            items.push(DiagItem::new(
                "NPM",
                "proxy",
                val,
                if ok {
                    "● Reachable".green().to_string()
                } else {
                    "▲ DEAD PORT! (npm install will fail)"
                        .red()
                        .bold()
                        .to_string()
                },
            ));
        }
        if let Some(val) = npm_https {
            let ok = verify_proxy_url(&val).await;
            if !ok {
                root_issues.push(format!(
                    "npm config https-proxy points to dead port `{}`",
                    val
                ));
            }
            items.push(DiagItem::new(
                "NPM",
                "https-proxy",
                val,
                if ok {
                    "● Reachable".green().to_string()
                } else {
                    "▲ DEAD PORT!".red().bold().to_string()
                },
            ));
        }
    }

    // 5. Yarn Config (~/.yarnrc / ~/.yarnrc.yml)
    if let Some(cfg) = get_yarn_proxy() {
        let ok = verify_proxy_url(&cfg.proxy).await;
        if !ok {
            root_issues.push(format!(
                "Yarn config ({}) points to dead proxy `{}` (yarn add & install will fail)",
                cfg.source, cfg.proxy
            ));
        }
        items.push(DiagItem::new(
            "Yarn",
            format!("{} proxy", cfg.source),
            cfg.proxy,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (yarn install fails)".red().bold().to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Yarn",
            "~/.yarnrc / ~/.yarnrc.yml",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // 6. pnpm Config (~/.pnpmrc)
    if let Some(val) = get_pnpm_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!(
                "pnpm proxy points to dead port `{}` (pnpm install will fail)",
                val
            ));
        }
        items.push(DiagItem::new(
            "pnpm",
            "~/.pnpmrc proxy",
            val,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (pnpm install fails)".red().bold().to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "pnpm",
            "~/.pnpmrc proxy",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // 7. Bun Config (~/.bunfig.toml)
    if let Some(val) = get_bun_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!(
                "Bun proxy in ~/.bunfig.toml points to dead port `{}` (bun install will fail)",
                val
            ));
        }
        items.push(DiagItem::new(
            "Bun",
            "~/.bunfig.toml [install] proxy",
            val,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (bun install fails)".red().bold().to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Bun",
            "~/.bunfig.toml proxy",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // Node / JS Ecosystem Registry Mirrors (~/.npmrc, ~/.yarnrc, ~/.pnpmrc, NPM_CONFIG_REGISTRY)
    let node_registries = get_npm_registries();
    for reg in node_registries {
        let is_deprecated_taobao = reg.url.contains("registry.npm.taobao.org");
        if is_deprecated_taobao {
            root_issues.push(format!(
                "{} is using DEPRECATED & OFFLINE domain 'registry.npm.taobao.org'! Run `proxydoc fix` to migrate to 'https://registry.npmmirror.com'.",
                reg.source
            ));
            items.push(DiagItem::new(
                "Node Registry",
                &reg.source,
                reg.url,
                "▲ DEPRECATED & OFFLINE! (SSL/fetch fails)"
                    .red()
                    .bold()
                    .to_string(),
            ));
        } else {
            let ok = verify_proxy_url(&reg.url).await;
            if !ok {
                root_issues.push(format!(
                    "{} mirror `{}` is unreachable! (npm/yarn/pnpm install will fail with network timeout)",
                    reg.source, reg.url
                ));
            }
            items.push(DiagItem::new(
                "Node Registry",
                &reg.source,
                reg.url,
                if ok {
                    "● Reachable Mirror".green().to_string()
                } else {
                    "▲ UNREACHABLE MIRROR!".red().bold().to_string()
                },
            ));
        }
    }

    // 8. Cargo Config
    if let Some(val) = get_cargo_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!(
                "Cargo [http] proxy points to dead port `{}` (cargo fetch will fail)",
                val
            ));
        }
        items.push(DiagItem::new(
            "Cargo",
            "cargo config [http] proxy",
            val,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (cargo will fail)".red().bold().to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Cargo",
            "cargo config [http] proxy",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // Cargo Crates.io Registry Mirror Replacements
    let cargo_mirrors = get_cargo_registry_replacements();
    if !cargo_mirrors.is_empty() {
        for cm in cargo_mirrors {
            if let Some(ref reg) = cm.registry_url {
                let clean_url = reg.trim_start_matches("sparse+");
                let ok = verify_proxy_url(clean_url).await;
                if !ok {
                    root_issues.push(format!(
                        "Cargo registry mirror `{}` (`{}`) for `{}` is unreachable! (cargo build/fetch will fail with index download error)",
                        cm.replace_with, clean_url, cm.source
                    ));
                }
                items.push(DiagItem::new(
                    "Cargo Mirror",
                    format!("{} replace-with {}", cm.source, cm.replace_with),
                    reg.clone(),
                    if ok {
                        "● Reachable Registry Mirror".green().to_string()
                    } else {
                        "▲ DEAD CRATES.IO MIRROR!".red().bold().to_string()
                    },
                ));
            } else {
                items.push(DiagItem::new(
                    "Cargo Mirror",
                    format!("{} replace-with {}", cm.source, cm.replace_with),
                    cm.replace_with.clone(),
                    "● Configured Mirror".dimmed().to_string(),
                ));
            }
        }
    }

    // Rustup Mirrors
    let rustup_vars = [
        ("RUSTUP_DIST_SERVER", "Rust toolchain download server"),
        ("RUSTUP_UPDATE_ROOT", "Rustup self-update server"),
    ];
    for (var, desc) in rustup_vars {
        if let Ok(val) = std::env::var(var) {
            let trim = val.trim();
            if !trim.is_empty() {
                let ok = verify_proxy_url(trim).await;
                if !ok {
                    root_issues.push(format!(
                        "Rustup environment variable `{}` points to unreachable mirror `{}`! (rustup toolchain install/update will fail)",
                        var, trim
                    ));
                }
                items.push(DiagItem::new(
                    "Rustup Mirror",
                    format!("{} ({})", var, desc),
                    trim.to_string(),
                    if ok {
                        "● Reachable Mirror".green().to_string()
                    } else {
                        "▲ UNREACHABLE MIRROR!".red().bold().to_string()
                    },
                ));
            }
        }
    }

    // 9. Pip Config
    if let Some(val) = get_pip_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!(
                "Pip proxy points to dead port `{}` (pip install will fail)",
                val
            ));
        }
        items.push(DiagItem::new(
            "Pip",
            "pip global.proxy",
            val,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (pip install will fail)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Pip",
            "pip global.proxy",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // Python / Astral uv Index Mirrors
    let python_index_vars = [
        ("UV_INDEX_URL", "Astral uv primary index mirror"),
        ("UV_EXTRA_INDEX_URL", "Astral uv extra index mirror"),
        ("PIP_INDEX_URL", "Pip default package index URL"),
        ("PIP_EXTRA_INDEX_URL", "Pip extra package index URL"),
    ];
    for (var, desc) in python_index_vars {
        if let Ok(val) = std::env::var(var) {
            let trim = val.trim();
            if !trim.is_empty() {
                let ok = verify_proxy_url(trim).await;
                if !ok {
                    root_issues.push(format!(
                        "Python index mirror `{}` points to unreachable URL `{}`! (uv / pip package downloads will fail)",
                        var, trim
                    ));
                }
                items.push(DiagItem::new(
                    "Python Index",
                    format!("{} ({})", var, desc),
                    trim.to_string(),
                    if ok {
                        "● Reachable Index".green().to_string()
                    } else {
                        "▲ UNREACHABLE INDEX! (Downloads fail)"
                            .red()
                            .bold()
                            .to_string()
                    },
                ));
            }
        }
    }

    // Astral uv Proxy (UV_HTTP_PROXY & ~/.config/uv/uv.toml)
    let mut uv_configured = false;
    if let Ok(val) = std::env::var("UV_HTTP_PROXY") {
        let trim = val.trim();
        if !trim.is_empty() {
            uv_configured = true;
            let ok = verify_proxy_url(trim).await;
            if !ok {
                root_issues.push(format!(
                    "Astral uv proxy `UV_HTTP_PROXY` points to dead port `{}`! (uv sync / run / pip will fail)",
                    trim
                ));
            }
            items.push(DiagItem::new(
                "Astral uv",
                "UV_HTTP_PROXY",
                trim.to_string(),
                if ok {
                    "● Reachable Proxy".green().to_string()
                } else {
                    "▲ DEAD PROXY PORT! (uv sync & pip fail)"
                        .red()
                        .bold()
                        .to_string()
                },
            ));
        }
    }
    if let Some(val) = get_uv_proxy() {
        uv_configured = true;
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!(
                "Astral uv config `uv.toml` http-proxy points to dead port `{}`! (uv sync / pip will fail)",
                val
            ));
        }
        items.push(DiagItem::new(
            "Astral uv",
            "uv.toml [http-proxy]",
            val,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (uv sync fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    }
    if !uv_configured {
        items.push(DiagItem::new(
            "Astral uv",
            "uv.toml / UV_HTTP_PROXY",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // 7. cURL Config (~/.curlrc)
    if let Some(val) = get_curlrc_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!(
                "~/.curlrc points to dead proxy `{}` (curl commands & bash installers will fail)",
                val
            ));
        }
        items.push(DiagItem::new(
            "cURL",
            "~/.curlrc proxy",
            val,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (curl commands fail)".red().bold().to_string()
            },
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
            root_issues.push(format!(
                "~/.wgetrc points to dead proxy `{}` (wget commands will fail)",
                val
            ));
        }
        items.push(DiagItem::new(
            "Wget",
            "~/.wgetrc proxy",
            val,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (wget will fail)".red().bold().to_string()
            },
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
                "▲ DEAD PORT! (docker build/pull will fail)"
                    .red()
                    .bold()
                    .to_string()
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

    // 12. NuGet (.NET) Config (~/.nuget/NuGet/NuGet.Config)
    if let Some((val, path)) = get_nuget_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!(
                "NuGet config ({}) points to dead proxy `{}` (.NET restore & package installs will fail)",
                path.display(),
                val
            ));
        }
        items.push(DiagItem::new(
            "NuGet (.NET)",
            "NuGet.Config http_proxy",
            val,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (dotnet restore fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "NuGet (.NET)",
            "NuGet.Config http_proxy",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // Gem (Ruby / CocoaPods) Config (~/.gemrc)
    if let Some(val) = get_gem_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!("Ruby/Gem config (~/.gemrc) points to dead proxy `{}` (gem install, bundle install & pod install will fail)", val));
        }
        items.push(DiagItem::new(
            "Gem (Ruby)",
            "~/.gemrc http_proxy",
            val,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (gem & pod fail)".red().bold().to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Gem (Ruby)",
            "~/.gemrc http_proxy",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // Subversion (SVN) Config (~/.subversion/servers)
    if let Some(val) = get_svn_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!("Subversion config (~/.subversion/servers) points to dead proxy `{}` (svn checkout/update will fail)", val));
        }
        items.push(DiagItem::new(
            "Subversion",
            "~/.subversion/servers",
            val,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (svn operations fail)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Subversion",
            "~/.subversion/servers",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // Mercurial (Hg) Config (~/.hgrc / Mercurial.ini)
    if let Some(val) = get_hg_proxy() {
        let ok = verify_proxy_url(&val).await;
        if !ok {
            root_issues.push(format!(
                "Mercurial config (~/.hgrc) points to dead proxy `{}` (hg clone/pull will fail)",
                val
            ));
        }
        items.push(DiagItem::new(
            "Mercurial",
            "~/.hgrc [http_proxy]",
            val,
            if ok {
                "● Reachable".green().to_string()
            } else {
                "▲ DEAD PORT! (hg operations fail)".red().bold().to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Mercurial",
            "~/.hgrc [http_proxy]",
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
                root_issues.push(format!(
                    "GOPROXY mirror `{}` is unreachable! (go get / go build will fail)",
                    mirror
                ));
                any_dead = true;
            }
        }
        let status = if mirrors.is_empty() {
            "○ Direct (direct / off)".dimmed().to_string()
        } else if any_dead {
            "▲ DEAD GOPROXY MIRROR! (go get fails)"
                .red()
                .bold()
                .to_string()
        } else {
            "● Reachable GOPROXY".green().to_string()
        };
        items.push(DiagItem::new("Go Modules", "GOPROXY", gp, status));
    } else {
        items.push(DiagItem::new(
            "Go Modules",
            "GOPROXY",
            "(default proxy.golang.org,direct)".dimmed().to_string(),
            "○ Default Official Proxy".dimmed().to_string(),
        ));
    }

    // 13. Conda Config (~/.condarc)
    if let Some(cfg) = get_conda_proxy() {
        let mut any_dead = false;
        let mut display_parts = Vec::new();

        if let Some(ref http) = cfg.http {
            let ok = verify_proxy_url(http).await;
            if !ok {
                root_issues.push(format!("Conda proxy (http) in ~/.condarc points to dead port `{}` (conda install will fail)", http));
                any_dead = true;
            }
            display_parts.push(format!("http: {}", http));
        }

        if let Some(ref https) = cfg.https {
            let ok = verify_proxy_url(https).await;
            if !ok {
                root_issues.push(format!("Conda proxy (https) in ~/.condarc points to dead port `{}` (conda install will fail)", https));
                any_dead = true;
            }
            display_parts.push(format!("https: {}", https));
        }

        let val_str = display_parts.join(", ");
        let status = if any_dead {
            "▲ DEAD PORT! (conda install fails)"
                .red()
                .bold()
                .to_string()
        } else {
            "● Reachable".green().to_string()
        };

        items.push(DiagItem::new(
            "Conda",
            "~/.condarc proxy_servers",
            val_str,
            status,
        ));
    } else {
        items.push(DiagItem::new(
            "Conda",
            "~/.condarc proxy_servers",
            "(not set)".dimmed().to_string(),
            "○ Clean (Inherits Shell or Direct)".dimmed().to_string(),
        ));
    }

    // Mamba / Micromamba Config (~/.mambarc)
    if let Some(mamba_path) = get_mambarc_path() {
        if let Ok(content) = std::fs::read_to_string(&mamba_path) {
            if let Some(cfg) = parse_condarc_proxy(&content) {
                let mut any_dead = false;
                let mut display_parts = Vec::new();

                if let Some(ref http) = cfg.http {
                    let ok = verify_proxy_url(http).await;
                    if !ok {
                        root_issues.push(format!(
                            "Mamba proxy (http) in {} points to dead port `{}` (mamba install will fail)",
                            mamba_path.display(),
                            http
                        ));
                        any_dead = true;
                    }
                    display_parts.push(format!("http: {}", http));
                }

                if let Some(ref https) = cfg.https {
                    let ok = verify_proxy_url(https).await;
                    if !ok {
                        root_issues.push(format!(
                            "Mamba proxy (https) in {} points to dead port `{}` (mamba install will fail)",
                            mamba_path.display(),
                            https
                        ));
                        any_dead = true;
                    }
                    display_parts.push(format!("https: {}", https));
                }

                let val_str = display_parts.join(", ");
                let status = if any_dead {
                    "▲ DEAD PORT! (mamba install fails)"
                        .red()
                        .bold()
                        .to_string()
                } else {
                    "● Reachable".green().to_string()
                };

                items.push(DiagItem::new(
                    "Mamba",
                    format!("{} proxy_servers", mamba_path.display()),
                    val_str,
                    status,
                ));
            }
        }
    }

    // PHP Composer Packagist Mirror & Proxy
    if let Some(comp_path) = get_composer_config_path() {
        if let Ok(content) = std::fs::read_to_string(&comp_path) {
            if let Some(cproxy) = parse_composer_proxy(&content) {
                let ok = verify_proxy_url(&cproxy).await;
                if !ok {
                    root_issues.push(format!(
                        "PHP Composer proxy `{}` in {} is dead! (composer install will fail with cURL error)",
                        cproxy, comp_path.display()
                    ));
                }
                items.push(DiagItem::new(
                    "PHP Composer",
                    "Config Proxy",
                    cproxy,
                    if ok {
                        "● Reachable Proxy".green().to_string()
                    } else {
                        "▲ DEAD PROXY PORT! (composer install fails)"
                            .red()
                            .bold()
                            .to_string()
                    },
                ));
            } else {
                items.push(DiagItem::new(
                    "PHP Composer",
                    "Config Proxy",
                    "(not set)".dimmed().to_string(),
                    "○ Inherits Shell or Direct".dimmed().to_string(),
                ));
            }

            if let Some(mirror_url) = parse_composer_packagist_repo(&content) {
                let ok = verify_proxy_url(&mirror_url).await;
                if !ok {
                    root_issues.push(format!(
                        "PHP Composer packagist mirror `{}` in {} is unreachable! (composer install will fail)",
                        mirror_url, comp_path.display()
                    ));
                }
                items.push(DiagItem::new(
                    "PHP Composer",
                    "Packagist Mirror",
                    mirror_url,
                    if ok {
                        "● Reachable Mirror".green().to_string()
                    } else {
                        "▲ UNREACHABLE MIRROR! (composer install fails)"
                            .red()
                            .bold()
                            .to_string()
                    },
                ));
            }
        }
    } else {
        items.push(DiagItem::new(
            "PHP Composer",
            "Config Proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Bazel Proxy Configuration (~/.bazelrc)
    if let Some(bpath) = get_bazelrc_path() {
        if let Ok(content) = std::fs::read_to_string(&bpath) {
            if let Some(bproxy) = parse_bazelrc_proxy(&content) {
                let ok = verify_proxy_url(&bproxy).await;
                if !ok {
                    root_issues.push(format!(
                        "Bazel ~/.bazelrc action_env proxy `{}` is dead! (Bazel external repo downloads will fail)",
                        bproxy
                    ));
                }
                items.push(DiagItem::new(
                    "Bazel",
                    "~/.bazelrc action_env",
                    bproxy,
                    if ok {
                        "● Reachable Proxy".green().to_string()
                    } else {
                        "▲ DEAD PROXY PORT! (Bazel build fails)"
                            .red()
                            .bold()
                            .to_string()
                    },
                ));
            } else {
                items.push(DiagItem::new(
                    "Bazel",
                    "~/.bazelrc action_env",
                    "(not set)".dimmed().to_string(),
                    "○ Inherits Shell or Direct".dimmed().to_string(),
                ));
            }
        }
    } else {
        items.push(DiagItem::new(
            "Bazel",
            "~/.bazelrc action_env",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Conan C/C++ Package Manager Proxy (~/.conan2/global.conf / ~/.conan/conan.conf)
    if let Some((conan_proxy, conan_path)) = get_conan_proxy() {
        let ok = verify_proxy_url(&conan_proxy).await;
        if !ok {
            root_issues.push(format!(
                "Conan C/C++ proxy `{}` in {} is dead! (conan install/download will fail with proxy error)",
                conan_proxy,
                conan_path.display()
            ));
        }
        items.push(DiagItem::new(
            "Conan C/C++",
            format!(
                "{} proxy",
                conan_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("config")
            ),
            conan_proxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (conan install fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Conan C/C++",
            "global.conf / conan.conf proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Scala Build Tool (sbt) Proxy (~/.sbt/sbtopts)
    if let Some((sproxy, spath)) = get_sbt_proxy() {
        let ok = verify_proxy_url(&sproxy).await;
        if !ok {
            root_issues.push(format!(
                "Scala Build Tool (sbt) proxy `{}` in {} is dead! (sbt compile/plugin downloads will fail)",
                sproxy,
                spath.display()
            ));
        }
        items.push(DiagItem::new(
            "Scala (sbt)",
            "~/.sbt/sbtopts proxy",
            sproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (sbt build fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Scala (sbt)",
            "~/.sbt/sbtopts proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Flutter / Dart Environment & Mirrors
    let flutter_vars = [
        ("PUB_HOSTED_URL", "Dart/Flutter package repository mirror"),
        ("FLUTTER_STORAGE_BASE_URL", "Flutter SDK download mirror"),
    ];
    let mut flutter_found = false;
    for (var, desc) in flutter_vars {
        if let Ok(val) = std::env::var(var) {
            let trim = val.trim();
            if !trim.is_empty() {
                flutter_found = true;
                let ok = verify_proxy_url(trim).await;
                if !ok {
                    root_issues.push(format!(
                        "Flutter/Dart environment variable `{}` points to unreachable mirror `{}`! (flutter pub get or engine download will fail)",
                        var, trim
                    ));
                }
                items.push(DiagItem::new(
                    "Flutter / Dart",
                    format!("{} ({})", var, desc),
                    trim,
                    if ok {
                        "● Reachable Mirror".green().to_string()
                    } else {
                        "▲ UNREACHABLE MIRROR! (flutter fails)"
                            .red()
                            .bold()
                            .to_string()
                    },
                ));
            }
        }
    }
    if !flutter_found {
        items.push(DiagItem::new(
            "Flutter / Dart",
            "PUB_HOSTED_URL & FLUTTER_STORAGE_BASE_URL",
            "(not set)".dimmed().to_string(),
            "○ Direct (pub.dev & googleapis.com)".dimmed().to_string(),
        ));
    }

    // Swift Package Manager & CocoaPods Mirrors
    let swift_vars = [
        ("CP_REPO_DIR_URL", "CocoaPods spec repository mirror"),
        ("SWIFT_PACKAGE_REGISTRY_URL", "Swift Package Manager registry mirror"),
        ("SWIFT_PACKAGE_COLLECTION_URL", "Swift Package collection URL"),
    ];
    let mut swift_found = false;
    for (var, desc) in swift_vars {
        if let Ok(val) = std::env::var(var) {
            let trim = val.trim();
            if !trim.is_empty() {
                swift_found = true;
                let ok = verify_proxy_url(trim).await;
                if !ok {
                    root_issues.push(format!(
                        "Swift/CocoaPods mirror variable `{}` points to unreachable mirror `{}`! (pod install or swift package resolve will fail)",
                        var, trim
                    ));
                }
                items.push(DiagItem::new(
                    "Swift / CocoaPods",
                    format!("{} ({})", var, desc),
                    trim,
                    if ok {
                        "● Reachable Mirror".green().to_string()
                    } else {
                        "▲ UNREACHABLE MIRROR! (pod/swift fails)"
                            .red()
                            .bold()
                            .to_string()
                    },
                ));
            }
        }
    }
    if !swift_found {
        items.push(DiagItem::new(
            "Swift / CocoaPods",
            "CP_REPO_DIR_URL & SWIFT_PACKAGE_REGISTRY_URL",
            "(not set)".dimmed().to_string(),
            "○ Direct (cdn.cocoapods.org & GitHub)".dimmed().to_string(),
        ));
    }

    // Swift Package Manager Mirror Config (~/.swiftpm/configuration/mirrors.json)
    if let Some(mpath) = get_swiftpm_mirrors_path() {
        if let Ok(content) = std::fs::read_to_string(&mpath) {
            let mirrors = parse_swiftpm_mirrors(&content);
            if mirrors.is_empty() {
                items.push(DiagItem::new(
                    "SwiftPM Mirrors",
                    "~/.swiftpm/configuration/mirrors.json",
                    "(empty)".dimmed().to_string(),
                    "○ Direct Official Repositories".dimmed().to_string(),
                ));
            } else {
                for mirror in mirrors {
                    let ok = verify_proxy_url(&mirror).await;
                    if !ok {
                        root_issues.push(format!(
                            "SwiftPM mirror `{}` in mirrors.json is unreachable! (swift package resolve will fail)",
                            mirror
                        ));
                    }
                    items.push(DiagItem::new(
                        "SwiftPM Mirrors",
                        "mirrors.json mirror",
                        mirror,
                        if ok {
                            "● Reachable Mirror".green().to_string()
                        } else {
                            "▲ UNREACHABLE MIRROR! (swift fails)"
                                .red()
                                .bold()
                                .to_string()
                        },
                    ));
                }
            }
        }
    }

    // Nix Package Manager Proxy & Binary Caches (~/.config/nix/nix.conf)
    if let Some((nproxy, npath)) = get_nix_proxy() {
        let ok = verify_proxy_url(&nproxy).await;
        if !ok {
            root_issues.push(format!(
                "Nix package manager http-proxy `{}` in {} is dead! (nix-shell / nix build will stall or fail)",
                nproxy,
                npath.display()
            ));
        }
        items.push(DiagItem::new(
            "Nix (nix.conf)",
            "nix.conf http-proxy",
            nproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (nix build fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Nix (nix.conf)",
            "nix.conf http-proxy",
            "(not set)".dimmed().to_string(),
            "○ Direct or Shell".dimmed().to_string(),
        ));
    }

    if let Some(npath) = get_nix_config_path() {
        if let Ok(content) = std::fs::read_to_string(&npath) {
            let subs = parse_nix_substituters(&content);
            if !subs.is_empty() {
                for sub in subs {
                    if sub == "https://cache.nixos.org" {
                        continue;
                    }
                    let ok = verify_proxy_url(&sub).await;
                    if !ok {
                        root_issues.push(format!(
                            "Nix substituter mirror `{}` in {} is unreachable! (nix-shell / nix build will stall)",
                            sub,
                            npath.display()
                        ));
                    }
                    items.push(DiagItem::new(
                        "Nix Substituter",
                        "nix.conf binary cache",
                        sub,
                        if ok {
                            "● Reachable Cache Mirror".green().to_string()
                        } else {
                            "▲ UNREACHABLE CACHE! (nix build stalls)"
                                .red()
                                .bold()
                                .to_string()
                        },
                    ));
                }
            }
        }
    }

    // Kubernetes (kubectl) Cluster Proxy Inspection (~/.kube/config)
    if let Some(kpath) = get_kubeconfig_path() {
        if let Ok(content) = std::fs::read_to_string(&kpath) {
            let kube_proxies = parse_kubeconfig_proxies(&content);
            if kube_proxies.is_empty() {
                items.push(DiagItem::new(
                    "Kubernetes (kubectl)",
                    "~/.kube/config cluster proxy-url",
                    "(not set)".dimmed().to_string(),
                    "○ Direct cluster connection".dimmed().to_string(),
                ));
            } else {
                for (cluster, proxy_url) in kube_proxies {
                    let ok = verify_proxy_url(&proxy_url).await;
                    if !ok {
                        root_issues.push(format!(
                            "Kubernetes kubeconfig cluster `{}` has dead proxy-url `{}`! (kubectl & k9s will hang/timeout)",
                            cluster, proxy_url
                        ));
                    }
                    items.push(DiagItem::new(
                        "Kubernetes (kubectl)",
                        format!("cluster `{}`", cluster),
                        proxy_url,
                        if ok {
                            "● Reachable Proxy".green().to_string()
                        } else {
                            "▲ DEAD PROXY PORT! (kubectl hangs)"
                                .red()
                                .bold()
                                .to_string()
                        },
                    ));
                }
            }
        } else {
            items.push(DiagItem::new(
                "Kubernetes (kubectl)",
                "~/.kube/config cluster proxy-url",
                "(not set)".dimmed().to_string(),
                "○ Direct cluster connection".dimmed().to_string(),
            ));
        }
    } else {
        items.push(DiagItem::new(
            "Kubernetes (kubectl)",
            "~/.kube/config cluster proxy-url",
            "(not set)".dimmed().to_string(),
            "○ Direct cluster connection".dimmed().to_string(),
        ));
    }

    // Rustup / Cargo Toolchain Mirrors
    let rustup_vars = [
        ("RUSTUP_DIST_SERVER", "Rust toolchain download server mirror"),
        ("RUSTUP_UPDATE_ROOT", "Rustup installer & update mirror"),
    ];
    let mut rustup_found = false;
    for (var, desc) in rustup_vars {
        if let Ok(val) = std::env::var(var) {
            let trim = val.trim();
            if !trim.is_empty() {
                rustup_found = true;
                let ok = verify_proxy_url(trim).await;
                if !ok {
                    root_issues.push(format!(
                        "Rustup mirror variable `{}` points to unreachable mirror `{}`! (rustup toolchain install or update will fail)",
                        var, trim
                    ));
                }
                items.push(DiagItem::new(
                    "Rustup / Cargo",
                    format!("{} ({})", var, desc),
                    trim,
                    if ok {
                        "● Reachable Mirror".green().to_string()
                    } else {
                        "▲ UNREACHABLE MIRROR! (rustup update fails)"
                            .red()
                            .bold()
                            .to_string()
                    },
                ));
            }
        }
    }
    if !rustup_found {
        items.push(DiagItem::new(
            "Rustup / Cargo",
            "RUSTUP_DIST_SERVER & RUSTUP_UPDATE_ROOT",
            "(default official static.rust-lang.org)".dimmed().to_string(),
            "○ Direct Official Server".dimmed().to_string(),
        ));
    }

    // Julia Package Manager (Pkg) Mirror
    if let Ok(val) = std::env::var("JULIA_PKG_SERVER") {
        let trim = val.trim();
        if !trim.is_empty() {
            let ok = verify_proxy_url(trim).await;
            if !ok {
                root_issues.push(format!(
                    "Julia package mirror `JULIA_PKG_SERVER` points to unreachable mirror `{}`! (Pkg.add/update will fail)",
                    trim
                ));
            }
            items.push(DiagItem::new(
                "Julia (Pkg)",
                "JULIA_PKG_SERVER mirror",
                trim.to_string(),
                if ok {
                    "● Reachable Mirror".green().to_string()
                } else {
                    "▲ UNREACHABLE MIRROR! (julia Pkg fails)"
                        .red()
                        .bold()
                        .to_string()
                },
            ));
        } else {
            items.push(DiagItem::new(
                "Julia (Pkg)",
                "JULIA_PKG_SERVER mirror",
                "(default official pkg.julialang.org)".dimmed().to_string(),
                "○ Direct Official Server".dimmed().to_string(),
            ));
        }
    } else {
        items.push(DiagItem::new(
            "Julia (Pkg)",
            "JULIA_PKG_SERVER mirror",
            "(default official pkg.julialang.org)".dimmed().to_string(),
            "○ Direct Official Server".dimmed().to_string(),
        ));
    }

    // Julia Startup Script Proxy (~/.julia/config/startup.jl)
    if let Some((jproxy, jpath)) = get_julia_startup_proxy() {
        let ok = verify_proxy_url(&jproxy).await;
        if !ok {
            root_issues.push(format!(
                "Julia startup proxy `{}` in {} is dead! (julia Pkg and network downloads will fail)",
                jproxy,
                jpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Julia (startup.jl)",
            "~/.julia/config/startup.jl proxy",
            jproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (julia Pkg fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Julia (startup.jl)",
            "~/.julia/config/startup.jl proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Deno Environment Proxy
    let deno_vars = [
        ("DENO_HTTP_PROXY", "Deno HTTP Proxy"),
        ("DENO_HTTPS_PROXY", "Deno HTTPS Proxy"),
    ];
    let mut deno_found = false;
    for (var, desc) in deno_vars {
        if let Ok(val) = std::env::var(var) {
            let trim = val.trim();
            if !trim.is_empty() {
                deno_found = true;
                let ok = verify_proxy_url(trim).await;
                if !ok {
                    root_issues.push(format!(
                        "Deno proxy variable `{}` points to dead proxy `{}`! (deno run/install will fail)",
                        var, trim
                    ));
                }
                items.push(DiagItem::new(
                    "Deno",
                    format!("{} ({})", var, desc),
                    trim,
                    if ok {
                        "● Reachable Proxy".green().to_string()
                    } else {
                        "▲ DEAD PROXY PORT! (deno fails)"
                            .red()
                            .bold()
                            .to_string()
                    },
                ));
            }
        }
    }
    if !deno_found {
        items.push(DiagItem::new(
            "Deno",
            "DENO_HTTP_PROXY / DENO_HTTPS_PROXY",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Haskell Cabal Proxy (~/.cabal/config)
    if let Some((cproxy, cpath)) = get_cabal_proxy() {
        let ok = verify_proxy_url(&cproxy).await;
        if !ok {
            root_issues.push(format!(
                "Haskell Cabal proxy `{}` in {} is dead! (cabal update / cabal install will fail)",
                cproxy,
                cpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Haskell Cabal",
            "~/.cabal/config proxy",
            cproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (cabal fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Haskell Cabal",
            "~/.cabal/config proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Haskell Stack Proxy (~/.stack/config.yaml)
    if let Some((sproxy, spath)) = get_stack_proxy() {
        let ok = verify_proxy_url(&sproxy).await;
        if !ok {
            root_issues.push(format!(
                "Haskell Stack proxy `{}` in {} is dead! (stack setup / stack build will fail)",
                sproxy,
                spath.display()
            ));
        }
        items.push(DiagItem::new(
            "Haskell Stack",
            "~/.stack/config.yaml proxy",
            sproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (stack fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Haskell Stack",
            "~/.stack/config.yaml proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Elixir / Erlang / Gleam Hex Proxy (~/.hex/hex.config)
    if let Some((hproxy, hpath)) = get_hex_proxy() {
        let ok = verify_proxy_url(&hproxy).await;
        if !ok {
            root_issues.push(format!(
                "Hex package proxy `{}` in {} is dead! (mix deps.get / rebar3 / gleam deps will fail)",
                hproxy,
                hpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Hex (Elixir/Gleam)",
            "~/.hex/hex.config proxy",
            hproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (mix/gleam fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Hex (Elixir/Gleam)",
            "~/.hex/hex.config proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // R Package Manager Proxy (~/.Renviron) & CRAN Mirrors
    if let Some((rproxy, rpath)) = get_renviron_proxy() {
        let ok = verify_proxy_url(&rproxy).await;
        if !ok {
            root_issues.push(format!(
                "R package manager proxy `{}` in {} is dead! (install.packages() / remotes / devtools will fail)",
                rproxy,
                rpath.display()
            ));
        }
        items.push(DiagItem::new(
            "R (~/.Renviron)",
            "~/.Renviron proxy",
            rproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (install.packages fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "R (~/.Renviron)",
            "~/.Renviron proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    let r_mirror_vars = [
        ("CRAN_MIRROR", "CRAN repository mirror"),
        ("R_CRAN_WEB", "R CRAN web mirror"),
    ];
    for (var, desc) in r_mirror_vars {
        if let Ok(val) = std::env::var(var) {
            let trim = val.trim();
            if !trim.is_empty() {
                let ok = verify_proxy_url(trim).await;
                if !ok {
                    root_issues.push(format!(
                        "R mirror variable `{}` points to unreachable mirror `{}`! (R package downloads will fail)",
                        var, trim
                    ));
                }
                items.push(DiagItem::new(
                    "R (CRAN Mirror)",
                    format!("{} ({})", var, desc),
                    trim,
                    if ok {
                        "● Reachable Mirror".green().to_string()
                    } else {
                        "▲ UNREACHABLE MIRROR! (R fails)"
                            .red()
                            .bold()
                            .to_string()
                    },
                ));
            }
        }
    }

    // OCaml OPAM Proxy (~/.opam/config)
    if let Some((oproxy, opath)) = get_opam_proxy() {
        let ok = verify_proxy_url(&oproxy).await;
        if !ok {
            root_issues.push(format!(
                "OCaml OPAM proxy `{}` in {} is dead! (opam update / opam install will fail)",
                oproxy,
                opath.display()
            ));
        }
        items.push(DiagItem::new(
            "OCaml OPAM",
            "~/.opam/config proxy",
            oproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (opam fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "OCaml OPAM",
            "~/.opam/config proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Terraform & OpenTofu Provider Network Mirrors (~/.terraformrc / ~/.tofurc)
    let tf_paths = get_terraform_config_paths();
    let mut tf_mirrors_found = false;
    for path in &tf_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            let mirrors = parse_terraform_network_mirrors(&content);
            for mirror in mirrors {
                tf_mirrors_found = true;
                let ok = verify_proxy_url(&mirror).await;
                if !ok {
                    root_issues.push(format!(
                        "Terraform/OpenTofu network mirror `{}` in {} is unreachable! (terraform init will fail)",
                        mirror,
                        path.display()
                    ));
                }
                items.push(DiagItem::new(
                    "Terraform / OpenTofu",
                    format!("Mirror ({})", path.file_name().unwrap_or_default().to_string_lossy()),
                    mirror,
                    if ok {
                        "● Reachable Mirror".green().to_string()
                    } else {
                        "▲ UNREACHABLE MIRROR! (init fails)"
                            .red()
                            .bold()
                            .to_string()
                    },
                ));
            }
        }
    }
    if !tf_mirrors_found {
        items.push(DiagItem::new(
            "Terraform / OpenTofu",
            "~/.terraformrc & ~/.tofurc network_mirror",
            "(not set)".dimmed().to_string(),
            "○ Direct (registry.terraform.io)".dimmed().to_string(),
        ));
    }

    // Clojure Leiningen Proxy (~/.lein/profiles.clj)
    if let Some((lproxy, lpath)) = get_lein_proxy() {
        let ok = verify_proxy_url(&lproxy).await;
        if !ok {
            root_issues.push(format!(
                "Clojure Leiningen proxy `{}` in {} is dead! (lein deps / lein new will fail)",
                lproxy,
                lpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Clojure (Leiningen)",
            "~/.lein/profiles.clj proxy",
            lproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (lein fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Clojure (Leiningen)",
            "~/.lein/profiles.clj proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Perl CPAN Proxy (~/.cpan/CPAN/MyConfig.pm)
    if let Some((cproxy, cpath)) = get_cpan_proxy() {
        let ok = verify_proxy_url(&cproxy).await;
        if !ok {
            root_issues.push(format!(
                "Perl CPAN proxy `{}` in {} is dead! (cpan / cpanm install will fail)",
                cproxy,
                cpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Perl (CPAN)",
            "~/.cpan/CPAN/MyConfig.pm proxy",
            cproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (cpan fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Perl (CPAN)",
            "~/.cpan/CPAN/MyConfig.pm proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Lua (LuaRocks) Proxy (~/.luarocks/config.lua)
    if let Some((lproxy, lpath)) = get_luarocks_proxy() {
        let ok = verify_proxy_url(&lproxy).await;
        if !ok {
            root_issues.push(format!(
                "LuaRocks proxy `{}` in {} is dead! (luarocks install will fail)",
                lproxy,
                lpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Lua (LuaRocks)",
            "~/.luarocks/config.lua proxy",
            lproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (luarocks fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Lua (LuaRocks)",
            "~/.luarocks/config.lua proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Nim (Nimble) Proxy (~/.config/nimble/nimble.ini)
    if let Some((nproxy, npath)) = get_nimble_proxy() {
        let ok = verify_proxy_url(&nproxy).await;
        if !ok {
            root_issues.push(format!(
                "Nimble proxy `{}` in {} is dead! (nimble install will fail)",
                nproxy,
                npath.display()
            ));
        }
        items.push(DiagItem::new(
            "Nim (Nimble)",
            "~/.config/nimble/nimble.ini proxy",
            nproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (nimble fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Nim (Nimble)",
            "~/.config/nimble/nimble.ini proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Erlang (Rebar3) Proxy (~/.config/rebar3/rebar.config)
    if let Some((rproxy, rpath)) = get_rebar3_proxy() {
        let ok = verify_proxy_url(&rproxy).await;
        if !ok {
            root_issues.push(format!(
                "Rebar3 proxy `{}` in {} is dead! (rebar3 compile/update will fail)",
                rproxy,
                rpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Erlang (Rebar3)",
            "~/.config/rebar3/rebar.config proxy",
            rproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (rebar3 fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Erlang (Rebar3)",
            "~/.config/rebar3/rebar.config proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Scoop (Windows Package Manager) Proxy (~/.config/scoop/config.json)
    if let Some((sproxy, spath)) = get_scoop_proxy() {
        let ok = verify_proxy_url(&sproxy).await;
        if !ok {
            root_issues.push(format!(
                "Scoop proxy `{}` in {} is dead! (scoop install/update will fail)",
                sproxy,
                spath.display()
            ));
        }
        items.push(DiagItem::new(
            "Scoop",
            "~/.config/scoop/config.json proxy",
            sproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (scoop fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Scoop",
            "~/.config/scoop/config.json proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Ruby (Bundler) Proxy (~/.bundle/config)
    if let Some((bproxy, bpath)) = get_bundler_proxy() {
        let ok = verify_proxy_url(&bproxy).await;
        if !ok {
            root_issues.push(format!(
                "Bundler proxy `{}` in {} is dead! (bundle install will fail)",
                bproxy,
                bpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Ruby (Bundler)",
            "~/.bundle/config proxy",
            bproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (bundle fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Ruby (Bundler)",
            "~/.bundle/config proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Google Cloud SDK (gcloud) Proxy (~/.config/gcloud/configurations/config_*)
    if let Some((gproxy, gpath)) = get_gcloud_proxy() {
        let ok = verify_proxy_url(&gproxy).await;
        if !ok {
            root_issues.push(format!(
                "Google Cloud SDK (gcloud) proxy `{}` in {} is dead! (gcloud commands will fail)",
                gproxy,
                gpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Google Cloud SDK (gcloud)",
            format!("{} [proxy]", gpath.display()),
            gproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (gcloud fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Google Cloud SDK (gcloud)",
            "gcloud config [proxy]",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // AWS CLI Proxy (~/.aws/config)
    if let Some((aproxy, apath)) = get_aws_proxy() {
        let ok = verify_proxy_url(&aproxy).await;
        if !ok {
            root_issues.push(format!(
                "AWS CLI proxy `{}` in {} is dead! (aws commands will fail)",
                aproxy,
                apath.display()
            ));
        }
        items.push(DiagItem::new(
            "AWS CLI",
            format!("{} proxy", apath.display()),
            aproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (aws fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "AWS CLI",
            "~/.aws/config proxy",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Microsoft Azure CLI (az) Proxy (~/.azure/config)
    if let Some((azproxy, azpath)) = get_azure_proxy() {
        let ok = verify_proxy_url(&azproxy).await;
        if !ok {
            root_issues.push(format!(
                "Azure CLI proxy `{}` in {} is dead! (az commands will fail)",
                azproxy,
                azpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Azure CLI",
            format!("{} [proxy]", azpath.display()),
            azproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (az fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Azure CLI",
            "~/.azure/config [proxy]",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Podman Container Engine Proxy (~/.config/containers/containers.conf)
    if let Some((podproxy, podpath)) = get_podman_proxy() {
        let ok = verify_proxy_url(&podproxy).await;
        if !ok {
            root_issues.push(format!(
                "Podman proxy `{}` in {} is dead! (podman pull / build will fail)",
                podproxy,
                podpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Podman",
            format!("{} [engine]", podpath.display()),
            podproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (podman fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Podman",
            "containers.conf [engine]",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Vagrant Development Environments Proxy (~/.vagrant.d/Vagrantfile)
    if let Some((vagproxy, vagpath)) = get_vagrant_proxy() {
        let ok = verify_proxy_url(&vagproxy).await;
        if !ok {
            root_issues.push(format!(
                "Vagrant proxy `{}` in {} is dead! (vagrant box add / up will fail)",
                vagproxy,
                vagpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Vagrant",
            format!("{} [config.proxy]", vagpath.display()),
            vagproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (vagrant fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Vagrant",
            "~/.vagrant.d/Vagrantfile",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Chocolatey Package Manager Proxy (chocolatey.config)
    if let Some((chocoproxy, chocopath)) = get_choco_proxy() {
        let ok = verify_proxy_url(&chocoproxy).await;
        if !ok {
            root_issues.push(format!(
                "Chocolatey proxy `{}` in {} is dead! (choco install / upgrade will fail)",
                chocoproxy,
                chocopath.display()
            ));
        }
        items.push(DiagItem::new(
            "Chocolatey",
            format!("{} [config.proxy]", chocopath.display()),
            chocoproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (choco fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Chocolatey",
            "chocolatey.config",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Colima Container Runtime Proxy (~/.colima/default/colima.yaml)
    if let Some((colimaproxy, colimapath)) = get_colima_proxy() {
        let ok = verify_proxy_url(&colimaproxy).await;
        if !ok {
            root_issues.push(format!(
                "Colima container runtime proxy `{}` in {} is dead! (colima start / pull will fail)",
                colimaproxy,
                colimapath.display()
            ));
        }
        items.push(DiagItem::new(
            "Colima",
            format!("{} [env]", colimapath.display()),
            colimaproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (colima fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Colima",
            "~/.colima/default/colima.yaml [env]",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Minikube Local Kubernetes Cluster Proxy (~/.minikube/config/config.json)
    if let Some((miniproxy, minipath)) = get_minikube_proxy() {
        let ok = verify_proxy_url(&miniproxy).await;
        if !ok {
            root_issues.push(format!(
                "Minikube cluster proxy `{}` in {} is dead! (minikube start / pull will fail)",
                miniproxy,
                minipath.display()
            ));
        }
        items.push(DiagItem::new(
            "Minikube",
            format!("{} [config.json]", minipath.display()),
            miniproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (minikube fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Minikube",
            "~/.minikube/config/config.json",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Spack Scientific & HPC Package Manager Proxy (~/.spack/config.yaml)
    if let Some((spackproxy, spackpath)) = get_spack_proxy() {
        let ok = verify_proxy_url(&spackproxy).await;
        if !ok {
            root_issues.push(format!(
                "Spack package manager proxy `{}` in {} is dead! (spack install / fetch will fail)",
                spackproxy,
                spackpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Spack",
            format!("{} [config.yaml]", spackpath.display()),
            spackproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (spack fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Spack",
            "~/.spack/config.yaml",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // MacPorts Package Manager Proxy (~/.macports/macports.conf / /opt/local/etc/macports/macports.conf)
    if let Some((mpproxy, mppath)) = get_macports_proxy() {
        let ok = verify_proxy_url(&mpproxy).await;
        if !ok {
            root_issues.push(format!(
                "MacPorts package manager proxy `{}` in {} is dead! (port install / sync will fail)",
                mpproxy,
                mppath.display()
            ));
        }
        items.push(DiagItem::new(
            "MacPorts",
            format!("{} [macports.conf]", mppath.display()),
            mpproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (port fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "MacPorts",
            "~/.macports/macports.conf",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // APT (Debian / Ubuntu / WSL / Containers) Package Manager Proxy (/etc/apt/apt.conf.d/* / ~/.apt/apt.conf)
    if let Some((aptproxy, aptpath)) = get_apt_proxy() {
        let ok = verify_proxy_url(&aptproxy).await;
        if !ok {
            root_issues.push(format!(
                "APT package manager proxy `{}` in {} is dead! (apt update / install will fail)",
                aptproxy,
                aptpath.display()
            ));
        }
        items.push(DiagItem::new(
            "APT",
            format!("{}", aptpath.display()),
            aptproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (apt fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "APT",
            "/etc/apt/apt.conf.d/* & ~/.apt/apt.conf",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // DNF & YUM (Fedora / RHEL / CentOS / Rocky / Alma / Amazon Linux) Package Manager Proxy (/etc/dnf/dnf.conf / /etc/yum.conf)
    if let Some((dnfproxy, dnfpath)) = get_dnf_proxy() {
        let ok = verify_proxy_url(&dnfproxy).await;
        if !ok {
            root_issues.push(format!(
                "DNF/YUM package manager proxy `{}` in {} is dead! (dnf/yum update/install will fail)",
                dnfproxy,
                dnfpath.display()
            ));
        }
        items.push(DiagItem::new(
            "DNF / YUM",
            format!("{}", dnfpath.display()),
            dnfproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (dnf fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "DNF / YUM",
            "/etc/dnf/dnf.conf & ~/.config/dnf/dnf.conf",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Zypper (openSUSE / SLES) Package Manager Proxy (/etc/zypp/zypp.conf / ~/.zypp/zypp.conf)
    if let Some((zyppproxy, zypppath)) = get_zypper_proxy() {
        let ok = verify_proxy_url(&zyppproxy).await;
        if !ok {
            root_issues.push(format!(
                "Zypper package manager proxy `{}` in {} is dead! (zypper update/install will fail)",
                zyppproxy,
                zypppath.display()
            ));
        }
        items.push(DiagItem::new(
            "Zypper",
            format!("{}", zypppath.display()),
            zyppproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (zypper fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Zypper",
            "/etc/zypp/zypp.conf & ~/.zypp/zypp.conf",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Pacman (Arch Linux / Manjaro / SteamOS / MSYS2) Package Manager Proxy (/etc/pacman.conf)
    if let Some((pacproxy, pacpath)) = get_pacman_proxy() {
        let ok = verify_proxy_url(&pacproxy).await;
        if !ok {
            root_issues.push(format!(
                "Pacman package manager proxy `{}` in {} is dead! (pacman -Syu will fail)",
                pacproxy,
                pacpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Pacman",
            format!("{}", pacpath.display()),
            pacproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (pacman fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Pacman",
            "/etc/pacman.conf & ~/.config/pacman/pacman.conf",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // FreeBSD pkg Package Manager Proxy (/usr/local/etc/pkg.conf / /etc/pkg.conf)
    if let Some((pkgproxy, pkgpath)) = get_pkg_proxy() {
        let ok = verify_proxy_url(&pkgproxy).await;
        if !ok {
            root_issues.push(format!(
                "FreeBSD pkg package manager proxy `{}` in {} is dead! (pkg install/update will fail)",
                pkgproxy,
                pkgpath.display()
            ));
        }
        items.push(DiagItem::new(
            "FreeBSD pkg",
            format!("{}", pkgpath.display()),
            pkgproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (pkg fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "FreeBSD pkg",
            "/usr/local/etc/pkg.conf & /etc/pkg.conf",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Portage (Gentoo / ChromeOS / Funtoo) Package Manager Proxy (/etc/portage/make.conf)
    if let Some((portproxy, portpath)) = get_portage_proxy() {
        let ok = verify_proxy_url(&portproxy).await;
        if !ok {
            root_issues.push(format!(
                "Portage package manager proxy `{}` in {} is dead! (emerge sync and builds will fail)",
                portproxy,
                portpath.display()
            ));
        }
        items.push(DiagItem::new(
            "Portage",
            format!("{}", portpath.display()),
            portproxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (emerge fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Portage",
            "/etc/portage/make.conf & ~/.config/portage/make.conf",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Docker Daemon Engine Proxy (/etc/docker/daemon.json / ~/.docker/daemon.json)
    if let Some((daemon_proxy, daemon_path)) = get_docker_daemon_proxy() {
        let ok = verify_proxy_url(&daemon_proxy).await;
        if !ok {
            root_issues.push(format!(
                "Docker Daemon proxy `{}` in {} is dead! (docker pull and image builds will fail)",
                daemon_proxy,
                daemon_path.display()
            ));
        }
        items.push(DiagItem::new(
            "Docker Daemon",
            format!("{}", daemon_path.display()),
            daemon_proxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (docker pull fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Docker Daemon",
            "/etc/docker/daemon.json & ~/.docker/daemon.json",
            "(not set)".dimmed().to_string(),
            "○ Inherits Host Network or Direct".dimmed().to_string(),
        ));
    }

    // Flatpak (OSTree / Linux Desktop Application Manager) Proxy (~/.local/share/flatpak/repo/config / /var/lib/flatpak/repo/config)
    if let Some((flatpak_proxy, flatpak_path)) = get_flatpak_proxy() {
        let ok = verify_proxy_url(&flatpak_proxy).await;
        if !ok {
            root_issues.push(format!(
                "Flatpak application package manager proxy `{}` in {} is dead! (flatpak install/update will fail)",
                flatpak_proxy,
                flatpak_path.display()
            ));
        }
        items.push(DiagItem::new(
            "Flatpak",
            format!("{}", flatpak_path.display()),
            flatpak_proxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (flatpak fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Flatpak",
            "~/.local/share/flatpak/repo/config & /var/lib/flatpak/repo/config",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Snap (Canonical Snapcraft / Linux Application Package Manager) Proxy (/etc/systemd/system/snapd.service.d/http-proxy.conf / snap set system proxy.http)
    if let Some((snap_proxy, snap_path)) = get_snap_proxy() {
        let ok = verify_proxy_url(&snap_proxy).await;
        if !ok {
            root_issues.push(format!(
                "Snap application package manager proxy `{}` in {} is dead! (snap install/refresh will fail)",
                snap_proxy,
                snap_path.display()
            ));
        }
        items.push(DiagItem::new(
            "Snap",
            format!("{}", snap_path.display()),
            snap_proxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (snap fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Snap",
            "/etc/systemd/system/snapd.service.d/http-proxy.conf & snap set system",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Ollama (Local AI Model Runner Daemon) Proxy (/etc/systemd/system/ollama.service.d/http-proxy.conf / ~/.ollama/environment / plist)
    if let Some((ollama_proxy, ollama_path)) = get_ollama_proxy() {
        let ok = verify_proxy_url(&ollama_proxy).await;
        if !ok {
            root_issues.push(format!(
                "Ollama local AI runner daemon proxy `{}` in {} is dead! (ollama pull/run will fail)",
                ollama_proxy,
                ollama_path.display()
            ));
        }
        items.push(DiagItem::new(
            "Ollama Daemon",
            format!("{}", ollama_path.display()),
            ollama_proxy,
            if ok {
                "● Reachable Proxy".green().to_string()
            } else {
                "▲ DEAD PROXY PORT! (ollama fails)"
                    .red()
                    .bold()
                    .to_string()
            },
        ));
    } else {
        items.push(DiagItem::new(
            "Ollama Daemon",
            "/etc/systemd/system/ollama.service.d/http-proxy.conf & ~/.ollama/environment",
            "(not set)".dimmed().to_string(),
            "○ Inherits Shell or Direct".dimmed().to_string(),
        ));
    }

    // Alpine Linux Package Manager (apk) Repositories & Mirrors (/etc/apk/repositories)
    let apk_paths = get_apk_repositories_paths();
    if !apk_paths.is_empty() {
        for apath in apk_paths {
            if let Ok(content) = std::fs::read_to_string(&apath) {
                let repos = parse_apk_repositories(&content);
                for repo in repos {
                    let ok = verify_proxy_url(&repo).await;
                    if !ok {
                        root_issues.push(format!(
                            "Alpine Linux apk repository mirror `{}` in {} is unreachable! (apk add/update will fail)",
                            repo,
                            apath.display()
                        ));
                    }
                    items.push(DiagItem::new(
                        "Alpine apk",
                        format!("{}", apath.display()),
                        repo,
                        if ok {
                            "● Reachable Repository".green().to_string()
                        } else {
                            "▲ UNREACHABLE REPOSITORY! (apk fails)"
                                .red()
                                .bold()
                                .to_string()
                        },
                    ));
                }
            }
        }
    } else {
        items.push(DiagItem::new(
            "Alpine apk",
            "/etc/apk/repositories",
            "(not set)".dimmed().to_string(),
            "○ Official Mirrors / Non-Alpine OS".dimmed().to_string(),
        ));
    }

    // C/C++ vcpkg Asset Caching & Binary Mirrors (X_VCPKG_ASSET_SOURCES & VCPKG_BINARY_SOURCES)
    let vcpkg_vars = [
        ("X_VCPKG_ASSET_SOURCES", "vcpkg asset caching mirror source"),
        ("VCPKG_BINARY_SOURCES", "vcpkg binary caching mirror source"),
    ];
    let mut vcpkg_found = false;
    for (var, desc) in vcpkg_vars {
        if let Ok(val) = std::env::var(var) {
            let trim = val.trim();
            if !trim.is_empty() {
                vcpkg_found = true;
                if let Some(url) = parse_vcpkg_asset_url(trim) {
                    let ok = verify_proxy_url(&url).await;
                    if !ok {
                        root_issues.push(format!(
                            "vcpkg mirror URL `{}` in `{}` is unreachable! (vcpkg install will fail)",
                            url, var
                        ));
                    }
                    items.push(DiagItem::new(
                        "C/C++ vcpkg",
                        format!("{} ({})", var, desc),
                        url,
                        if ok {
                            "● Reachable Mirror".green().to_string()
                        } else {
                            "▲ UNREACHABLE MIRROR! (vcpkg fails)"
                                .red()
                                .bold()
                                .to_string()
                        },
                    ));
                } else {
                    items.push(DiagItem::new(
                        "C/C++ vcpkg",
                        format!("{} ({})", var, desc),
                        trim.to_string(),
                        "● Custom Configured".green().to_string(),
                    ));
                }
            }
        }
    }
    if !vcpkg_found {
        items.push(DiagItem::new(
            "C/C++ vcpkg",
            "X_VCPKG_ASSET_SOURCES & VCPKG_BINARY_SOURCES",
            "(not set)".dimmed().to_string(),
            "○ Direct / Official Mirrors".dimmed().to_string(),
        ));
    }

    // C/C++ vcpkg Configuration Registries (vcpkg-configuration.json)
    let vcpkg_paths = get_vcpkg_configuration_paths();
    if !vcpkg_paths.is_empty() {
        for vpath in vcpkg_paths {
            if let Ok(content) = std::fs::read_to_string(&vpath) {
                let registries = parse_vcpkg_configuration_registries(&content);
                for reg in registries {
                    let ok = verify_proxy_url(&reg).await;
                    if !ok {
                        root_issues.push(format!(
                            "vcpkg configuration registry `{}` in {} is unreachable! (vcpkg install will fail)",
                            reg,
                            vpath.display()
                        ));
                    }
                    items.push(DiagItem::new(
                        "C/C++ vcpkg",
                        format!("{} registry", vpath.display()),
                        reg,
                        if ok {
                            "● Reachable Registry".green().to_string()
                        } else {
                            "▲ UNREACHABLE REGISTRY! (vcpkg fails)"
                                .red()
                                .bold()
                                .to_string()
                        },
                    ));
                }
            }
        }
    }

    // Kubernetes Helm Chart Repositories (~/.config/helm/repositories.yaml)
    let helm_paths = get_helm_repositories_paths();
    if !helm_paths.is_empty() {
        for hpath in helm_paths {
            if let Ok(content) = std::fs::read_to_string(&hpath) {
                let repos = parse_helm_repositories(&content);
                for repo in repos {
                    let ok = verify_proxy_url(&repo).await;
                    if !ok {
                        root_issues.push(format!(
                            "Helm chart repository `{}` in {} is unreachable! (helm repo update will fail)",
                            repo,
                            hpath.display()
                        ));
                    }
                    items.push(DiagItem::new(
                        "Kubernetes Helm",
                        format!("{} repo", hpath.display()),
                        repo,
                        if ok {
                            "● Reachable Chart Repo".green().to_string()
                        } else {
                            "▲ UNREACHABLE REPO! (helm fails)"
                                .red()
                                .bold()
                                .to_string()
                        },
                    ));
                }
            }
        }
    }

    // 14. AI Developer Endpoints (Base URLs)
    let ai_vars = [
        "OPENAI_BASE_URL",
        "OPENAI_API_BASE",
        "ANTHROPIC_BASE_URL",
        "GEMINI_BASE_URL",
        "GOOGLE_AI_BASE_URL",
        "DEEPSEEK_BASE_URL",
        "DEEPSEEK_API_BASE",
        "GROQ_BASE_URL",
        "OPENROUTER_BASE_URL",
        "MISTRAL_BASE_URL",
        "PERPLEXITY_BASE_URL",
        "COHERE_BASE_URL",
        "XAI_BASE_URL",
        "SILICONFLOW_BASE_URL",
        "TOGETHER_BASE_URL",
        "MOONSHOT_BASE_URL",
        "DASHSCOPE_BASE_URL",
        "DEEPINFRA_BASE_URL",
        "CEREBRAS_BASE_URL",
        "FIREWORKS_BASE_URL",
        "ZHIPU_BASE_URL",
        "MINIMAX_BASE_URL",
        "COPILOT_PROXY",
        "SAMBANOVA_BASE_URL",
        "REPLICATE_API_BASE",
        "QIANFAN_BASE_URL",
        "DEEPL_BASE_URL",
        "STEPFUN_BASE_URL",
        "LINGYIWANWU_BASE_URL",
        "YI_BASE_URL",
        "VOYAGE_BASE_URL",
        "NOVITA_BASE_URL",
        "HYPERBOLIC_BASE_URL",
        "LAMBDA_BASE_URL",
        "UPSTAGE_BASE_URL",
        "AI21_BASE_URL",
        "BAICHUAN_BASE_URL",
        "SENSENOVA_BASE_URL",
        "INTERNLM_BASE_URL",
        "HUNYUAN_BASE_URL",
        "BASETEN_BASE_URL",
        "FRIENDLI_BASE_URL",
        "LEPTON_BASE_URL",
        "NEBIUS_BASE_URL",
        "CHUTES_BASE_URL",
        "INFINI_BASE_URL",
        "AIMLAPI_BASE_URL",
        "CRUSOE_BASE_URL",
        "MODAL_BASE_URL",
        "NVIDIA_BASE_URL",
        "ARK_BASE_URL",
        "VOLC_BASE_URL",
        "RUNPOD_BASE_URL",
        "FAL_BASE_URL",
        "HELICONE_BASE_URL",
        "CLAUDE_BASE_URL",
        "ELEVENLABS_BASE_URL",
        "PORTKEY_BASE_URL",
        "LANGFUSE_BASE_URL",
        "CARTESIA_BASE_URL",
        "GLAB_PROXY",
        "OLLAMA_HOST",
        "HF_ENDPOINT",
        "LIVEKIT_URL",
        "DEEPGRAM_BASE_URL",
        "WANDB_BASE_URL",
        "PINECONE_BASE_URL",
        "TAVILY_BASE_URL",
        "EXA_BASE_URL",
        "QDRANT_URL",
        "ASSEMBLYAI_BASE_URL",
        "JINA_BASE_URL",
        "FIRECRAWL_BASE_URL",
        "WEAVIATE_URL",
        "CHROMA_URL",
        "ZILLIZ_BASE_URL",
        "CURSOR_BASE_URL",
        "LANCEDB_URL",
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
            "AI Base URLs (OpenAI/Anthropic/Gemini/DeepSeek/Groq/OpenRouter/Mistral/Perplexity/Cohere/xAI/SiliconFlow/Together/Moonshot/DashScope/DeepInfra/Cerebras/Fireworks/Zhipu/MiniMax/Copilot/SambaNova/Replicate/Qianfan/DeepL/NVIDIA/Doubao/RunPod/Fal/Helicone/Kling/ElevenLabs/Cartesia/Portkey/Langfuse/LiveKit/Deepgram/WandB/Pinecone/Tavily/Exa/Qdrant/AssemblyAI/Ollama/HuggingFace)",
            "(default public endpoints)".dimmed().to_string(),
            "○ Direct Official Endpoints".dimmed().to_string(),
        ));
    }

    // 15. IDE Proxy & SSL Configuration (VS Code, Cursor, Windsurf, VSCodium)
    let ide_configs = get_ide_proxies();
    if ide_configs.is_empty() {
        items.push(DiagItem::new(
            "IDE Proxy",
            "VS Code / Cursor settings.json",
            "(no IDE settings.json found)".dimmed().to_string(),
            "○ Skipped (Direct or not installed)".dimmed().to_string(),
        ));
    } else {
        for ide in ide_configs {
            let label = format!("{} settings.json", ide.ide_name);
            match ide.proxy {
                Some(ref val) => {
                    let ok = verify_proxy_url(val).await;
                    if !ok {
                        root_issues.push(format!(
                            "{} has http.proxy pointing to dead port `{}`! (GitHub Copilot, Cursor AI & extensions will fail)",
                            label, val
                        ));
                    }
                    items.push(DiagItem::new(
                        "IDE Proxy",
                        &label,
                        val,
                        if ok {
                            "● Reachable".green().to_string()
                        } else {
                            "▲ DEAD PORT! (Copilot & extensions fail)"
                                .red()
                                .bold()
                                .to_string()
                        },
                    ));
                }
                None => {
                    items.push(DiagItem::new(
                        "IDE Proxy",
                        &label,
                        "(not set)".dimmed().to_string(),
                        "○ Clean (Inherits Terminal or Direct)".dimmed().to_string(),
                    ));
                }
            }

            if let Some(false) = ide.strict_ssl {
                root_issues.push(format!(
                    "{} has `http.proxyStrictSSL` set to false! Strict TLS verification is disabled.",
                    label
                ));
                items.push(DiagItem::new(
                    "IDE SSL",
                    format!("{} strictSSL", ide.ide_name),
                    "false".yellow().to_string(),
                    "▲ DISABLED! (Security risk / Strict SSL off)"
                        .yellow()
                        .bold()
                        .to_string(),
                ));
            }
        }
    }

    // JetBrains IDE Family Proxy (IntelliJ IDEA, PyCharm, Android Studio, WebStorm, CLion, RustRover)
    let jb_configs = get_jetbrains_proxies();
    if jb_configs.is_empty() {
        items.push(DiagItem::new(
            "JetBrains IDE",
            "JetBrains / Android Studio proxy.settings.xml",
            "(no JetBrains settings found)".dimmed().to_string(),
            "○ Skipped (Direct or not installed)".dimmed().to_string(),
        ));
    } else {
        for jb in jb_configs {
            let label = format!("{} proxy.settings.xml", jb.ide_name);
            if jb.use_proxy {
                if let (Some(ref host), Some(port)) = (&jb.host, jb.port) {
                    let reachable = scanner::check_host_port(host, port).await;
                    let proto = if jb.is_socks { "socks5" } else { "http" };
                    let proxy_str = format!("{}://{}:{}", proto, host, port);
                    if !reachable {
                        root_issues.push(format!(
                            "JetBrains IDE ({}) proxy points to dead port `{}`! (Plugin marketplace, Copilot, Gradle & SDK sync will fail)",
                            jb.ide_name, proxy_str
                        ));
                    }
                    items.push(DiagItem::new(
                        "JetBrains IDE",
                        &label,
                        proxy_str,
                        if reachable {
                            "● Reachable Port".green().to_string()
                        } else {
                            "▲ DEAD PORT! (Plugins & AI fail)".red().bold().to_string()
                        },
                    ));
                } else {
                    items.push(DiagItem::new(
                        "JetBrains IDE",
                        &label,
                        "enabled (no host/port)".yellow().to_string(),
                        "▲ Missing Host/Port".yellow().to_string(),
                    ));
                }
            } else if jb.use_pac {
                if let Some(ref pac) = jb.pac_url {
                    let reachable = verify_proxy_url(pac).await;
                    if !reachable {
                        root_issues.push(format!(
                            "JetBrains IDE ({}) has unreachable PAC URL `{}`!",
                            jb.ide_name, pac
                        ));
                    }
                    items.push(DiagItem::new(
                        "JetBrains IDE",
                        &label,
                        format!("PAC: {}", pac),
                        if reachable {
                            "● Reachable PAC".green().to_string()
                        } else {
                            "▲ UNREACHABLE PAC!".red().bold().to_string()
                        },
                    ));
                } else {
                    items.push(DiagItem::new(
                        "JetBrains IDE",
                        &label,
                        "PAC enabled (default/auto)".dimmed().to_string(),
                        "○ Clean (Auto PAC)".dimmed().to_string(),
                    ));
                }
            } else {
                items.push(DiagItem::new(
                    "JetBrains IDE",
                    &label,
                    "(proxy disabled)".dimmed().to_string(),
                    "○ Direct connection".dimmed().to_string(),
                ));
            }
        }
    }

    // 13. Homebrew Environment & Mirrors
    let brew_vars = [
        "HOMEBREW_BOTTLE_DOMAIN",
        "HOMEBREW_API_DOMAIN",
        "HOMEBREW_BREW_GIT_REMOTE",
        "HOMEBREW_CORE_GIT_REMOTE",
    ];
    for var in brew_vars {
        if let Ok(val) = std::env::var(var) {
            let trim = val.trim();
            if !trim.is_empty() {
                let ok = verify_proxy_url(trim).await;
                if !ok {
                    root_issues.push(format!(
                        "Homebrew mirror variable `{}` points to unreachable mirror `{}`! (brew install / update will fail)",
                        var, trim
                    ));
                }
                items.push(DiagItem::new(
                    "Homebrew",
                    var,
                    trim.to_string(),
                    if ok {
                        "● Reachable Mirror".green().to_string()
                    } else {
                        "▲ UNREACHABLE MIRROR! (brew hangs)"
                            .red()
                            .bold()
                            .to_string()
                    },
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

    if let Ok(val) = std::env::var("NODE_TLS_REJECT_UNAUTHORIZED") {
        if val.trim() == "0" {
            root_issues.push("NODE_TLS_REJECT_UNAUTHORIZED='0' is set in terminal environment! Node.js TLS certificate validation is disabled, creating security risks.".to_string());
            items.push(DiagItem::new(
                "TLS Security",
                "NODE_TLS_REJECT_UNAUTHORIZED",
                "0".yellow().to_string(),
                "▲ DISABLED! (Node TLS security disabled)"
                    .yellow()
                    .bold()
                    .to_string(),
            ));
        }
    }

    // 15. Hosts File Pinned Overrides Inspection
    let hosts_path = get_hosts_file_path();
    let hosts_entries = get_hosts_entries();
    let dev_domains = [
        "github.com",
        "raw.githubusercontent.com",
        "api.github.com",
        "gist.github.com",
        "api.openai.com",
        "api.anthropic.com",
        "api.deepseek.com",
        "huggingface.co",
        "pypi.org",
        "registry.npmjs.org",
        "registry.yarnpkg.com",
        "crates.io",
        "proxy.golang.org",
        "registry.ollama.ai",
        "ghcr.io",
        "objects.githubusercontent.com",
        "api.perplexity.ai",
        "api.mistral.ai",
    ];

    let mut hosts_overrides_found = false;
    for entry in &hosts_entries {
        if dev_domains
            .iter()
            .any(|d| entry.domain.eq_ignore_ascii_case(d))
        {
            hosts_overrides_found = true;
            let ip_str = &entry.ip;
            let is_blocked = ip_str == "0.0.0.0" || ip_str == "127.0.0.1" || ip_str == "::1";
            if is_blocked {
                root_issues.push(format!(
                    "Hosts file ({}) sinkholes/blocks `{}` to `{}`! Remove or comment out this line to allow connection.",
                    hosts_path.display(),
                    entry.domain,
                    ip_str
                ));
                items.push(DiagItem::new(
                    "Hosts File",
                    format!(
                        "{} (in {})",
                        entry.domain,
                        hosts_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("hosts")
                    ),
                    ip_str.clone(),
                    "▲ SINKHOLED / BLOCKED!".red().bold().to_string(),
                ));
            } else {
                let reachable = scanner::check_host_port(ip_str, 443).await;
                if reachable {
                    items.push(DiagItem::new(
                        "Hosts File",
                        format!(
                            "{} (in {})",
                            entry.domain,
                            hosts_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("hosts")
                        ),
                        ip_str.clone(),
                        "● Pinned IP Reachable (Port 443)".green().to_string(),
                    ));
                } else {
                    root_issues.push(format!(
                        "Hosts file ({}) pins `{}` to `{}` which is UNREACHABLE! Git clone or API calls will hang or timeout. Remove this stale entry from your hosts file.",
                        hosts_path.display(),
                        entry.domain,
                        ip_str
                    ));
                    items.push(DiagItem::new(
                        "Hosts File",
                        format!(
                            "{} (in {})",
                            entry.domain,
                            hosts_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("hosts")
                        ),
                        ip_str.clone(),
                        "▲ STALE / DEAD PINNED IP! (Causes timeouts)"
                            .red()
                            .bold()
                            .to_string(),
                    ));
                }
            }
        }
    }
    if !hosts_overrides_found {
        items.push(DiagItem::new(
            "Hosts File",
            format!(
                "{} overrides",
                hosts_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("hosts")
            ),
            "(clean)".dimmed().to_string(),
            "○ No developer domains hijacked".dimmed().to_string(),
        ));
    }

    // 16. DNS Poisoning & Resolution Check
    let dns_targets = [
        ("github.com", 443),
        ("raw.githubusercontent.com", 443),
        ("api.openai.com", 443),
        ("api.anthropic.com", 443),
        ("api.deepseek.com", 443),
        ("pypi.org", 443),
        ("proxy.golang.org", 443),
        ("registry.yarnpkg.com", 443),
        ("api.perplexity.ai", 443),
        ("api.mistral.ai", 443),
        ("api.together.xyz", 443),
        ("api.moonshot.cn", 443),
        ("dashscope.aliyuncs.com", 443),
        ("cran.r-project.org", 443),
        ("opam.ocaml.org", 443),
        ("api.deepgram.com", 443),
        ("api.wandb.ai", 443),
        ("api.pinecone.io", 443),
        ("cloud.livekit.io", 443),
        ("api.tavily.com", 443),
        ("api.exa.ai", 443),
        ("cloud.qdrant.io", 443),
        ("api.assemblyai.com", 443),
        ("clojars.org", 443),
        ("fastapi.metacpan.org", 443),
        ("luarocks.org", 443),
        ("nimble.directory", 443),
        ("repo.hex.pm", 443),
        ("api.perplexity.ai", 443),
        ("r.jina.ai", 443),
        ("api.firecrawl.dev", 443),
        ("api.weaviate.io", 443),
        ("rubygems.org", 443),
        ("api2.cursor.sh", 443),
        ("api.lancedb.com", 443),
        ("registry.ollama.ai", 443),
        ("ghcr.io", 443),
        ("objects.githubusercontent.com", 443),
        ("api.tailscale.com", 443),
        ("api.replit.com", 443),
    ];

    for (domain, port) in dns_targets {
        let lookup_res = lookup_host(format!("{}:{}", domain, port)).await;
        match lookup_res {
            Ok(mut addrs) => {
                if let Some(first) = addrs.next() {
                    let ip = first.ip();
                    let is_poisoned =
                        ip.is_loopback() || ip.is_unspecified() || ip.to_string() == "0.0.0.0";
                    if is_poisoned {
                        root_issues.push(format!(
                            "DNS for {} is poisoned/blocked (Resolved to {})",
                            domain, ip
                        ));
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
        println!(
            "{}",
            "✨ All configurations healthy and aligned! No dead ports detected."
                .green()
                .bold()
        );
    }

    println!();
    Ok(())
}

pub fn get_git_config(key: &str) -> Option<String> {
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
        let port_part: String = val[idx + 1..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(p) = port_part.parse::<u16>() {
            let host = if host_part.is_empty() || host_part == "localhost" {
                "127.0.0.1"
            } else {
                host_part
                    .trim_start_matches("http://")
                    .trim_start_matches("https://")
                    .trim_start_matches("socks5://")
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
        assert_eq!(
            parse_cargo_proxy_toml(content),
            Some("http://127.0.0.1:7890".to_string())
        );

        let empty = r#"
[build]
jobs = 4
"#;
        assert_eq!(parse_cargo_proxy_toml(empty), None);
    }

    #[test]
    fn test_parse_curlrc_proxy() {
        let c1 = "proxy = \"http://127.0.0.1:7890\"\n";
        assert_eq!(
            parse_curlrc_proxy(c1),
            Some("http://127.0.0.1:7890".to_string())
        );

        let c2 = "# some comment\n-x 127.0.0.1:1080\n";
        assert_eq!(parse_curlrc_proxy(c2), Some("127.0.0.1:1080".to_string()));

        let c3 = "--proxy socks5://127.0.0.1:1082\n";
        assert_eq!(
            parse_curlrc_proxy(c3),
            Some("socks5://127.0.0.1:1082".to_string())
        );

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
        assert_eq!(
            parse_pip_config_proxy(c1),
            Some("http://127.0.0.1:7890".to_string())
        );

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
        assert_eq!(
            parse_docker_config_proxy(json_with_proxy),
            Some("http://127.0.0.1:7890".to_string())
        );

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
        assert_eq!(
            parse_wgetrc_proxy(raw),
            Some("http://127.0.0.1:7890/".to_string())
        );

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
        assert_eq!(
            parse_npmrc_proxy(content, "proxy"),
            Some("http://127.0.0.1:7890".to_string())
        );
        assert_eq!(
            parse_npmrc_proxy(content, "https-proxy"),
            Some("http://127.0.0.1:7890".to_string())
        );
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
        assert_eq!(
            parsed,
            vec!["https://goproxy.cn", "https://proxy.golang.org"]
        );

        let direct_only = "direct";
        assert!(parse_goproxy(direct_only).is_empty());
    }

    #[test]
    fn test_parse_yarnrc_proxy() {
        let content = "# yarn config\nproxy \"http://127.0.0.1:7890\"\nhttps-proxy \"http://127.0.0.1:7890\"\n";
        assert_eq!(
            parse_yarnrc_proxy(content),
            Some("http://127.0.0.1:7890".to_string())
        );

        let unquoted = "proxy http://127.0.0.1:1080\n";
        assert_eq!(
            parse_yarnrc_proxy(unquoted),
            Some("http://127.0.0.1:1080".to_string())
        );

        let commented = "# proxy \"http://127.0.0.1:7890\"\n";
        assert_eq!(parse_yarnrc_proxy(commented), None);
    }

    #[test]
    fn test_parse_yarnrc_yml_proxy() {
        let content =
            "httpProxy: \"http://127.0.0.1:7890\"\nhttpsProxy: \"http://127.0.0.1:7890\"\n";
        assert_eq!(
            parse_yarnrc_yml_proxy(content),
            Some("http://127.0.0.1:7890".to_string())
        );

        let unquoted = "httpProxy: http://127.0.0.1:1080\n";
        assert_eq!(
            parse_yarnrc_yml_proxy(unquoted),
            Some("http://127.0.0.1:1080".to_string())
        );

        let empty = "nodeLinker: node-modules\n";
        assert_eq!(parse_yarnrc_yml_proxy(empty), None);
    }

    #[test]
    fn test_check_case_mismatch() {
        assert!(check_case_mismatch(
            "http://127.0.0.1:7890",
            "http://127.0.0.1:1080"
        ));
        assert!(!check_case_mismatch(
            "http://127.0.0.1:7890",
            "http://127.0.0.1:7890"
        ));
        assert!(!check_case_mismatch(
            "http://127.0.0.1:7890/",
            "http://127.0.0.1:7890"
        ));
        assert!(!check_case_mismatch("", "http://127.0.0.1:7890"));
    }

    #[test]
    fn test_parse_condarc_proxy() {
        let content = r#"
channels:
  - defaults
proxy_servers:
  http: http://127.0.0.1:7890
  https: "http://127.0.0.1:7890"
ssl_verify: true
"#;
        let cfg = parse_condarc_proxy(content).unwrap();
        assert_eq!(cfg.http, Some("http://127.0.0.1:7890".to_string()));
        assert_eq!(cfg.https, Some("http://127.0.0.1:7890".to_string()));

        let empty = "channels:\n  - conda-forge\n";
        assert_eq!(parse_condarc_proxy(empty), None);
    }

    #[test]
    fn test_parse_hosts_file() {
        let content = r#"
# Loopback entries
127.0.0.1 localhost
::1 localhost

# Pinned developer domains
140.82.112.4 github.com api.github.com
185.199.108.133 raw.githubusercontent.com
0.0.0.0 telemetry.example.com
"#;
        let entries = parse_hosts_file(content);
        assert!(entries
            .iter()
            .any(|e| e.ip == "140.82.112.4" && e.domain == "github.com"));
        assert!(entries
            .iter()
            .any(|e| e.ip == "140.82.112.4" && e.domain == "api.github.com"));
        assert!(entries
            .iter()
            .any(|e| e.ip == "185.199.108.133" && e.domain == "raw.githubusercontent.com"));
        assert!(entries
            .iter()
            .any(|e| e.ip == "0.0.0.0" && e.domain == "telemetry.example.com"));
    }

    #[test]
    fn test_parse_bunfig_proxy() {
        let content = r#"
[install]
proxy = "http://127.0.0.1:7890"
exact = true
"#;
        assert_eq!(
            parse_bunfig_proxy(content),
            Some("http://127.0.0.1:7890".to_string())
        );

        let toplevel = "proxy = 'http://127.0.0.1:1080'\n";
        assert_eq!(
            parse_bunfig_proxy(toplevel),
            Some("http://127.0.0.1:1080".to_string())
        );

        let clean = "[install]\nexact = true\n";
        assert_eq!(parse_bunfig_proxy(clean), None);
    }

    #[test]
    fn test_parse_pnpmrc_proxy() {
        let content = r#"
proxy=http://127.0.0.1:7890
https-proxy="http://127.0.0.1:7890"
registry=https://registry.npmjs.org/
"#;
        assert_eq!(
            parse_pnpmrc_proxy(content, "proxy"),
            Some("http://127.0.0.1:7890".to_string())
        );
        assert_eq!(
            parse_pnpmrc_proxy(content, "https-proxy"),
            Some("http://127.0.0.1:7890".to_string())
        );
        assert_eq!(parse_pnpmrc_proxy(content, "nonexistent"), None);
    }

    #[test]
    fn test_parse_cargo_registry_replacements() {
        let content = r#"
[source.crates-io]
replace-with = "ustc"

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"
"#;
        let replacements = parse_cargo_registry_replacements(content);
        assert_eq!(replacements.len(), 1);
        assert_eq!(replacements[0].source, "crates-io");
        assert_eq!(replacements[0].replace_with, "ustc");
        assert_eq!(
            replacements[0].registry_url,
            Some("https://mirrors.ustc.edu.cn/crates.io-index".to_string())
        );
    }

    #[test]
    fn test_parse_windows_reg_proxy_single() {
        let enable = "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings\n    ProxyEnable    REG_DWORD    0x1\n";
        let server = "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings\n    ProxyServer    REG_SZ    127.0.0.1:7890\n";
        let res = parse_windows_reg_proxy(enable, server).unwrap();
        assert!(res.http_enabled);
        assert_eq!(res.http_host, "127.0.0.1");
        assert_eq!(res.http_port, 7890);
        assert!(res.https_enabled);
        assert_eq!(res.https_port, 7890);
        assert!(!res.socks_enabled);
    }

    #[test]
    fn test_parse_windows_reg_proxy_multi() {
        let enable = "    ProxyEnable    REG_DWORD    0x1\n";
        let server = "    ProxyServer    REG_SZ    http=127.0.0.1:7890;https=127.0.0.1:7890;socks=127.0.0.1:1080\n";
        let res = parse_windows_reg_proxy(enable, server).unwrap();
        assert!(res.http_enabled);
        assert_eq!(res.http_port, 7890);
        assert!(res.https_enabled);
        assert_eq!(res.https_port, 7890);
        assert!(res.socks_enabled);
        assert_eq!(res.socks_port, 1080);
    }

    #[test]
    fn test_parse_windows_reg_proxy_disabled() {
        let enable = "    ProxyEnable    REG_DWORD    0x0\n";
        let server = "    ProxyServer    REG_SZ    127.0.0.1:7890\n";
        let res = parse_windows_reg_proxy(enable, server).unwrap();
        assert!(!res.http_enabled);
        assert!(!res.https_enabled);
        assert!(!res.socks_enabled);
    }

    #[test]
    fn test_parse_gnome_proxy_manual() {
        let res = parse_gnome_proxy(
            "'manual'",
            "'127.0.0.1'",
            "7890",
            "'127.0.0.1'",
            "7890",
            "'127.0.0.1'",
            "1080",
        )
        .unwrap();
        assert!(res.http_enabled);
        assert_eq!(res.http_host, "127.0.0.1");
        assert_eq!(res.http_port, 7890);
        assert!(res.https_enabled);
        assert_eq!(res.https_port, 7890);
        assert!(res.socks_enabled);
        assert_eq!(res.socks_port, 1080);
    }

    #[test]
    fn test_parse_gnome_proxy_none() {
        let res = parse_gnome_proxy(
            "'none'",
            "'127.0.0.1'",
            "7890",
            "'127.0.0.1'",
            "7890",
            "''",
            "0",
        )
        .unwrap();
        assert!(!res.http_enabled);
        assert!(!res.https_enabled);
        assert!(!res.socks_enabled);
    }

    #[test]
    fn test_clean_jsonc_with_comments_and_trailing_commas() {
        let input = r#"{
            // Single line comment
            "http.proxy": "http://127.0.0.1:7890", /* block comment */
            "http.proxyStrictSSL": false, // trailing comma follows
        }"#;
        let cleaned = clean_jsonc(input);
        let val: serde_json::Value =
            serde_json::from_str(&cleaned).expect("Should parse valid JSON");
        assert_eq!(val["http.proxy"], "http://127.0.0.1:7890");
        assert_eq!(val["http.proxyStrictSSL"], false);
    }

    #[test]
    fn test_clean_jsonc_preserves_urls() {
        let input = r#"{"proxy": "http://127.0.0.1:7890//test"}"#;
        let cleaned = clean_jsonc(input);
        let val: serde_json::Value =
            serde_json::from_str(&cleaned).expect("Should preserve // inside strings");
        assert_eq!(val["proxy"], "http://127.0.0.1:7890//test");
    }

    #[test]
    fn test_parse_ide_settings() {
        let input = r#"{
            "workbench.colorTheme": "Cursor Dark",
            "http.proxy": "http://127.0.0.1:7890",
            "http.proxyStrictSSL": false,
            "http.proxySupport": "override",
        }"#;
        let path = std::path::Path::new("/dummy/settings.json");
        let parsed = parse_ide_settings("Cursor", path, input).expect("Should parse IDE settings");
        assert_eq!(parsed.ide_name, "Cursor");
        assert_eq!(parsed.proxy.as_deref(), Some("http://127.0.0.1:7890"));
        assert_eq!(parsed.strict_ssl, Some(false));
        assert_eq!(parsed.proxy_support.as_deref(), Some("override"));
    }

    #[test]
    fn test_parse_jetbrains_proxy_xml_http() {
        let input = r#"<application>
  <component name="HttpConfigurable">
    <option name="PROXY_TYPE_NUM" value="1" />
    <option name="USE_HTTP_PROXY" value="true" />
    <option name="PROXY_HOST" value="127.0.0.1" />
    <option name="PROXY_PORT" value="7890" />
    <option name="USE_PROXY_PAC" value="false" />
  </component>
</application>"#;
        let path = std::path::Path::new("/dummy/proxy.settings.xml");
        let parsed = parse_jetbrains_proxy_xml("IntelliJIdea2024.1", path, input)
            .expect("Should parse JetBrains HTTP proxy");
        assert_eq!(parsed.ide_name, "IntelliJIdea2024.1");
        assert!(parsed.use_proxy);
        assert!(!parsed.is_socks);
        assert_eq!(parsed.host.as_deref(), Some("127.0.0.1"));
        assert_eq!(parsed.port, Some(7890));
        assert!(!parsed.use_pac);
    }

    #[test]
    fn test_parse_jetbrains_proxy_xml_socks() {
        let input = r#"<application>
  <component name="HttpConfigurable">
    <option name="PROXY_TYPE_NUM" value="2" />
    <option name="USE_HTTP_PROXY" value="true" />
    <option name="PROXY_HOST" value="127.0.0.1" />
    <option name="PROXY_PORT" value="1080" />
  </component>
</application>"#;
        let path = std::path::Path::new("/dummy/proxy.settings.xml");
        let parsed = parse_jetbrains_proxy_xml("PyCharm2024.2", path, input)
            .expect("Should parse JetBrains SOCKS proxy");
        assert_eq!(parsed.ide_name, "PyCharm2024.2");
        assert!(parsed.use_proxy);
        assert!(parsed.is_socks);
        assert_eq!(parsed.host.as_deref(), Some("127.0.0.1"));
        assert_eq!(parsed.port, Some(1080));
    }

    #[test]
    fn test_parse_jetbrains_proxy_xml_pac() {
        let input = r#"<application>
  <component name="HttpConfigurable">
    <option name="USE_HTTP_PROXY" value="false" />
    <option name="USE_PROXY_PAC" value="true" />
    <option name="PAC_URL" value="http://127.0.0.1:7890/pac" />
  </component>
</application>"#;
        let path = std::path::Path::new("/dummy/proxy.settings.xml");
        let parsed = parse_jetbrains_proxy_xml("AndroidStudio2026.1", path, input)
            .expect("Should parse JetBrains PAC proxy");
        assert!(!parsed.use_proxy);
        assert!(parsed.use_pac);
        assert_eq!(parsed.pac_url.as_deref(), Some("http://127.0.0.1:7890/pac"));
    }

    #[test]
    fn test_parse_jetbrains_proxy_xml_disabled() {
        let input = r#"<application>
  <component name="HttpConfigurable">
    <option name="USE_PROXY_PAC" value="true" />
  </component>
</application>"#;
        let path = std::path::Path::new("/dummy/proxy.settings.xml");
        let parsed = parse_jetbrains_proxy_xml("AndroidStudio2026.1", path, input)
            .expect("Should parse disabled PAC proxy");
        assert!(!parsed.use_proxy);
        assert!(parsed.use_pac);
        assert!(parsed.pac_url.is_none());
    }

    #[test]
    fn test_parse_gh_config_proxy() {
        let input = r#"# The current version of the config schema
version: 1
git_protocol: https
# A proxy to use for HTTP requests
http_proxy: http://127.0.0.1:7890
"#;
        let proxy = parse_gh_config_proxy(input).expect("Should find http_proxy");
        assert_eq!(proxy, "http://127.0.0.1:7890");
    }

    #[test]
    fn test_parse_gh_config_proxy_none() {
        let input = r#"version: 1
git_protocol: https
# http_proxy: http://127.0.0.1:7890
editor:
"#;
        assert!(parse_gh_config_proxy(input).is_none());
    }

    #[test]
    fn test_parse_nuget_config_proxy_standard() {
        let input = r#"<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <packageSources>
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" protocolVersion="3" />
  </packageSources>
  <config>
    <add key="http_proxy" value="http://127.0.0.1:7890" />
  </config>
</configuration>"#;
        let proxy =
            parse_nuget_config_proxy(input).expect("Should find http_proxy in NuGet.Config");
        assert_eq!(proxy, "http://127.0.0.1:7890");
    }

    #[test]
    fn test_parse_nuget_config_proxy_case_and_quotes() {
        let input = r#"<configuration>
  <config>
    <add KEY = 'HTTP_PROXY' VALUE = 'http://127.0.0.1:1080' />
  </config>
</configuration>"#;
        let proxy =
            parse_nuget_config_proxy(input).expect("Should find HTTP_PROXY with single quotes");
        assert_eq!(proxy, "http://127.0.0.1:1080");
    }

    #[test]
    fn test_parse_nuget_config_proxy_commented() {
        let input = r#"<configuration>
  <!-- <config>
    <add key="http_proxy" value="http://127.0.0.1:7890" />
  </config> -->
</configuration>"#;
        assert!(parse_nuget_config_proxy(input).is_none());
    }

    #[test]
    fn test_parse_nuget_config_proxy_none() {
        let input = r#"<configuration>
  <packageSources>
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" />
  </packageSources>
</configuration>"#;
        assert!(parse_nuget_config_proxy(input).is_none());
    }

    #[test]
    fn test_parse_gemrc_proxy() {
        let yml1 = "---\n:backtrace: false\nhttp_proxy: http://127.0.0.1:7890\n";
        assert_eq!(
            parse_gemrc_proxy(yml1),
            Some("http://127.0.0.1:7890".to_string())
        );

        let yml2 = "---\n:http_proxy: \"http://127.0.0.1:1080\"\n";
        assert_eq!(
            parse_gemrc_proxy(yml2),
            Some("http://127.0.0.1:1080".to_string())
        );

        let yml3 = "# comment\nhttp_proxy: :system\n";
        assert_eq!(parse_gemrc_proxy(yml3), None);

        let yml4 = "---\n:sources:\n- https://rubygems.org/\n";
        assert_eq!(parse_gemrc_proxy(yml4), None);
    }

    #[test]
    fn test_parse_svn_servers_proxy() {
        let ini1 = r#"
[groups]
group1 = *.example.com

[global]
# http-proxy-exceptions = *.internal.net
http-proxy-host = 127.0.0.1
http-proxy-port = 7890
"#;
        assert_eq!(
            parse_svn_servers_proxy(ini1),
            Some("http://127.0.0.1:7890".to_string())
        );

        let ini2 = r#"
[global]
http-proxy-host = 10.0.0.1
"#;
        assert_eq!(
            parse_svn_servers_proxy(ini2),
            Some("http://10.0.0.1:80".to_string())
        );

        let ini3 = r#"
# all commented
# [global]
# http-proxy-host = 127.0.0.1
"#;
        assert_eq!(parse_svn_servers_proxy(ini3), None);
    }

    #[test]
    fn test_parse_hgrc_proxy() {
        let ini1 = r#"
[ui]
username = Alice

[http_proxy]
host = 127.0.0.1:7890
no = localhost, 127.0.0.1
"#;
        assert_eq!(
            parse_hgrc_proxy(ini1),
            Some("http://127.0.0.1:7890".to_string())
        );

        let ini2 = r#"
[http_proxy]
host = "socks5://127.0.0.1:1080"
"#;
        assert_eq!(
            parse_hgrc_proxy(ini2),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let ini3 = r#"
[ui]
username = Bob
# [http_proxy]
# host = 127.0.0.1:7890
"#;
        assert_eq!(parse_hgrc_proxy(ini3), None);
    }

    #[test]
    fn test_parse_yarnrc_registry() {
        let content = r#"
# yarn lockfile v1
registry "https://registry.npmmirror.com/"
lastUpdateCheck 1590000000000
"#;
        assert_eq!(
            parse_yarnrc_registry(content),
            Some("https://registry.npmmirror.com/".to_string())
        );

        let empty = "# only comments\n";
        assert_eq!(parse_yarnrc_registry(empty), None);
    }

    #[test]
    fn test_parse_yarnrc_yml_registry() {
        let yaml = r#"
npmRegistryServer: "https://registry.npmjs.org"
yarnPath: ".yarn/releases/yarn-4.0.0.cjs"
"#;
        assert_eq!(
            parse_yarnrc_yml_registry(yaml),
            Some("https://registry.npmjs.org".to_string())
        );

        let empty = "plugins: []\n";
        assert_eq!(parse_yarnrc_yml_registry(empty), None);
    }

    #[test]
    fn test_parse_uv_proxy() {
        let toml1 = r#"
http-proxy = "http://127.0.0.1:7890"
"#;
        assert_eq!(
            parse_uv_proxy(toml1),
            Some("http://127.0.0.1:7890".to_string())
        );

        let toml2 = r#"
[pip]
proxy = "socks5://127.0.0.1:1080"
"#;
        assert_eq!(
            parse_uv_proxy(toml2),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let toml3 = r#"
# http-proxy = "http://127.0.0.1:7890"
[tool.uv]
cache-dir = "~/.cache/uv"
"#;
        assert_eq!(parse_uv_proxy(toml3), None);
    }

    #[test]
    fn test_parse_go_env_content() {
        let env1 = "GOPROXY=\"http://127.0.0.1:7890,direct\"\nGONOPROXY=\n";
        assert_eq!(
            parse_go_env_content(env1),
            Some("http://127.0.0.1:7890,direct".to_string())
        );

        let env2 = "GOPRIVATE=*.internal.corp\n";
        assert_eq!(parse_go_env_content(env2), None);

        let env3 = "# GOPROXY=http://127.0.0.1:8888\n";
        assert_eq!(parse_go_env_content(env3), None);
    }

    #[test]
    fn test_parse_composer_packagist_repo() {
        let json1 = r#"{
            "repositories": {
                "packagist": {
                    "type": "composer",
                    "url": "https://mirrors.aliyun.com/composer/"
                }
            }
        }"#;
        assert_eq!(
            parse_composer_packagist_repo(json1),
            Some("https://mirrors.aliyun.com/composer/".to_string())
        );

        let json2 = r#"{ "config": {} }"#;
        assert_eq!(parse_composer_packagist_repo(json2), None);
    }

    #[test]
    fn test_parse_composer_proxy() {
        let json1 = r#"{
            "config": {
                "http-proxy": "http://127.0.0.1:7890",
                "https-proxy": "http://127.0.0.1:7890"
            }
        }"#;
        assert_eq!(
            parse_composer_proxy(json1),
            Some("http://127.0.0.1:7890".to_string())
        );

        let json2 = r#"{
            "config": {
                "http-proxy": "http://127.0.0.1:8080"
            }
        }"#;
        assert_eq!(
            parse_composer_proxy(json2),
            Some("http://127.0.0.1:8080".to_string())
        );

        let json3 = r#"{ "config": {} }"#;
        assert_eq!(parse_composer_proxy(json3), None);

        let json4 = "invalid json";
        assert_eq!(parse_composer_proxy(json4), None);
    }

    #[test]
    fn test_parse_bazelrc_proxy() {
        let bazel1 = "build --action_env=HTTP_PROXY=http://127.0.0.1:7890\n";
        assert_eq!(
            parse_bazelrc_proxy(bazel1),
            Some("http://127.0.0.1:7890".to_string())
        );

        let bazel2 =
            "# build --action_env=HTTP_PROXY=http://127.0.0.1:7890\ntest --verbose_failures\n";
        assert_eq!(parse_bazelrc_proxy(bazel2), None);
    }

    #[test]
    fn test_parse_conan2_proxy() {
        let conf1 = r#"
core.net.http:proxies = {"http": "http://127.0.0.1:7890", "https": "http://127.0.0.1:7890"}
"#;
        assert_eq!(
            parse_conan2_proxy(conf1),
            Some("http://127.0.0.1:7890".to_string())
        );

        let conf2 = r#"
core.net.http:proxies = {'http': 'socks5://127.0.0.1:1080'}
"#;
        assert_eq!(
            parse_conan2_proxy(conf2),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let conf3 =
            "# core.net.http:proxies = {'http': 'http://127.0.0.1:7890'}\ntools.build:jobs = 8\n";
        assert_eq!(parse_conan2_proxy(conf3), None);

        let conf4 = "tools.build:jobs = 4\n";
        assert_eq!(parse_conan2_proxy(conf4), None);
    }

    #[test]
    fn test_parse_conan1_proxy() {
        let ini1 = r#"
[general]
revisions_enabled = 1

[proxies]
http = http://127.0.0.1:7890
https = http://127.0.0.1:8080
"#;
        assert_eq!(
            parse_conan1_proxy(ini1),
            Some("http://127.0.0.1:8080".to_string())
        );

        let ini2 = r#"
# [proxies]
# http = http://127.0.0.1:7890
"#;
        assert_eq!(parse_conan1_proxy(ini2), None);
    }

    #[test]
    fn test_parse_kubeconfig_proxies() {
        let yaml = r#"
apiVersion: v1
clusters:
- cluster:
    certificate-authority: /path/to/ca.crt
    proxy-url: http://127.0.0.1:7890
    server: https://192.168.49.2:8443
  name: minikube
- cluster:
    server: https://127.0.0.1:6443
  name: docker-desktop
contexts:
- context:
    cluster: minikube
    user: minikube
  name: minikube
"#;
        let proxies = parse_kubeconfig_proxies(yaml);
        assert_eq!(proxies.len(), 1);
        assert_eq!(proxies[0].0, "minikube");
        assert_eq!(proxies[0].1, "http://127.0.0.1:7890");

        let yaml_none = r#"
apiVersion: v1
clusters:
- cluster:
    server: https://127.0.0.1:6443
  name: docker-desktop
"#;
        assert!(parse_kubeconfig_proxies(yaml_none).is_empty());
    }

    #[test]
    fn test_parse_sbtopts_proxy() {
        let sbt1 = "-J-Xmx2G\n-Dhttp.proxyHost=127.0.0.1\n-Dhttp.proxyPort=7890\n-Dhttps.proxyHost=127.0.0.1\n-Dhttps.proxyPort=7890\n";
        assert_eq!(
            parse_sbtopts_proxy(sbt1),
            Some("http://127.0.0.1:7890".to_string())
        );

        let sbt_inline = "-Dhttp.proxyHost=\"10.0.0.1\" -Dhttp.proxyPort='8080'";
        assert_eq!(
            parse_sbtopts_proxy(sbt_inline),
            Some("http://10.0.0.1:8080".to_string())
        );

        let sbt_socks = "-DsocksProxyHost=127.0.0.1 -DsocksProxyPort=1080";
        assert_eq!(
            parse_sbtopts_proxy(sbt_socks),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let sbt_none = "# -Dhttp.proxyHost=127.0.0.1\n-J-Xmx4G\n";
        assert_eq!(parse_sbtopts_proxy(sbt_none), None);
    }

    #[test]
    fn test_parse_glab_config_proxy() {
        let input = r#"version: 1
git_protocol: https
glab_proxy: http://127.0.0.1:7890
"#;
        assert_eq!(
            parse_glab_config_proxy(input),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_http = "http_proxy: \"http://10.0.0.1:8080\"\n";
        assert_eq!(
            parse_glab_config_proxy(input_http),
            Some("http://10.0.0.1:8080".to_string())
        );
    }

    #[test]
    fn test_parse_glab_config_proxy_none() {
        let input = "git_protocol: ssh\n# glab_proxy: http://127.0.0.1:7890\n";
        assert_eq!(parse_glab_config_proxy(input), None);
    }

    #[test]
    fn test_parse_julia_startup_proxy() {
        let input = r#"
# Some comments
println("Welcome to Julia")
ENV["HTTP_PROXY"] = "http://127.0.0.1:7890"
ENV["HTTPS_PROXY"] = "http://127.0.0.1:7890"
"#;
        assert_eq!(
            parse_julia_startup_proxy(input),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_socks = "ENV[\"all_proxy\"] = \"socks5://127.0.0.1:1080\";\n";
        assert_eq!(
            parse_julia_startup_proxy(input_socks),
            Some("socks5://127.0.0.1:1080".to_string())
        );
    }

    #[test]
    fn test_parse_julia_startup_proxy_none() {
        let input = "# ENV[\"HTTP_PROXY\"] = \"http://127.0.0.1:7890\"\nusing Pkg\n";
        assert_eq!(parse_julia_startup_proxy(input), None);
    }

    #[test]
    fn test_parse_nix_substituters() {
        let input = r#"
substituters = https://cache.nixos.org https://mirrors.tuna.tsinghua.edu.cn/nix-channels/store https://mirror.sjtu.edu.cn/nix-channels/store
trusted-users = root @wheel
"#;
        let subs = parse_nix_substituters(input);
        assert_eq!(subs.len(), 3);
        assert!(subs.contains(&"https://cache.nixos.org".to_string()));
        assert!(subs.contains(&"https://mirrors.tuna.tsinghua.edu.cn/nix-channels/store".to_string()));
    }

    #[test]
    fn test_parse_swiftpm_mirrors() {
        let input = r#"{
  "mirrors": {
    "https://github.com/apple/swift-argument-parser.git": "https://hub.fastgit.xyz/apple/swift-argument-parser.git"
  },
  "version": 1
}"#;
        let mirrors = parse_swiftpm_mirrors(input);
        assert_eq!(mirrors, vec!["https://hub.fastgit.xyz/apple/swift-argument-parser.git".to_string()]);
    }

    #[test]
    fn test_parse_nix_proxy() {
        let input = r#"
# Nix configuration
substituters = https://cache.nixos.org
http-proxy = http://127.0.0.1:7890
trusted-users = root
"#;
        assert_eq!(
            parse_nix_proxy(input),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_quoted = "http-proxy = \"socks5h://127.0.0.1:1080\"\n";
        assert_eq!(
            parse_nix_proxy(input_quoted),
            Some("socks5h://127.0.0.1:1080".to_string())
        );

        let input_none = "# http-proxy = http://127.0.0.1:7890\nmax-jobs = auto\n";
        assert_eq!(parse_nix_proxy(input_none), None);
    }

    #[test]
    fn test_parse_cabal_config_proxy() {
        let input = r#"
-- Cabal config
-- proxy: http://example.com:8080
proxy: http://127.0.0.1:7890
remote-repo-cache: ~/.cabal/packages
"#;
        assert_eq!(
            parse_cabal_config_proxy(input),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_none = "-- proxy: http://127.0.0.1:7890\n-- cabal comments\n";
        assert_eq!(parse_cabal_config_proxy(input_none), None);
    }

    #[test]
    fn test_parse_stack_config_proxy() {
        let input_yaml = r#"
# Haskell stack config
proxy:
  http: "http://127.0.0.1:7890"
  https: "http://127.0.0.1:7890"
system-ghc: true
"#;
        assert_eq!(
            parse_stack_config_proxy(input_yaml),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_flow = r#"
proxy: { http: "http://127.0.0.1:7897", https: "http://127.0.0.1:7897" }
"#;
        assert_eq!(
            parse_stack_config_proxy(input_flow),
            Some("http://127.0.0.1:7897".to_string())
        );

        let input_none = "# proxy:\n#   http: http://127.0.0.1:7890\n";
        assert_eq!(parse_stack_config_proxy(input_none), None);
    }

    #[test]
    fn test_parse_hex_config_proxy() {
        let input_erlang = r#"
% Hex config
{http_proxy, "http://127.0.0.1:7890"}.
{https_proxy, "http://127.0.0.1:7890"}.
"#;
        assert_eq!(
            parse_hex_config_proxy(input_erlang),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_binary = r#"
{http_proxy, <<"socks5://127.0.0.1:1080">>}.
"#;
        assert_eq!(
            parse_hex_config_proxy(input_binary),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_none = "% {http_proxy, \"http://127.0.0.1:7890\"}.\n";
        assert_eq!(parse_hex_config_proxy(input_none), None);
    }

    #[test]
    fn test_clean_swiftpm_mirrors() {
        let json_mirrors_map = r#"{
  "mirrors": {
    "https://github.com/apple/swift-nio.git": "https://mirror.dead.local/swift-nio.git",
    "https://github.com/apple/swift-log.git": "https://mirror.good.org/swift-log.git"
  }
}"#;
        let dead = ["https://mirror.dead.local/swift-nio.git"];
        let (cleaned, modified) = clean_swiftpm_mirrors(json_mirrors_map, &dead);
        assert!(modified);
        assert!(!cleaned.contains("mirror.dead.local"));
        assert!(cleaned.contains("mirror.good.org"));

        let json_object_list = r#"{
  "object": [
    {
      "original": "https://github.com/apple/swift-nio.git",
      "mirror": "https://mirror.dead.local/swift-nio.git"
    },
    {
      "original": "https://github.com/apple/swift-log.git",
      "mirror": "https://mirror.good.org/swift-log.git"
    }
  ],
  "version": 1
}"#;
        let (cleaned2, modified2) = clean_swiftpm_mirrors(json_object_list, &dead);
        assert!(modified2);
        assert!(!cleaned2.contains("mirror.dead.local"));
        assert!(cleaned2.contains("mirror.good.org"));
    }

    #[test]
    fn test_parse_renviron_proxy() {
        let input_renviron = r#"
# User R environment
http_proxy=http://127.0.0.1:7890
https_proxy="http://127.0.0.1:7890"
CRAN_MIRROR="https://cloud.r-project.org"
"#;
        assert_eq!(
            parse_renviron_proxy(input_renviron),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_socks = r#"
all_proxy='socks5://127.0.0.1:1080'
"#;
        assert_eq!(
            parse_renviron_proxy(input_socks),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_none = "# http_proxy=http://127.0.0.1:7890\n";
        assert_eq!(parse_renviron_proxy(input_none), None);
    }

    #[test]
    fn test_parse_opam_config_proxy() {
        let input_opam = r#"
opam-version: "2.0"
http-proxy: "http://127.0.0.1:7890"
https-proxy: "http://127.0.0.1:7890"
"#;
        assert_eq!(
            parse_opam_config_proxy(input_opam),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_socks = r#"
http_proxy: "socks5://127.0.0.1:1080"
"#;
        assert_eq!(
            parse_opam_config_proxy(input_socks),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_none = "# http-proxy: \"http://127.0.0.1:7890\"\n";
        assert_eq!(parse_opam_config_proxy(input_none), None);
    }

    #[test]
    fn test_parse_terraform_network_mirrors() {
        let input_tf = r#"
provider_installation {
  network_mirror {
    url = "https://terraform-mirror.example.com/"
  }
  direct {
    exclude = ["example.com/*/*"]
  }
}
"#;
        let mirrors = parse_terraform_network_mirrors(input_tf);
        assert_eq!(mirrors.len(), 1);
        assert_eq!(mirrors[0], "https://terraform-mirror.example.com/");

        let input_none = "plugin_cache_dir = \"$HOME/.terraform.d/plugin-cache\"\n";
        assert!(parse_terraform_network_mirrors(input_none).is_empty());
    }

    #[test]
    fn test_clean_terraform_network_mirrors() {
        let input_tf = r#"
provider_installation {
  network_mirror {
    url = "https://dead.mirror.example.com/"
  }
  network_mirror {
    url = "https://good.mirror.example.com/"
  }
}
"#;
        let (cleaned, changed) = clean_terraform_network_mirrors(
            input_tf,
            &["https://dead.mirror.example.com/"],
        );
        assert!(changed);
        assert!(cleaned.contains("# [proxydoc disabled dead mirror(s)]"));
        assert!(cleaned.contains("#   network_mirror {"));
        assert!(cleaned.contains("#     url = \"https://dead.mirror.example.com/\""));
        assert!(cleaned.contains("https://good.mirror.example.com/"));

        // Single line url check
        let single_tf = "url = \"https://dead.mirror.example.com/\"\n";
        let (cleaned_single, changed_single) = clean_terraform_network_mirrors(
            single_tf,
            &["https://dead.mirror.example.com/"],
        );
        assert!(changed_single);
        assert!(cleaned_single.contains("# [proxydoc disabled dead mirror]"));
    }

    #[test]
    fn test_parse_lein_proxy() {
        let input_lein = r#"
{:user {:plugins [[lein-ancient "0.6.15"]]
        :http-proxy "http://127.0.0.1:7890"
        :https-proxy "http://127.0.0.1:7890"}}
"#;
        assert_eq!(
            parse_lein_proxy(input_lein),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_jvm = r#"
{:user {:jvm-opts ["-Dhttp.proxyHost=127.0.0.1" "-Dhttp.proxyPort=1080"]}}
"#;
        assert_eq!(
            parse_lein_proxy(input_jvm),
            Some("http://127.0.0.1:1080".to_string())
        );

        let input_none = "{:user {:plugins []}}\n";
        assert_eq!(parse_lein_proxy(input_none), None);
    }

    #[test]
    fn test_parse_cpan_proxy() {
        let input_cpan = r#"
$CPAN::Config = {
  'http_proxy' => q[http://127.0.0.1:7890],
  'ftp_proxy' => q[http://127.0.0.1:7890],
};
1;
__END__
"#;
        assert_eq!(
            parse_cpan_proxy(input_cpan),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_quotes = r#"
$CPAN::Config = {
  "http_proxy" => "http://127.0.0.1:1080",
};
"#;
        assert_eq!(
            parse_cpan_proxy(input_quotes),
            Some("http://127.0.0.1:1080".to_string())
        );

        let input_none = "# 'http_proxy' => q[http://127.0.0.1:7890],\n";
        assert_eq!(parse_cpan_proxy(input_none), None);
    }

    #[test]
    fn test_parse_vcpkg_asset_url() {
        let input_azurl = "x-azurl,https://mirror.example.com/vcpkg-assets,read";
        assert_eq!(
            parse_vcpkg_asset_url(input_azurl),
            Some("https://mirror.example.com/vcpkg-assets".to_string())
        );

        let input_binary = "clear;nuget,http://127.0.0.1:8080/v2/index.json";
        assert_eq!(
            parse_vcpkg_asset_url(input_binary),
            Some("http://127.0.0.1:8080/v2/index.json".to_string())
        );

        let input_none = "clear;files,C:\\vcpkg-cache,readwrite";
        assert_eq!(parse_vcpkg_asset_url(input_none), None);
    }

    #[test]
    fn test_parse_luarocks_proxy() {
        let input_std = r#"
variables = {
    HTTP_PROXY = "http://127.0.0.1:7890",
    HTTPS_PROXY = "http://127.0.0.1:7890",
}
"#;
        assert_eq!(
            parse_luarocks_proxy(input_std),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_simple = "http_proxy = 'http://127.0.0.1:1080'\nhttps_proxy = 'http://127.0.0.1:1080'";
        assert_eq!(
            parse_luarocks_proxy(input_simple),
            Some("http://127.0.0.1:1080".to_string())
        );

        let input_commented = "-- http_proxy = 'http://127.0.0.1:7890'\n";
        assert_eq!(parse_luarocks_proxy(input_commented), None);
    }

    #[test]
    fn test_parse_vcpkg_configuration_registries() {
        let input_json = r#"{
  "default-registry": {
    "kind": "git",
    "repository": "https://github.com/microsoft/vcpkg",
    "baseline": "12345"
  },
  "registries": [
    {
      "kind": "git",
      "repository": "https://mirror.example.com/vcpkg-custom",
      "baseline": "abcde"
    }
  ]
}"#;
        let regs = parse_vcpkg_configuration_registries(input_json);
        assert_eq!(regs.len(), 2);
        assert_eq!(regs[0], "https://github.com/microsoft/vcpkg");
        assert_eq!(regs[1], "https://mirror.example.com/vcpkg-custom");
    }

    #[test]
    fn test_clean_vcpkg_configuration_registries() {
        let input_json = r#"{
  "default-registry": {
    "kind": "git",
    "repository": "https://dead-mirror.example.com/vcpkg",
    "baseline": "12345"
  },
  "registries": [
    {
      "kind": "git",
      "repository": "https://mirror.example.com/dead-reg",
      "baseline": "abc"
    },
    {
      "kind": "git",
      "repository": "https://live.example.com/reg",
      "baseline": "xyz"
    }
  ]
}"#;
        let dead = ["https://dead-mirror.example.com/vcpkg", "https://mirror.example.com/dead-reg"];
        let (cleaned, modified) = clean_vcpkg_configuration_registries(input_json, &dead);
        assert!(modified);
        let remaining = parse_vcpkg_configuration_registries(&cleaned);
        assert_eq!(remaining.len(), 2);
        assert_eq!(remaining[0], "https://github.com/microsoft/vcpkg");
        assert_eq!(remaining[1], "https://live.example.com/reg");
    }

    #[test]
    fn test_parse_nimble_proxy() {
        let input_ini = r#"
# Nimble configuration file
[Package]
http_proxy = "http://127.0.0.1:7890"
https_proxy = "http://127.0.0.1:7890"
"#;
        assert_eq!(
            parse_nimble_proxy(input_ini),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_simple = "http_proxy = http://127.0.0.1:1080\n";
        assert_eq!(
            parse_nimble_proxy(input_simple),
            Some("http://127.0.0.1:1080".to_string())
        );

        let input_commented = "; http_proxy = \"http://127.0.0.1:7890\"\n# https_proxy = 127.0.0.1\n";
        assert_eq!(parse_nimble_proxy(input_commented), None);
    }

    #[test]
    fn test_parse_rebar3_proxy() {
        let input_erlang = r#"
%% Rebar3 global configuration
{http_proxy, "http://127.0.0.1:7890"}.
{https_proxy, "http://127.0.0.1:7890"}.
"#;
        assert_eq!(
            parse_rebar3_proxy(input_erlang),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_binary = r#"
{http_proxy, <<"socks5://127.0.0.1:1080">>}.
"#;
        assert_eq!(
            parse_rebar3_proxy(input_binary),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_commented = "% {http_proxy, \"http://127.0.0.1:7890\"}.\n";
        assert_eq!(parse_rebar3_proxy(input_commented), None);
    }

    #[test]
    fn test_parse_helm_repositories() {
        let input_yaml = r#"
apiVersion: ""
generated: "2024-01-01T00:00:00Z"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable
  - name: bitnami
    url: https://charts.bitnami.com/bitnami
"#;
        let repos = parse_helm_repositories(input_yaml);
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0], "https://charts.helm.sh/stable");
        assert_eq!(repos[1], "https://charts.bitnami.com/bitnami");
    }

    #[test]
    fn test_clean_helm_repositories() {
        let input_yaml = r#"apiVersion: ""
generated: "2024-01-01T00:00:00Z"
repositories:
  - name: dead-repo
    url: https://dead.helm.example.com/charts
  - name: bitnami
    url: https://charts.bitnami.com/bitnami
"#;
        let dead = ["https://dead.helm.example.com/charts"];
        let (cleaned, modified) = clean_helm_repositories(input_yaml, &dead);
        assert!(modified);
        assert!(cleaned.contains("# [proxydoc disabled unreachable helm repository]"));
        assert!(cleaned.contains("#   - name: dead-repo"));
        assert!(cleaned.contains("https://charts.bitnami.com/bitnami"));
        let remaining = parse_helm_repositories(&cleaned);
        assert_eq!(remaining, vec!["https://charts.bitnami.com/bitnami".to_string()]);
    }

    #[test]
    fn test_parse_scoop_proxy() {
        let input_json = r#"{
    "proxy": "http://127.0.0.1:7890",
    "root_path": "C:\\Users\\user\\scoop"
}"#;
        assert_eq!(
            parse_scoop_proxy(input_json),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_host_port = r#"{ "proxy": "127.0.0.1:1080" }"#;
        assert_eq!(
            parse_scoop_proxy(input_host_port),
            Some("127.0.0.1:1080".to_string())
        );

        let input_none = r#"{ "proxy": "none" }"#;
        assert_eq!(parse_scoop_proxy(input_none), None);

        let input_empty = r#"{ "proxy": "" }"#;
        assert_eq!(parse_scoop_proxy(input_empty), None);
    }

    #[test]
    fn test_parse_bundler_proxy() {
        let input_yaml = r#"---
BUNDLE_HTTP_PROXY: "http://127.0.0.1:7890"
BUNDLE_HTTPS_PROXY: "http://127.0.0.1:7890"
"#;
        assert_eq!(
            parse_bundler_proxy(input_yaml),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_unquoted = "BUNDLE_HTTP_PROXY: http://127.0.0.1:1080\n";
        assert_eq!(
            parse_bundler_proxy(input_unquoted),
            Some("http://127.0.0.1:1080".to_string())
        );

        let input_lowercase = "http_proxy: \"socks5://127.0.0.1:1080\"\n";
        assert_eq!(
            parse_bundler_proxy(input_lowercase),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_commented = "# BUNDLE_HTTP_PROXY: \"http://127.0.0.1:7890\"\n";
        assert_eq!(parse_bundler_proxy(input_commented), None);
    }

    #[test]
    fn test_parse_gcloud_config_proxy() {
        let input_ini = r#"[core]
account = test@example.com
project = my-project

[proxy]
address = 127.0.0.1
port = 7890
type = http
"#;
        assert_eq!(
            parse_gcloud_config_proxy(input_ini),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_socks = r#"[proxy]
address = 127.0.0.1
port = 1080
type = socks5
"#;
        assert_eq!(
            parse_gcloud_config_proxy(input_socks),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_url = r#"[proxy]
address = http://127.0.0.1:7890
"#;
        assert_eq!(
            parse_gcloud_config_proxy(input_url),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_none = r#"[core]
project = my-project
"#;
        assert_eq!(parse_gcloud_config_proxy(input_none), None);
    }

    #[test]
    fn test_parse_aws_config_proxy() {
        let input_ini = r#"[default]
region = us-west-2
output = json
proxy = http://127.0.0.1:7890
"#;
        assert_eq!(
            parse_aws_config_proxy(input_ini, Some("default")),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_profile = r#"[profile dev]
region = us-east-1
http_proxy = socks5://127.0.0.1:1080
"#;
        assert_eq!(
            parse_aws_config_proxy(input_profile, Some("dev")),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_host_port = r#"[default]
proxy = 127.0.0.1:7890
"#;
        assert_eq!(
            parse_aws_config_proxy(input_host_port, Some("default")),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_none = r#"[default]
region = us-west-2
"#;
        assert_eq!(parse_aws_config_proxy(input_none, Some("default")), None);
    }

    #[test]
    fn test_parse_azure_config_proxy() {
        let input_standard = r#"[cloud]
name = AzureCloud

[proxy]
mode = manual
http = http://127.0.0.1:7890
https = http://127.0.0.1:7890
"#;
        assert_eq!(
            parse_azure_config_proxy(input_standard),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_socks = r#"[proxy]
mode = manual
https = socks5://127.0.0.1:1080
"#;
        assert_eq!(
            parse_azure_config_proxy(input_socks),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_url = r#"[proxy]
proxy_url = 127.0.0.1:7890
"#;
        assert_eq!(
            parse_azure_config_proxy(input_url),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_disabled = r#"[proxy]
mode = none
http = http://127.0.0.1:7890
"#;
        assert_eq!(parse_azure_config_proxy(input_disabled), None);

        let input_none = r#"[cloud]
name = AzureCloud
"#;
        assert_eq!(parse_azure_config_proxy(input_none), None);
    }

    #[test]
    fn test_parse_podman_proxy() {
        let input_env_array = r#"[engine]
env = [
  "http_proxy=http://127.0.0.1:7890",
  "https_proxy=http://127.0.0.1:7890"
]
"#;
        assert_eq!(
            parse_podman_proxy(input_env_array),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_engine_env = r#"[engine.env]
http_proxy = "socks5://127.0.0.1:1080"
"#;
        assert_eq!(
            parse_podman_proxy(input_engine_env),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_none = r#"[engine]
cgroup_manager = "systemd"
"#;
        assert_eq!(parse_podman_proxy(input_none), None);
    }

    #[test]
    fn test_parse_vagrant_proxy() {
        let input_vagrantfile = r#"Vagrant.configure("2") do |config|
  if Vagrant.has_plugin?("vagrant-proxyconf")
    config.proxy.http     = "http://127.0.0.1:7890"
    config.proxy.https    = "http://127.0.0.1:7890"
    config.proxy.no_proxy = "localhost,127.0.0.1"
  end
end
"#;
        assert_eq!(
            parse_vagrant_proxy(input_vagrantfile),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_none = r#"Vagrant.configure("2") do |config|
  config.vm.box = "ubuntu/focal64"
end
"#;
        assert_eq!(parse_vagrant_proxy(input_none), None);
    }

    #[test]
    fn test_parse_choco_proxy() {
        let input_choco = r#"<?xml version="1.0" encoding="utf-8"?>
<chocolatey>
  <config>
    <add key="proxy" value="http://127.0.0.1:7890" description="proxy" />
    <add key="proxyUser" value="" />
  </config>
</chocolatey>
"#;
        assert_eq!(
            parse_choco_proxy(input_choco),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_empty = r#"<?xml version="1.0" encoding="utf-8"?>
<chocolatey>
  <config>
    <add key="proxy" value="" />
  </config>
</chocolatey>
"#;
        assert_eq!(parse_choco_proxy(input_empty), None);

        let input_none = r#"<?xml version="1.0" encoding="utf-8"?>
<chocolatey>
  <config>
    <add key="cacheLocation" value="" />
  </config>
</chocolatey>
"#;
        assert_eq!(parse_choco_proxy(input_none), None);
    }

    #[test]
    fn test_parse_colima_proxy() {
        let input_yaml = r#"cpu: 2
memory: 4
env:
  HTTP_PROXY: "http://127.0.0.1:7890"
  HTTPS_PROXY: "http://127.0.0.1:7890"
  ALL_PROXY: "socks5://127.0.0.1:1080"
network:
  address: false
"#;
        assert_eq!(
            parse_colima_proxy(input_yaml),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_none = r#"cpu: 2
memory: 4
network:
  address: false
"#;
        assert_eq!(parse_colima_proxy(input_none), None);
    }

    #[test]
    fn test_parse_minikube_proxy() {
        let input_json = r#"{
    "http_proxy": "http://127.0.0.1:7890",
    "https_proxy": "http://127.0.0.1:7890",
    "no_proxy": "localhost,127.0.0.1"
}"#;
        assert_eq!(
            parse_minikube_proxy(input_json),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_upper = r#"{ "HTTP_PROXY": "127.0.0.1:1080" }"#;
        assert_eq!(
            parse_minikube_proxy(input_upper),
            Some("http://127.0.0.1:1080".to_string())
        );

        let input_none = r#"{ "driver": "docker" }"#;
        assert_eq!(parse_minikube_proxy(input_none), None);

        let input_empty = r#"{ "http_proxy": "" }"#;
        assert_eq!(parse_minikube_proxy(input_empty), None);
    }

    #[test]
    fn test_parse_spack_proxy() {
        let input_yaml = r#"config:
  proxy: "http://127.0.0.1:7890"
  connect_timeout: 10
"#;
        assert_eq!(
            parse_spack_proxy(input_yaml),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_http_proxy = r#"config:
  http_proxy: http://127.0.0.1:1080
"#;
        assert_eq!(
            parse_spack_proxy(input_http_proxy),
            Some("http://127.0.0.1:1080".to_string())
        );

        let input_raw_host_port = "proxy: 127.0.0.1:8888\n";
        assert_eq!(
            parse_spack_proxy(input_raw_host_port),
            Some("http://127.0.0.1:8888".to_string())
        );

        let input_commented = "# proxy: http://127.0.0.1:7890\n";
        assert_eq!(parse_spack_proxy(input_commented), None);

        let input_empty = "config:\n  connect_timeout: 10\n";
        assert_eq!(parse_spack_proxy(input_empty), None);
    }

    #[test]
    fn test_parse_macports_proxy() {
        let input_conf = r#"
# MacPorts configuration file
rsync_server            rsync.macports.org
proxy_http              http://127.0.0.1:7890
proxy_https             http://127.0.0.1:7890
proxy_skip              127.0.0.1, localhost
"#;
        assert_eq!(
            parse_macports_proxy(input_conf),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_socks = "proxy_http socks5://127.0.0.1:1080\n";
        assert_eq!(
            parse_macports_proxy(input_socks),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_raw = "proxy_https 127.0.0.1:8888\n";
        assert_eq!(
            parse_macports_proxy(input_raw),
            Some("http://127.0.0.1:8888".to_string())
        );

        let input_commented = "# proxy_http http://127.0.0.1:7890\n";
        assert_eq!(parse_macports_proxy(input_commented), None);

        let input_none = "rsync_server rsync.macports.org\n";
        assert_eq!(parse_macports_proxy(input_none), None);
    }

    #[test]
    fn test_parse_apt_proxy() {
        let input_apt = r#"
// Configured by proxydoc
Acquire::http::Proxy "http://127.0.0.1:7890";
Acquire::https::Proxy "http://127.0.0.1:7890";
"#;
        assert_eq!(
            parse_apt_proxy(input_apt),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_socks = "Acquire::http::proxy \"socks5h://127.0.0.1:1080\";\n";
        assert_eq!(
            parse_apt_proxy(input_socks),
            Some("socks5h://127.0.0.1:1080".to_string())
        );

        let input_direct = "Acquire::http::Proxy \"DIRECT\";\n";
        assert_eq!(parse_apt_proxy(input_direct), None);

        let input_commented = "// Acquire::http::Proxy \"http://127.0.0.1:7890\";\n";
        assert_eq!(parse_apt_proxy(input_commented), None);

        let input_none = "APT::Install-Recommends \"0\";\n";
        assert_eq!(parse_apt_proxy(input_none), None);
    }

    #[test]
    fn test_parse_dnf_proxy() {
        let input_dnf = r#"
[main]
gpgcheck=1
installonly_limit=3
clean_requirements_on_remove=True
best=False
skip_if_unavailable=True
proxy=http://127.0.0.1:7890
"#;
        assert_eq!(
            parse_dnf_proxy(input_dnf),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_socks = "[main]\nproxy = \"socks5://127.0.0.1:1080\"\n";
        assert_eq!(
            parse_dnf_proxy(input_socks),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_no_proto = "[main]\nproxy=127.0.0.1:8888\n";
        assert_eq!(
            parse_dnf_proxy(input_no_proto),
            Some("http://127.0.0.1:8888".to_string())
        );

        let input_none = "[main]\nproxy=_none_\n";
        assert_eq!(parse_dnf_proxy(input_none), None);

        let input_commented = "[main]\n# proxy=http://127.0.0.1:7890\n; proxy=http://127.0.0.1:1080\n";
        assert_eq!(parse_dnf_proxy(input_commented), None);

        let input_clean = "[main]\ngpgcheck=1\n";
        assert_eq!(parse_dnf_proxy(input_clean), None);
    }

    #[test]
    fn test_parse_zypper_proxy() {
        let input_zypp = r#"
## Proxy settings
proxy = http://127.0.0.1:7890
"#;
        assert_eq!(
            parse_zypper_proxy(input_zypp),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_http = "proxy.http = \"socks5h://127.0.0.1:1080\"\n";
        assert_eq!(
            parse_zypper_proxy(input_http),
            Some("socks5h://127.0.0.1:1080".to_string())
        );

        let input_env_style = "http_proxy = http://proxy.corp.com:8080\n";
        assert_eq!(
            parse_zypper_proxy(input_env_style),
            Some("http://proxy.corp.com:8080".to_string())
        );

        let input_commented = "# proxy = http://127.0.0.1:7890\n; proxy.http = http://127.0.0.1:7890\n";
        assert_eq!(parse_zypper_proxy(input_commented), None);

        let input_none = "proxy = none\n";
        assert_eq!(parse_zypper_proxy(input_none), None);
    }

    #[test]
    fn test_parse_pacman_proxy() {
        let input_pacman = r#"
[options]
XferCommand = /usr/bin/curl -g -L -C - --fail --retry 3 --proxy http://127.0.0.1:7890 -o %o %u
"#;
        assert_eq!(
            parse_pacman_proxy(input_pacman),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_short = "XferCommand = /usr/bin/curl -x socks5://127.0.0.1:1080 -L -C - -f %u -o %o\n";
        assert_eq!(
            parse_pacman_proxy(input_short),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_equal = "XferCommand = /usr/bin/curl --proxy=http://10.0.0.1:8080 -o %o %u\n";
        assert_eq!(
            parse_pacman_proxy(input_equal),
            Some("http://10.0.0.1:8080".to_string())
        );

        let input_commented = "#XferCommand = /usr/bin/curl --proxy http://127.0.0.1:7890\n";
        assert_eq!(parse_pacman_proxy(input_commented), None);

        let input_clean = "[options]\nHoldPkg = pacman glibc\n";
        assert_eq!(parse_pacman_proxy(input_clean), None);
    }

    #[test]
    fn test_parse_pkg_proxy() {
        let input_pkg = r#"
# FreeBSD pkg configuration
http_proxy: "http://127.0.0.1:7890";
"#;
        assert_eq!(
            parse_pkg_proxy(input_pkg),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_eq = "pkg_proxy = socks5://127.0.0.1:1080\n";
        assert_eq!(
            parse_pkg_proxy(input_eq),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_commented = "# http_proxy: \"http://127.0.0.1:7890\";\n";
        assert_eq!(parse_pkg_proxy(input_commented), None);

        let input_none = "http_proxy: none;\n";
        assert_eq!(parse_pkg_proxy(input_none), None);
    }

    #[test]
    fn test_parse_docker_daemon_proxy() {
        let input_daemon = r#"{
  "proxies": {
    "http-proxy": "http://127.0.0.1:7890",
    "https-proxy": "http://127.0.0.1:7890"
  }
}"#;
        assert_eq!(
            parse_docker_daemon_proxy(input_daemon),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_camel = r#"{
  "proxies": {
    "httpProxy": "socks5://127.0.0.1:1080"
  }
}"#;
        assert_eq!(
            parse_docker_daemon_proxy(input_camel),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_clean = r#"{ "log-level": "info" }"#;
        assert_eq!(parse_docker_daemon_proxy(input_clean), None);
    }

    #[test]
    fn test_parse_portage_proxy() {
        let input_portage = r#"
# Gentoo Portage make.conf
CFLAGS="-O2 -pipe"
http_proxy="http://127.0.0.1:7890"
https_proxy="http://127.0.0.1:7890"
"#;
        assert_eq!(
            parse_portage_proxy(input_portage),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_upper = "HTTP_PROXY=\"socks5://127.0.0.1:1080\"\n";
        assert_eq!(
            parse_portage_proxy(input_upper),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_all = "all_proxy='socks5://127.0.0.1:1080'\n";
        assert_eq!(
            parse_portage_proxy(input_all),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_comment = "#http_proxy=\"http://127.0.0.1:7890\"\n";
        assert_eq!(parse_portage_proxy(input_comment), None);

        let input_none = "http_proxy=\"none\"\n";
        assert_eq!(parse_portage_proxy(input_none), None);
    }

    #[test]
    fn test_parse_flatpak_proxy() {
        let input_core = r#"
[core]
repo_version=1
mode=bare-user-only
proxy = http://127.0.0.1:7890

[remote "flathub"]
url=https://dl.flathub.org/repo/
"#;
        assert_eq!(
            parse_flatpak_proxy(input_core),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_http_proxy = r#"
[remote "flathub"]
url=https://dl.flathub.org/repo/
http-proxy = "socks5://127.0.0.1:1080"
"#;
        assert_eq!(
            parse_flatpak_proxy(input_http_proxy),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_comment = "#proxy=http://127.0.0.1:7890\n;http-proxy=http://127.0.0.1:7890\n";
        assert_eq!(parse_flatpak_proxy(input_comment), None);

        let input_clean = "[core]\nrepo_version=1\n";
        assert_eq!(parse_flatpak_proxy(input_clean), None);
    }

    #[test]
    fn test_parse_snap_proxy() {
        let input_systemd = r#"
[Service]
Environment="HTTP_PROXY=http://127.0.0.1:7890"
Environment="HTTPS_PROXY=http://127.0.0.1:7890"
"#;
        assert_eq!(
            parse_snap_proxy(input_systemd),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_env = "http_proxy=socks5://127.0.0.1:1080\nhttps_proxy=socks5://127.0.0.1:1080\n";
        assert_eq!(
            parse_snap_proxy(input_env),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_ini = "[system]\nproxy.http = http://127.0.0.1:8888\nproxy.https = http://127.0.0.1:8888\n";
        assert_eq!(
            parse_snap_proxy(input_ini),
            Some("http://127.0.0.1:8888".to_string())
        );

        let input_comment = "#proxy.http=http://127.0.0.1:7890\n;Environment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        assert_eq!(parse_snap_proxy(input_comment), None);

        let input_empty = "";
        assert_eq!(parse_snap_proxy(input_empty), None);
    }

    #[test]
    fn test_parse_ollama_proxy() {
        let input_systemd = r#"
[Service]
Environment="HTTP_PROXY=http://127.0.0.1:7890"
Environment="HTTPS_PROXY=http://127.0.0.1:7890"
"#;
        assert_eq!(
            parse_ollama_proxy(input_systemd),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_env = "HTTP_PROXY=socks5://127.0.0.1:1080\nHTTPS_PROXY=socks5://127.0.0.1:1080\n";
        assert_eq!(
            parse_ollama_proxy(input_env),
            Some("socks5://127.0.0.1:1080".to_string())
        );

        let input_ollama_proxy = "OLLAMA_PROXY=http://127.0.0.1:8888\n";
        assert_eq!(
            parse_ollama_proxy(input_ollama_proxy),
            Some("http://127.0.0.1:8888".to_string())
        );

        let input_plist = r#"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>EnvironmentVariables</key>
    <dict>
        <key>HTTP_PROXY</key>
        <string>http://127.0.0.1:7890</string>
    </dict>
</dict>
</plist>
"#;
        assert_eq!(
            parse_ollama_proxy(input_plist),
            Some("http://127.0.0.1:7890".to_string())
        );

        let input_comment = "#HTTP_PROXY=http://127.0.0.1:7890\n;Environment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        assert_eq!(parse_ollama_proxy(input_comment), None);

        let input_empty = "";
        assert_eq!(parse_ollama_proxy(input_empty), None);
    }

    #[test]
    fn test_parse_and_clean_apk_repositories() {
        let input_repos = r#"
# Alpine official repositories
https://dl-cdn.alpinelinux.org/alpine/v3.20/main
https://dl-cdn.alpinelinux.org/alpine/v3.20/community
@testing https://mirror.example.com/alpine/edge/testing
https://dead-mirror.example.com/alpine/v3.20/main
"#;
        let repos = parse_apk_repositories(input_repos);
        assert_eq!(repos.len(), 4);
        assert_eq!(repos[0], "https://dl-cdn.alpinelinux.org/alpine/v3.20/main");
        assert_eq!(repos[1], "https://dl-cdn.alpinelinux.org/alpine/v3.20/community");
        assert_eq!(repos[2], "https://mirror.example.com/alpine/edge/testing");
        assert_eq!(repos[3], "https://dead-mirror.example.com/alpine/v3.20/main");

        let dead = ["https://dead-mirror.example.com/alpine/v3.20/main"];
        let (cleaned, modified) = clean_apk_repositories(input_repos, &dead);
        assert!(modified);
        assert!(cleaned.contains("# [proxydoc disabled dead repository] https://dead-mirror.example.com/alpine/v3.20/main"));
        assert!(cleaned.contains("https://dl-cdn.alpinelinux.org/alpine/v3.20/main"));

        let remaining = parse_apk_repositories(&cleaned);
        assert_eq!(remaining.len(), 3);
        assert!(!remaining.contains(&"https://dead-mirror.example.com/alpine/v3.20/main".to_string()));
    }
}
