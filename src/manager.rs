use anyhow::Result;
use colored::*;
use std::path::PathBuf;
use std::process::Command;

use crate::doctor::get_system_proxy;
use crate::scanner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Fish,
    PowerShell,
}

pub fn detect_shell(explicit: Option<&str>) -> ShellType {
    if let Some(s) = explicit {
        match s.to_ascii_lowercase().as_str() {
            "fish" => return ShellType::Fish,
            "pwsh" | "powershell" => return ShellType::PowerShell,
            "bash" | "zsh" | "sh" => return ShellType::Bash,
            _ => {}
        }
    }
    if let Ok(sh) = std::env::var("SHELL") {
        if sh.ends_with("/fish") {
            return ShellType::Fish;
        }
    }
    ShellType::Bash
}

pub fn generate_shell_export(proxy_url: &str, shell: ShellType) -> Vec<String> {
    let bypass = "localhost,127.0.0.1,::1,localaddress,.local";
    let vars = [
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
    ];
    let no_proxy_vars = ["no_proxy", "NO_PROXY"];

    match shell {
        ShellType::Bash => {
            let mut lines = Vec::new();
            for v in vars {
                lines.push(format!("export {}=\"{}\"", v, proxy_url));
            }
            for v in no_proxy_vars {
                lines.push(format!("export {}=\"{}\"", v, bypass));
            }
            lines
        }
        ShellType::Fish => {
            let mut lines = Vec::new();
            for v in vars {
                lines.push(format!("set -gx {} \"{}\";", v, proxy_url));
            }
            for v in no_proxy_vars {
                lines.push(format!("set -gx {} \"{}\";", v, bypass));
            }
            lines
        }
        ShellType::PowerShell => {
            let mut lines = Vec::new();
            for v in vars {
                lines.push(format!("$env:{}=\"{}\"", v, proxy_url));
            }
            for v in no_proxy_vars {
                lines.push(format!("$env:{}=\"{}\"", v, bypass));
            }
            lines
        }
    }
}

pub fn generate_shell_unset(shell: ShellType) -> Vec<String> {
    let vars = [
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "no_proxy",
        "NO_PROXY",
    ];

    match shell {
        ShellType::Bash => vars.iter().map(|v| format!("unset {}", v)).collect(),
        ShellType::Fish => vars.iter().map(|v| format!("set -e {};", v)).collect(),
        ShellType::PowerShell => vars.iter().map(|v| format!("$env:{}=$null", v)).collect(),
    }
}

pub fn apply_docker_proxy_json(root: &mut serde_json::Value, proxy_url: &str) {
    if !root.is_object() {
        *root = serde_json::json!({});
    }

    let proxies = root
        .as_object_mut()
        .unwrap()
        .entry("proxies")
        .or_insert_with(|| serde_json::json!({}));
    if !proxies.is_object() {
        *proxies = serde_json::json!({});
    }

    let default_entry = proxies
        .as_object_mut()
        .unwrap()
        .entry("default")
        .or_insert_with(|| serde_json::json!({}));
    if !default_entry.is_object() {
        *default_entry = serde_json::json!({});
    }

    let default_map = default_entry.as_object_mut().unwrap();
    default_map.insert(
        "httpProxy".to_string(),
        serde_json::Value::String(proxy_url.to_string()),
    );
    default_map.insert(
        "httpsProxy".to_string(),
        serde_json::Value::String(proxy_url.to_string()),
    );
    default_map.insert(
        "noProxy".to_string(),
        serde_json::Value::String("localhost,127.0.0.1,::1".to_string()),
    );
}

pub fn remove_docker_proxy_json(root: &mut serde_json::Value) -> bool {
    let mut removed = false;
    if let Some(proxies) = root.get_mut("proxies").and_then(|p| p.as_object_mut()) {
        if proxies.remove("default").is_some() {
            removed = true;
        }
    }
    if removed {
        if let Some(proxies) = root.get("proxies") {
            if proxies.as_object().map_or(false, |m| m.is_empty()) {
                root.as_object_mut().unwrap().remove("proxies");
            }
        }
    }
    removed
}

pub fn apply_cargo_proxy_toml(content: &str, proxy_url: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut in_http = false;
    let mut http_seen = false;
    let mut proxy_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') {
            if in_http && !proxy_set {
                lines.push(format!("proxy = \"{}\"", proxy_url));
                proxy_set = true;
            }
            in_http = trim == "[http]";
            if in_http {
                http_seen = true;
            }
            lines.push(line.to_string());
            continue;
        }

        if in_http {
            if let Some((k, _)) = trim.split_once('=') {
                if k.trim() == "proxy" {
                    lines.push(format!("proxy = \"{}\"", proxy_url));
                    proxy_set = true;
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    if in_http && !proxy_set {
        lines.push(format!("proxy = \"{}\"", proxy_url));
    }

    if !http_seen {
        if !lines.is_empty() && !lines.last().map(|l| l.trim().is_empty()).unwrap_or(true) {
            lines.push(String::new());
        }
        lines.push("[http]".to_string());
        lines.push(format!("proxy = \"{}\"", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_cargo_proxy_toml(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut in_http = false;
    let mut removed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') {
            in_http = trim == "[http]";
            lines.push(line.to_string());
            continue;
        }

        if in_http {
            if let Some((k, _)) = trim.split_once('=') {
                if k.trim() == "proxy" {
                    removed = true;
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_cargo_proxy(proxy_url: &str) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let cargo_dir = home.join(".cargo");
    std::fs::create_dir_all(&cargo_dir)?;
    let config_path = if cargo_dir.join("config.toml").exists() {
        cargo_dir.join("config.toml")
    } else if cargo_dir.join("config").exists() {
        cargo_dir.join("config")
    } else {
        cargo_dir.join("config.toml")
    };

    let content = if config_path.exists() {
        std::fs::read_to_string(&config_path)?
    } else {
        String::new()
    };

    let new_content = apply_cargo_proxy_toml(&content, proxy_url);
    std::fs::write(&config_path, new_content)?;
    Ok(())
}

pub fn unset_cargo_proxy() -> Result<bool> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Ok(false),
    };
    let cargo_dir = home.join(".cargo");
    let mut modified = false;

    for file_name in &["config.toml", "config"] {
        let path = cargo_dir.join(file_name);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_cargo_proxy_toml(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    modified = true;
                }
            }
        }
    }

    Ok(modified)
}

pub fn set_docker_proxy(proxy_url: &str) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let docker_dir = home.join(".docker");
    std::fs::create_dir_all(&docker_dir)?;
    let config_path = docker_dir.join("config.json");
    let mut root: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    apply_docker_proxy_json(&mut root, proxy_url);

    let output = serde_json::to_string_pretty(&root)?;
    std::fs::write(&config_path, output)?;
    Ok(())
}

pub fn unset_docker_proxy() -> Result<bool> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Ok(false),
    };
    let config_path = home.join(".docker").join("config.json");
    if !config_path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&config_path)?;
    let mut root: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Ok(false),
    };

    let removed = remove_docker_proxy_json(&mut root);
    if removed {
        let output = serde_json::to_string_pretty(&root)?;
        std::fs::write(&config_path, output)?;
    }
    Ok(removed)
}

pub fn extract_host_port(url_str: &str) -> (String, u16) {
    if let Ok(url) = url::Url::parse(url_str) {
        let host = url.host_str().unwrap_or("127.0.0.1").to_string();
        let port = url.port().unwrap_or(7890);
        return (host, port);
    }
    if let Some((h, p)) = url_str.split_once(':') {
        let host = if h.is_empty() || h == "localhost" {
            "127.0.0.1".to_string()
        } else {
            h.trim_start_matches("http://")
                .trim_start_matches("https://")
                .to_string()
        };
        let port = p.parse().unwrap_or(7890);
        return (host, port);
    }
    ("127.0.0.1".to_string(), 7890)
}

pub fn apply_gradle_proxy_properties(content: &str, host: &str, port: u16) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut updated_keys = std::collections::HashSet::new();

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with('!') {
            lines.push(line.to_string());
            continue;
        }
        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim();
            match key {
                "systemProp.http.proxyHost" => {
                    lines.push(format!("systemProp.http.proxyHost={}", host));
                    updated_keys.insert("http.proxyHost");
                }
                "systemProp.http.proxyPort" => {
                    lines.push(format!("systemProp.http.proxyPort={}", port));
                    updated_keys.insert("http.proxyPort");
                }
                "systemProp.https.proxyHost" => {
                    lines.push(format!("systemProp.https.proxyHost={}", host));
                    updated_keys.insert("https.proxyHost");
                }
                "systemProp.https.proxyPort" => {
                    lines.push(format!("systemProp.https.proxyPort={}", port));
                    updated_keys.insert("https.proxyPort");
                }
                "systemProp.http.nonProxyHosts" => {
                    lines.push(
                        "systemProp.http.nonProxyHosts=localhost|127.0.0.1|*.local".to_string(),
                    );
                    updated_keys.insert("http.nonProxyHosts");
                }
                _ => lines.push(line.to_string()),
            }
        } else {
            lines.push(line.to_string());
        }
    }

    if !updated_keys.contains("http.proxyHost") {
        lines.push(format!("systemProp.http.proxyHost={}", host));
    }
    if !updated_keys.contains("http.proxyPort") {
        lines.push(format!("systemProp.http.proxyPort={}", port));
    }
    if !updated_keys.contains("https.proxyHost") {
        lines.push(format!("systemProp.https.proxyHost={}", host));
    }
    if !updated_keys.contains("https.proxyPort") {
        lines.push(format!("systemProp.https.proxyPort={}", port));
    }
    if !updated_keys.contains("http.nonProxyHosts") {
        lines.push("systemProp.http.nonProxyHosts=localhost|127.0.0.1|*.local".to_string());
    }

    lines.join("\n") + "\n"
}

pub fn remove_gradle_proxy_properties(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut removed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with("systemProp.http.proxyHost")
            || trim.starts_with("systemProp.http.proxyPort")
            || trim.starts_with("systemProp.https.proxyHost")
            || trim.starts_with("systemProp.https.proxyPort")
            || trim.starts_with("systemProp.http.nonProxyHosts")
        {
            removed = true;
            continue;
        }
        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_gradle_proxy(proxy_url: &str) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let gradle_dir = home.join(".gradle");
    std::fs::create_dir_all(&gradle_dir)?;
    let prop_path = gradle_dir.join("gradle.properties");
    let content = if prop_path.exists() {
        std::fs::read_to_string(&prop_path)?
    } else {
        String::new()
    };

    let (host, port) = extract_host_port(proxy_url);
    let new_content = apply_gradle_proxy_properties(&content, &host, port);
    std::fs::write(&prop_path, new_content)?;
    Ok(())
}

pub fn unset_gradle_proxy() -> Result<bool> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Ok(false),
    };
    let prop_path = home.join(".gradle").join("gradle.properties");
    if !prop_path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&prop_path)?;
    let (new_content, removed) = remove_gradle_proxy_properties(&content);
    if removed {
        std::fs::write(&prop_path, new_content)?;
    }
    Ok(removed)
}

pub fn apply_maven_proxy_xml(content: &str, host: &str, port: u16) -> String {
    let proxy_snippet = format!(
        "    <proxy>\n      <id>proxydoc-proxy</id>\n      <active>true</active>\n      <protocol>http</protocol>\n      <host>{}</host>\n      <port>{}</port>\n      <nonProxyHosts>localhost|127.0.0.1|*.local</nonProxyHosts>\n    </proxy>",
        host, port
    );

    if content.trim().is_empty() {
        return format!(
            "<settings xmlns=\"http://maven.apache.org/SETTINGS/1.0.0\"\n  xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"\n  xsi:schemaLocation=\"http://maven.apache.org/SETTINGS/1.0.0\n                      https://maven.apache.org/xsd/settings-1.0.0.xsd\">\n  <proxies>\n{}\n  </proxies>\n</settings>\n",
            proxy_snippet
        );
    }

    if let Some(doc_idx) = content.find("<id>proxydoc-proxy</id>") {
        if let Some(p_start) = content[..doc_idx].rfind("<proxy>") {
            if let Some(p_end) = content[doc_idx..].find("</proxy>") {
                let end_pos = doc_idx + p_end + 8;
                let mut res = String::new();
                res.push_str(&content[..p_start]);
                res.push_str(&proxy_snippet.trim());
                res.push_str(&content[end_pos..]);
                return res;
            }
        }
    }

    if let Some(proxies_end) = content.find("</proxies>") {
        let mut res = String::new();
        res.push_str(&content[..proxies_end]);
        res.push_str(&proxy_snippet);
        res.push_str("\n  ");
        res.push_str(&content[proxies_end..]);
        return res;
    }

    if let Some(proxies_sc) = content.find("<proxies/>") {
        let mut res = String::new();
        res.push_str(&content[..proxies_sc]);
        res.push_str("<proxies>\n");
        res.push_str(&proxy_snippet);
        res.push_str("\n  </proxies>");
        res.push_str(&content[proxies_sc + 10..]);
        return res;
    }

    if let Some(settings_end) = content.find("</settings>") {
        let mut res = String::new();
        res.push_str(&content[..settings_end]);
        res.push_str("  <proxies>\n");
        res.push_str(&proxy_snippet);
        res.push_str("\n  </proxies>\n");
        res.push_str(&content[settings_end..]);
        return res;
    }

    format!(
        "<settings>\n  <proxies>\n{}\n  </proxies>\n{}\n</settings>\n",
        proxy_snippet, content
    )
}

pub fn remove_maven_proxy_xml(content: &str) -> (String, bool) {
    let mut modified = false;
    let mut res = content.to_string();

    if let Some(doc_idx) = res.find("<id>proxydoc-proxy</id>") {
        if let Some(p_start) = res[..doc_idx].rfind("<proxy>") {
            if let Some(p_end) = res[doc_idx..].find("</proxy>") {
                let end_pos = doc_idx + p_end + 8;
                res = format!(
                    "{}{}",
                    &res[..p_start],
                    res[end_pos..].trim_start_matches('\n')
                );
                modified = true;
            }
        }
    }

    if !modified {
        if let Some(_) = res.find("<active>true</active>") {
            res = res.replacen("<active>true</active>", "<active>false</active>", 1);
            modified = true;
        }
    }

    (res, modified)
}

pub fn set_maven_proxy(proxy_url: &str) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let m2_dir = home.join(".m2");
    std::fs::create_dir_all(&m2_dir)?;
    let settings_path = m2_dir.join("settings.xml");
    let content = if settings_path.exists() {
        std::fs::read_to_string(&settings_path)?
    } else {
        String::new()
    };

    let (host, port) = extract_host_port(proxy_url);
    let new_content = apply_maven_proxy_xml(&content, &host, port);
    std::fs::write(&settings_path, new_content)?;
    Ok(())
}

pub fn unset_maven_proxy() -> Result<bool> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Ok(false),
    };
    let settings_path = home.join(".m2").join("settings.xml");
    if !settings_path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&settings_path)?;
    let (new_content, removed) = remove_maven_proxy_xml(&content);
    if removed {
        std::fs::write(&settings_path, new_content)?;
    }
    Ok(removed)
}

pub fn apply_yarn_proxy_yarnrc(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut has_proxy = false;
    let mut has_https_proxy = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("proxy ") || lower.starts_with("proxy=") {
            lines.push(format!("proxy \"{}\"", proxy_url));
            has_proxy = true;
        } else if lower.starts_with("https-proxy ") || lower.starts_with("https-proxy=") {
            lines.push(format!("https-proxy \"{}\"", proxy_url));
            has_https_proxy = true;
        } else {
            lines.push(line.to_string());
        }
    }

    if !has_proxy {
        lines.push(format!("proxy \"{}\"", proxy_url));
    }
    if !has_https_proxy {
        lines.push(format!("https-proxy \"{}\"", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_yarn_proxy_yarnrc(content: &str) -> (String, bool) {
    let mut removed = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("proxy ")
            || lower.starts_with("https-proxy ")
            || lower.starts_with("proxy=")
            || lower.starts_with("https-proxy=")
        {
            removed = true;
        } else {
            lines.push(line.to_string());
        }
    }

    (lines.join("\n") + "\n", removed)
}

pub fn apply_yarn_proxy_yml(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut has_http = false;
    let mut has_https = false;

    for line in content.lines() {
        let trim = line.trim();
        if let Some((k, _)) = trim.split_once(':') {
            let k_trim = k.trim();
            if k_trim == "httpProxy" {
                lines.push(format!("httpProxy: \"{}\"", proxy_url));
                has_http = true;
                continue;
            } else if k_trim == "httpsProxy" {
                lines.push(format!("httpsProxy: \"{}\"", proxy_url));
                has_https = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !has_http {
        lines.push(format!("httpProxy: \"{}\"", proxy_url));
    }
    if !has_https {
        lines.push(format!("httpsProxy: \"{}\"", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_yarn_proxy_yml(content: &str) -> (String, bool) {
    let mut removed = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        let trim = line.trim();
        if let Some((k, _)) = trim.split_once(':') {
            let k_trim = k.trim();
            if k_trim == "httpProxy" || k_trim == "httpsProxy" {
                removed = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_yarn_proxy(proxy_url: &str) -> Result<()> {
    if let Some(home) = dirs::home_dir() {
        let yml_path = home.join(".yarnrc.yml");
        if yml_path.exists() {
            let content = std::fs::read_to_string(&yml_path).unwrap_or_default();
            let new_content = apply_yarn_proxy_yml(&content, proxy_url);
            std::fs::write(&yml_path, new_content)?;
        }

        let rc_path = home.join(".yarnrc");
        let content = std::fs::read_to_string(&rc_path).unwrap_or_default();
        let new_content = apply_yarn_proxy_yarnrc(&content, proxy_url);
        std::fs::write(&rc_path, new_content)?;
    }
    let _ = Command::new("yarn")
        .args(["config", "set", "proxy", proxy_url])
        .output();
    let _ = Command::new("yarn")
        .args(["config", "set", "https-proxy", proxy_url])
        .output();
    Ok(())
}

pub fn unset_yarn_proxy() -> Result<bool> {
    let mut removed = false;
    if let Some(home) = dirs::home_dir() {
        let yml_path = home.join(".yarnrc.yml");
        if yml_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&yml_path) {
                let (new_content, r) = remove_yarn_proxy_yml(&content);
                if r {
                    std::fs::write(&yml_path, new_content)?;
                    removed = true;
                }
            }
        }

        let rc_path = home.join(".yarnrc");
        if rc_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&rc_path) {
                let (new_content, r) = remove_yarn_proxy_yarnrc(&content);
                if r {
                    std::fs::write(&rc_path, new_content)?;
                    removed = true;
                }
            }
        }
    }
    let _ = Command::new("yarn")
        .args(["config", "delete", "proxy"])
        .output();
    let _ = Command::new("yarn")
        .args(["config", "delete", "https-proxy"])
        .output();
    Ok(removed)
}

pub fn apply_condarc_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut in_proxy_servers = false;
    let mut proxy_section_inserted = false;

    let proxy_block = format!(
        "proxy_servers:\n  http: {}\n  https: {}",
        proxy_url, proxy_url
    );

    for line in content.lines() {
        let trim = line.trim();
        if !line.starts_with(' ') && !line.starts_with('\t') {
            if in_proxy_servers {
                in_proxy_servers = false;
            }
            if trim.starts_with("proxy_servers:") {
                in_proxy_servers = true;
                lines.push(proxy_block.clone());
                proxy_section_inserted = true;
                continue;
            }
        } else if in_proxy_servers {
            continue;
        }
        lines.push(line.to_string());
    }

    if !proxy_section_inserted {
        if !lines.is_empty() && !lines.last().map(|l| l.trim().is_empty()).unwrap_or(true) {
            lines.push(String::new());
        }
        lines.push(proxy_block);
    }

    lines.join("\n") + "\n"
}

pub fn remove_condarc_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut in_proxy_servers = false;
    let mut removed = false;

    for line in content.lines() {
        let trim = line.trim();
        if !line.starts_with(' ') && !line.starts_with('\t') {
            if in_proxy_servers {
                in_proxy_servers = false;
            }
            if trim.starts_with("proxy_servers:") {
                in_proxy_servers = true;
                removed = true;
                continue;
            }
        } else if in_proxy_servers {
            continue;
        }
        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn get_condarc_target_path() -> Result<std::path::PathBuf> {
    if let Ok(condarc_env) = std::env::var("CONDARC") {
        let p = std::path::PathBuf::from(condarc_env.trim());
        return Ok(p);
    }
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    Ok(home.join(".condarc"))
}

pub fn set_conda_proxy(proxy_url: &str) -> Result<()> {
    let path = get_condarc_target_path()?;
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_condarc_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(())
}

pub fn unset_conda_proxy() -> Result<bool> {
    let path = match get_condarc_target_path() {
        Ok(p) => p,
        Err(_) => return Ok(false),
    };
    if !path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&path)?;
    let (new_content, removed) = remove_condarc_proxy(&content);
    if removed {
        std::fs::write(&path, new_content)?;
    }
    Ok(removed)
}

pub fn apply_pnpmrc_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut has_proxy = false;
    let mut has_https_proxy = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }
        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "proxy" || key == "http-proxy" {
                lines.push(format!("proxy={}", proxy_url));
                has_proxy = true;
                continue;
            } else if key == "https-proxy" {
                lines.push(format!("https-proxy={}", proxy_url));
                has_https_proxy = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !has_proxy {
        lines.push(format!("proxy={}", proxy_url));
    }
    if !has_https_proxy {
        lines.push(format!("https-proxy={}", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_pnpmrc_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut removed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }
        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "proxy" || key == "http-proxy" || key == "https-proxy" {
                removed = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_pnpm_proxy(proxy_url: &str) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let pnpmrc_path = home.join(".pnpmrc");
    let content = if pnpmrc_path.exists() {
        std::fs::read_to_string(&pnpmrc_path)?
    } else {
        String::new()
    };
    let new_content = apply_pnpmrc_proxy(&content, proxy_url);
    std::fs::write(&pnpmrc_path, new_content)?;

    let _ = Command::new("pnpm")
        .args(["config", "set", "proxy", proxy_url])
        .output();
    let _ = Command::new("pnpm")
        .args(["config", "set", "https-proxy", proxy_url])
        .output();

    Ok(())
}

pub fn unset_pnpm_proxy() -> Result<bool> {
    let mut removed = false;
    if let Some(home) = dirs::home_dir() {
        let pnpmrc_path = home.join(".pnpmrc");
        if pnpmrc_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&pnpmrc_path) {
                let (new_content, r) = remove_pnpmrc_proxy(&content);
                if r {
                    std::fs::write(&pnpmrc_path, new_content)?;
                    removed = true;
                }
            }
        }
    }
    let _ = Command::new("pnpm")
        .args(["config", "delete", "proxy"])
        .output();
    let _ = Command::new("pnpm")
        .args(["config", "delete", "https-proxy"])
        .output();
    Ok(removed)
}

pub fn apply_bun_proxy_toml(content: &str, proxy_url: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut in_install = false;
    let mut has_install_section = false;
    let mut proxy_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let sec = trim.trim_matches(|c| c == '[' || c == ']').trim();
            if in_install && !proxy_set {
                lines.push(format!("proxy = \"{}\"", proxy_url));
                proxy_set = true;
            }
            in_install = sec == "install";
            if in_install {
                has_install_section = true;
            }
            lines.push(line.to_string());
            continue;
        }

        if in_install {
            if let Some((k, _)) = trim.split_once('=') {
                if k.trim() == "proxy" {
                    lines.push(format!("proxy = \"{}\"", proxy_url));
                    proxy_set = true;
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    if in_install && !proxy_set {
        lines.push(format!("proxy = \"{}\"", proxy_url));
        proxy_set = true;
    }

    if !proxy_set {
        if !has_install_section {
            if !lines.is_empty() && !lines.last().map(|l| l.trim().is_empty()).unwrap_or(true) {
                lines.push(String::new());
            }
            lines.push("[install]".to_string());
            lines.push(format!("proxy = \"{}\"", proxy_url));
        }
    }

    lines.join("\n") + "\n"
}

pub fn remove_bun_proxy_toml(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut in_install = false;
    let mut removed = false;
    let has_install = content.lines().any(|l| {
        l.trim().starts_with('[')
            && l.trim().trim_matches(|c| c == '[' || c == ']').trim() == "install"
    });

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            in_install = trim.trim_matches(|c| c == '[' || c == ']').trim() == "install";
            lines.push(line.to_string());
            continue;
        }

        if in_install || (!has_install && !trim.starts_with('[')) {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim();
                if key == "proxy" || key == "http_proxy" || key == "https_proxy" {
                    removed = true;
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_bun_proxy(proxy_url: &str) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let bunfig_path = home.join(".bunfig.toml");
    let content = if bunfig_path.exists() {
        std::fs::read_to_string(&bunfig_path)?
    } else {
        String::new()
    };
    let new_content = apply_bun_proxy_toml(&content, proxy_url);
    std::fs::write(&bunfig_path, new_content)?;
    Ok(())
}

pub fn unset_bun_proxy() -> Result<bool> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Ok(false),
    };
    let bunfig_path = home.join(".bunfig.toml");
    if !bunfig_path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&bunfig_path)?;
    let (new_content, removed) = remove_bun_proxy_toml(&content);
    if removed {
        std::fs::write(&bunfig_path, new_content)?;
    }
    Ok(removed)
}

pub fn apply_vscode_proxy_json(root: &mut serde_json::Value, proxy_url: &str) {
    if !root.is_object() {
        *root = serde_json::json!({});
    }
    let map = root.as_object_mut().unwrap();
    map.insert(
        "http.proxy".to_string(),
        serde_json::Value::String(proxy_url.to_string()),
    );
    map.insert(
        "http.proxySupport".to_string(),
        serde_json::Value::String("override".to_string()),
    );
}

pub fn remove_vscode_proxy_json(root: &mut serde_json::Value) -> bool {
    let mut removed = false;
    if let Some(map) = root.as_object_mut() {
        if map.remove("http.proxy").is_some() {
            removed = true;
        }
        if map.remove("http.proxySupport").is_some() {
            removed = true;
        }
    }
    removed
}

pub fn set_vscode_proxy(proxy_url: &str) -> Result<Vec<String>> {
    let mut updated = Vec::new();
    let ides = crate::doctor::get_ide_settings_paths();
    let mut found_existing = false;

    for (name, path) in &ides {
        if path.exists() {
            found_existing = true;
            let content = std::fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
            let cleaned = crate::doctor::clean_jsonc(&content);
            let mut root: serde_json::Value =
                serde_json::from_str(&cleaned).unwrap_or_else(|_| serde_json::json!({}));
            apply_vscode_proxy_json(&mut root, proxy_url);
            let out = serde_json::to_string_pretty(&root)?;
            std::fs::write(path, out)?;
            updated.push(name.to_string());
        }
    }

    if !found_existing {
        if let Some((name, path)) = ides.first() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut root = serde_json::json!({});
            apply_vscode_proxy_json(&mut root, proxy_url);
            let out = serde_json::to_string_pretty(&root)?;
            std::fs::write(path, out)?;
            updated.push(name.to_string());
        }
    }

    Ok(updated)
}

pub fn unset_vscode_proxy() -> Result<Vec<String>> {
    let mut removed = Vec::new();
    let ides = crate::doctor::get_ide_settings_paths();

    for (name, path) in &ides {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                let cleaned = crate::doctor::clean_jsonc(&content);
                if let Ok(mut root) = serde_json::from_str::<serde_json::Value>(&cleaned) {
                    if remove_vscode_proxy_json(&mut root) {
                        let out = serde_json::to_string_pretty(&root)?;
                        std::fs::write(path, out)?;
                        removed.push(name.to_string());
                    }
                }
            }
        }
    }

    Ok(removed)
}

pub fn apply_gh_config_proxy(content: &str, proxy_url: &str) -> String {
    let mut replaced = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && trim.starts_with("http_proxy:") {
            lines.push(format!("http_proxy: {}", proxy_url));
            replaced = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !replaced {
        lines.push(format!("http_proxy: {}", proxy_url));
    }
    lines.join("\n") + "\n"
}

pub fn remove_gh_config_proxy(content: &str) -> (String, bool) {
    let mut modified = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && trim.starts_with("http_proxy:") {
            modified = true;
            continue;
        }
        lines.push(line.to_string());
    }
    (lines.join("\n") + "\n", modified)
}

pub fn set_gh_proxy(proxy_url: &str) -> Result<()> {
    let path = crate::doctor::get_gh_config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_gh_config_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    let _ = Command::new("gh")
        .args(["config", "set", "http_proxy", proxy_url])
        .output();
    Ok(())
}

pub fn unset_gh_proxy() -> Result<bool> {
    let path = crate::doctor::get_gh_config_path();
    let mut removed = false;
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_c, m) = remove_gh_config_proxy(&content);
            if m {
                std::fs::write(&path, new_c)?;
                removed = true;
            }
        }
    }
    let _ = Command::new("gh")
        .args(["config", "set", "http_proxy", ""])
        .output();
    Ok(removed)
}

pub fn apply_glab_config_proxy(content: &str, proxy_url: &str) -> String {
    let mut replaced = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && (trim.starts_with("glab_proxy:") || trim.starts_with("http_proxy:")) {
            lines.push(format!("glab_proxy: {}", proxy_url));
            replaced = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !replaced {
        lines.push(format!("glab_proxy: {}", proxy_url));
    }
    lines.join("\n") + "\n"
}

pub fn remove_glab_config_proxy(content: &str) -> (String, bool) {
    let mut modified = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && (trim.starts_with("glab_proxy:") || trim.starts_with("http_proxy:")) {
            modified = true;
            continue;
        }
        lines.push(line.to_string());
    }
    (lines.join("\n") + "\n", modified)
}

pub fn set_glab_proxy(proxy_url: &str) -> Result<()> {
    let path = crate::doctor::get_glab_config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_glab_config_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    let _ = Command::new("glab")
        .args(["config", "set", "glab_proxy", proxy_url])
        .output();
    Ok(())
}

pub fn unset_glab_proxy() -> Result<bool> {
    let path = crate::doctor::get_glab_config_path();
    let mut removed = false;
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_c, m) = remove_glab_config_proxy(&content);
            if m {
                std::fs::write(&path, new_c)?;
                removed = true;
            }
        }
    }
    let _ = Command::new("glab")
        .args(["config", "set", "glab_proxy", ""])
        .output();
    Ok(removed)
}

pub fn apply_jetbrains_proxy_xml(content: &str, host: &str, port: u16, is_socks: bool) -> String {
    let proxy_type_val = if is_socks { "2" } else { "1" };
    let new_options = format!(
        "    <option name=\"PROXY_TYPE_NUM\" value=\"{}\" />\n    <option name=\"USE_HTTP_PROXY\" value=\"true\" />\n    <option name=\"PROXY_HOST\" value=\"{}\" />\n    <option name=\"PROXY_PORT\" value=\"{}\" />\n    <option name=\"USE_PROXY_PAC\" value=\"false\" />",
        proxy_type_val, host, port
    );

    if content.trim().is_empty() {
        return format!(
            "<application>\n  <component name=\"HttpConfigurable\">\n{}\n  </component>\n</application>\n",
            new_options
        );
    }

    let mut out_lines = Vec::new();
    let mut in_component = false;
    let mut inserted = false;

    for line in content.lines() {
        if line.contains("<component name=\"HttpConfigurable\"") {
            in_component = true;
            out_lines.push(line.to_string());
            out_lines.push(new_options.clone());
            inserted = true;
            continue;
        }
        if in_component {
            if line.contains("</component>") {
                in_component = false;
                out_lines.push(line.to_string());
                continue;
            }
            let trim = line.trim();
            if trim.contains("PROXY_TYPE_NUM")
                || trim.contains("USE_HTTP_PROXY")
                || trim.contains("PROXY_HOST")
                || trim.contains("PROXY_PORT")
                || trim.contains("USE_PROXY_PAC")
                || trim.contains("PAC_URL")
            {
                continue;
            }
        }
        out_lines.push(line.to_string());
    }

    if !inserted {
        if let Some(pos) = content.rfind("</application>") {
            let before = &content[..pos];
            let after = &content[pos..];
            return format!(
                "{}  <component name=\"HttpConfigurable\">\n{}\n  </component>\n{}",
                before, new_options, after
            );
        } else {
            return format!(
                "<application>\n  <component name=\"HttpConfigurable\">\n{}\n  </component>\n</application>\n",
                new_options
            );
        }
    }

    out_lines.join("\n") + "\n"
}

pub fn deactivate_jetbrains_proxy_xml(content: &str) -> (String, bool) {
    let mut modified = false;
    let mut out_lines = Vec::new();
    for line in content.lines() {
        if line.contains("name=\"USE_HTTP_PROXY\"") && line.contains("value=\"true\"") {
            out_lines.push(line.replace("value=\"true\"", "value=\"false\""));
            modified = true;
        } else if line.contains("name=\"USE_PROXY_PAC\"") && line.contains("value=\"true\"") {
            out_lines.push(line.replace("value=\"true\"", "value=\"false\""));
            modified = true;
        } else {
            out_lines.push(line.to_string());
        }
    }
    (out_lines.join("\n") + "\n", modified)
}

pub fn set_jetbrains_proxy(proxy_url: &str) -> Result<Vec<String>> {
    let (host, port) = extract_host_port(proxy_url);
    let is_socks = proxy_url.to_ascii_lowercase().starts_with("socks");
    let mut updated = Vec::new();

    let configs = crate::doctor::get_jetbrains_proxies();
    for cfg in configs {
        let content = std::fs::read_to_string(&cfg.path).unwrap_or_default();
        let new_c = apply_jetbrains_proxy_xml(&content, &host, port, is_socks);
        if std::fs::write(&cfg.path, new_c).is_ok() {
            updated.push(cfg.ide_name);
        }
    }
    Ok(updated)
}

pub fn unset_jetbrains_proxy() -> Result<Vec<String>> {
    let mut deactivated = Vec::new();
    let configs = crate::doctor::get_jetbrains_proxies();
    for cfg in configs {
        if cfg.use_proxy || cfg.use_pac {
            if let Ok(content) = std::fs::read_to_string(&cfg.path) {
                let (new_c, m) = deactivate_jetbrains_proxy_xml(&content);
                if m {
                    let _ = std::fs::write(&cfg.path, new_c);
                    deactivated.push(cfg.ide_name);
                }
            }
        }
    }
    Ok(deactivated)
}

pub fn apply_nuget_proxy_xml(content: &str, proxy_url: &str) -> String {
    let proxy_line = format!("    <add key=\"http_proxy\" value=\"{}\" />", proxy_url);

    if content.trim().is_empty() {
        return format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<configuration>\n  <config>\n{}\n  </config>\n</configuration>\n",
            proxy_line
        );
    }

    let mut replaced = false;
    let mut out_lines = Vec::new();

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();
        if !trim.starts_with("<!--")
            && trim.starts_with("<add")
            && (lower.contains("key=\"http_proxy\"")
                || lower.contains("key='http_proxy'")
                || lower.contains("key=\"https_proxy\"")
                || lower.contains("key='https_proxy'")
                || (lower.contains("http_proxy") && lower.contains("key=")))
        {
            if !replaced {
                let leading_ws = line.len() - line.trim_start().len();
                let indent = if leading_ws > 0 {
                    &line[..leading_ws]
                } else {
                    "    "
                };
                out_lines.push(format!(
                    "{}<add key=\"http_proxy\" value=\"{}\" />",
                    indent, proxy_url
                ));
                replaced = true;
            }
        } else {
            out_lines.push(line.to_string());
        }
    }

    if replaced {
        let mut res = out_lines.join("\n");
        if content.ends_with('\n') {
            res.push('\n');
        }
        return res;
    }

    let text = out_lines.join("\n");
    if let Some(config_idx) = text.find("<config>") {
        let insert_pos = config_idx + 8;
        let mut res = String::new();
        res.push_str(&text[..insert_pos]);
        res.push('\n');
        res.push_str(&proxy_line);
        res.push_str(&text[insert_pos..]);
        if content.ends_with('\n') && !res.ends_with('\n') {
            res.push('\n');
        }
        return res;
    }

    if let Some(config_sc) = text.find("<config/>") {
        let mut res = String::new();
        res.push_str(&text[..config_sc]);
        res.push_str("<config>\n");
        res.push_str(&proxy_line);
        res.push_str("\n  </config>");
        res.push_str(&text[config_sc + 9..]);
        if content.ends_with('\n') && !res.ends_with('\n') {
            res.push('\n');
        }
        return res;
    }

    if let Some(config_sc2) = text.find("<config />") {
        let mut res = String::new();
        res.push_str(&text[..config_sc2]);
        res.push_str("<config>\n");
        res.push_str(&proxy_line);
        res.push_str("\n  </config>");
        res.push_str(&text[config_sc2 + 10..]);
        if content.ends_with('\n') && !res.ends_with('\n') {
            res.push('\n');
        }
        return res;
    }

    if let Some(conf_end) = text.find("</configuration>") {
        let mut res = String::new();
        res.push_str(&text[..conf_end]);
        res.push_str("  <config>\n");
        res.push_str(&proxy_line);
        res.push_str("\n  </config>\n");
        res.push_str(&text[conf_end..]);
        if content.ends_with('\n') && !res.ends_with('\n') {
            res.push('\n');
        }
        return res;
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<configuration>\n  <config>\n{}\n  </config>\n{}\n</configuration>\n",
        proxy_line, text
    )
}

pub fn remove_nuget_proxy_xml(content: &str) -> (String, bool) {
    let mut modified = false;
    let mut out_lines = Vec::new();

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();
        if !trim.starts_with("<!--")
            && trim.starts_with("<add")
            && (lower.contains("http_proxy") || lower.contains("https_proxy"))
        {
            modified = true;
        } else {
            out_lines.push(line.to_string());
        }
    }

    let mut res = out_lines.join("\n");
    if content.ends_with('\n') && !res.is_empty() {
        res.push('\n');
    }
    (res, modified)
}

pub fn set_nuget_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_nuget_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_nuget_proxy_xml(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_nuget_proxy() -> Result<bool> {
    let mut any_removed = false;
    let mut candidates = Vec::new();
    #[cfg(windows)]
    if let Some(appdata) = dirs::config_dir() {
        candidates.push(appdata.join("NuGet").join("NuGet.Config"));
    }
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join(".nuget").join("NuGet").join("NuGet.Config"));
        candidates.push(home.join(".config").join("NuGet").join("NuGet.Config"));
    }
    for path in candidates {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_nuget_proxy_xml(&content);
                if removed {
                    std::fs::write(&path, new_content)?;
                    any_removed = true;
                }
            }
        }
    }
    Ok(any_removed)
}

pub fn apply_curlrc_lines(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut replaced = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#')
            && (trim.starts_with("proxy") || trim.starts_with("-x") || trim.starts_with("--proxy"))
        {
            if !replaced {
                lines.push(format!("proxy = \"{}\"", proxy_url));
                replaced = true;
            }
        } else {
            lines.push(line.to_string());
        }
    }
    if !replaced {
        lines.push(format!("proxy = \"{}\"", proxy_url));
    }
    lines.join("\n") + "\n"
}

pub fn remove_curlrc_lines(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;
    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#')
            && (trim.starts_with("proxy") || trim.starts_with("-x") || trim.starts_with("--proxy"))
        {
            changed = true;
        } else {
            lines.push(line.to_string());
        }
    }
    let res = lines.join("\n") + if lines.is_empty() { "" } else { "\n" };
    (res, changed)
}

pub fn apply_curlrc_proxy(proxy_url: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let path = home.join(".curlrc");
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_curlrc_lines(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_curlrc_proxy() -> Result<bool> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let path = home.join(".curlrc");
    if !path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&path)?;
    let (new_content, changed) = remove_curlrc_lines(&content);
    if changed {
        std::fs::write(&path, new_content)?;
    }
    Ok(changed)
}

pub fn apply_wgetrc_lines(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut set_http = false;
    let mut set_https = false;
    let mut set_use = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();
        if !trim.starts_with('#') && lower.starts_with("http_proxy") {
            if !set_http {
                lines.push(format!("http_proxy = {}", proxy_url));
                set_http = true;
            }
        } else if !trim.starts_with('#') && lower.starts_with("https_proxy") {
            if !set_https {
                lines.push(format!("https_proxy = {}", proxy_url));
                set_https = true;
            }
        } else if !trim.starts_with('#') && lower.starts_with("use_proxy") {
            if !set_use {
                lines.push("use_proxy = on".to_string());
                set_use = true;
            }
        } else {
            lines.push(line.to_string());
        }
    }
    if !set_http {
        lines.push(format!("http_proxy = {}", proxy_url));
    }
    if !set_https {
        lines.push(format!("https_proxy = {}", proxy_url));
    }
    if !set_use {
        lines.push("use_proxy = on".to_string());
    }
    lines.join("\n") + "\n"
}

pub fn remove_wgetrc_lines(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;
    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();
        if !trim.starts_with('#')
            && (lower.starts_with("http_proxy")
                || lower.starts_with("https_proxy")
                || lower.starts_with("use_proxy"))
        {
            changed = true;
        } else {
            lines.push(line.to_string());
        }
    }
    let res = lines.join("\n") + if lines.is_empty() { "" } else { "\n" };
    (res, changed)
}

pub fn apply_wgetrc_proxy(proxy_url: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let path = home.join(".wgetrc");
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_wgetrc_lines(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_wgetrc_proxy() -> Result<bool> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let path = home.join(".wgetrc");
    if !path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&path)?;
    let (new_content, changed) = remove_wgetrc_lines(&content);
    if changed {
        std::fs::write(&path, new_content)?;
    }
    Ok(changed)
}

pub fn apply_gemrc_lines(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut replaced = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#')
            && (trim.starts_with("http_proxy:") || trim.starts_with(":http_proxy:"))
        {
            if !replaced {
                lines.push(format!("http_proxy: {}", proxy_url));
                replaced = true;
            }
        } else {
            lines.push(line.to_string());
        }
    }
    if !replaced {
        lines.push(format!("http_proxy: {}", proxy_url));
    }
    lines.join("\n") + "\n"
}

pub fn remove_gemrc_lines(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#')
            && (trim.starts_with("http_proxy:") || trim.starts_with(":http_proxy:"))
        {
            changed = true;
        } else {
            lines.push(line.to_string());
        }
    }
    let res = lines.join("\n") + if lines.is_empty() { "" } else { "\n" };
    (res, changed)
}

pub fn set_gem_proxy(proxy_url: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let path = home.join(".gemrc");
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_gemrc_lines(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_gem_proxy() -> Result<bool> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let path = home.join(".gemrc");
    if !path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&path)?;
    let (new_content, changed) = remove_gemrc_lines(&content);
    if changed {
        std::fs::write(&path, new_content)?;
    }
    Ok(changed)
}

pub fn apply_svn_servers_content(content: &str, host: &str, port: u16) -> String {
    let mut lines = Vec::new();
    let mut in_global = false;
    let mut global_found = false;
    let mut host_set = false;
    let mut port_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = &trim[1..trim.len() - 1].trim();
            if in_global {
                if !host_set {
                    lines.push(format!("http-proxy-host = {}", host));
                    host_set = true;
                }
                if !port_set {
                    lines.push(format!("http-proxy-port = {}", port));
                    port_set = true;
                }
            }
            in_global = section.eq_ignore_ascii_case("global");
            if in_global {
                global_found = true;
            }
            lines.push(line.to_string());
            continue;
        }

        if in_global {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "http-proxy-host" {
                    lines.push(format!("http-proxy-host = {}", host));
                    host_set = true;
                    continue;
                } else if key == "http-proxy-port" {
                    lines.push(format!("http-proxy-port = {}", port));
                    port_set = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    if in_global {
        if !host_set {
            lines.push(format!("http-proxy-host = {}", host));
        }
        if !port_set {
            lines.push(format!("http-proxy-port = {}", port));
        }
    } else if !global_found {
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push(String::new());
        }
        lines.push("[global]".to_string());
        lines.push(format!("http-proxy-host = {}", host));
        lines.push(format!("http-proxy-port = {}", port));
    }

    lines.join("\n") + "\n"
}

pub fn remove_svn_servers_content(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut in_global = false;
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = &trim[1..trim.len() - 1].trim();
            in_global = section.eq_ignore_ascii_case("global");
            lines.push(line.to_string());
            continue;
        }
        if in_global && !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "http-proxy-host" || key == "http-proxy-port" {
                    changed = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }
    let res = lines.join("\n") + if lines.is_empty() { "" } else { "\n" };
    (res, changed)
}

pub fn set_svn_proxy(proxy_url: &str) -> Result<PathBuf> {
    let (host, port) = extract_host_port(proxy_url);
    let path = crate::doctor::get_svn_servers_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_svn_servers_content(&content, &host, port);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_svn_proxy() -> Result<bool> {
    let path = crate::doctor::get_svn_servers_path();
    if !path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&path)?;
    let (new_content, changed) = remove_svn_servers_content(&content);
    if changed {
        std::fs::write(&path, new_content)?;
    }
    Ok(changed)
}

pub fn apply_hgrc_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut in_http_proxy = false;
    let mut section_found = false;
    let mut host_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = &trim[1..trim.len() - 1].trim();
            if in_http_proxy && !host_set {
                lines.push(format!("host = {}", proxy_url));
                host_set = true;
            }
            in_http_proxy = section.eq_ignore_ascii_case("http_proxy");
            if in_http_proxy {
                section_found = true;
            }
            lines.push(line.to_string());
            continue;
        }

        if in_http_proxy {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "host" {
                    lines.push(format!("host = {}", proxy_url));
                    host_set = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    if in_http_proxy && !host_set {
        lines.push(format!("host = {}", proxy_url));
    } else if !section_found {
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push(String::new());
        }
        lines.push("[http_proxy]".to_string());
        lines.push(format!("host = {}", proxy_url));
        lines.push("no = localhost, 127.0.0.1".to_string());
    }

    lines.join("\n") + "\n"
}

pub fn remove_hgrc_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut in_http_proxy = false;
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = &trim[1..trim.len() - 1].trim();
            in_http_proxy = section.eq_ignore_ascii_case("http_proxy");
            lines.push(line.to_string());
            continue;
        }
        if in_http_proxy && !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "host" || key == "no" {
                    changed = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }
    let res = lines.join("\n") + if lines.is_empty() { "" } else { "\n" };
    (res, changed)
}

pub fn set_hg_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_hgrc_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_hgrc_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_hg_proxy() -> Result<bool> {
    let path = crate::doctor::get_hgrc_path();
    if !path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&path)?;
    let (new_content, changed) = remove_hgrc_proxy(&content);
    if changed {
        std::fs::write(&path, new_content)?;
    }
    Ok(changed)
}

pub fn apply_uv_proxy_toml(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut updated = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "http-proxy" || key == "proxy" {
                    lines.push(format!("http-proxy = \"{}\"", proxy_url));
                    updated = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    if !updated {
        let mut inserted = false;
        let mut final_lines = Vec::new();
        for line in &lines {
            if !inserted && line.trim().starts_with('[') {
                final_lines.push(format!("http-proxy = \"{}\"", proxy_url));
                final_lines.push(String::new());
                inserted = true;
            }
            final_lines.push(line.clone());
        }
        if !inserted {
            final_lines.push(format!("http-proxy = \"{}\"", proxy_url));
        }
        return final_lines.join("\n") + "\n";
    }

    lines.join("\n") + "\n"
}

pub fn remove_uv_proxy_toml(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "http-proxy" || key == "proxy" {
                    changed = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    (
        lines.join("\n") + if lines.is_empty() { "" } else { "\n" },
        changed,
    )
}

pub fn set_uv_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_uv_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_uv_proxy_toml(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_uv_proxy() -> Result<bool> {
    let path = crate::doctor::get_uv_config_path();
    if !path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&path)?;
    let (new_content, changed) = remove_uv_proxy_toml(&content);
    if changed {
        std::fs::write(&path, new_content)?;
    }
    Ok(changed)
}

pub fn apply_go_env_content(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut updated = false;
    let proxy_val = format!("{},direct", proxy_url);

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') {
            if let Some((k, _)) = trim.split_once('=') {
                if k.trim() == "GOPROXY" {
                    lines.push(format!("GOPROXY={}", proxy_val));
                    updated = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    if !updated {
        lines.push(format!("GOPROXY={}", proxy_val));
    }

    lines.join("\n") + "\n"
}

pub fn remove_go_env_content(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') {
            if let Some((k, _)) = trim.split_once('=') {
                if k.trim() == "GOPROXY" {
                    changed = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    (
        lines.join("\n") + if lines.is_empty() { "" } else { "\n" },
        changed,
    )
}

pub fn set_go_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_go_env_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_go_env_content(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    let proxy_val = format!("{},direct", proxy_url);
    let _ = Command::new("go")
        .args(["env", "-w", &format!("GOPROXY={}", proxy_val)])
        .status();

    Ok(path)
}

pub fn unset_go_proxy() -> Result<bool> {
    let path = crate::doctor::get_go_env_path();
    let mut changed = false;
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let (new_content, c) = remove_go_env_content(&content);
        if c {
            std::fs::write(&path, new_content)?;
            changed = true;
        }
    }
    let _ = Command::new("go").args(["env", "-u", "GOPROXY"]).status();
    Ok(changed)
}

pub fn apply_bazelrc_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut found_http = false;
    let mut found_https = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && trim.contains("--action_env=") {
            let lower = trim.to_ascii_lowercase();
            if lower.contains("--action_env=http_proxy=") {
                lines.push(format!("build --action_env=HTTP_PROXY={}", proxy_url));
                found_http = true;
                continue;
            } else if lower.contains("--action_env=https_proxy=") {
                lines.push(format!("build --action_env=HTTPS_PROXY={}", proxy_url));
                found_https = true;
                continue;
            } else if lower.contains("--action_env=all_proxy=") {
                lines.push(format!("build --action_env=ALL_PROXY={}", proxy_url));
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !found_http {
        lines.push(format!("build --action_env=HTTP_PROXY={}", proxy_url));
    }
    if !found_https {
        lines.push(format!("build --action_env=HTTPS_PROXY={}", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_bazelrc_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && trim.contains("--action_env=") {
            let lower = trim.to_ascii_lowercase();
            if lower.contains("--action_env=http_proxy=")
                || lower.contains("--action_env=https_proxy=")
                || lower.contains("--action_env=all_proxy=")
            {
                changed = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    (
        lines.join("\n") + if lines.is_empty() { "" } else { "\n" },
        changed,
    )
}

pub fn set_bazel_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_bazelrc_write_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_bazelrc_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_bazel_proxy() -> Result<bool> {
    let path = crate::doctor::get_bazelrc_write_path();
    if !path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&path)?;
    let (new_content, changed) = remove_bazelrc_proxy(&content);
    if changed {
        std::fs::write(&path, new_content)?;
    }
    Ok(changed)
}

pub fn apply_composer_proxy_json(content: &str, proxy_url: &str) -> String {
    let mut val: serde_json::Value = if content.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(content).unwrap_or_else(|_| serde_json::json!({}))
    };

    if !val.is_object() {
        val = serde_json::json!({});
    }

    let obj = val.as_object_mut().unwrap();
    let config = obj
        .entry("config")
        .or_insert_with(|| serde_json::json!({}));

    if !config.is_object() {
        *config = serde_json::json!({});
    }

    let config_obj = config.as_object_mut().unwrap();
    config_obj.insert(
        "http-proxy".to_string(),
        serde_json::Value::String(proxy_url.to_string()),
    );
    config_obj.insert(
        "https-proxy".to_string(),
        serde_json::Value::String(proxy_url.to_string()),
    );

    serde_json::to_string_pretty(&val).unwrap_or_else(|_| content.to_string()) + "\n"
}

pub fn remove_composer_proxy_json(content: &str) -> (String, bool) {
    if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(obj) = val.as_object_mut() {
            if let Some(cfg) = obj.get_mut("config").and_then(|c| c.as_object_mut()) {
                let had_http = cfg.remove("http-proxy").is_some();
                let had_https = cfg.remove("https-proxy").is_some();
                if had_http || had_https {
                    let new_content = serde_json::to_string_pretty(&val)
                        .unwrap_or_else(|_| content.to_string())
                        + "\n";
                    return (new_content, true);
                }
            }
        }
    }
    (content.to_string(), false)
}

pub fn set_composer_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_composer_config_write_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_composer_proxy_json(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_composer_proxy() -> Result<bool> {
    let path = crate::doctor::get_composer_config_write_path();
    if !path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&path)?;
    let (new_content, changed) = remove_composer_proxy_json(&content);
    if changed {
        std::fs::write(&path, new_content)?;
    }
    Ok(changed)
}

pub fn apply_conan2_global_conf(content: &str, proxy_url: &str) -> String {
    let new_line = format!(
        "core.net.http:proxies = {{\"http\": \"{}\", \"https\": \"{}\"}}",
        proxy_url, proxy_url
    );
    let mut replaced = false;
    let mut out_lines = Vec::new();

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with("core.net.http:proxies") {
            out_lines.push(new_line.clone());
            replaced = true;
        } else {
            out_lines.push(line.to_string());
        }
    }

    if !replaced {
        if !out_lines.is_empty() && !out_lines.last().unwrap().trim().is_empty() {
            out_lines.push(String::new());
        }
        out_lines.push(new_line);
    }

    out_lines.join("\n") + "\n"
}

pub fn remove_conan2_global_conf(content: &str) -> (String, bool) {
    let mut modified = false;
    let mut out_lines = Vec::new();

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with("core.net.http:proxies") {
            modified = true;
        } else {
            out_lines.push(line.to_string());
        }
    }

    (out_lines.join("\n") + "\n", modified)
}

pub fn apply_conan1_conf(content: &str, proxy_url: &str) -> String {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut section_start = None;
    let mut section_end = None;

    for (i, line) in lines.iter().enumerate() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let sec = trim[1..trim.len() - 1].trim();
            if sec.eq_ignore_ascii_case("proxies") {
                section_start = Some(i);
            } else if section_start.is_some() && section_end.is_none() {
                section_end = Some(i);
            }
        }
    }

    if let Some(start) = section_start {
        let mut end = section_end.unwrap_or(lines.len());
        let mut idx = start + 1;
        while idx < end && idx < lines.len() {
            let trim = lines[idx].trim();
            if trim.starts_with("http ")
                || trim.starts_with("http=")
                || trim.starts_with("https ")
                || trim.starts_with("https=")
            {
                lines.remove(idx);
                end -= 1;
            } else {
                idx += 1;
            }
        }
        lines.insert(start + 1, format!("https = {}", proxy_url));
        lines.insert(start + 1, format!("http = {}", proxy_url));
    } else {
        if !lines.is_empty() && !lines.last().unwrap().trim().is_empty() {
            lines.push(String::new());
        }
        lines.push("[proxies]".to_string());
        lines.push(format!("http = {}", proxy_url));
        lines.push(format!("https = {}", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_conan1_conf(content: &str) -> (String, bool) {
    let mut modified = false;
    let mut in_proxies = false;
    let mut out_lines = Vec::new();

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let sec = trim[1..trim.len() - 1].trim();
            in_proxies = sec.eq_ignore_ascii_case("proxies");
            out_lines.push(line.to_string());
            continue;
        }
        if in_proxies {
            if trim.starts_with("http ")
                || trim.starts_with("http=")
                || trim.starts_with("https ")
                || trim.starts_with("https=")
            {
                modified = true;
                continue;
            }
        }
        out_lines.push(line.to_string());
    }

    (out_lines.join("\n") + "\n", modified)
}

pub fn set_conan_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_conan_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path)?
    } else {
        String::new()
    };
    let new_content = if write_path.ends_with("conan.conf") {
        apply_conan1_conf(&content, proxy_url)
    } else {
        apply_conan2_global_conf(&content, proxy_url)
    };
    std::fs::write(&write_path, new_content)?;

    if let Some(c1_path) = crate::doctor::get_conan1_config_path() {
        if c1_path != write_path {
            if let Ok(c1_content) = std::fs::read_to_string(&c1_path) {
                let updated = apply_conan1_conf(&c1_content, proxy_url);
                let _ = std::fs::write(&c1_path, updated);
            }
        }
    }

    Ok(write_path)
}

pub fn unset_conan_proxy() -> Result<bool> {
    let mut changed_any = false;
    if let Some(p2) = crate::doctor::get_conan2_config_path() {
        if let Ok(content) = std::fs::read_to_string(&p2) {
            let (new_content, changed) = remove_conan2_global_conf(&content);
            if changed {
                std::fs::write(&p2, new_content)?;
                changed_any = true;
            }
        }
    }
    if let Some(p1) = crate::doctor::get_conan1_config_path() {
        if let Ok(content) = std::fs::read_to_string(&p1) {
            let (new_content, changed) = remove_conan1_conf(&content);
            if changed {
                std::fs::write(&p1, new_content)?;
                changed_any = true;
            }
        }
    }
    Ok(changed_any)
}

pub fn remove_sbtopts_proxy(content: &str) -> (String, bool) {
    let mut modified = false;
    let mut out_lines = Vec::new();

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            out_lines.push(line.to_string());
            continue;
        }

        let mut remaining_tokens = Vec::new();
        let mut line_modified = false;

        for token in line.split_whitespace() {
            if token.starts_with("-Dhttp.proxyHost")
                || token.starts_with("-Dhttp.proxyPort")
                || token.starts_with("-Dhttps.proxyHost")
                || token.starts_with("-Dhttps.proxyPort")
                || token.starts_with("-DsocksProxyHost")
                || token.starts_with("-DsocksProxyPort")
                || token.starts_with("-Dhttp.nonProxyHosts")
                || token.starts_with("-Dhttps.nonProxyHosts")
            {
                line_modified = true;
                modified = true;
            } else {
                remaining_tokens.push(token);
            }
        }

        if !remaining_tokens.is_empty() {
            out_lines.push(remaining_tokens.join(" "));
        } else if !line_modified && !trim.is_empty() {
            out_lines.push(line.to_string());
        }
    }

    let res = out_lines.join("\n") + if out_lines.is_empty() { "" } else { "\n" };
    (res, modified)
}

pub fn apply_sbtopts_proxy(content: &str, proxy_url: &str) -> String {
    let (cleaned, _) = remove_sbtopts_proxy(content);
    let mut lines: Vec<String> = cleaned.lines().map(|s| s.to_string()).collect();

    let (host, port) = extract_host_port(proxy_url);
    let is_socks = proxy_url.to_ascii_lowercase().starts_with("socks");

    if is_socks {
        lines.push(format!("-DsocksProxyHost={}", host));
        lines.push(format!("-DsocksProxyPort={}", port));
    } else {
        lines.push(format!("-Dhttp.proxyHost={}", host));
        lines.push(format!("-Dhttp.proxyPort={}", port));
        lines.push(format!("-Dhttps.proxyHost={}", host));
        lines.push(format!("-Dhttps.proxyPort={}", port));
        lines.push("-Dhttp.nonProxyHosts=\"localhost|127.0.0.1\"".to_string());
    }

    lines.join("\n") + "\n"
}

pub fn set_sbt_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_sbtopts_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path)?
    } else {
        String::new()
    };
    let new_content = apply_sbtopts_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_sbt_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_sbtopts_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_sbtopts_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn remove_kubeconfig_proxy(content: &str) -> (String, bool) {
    let mut modified = false;
    let mut out_lines = Vec::new();

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with("proxy-url:") {
            modified = true;
            continue;
        }
        out_lines.push(line.to_string());
    }

    let res = out_lines.join("\n") + if out_lines.is_empty() { "" } else { "\n" };
    (res, modified)
}

pub fn apply_kubeconfig_proxy(content: &str, proxy_url: &str) -> String {
    let (cleaned, _) = remove_kubeconfig_proxy(content);
    if cleaned.trim().is_empty() {
        return format!(
            "apiVersion: v1\nclusters:\n- cluster:\n    proxy-url: {}\n  name: default\n",
            proxy_url
        );
    }

    let mut out_lines = Vec::new();
    let mut in_clusters = false;
    let mut proxy_inserted = false;

    for line in cleaned.lines() {
        let trim = line.trim();

        if !line.starts_with(' ') && !line.starts_with('\t') && !line.starts_with('-') {
            if trim.starts_with("clusters:") {
                in_clusters = true;
            } else if in_clusters {
                in_clusters = false;
            }
        }

        out_lines.push(line.to_string());

        if in_clusters {
            if trim == "- cluster:" || trim == "cluster:" {
                out_lines.push(format!("    proxy-url: {}", proxy_url));
                proxy_inserted = true;
            }
        }
    }

    if !proxy_inserted {
        if !out_lines.is_empty() && !out_lines.last().unwrap().trim().is_empty() {
            out_lines.push(String::new());
        }
        out_lines.push("clusters:".to_string());
        out_lines.push("- cluster:".to_string());
        out_lines.push(format!("    proxy-url: {}", proxy_url));
        out_lines.push("  name: default".to_string());
    }

    out_lines.join("\n") + "\n"
}

pub fn set_kube_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_kubeconfig_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path)?
    } else {
        String::new()
    };
    let new_content = apply_kubeconfig_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_kube_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_kubeconfig_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_kubeconfig_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn remove_julia_startup_proxy(content: &str) -> (String, bool) {
    let mut modified = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        let trim = line.trim();
        let upper = trim.to_ascii_uppercase();
        if (upper.starts_with("ENV[") || trim.contains("[proxydoc"))
            && (upper.contains("HTTP_PROXY") || upper.contains("HTTPS_PROXY") || upper.contains("ALL_PROXY"))
        {
            modified = true;
            continue;
        }
        lines.push(line.to_string());
    }
    let res = lines.join("\n") + if lines.is_empty() { "" } else { "\n" };
    (res, modified)
}

pub fn apply_julia_startup_proxy(content: &str, proxy_url: &str) -> String {
    let (cleaned, _) = remove_julia_startup_proxy(content);
    let mut lines: Vec<String> = cleaned.lines().map(|s| s.to_string()).collect();

    while let Some(last) = lines.last() {
        if last.trim().is_empty() {
            lines.pop();
        } else {
            break;
        }
    }

    lines.push("# [proxydoc managed proxy]".to_string());
    lines.push(format!("ENV[\"HTTP_PROXY\"] = \"{}\"", proxy_url));
    lines.push(format!("ENV[\"HTTPS_PROXY\"] = \"{}\"", proxy_url));
    lines.push(format!("ENV[\"all_proxy\"] = \"{}\"", proxy_url));

    lines.join("\n") + "\n"
}

pub fn set_julia_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_julia_startup_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path)?
    } else {
        String::new()
    };
    let new_content = apply_julia_startup_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_julia_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_julia_startup_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_julia_startup_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_nix_config_proxy(content: &str, proxy_url: &str) -> String {
    let mut replaced = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') {
            if let Some((k, _)) = trim.split_once('=') {
                if k.trim() == "http-proxy" {
                    lines.push(format!("http-proxy = {}", proxy_url));
                    replaced = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }
    if !replaced {
        lines.push(format!("http-proxy = {}", proxy_url));
    }
    lines.join("\n") + "\n"
}

pub fn remove_nix_config_proxy(content: &str) -> (String, bool) {
    let mut modified = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') {
            if let Some((k, _)) = trim.split_once('=') {
                if k.trim() == "http-proxy" {
                    modified = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }
    (lines.join("\n") + "\n", modified)
}

pub fn set_nix_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_nix_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path)?
    } else {
        String::new()
    };
    let new_content = apply_nix_config_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_nix_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_nix_config_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_nix_config_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_cabal_config_proxy(content: &str, proxy_url: &str) -> String {
    let mut replaced = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with("--") && !trim.starts_with('#') && trim.starts_with("proxy:") {
            lines.push(format!("proxy: {}", proxy_url));
            replaced = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !replaced {
        lines.push(format!("proxy: {}", proxy_url));
    }
    lines.join("\n") + "\n"
}

pub fn remove_cabal_config_proxy(content: &str) -> (String, bool) {
    let mut modified = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with("--") && !trim.starts_with('#') && trim.starts_with("proxy:") {
            modified = true;
            continue;
        }
        lines.push(line.to_string());
    }
    (lines.join("\n") + "\n", modified)
}

pub fn set_cabal_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_cabal_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path)?
    } else {
        String::new()
    };
    let new_content = apply_cabal_config_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_cabal_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_cabal_config_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_cabal_config_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_stack_config_proxy(content: &str, proxy_url: &str) -> String {
    let mut in_proxy = false;
    let mut lines = Vec::new();
    let mut replaced = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && trim.starts_with("proxy:") {
            in_proxy = true;
            if !replaced {
                lines.push("proxy:".to_string());
                lines.push(format!("  http: \"{}\"", proxy_url));
                lines.push(format!("  https: \"{}\"", proxy_url));
                replaced = true;
            }
            continue;
        }
        if in_proxy {
            if line.starts_with(' ') || line.starts_with('\t') {
                continue;
            } else {
                in_proxy = false;
                lines.push(line.to_string());
            }
        } else {
            lines.push(line.to_string());
        }
    }

    if !replaced {
        if !lines.is_empty() && !lines.last().map(|s| s.is_empty()).unwrap_or(true) {
            lines.push(String::new());
        }
        lines.push("proxy:".to_string());
        lines.push(format!("  http: \"{}\"", proxy_url));
        lines.push(format!("  https: \"{}\"", proxy_url));
    }
    lines.join("\n") + "\n"
}

pub fn remove_stack_config_proxy(content: &str) -> (String, bool) {
    let mut in_proxy = false;
    let mut lines = Vec::new();
    let mut modified = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && trim.starts_with("proxy:") {
            in_proxy = true;
            modified = true;
            continue;
        }
        if in_proxy {
            if line.starts_with(' ') || line.starts_with('\t') {
                modified = true;
                continue;
            } else {
                in_proxy = false;
                lines.push(line.to_string());
            }
        } else {
            lines.push(line.to_string());
        }
    }
    (lines.join("\n") + "\n", modified)
}

pub fn set_stack_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_stack_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path)?
    } else {
        String::new()
    };
    let new_content = apply_stack_config_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_stack_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_stack_config_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_stack_config_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_hex_config_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut replaced = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('%') && !trim.starts_with('#') && (trim.contains("http_proxy,") || trim.contains("https_proxy,")) {
            if !replaced {
                lines.push(format!("{{http_proxy, \"{}\"}}.", proxy_url));
                lines.push(format!("{{https_proxy, \"{}\"}}.", proxy_url));
                replaced = true;
            }
        } else {
            lines.push(line.to_string());
        }
    }

    if !replaced {
        if !lines.is_empty() && !lines.last().map(|s| s.is_empty()).unwrap_or(true) {
            lines.push(String::new());
        }
        lines.push(format!("{{http_proxy, \"{}\"}}.", proxy_url));
        lines.push(format!("{{https_proxy, \"{}\"}}.", proxy_url));
    }
    lines.join("\n") + "\n"
}

pub fn remove_hex_config_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut modified = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('%') && !trim.starts_with('#') && (trim.contains("http_proxy,") || trim.contains("https_proxy,")) {
            modified = true;
            continue;
        }
        lines.push(line.to_string());
    }
    (lines.join("\n") + "\n", modified)
}

pub fn set_hex_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_hex_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path)?
    } else {
        String::new()
    };
    let new_content = apply_hex_config_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_hex_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_hex_config_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_hex_config_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_renviron_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_lowercase();
                if key == "http_proxy" || key == "https_proxy" || key == "all_proxy" || key == "no_proxy" {
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    if !lines.is_empty() && !lines.last().map(|s| s.is_empty()).unwrap_or(true) {
        lines.push(String::new());
    }
    lines.push(format!("http_proxy=\"{}\"", proxy_url));
    lines.push(format!("https_proxy=\"{}\"", proxy_url));
    lines.push(format!("all_proxy=\"{}\"", proxy_url));
    lines.push("no_proxy=\"localhost,127.0.0.1\"".to_string());
    lines.join("\n") + "\n"
}

pub fn remove_renviron_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut modified = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_lowercase();
                if key == "http_proxy" || key == "https_proxy" || key == "all_proxy" || key == "no_proxy" {
                    modified = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }
    (lines.join("\n") + "\n", modified)
}

pub fn set_renviron_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_renviron_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path)?
    } else {
        String::new()
    };
    let new_content = apply_renviron_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_renviron_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_renviron_config_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_renviron_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_opam_config_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut replaced = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') {
            if let Some((k, _)) = trim.split_once(':') {
                let key = k.trim().to_lowercase();
                if key == "http-proxy" || key == "https-proxy" || key == "http_proxy" || key == "https_proxy" {
                    if !replaced {
                        lines.push(format!("http-proxy: \"{}\"", proxy_url));
                        lines.push(format!("https-proxy: \"{}\"", proxy_url));
                        replaced = true;
                    }
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    if !replaced {
        if !lines.is_empty() && !lines.last().map(|s| s.is_empty()).unwrap_or(true) {
            lines.push(String::new());
        }
        lines.push(format!("http-proxy: \"{}\"", proxy_url));
        lines.push(format!("https-proxy: \"{}\"", proxy_url));
    }
    lines.join("\n") + "\n"
}

pub fn remove_opam_config_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut modified = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') {
            if let Some((k, _)) = trim.split_once(':') {
                let key = k.trim().to_lowercase();
                if key == "http-proxy" || key == "https-proxy" || key == "http_proxy" || key == "https_proxy" {
                    modified = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }
    (lines.join("\n") + "\n", modified)
}

pub fn set_opam_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_opam_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path)?
    } else {
        String::new()
    };
    let new_content = apply_opam_config_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_opam_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_opam_config_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_opam_config_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_lein_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with(';') {
            if trim.contains(":http-proxy") {
                lines.push(format!("        :http-proxy \"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if trim.contains(":https-proxy") {
                lines.push(format!("        :https-proxy \"{}\"", proxy_url));
                replaced_https = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if replaced_http || replaced_https {
        if !replaced_http {
            let idx = lines.len().saturating_sub(1);
            lines.insert(idx, format!("        :http-proxy \"{}\"", proxy_url));
        }
        if !replaced_https {
            let idx = lines.len().saturating_sub(1);
            lines.insert(idx, format!("        :https-proxy \"{}\"", proxy_url));
        }
        let mut res = lines.join("\n");
        if content.ends_with('\n') {
            res.push('\n');
        }
        return res;
    }

    if content.trim().is_empty() {
        return format!(
            "{{:user {{:http-proxy \"{}\"\n        :https-proxy \"{}\"}}}}\n",
            proxy_url, proxy_url
        );
    }

    let mut out_lines = Vec::new();
    let mut inserted = false;
    for line in content.lines() {
        out_lines.push(line.to_string());
        if !inserted && (line.contains("{:user {") || line.contains(":user {")) {
            out_lines.push(format!("        :http-proxy \"{}\"", proxy_url));
            out_lines.push(format!("        :https-proxy \"{}\"", proxy_url));
            inserted = true;
        }
    }

    if !inserted {
        if let Some(pos) = content.rfind('}') {
            let before = &content[..pos];
            let after = &content[pos..];
            return format!(
                "{}\n        :http-proxy \"{}\"\n        :https-proxy \"{}\"{}",
                before.trim_end(),
                proxy_url,
                proxy_url,
                after
            );
        } else {
            return format!(
                "{{:user {{:http-proxy \"{}\"\n        :https-proxy \"{}\"}}}}\n",
                proxy_url, proxy_url
            );
        }
    }

    let mut res = out_lines.join("\n");
    if content.ends_with('\n') {
        res.push('\n');
    }
    res
}

pub fn remove_lein_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut modified = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with(';') && (trim.contains(":http-proxy") || trim.contains(":https-proxy")) {
            modified = true;
            continue;
        }
        lines.push(line.to_string());
    }

    (
        lines.join("\n")
            + if lines.is_empty() {
                ""
            } else if content.ends_with('\n') {
                "\n"
            } else {
                ""
            },
        modified,
    )
}

pub fn set_lein_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_lein_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path)?
    } else {
        String::new()
    };
    let new_content = apply_lein_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_lein_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_lein_profiles_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_lein_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_cpan_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut set_http = false;
    let mut set_ftp = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') {
            if trim.contains("'http_proxy'") || trim.contains("\"http_proxy\"") {
                lines.push(format!("  'http_proxy' => q[{}],", proxy_url));
                set_http = true;
                continue;
            } else if trim.contains("'ftp_proxy'") || trim.contains("\"ftp_proxy\"") {
                lines.push(format!("  'ftp_proxy' => q[{}],", proxy_url));
                set_ftp = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if set_http || set_ftp {
        if !set_http {
            let idx = lines.len().saturating_sub(1);
            lines.insert(idx, format!("  'http_proxy' => q[{}],", proxy_url));
        }
        if !set_ftp {
            let idx = lines.len().saturating_sub(1);
            lines.insert(idx, format!("  'ftp_proxy' => q[{}],", proxy_url));
        }
        let mut res = lines.join("\n");
        if content.ends_with('\n') {
            res.push('\n');
        }
        return res;
    }

    if content.trim().is_empty() {
        return format!(
            "$CPAN::Config = {{\n  'http_proxy' => q[{}],\n  'ftp_proxy' => q[{}],\n}};\n1;\n__END__\n",
            proxy_url, proxy_url
        );
    }

    let mut out_lines = Vec::new();
    let mut inserted = false;
    for line in content.lines() {
        let trim = line.trim();
        if !inserted && (trim == "};" || trim.starts_with("};")) {
            out_lines.push(format!("  'http_proxy' => q[{}],", proxy_url));
            out_lines.push(format!("  'ftp_proxy' => q[{}],", proxy_url));
            inserted = true;
        }
        out_lines.push(line.to_string());
    }
    if !inserted {
        out_lines.push(format!("  'http_proxy' => q[{}],", proxy_url));
        out_lines.push(format!("  'ftp_proxy' => q[{}],", proxy_url));
    }

    let mut res = out_lines.join("\n");
    if content.ends_with('\n') {
        res.push('\n');
    }
    res
}

pub fn remove_cpan_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut modified = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') {
            if trim.contains("'http_proxy'")
                || trim.contains("\"http_proxy\"")
                || trim.contains("'ftp_proxy'")
                || trim.contains("\"ftp_proxy\"")
            {
                modified = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    (
        lines.join("\n")
            + if lines.is_empty() {
                ""
            } else if content.ends_with('\n') {
                "\n"
            } else {
                ""
            },
        modified,
    )
}

pub fn set_cpan_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_cpan_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path)?
    } else {
        String::new()
    };
    let new_content = apply_cpan_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_cpan_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_cpan_config_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_cpan_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub async fn turn_on(
    target: &str,
    git: bool,
    cargo: bool,
    npm: bool,
    yarn: bool,
    pnpm: bool,
    bun: bool,
    pip: bool,
    conda: bool,
    docker: bool,
    gradle: bool,
    maven: bool,
    vscode: bool,
    gh: bool,
    glab: bool,
    jetbrains: bool,
    nuget: bool,
    curl: bool,
    wget: bool,
    gem: bool,
    svn: bool,
    hg: bool,
    uv: bool,
    go: bool,
    bazel: bool,
    composer: bool,
    conan: bool,
    sbt: bool,
    kube: bool,
    julia: bool,
    nix: bool,
    cabal: bool,
    stack: bool,
    hex: bool,
    r: bool,
    opam: bool,
    lein: bool,
    cpan: bool,
    all: bool,
    shell: Option<&str>,
) -> Result<()> {
    let set_git = git || all;
    let set_cargo = cargo || all;
    let set_npm = npm || all;
    let set_yarn = yarn || all;
    let set_pnpm = pnpm || all;
    let set_bun = bun || all;
    let set_pip = pip || all;
    let set_conda = conda || all;
    let set_docker = docker || all;
    let set_gradle = gradle || all;
    let set_maven = maven || all;
    let set_vscode = vscode || all;
    let set_gh = gh || all;
    let set_glab = glab || all;
    let set_jetbrains = jetbrains || all;
    let set_nuget = nuget || all;
    let set_gem = gem || all;
    let set_svn = svn || all;
    let set_hg = hg || all;
    let set_uv = uv || all;
    let set_go = go || all;
    let set_bazel = bazel || all;
    let set_composer = composer || all;
    let set_conan = conan || all;
    let set_sbt = sbt || all;
    let set_kube = kube || all;
    let set_julia = julia || all;
    let set_nix = nix || all;
    let set_cabal = cabal || all;
    let set_stack = stack || all;
    let set_hex = hex || all;
    let set_r = r || all;
    let set_opam = opam || all;
    let set_lein = lein || all;
    let set_cpan = cpan || all;
    let set_curl = curl;
    let set_wget = wget;

    // Resolve target proxy address
    let proxy_url = if target == "auto" {
        // 1. Try system proxy from OS
        if let Some(sys_info) = get_system_proxy() {
            if sys_info.state.http_enabled && sys_info.state.http_port > 0 {
                format!(
                    "http://{}:{}",
                    sys_info.state.http_host, sys_info.state.http_port
                )
            } else if let Some(p) = scanner::find_first_active_proxy().await {
                format!("http://127.0.0.1:{}", p.port)
            } else {
                "http://127.0.0.1:7890".to_string()
            }
        } else if let Some(p) = scanner::find_first_active_proxy().await {
            format!("http://127.0.0.1:{}", p.port)
        } else {
            "http://127.0.0.1:7890".to_string()
        }
    } else if target.chars().all(|c| c.is_ascii_digit()) {
        format!("http://127.0.0.1:{}", target)
    } else if !target.contains("://") {
        format!("http://{}", target)
    } else {
        target.to_string()
    };

    let target_shell = detect_shell(shell);
    for line in generate_shell_export(&proxy_url, target_shell) {
        println!("{}", line);
    }

    if set_git {
        let _ = Command::new("git")
            .args(["config", "--global", "http.proxy", &proxy_url])
            .status();
        let _ = Command::new("git")
            .args(["config", "--global", "https.proxy", &proxy_url])
            .status();
        eprintln!(
            "{}",
            format!("✔ Git global proxy set to {}", proxy_url).green()
        );
    }

    if set_cargo {
        match set_cargo_proxy(&proxy_url) {
            Ok(_) => eprintln!(
                "{}",
                format!(
                    "✔ Cargo [http] proxy (~/.cargo/config.toml) set to {}",
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Cargo config: {}", "▲".yellow(), e),
        }
    }

    if set_npm {
        let _ = Command::new("npm")
            .args(["config", "set", "proxy", &proxy_url])
            .status();
        let _ = Command::new("npm")
            .args(["config", "set", "https-proxy", &proxy_url])
            .status();
        eprintln!("{}", format!("✔ NPM proxy set to {}", proxy_url).green());
    }

    if set_yarn {
        match set_yarn_proxy(&proxy_url) {
            Ok(_) => eprintln!(
                "{}",
                format!(
                    "✔ Yarn proxy (~/.yarnrc / ~/.yarnrc.yml) set to {}",
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Yarn config: {}", "▲".yellow(), e),
        }
    }

    if set_pnpm {
        match set_pnpm_proxy(&proxy_url) {
            Ok(_) => eprintln!(
                "{}",
                format!("✔ pnpm proxy (~/.pnpmrc) set to {}", proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update pnpm config: {}", "▲".yellow(), e),
        }
    }

    if set_bun {
        match set_bun_proxy(&proxy_url) {
            Ok(_) => eprintln!(
                "{}",
                format!("✔ Bun proxy (~/.bunfig.toml) set to {}", proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Bun config: {}", "▲".yellow(), e),
        }
    }

    if set_pip {
        let _ = Command::new("pip")
            .args(["config", "set", "global.proxy", &proxy_url])
            .status();
        eprintln!(
            "{}",
            format!("✔ Pip global proxy set to {}", proxy_url).green()
        );
    }

    if set_conda {
        match set_conda_proxy(&proxy_url) {
            Ok(_) => eprintln!(
                "{}",
                format!("✔ Conda proxy (~/.condarc) set to {}", proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Conda config: {}", "▲".yellow(), e),
        }
    }

    if set_docker {
        match set_docker_proxy(&proxy_url) {
            Ok(_) => eprintln!(
                "{}",
                format!(
                    "✔ Docker client proxy (~/.docker/config.json) set to {}",
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Docker config: {}", "▲".yellow(), e),
        }
    }

    if set_gradle {
        match set_gradle_proxy(&proxy_url) {
            Ok(_) => eprintln!(
                "{}",
                format!(
                    "✔ Gradle global proxy (~/.gradle/gradle.properties) set to {}",
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Gradle config: {}", "▲".yellow(), e),
        }
    }

    if set_maven {
        match set_maven_proxy(&proxy_url) {
            Ok(_) => eprintln!(
                "{}",
                format!(
                    "✔ Maven global proxy (~/.m2/settings.xml) set to {}",
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Maven config: {}", "▲".yellow(), e),
        }
    }

    if set_vscode {
        match set_vscode_proxy(&proxy_url) {
            Ok(ides) => {
                let list = if ides.is_empty() {
                    "VS Code".to_string()
                } else {
                    ides.join(", ")
                };
                eprintln!(
                    "{}",
                    format!("✔ IDE proxy ({}) set to {}", list, proxy_url).green()
                );
            }
            Err(e) => eprintln!(
                "{} Failed to update VS Code / Cursor IDE settings: {}",
                "▲".yellow(),
                e
            ),
        }
    }

    if set_gh {
        match set_gh_proxy(&proxy_url) {
            Ok(_) => eprintln!(
                "{}",
                format!("✔ GitHub CLI (gh) proxy set to {}", proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update GitHub CLI config: {}", "▲".yellow(), e),
        }
    }

    if set_glab {
        match set_glab_proxy(&proxy_url) {
            Ok(_) => eprintln!(
                "{}",
                format!("✔ GitLab CLI (glab) proxy set to {}", proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update GitLab CLI config: {}", "▲".yellow(), e),
        }
    }

    if set_jetbrains {
        match set_jetbrains_proxy(&proxy_url) {
            Ok(ides) => {
                let list = if ides.is_empty() {
                    "No JetBrains IDE found".to_string()
                } else {
                    ides.join(", ")
                };
                eprintln!(
                    "{}",
                    format!("✔ JetBrains IDE proxy ({}) set to {}", list, proxy_url).green()
                );
            }
            Err(e) => eprintln!(
                "{} Failed to update JetBrains IDE config: {}",
                "▲".yellow(),
                e
            ),
        }
    }

    if set_nuget {
        match set_nuget_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ NuGet (.NET) proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update NuGet config: {}", "▲".yellow(), e),
        }
    }

    if set_curl {
        match apply_curlrc_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ cURL proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update ~/.curlrc: {}", "▲".yellow(), e),
        }
    }

    if set_wget {
        match apply_wgetrc_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Wget proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update ~/.wgetrc: {}", "▲".yellow(), e),
        }
    }

    if set_gem {
        match set_gem_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Gem (Ruby) proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update ~/.gemrc: {}", "▲".yellow(), e),
        }
    }

    if set_svn {
        match set_svn_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Subversion proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Subversion config: {}", "▲".yellow(), e),
        }
    }

    if set_hg {
        match set_hg_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Mercurial (Hg) proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Mercurial config: {}", "▲".yellow(), e),
        }
    }

    if set_uv {
        match set_uv_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Astral uv proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Astral uv config: {}", "▲".yellow(), e),
        }
    }

    if set_go {
        match set_go_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Go GOPROXY ({}) set to {},direct", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!(
                "{} Failed to update Go environment config: {}",
                "▲".yellow(),
                e
            ),
        }
    }

    if set_bazel {
        match set_bazel_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Bazel action_env ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Bazel ~/.bazelrc: {}", "▲".yellow(), e),
        }
    }

    if set_composer {
        match set_composer_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ PHP Composer proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!(
                "{} Failed to update Composer config: {}",
                "▲".yellow(),
                e
            ),
        }
    }

    if set_conan {
        match set_conan_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Conan C/C++ proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Conan config: {}", "▲".yellow(), e),
        }
    }

    if set_sbt {
        match set_sbt_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Scala (sbt) proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update sbt config: {}", "▲".yellow(), e),
        }
    }

    if set_kube {
        match set_kube_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Kubernetes (kubectl) proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update kubeconfig: {}", "▲".yellow(), e),
        }
    }

    if set_julia {
        match set_julia_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Julia startup proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Julia startup.jl: {}", "▲".yellow(), e),
        }
    }

    if set_nix {
        match set_nix_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Nix package manager proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Nix nix.conf: {}", "▲".yellow(), e),
        }
    }

    if set_cabal {
        match set_cabal_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Haskell Cabal proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Cabal config: {}", "▲".yellow(), e),
        }
    }

    if set_stack {
        match set_stack_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Haskell Stack proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Haskell Stack config: {}", "▲".yellow(), e),
        }
    }

    if set_hex {
        match set_hex_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Hex (Elixir/Gleam) proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Hex hex.config: {}", "▲".yellow(), e),
        }
    }

    if set_r {
        match set_renviron_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ R (~/.Renviron) proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update R Renviron config: {}", "▲".yellow(), e),
        }
    }

    if set_opam {
        match set_opam_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ OCaml OPAM proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update OPAM config: {}", "▲".yellow(), e),
        }
    }

    if set_lein {
        match set_lein_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Clojure Leiningen proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Leiningen profiles.clj: {}", "▲".yellow(), e),
        }
    }

    if set_cpan {
        match set_cpan_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Perl CPAN proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Perl CPAN MyConfig.pm: {}", "▲".yellow(), e),
        }
    }

    eprintln!(
        "{}",
        format!("🟢 Proxy enabled! Target: {}", proxy_url)
            .green()
            .bold()
    );
    eprintln!(
        "{}",
        "Tip: To apply directly to current terminal, run: eval $(proxydoc on)".dimmed()
    );

    Ok(())
}

pub fn turn_off(
    git: bool,
    cargo: bool,
    npm: bool,
    yarn: bool,
    pnpm: bool,
    bun: bool,
    pip: bool,
    conda: bool,
    docker: bool,
    gradle: bool,
    maven: bool,
    vscode: bool,
    gh: bool,
    glab: bool,
    jetbrains: bool,
    nuget: bool,
    curl: bool,
    wget: bool,
    gem: bool,
    svn: bool,
    hg: bool,
    uv: bool,
    go: bool,
    bazel: bool,
    composer: bool,
    conan: bool,
    sbt: bool,
    kube: bool,
    julia: bool,
    nix: bool,
    cabal: bool,
    stack: bool,
    hex: bool,
    r: bool,
    opam: bool,
    lein: bool,
    cpan: bool,
    all: bool,
    shell: Option<&str>,
) -> Result<()> {
    let unset_git = git || all;
    let unset_cargo = cargo || all;
    let unset_npm = npm || all;
    let unset_yarn = yarn || all;
    let unset_pnpm = pnpm || all;
    let unset_bun = bun || all;
    let unset_pip = pip || all;
    let unset_conda = conda || all;
    let unset_docker = docker || all;
    let unset_gradle = gradle || all;
    let unset_maven = maven || all;
    let unset_vscode = vscode || all;
    let unset_gh = gh || all;
    let unset_glab = glab || all;
    let unset_jetbrains = jetbrains || all;
    let unset_nuget = nuget || all;
    let unset_gem = gem || all;
    let unset_svn = svn || all;
    let unset_hg = hg || all;
    let unset_uv = uv || all;
    let unset_go = go || all;
    let unset_bazel = bazel || all;
    let unset_composer = composer || all;
    let unset_conan = conan || all;
    let unset_sbt = sbt || all;
    let unset_kube = kube || all;
    let unset_julia = julia || all;
    let unset_nix = nix || all;
    let unset_cabal = cabal || all;
    let unset_stack = stack || all;
    let unset_hex = hex || all;
    let unset_r = r || all;
    let unset_opam = opam || all;
    let unset_lein = lein || all;
    let unset_cpan = cpan || all;
    let unset_curl = curl;
    let unset_wget = wget;

    let target_shell = detect_shell(shell);
    for line in generate_shell_unset(target_shell) {
        println!("{}", line);
    }

    if unset_git {
        let _ = Command::new("git")
            .args(["config", "--global", "--unset", "http.proxy"])
            .status();
        let _ = Command::new("git")
            .args(["config", "--global", "--unset", "https.proxy"])
            .status();
        eprintln!("{}", "✔ Git global proxy unset".green());
    }

    if unset_cargo {
        match unset_cargo_proxy() {
            Ok(true) => eprintln!(
                "{}",
                "✔ Cargo proxy unset from ~/.cargo/config.toml".green()
            ),
            Ok(false) => eprintln!("{}", "○ Cargo proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update Cargo config: {}", "▲".yellow(), e),
        }
    }

    if unset_npm {
        let _ = Command::new("npm")
            .args(["config", "delete", "proxy"])
            .status();
        let _ = Command::new("npm")
            .args(["config", "delete", "https-proxy"])
            .status();
        eprintln!("{}", "✔ NPM proxy unset".green());
    }

    if unset_yarn {
        match unset_yarn_proxy() {
            Ok(true) => eprintln!(
                "{}",
                "✔ Cleared Yarn proxy (~/.yarnrc / ~/.yarnrc.yml)".green()
            ),
            Ok(false) => eprintln!("{}", "○ Yarn proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to clear Yarn config: {}", "▲".yellow(), e),
        }
    }

    if unset_pnpm {
        match unset_pnpm_proxy() {
            Ok(true) => eprintln!("{}", "✔ pnpm proxy unset from ~/.pnpmrc".green()),
            Ok(false) => eprintln!("{}", "○ pnpm proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update pnpm config: {}", "▲".yellow(), e),
        }
    }

    if unset_bun {
        match unset_bun_proxy() {
            Ok(true) => eprintln!("{}", "✔ Bun proxy unset from ~/.bunfig.toml".green()),
            Ok(false) => eprintln!("{}", "○ Bun proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update Bun config: {}", "▲".yellow(), e),
        }
    }

    if unset_pip {
        let _ = Command::new("pip")
            .args(["config", "unset", "global.proxy"])
            .status();
        eprintln!("{}", "✔ Pip global proxy unset".green());
    }

    if unset_conda {
        match unset_conda_proxy() {
            Ok(true) => eprintln!("{}", "✔ Conda proxy unset from ~/.condarc".green()),
            Ok(false) => eprintln!("{}", "○ Conda proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update Conda config: {}", "▲".yellow(), e),
        }
    }

    if unset_docker {
        match unset_docker_proxy() {
            Ok(true) => eprintln!(
                "{}",
                "✔ Docker client proxy unset from ~/.docker/config.json".green()
            ),
            Ok(false) => eprintln!("{}", "○ Docker client proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update Docker config: {}", "▲".yellow(), e),
        }
    }

    if unset_gradle {
        match unset_gradle_proxy() {
            Ok(true) => eprintln!(
                "{}",
                "✔ Gradle global proxy unset from ~/.gradle/gradle.properties".green()
            ),
            Ok(false) => eprintln!("{}", "○ Gradle global proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update Gradle config: {}", "▲".yellow(), e),
        }
    }

    if unset_maven {
        match unset_maven_proxy() {
            Ok(true) => eprintln!(
                "{}",
                "✔ Maven global proxy unset from ~/.m2/settings.xml".green()
            ),
            Ok(false) => eprintln!("{}", "○ Maven global proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update Maven config: {}", "▲".yellow(), e),
        }
    }

    if unset_vscode {
        match unset_vscode_proxy() {
            Ok(ides) if !ides.is_empty() => eprintln!(
                "{}",
                format!("✔ IDE proxy unset from {}", ides.join(", ")).green()
            ),
            Ok(_) => eprintln!("{}", "○ IDE proxy was not set".dimmed()),
            Err(e) => eprintln!(
                "{} Failed to update VS Code / Cursor settings: {}",
                "▲".yellow(),
                e
            ),
        }
    }

    if unset_gh {
        match unset_gh_proxy() {
            Ok(true) => eprintln!("{}", "✔ Cleared GitHub CLI (gh) proxy".green()),
            Ok(false) => eprintln!("{}", "○ GitHub CLI proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to clear GitHub CLI config: {}", "▲".yellow(), e),
        }
    }

    if unset_glab {
        match unset_glab_proxy() {
            Ok(true) => eprintln!("{}", "✔ Cleared GitLab CLI (glab) proxy".green()),
            Ok(false) => eprintln!("{}", "○ GitLab CLI (glab) proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to clear GitLab CLI config: {}", "▲".yellow(), e),
        }
    }

    if unset_jetbrains {
        match unset_jetbrains_proxy() {
            Ok(ides) if !ides.is_empty() => eprintln!(
                "{}",
                format!("✔ JetBrains IDE proxy deactivated in {}", ides.join(", ")).green()
            ),
            Ok(_) => eprintln!("{}", "○ JetBrains IDE proxy was not set".dimmed()),
            Err(e) => eprintln!(
                "{} Failed to deactivate JetBrains IDE proxy: {}",
                "▲".yellow(),
                e
            ),
        }
    }

    if unset_nuget {
        match unset_nuget_proxy() {
            Ok(true) => eprintln!(
                "{}",
                "✔ NuGet global proxy removed from NuGet.Config".green()
            ),
            Ok(false) => eprintln!("{}", "○ NuGet global proxy was not configured".dimmed()),
            Err(e) => eprintln!("{} Failed to unset NuGet proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_curl {
        match unset_curlrc_proxy() {
            Ok(true) => eprintln!("{}", "✔ cURL proxy removed from ~/.curlrc".green()),
            Ok(false) => eprintln!("{}", "○ cURL proxy was not set in ~/.curlrc".dimmed()),
            Err(e) => eprintln!("{} Failed to unset ~/.curlrc proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_wget {
        match unset_wgetrc_proxy() {
            Ok(true) => eprintln!("{}", "✔ Wget proxy removed from ~/.wgetrc".green()),
            Ok(false) => eprintln!("{}", "○ Wget proxy was not set in ~/.wgetrc".dimmed()),
            Err(e) => eprintln!("{} Failed to unset ~/.wgetrc proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_gem {
        match unset_gem_proxy() {
            Ok(true) => eprintln!("{}", "✔ Gem (Ruby) proxy removed from ~/.gemrc".green()),
            Ok(false) => eprintln!("{}", "○ Gem (Ruby) proxy was not set in ~/.gemrc".dimmed()),
            Err(e) => eprintln!("{} Failed to unset ~/.gemrc proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_svn {
        match unset_svn_proxy() {
            Ok(true) => eprintln!(
                "{}",
                "✔ Subversion proxy removed from servers config".green()
            ),
            Ok(false) => eprintln!(
                "{}",
                "○ Subversion proxy was not set in servers config".dimmed()
            ),
            Err(e) => eprintln!("{} Failed to unset Subversion proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_hg {
        match unset_hg_proxy() {
            Ok(true) => eprintln!("{}", "✔ Mercurial (Hg) proxy removed from ~/.hgrc".green()),
            Ok(false) => eprintln!(
                "{}",
                "○ Mercurial (Hg) proxy was not set in ~/.hgrc".dimmed()
            ),
            Err(e) => eprintln!("{} Failed to unset Mercurial proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_uv {
        match unset_uv_proxy() {
            Ok(true) => eprintln!("{}", "✔ Astral uv proxy unset from uv.toml".green()),
            Ok(false) => eprintln!("{}", "○ Astral uv proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update Astral uv config: {}", "▲".yellow(), e),
        }
    }

    if unset_go {
        match unset_go_proxy() {
            Ok(true) => eprintln!("{}", "✔ Go GOPROXY unset / reset to default".green()),
            Ok(false) => eprintln!("{}", "○ Go GOPROXY was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update Go config: {}", "▲".yellow(), e),
        }
    }

    if unset_bazel {
        match unset_bazel_proxy() {
            Ok(true) => eprintln!("{}", "✔ Bazel action_env proxy removed from ~/.bazelrc".green()),
            Ok(false) => eprintln!("{}", "○ Bazel action_env proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Bazel proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_composer {
        match unset_composer_proxy() {
            Ok(true) => eprintln!("{}", "✔ PHP Composer proxy removed from config.json".green()),
            Ok(false) => eprintln!("{}", "○ PHP Composer proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset PHP Composer proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_conan {
        match unset_conan_proxy() {
            Ok(true) => eprintln!("{}", "✔ Conan C/C++ proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Conan C/C++ proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Conan C/C++ proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_sbt {
        match unset_sbt_proxy() {
            Ok(true) => eprintln!("{}", "✔ Scala (sbt) proxy removed from sbtopts".green()),
            Ok(false) => eprintln!("{}", "○ Scala (sbt) proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Scala (sbt) proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_kube {
        match unset_kube_proxy() {
            Ok(true) => eprintln!("{}", "✔ Kubernetes (kubectl) cluster proxy-url removed".green()),
            Ok(false) => eprintln!("{}", "○ Kubernetes (kubectl) cluster proxy-url was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Kubernetes cluster proxy-url: {}", "▲".yellow(), e),
        }
    }

    if unset_julia {
        match unset_julia_proxy() {
            Ok(true) => eprintln!("{}", "✔ Julia startup proxy removed from startup.jl".green()),
            Ok(false) => eprintln!("{}", "○ Julia startup proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Julia startup proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_nix {
        match unset_nix_proxy() {
            Ok(true) => eprintln!("{}", "✔ Nix http-proxy removed from nix.conf".green()),
            Ok(false) => eprintln!("{}", "○ Nix http-proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Nix http-proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_cabal {
        match unset_cabal_proxy() {
            Ok(true) => eprintln!("{}", "✔ Haskell Cabal proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Haskell Cabal proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Haskell Cabal proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_stack {
        match unset_stack_proxy() {
            Ok(true) => eprintln!("{}", "✔ Haskell Stack proxy removed from config.yaml".green()),
            Ok(false) => eprintln!("{}", "○ Haskell Stack proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Haskell Stack proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_hex {
        match unset_hex_proxy() {
            Ok(true) => eprintln!("{}", "✔ Hex (Elixir/Gleam) proxy removed from hex.config".green()),
            Ok(false) => eprintln!("{}", "○ Hex proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Hex proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_r {
        match unset_renviron_proxy() {
            Ok(true) => eprintln!("{}", "✔ R proxy removed from ~/.Renviron".green()),
            Ok(false) => eprintln!("{}", "○ R proxy was not set in ~/.Renviron".dimmed()),
            Err(e) => eprintln!("{} Failed to unset R Renviron proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_opam {
        match unset_opam_proxy() {
            Ok(true) => eprintln!("{}", "✔ OCaml OPAM proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ OCaml OPAM proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset OCaml OPAM proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_lein {
        match unset_lein_proxy() {
            Ok(true) => eprintln!("{}", "✔ Clojure Leiningen proxy removed from profiles.clj".green()),
            Ok(false) => eprintln!("{}", "○ Clojure Leiningen proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Leiningen proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_cpan {
        match unset_cpan_proxy() {
            Ok(true) => eprintln!("{}", "✔ Perl CPAN proxy removed from MyConfig.pm".green()),
            Ok(false) => eprintln!("{}", "○ Perl CPAN proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Perl CPAN proxy: {}", "▲".yellow(), e),
        }
    }

    eprintln!(
        "{}",
        "🔴 Proxy disabled across terminal session!".yellow().bold()
    );

    Ok(())
}

pub fn print_init_script(shell: Option<&str>) {
    let target_shell = detect_shell(shell);
    match target_shell {
        ShellType::Fish => {
            println!(
                r#"# proxydoc shell integration for fish
function pon
    set -l target $argv[1]
    test -z "$target"; and set target "auto"
    eval (proxydoc on $target --shell fish)
end

function poff
    eval (proxydoc off --shell fish)
end

alias pdoc="proxydoc doctor"
alias pfix="proxydoc fix"
alias ptest="proxydoc test"
alias pcomp="proxydoc test --compare"
"#
            );
        }
        ShellType::PowerShell => {
            println!(
                r#"# proxydoc shell integration for PowerShell
function Set-ProxyDocOn {{
    param([string]$target = "auto")
    Invoke-Expression (& proxydoc on $target --shell pwsh | Out-String)
}}

function Set-ProxyDocOff {{
    Invoke-Expression (& proxydoc off --shell pwsh | Out-String)
}}

Set-Alias pon Set-ProxyDocOn
Set-Alias poff Set-ProxyDocOff
Set-Alias pdoc "proxydoc doctor"
Set-Alias pfix "proxydoc fix"
Set-Alias ptest "proxydoc test"
Set-Alias pcomp "proxydoc test --compare"
"#
            );
        }
        ShellType::Bash => {
            println!(
                r#"# proxydoc shell integration for bash/zsh
proxydoc-on() {{
    local target="${{1:-auto}}"
    eval "$(proxydoc on "$target")"
}}

proxydoc-off() {{
    eval "$(proxydoc off)"
}}

alias pon="proxydoc-on"
alias poff="proxydoc-off"
alias pdoc="proxydoc doctor"
alias pfix="proxydoc fix"
alias ptest="proxydoc test"
alias pcomp="proxydoc test --compare"
"#
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_shell() {
        assert_eq!(detect_shell(Some("fish")), ShellType::Fish);
        assert_eq!(detect_shell(Some("pwsh")), ShellType::PowerShell);
        assert_eq!(detect_shell(Some("powershell")), ShellType::PowerShell);
        assert_eq!(detect_shell(Some("bash")), ShellType::Bash);
        assert_eq!(detect_shell(Some("zsh")), ShellType::Bash);
    }

    #[test]
    fn test_generate_shell_export_bash() {
        let lines = generate_shell_export("http://127.0.0.1:7890", ShellType::Bash);
        assert!(lines.contains(&"export http_proxy=\"http://127.0.0.1:7890\"".to_string()));
        assert!(lines.contains(
            &"export no_proxy=\"localhost,127.0.0.1,::1,localaddress,.local\"".to_string()
        ));
    }

    #[test]
    fn test_generate_shell_export_fish() {
        let lines = generate_shell_export("http://127.0.0.1:7890", ShellType::Fish);
        assert!(lines.contains(&"set -gx http_proxy \"http://127.0.0.1:7890\";".to_string()));
        assert!(lines.contains(
            &"set -gx no_proxy \"localhost,127.0.0.1,::1,localaddress,.local\";".to_string()
        ));
    }

    #[test]
    fn test_generate_shell_export_pwsh() {
        let lines = generate_shell_export("http://127.0.0.1:7890", ShellType::PowerShell);
        assert!(lines.contains(&"$env:http_proxy=\"http://127.0.0.1:7890\"".to_string()));
    }

    #[test]
    fn test_generate_shell_unset_all() {
        let bash_unset = generate_shell_unset(ShellType::Bash);
        assert!(bash_unset.contains(&"unset http_proxy".to_string()));

        let fish_unset = generate_shell_unset(ShellType::Fish);
        assert!(fish_unset.contains(&"set -e http_proxy;".to_string()));

        let pwsh_unset = generate_shell_unset(ShellType::PowerShell);
        assert!(pwsh_unset.contains(&"$env:http_proxy=$null".to_string()));
    }

    #[test]
    fn test_apply_and_remove_docker_proxy_json() {
        let mut root = serde_json::json!({
            "auths": {
                "registry.example.com": {}
            }
        });

        apply_docker_proxy_json(&mut root, "http://127.0.0.1:7890");
        assert_eq!(
            root.pointer("/proxies/default/httpProxy")
                .unwrap()
                .as_str()
                .unwrap(),
            "http://127.0.0.1:7890"
        );
        // Preserved original keys
        assert!(root.get("auths").is_some());

        let removed = remove_docker_proxy_json(&mut root);
        assert!(removed);
        assert!(root.pointer("/proxies/default").is_none());
        assert!(root.get("auths").is_some());
    }

    #[test]
    fn test_extract_host_port() {
        assert_eq!(
            extract_host_port("http://127.0.0.1:7890"),
            ("127.0.0.1".to_string(), 7890)
        );
        assert_eq!(
            extract_host_port("127.0.0.1:1082"),
            ("127.0.0.1".to_string(), 1082)
        );
        assert_eq!(extract_host_port("7897"), ("127.0.0.1".to_string(), 7890));
    }

    #[test]
    fn test_apply_and_remove_gradle_proxy_properties() {
        let initial = "org.gradle.jvmargs=-Xmx2048m\n";
        let with_proxy = apply_gradle_proxy_properties(initial, "127.0.0.1", 7890);
        assert!(with_proxy.contains("systemProp.http.proxyHost=127.0.0.1"));
        assert!(with_proxy.contains("systemProp.http.proxyPort=7890"));
        assert!(with_proxy.contains("systemProp.https.proxyHost=127.0.0.1"));
        assert!(with_proxy.contains("systemProp.https.proxyPort=7890"));
        assert!(with_proxy.contains("org.gradle.jvmargs=-Xmx2048m"));

        let (cleaned, removed) = remove_gradle_proxy_properties(&with_proxy);
        assert!(removed);
        assert!(!cleaned.contains("systemProp.http.proxyHost"));
        assert!(!cleaned.contains("systemProp.https.proxyPort"));
        assert!(cleaned.contains("org.gradle.jvmargs=-Xmx2048m"));
    }

    #[test]
    fn test_apply_and_remove_maven_proxy_xml() {
        let initial = "<settings>\n  <localRepository>/path/to/repo</localRepository>\n</settings>";
        let with_proxy = apply_maven_proxy_xml(initial, "127.0.0.1", 7890);
        assert!(with_proxy.contains("<id>proxydoc-proxy</id>"));
        assert!(with_proxy.contains("<host>127.0.0.1</host>"));
        assert!(with_proxy.contains("<port>7890</port>"));
        assert!(with_proxy.contains("<localRepository>/path/to/repo</localRepository>"));

        let (cleaned, removed) = remove_maven_proxy_xml(&with_proxy);
        assert!(removed);
        assert!(!cleaned.contains("<id>proxydoc-proxy</id>"));
        assert!(cleaned.contains("<localRepository>/path/to/repo</localRepository>"));

        let empty_initial = "";
        let from_empty = apply_maven_proxy_xml(empty_initial, "127.0.0.1", 7890);
        assert!(from_empty.contains("<settings"));
        assert!(from_empty.contains("<id>proxydoc-proxy</id>"));
    }

    #[test]
    fn test_apply_and_remove_cargo_proxy_toml() {
        let initial = "[build]\njobs = 4\n";
        let with_proxy = apply_cargo_proxy_toml(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("[http]"));
        assert!(with_proxy.contains("proxy = \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("jobs = 4"));

        // Replace existing proxy
        let updated = apply_cargo_proxy_toml(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("proxy = \"http://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_cargo_proxy_toml(&updated);
        assert!(removed);
        assert!(!cleaned.contains("proxy = "));
        assert!(cleaned.contains("jobs = 4"));
    }

    #[test]
    fn test_apply_and_remove_yarn_proxy_yarnrc() {
        let initial = "registry \"https://registry.yarnpkg.com\"\n";
        let with_proxy = apply_yarn_proxy_yarnrc(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("proxy \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("https-proxy \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("registry \"https://registry.yarnpkg.com\""));

        // Replace existing
        let updated = apply_yarn_proxy_yarnrc(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("proxy \"http://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_yarn_proxy_yarnrc(&updated);
        assert!(removed);
        assert!(!cleaned.contains("proxy"));
        assert!(cleaned.contains("registry"));
    }

    #[test]
    fn test_apply_and_remove_yarn_proxy_yml() {
        let initial = "nodeLinker: node-modules\n";
        let with_proxy = apply_yarn_proxy_yml(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("httpProxy: \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("httpsProxy: \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("nodeLinker: node-modules"));

        // Replace existing
        let updated = apply_yarn_proxy_yml(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("httpProxy: \"http://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_yarn_proxy_yml(&updated);
        assert!(removed);
        assert!(!cleaned.contains("httpProxy"));
        assert!(!cleaned.contains("httpsProxy"));
        assert!(cleaned.contains("nodeLinker: node-modules"));
    }

    #[test]
    fn test_apply_and_remove_condarc_proxy() {
        let initial = "channels:\n  - defaults\nssl_verify: true\n";
        let with_proxy = apply_condarc_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("proxy_servers:"));
        assert!(with_proxy.contains("http: http://127.0.0.1:7890"));
        assert!(with_proxy.contains("https: http://127.0.0.1:7890"));
        assert!(with_proxy.contains("channels:\n  - defaults"));
        assert!(with_proxy.contains("ssl_verify: true"));

        // Replace existing proxy
        let updated = apply_condarc_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("http: http://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_condarc_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("proxy_servers:"));
        assert!(!cleaned.contains("1080"));
        assert!(cleaned.contains("channels:\n  - defaults"));
        assert!(cleaned.contains("ssl_verify: true"));
    }

    #[test]
    fn test_apply_and_remove_pnpmrc_proxy() {
        let initial = "registry=https://registry.npmjs.org/\n";
        let with_proxy = apply_pnpmrc_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("proxy=http://127.0.0.1:7890"));
        assert!(with_proxy.contains("https-proxy=http://127.0.0.1:7890"));
        assert!(with_proxy.contains("registry=https://registry.npmjs.org/"));

        // Replace existing proxy
        let updated = apply_pnpmrc_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("proxy=http://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_pnpmrc_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("proxy="));
        assert!(cleaned.contains("registry="));
    }

    #[test]
    fn test_apply_and_remove_bun_proxy_toml() {
        let initial = "[install]\nexact = true\n";
        let with_proxy = apply_bun_proxy_toml(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("proxy = \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("exact = true"));

        // Replace existing
        let updated = apply_bun_proxy_toml(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("proxy = \"http://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_bun_proxy_toml(&updated);
        assert!(removed);
        assert!(!cleaned.contains("proxy = "));
        assert!(cleaned.contains("exact = true"));

        // Fresh empty file
        let empty_with = apply_bun_proxy_toml("", "http://127.0.0.1:7890");
        assert!(empty_with.contains("[install]"));
        assert!(empty_with.contains("proxy = \"http://127.0.0.1:7890\""));
    }

    #[test]
    fn test_apply_and_remove_vscode_proxy_json() {
        let mut root = serde_json::json!({
            "workbench.colorTheme": "One Dark Pro",
            "editor.fontSize": 14
        });

        apply_vscode_proxy_json(&mut root, "http://127.0.0.1:7890");
        assert_eq!(root["http.proxy"], "http://127.0.0.1:7890");
        assert_eq!(root["http.proxySupport"], "override");
        assert_eq!(root["workbench.colorTheme"], "One Dark Pro");

        let removed = remove_vscode_proxy_json(&mut root);
        assert!(removed);
        assert!(root.get("http.proxy").is_none());
        assert!(root.get("http.proxySupport").is_none());
        assert_eq!(root["workbench.colorTheme"], "One Dark Pro");
    }

    #[test]
    fn test_apply_and_remove_gh_config_proxy() {
        let initial = "# gh configuration\ngit_protocol: https\neditor: vim\n";
        let with_proxy = apply_gh_config_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("http_proxy: http://127.0.0.1:7890"));
        assert!(with_proxy.contains("git_protocol: https"));

        // Replace existing
        let updated = apply_gh_config_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("http_proxy: http://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_gh_config_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("http_proxy:"));
        assert!(cleaned.contains("git_protocol: https"));
    }

    #[test]
    fn test_apply_and_deactivate_jetbrains_proxy_xml() {
        let initial = r#"<application>
  <component name="HttpConfigurable">
    <option name="USE_PROXY_PAC" value="true" />
  </component>
</application>"#;
        let configured = apply_jetbrains_proxy_xml(initial, "127.0.0.1", 7890, false);
        assert!(configured.contains(r#"<option name="USE_HTTP_PROXY" value="true" />"#));
        assert!(configured.contains(r#"<option name="PROXY_HOST" value="127.0.0.1" />"#));
        assert!(configured.contains(r#"<option name="PROXY_PORT" value="7890" />"#));
        assert!(configured.contains(r#"<option name="PROXY_TYPE_NUM" value="1" />"#));

        let (deactivated, modified) = deactivate_jetbrains_proxy_xml(&configured);
        assert!(modified);
        assert!(deactivated.contains(r#"<option name="USE_HTTP_PROXY" value="false" />"#));

        // Test SOCKS
        let socks_cfg = apply_jetbrains_proxy_xml(initial, "127.0.0.1", 1080, true);
        assert!(socks_cfg.contains(r#"<option name="PROXY_TYPE_NUM" value="2" />"#));
    }

    #[test]
    fn test_apply_jetbrains_proxy_xml_new() {
        let empty = "";
        let configured = apply_jetbrains_proxy_xml(empty, "127.0.0.1", 7890, false);
        assert!(configured.contains("<component name=\"HttpConfigurable\">"));
        assert!(configured.contains(r#"<option name="USE_HTTP_PROXY" value="true" />"#));
        assert!(configured.contains(r#"<option name="PROXY_PORT" value="7890" />"#));
    }

    #[test]
    fn test_apply_and_remove_nuget_proxy_xml() {
        let initial = r#"<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <packageSources>
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" />
  </packageSources>
  <config>
    <add key="repositoryPath" value="./packages" />
  </config>
</configuration>"#;
        let configured = apply_nuget_proxy_xml(initial, "http://127.0.0.1:7890");
        assert!(configured.contains(r#"<add key="http_proxy" value="http://127.0.0.1:7890" />"#));
        assert!(configured.contains(r#"<add key="repositoryPath" value="./packages" />"#));

        let (cleaned, modified) = remove_nuget_proxy_xml(&configured);
        assert!(modified);
        assert!(!cleaned.contains("http_proxy"));
        assert!(cleaned.contains(r#"<add key="repositoryPath" value="./packages" />"#));
    }

    #[test]
    fn test_apply_nuget_proxy_xml_new() {
        let empty = "";
        let configured = apply_nuget_proxy_xml(empty, "http://127.0.0.1:7890");
        assert!(configured.contains("<configuration>"));
        assert!(configured.contains("<config>"));
        assert!(configured.contains(r#"<add key="http_proxy" value="http://127.0.0.1:7890" />"#));
    }

    #[test]
    fn test_apply_nuget_proxy_xml_replace_existing() {
        let initial = r#"<configuration>
  <config>
    <add key="http_proxy" value="http://127.0.0.1:1080" />
  </config>
</configuration>"#;
        let configured = apply_nuget_proxy_xml(initial, "http://127.0.0.1:7890");
        assert!(configured.contains(r#"<add key="http_proxy" value="http://127.0.0.1:7890" />"#));
        assert!(!configured.contains("1080"));
    }

    #[test]
    fn test_apply_and_remove_curlrc_lines() {
        let initial = "# curl config\nsilent\nproxy = \"http://127.0.0.1:1080\"\n";
        let configured = apply_curlrc_lines(initial, "http://127.0.0.1:7890");
        assert!(configured.contains("proxy = \"http://127.0.0.1:7890\""));
        assert!(!configured.contains("1080"));
        assert!(configured.contains("silent"));

        let (cleaned, changed) = remove_curlrc_lines(&configured);
        assert!(changed);
        assert!(!cleaned.contains("proxy ="));
        assert!(cleaned.contains("silent"));
    }

    #[test]
    fn test_apply_and_remove_wgetrc_lines() {
        let initial = "# wget config\nhttp_proxy = http://127.0.0.1:1080\nhttps_proxy = http://127.0.0.1:1080\nuse_proxy = on\n";
        let configured = apply_wgetrc_lines(initial, "http://127.0.0.1:7890");
        assert!(configured.contains("http_proxy = http://127.0.0.1:7890"));
        assert!(configured.contains("https_proxy = http://127.0.0.1:7890"));
        assert!(!configured.contains("1080"));

        let (cleaned, changed) = remove_wgetrc_lines(&configured);
        assert!(changed);
        assert!(!cleaned.contains("http_proxy"));
        assert!(!cleaned.contains("https_proxy"));
        assert!(!cleaned.contains("use_proxy"));
    }

    #[test]
    fn test_apply_and_remove_gemrc_proxy() {
        let initial = "---\n:backtrace: false\nhttp_proxy: http://127.0.0.1:1080\n";
        let configured = apply_gemrc_lines(initial, "http://127.0.0.1:7890");
        assert!(configured.contains("http_proxy: http://127.0.0.1:7890"));
        assert!(!configured.contains("1080"));

        let (cleaned, changed) = remove_gemrc_lines(&configured);
        assert!(changed);
        assert!(!cleaned.contains("http_proxy"));
        assert!(cleaned.contains(":backtrace: false"));
    }

    #[test]
    fn test_apply_and_remove_svn_servers_proxy() {
        let initial = "[groups]\ng1 = *.internal\n\n[global]\nhttp-proxy-host = 127.0.0.1\nhttp-proxy-port = 1080\n";
        let configured = apply_svn_servers_content(initial, "127.0.0.1", 7890);
        assert!(configured.contains("http-proxy-port = 7890"));
        assert!(!configured.contains("1080"));

        let (cleaned, changed) = remove_svn_servers_content(&configured);
        assert!(changed);
        assert!(!cleaned.contains("http-proxy-host"));
        assert!(!cleaned.contains("http-proxy-port"));
        assert!(cleaned.contains("[groups]"));
    }

    #[test]
    fn test_apply_and_remove_hgrc_proxy() {
        let initial = "[ui]\nusername = Bob\n";
        let configured = apply_hgrc_proxy(initial, "http://127.0.0.1:7890");
        assert!(configured.contains("[http_proxy]"));
        assert!(configured.contains("host = http://127.0.0.1:7890"));
        assert!(configured.contains("no = localhost, 127.0.0.1"));

        // Replace existing
        let updated = apply_hgrc_proxy(&configured, "http://127.0.0.1:1082");
        assert!(updated.contains("host = http://127.0.0.1:1082"));
        assert!(!updated.contains("7890"));

        // Remove
        let (cleaned, changed) = remove_hgrc_proxy(&updated);
        assert!(changed);
        assert!(!cleaned.contains("host = "));
        assert!(cleaned.contains("[ui]"));
        assert!(cleaned.contains("username = Bob"));
    }

    #[test]
    fn test_apply_and_remove_uv_proxy_toml() {
        let initial = "[tool.uv]\ncache-dir = \"~/.cache/uv\"\n";
        let configured = apply_uv_proxy_toml(initial, "http://127.0.0.1:7890");
        assert!(configured.contains("http-proxy = \"http://127.0.0.1:7890\""));
        assert!(configured.contains("[tool.uv]"));

        // Replace existing
        let updated = apply_uv_proxy_toml(&configured, "http://127.0.0.1:1080");
        assert!(updated.contains("http-proxy = \"http://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        // Remove
        let (cleaned, changed) = remove_uv_proxy_toml(&updated);
        assert!(changed);
        assert!(!cleaned.contains("http-proxy"));
        assert!(cleaned.contains("[tool.uv]"));
    }

    #[test]
    fn test_apply_and_remove_go_env_content() {
        let initial = "GOPRIVATE=*.corp.example.com\n";
        let configured = apply_go_env_content(initial, "http://127.0.0.1:7890");
        assert!(configured.contains("GOPROXY=http://127.0.0.1:7890,direct"));
        assert!(configured.contains("GOPRIVATE="));

        // Replace existing
        let updated = apply_go_env_content(&configured, "http://127.0.0.1:1080");
        assert!(updated.contains("GOPROXY=http://127.0.0.1:1080,direct"));
        assert!(!updated.contains("7890"));

        // Remove
        let (cleaned, changed) = remove_go_env_content(&updated);
        assert!(changed);
        assert!(!cleaned.contains("GOPROXY"));
        assert!(cleaned.contains("GOPRIVATE"));
    }

    #[test]
    fn test_apply_and_remove_bazelrc_proxy() {
        let initial = "build --cxxopt=-std=c++17\ntest --verbose_failures\n";
        let configured = apply_bazelrc_proxy(initial, "http://127.0.0.1:7890");
        assert!(configured.contains("build --action_env=HTTP_PROXY=http://127.0.0.1:7890"));
        assert!(configured.contains("build --action_env=HTTPS_PROXY=http://127.0.0.1:7890"));
        assert!(configured.contains("--cxxopt=-std=c++17"));

        // Replace existing
        let updated = apply_bazelrc_proxy(&configured, "http://127.0.0.1:8080");
        assert!(updated.contains("build --action_env=HTTP_PROXY=http://127.0.0.1:8080"));
        assert!(updated.contains("build --action_env=HTTPS_PROXY=http://127.0.0.1:8080"));
        assert!(!updated.contains("7890"));

        // Remove
        let (cleaned, changed) = remove_bazelrc_proxy(&updated);
        assert!(changed);
        assert!(!cleaned.contains("--action_env=HTTP_PROXY="));
        assert!(!cleaned.contains("--action_env=HTTPS_PROXY="));
        assert!(cleaned.contains("--cxxopt=-std=c++17"));
    }

    #[test]
    fn test_apply_and_remove_composer_proxy_json() {
        let initial = r#"{ "repositories": { "packagist": { "type": "composer", "url": "https://packagist.org" } } }"#;
        let configured = apply_composer_proxy_json(initial, "http://127.0.0.1:7890");
        assert!(configured.contains("\"http-proxy\": \"http://127.0.0.1:7890\""));
        assert!(configured.contains("\"https-proxy\": \"http://127.0.0.1:7890\""));
        assert!(configured.contains("packagist.org"));

        // Replace existing
        let updated = apply_composer_proxy_json(&configured, "http://127.0.0.1:8080");
        assert!(updated.contains("\"http-proxy\": \"http://127.0.0.1:8080\""));
        assert!(!updated.contains("7890"));

        // Remove
        let (cleaned, changed) = remove_composer_proxy_json(&updated);
        assert!(changed);
        assert!(!cleaned.contains("http-proxy"));
        assert!(!cleaned.contains("https-proxy"));
        assert!(cleaned.contains("packagist.org"));
    }

    #[test]
    fn test_apply_and_remove_conan2_global_conf() {
        let initial = "tools.build:jobs = 8\n";
        let configured = apply_conan2_global_conf(initial, "http://127.0.0.1:7890");
        assert!(configured.contains("core.net.http:proxies = {\"http\": \"http://127.0.0.1:7890\", \"https\": \"http://127.0.0.1:7890\"}"));
        assert!(configured.contains("tools.build:jobs = 8"));

        // Replace existing
        let updated = apply_conan2_global_conf(&configured, "http://127.0.0.1:8080");
        assert!(updated.contains("8080"));
        assert!(!updated.contains("7890"));

        // Remove
        let (cleaned, changed) = remove_conan2_global_conf(&updated);
        assert!(changed);
        assert!(!cleaned.contains("core.net.http:proxies"));
        assert!(cleaned.contains("tools.build:jobs = 8"));
    }

    #[test]
    fn test_apply_and_remove_conan1_conf() {
        let initial = "[general]\nrevisions_enabled = 1\n";
        let configured = apply_conan1_conf(initial, "http://127.0.0.1:7890");
        assert!(configured.contains("[proxies]"));
        assert!(configured.contains("http = http://127.0.0.1:7890"));
        assert!(configured.contains("https = http://127.0.0.1:7890"));
        assert!(configured.contains("revisions_enabled = 1"));

        // Replace existing
        let updated = apply_conan1_conf(&configured, "http://127.0.0.1:8080");
        assert!(updated.contains("8080"));
        assert!(!updated.contains("7890"));

        // Remove
        let (cleaned, changed) = remove_conan1_conf(&updated);
        assert!(changed);
        assert!(!cleaned.contains("http = http://127.0.0.1:8080"));
        assert!(!cleaned.contains("https = http://127.0.0.1:8080"));
        assert!(cleaned.contains("revisions_enabled = 1"));
    }

    #[test]
    fn test_apply_and_remove_sbtopts_proxy() {
        let initial = "-J-Xmx2G\n";
        let configured = apply_sbtopts_proxy(initial, "http://127.0.0.1:7890");
        assert!(configured.contains("-Dhttp.proxyHost=127.0.0.1"));
        assert!(configured.contains("-Dhttp.proxyPort=7890"));
        assert!(configured.contains("-Dhttps.proxyHost=127.0.0.1"));
        assert!(configured.contains("-Dhttps.proxyPort=7890"));
        assert!(configured.contains("-J-Xmx2G"));

        // Replace existing
        let updated = apply_sbtopts_proxy(&configured, "http://127.0.0.1:8080");
        assert!(updated.contains("-Dhttp.proxyPort=8080"));
        assert!(!updated.contains("7890"));

        // Remove
        let (cleaned, changed) = remove_sbtopts_proxy(&updated);
        assert!(changed);
        assert!(!cleaned.contains("-Dhttp.proxyHost"));
        assert!(!cleaned.contains("-Dhttp.proxyPort"));
        assert!(cleaned.contains("-J-Xmx2G"));
    }

    #[test]
    fn test_apply_and_remove_kubeconfig_proxy() {
        let initial = r#"apiVersion: v1
clusters:
- cluster:
    server: https://192.168.1.100:6443
  name: my-cluster
contexts: []
"#;
        let configured = apply_kubeconfig_proxy(initial, "http://127.0.0.1:7890");
        assert!(configured.contains("proxy-url: http://127.0.0.1:7890"));
        assert!(configured.contains("server: https://192.168.1.100:6443"));

        // Replace existing
        let updated = apply_kubeconfig_proxy(&configured, "http://127.0.0.1:8080");
        assert!(updated.contains("proxy-url: http://127.0.0.1:8080"));
        assert!(!updated.contains("7890"));

        // Remove
        let (cleaned, changed) = remove_kubeconfig_proxy(&updated);
        assert!(changed);
        assert!(!cleaned.contains("proxy-url:"));
        assert!(cleaned.contains("server: https://192.168.1.100:6443"));
    }

    #[test]
    fn test_apply_and_remove_glab_config_proxy() {
        let initial = "version: 1\ngit_protocol: https\n";
        let with_proxy = apply_glab_config_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("glab_proxy: http://127.0.0.1:7890"));
        assert!(with_proxy.contains("git_protocol: https"));

        // Replace existing
        let updated = apply_glab_config_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("glab_proxy: http://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_glab_config_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("glab_proxy:"));
        assert!(cleaned.contains("git_protocol: https"));
    }

    #[test]
    fn test_apply_and_remove_julia_startup_proxy() {
        let initial = "using Pkg\nprintln(\"ready\")\n";
        let configured = apply_julia_startup_proxy(initial, "http://127.0.0.1:7890");
        assert!(configured.contains("ENV[\"HTTP_PROXY\"] = \"http://127.0.0.1:7890\""));
        assert!(configured.contains("ENV[\"HTTPS_PROXY\"] = \"http://127.0.0.1:7890\""));
        assert!(configured.contains("ENV[\"all_proxy\"] = \"http://127.0.0.1:7890\""));
        assert!(configured.contains("using Pkg"));

        // Replace existing
        let updated = apply_julia_startup_proxy(&configured, "http://127.0.0.1:8080");
        assert!(updated.contains("8080"));
        assert!(!updated.contains("7890"));

        // Remove
        let (cleaned, changed) = remove_julia_startup_proxy(&updated);
        assert!(changed);
        assert!(!cleaned.contains("HTTP_PROXY"));
        assert!(!cleaned.contains("all_proxy"));
        assert!(cleaned.contains("using Pkg"));
    }

    #[test]
    fn test_apply_and_remove_nix_config_proxy() {
        let initial = "substituters = https://cache.nixos.org\nmax-jobs = auto\n";
        let with_proxy = apply_nix_config_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("http-proxy = http://127.0.0.1:7890"));
        assert!(with_proxy.contains("substituters = https://cache.nixos.org"));

        // Replace existing
        let updated = apply_nix_config_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("http-proxy = http://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_nix_config_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("http-proxy"));
        assert!(cleaned.contains("max-jobs = auto"));
    }

    #[test]
    fn test_apply_and_remove_cabal_config_proxy() {
        let initial = "-- Cabal config\nremote-repo-cache: ~/.cabal/packages\n";
        let with_proxy = apply_cabal_config_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("proxy: http://127.0.0.1:7890"));
        assert!(with_proxy.contains("remote-repo-cache:"));

        // Replace existing
        let updated = apply_cabal_config_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("proxy: http://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_cabal_config_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("proxy:"));
        assert!(cleaned.contains("remote-repo-cache:"));
    }

    #[test]
    fn test_apply_and_remove_stack_config_proxy() {
        let initial = "# Stack config\nsystem-ghc: true\n";
        let with_proxy = apply_stack_config_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("proxy:"));
        assert!(with_proxy.contains("  http: \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("system-ghc: true"));

        // Replace existing
        let updated = apply_stack_config_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("  http: \"http://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_stack_config_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("proxy:"));
        assert!(cleaned.contains("system-ghc: true"));
    }

    #[test]
    fn test_apply_and_remove_hex_config_proxy() {
        let initial = "% Hex config\n{release, true}.\n";
        let with_proxy = apply_hex_config_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("{http_proxy, \"http://127.0.0.1:7890\"}."));
        assert!(with_proxy.contains("{https_proxy, \"http://127.0.0.1:7890\"}."));
        assert!(with_proxy.contains("{release, true}."));

        // Replace existing
        let updated = apply_hex_config_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("{http_proxy, \"http://127.0.0.1:1080\"}."));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_hex_config_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("http_proxy"));
        assert!(cleaned.contains("{release, true}."));
    }

    #[test]
    fn test_apply_and_remove_renviron_proxy() {
        let initial = "# R user environment\nCRAN_MIRROR=\"https://cloud.r-project.org\"\n";
        let with_proxy = apply_renviron_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("http_proxy=\"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("https_proxy=\"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("all_proxy=\"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("no_proxy=\"localhost,127.0.0.1\""));
        assert!(with_proxy.contains("CRAN_MIRROR"));

        // Replace existing
        let updated = apply_renviron_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("http_proxy=\"http://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_renviron_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("http_proxy="));
        assert!(!cleaned.contains("https_proxy="));
        assert!(cleaned.contains("CRAN_MIRROR"));
    }

    #[test]
    fn test_apply_and_remove_opam_config_proxy() {
        let initial = "opam-version: \"2.0\"\nrepositories: \"default\"\n";
        let with_proxy = apply_opam_config_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("http-proxy: \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("https-proxy: \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("opam-version"));

        // Replace existing
        let updated = apply_opam_config_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("http-proxy: \"http://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_opam_config_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("http-proxy:"));
        assert!(!cleaned.contains("https-proxy:"));
        assert!(cleaned.contains("opam-version"));
    }

    #[test]
    fn test_apply_and_remove_lein_proxy() {
        let initial = "{:user {:plugins [[lein-ancient \"0.6.15\"]]}}\n";
        let with_proxy = apply_lein_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains(":http-proxy \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains(":https-proxy \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("lein-ancient"));

        // Replace existing
        let updated = apply_lein_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains(":http-proxy \"http://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_lein_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains(":http-proxy"));
        assert!(!cleaned.contains(":https-proxy"));
        assert!(cleaned.contains("lein-ancient"));
    }

    #[test]
    fn test_apply_and_remove_cpan_proxy() {
        let initial = "$CPAN::Config = {\n  'urllist' => [q[https://cpan.metacpan.org/]],\n};\n1;\n__END__\n";
        let with_proxy = apply_cpan_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("'http_proxy' => q[http://127.0.0.1:7890],"));
        assert!(with_proxy.contains("'ftp_proxy' => q[http://127.0.0.1:7890],"));
        assert!(with_proxy.contains("urllist"));

        // Replace existing
        let updated = apply_cpan_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("'http_proxy' => q[http://127.0.0.1:1080],"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_cpan_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("'http_proxy'"));
        assert!(!cleaned.contains("'ftp_proxy'"));
        assert!(cleaned.contains("urllist"));
    }
}
