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

    // PHP Composer Packagist Mirror
    if let Some(comp_path) = get_composer_config_path() {
        if let Ok(content) = std::fs::read_to_string(&comp_path) {
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
        "OLLAMA_HOST",
        "HF_ENDPOINT",
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
            "AI Base URLs (OpenAI/Anthropic/Gemini/DeepSeek/Groq/OpenRouter/Mistral/Perplexity/Cohere/xAI/SiliconFlow/Together/Moonshot/DashScope/DeepInfra/Cerebras/Fireworks/Zhipu/MiniMax/Copilot/SambaNova/Replicate/Qianfan/DeepL/Ollama/HuggingFace)",
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
}
