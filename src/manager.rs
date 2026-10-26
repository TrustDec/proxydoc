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

pub fn apply_luarocks_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut set_http = false;
    let mut set_https = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with("--") {
            let lower = trim.to_lowercase();
            if lower.starts_with("http_proxy") || lower.starts_with("variables.http_proxy") {
                lines.push(format!("http_proxy = \"{}\"", proxy_url));
                set_http = true;
                continue;
            } else if lower.starts_with("https_proxy") || lower.starts_with("variables.https_proxy") {
                lines.push(format!("https_proxy = \"{}\"", proxy_url));
                set_https = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !set_http {
        lines.push(format!("http_proxy = \"{}\"", proxy_url));
    }
    if !set_https {
        lines.push(format!("https_proxy = \"{}\"", proxy_url));
    }

    let mut res = lines.join("\n");
    if content.ends_with('\n') || content.is_empty() {
        res.push('\n');
    }
    res
}

pub fn remove_luarocks_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut modified = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with("--") {
            let lower = trim.to_lowercase();
            if lower.contains("http_proxy") || lower.contains("https_proxy") {
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

pub fn set_luarocks_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_luarocks_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path).unwrap_or_default()
    } else {
        String::new()
    };
    let new_content = apply_luarocks_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_luarocks_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_luarocks_config_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_luarocks_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_nimble_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut set_http = false;
    let mut set_https = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && !trim.starts_with(';') {
            let lower = trim.to_lowercase();
            if lower.starts_with("http_proxy") {
                lines.push(format!("http_proxy = \"{}\"", proxy_url));
                set_http = true;
                continue;
            } else if lower.starts_with("https_proxy") {
                lines.push(format!("https_proxy = \"{}\"", proxy_url));
                set_https = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !set_http {
        lines.push(format!("http_proxy = \"{}\"", proxy_url));
    }
    if !set_https {
        lines.push(format!("https_proxy = \"{}\"", proxy_url));
    }

    let mut res = lines.join("\n");
    if content.ends_with('\n') || content.is_empty() {
        res.push('\n');
    }
    res
}

pub fn remove_nimble_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut modified = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && !trim.starts_with(';') {
            let lower = trim.to_lowercase();
            if lower.starts_with("http_proxy") || lower.starts_with("https_proxy") {
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

pub fn set_nimble_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_nimble_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path).unwrap_or_default()
    } else {
        String::new()
    };
    let new_content = apply_nimble_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_nimble_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_nimble_config_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_nimble_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_rebar3_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut set_http = false;
    let mut set_https = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('%') && !trim.starts_with('#') {
            if trim.contains("http_proxy,") {
                lines.push(format!("{{http_proxy, \"{}\"}}.", proxy_url));
                set_http = true;
                continue;
            } else if trim.contains("https_proxy,") {
                lines.push(format!("{{https_proxy, \"{}\"}}.", proxy_url));
                set_https = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !set_http {
        lines.push(format!("{{http_proxy, \"{}\"}}.", proxy_url));
    }
    if !set_https {
        lines.push(format!("{{https_proxy, \"{}\"}}.", proxy_url));
    }

    let mut res = lines.join("\n");
    if content.ends_with('\n') || content.is_empty() {
        res.push('\n');
    }
    res
}

pub fn remove_rebar3_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut modified = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('%') && !trim.starts_with('#') {
            if trim.contains("http_proxy,") || trim.contains("https_proxy,") {
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

pub fn set_rebar3_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_rebar3_write_path();
    if let Some(parent) = write_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path).unwrap_or_default()
    } else {
        String::new()
    };
    let new_content = apply_rebar3_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_rebar3_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_rebar3_config_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let (new_content, changed) = remove_rebar3_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_scoop_proxy(content: &str, proxy_url: &str) -> String {
    let mut val: serde_json::Value = if content.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(content).unwrap_or_else(|_| serde_json::json!({}))
    };

    if !val.is_object() {
        val = serde_json::json!({});
    }

    let obj = val.as_object_mut().unwrap();
    obj.insert(
        "proxy".to_string(),
        serde_json::Value::String(proxy_url.to_string()),
    );

    serde_json::to_string_pretty(&val).unwrap_or_else(|_| content.to_string()) + "\n"
}

pub fn remove_scoop_proxy(content: &str) -> (String, bool) {
    if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(obj) = val.as_object_mut() {
            if obj.remove("proxy").is_some() {
                let new_content = serde_json::to_string_pretty(&val)
                    .unwrap_or_else(|_| content.to_string())
                    + "\n";
                return (new_content, true);
            }
        }
    }
    (content.to_string(), false)
}

pub fn set_scoop_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_scoop_write_path();
    if let Some(parent) = write_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path).unwrap_or_default()
    } else {
        String::new()
    };
    let new_content = apply_scoop_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_scoop_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_scoop_config_path() {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let (new_content, changed) = remove_scoop_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
            }
            return Ok(changed);
        }
    }
    Ok(false)
}

pub fn apply_bundler_proxy(content: &str, proxy_url: &str) -> String {
    let mut out_lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let has_header = content.lines().any(|l| l.trim() == "---");

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_lowercase();
        if lower.starts_with("bundle_http_proxy") || lower.starts_with("http_proxy:") {
            out_lines.push(format!("BUNDLE_HTTP_PROXY: \"{}\"", proxy_url));
            replaced_http = true;
        } else if lower.starts_with("bundle_https_proxy") || lower.starts_with("https_proxy:") {
            out_lines.push(format!("BUNDLE_HTTPS_PROXY: \"{}\"", proxy_url));
            replaced_https = true;
        } else {
            out_lines.push(line.to_string());
        }
    }

    if !replaced_http {
        if out_lines.is_empty() && !has_header {
            out_lines.push("---".to_string());
        }
        out_lines.push(format!("BUNDLE_HTTP_PROXY: \"{}\"", proxy_url));
    }
    if !replaced_https {
        out_lines.push(format!("BUNDLE_HTTPS_PROXY: \"{}\"", proxy_url));
    }

    out_lines.join("\n") + "\n"
}

pub fn remove_bundler_proxy(content: &str) -> (String, bool) {
    let mut out_lines = Vec::new();
    let mut removed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_lowercase();
        if lower.starts_with("bundle_http_proxy")
            || lower.starts_with("bundle_https_proxy")
            || lower.starts_with("http_proxy:")
            || lower.starts_with("https_proxy:")
        {
            removed = true;
        } else {
            out_lines.push(line.to_string());
        }
    }

    if removed {
        let non_empty: Vec<&String> = out_lines
            .iter()
            .filter(|l| !l.trim().is_empty() && l.trim() != "---")
            .collect();
        if non_empty.is_empty() {
            return (String::new(), true);
        }
        (out_lines.join("\n") + "\n", true)
    } else {
        (content.to_string(), false)
    }
}

pub fn set_bundler_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_bundler_write_path();
    if let Some(parent) = write_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path).unwrap_or_default()
    } else {
        String::new()
    };
    let new_content = apply_bundler_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_bundler_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_bundler_config_path() {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let (new_content, changed) = remove_bundler_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
            }
            return Ok(changed);
        }
    }
    Ok(false)
}

pub fn apply_gcloud_config_proxy(content: &str, proxy_url: &str) -> String {
    let (host, port) = extract_host_port(proxy_url);
    let proxy_type = if proxy_url.starts_with("socks5") {
        "socks5"
    } else if proxy_url.starts_with("socks4") {
        "socks4"
    } else {
        "http"
    };

    let mut lines = Vec::new();
    let mut in_proxy = false;
    let mut section_found = false;
    let mut address_set = false;
    let mut port_set = false;
    let mut type_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim();
            if in_proxy {
                if !address_set {
                    lines.push(format!("address = {}", host));
                }
                if !port_set {
                    lines.push(format!("port = {}", port));
                }
                if !type_set {
                    lines.push(format!("type = {}", proxy_type));
                }
            }
            in_proxy = section.eq_ignore_ascii_case("proxy");
            if in_proxy {
                section_found = true;
            }
            lines.push(line.to_string());
            continue;
        }

        if in_proxy {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                match key.as_str() {
                    "address" | "host" => {
                        lines.push(format!("address = {}", host));
                        address_set = true;
                        continue;
                    }
                    "port" => {
                        lines.push(format!("port = {}", port));
                        port_set = true;
                        continue;
                    }
                    "type" => {
                        lines.push(format!("type = {}", proxy_type));
                        type_set = true;
                        continue;
                    }
                    _ => {}
                }
            }
        }
        lines.push(line.to_string());
    }

    if in_proxy {
        if !address_set {
            lines.push(format!("address = {}", host));
        }
        if !port_set {
            lines.push(format!("port = {}", port));
        }
        if !type_set {
            lines.push(format!("type = {}", proxy_type));
        }
    } else if !section_found {
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push(String::new());
        }
        lines.push("[proxy]".to_string());
        lines.push(format!("address = {}", host));
        lines.push(format!("port = {}", port));
        lines.push(format!("type = {}", proxy_type));
    }

    lines.join("\n") + "\n"
}

pub fn remove_gcloud_config_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut in_proxy = false;
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim();
            if section.eq_ignore_ascii_case("proxy") {
                in_proxy = true;
                changed = true;
                continue;
            } else {
                in_proxy = false;
            }
            lines.push(line.to_string());
            continue;
        }
        if in_proxy {
            changed = true;
            continue;
        }
        lines.push(line.to_string());
    }

    while lines.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
        lines.pop();
    }
    let res = if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n") + "\n"
    };
    (res, changed)
}

pub fn set_gcloud_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_gcloud_write_path();
    if let Some(parent) = write_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path).unwrap_or_default()
    } else {
        String::new()
    };
    let new_content = apply_gcloud_config_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_gcloud_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_gcloud_config_path() {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let (new_content, changed) = remove_gcloud_config_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
            }
            return Ok(changed);
        }
    }
    Ok(false)
}

pub fn apply_aws_config_proxy(content: &str, proxy_url: &str, profile: Option<&str>) -> String {
    let env_profile = std::env::var("AWS_PROFILE").unwrap_or_else(|_| "default".to_string());
    let target_profile = profile.unwrap_or(&env_profile);
    let mut lines = Vec::new();
    let mut in_target = false;
    let mut section_found = false;
    let mut proxy_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim();
            if in_target && !proxy_set {
                lines.push(format!("proxy = {}", proxy_url));
                proxy_set = true;
            }
            if target_profile == "default" {
                in_target = section.eq_ignore_ascii_case("default")
                    || section.eq_ignore_ascii_case("profile default");
            } else {
                in_target = section.eq_ignore_ascii_case(target_profile)
                    || section.eq_ignore_ascii_case(&format!("profile {}", target_profile));
            }
            if in_target {
                section_found = true;
            }
            lines.push(line.to_string());
            continue;
        }

        if in_target {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "proxy" || key == "http_proxy" {
                    lines.push(format!("proxy = {}", proxy_url));
                    proxy_set = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    if in_target && !proxy_set {
        lines.push(format!("proxy = {}", proxy_url));
    } else if !section_found {
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push(String::new());
        }
        if target_profile == "default" {
            lines.push("[default]".to_string());
        } else {
            lines.push(format!("[profile {}]", target_profile));
        }
        lines.push(format!("proxy = {}", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_aws_config_proxy(content: &str, profile: Option<&str>) -> (String, bool) {
    let env_profile = std::env::var("AWS_PROFILE").unwrap_or_else(|_| "default".to_string());
    let target_profile = profile.unwrap_or(&env_profile);
    let mut lines = Vec::new();
    let mut in_target = false;
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim();
            if target_profile == "default" {
                in_target = section.eq_ignore_ascii_case("default")
                    || section.eq_ignore_ascii_case("profile default");
            } else {
                in_target = section.eq_ignore_ascii_case(target_profile)
                    || section.eq_ignore_ascii_case(&format!("profile {}", target_profile));
            }
            lines.push(line.to_string());
            continue;
        }

        if in_target && !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "proxy" || key == "http_proxy" || key == "https_proxy" {
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

pub fn set_aws_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_aws_write_path();
    if let Some(parent) = write_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path).unwrap_or_default()
    } else {
        String::new()
    };
    let new_content = apply_aws_config_proxy(&content, proxy_url, None);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_aws_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_aws_config_path() {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let (new_content, changed) = remove_aws_config_proxy(&content, None);
            if changed {
                std::fs::write(&path, new_content)?;
            }
            return Ok(changed);
        }
    }
    Ok(false)
}

pub fn apply_azure_config_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut in_proxy = false;
    let mut section_found = false;
    let mut proxy_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim();
            if in_proxy && !proxy_set {
                lines.push("mode = manual".to_string());
                lines.push(format!("http = {}", proxy_url));
                lines.push(format!("https = {}", proxy_url));
                proxy_set = true;
            }
            in_proxy = section.eq_ignore_ascii_case("proxy");
            if in_proxy {
                section_found = true;
            }
            lines.push(line.to_string());
            continue;
        }

        if in_proxy {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "mode"
                    || key == "http"
                    || key == "https"
                    || key == "http_proxy"
                    || key == "https_proxy"
                    || key == "proxy_url"
                {
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    if in_proxy && !proxy_set {
        lines.push("mode = manual".to_string());
        lines.push(format!("http = {}", proxy_url));
        lines.push(format!("https = {}", proxy_url));
    } else if !section_found {
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push(String::new());
        }
        lines.push("[proxy]".to_string());
        lines.push("mode = manual".to_string());
        lines.push(format!("http = {}", proxy_url));
        lines.push(format!("https = {}", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_azure_config_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut in_proxy = false;
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim();
            in_proxy = section.eq_ignore_ascii_case("proxy");
            lines.push(line.to_string());
            continue;
        }

        if in_proxy && !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "mode"
                    || key == "http"
                    || key == "https"
                    || key == "http_proxy"
                    || key == "https_proxy"
                    || key == "proxy_url"
                {
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

pub fn set_azure_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_azure_write_path();
    if let Some(parent) = write_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path).unwrap_or_default()
    } else {
        String::new()
    };
    let new_content = apply_azure_config_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_azure_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_azure_config_path() {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let (new_content, changed) = remove_azure_config_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
            }
            return Ok(changed);
        }
    }
    Ok(false)
}

pub fn unset_mamba_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_mambarc_path() {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let (new_content, removed) = remove_condarc_proxy(&content);
            if removed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn set_mamba_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_mamba_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_condarc_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn apply_podman_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut in_engine = false;
    let mut engine_found = false;
    let mut in_env_array = false;
    let mut proxy_applied = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim().to_ascii_lowercase();
            if in_engine && !proxy_applied {
                lines.push(format!("env = [\"http_proxy={0}\", \"https_proxy={0}\", \"HTTP_PROXY={0}\", \"HTTPS_PROXY={0}\"]", proxy_url));
                proxy_applied = true;
            }
            in_engine = section == "engine";
            if in_engine {
                engine_found = true;
            }
            in_env_array = false;
            lines.push(line.to_string());
            continue;
        }

        if in_engine {
            if trim.starts_with("env") && trim.contains('=') {
                if trim.contains('[') && !trim.contains(']') {
                    in_env_array = true;
                }
                if !proxy_applied {
                    lines.push(format!("env = [\"http_proxy={0}\", \"https_proxy={0}\", \"HTTP_PROXY={0}\", \"HTTPS_PROXY={0}\"]", proxy_url));
                    proxy_applied = true;
                }
                continue;
            }
            if in_env_array {
                if trim.contains(']') {
                    in_env_array = false;
                }
                continue;
            }
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "http_proxy" || key == "https_proxy" || key == "all_proxy" {
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    if in_engine && !proxy_applied {
        lines.push(format!("env = [\"http_proxy={0}\", \"https_proxy={0}\", \"HTTP_PROXY={0}\", \"HTTPS_PROXY={0}\"]", proxy_url));
    } else if !engine_found {
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push(String::new());
        }
        lines.push("[engine]".to_string());
        lines.push(format!("env = [\"http_proxy={0}\", \"https_proxy={0}\", \"HTTP_PROXY={0}\", \"HTTPS_PROXY={0}\"]", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_podman_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut in_engine = false;
    let mut in_env_array = false;
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim().to_ascii_lowercase();
            in_engine = section == "engine" || section == "engine.env";
            in_env_array = false;
            lines.push(line.to_string());
            continue;
        }

        if in_engine {
            if trim.starts_with("env") && trim.contains('=') {
                if trim.contains('[') && !trim.contains(']') {
                    in_env_array = true;
                }
                let lower = trim.to_ascii_lowercase();
                if lower.contains("proxy") {
                    changed = true;
                    if trim.contains(']') {
                        continue;
                    }
                } else {
                    lines.push(line.to_string());
                }
                continue;
            }

            if in_env_array {
                let lower = trim.to_ascii_lowercase();
                if lower.contains("proxy") {
                    changed = true;
                } else if !trim.is_empty() && trim != "]" {
                    lines.push(line.to_string());
                }
                if trim.contains(']') {
                    in_env_array = false;
                }
                continue;
            }

            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "http_proxy" || key == "https_proxy" || key == "all_proxy" {
                    changed = true;
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_podman_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_containers_conf_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_podman_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_podman_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_containers_conf_path() {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let (new_content, removed) = remove_podman_proxy(&content);
            if removed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_vagrant_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut in_configure = false;
    let mut configure_found = false;
    let mut proxy_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with("Vagrant.configure") {
            in_configure = true;
            configure_found = true;
            lines.push(line.to_string());
            continue;
        }

        if in_configure {
            let lower = trim.to_ascii_lowercase();
            if lower.starts_with("config.proxy.") {
                continue;
            }
            if trim == "end" && !proxy_set {
                lines.push(format!("  if Vagrant.has_plugin?(\"vagrant-proxyconf\")\n    config.proxy.http = \"{0}\"\n    config.proxy.https = \"{0}\"\n  end", proxy_url));
                proxy_set = true;
            }
        }

        lines.push(line.to_string());
    }

    if !configure_found {
        let template = format!(
            "Vagrant.configure(\"2\") do |config|\n  if Vagrant.has_plugin?(\"vagrant-proxyconf\")\n    config.proxy.http = \"{0}\"\n    config.proxy.https = \"{0}\"\n  end\nend\n",
            proxy_url
        );
        return template;
    }

    lines.join("\n") + "\n"
}

pub fn remove_vagrant_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("config.proxy.http") || lower.starts_with("config.proxy.https") {
            changed = true;
            continue;
        }
        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_vagrant_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_vagrant_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_vagrant_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_vagrant_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_vagrant_config_path() {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let (new_content, removed) = remove_vagrant_proxy(&content);
            if removed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_choco_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut found_proxy_line = false;
    let mut in_config = false;
    let mut config_found = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("<config>") || lower.starts_with("<config ") {
            in_config = true;
            config_found = true;
            lines.push(line.to_string());
            continue;
        }
        if in_config && lower.starts_with("</config>") {
            if !found_proxy_line {
                lines.push(format!("    <add key=\"proxy\" value=\"{}\" description=\"proxy\" />", proxy_url));
                found_proxy_line = true;
            }
            in_config = false;
            lines.push(line.to_string());
            continue;
        }

        if lower.contains("<add ") && (lower.contains("key=\"proxy\"") || lower.contains("key='proxy'")) {
            let new_line = if lower.contains("value=") {
                let mut rebuilt = String::new();
                let lower_line = line.to_ascii_lowercase();
                if let Some(vpos) = lower_line.find("value=") {
                    rebuilt.push_str(&line[..vpos]);
                    let after_v = &line[vpos + 6..];
                    if let Some(quote) = after_v.chars().next() {
                        if quote == '"' || quote == '\'' {
                            let rest = &after_v[quote.len_utf8()..];
                            if let Some(close_q) = rest.find(quote) {
                                rebuilt.push_str(&format!("value=\"{}\"{}", proxy_url, &rest[close_q + quote.len_utf8()..]));
                            } else {
                                rebuilt.push_str(&format!("value=\"{}\"", proxy_url));
                            }
                        } else {
                            rebuilt.push_str(&format!("value=\"{}\"", proxy_url));
                        }
                    } else {
                        rebuilt.push_str(&format!("value=\"{}\"", proxy_url));
                    }
                } else {
                    rebuilt.push_str(&format!("    <add key=\"proxy\" value=\"{}\" description=\"proxy\" />", proxy_url));
                }
                rebuilt
            } else {
                format!("    <add key=\"proxy\" value=\"{}\" description=\"proxy\" />", proxy_url)
            };
            lines.push(new_line);
            found_proxy_line = true;
            continue;
        }

        lines.push(line.to_string());
    }

    if !config_found {
        return format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<chocolatey>\n  <config>\n    <add key=\"proxy\" value=\"{}\" description=\"proxy\" />\n  </config>\n</chocolatey>\n",
            proxy_url
        );
    }

    lines.join("\n") + "\n"
}

pub fn remove_choco_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.contains("<add ") && (lower.contains("key=\"proxy\"") || lower.contains("key='proxy'")) {
            if let Some(val) = crate::doctor::extract_xml_attr(trim, "value") {
                if !val.trim().is_empty() {
                    changed = true;
                    let mut rebuilt = String::new();
                    let lower_line = line.to_ascii_lowercase();
                    if let Some(vpos) = lower_line.find("value=") {
                        rebuilt.push_str(&line[..vpos]);
                        let after_v = &line[vpos + 6..];
                        if let Some(quote) = after_v.chars().next() {
                            if quote == '"' || quote == '\'' {
                                let rest = &after_v[quote.len_utf8()..];
                                if let Some(close_q) = rest.find(quote) {
                                    rebuilt.push_str(&format!("value=\"\"{}", &rest[close_q + quote.len_utf8()..]));
                                } else {
                                    rebuilt.push_str("value=\"\"");
                                }
                            } else {
                                rebuilt.push_str("value=\"\"");
                            }
                        } else {
                            rebuilt.push_str("value=\"\"");
                        }
                    } else {
                        rebuilt = line.to_string();
                    }
                    lines.push(rebuilt);
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_choco_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_choco_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_choco_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_choco_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_choco_config_path() {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let (new_content, removed) = remove_choco_proxy(&content);
            if removed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_colima_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut in_env = false;
    let mut env_found = false;
    let mut http_set = false;
    let mut https_set = false;
    let mut all_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim == "env:" || trim.starts_with("env:") {
            in_env = true;
            env_found = true;
            lines.push(line.to_string());
            continue;
        }

        if in_env {
            if !line.starts_with(' ') && !line.starts_with('\t') && trim.ends_with(':') {
                if !http_set {
                    lines.push(format!("  HTTP_PROXY: \"{}\"", proxy_url));
                    http_set = true;
                }
                if !https_set {
                    lines.push(format!("  HTTPS_PROXY: \"{}\"", proxy_url));
                    https_set = true;
                }
                if !all_set {
                    lines.push(format!("  ALL_PROXY: \"{}\"", proxy_url));
                    all_set = true;
                }
                in_env = false;
                lines.push(line.to_string());
                continue;
            }

            if let Some((k, _)) = trim.split_once(':') {
                let key = k.trim().to_ascii_lowercase();
                if key == "http_proxy" {
                    lines.push(format!("  HTTP_PROXY: \"{}\"", proxy_url));
                    http_set = true;
                    continue;
                }
                if key == "https_proxy" {
                    lines.push(format!("  HTTPS_PROXY: \"{}\"", proxy_url));
                    https_set = true;
                    continue;
                }
                if key == "all_proxy" {
                    lines.push(format!("  ALL_PROXY: \"{}\"", proxy_url));
                    all_set = true;
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    if in_env {
        if !http_set {
            lines.push(format!("  HTTP_PROXY: \"{}\"", proxy_url));
        }
        if !https_set {
            lines.push(format!("  HTTPS_PROXY: \"{}\"", proxy_url));
        }
        if !all_set {
            lines.push(format!("  ALL_PROXY: \"{}\"", proxy_url));
        }
    } else if !env_found {
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push(String::new());
        }
        lines.push("env:".to_string());
        lines.push(format!("  HTTP_PROXY: \"{}\"", proxy_url));
        lines.push(format!("  HTTPS_PROXY: \"{}\"", proxy_url));
        lines.push(format!("  ALL_PROXY: \"{}\"", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_colima_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut in_env = false;
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim == "env:" || trim.starts_with("env:") {
            in_env = true;
            lines.push(line.to_string());
            continue;
        }

        if in_env {
            if !line.starts_with(' ') && !line.starts_with('\t') && trim.ends_with(':') {
                in_env = false;
                lines.push(line.to_string());
                continue;
            }

            if let Some((k, _)) = trim.split_once(':') {
                let key = k.trim().to_ascii_lowercase();
                if key == "http_proxy" || key == "https_proxy" || key == "all_proxy" {
                    changed = true;
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_colima_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_colima_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_colima_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_colima_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_colima_config_path() {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let (new_content, removed) = remove_colima_proxy(&content);
            if removed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_minikube_proxy(content: &str, proxy_url: &str) -> String {
    let mut val: serde_json::Value = if content.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(content).unwrap_or_else(|_| serde_json::json!({}))
    };

    if !val.is_object() {
        val = serde_json::json!({});
    }

    let obj = val.as_object_mut().unwrap();
    obj.insert(
        "http_proxy".to_string(),
        serde_json::Value::String(proxy_url.to_string()),
    );
    obj.insert(
        "https_proxy".to_string(),
        serde_json::Value::String(proxy_url.to_string()),
    );
    if !obj.contains_key("no_proxy") {
        obj.insert(
            "no_proxy".to_string(),
            serde_json::Value::String("localhost,127.0.0.1,10.96.0.0/12,192.168.59.0/24".to_string()),
        );
    }

    serde_json::to_string_pretty(&val).unwrap_or_else(|_| content.to_string()) + "\n"
}

pub fn remove_minikube_proxy(content: &str) -> (String, bool) {
    if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(obj) = val.as_object_mut() {
            let mut changed = false;
            for key in ["http_proxy", "https_proxy", "HTTP_PROXY", "HTTPS_PROXY"] {
                if obj.remove(key).is_some() {
                    changed = true;
                }
            }
            if changed {
                let new_content = serde_json::to_string_pretty(&val)
                    .unwrap_or_else(|_| content.to_string())
                    + "\n";
                return (new_content, true);
            }
        }
    }
    (content.to_string(), false)
}

pub fn set_minikube_proxy(proxy_url: &str) -> Result<PathBuf> {
    let write_path = crate::doctor::get_minikube_write_path();
    if let Some(parent) = write_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if write_path.exists() {
        std::fs::read_to_string(&write_path).unwrap_or_default()
    } else {
        String::new()
    };
    let new_content = apply_minikube_proxy(&content, proxy_url);
    std::fs::write(&write_path, new_content)?;
    Ok(write_path)
}

pub fn unset_minikube_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_minikube_config_path() {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let (new_content, changed) = remove_minikube_proxy(&content);
            if changed {
                std::fs::write(&path, new_content)?;
            }
            return Ok(changed);
        }
    }
    Ok(false)
}

pub fn apply_spack_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut in_config = false;
    let mut config_found = false;
    let mut proxy_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim == "config:" || trim.starts_with("config:") {
            in_config = true;
            config_found = true;
            lines.push(line.to_string());
            continue;
        }

        if in_config {
            if !line.starts_with(' ') && !line.starts_with('\t') && trim.ends_with(':') {
                if !proxy_set {
                    lines.push(format!("  proxy: \"{}\"", proxy_url));
                    proxy_set = true;
                }
                in_config = false;
                lines.push(line.to_string());
                continue;
            }

            if let Some((k, _)) = trim.split_once(':') {
                let key = k.trim().to_ascii_lowercase();
                if key == "proxy" || key == "http_proxy" || key == "https_proxy" || key == "all_proxy" {
                    lines.push(format!("  proxy: \"{}\"", proxy_url));
                    proxy_set = true;
                    continue;
                }
            }
        } else if let Some((k, _)) = trim.split_once(':') {
            let key = k.trim().to_ascii_lowercase();
            if key == "proxy" || key == "http_proxy" || key == "https_proxy" || key == "all_proxy" {
                lines.push(format!("proxy: \"{}\"", proxy_url));
                proxy_set = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if in_config && !proxy_set {
        lines.push(format!("  proxy: \"{}\"", proxy_url));
        proxy_set = true;
    }

    if !config_found && !proxy_set {
        return format!("config:\n  proxy: \"{}\"\n", proxy_url);
    }

    lines.join("\n") + "\n"
}

pub fn remove_spack_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            lines.push(line.to_string());
            continue;
        }

        if let Some((k, _)) = trim.split_once(':') {
            let key = k.trim().to_ascii_lowercase();
            if key == "proxy" || key == "http_proxy" || key == "https_proxy" || key == "all_proxy" {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_spack_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_spack_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_spack_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_spack_proxy() -> Result<bool> {
    if let Some(path) = crate::doctor::get_spack_config_path() {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let (new_content, removed) = remove_spack_proxy(&content);
            if removed {
                std::fs::write(&path, new_content)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn apply_macports_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut http_set = false;
    let mut https_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            lines.push(line.to_string());
            continue;
        }
        let parts: Vec<&str> = trim.split_whitespace().collect();
        if !parts.is_empty() {
            let key = parts[0].to_ascii_lowercase();
            if key == "proxy_http" {
                lines.push(format!("proxy_http              {}", proxy_url));
                http_set = true;
                continue;
            } else if key == "proxy_https" {
                lines.push(format!("proxy_https             {}", proxy_url));
                https_set = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !http_set {
        lines.push(format!("proxy_http              {}", proxy_url));
    }
    if !https_set {
        lines.push(format!("proxy_https             {}", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_macports_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            lines.push(line.to_string());
            continue;
        }
        let parts: Vec<&str> = trim.split_whitespace().collect();
        if !parts.is_empty() {
            let key = parts[0].to_ascii_lowercase();
            if key == "proxy_http" || key == "proxy_https" || key == "proxy_ftp" {
                changed = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_macports_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_macports_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_macports_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_macports_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_macports_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_macports_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    if !any_removed {
        if let Some(path) = crate::doctor::get_macports_config_path() {
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                let (new_content, removed) = remove_macports_proxy(&content);
                if removed {
                    std::fs::write(&path, new_content)?;
                    return Ok(true);
                }
            }
        }
    }
    Ok(any_removed)
}

pub fn apply_apt_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut http_set = false;
    let mut https_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with("//") || trim.starts_with('#') || trim.starts_with("/*") {
            lines.push(line.to_string());
            continue;
        }
        let lower = trim.to_ascii_lowercase();
        if lower.contains("acquire::http::proxy") {
            lines.push(format!("Acquire::http::Proxy \"{}\";", proxy_url));
            http_set = true;
            continue;
        } else if lower.contains("acquire::https::proxy") {
            lines.push(format!("Acquire::https::Proxy \"{}\";", proxy_url));
            https_set = true;
            continue;
        }
        lines.push(line.to_string());
    }

    if !http_set {
        lines.push(format!("Acquire::http::Proxy \"{}\";", proxy_url));
    }
    if !https_set {
        lines.push(format!("Acquire::https::Proxy \"{}\";", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_apt_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with("//") || trim.starts_with('#') || trim.starts_with("/*") {
            lines.push(line.to_string());
            continue;
        }
        let lower = trim.to_ascii_lowercase();
        if lower.contains("acquire::http::proxy")
            || lower.contains("acquire::https::proxy")
            || lower.contains("acquire::socks::proxy")
        {
            changed = true;
            continue;
        }
        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_apt_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_apt_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_apt_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_apt_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_apt_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_apt_proxy(&content);
                if removed {
                    if new_content.trim().is_empty()
                        && path.file_name().map_or(false, |n| n == "99proxydoc.conf")
                    {
                        let _ = std::fs::remove_file(&path);
                    } else {
                        let _ = std::fs::write(&path, new_content);
                    }
                    any_removed = true;
                }
            }
        }
    }
    Ok(any_removed)
}

pub fn apply_dnf_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut in_main = false;
    let mut proxy_set = false;
    let mut main_found = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim().to_ascii_lowercase();
            if in_main && !proxy_set {
                lines.push(format!("proxy={}", proxy_url));
                proxy_set = true;
            }
            in_main = section == "main";
            if in_main {
                main_found = true;
            }
            lines.push(line.to_string());
            continue;
        }

        if in_main && !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                if k.trim().eq_ignore_ascii_case("proxy") {
                    lines.push(format!("proxy={}", proxy_url));
                    proxy_set = true;
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    if in_main && !proxy_set {
        lines.push(format!("proxy={}", proxy_url));
    }

    if !main_found {
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push(String::new());
        }
        lines.push("[main]".to_string());
        lines.push(format!("proxy={}", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_dnf_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "proxy"
                    || key == "proxy_username"
                    || key == "proxy_password"
                    || key == "proxy_auth_method"
                {
                    changed = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_dnf_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_dnf_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_dnf_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_dnf_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_dnf_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_dnf_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    Ok(any_removed)
}

pub fn apply_zypper_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut proxy_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "proxy"
                    || key == "proxy.http"
                    || key == "proxy.https"
                    || key == "http_proxy"
                    || key == "https_proxy"
                {
                    lines.push(format!("proxy = {}", proxy_url));
                    proxy_set = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    if !proxy_set {
        lines.push(format!("proxy = {}", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_zypper_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "proxy"
                    || key == "proxy.http"
                    || key == "proxy.https"
                    || key == "http_proxy"
                    || key == "https_proxy"
                    || key == "proxy_password"
                    || key == "proxy_user"
                {
                    changed = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_zypper_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_zypper_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_zypper_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_zypper_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_zypper_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_zypper_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    Ok(any_removed)
}

pub fn apply_pacman_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut in_options = false;
    let mut options_found = false;
    let mut xfer_updated = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim().to_ascii_lowercase();
            if in_options && !xfer_updated {
                lines.push(format!("XferCommand = /usr/bin/curl -g -L -C - --fail --retry 3 --proxy {} -o %o %u", proxy_url));
                xfer_updated = true;
            }
            in_options = section == "options";
            if in_options {
                options_found = true;
            }
            lines.push(line.to_string());
            continue;
        }

        if in_options && !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                if k.trim().eq_ignore_ascii_case("xfercommand") {
                    lines.push(format!("XferCommand = /usr/bin/curl -g -L -C - --fail --retry 3 --proxy {} -o %o %u", proxy_url));
                    xfer_updated = true;
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    if in_options && !xfer_updated {
        lines.push(format!("XferCommand = /usr/bin/curl -g -L -C - --fail --retry 3 --proxy {} -o %o %u", proxy_url));
    }

    if !options_found {
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push(String::new());
        }
        lines.push("[options]".to_string());
        lines.push(format!("XferCommand = /usr/bin/curl -g -L -C - --fail --retry 3 --proxy {} -o %o %u", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_pacman_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, v)) = trim.split_once('=') {
                if k.trim().eq_ignore_ascii_case("xfercommand") {
                    let cmd = v.trim();
                    if cmd.contains("--proxy") || cmd.contains("-x ") || cmd.contains("-x=") || cmd.contains("http_proxy=") {
                        if cmd.contains("--retry 3") || cmd.starts_with("/usr/bin/curl") {
                            lines.push(format!("#XferCommand = {}", cmd));
                        }
                        changed = true;
                        continue;
                    }
                }
            }
        }
        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_pacman_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_pacman_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_pacman_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_pacman_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_pacman_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_pacman_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    Ok(any_removed)
}

pub fn apply_pkg_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut proxy_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && !trim.starts_with(';') {
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
            if let Some((k, _)) = split_opt {
                let key = k.trim().to_ascii_lowercase();
                if key == "http_proxy" || key == "https_proxy" || key == "pkg_proxy" || key == "proxy" {
                    lines.push(format!("http_proxy: \"{}\";", proxy_url));
                    proxy_set = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    if !proxy_set {
        lines.push(format!("http_proxy: \"{}\";", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_pkg_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && !trim.starts_with(';') {
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
            if let Some((k, _)) = split_opt {
                let key = k.trim().to_ascii_lowercase();
                if key == "http_proxy" || key == "https_proxy" || key == "pkg_proxy" || key == "proxy" {
                    changed = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_pkg_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_pkg_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_pkg_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_pkg_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_pkg_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_pkg_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    Ok(any_removed)
}

pub fn apply_portage_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut http_set = false;
    let mut https_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "http_proxy" {
                    lines.push(format!("http_proxy=\"{}\"", proxy_url));
                    http_set = true;
                    continue;
                } else if key == "https_proxy" {
                    lines.push(format!("https_proxy=\"{}\"", proxy_url));
                    https_set = true;
                    continue;
                } else if key == "all_proxy" || key == "ftp_proxy" {
                    lines.push(format!("{}=\"{}\"", k.trim(), proxy_url));
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    if !http_set {
        lines.push(format!("http_proxy=\"{}\"", proxy_url));
    }
    if !https_set {
        lines.push(format!("https_proxy=\"{}\"", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_portage_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "http_proxy" || key == "https_proxy" || key == "all_proxy" || key == "ftp_proxy" {
                    changed = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_portage_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_portage_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_portage_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_portage_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_portage_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_portage_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    Ok(any_removed)
}

pub fn apply_docker_daemon_proxy_json(root: &mut serde_json::Value, proxy_url: &str) {
    if !root.is_object() {
        *root = serde_json::json!({});
    }

    let obj = root.as_object_mut().unwrap();
    let proxies_val = obj.entry("proxies").or_insert_with(|| serde_json::json!({}));
    if !proxies_val.is_object() {
        *proxies_val = serde_json::json!({});
    }

    let proxies_map = proxies_val.as_object_mut().unwrap();
    proxies_map.insert(
        "http-proxy".to_string(),
        serde_json::Value::String(proxy_url.to_string()),
    );
    proxies_map.insert(
        "https-proxy".to_string(),
        serde_json::Value::String(proxy_url.to_string()),
    );
    if !proxies_map.contains_key("no-proxy") {
        proxies_map.insert(
            "no-proxy".to_string(),
            serde_json::Value::String("localhost,127.0.0.1,::1".to_string()),
        );
    }
}

pub fn set_docker_daemon_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_docker_daemon_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut val: serde_json::Value = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    apply_docker_daemon_proxy_json(&mut val, proxy_url);
    let pretty = serde_json::to_string_pretty(&val)?;
    std::fs::write(&path, pretty + "\n")?;
    Ok(path)
}

pub fn remove_docker_daemon_proxy_json(root: &mut serde_json::Value) -> bool {
    let mut modified = false;
    if let Some(obj) = root.as_object_mut() {
        if let Some(proxies) = obj.get_mut("proxies").and_then(|p| p.as_object_mut()) {
            if proxies.remove("http-proxy").is_some() {
                modified = true;
            }
            if proxies.remove("https-proxy").is_some() {
                modified = true;
            }
            if proxies.remove("httpProxy").is_some() {
                modified = true;
            }
            if proxies.remove("httpsProxy").is_some() {
                modified = true;
            }
            if proxies.remove("no-proxy").is_some() {
                modified = true;
            }
            if proxies.remove("noProxy").is_some() {
                modified = true;
            }
        }
        if let Some(proxies) = obj.get("proxies") {
            if proxies.as_object().map_or(false, |m| m.is_empty()) {
                obj.remove("proxies");
                modified = true;
            }
        }
        for key in ["http-proxy", "https-proxy", "httpProxy", "httpsProxy"] {
            if obj.remove(key).is_some() {
                modified = true;
            }
        }
    }
    modified
}

pub fn unset_docker_daemon_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_docker_daemon_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if remove_docker_daemon_proxy_json(&mut val) {
                        if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                            let _ = std::fs::write(&path, pretty + "\n");
                            any_removed = true;
                        }
                    }
                }
            }
        }
    }
    Ok(any_removed)
}

pub fn apply_flatpak_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut in_core = false;
    let mut core_found = false;
    let mut proxy_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let section = trim[1..trim.len() - 1].trim().to_ascii_lowercase();
            if in_core && !proxy_set {
                lines.push(format!("proxy = {}", proxy_url));
                proxy_set = true;
            }
            in_core = section == "core";
            if in_core {
                core_found = true;
            }
            lines.push(line.to_string());
            continue;
        }

        if in_core && !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "proxy" || key == "http-proxy" || key == "http_proxy" {
                    lines.push(format!("proxy = {}", proxy_url));
                    proxy_set = true;
                    continue;
                }
            }
        }

        // Also update any proxy setting inside remote sections to avoid conflict
        if !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "proxy" || key == "http-proxy" || key == "http_proxy" {
                    lines.push(format!("proxy = {}", proxy_url));
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    if in_core && !proxy_set {
        lines.push(format!("proxy = {}", proxy_url));
    }

    if !core_found {
        let mut new_lines = vec![
            "[core]".to_string(),
            "repo_version = 1".to_string(),
            "mode = bare-user-only".to_string(),
            format!("proxy = {}", proxy_url),
            String::new(),
        ];
        new_lines.extend(lines);
        return new_lines.join("\n").trim_end().to_string() + "\n";
    }

    lines.join("\n") + "\n"
}

pub fn remove_flatpak_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && !trim.starts_with(';') {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_ascii_lowercase();
                if key == "proxy" || key == "http-proxy" || key == "http_proxy" {
                    changed = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_flatpak_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_flatpak_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_flatpak_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_flatpak_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_flatpak_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_flatpak_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    Ok(any_removed)
}

pub fn apply_snap_proxy(content: &str, proxy_url: &str) -> String {
    let trim_content = content.trim();
    if trim_content.is_empty() {
        return format!(
            "[Service]\nEnvironment=\"HTTP_PROXY={}\"\nEnvironment=\"HTTPS_PROXY={}\"\n",
            proxy_url, proxy_url
        );
    }

    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_system_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[system]"));

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..].trim().trim_matches('"').trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" {
                    lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    replaced_http = true;
                    continue;
                } else if k_low == "https_proxy" {
                    lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                    replaced_https = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "proxy.http" || key == "proxy_http" || key == "http_proxy" {
                lines.push(format!("{} = {}", k.trim(), proxy_url));
                replaced_http = true;
                continue;
            } else if key == "proxy.https" || key == "proxy_https" || key == "https_proxy" {
                lines.push(format!("{} = {}", k.trim(), proxy_url));
                replaced_https = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_http {
        if has_service_section {
            let mut new_lines = Vec::new();
            for l in &lines {
                new_lines.push(l.clone());
                if l.trim().eq_ignore_ascii_case("[service]") {
                    new_lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    if !replaced_https {
                        new_lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                    }
                }
            }
            lines = new_lines;
        } else if has_system_section {
            let mut new_lines = Vec::new();
            for l in &lines {
                new_lines.push(l.clone());
                if l.trim().eq_ignore_ascii_case("[system]") {
                    new_lines.push(format!("proxy.http = {}", proxy_url));
                    if !replaced_https {
                        new_lines.push(format!("proxy.https = {}", proxy_url));
                        replaced_https = true;
                    }
                }
            }
            lines = new_lines;
        } else {
            lines.push(format!("http_proxy={}", proxy_url));
            if !replaced_https {
                lines.push(format!("https_proxy={}", proxy_url));
                replaced_https = true;
            }
        }
    }

    if !replaced_https {
        if has_service_section {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        } else if has_system_section {
            lines.push(format!("proxy.https = {}", proxy_url));
        } else {
            lines.push(format!("https_proxy={}", proxy_url));
        }
    }

    lines.join("\n") + "\n"
}

pub fn remove_snap_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..].trim().trim_matches('"').trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" || k_low == "https_proxy" || k_low == "all_proxy" {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
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
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_snap_proxy(proxy_url: &str) -> Result<PathBuf> {
    let _ = Command::new("snap")
        .args([
            "set",
            "system",
            &format!("proxy.http={}", proxy_url),
            &format!("proxy.https={}", proxy_url),
        ])
        .status();

    let path = crate::doctor::get_snap_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_snap_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;
    Ok(path)
}

pub fn unset_snap_proxy() -> Result<bool> {
    let mut any_removed = false;
    if let Ok(st) = Command::new("snap")
        .args(["unset", "system", "proxy.http", "proxy.https"])
        .status()
    {
        if st.success() {
            any_removed = true;
        }
    }
    for path in crate::doctor::get_snap_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_snap_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    Ok(any_removed)
}

pub fn apply_ollama_proxy(content: &str, proxy_url: &str) -> String {
    let trim_content = content.trim();
    if trim_content.is_empty() {
        return format!(
            "[Service]\nEnvironment=\"HTTP_PROXY={}\"\nEnvironment=\"HTTPS_PROXY={}\"\n",
            proxy_url, proxy_url
        );
    }

    // Handle macOS LaunchAgent plist XML format
    if content.contains("<plist") {
        if content.contains("<key>EnvironmentVariables</key>") {
            let mut lines = Vec::new();
            let mut in_env_dict = false;
            let mut replaced_http = false;
            let mut replaced_https = false;
            for line in content.lines() {
                let lower = line.trim().to_ascii_lowercase();
                if lower.contains("<key>environmentvariables</key>") {
                    in_env_dict = true;
                    lines.push(line.to_string());
                    continue;
                }
                if in_env_dict && lower.contains("</dict>") {
                    if !replaced_http {
                        lines.push(format!("        <key>HTTP_PROXY</key>\n        <string>{}</string>", proxy_url));
                    }
                    if !replaced_https {
                        lines.push(format!("        <key>HTTPS_PROXY</key>\n        <string>{}</string>", proxy_url));
                    }
                    in_env_dict = false;
                    lines.push(line.to_string());
                    continue;
                }
                if in_env_dict && (lower.contains("<key>http_proxy</key>") || lower.contains("<key>https_proxy</key>")) {
                    if lower.contains("<key>http_proxy</key>") {
                        lines.push(format!("        <key>HTTP_PROXY</key>\n        <string>{}</string>", proxy_url));
                        replaced_http = true;
                    } else {
                        lines.push(format!("        <key>HTTPS_PROXY</key>\n        <string>{}</string>", proxy_url));
                        replaced_https = true;
                    }
                    continue;
                }
                lines.push(line.to_string());
            }
            return lines.join("\n") + "\n";
        } else if let Some(dict_pos) = content.find("<dict>") {
            let (before, after) = content.split_at(dict_pos + 6);
            return format!(
                "{}\n    <key>EnvironmentVariables</key>\n    <dict>\n        <key>HTTP_PROXY</key>\n        <string>{}</string>\n        <key>HTTPS_PROXY</key>\n        <string>{}</string>\n    </dict>{}",
                before, proxy_url, proxy_url, after
            );
        }
    }

    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" {
                    lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    replaced_http = true;
                    continue;
                } else if k_low == "https_proxy" {
                    lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                    replaced_https = true;
                    continue;
                } else if k_low == "all_proxy" || k_low == "ollama_proxy" {
                    lines.push(format!("Environment=\"OLLAMA_PROXY={}\"", proxy_url));
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy" {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if key == "https_proxy" {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if key == "all_proxy" || key == "ollama_proxy" {
                lines.push(format!("OLLAMA_PROXY=\"{}\"", proxy_url));
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_http {
        if has_service_section {
            let mut new_lines = Vec::new();
            for l in &lines {
                new_lines.push(l.clone());
                if l.trim().eq_ignore_ascii_case("[service]") {
                    new_lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    if !replaced_https {
                        new_lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                    }
                }
            }
            lines = new_lines;
        } else {
            lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
            if !replaced_https {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
            }
        }
    }

    if !replaced_https {
        if has_service_section {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
        }
    }

    lines.join("\n") + "\n"
}

pub fn remove_ollama_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "ollama_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "ollama_proxy"
                || key == "proxy"
            {
                changed = true;
                continue;
            }
        }

        if lower.contains("<key>http_proxy</key>")
            || lower.contains("<key>https_proxy</key>")
            || lower.contains("<key>ollama_proxy</key>")
        {
            changed = true;
            continue;
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_ollama_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_ollama_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_ollama_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_ollama_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_ollama_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_ollama_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_containerd_proxy(content: &str, proxy_url: &str) -> String {
    let trim_content = content.trim();
    let no_proxy_def = "localhost,127.0.0.1,docker.internal,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,.svc,.cluster.local";
    if trim_content.is_empty() {
        return format!(
            "[Service]\nEnvironment=\"HTTP_PROXY={}\"\nEnvironment=\"HTTPS_PROXY={}\"\nEnvironment=\"NO_PROXY={}\"\n",
            proxy_url, proxy_url, no_proxy_def
        );
    }

    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let mut replaced_no = false;
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" {
                    lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    replaced_http = true;
                    continue;
                } else if k_low == "https_proxy" {
                    lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                    replaced_https = true;
                    continue;
                } else if k_low == "all_proxy" || k_low == "containerd_http_proxy" {
                    lines.push(format!("Environment=\"CONTAINERD_HTTP_PROXY={}\"", proxy_url));
                    continue;
                } else if k_low == "no_proxy" {
                    lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                    replaced_no = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy" {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if key == "https_proxy" {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if key == "all_proxy" || key == "containerd_http_proxy" {
                lines.push(format!("CONTAINERD_HTTP_PROXY=\"{}\"", proxy_url));
                continue;
            } else if key == "no_proxy" {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_http {
        if has_service_section {
            let mut new_lines = Vec::new();
            for l in &lines {
                new_lines.push(l.clone());
                if l.trim().eq_ignore_ascii_case("[service]") {
                    new_lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    if !replaced_https {
                        new_lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                    }
                    if !replaced_no {
                        new_lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                    }
                }
            }
            lines = new_lines;
        } else {
            lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
            if !replaced_https {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
            }
            if !replaced_no {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
            }
        }
    }

    if !replaced_https {
        if has_service_section {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_no {
        if has_service_section {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        } else {
            lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
        }
    }

    lines.join("\n") + "\n"
}

pub fn remove_containerd_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "containerd_http_proxy"
                    || k_low == "containerd_https_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "containerd_http_proxy"
                || key == "containerd_https_proxy"
                || key == "no_proxy"
                || key == "proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_containerd_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_containerd_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_containerd_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_containerd_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_containerd_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_containerd_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_buildkit_proxy(content: &str, proxy_url: &str) -> String {
    let trim_content = content.trim();
    let no_proxy_def = "localhost,127.0.0.1,docker.internal,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16";
    if trim_content.is_empty() {
        return format!(
            "[Service]\nEnvironment=\"HTTP_PROXY={}\"\nEnvironment=\"HTTPS_PROXY={}\"\nEnvironment=\"NO_PROXY={}\"\n",
            proxy_url, proxy_url, no_proxy_def
        );
    }

    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let mut replaced_no = false;
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" {
                    lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    replaced_http = true;
                    continue;
                } else if k_low == "https_proxy" {
                    lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                    replaced_https = true;
                    continue;
                } else if k_low == "all_proxy" || k_low == "buildkit_http_proxy" {
                    lines.push(format!("Environment=\"BUILDKIT_HTTP_PROXY={}\"", proxy_url));
                    continue;
                } else if k_low == "no_proxy" {
                    lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                    replaced_no = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy" {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if key == "https_proxy" {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if key == "all_proxy" || key == "buildkit_http_proxy" {
                lines.push(format!("BUILDKIT_HTTP_PROXY=\"{}\"", proxy_url));
                continue;
            } else if key == "no_proxy" {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_http {
        if has_service_section {
            let mut new_lines = Vec::new();
            for l in &lines {
                new_lines.push(l.clone());
                if l.trim().eq_ignore_ascii_case("[service]") {
                    new_lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    if !replaced_https {
                        new_lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                    }
                    if !replaced_no {
                        new_lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                    }
                }
            }
            lines = new_lines;
        } else {
            lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
            if !replaced_https {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
            }
            if !replaced_no {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
            }
        }
    }

    if !replaced_https {
        if has_service_section {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_no {
        if has_service_section {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        } else {
            lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
        }
    }

    lines.join("\n") + "\n"
}

pub fn remove_buildkit_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "buildkit_http_proxy"
                    || k_low == "buildkit_https_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "buildkit_http_proxy"
                || key == "buildkit_https_proxy"
                || key == "no_proxy"
                || key == "http-proxy"
                || key == "https-proxy"
                || key == "proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_buildkit_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_buildkit_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_buildkit_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_buildkit_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_buildkit_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_buildkit_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_k3s_proxy(content: &str, proxy_url: &str) -> String {
    let trim_content = content.trim();
    let no_proxy_def = "127.0.0.0/8,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,.svc,.cluster.local,localhost";
    if trim_content.is_empty() {
        return format!(
            "HTTP_PROXY=\"{}\"\nHTTPS_PROXY=\"{}\"\nNO_PROXY=\"{}\"\n",
            proxy_url, proxy_url, no_proxy_def
        );
    }

    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let mut replaced_no = false;
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" {
                    lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    replaced_http = true;
                    continue;
                } else if k_low == "https_proxy" {
                    lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                    replaced_https = true;
                    continue;
                } else if k_low == "all_proxy" || k_low == "k3s_http_proxy" || k_low == "containerd_http_proxy" {
                    lines.push(format!("Environment=\"CONTAINERD_HTTP_PROXY={}\"", proxy_url));
                    continue;
                } else if k_low == "no_proxy" {
                    lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                    replaced_no = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy" {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if key == "https_proxy" {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if key == "all_proxy" || key == "k3s_http_proxy" || key == "containerd_http_proxy" {
                lines.push(format!("CONTAINERD_HTTP_PROXY=\"{}\"", proxy_url));
                continue;
            } else if key == "no_proxy" {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_http {
        if has_service_section {
            let mut new_lines = Vec::new();
            for l in &lines {
                new_lines.push(l.clone());
                if l.trim().eq_ignore_ascii_case("[service]") {
                    new_lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    if !replaced_https {
                        new_lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                    }
                    if !replaced_no {
                        new_lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                    }
                }
            }
            lines = new_lines;
        } else {
            lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
            if !replaced_https {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
            }
            if !replaced_no {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
            }
        }
    }

    if !replaced_https {
        if has_service_section {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_no {
        if has_service_section {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        } else {
            lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
        }
    }

    lines.join("\n") + "\n"
}

pub fn remove_k3s_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "k3s_http_proxy"
                    || k_low == "k3s_https_proxy"
                    || k_low == "containerd_http_proxy"
                    || k_low == "containerd_https_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "k3s_http_proxy"
                || key == "k3s_https_proxy"
                || key == "containerd_http_proxy"
                || key == "containerd_https_proxy"
                || key == "no_proxy"
                || key == "proxy"
            {
                changed = true;
                continue;
            }
        }

        if let Some((k, _)) = trim.split_once(':') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http-proxy" || key == "https-proxy" || key == "http_proxy" || key == "https_proxy" {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_k3s_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_k3s_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_k3s_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_k3s_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_k3s_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_k3s_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_microk8s_proxy(content: &str, proxy_url: &str) -> String {
    let trim_content = content.trim();
    let no_proxy_def = "10.1.0.0/16,10.152.183.0/24,127.0.0.1,127.0.0.0/8,localhost,::1,.svc,.cluster.local";
    if trim_content.is_empty() {
        return format!(
            "HTTP_PROXY={}\nHTTPS_PROXY={}\nNO_PROXY={}\n",
            proxy_url, proxy_url, no_proxy_def
        );
    }

    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let mut replaced_no = false;
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" {
                    lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    replaced_http = true;
                    continue;
                } else if k_low == "https_proxy" {
                    lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                    replaced_https = true;
                    continue;
                } else if k_low == "all_proxy" || k_low == "microk8s_http_proxy" {
                    lines.push(format!("Environment=\"MICROK8S_HTTP_PROXY={}\"", proxy_url));
                    continue;
                } else if k_low == "no_proxy" {
                    lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                    replaced_no = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy" {
                lines.push(format!("HTTP_PROXY={}", proxy_url));
                replaced_http = true;
                continue;
            } else if key == "https_proxy" {
                lines.push(format!("HTTPS_PROXY={}", proxy_url));
                replaced_https = true;
                continue;
            } else if key == "all_proxy" || key == "microk8s_http_proxy" {
                lines.push(format!("MICROK8S_HTTP_PROXY={}", proxy_url));
                continue;
            } else if key == "no_proxy" {
                lines.push(format!("NO_PROXY={}", no_proxy_def));
                replaced_no = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_http {
        if has_service_section {
            let mut new_lines = Vec::new();
            for l in &lines {
                new_lines.push(l.clone());
                if l.trim().eq_ignore_ascii_case("[service]") {
                    new_lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    if !replaced_https {
                        new_lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                    }
                    if !replaced_no {
                        new_lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                    }
                }
            }
            lines = new_lines;
        } else {
            lines.push(format!("HTTP_PROXY={}", proxy_url));
            if !replaced_https {
                lines.push(format!("HTTPS_PROXY={}", proxy_url));
                replaced_https = true;
            }
            if !replaced_no {
                lines.push(format!("NO_PROXY={}", no_proxy_def));
                replaced_no = true;
            }
        }
    }

    if !replaced_https {
        if has_service_section {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTPS_PROXY={}", proxy_url));
        }
    }

    if !replaced_no {
        if has_service_section {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        } else {
            lines.push(format!("NO_PROXY={}", no_proxy_def));
        }
    }

    lines.join("\n") + "\n"
}

pub fn remove_microk8s_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "microk8s_http_proxy"
                    || k_low == "microk8s_https_proxy"
                    || k_low == "containerd_http_proxy"
                    || k_low == "containerd_https_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "microk8s_http_proxy"
                || key == "microk8s_https_proxy"
                || key == "containerd_http_proxy"
                || key == "containerd_https_proxy"
                || key == "no_proxy"
                || key == "proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_microk8s_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_microk8s_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_microk8s_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_microk8s_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_microk8s_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_microk8s_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_k0s_proxy(content: &str, proxy_url: &str) -> String {
    let trim_content = content.trim();
    let no_proxy_def = "10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,127.0.0.0/8,localhost,::1,.svc,.cluster.local";
    if trim_content.is_empty() {
        return format!(
            "[Service]\nEnvironment=\"HTTP_PROXY={}\"\nEnvironment=\"HTTPS_PROXY={}\"\nEnvironment=\"NO_PROXY={}\"\n",
            proxy_url, proxy_url, no_proxy_def
        );
    }

    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let mut replaced_no = false;
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" {
                    lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    replaced_http = true;
                    continue;
                } else if k_low == "https_proxy" {
                    lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                    replaced_https = true;
                    continue;
                } else if k_low == "all_proxy" || k_low == "k0s_http_proxy" {
                    lines.push(format!("Environment=\"K0S_HTTP_PROXY={}\"", proxy_url));
                    continue;
                } else if k_low == "no_proxy" {
                    lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                    replaced_no = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy" {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if key == "https_proxy" {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if key == "all_proxy" || key == "k0s_http_proxy" {
                lines.push(format!("K0S_HTTP_PROXY=\"{}\"", proxy_url));
                continue;
            } else if key == "no_proxy" {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_http {
        if has_service_section {
            let mut new_lines = Vec::new();
            for l in &lines {
                new_lines.push(l.clone());
                if l.trim().eq_ignore_ascii_case("[service]") {
                    new_lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    if !replaced_https {
                        new_lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                    }
                    if !replaced_no {
                        new_lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                    }
                }
            }
            lines = new_lines;
        } else {
            lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
            if !replaced_https {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
            }
            if !replaced_no {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
            }
        }
    }

    if !replaced_https {
        if has_service_section {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_no {
        if has_service_section {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        } else {
            lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
        }
    }

    lines.join("\n") + "\n"
}

pub fn remove_k0s_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "k0s_http_proxy"
                    || k_low == "k0s_https_proxy"
                    || k_low == "containerd_http_proxy"
                    || k_low == "containerd_https_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "k0s_http_proxy"
                || key == "k0s_https_proxy"
                || key == "containerd_http_proxy"
                || key == "containerd_https_proxy"
                || key == "no_proxy"
                || key == "proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_k0s_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_k0s_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_k0s_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_k0s_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_k0s_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_k0s_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_rke2_proxy(content: &str, proxy_url: &str) -> String {
    let trim_content = content.trim();
    let no_proxy_def = "10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,127.0.0.0/8,localhost,::1,.svc,.cluster.local";
    if trim_content.is_empty() {
        return format!(
            "[Service]\nEnvironment=\"HTTP_PROXY={}\"\nEnvironment=\"HTTPS_PROXY={}\"\nEnvironment=\"CONTAINERD_HTTP_PROXY={}\"\nEnvironment=\"CONTAINERD_HTTPS_PROXY={}\"\nEnvironment=\"NO_PROXY={}\"\n",
            proxy_url, proxy_url, proxy_url, proxy_url, no_proxy_def
        );
    }

    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let mut replaced_c_http = false;
    let mut replaced_c_https = false;
    let mut replaced_no = false;
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" {
                    lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    replaced_http = true;
                    continue;
                } else if k_low == "https_proxy" {
                    lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                    replaced_https = true;
                    continue;
                } else if k_low == "containerd_http_proxy" {
                    lines.push(format!("Environment=\"CONTAINERD_HTTP_PROXY={}\"", proxy_url));
                    replaced_c_http = true;
                    continue;
                } else if k_low == "containerd_https_proxy" {
                    lines.push(format!("Environment=\"CONTAINERD_HTTPS_PROXY={}\"", proxy_url));
                    replaced_c_https = true;
                    continue;
                } else if k_low == "all_proxy" || k_low == "rke2_http_proxy" {
                    lines.push(format!("Environment=\"RKE2_HTTP_PROXY={}\"", proxy_url));
                    continue;
                } else if k_low == "no_proxy" {
                    lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                    replaced_no = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy" {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if key == "https_proxy" {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if key == "containerd_http_proxy" {
                lines.push(format!("CONTAINERD_HTTP_PROXY=\"{}\"", proxy_url));
                replaced_c_http = true;
                continue;
            } else if key == "containerd_https_proxy" {
                lines.push(format!("CONTAINERD_HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_c_https = true;
                continue;
            } else if key == "all_proxy" || key == "rke2_http_proxy" {
                lines.push(format!("RKE2_HTTP_PROXY=\"{}\"", proxy_url));
                continue;
            } else if key == "no_proxy" {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_http {
        if has_service_section {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_https {
        if has_service_section {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_c_http {
        if has_service_section {
            lines.push(format!("Environment=\"CONTAINERD_HTTP_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("CONTAINERD_HTTP_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_c_https {
        if has_service_section {
            lines.push(format!("Environment=\"CONTAINERD_HTTPS_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("CONTAINERD_HTTPS_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_no {
        if has_service_section {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        } else {
            lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
        }
    }

    lines.join("\n") + "\n"
}

pub fn remove_rke2_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "rke2_http_proxy"
                    || k_low == "rke2_https_proxy"
                    || k_low == "containerd_http_proxy"
                    || k_low == "containerd_https_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "rke2_http_proxy"
                || key == "rke2_https_proxy"
                || key == "containerd_http_proxy"
                || key == "containerd_https_proxy"
                || key == "no_proxy"
                || key == "proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_rke2_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_rke2_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_rke2_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_rke2_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_rke2_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_rke2_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_nomad_proxy(content: &str, proxy_url: &str) -> String {
    let trim_content = content.trim();
    let no_proxy_def = "10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,127.0.0.0/8,localhost,::1";
    if trim_content.is_empty() {
        return format!(
            "[Service]\nEnvironment=\"HTTP_PROXY={}\"\nEnvironment=\"HTTPS_PROXY={}\"\nEnvironment=\"NO_PROXY={}\"\n",
            proxy_url, proxy_url, no_proxy_def
        );
    }

    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let mut replaced_no = false;
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" {
                    lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    replaced_http = true;
                    continue;
                } else if k_low == "https_proxy" {
                    lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                    replaced_https = true;
                    continue;
                } else if k_low == "all_proxy" || k_low == "nomad_http_proxy" {
                    lines.push(format!("Environment=\"NOMAD_HTTP_PROXY={}\"", proxy_url));
                    continue;
                } else if k_low == "no_proxy" {
                    lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                    replaced_no = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy" {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if key == "https_proxy" {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if key == "all_proxy" || key == "nomad_http_proxy" {
                lines.push(format!("NOMAD_HTTP_PROXY=\"{}\"", proxy_url));
                continue;
            } else if key == "no_proxy" {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_http {
        if has_service_section {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_https {
        if has_service_section {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_no {
        if has_service_section {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        } else {
            lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
        }
    }

    lines.join("\n") + "\n"
}

pub fn remove_nomad_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "nomad_http_proxy"
                    || k_low == "nomad_https_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "nomad_http_proxy"
                || key == "nomad_https_proxy"
                || key == "no_proxy"
                || key == "proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_nomad_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_nomad_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_nomad_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_nomad_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_nomad_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_nomad_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_kubelet_proxy(content: &str, proxy_url: &str) -> String {
    let trim_content = content.trim();
    let no_proxy_def = "10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,127.0.0.0/8,localhost,.svc,.cluster.local";
    if trim_content.is_empty() {
        return format!(
            "[Service]\nEnvironment=\"HTTP_PROXY={}\"\nEnvironment=\"HTTPS_PROXY={}\"\nEnvironment=\"NO_PROXY={}\"\n",
            proxy_url, proxy_url, no_proxy_def
        );
    }

    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let mut replaced_no = false;
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" {
                    lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    replaced_http = true;
                    continue;
                } else if k_low == "https_proxy" {
                    lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                    replaced_https = true;
                    continue;
                } else if k_low == "all_proxy" || k_low == "kubelet_http_proxy" {
                    lines.push(format!("Environment=\"KUBELET_HTTP_PROXY={}\"", proxy_url));
                    continue;
                } else if k_low == "no_proxy" {
                    lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                    replaced_no = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy" {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if key == "https_proxy" {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if key == "all_proxy" || key == "kubelet_http_proxy" {
                lines.push(format!("KUBELET_HTTP_PROXY=\"{}\"", proxy_url));
                continue;
            } else if key == "no_proxy" {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_http {
        if has_service_section {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_https {
        if has_service_section {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_no {
        if has_service_section {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        } else {
            lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
        }
    }

    lines.join("\n") + "\n"
}

pub fn remove_kubelet_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "kubelet_http_proxy"
                    || k_low == "kubelet_https_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "kubelet_http_proxy"
                || key == "kubelet_https_proxy"
                || key == "no_proxy"
                || key == "proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_kubelet_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_kubelet_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_kubelet_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_kubelet_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_kubelet_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_kubelet_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

fn update_toml_environment_array(line: &str, proxy_url: &str, no_proxy: &str) -> String {
    let before_bracket = line.split('[').next().unwrap_or("  environment = ");
    let bracket_content = line
        .split_once('[')
        .and_then(|(_, rest)| rest.rsplit_once(']'))
        .map(|(inside, _)| inside)
        .unwrap_or("");

    let mut remaining_entries = Vec::new();
    for item in bracket_content.split(',') {
        let trim_item = item.trim();
        let unquoted = trim_item.trim_matches('"').trim_matches('\'').trim();
        let unquoted_low = unquoted.to_ascii_lowercase();
        if !unquoted_low.starts_with("http_proxy=")
            && !unquoted_low.starts_with("https_proxy=")
            && !unquoted_low.starts_with("all_proxy=")
            && !unquoted_low.starts_with("no_proxy=")
            && !unquoted_low.starts_with("ci_server_proxy=")
            && !unquoted_low.starts_with("gitlab_runner_http_proxy=")
            && !trim_item.is_empty()
        {
            remaining_entries.push(trim_item.to_string());
        }
    }
    remaining_entries.push(format!("\"HTTP_PROXY={}\"", proxy_url));
    remaining_entries.push(format!("\"HTTPS_PROXY={}\"", proxy_url));
    remaining_entries.push(format!("\"NO_PROXY={}\"", no_proxy));

    format!("{}[{}]", before_bracket, remaining_entries.join(", "))
}

fn remove_toml_environment_array(line: &str) -> (String, bool) {
    let before_bracket = line.split('[').next().unwrap_or("  environment = ");
    let bracket_content = match line
        .split_once('[')
        .and_then(|(_, rest)| rest.rsplit_once(']'))
        .map(|(inside, _)| inside)
    {
        Some(s) => s,
        None => return (line.to_string(), false),
    };

    let mut remaining_entries = Vec::new();
    let mut changed = false;
    for item in bracket_content.split(',') {
        let trim_item = item.trim();
        let unquoted = trim_item.trim_matches('"').trim_matches('\'').trim();
        let unquoted_low = unquoted.to_ascii_lowercase();
        if unquoted_low.starts_with("http_proxy=")
            || unquoted_low.starts_with("https_proxy=")
            || unquoted_low.starts_with("all_proxy=")
            || unquoted_low.starts_with("no_proxy=")
            || unquoted_low.starts_with("ci_server_proxy=")
            || unquoted_low.starts_with("gitlab_runner_http_proxy=")
        {
            changed = true;
        } else if !trim_item.is_empty() {
            remaining_entries.push(trim_item.to_string());
        }
    }

    if changed {
        (format!("{}[{}]", before_bracket, remaining_entries.join(", ")), true)
    } else {
        (line.to_string(), false)
    }
}

pub fn apply_gitlab_runner_proxy(content: &str, proxy_url: &str) -> String {
    let trim_content = content.trim();
    let no_proxy_def = "localhost,127.0.0.1,.local";

    let is_toml = trim_content.contains("[[runners]]")
        || trim_content.contains("concurrent =")
        || trim_content.contains("[session_server]");

    if trim_content.is_empty() {
        return format!(
            "[[runners]]\n  environment = [\"HTTP_PROXY={}\", \"HTTPS_PROXY={}\", \"NO_PROXY={}\"]\n",
            proxy_url, proxy_url, no_proxy_def
        );
    }

    if is_toml {
        let mut lines = Vec::new();
        let mut in_runner = false;
        let mut env_replaced = false;

        for line in content.lines() {
            let trim = line.trim();
            if trim.starts_with("[[runners]]") {
                if in_runner && !env_replaced {
                    lines.push(format!(
                        "  environment = [\"HTTP_PROXY={}\", \"HTTPS_PROXY={}\", \"NO_PROXY={}\"]",
                        proxy_url, proxy_url, no_proxy_def
                    ));
                }
                in_runner = true;
                env_replaced = false;
                lines.push(line.to_string());
                continue;
            }

            if in_runner && trim.starts_with("environment =") {
                lines.push(update_toml_environment_array(line, proxy_url, no_proxy_def));
                env_replaced = true;
                continue;
            }

            lines.push(line.to_string());
        }

        if in_runner && !env_replaced {
            lines.push(format!(
                "  environment = [\"HTTP_PROXY={}\", \"HTTPS_PROXY={}\", \"NO_PROXY={}\"]",
                proxy_url, proxy_url, no_proxy_def
            ));
        }

        return lines.join("\n") + "\n";
    }

    // Systemd drop-in or env format
    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let mut replaced_no = false;
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" {
                    lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                    replaced_http = true;
                    continue;
                } else if k_low == "https_proxy" {
                    lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                    replaced_https = true;
                    continue;
                } else if k_low == "all_proxy" || k_low == "ci_server_proxy" {
                    lines.push(format!("Environment=\"CI_SERVER_PROXY={}\"", proxy_url));
                    continue;
                } else if k_low == "no_proxy" {
                    lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                    replaced_no = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy" {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if key == "https_proxy" {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if key == "all_proxy" || key == "ci_server_proxy" {
                lines.push(format!("CI_SERVER_PROXY=\"{}\"", proxy_url));
                continue;
            } else if key == "no_proxy" {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_http {
        if has_service_section {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_https {
        if has_service_section {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        } else {
            lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
        }
    }

    if !replaced_no {
        if has_service_section {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        } else {
            lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
        }
    }

    lines.join("\n") + "\n"
}

pub fn remove_gitlab_runner_proxy(content: &str) -> (String, bool) {
    let trim_content = content.trim();
    let is_toml = trim_content.contains("[[runners]]")
        || trim_content.contains("concurrent =")
        || trim_content.contains("[session_server]");

    if is_toml {
        let mut lines = Vec::new();
        let mut changed = false;

        for line in content.lines() {
            let trim = line.trim();
            if trim.starts_with("environment =") {
                let (updated, did_change) = remove_toml_environment_array(line);
                if did_change {
                    changed = true;
                }
                lines.push(updated);
            } else {
                lines.push(line.to_string());
            }
        }

        return (lines.join("\n") + "\n", changed);
    }

    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "ci_server_proxy"
                    || k_low == "gitlab_runner_http_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "ci_server_proxy"
                || key == "gitlab_runner_http_proxy"
                || key == "no_proxy"
                || key == "proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_gitlab_runner_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_gitlab_runner_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_gitlab_runner_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_gitlab_runner_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_gitlab_runner_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_gitlab_runner_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_actions_runner_proxy(content: &str, proxy_url: &str) -> String {
    let no_proxy_def = "localhost,127.0.0.1,.local";

    // Systemd drop-in format: contains [Service] or Environment=
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "all_proxy" || k_low == "actions_runner_proxy" || k_low == "actions_runner_http_proxy" {
                        lines.push(format!("Environment=\"ACTIONS_RUNNER_PROXY={}\"", proxy_url));
                        continue;
                    } else if k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // Key-value .env format: contains http_proxy=, https_proxy=, etc.
    let has_kv = content.lines().any(|l| {
        let t = l.trim().to_ascii_lowercase();
        t.starts_with("http_proxy=") || t.starts_with("https_proxy=") || t.starts_with("actions_runner_proxy=")
    });

    if has_kv {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            if let Some((k, _)) = trim.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" {
                    lines.push(format!("http_proxy={}", proxy_url));
                    replaced_http = true;
                    continue;
                } else if k_low == "https_proxy" {
                    lines.push(format!("https_proxy={}", proxy_url));
                    replaced_https = true;
                    continue;
                } else if k_low == "no_proxy" {
                    lines.push(format!("no_proxy={}", no_proxy_def));
                    replaced_no = true;
                    continue;
                } else if k_low == "actions_runner_proxy" {
                    lines.push(format!("ACTIONS_RUNNER_PROXY={}", proxy_url));
                    continue;
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("http_proxy={}", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("https_proxy={}", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("no_proxy={}", no_proxy_def));
        }
        return lines.join("\n") + "\n";
    }

    // Default GitHub Actions Runner `.proxy` file format: single URL line!
    format!("{}\n", proxy_url)
}

pub fn remove_actions_runner_proxy(content: &str) -> (String, bool) {
    let trim = content.trim();
    if trim.is_empty() {
        return (String::new(), false);
    }

    // If it's a plain .proxy single-line URL
    if trim.starts_with("http://")
        || trim.starts_with("https://")
        || trim.starts_with("socks://")
        || trim.starts_with("socks5://")
    {
        return (String::new(), true);
    }

    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "actions_runner_proxy"
                    || k_low == "actions_runner_http_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "actions_runner_proxy"
                || key == "actions_runner_http_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_actions_runner_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_actions_runner_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_actions_runner_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_actions_runner_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_actions_runner_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_actions_runner_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_jenkins_proxy(content: &str, proxy_url: &str) -> String {
    let (host, port) = extract_host_port(proxy_url);

    // 1. XML proxy.xml format
    let is_xml = content.trim().is_empty()
        || content.contains("<proxy")
        || content.contains("<?xml")
        || (!content.contains("[Service]") && !content.contains("JAVA_ARGS=") && !content.contains("http_proxy="));

    if is_xml {
        if content.contains("<proxy>") {
            let mut lines = Vec::new();
            let mut replaced_name = false;
            let mut replaced_port = false;

            for line in content.lines() {
                let trim = line.trim();
                if trim.starts_with("<name>") && trim.ends_with("</name>") {
                    lines.push(format!("  <name>{}</name>", host));
                    replaced_name = true;
                    continue;
                }
                if trim.starts_with("<proxyHost>") && trim.ends_with("</proxyHost>") {
                    lines.push(format!("  <proxyHost>{}</proxyHost>", host));
                    replaced_name = true;
                    continue;
                }
                if trim.starts_with("<port>") && trim.ends_with("</port>") {
                    lines.push(format!("  <port>{}</port>", port));
                    replaced_port = true;
                    continue;
                }
                if trim.starts_with("<proxyPort>") && trim.ends_with("</proxyPort>") {
                    lines.push(format!("  <proxyPort>{}</proxyPort>", port));
                    replaced_port = true;
                    continue;
                }
                lines.push(line.to_string());
            }

            if !replaced_name || !replaced_port {
                return format!(
                    "<?xml version='1.1' encoding='UTF-8'?>\n<proxy>\n  <name>{}</name>\n  <port>{}</port>\n  <noProxyHost>localhost\n127.0.0.1</noProxyHost>\n</proxy>\n",
                    host, port
                );
            }
            return lines.join("\n") + "\n";
        }

        return format!(
            "<?xml version='1.1' encoding='UTF-8'?>\n<proxy>\n  <name>{}</name>\n  <port>{}</port>\n  <noProxyHost>localhost\n127.0.0.1</noProxyHost>\n</proxy>\n",
            host, port
        );
    }

    // 2. Systemd drop-in format
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let no_proxy_def = "localhost,127.0.0.1,.local";
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // 3. JAVA_ARGS / Environment format in /etc/default/jenkins or /etc/sysconfig/jenkins
    let mut lines = Vec::new();
    let mut handled_java_args = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with("JAVA_ARGS=") || trim.starts_with("JENKINS_JAVA_OPTIONS=") {
            let (prefix, val) = trim.split_once('=').unwrap();
            let clean_val = val.trim().trim_matches('"').trim_matches('\'');
            let filtered_words: Vec<&str> = clean_val
                .split_whitespace()
                .filter(|w| {
                    !w.starts_with("-Dhttp.proxyHost=")
                        && !w.starts_with("-Dhttp.proxyPort=")
                        && !w.starts_with("-Dhttps.proxyHost=")
                        && !w.starts_with("-Dhttps.proxyPort=")
                        && !w.starts_with("-Dhttp.nonProxyHosts=")
                })
                .collect();
            let mut words = filtered_words;
            let host_arg = format!("-Dhttp.proxyHost={}", host);
            let port_arg = format!("-Dhttp.proxyPort={}", port);
            let https_host_arg = format!("-Dhttps.proxyHost={}", host);
            let https_port_arg = format!("-Dhttps.proxyPort={}", port);
            let non_proxy_arg = "-Dhttp.nonProxyHosts=\"localhost|127.0.0.1\"".to_string();

            words.push(&host_arg);
            words.push(&port_arg);
            words.push(&https_host_arg);
            words.push(&https_port_arg);
            words.push(&non_proxy_arg);

            lines.push(format!("{}=\"{}\"", prefix, words.join(" ")));
            handled_java_args = true;
            continue;
        }
        lines.push(line.to_string());
    }

    if !handled_java_args {
        lines.push(format!(
            "JAVA_ARGS=\"-Dhttp.proxyHost={} -Dhttp.proxyPort={} -Dhttps.proxyHost={} -Dhttps.proxyPort={}\"",
            host, port, host, port
        ));
    }

    lines.join("\n") + "\n"
}

pub fn remove_jenkins_proxy(content: &str) -> (String, bool) {
    if content.contains("<proxy>") {
        return ("<?xml version='1.1' encoding='UTF-8'?>\n<proxy/>\n".to_string(), true);
    }

    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "jenkins_http_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if trim.starts_with("JAVA_ARGS=") || trim.starts_with("JENKINS_JAVA_OPTIONS=") {
            if trim.contains("-Dhttp.proxyHost=") || trim.contains("-Dhttps.proxyHost=") {
                let (prefix, val) = trim.split_once('=').unwrap();
                let clean_val = val.trim().trim_matches('"').trim_matches('\'');
                let filtered_words: Vec<&str> = clean_val
                    .split_whitespace()
                    .filter(|w| {
                        !w.starts_with("-Dhttp.proxyHost=")
                            && !w.starts_with("-Dhttp.proxyPort=")
                            && !w.starts_with("-Dhttps.proxyHost=")
                            && !w.starts_with("-Dhttps.proxyPort=")
                            && !w.starts_with("-Dhttp.nonProxyHosts=")
                    })
                    .collect();
                lines.push(format!("{}=\"{}\"", prefix, filtered_words.join(" ")));
                changed = true;
                continue;
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "jenkins_http_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_jenkins_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_jenkins_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_jenkins_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_jenkins_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_jenkins_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_jenkins_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_buildkite_agent_proxy(content: &str, proxy_url: &str) -> String {
    let no_proxy_def = "localhost,127.0.0.1,.local";

    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "all_proxy" || k_low == "buildkite_agent_proxy" || k_low == "buildkite_proxy" {
                        lines.push(format!("Environment=\"BUILDKITE_AGENT_PROXY={}\"", proxy_url));
                        continue;
                    } else if k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // Standard buildkite-agent.cfg format: proxy="url"
    let mut lines = Vec::new();
    let mut replaced_proxy = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_lowercase();
            if key == "proxy" || key == "http-proxy" || key == "buildkite_agent_proxy" || key == "http_proxy" {
                lines.push(format!("proxy=\"{}\"", proxy_url));
                replaced_proxy = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_proxy {
        lines.push(format!("proxy=\"{}\"", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_buildkite_agent_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "no_proxy"
                    || k_low == "buildkite_agent_proxy"
                    || k_low == "buildkite_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_lowercase();
            if key == "proxy"
                || key == "http-proxy"
                || key == "buildkite_agent_proxy"
                || key == "buildkite_proxy"
                || key == "http_proxy"
                || key == "https_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_buildkite_agent_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_buildkite_agent_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_buildkite_agent_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_buildkite_agent_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_buildkite_agent_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_buildkite_agent_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_woodpecker_agent_proxy(content: &str, proxy_url: &str) -> String {
    let no_proxy_def = "localhost,127.0.0.1,.local";

    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "woodpecker_http_proxy" || k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "woodpecker_https_proxy" || k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "woodpecker_no_proxy" || k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // Standard agent.conf or /etc/default/woodpecker-agent: WOODPECKER_HTTP_PROXY="url"
    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let mut replaced_no = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_lowercase();
            if key == "woodpecker_http_proxy" || key == "woodpecker_agent_http_proxy" || key == "http_proxy" {
                lines.push(format!("WOODPECKER_HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if key == "woodpecker_https_proxy" || key == "woodpecker_agent_https_proxy" || key == "https_proxy" {
                lines.push(format!("WOODPECKER_HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if key == "woodpecker_no_proxy" || key == "no_proxy" {
                lines.push(format!("WOODPECKER_NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_http {
        lines.push(format!("WOODPECKER_HTTP_PROXY=\"{}\"", proxy_url));
    }
    if !replaced_https {
        lines.push(format!("WOODPECKER_HTTPS_PROXY=\"{}\"", proxy_url));
    }
    if !replaced_no {
        lines.push(format!("WOODPECKER_NO_PROXY=\"{}\"", no_proxy_def));
    }

    lines.join("\n") + "\n"
}

pub fn remove_woodpecker_agent_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "woodpecker_http_proxy"
                    || k_low == "woodpecker_agent_http_proxy"
                    || k_low == "woodpecker_https_proxy"
                    || k_low == "woodpecker_no_proxy"
                    || k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_lowercase();
            if key == "woodpecker_http_proxy"
                || key == "woodpecker_agent_http_proxy"
                || key == "woodpecker_https_proxy"
                || key == "woodpecker_no_proxy"
                || key == "woodpecker_proxy"
                || key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_woodpecker_agent_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_woodpecker_agent_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_woodpecker_agent_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_woodpecker_agent_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_woodpecker_agent_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_woodpecker_agent_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_drone_runner_proxy(content: &str, proxy_url: &str) -> String {
    let no_proxy_def = "localhost,127.0.0.1,.local";

    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "drone_http_proxy" || k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "drone_https_proxy" || k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "drone_no_proxy" || k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // Standard config or /etc/default/drone-runner: DRONE_HTTP_PROXY="url"
    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let mut replaced_no = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_lowercase();
            if key == "drone_http_proxy" || key == "drone_runner_http_proxy" || key == "http_proxy" {
                lines.push(format!("DRONE_HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if key == "drone_https_proxy" || key == "drone_runner_https_proxy" || key == "https_proxy" {
                lines.push(format!("DRONE_HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if key == "drone_no_proxy" || key == "no_proxy" {
                lines.push(format!("DRONE_NO_PROXY=\"{}\"", no_proxy_def));
                replaced_no = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_http {
        lines.push(format!("DRONE_HTTP_PROXY=\"{}\"", proxy_url));
    }
    if !replaced_https {
        lines.push(format!("DRONE_HTTPS_PROXY=\"{}\"", proxy_url));
    }
    if !replaced_no {
        lines.push(format!("DRONE_NO_PROXY=\"{}\"", no_proxy_def));
    }

    lines.join("\n") + "\n"
}

pub fn remove_drone_runner_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "drone_http_proxy"
                    || k_low == "drone_https_proxy"
                    || k_low == "drone_no_proxy"
                    || k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_lowercase();
            if key == "drone_http_proxy"
                || key == "drone_runner_http_proxy"
                || key == "drone_https_proxy"
                || key == "drone_runner_https_proxy"
                || key == "drone_no_proxy"
                || key == "drone_proxy"
                || key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_drone_runner_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_drone_runner_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_drone_runner_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_drone_runner_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_drone_runner_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_drone_runner_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_act_runner_proxy(content: &str, proxy_url: &str) -> String {
    let no_proxy_def = "localhost,127.0.0.1,.local";

    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "act_runner_http_proxy" || k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "act_runner_https_proxy" || k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "act_runner_no_proxy" || k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // Default env file (e.g. /etc/default/act-runner) if it has equals sign but no runner:
    if content.lines().any(|l| l.contains('=') && !l.trim().starts_with('#')) && !content.contains("runner:") {
        let mut lines = Vec::new();
        let mut replaced = false;
        for line in content.lines() {
            let trim = line.trim();
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().trim_matches('"').trim_matches('\'').to_lowercase();
                if key == "act_runner_http_proxy" || key == "http_proxy" {
                    lines.push(format!("ACT_RUNNER_HTTP_PROXY=\"{}\"", proxy_url));
                    replaced = true;
                    continue;
                }
            }
            lines.push(line.to_string());
        }
        if !replaced {
            lines.push(format!("ACT_RUNNER_HTTP_PROXY=\"{}\"", proxy_url));
        }
        return lines.join("\n") + "\n";
    }

    // YAML format: config.yaml / .runner
    let mut lines = Vec::new();
    let mut in_runner = false;
    let mut in_envs = false;
    let mut envs_found = false;
    let mut runner_found = false;
    let mut http_set = false;
    let mut https_set = false;
    let mut no_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            lines.push(line.to_string());
            continue;
        }

        if trim == "runner:" || trim.starts_with("runner:") {
            in_runner = true;
            runner_found = true;
            lines.push(line.to_string());
            continue;
        }

        if in_runner && (line.starts_with("  envs:") || trim == "envs:" || trim.starts_with("envs:")) {
            in_envs = true;
            envs_found = true;
            lines.push(line.to_string());
            continue;
        }

        // Exiting runner or envs section
        if in_envs && !line.starts_with("    ") && !trim.is_empty() {
            if !http_set {
                lines.push(format!("    http_proxy: \"{}\"", proxy_url));
                http_set = true;
            }
            if !https_set {
                lines.push(format!("    https_proxy: \"{}\"", proxy_url));
                https_set = true;
            }
            if !no_set {
                lines.push(format!("    no_proxy: \"{}\"", no_proxy_def));
                no_set = true;
            }
            in_envs = false;
            if !line.starts_with("  ") {
                in_runner = false;
            }
        } else if in_runner && !in_envs && !line.starts_with("  ") && !trim.is_empty() {
            in_runner = false;
        }

        if in_envs {
            if let Some((k, _)) = trim.split_once(':') {
                let key = k.trim().to_lowercase();
                if key == "http_proxy" {
                    lines.push(format!("    http_proxy: \"{}\"", proxy_url));
                    http_set = true;
                    continue;
                } else if key == "https_proxy" {
                    lines.push(format!("    https_proxy: \"{}\"", proxy_url));
                    https_set = true;
                    continue;
                } else if key == "no_proxy" {
                    lines.push(format!("    no_proxy: \"{}\"", no_proxy_def));
                    no_set = true;
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    if in_envs {
        if !http_set {
            lines.push(format!("    http_proxy: \"{}\"", proxy_url));
        }
        if !https_set {
            lines.push(format!("    https_proxy: \"{}\"", proxy_url));
        }
        if !no_set {
            lines.push(format!("    no_proxy: \"{}\"", no_proxy_def));
        }
    } else if runner_found && !envs_found {
        // runner: was present but no envs:
        let mut new_lines = Vec::new();
        for line in lines {
            new_lines.push(line.clone());
            if line.trim() == "runner:" || line.trim().starts_with("runner:") {
                new_lines.push("  envs:".to_string());
                new_lines.push(format!("    http_proxy: \"{}\"", proxy_url));
                new_lines.push(format!("    https_proxy: \"{}\"", proxy_url));
                new_lines.push(format!("    no_proxy: \"{}\"", no_proxy_def));
            }
        }
        lines = new_lines;
    } else if !runner_found {
        lines.push("runner:".to_string());
        lines.push("  envs:".to_string());
        lines.push(format!("    http_proxy: \"{}\"", proxy_url));
        lines.push(format!("    https_proxy: \"{}\"", proxy_url));
        lines.push(format!("    no_proxy: \"{}\"", no_proxy_def));
    }

    lines.join("\n") + "\n"
}

pub fn remove_act_runner_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "act_runner_http_proxy"
                    || k_low == "act_runner_https_proxy"
                    || k_low == "act_runner_no_proxy"
                    || k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once(':') {
            let key = k.trim().to_lowercase();
            if key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "no_proxy"
                || key == "act_runner_http_proxy"
            {
                changed = true;
                continue;
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_lowercase();
            if key == "act_runner_http_proxy"
                || key == "act_runner_https_proxy"
                || key == "act_runner_no_proxy"
                || key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_act_runner_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_act_runner_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_act_runner_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_act_runner_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_act_runner_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_act_runner_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_circleci_runner_proxy(content: &str, proxy_url: &str) -> String {
    let no_proxy_def = "localhost,127.0.0.1,.local";

    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "all_proxy" || k_low == "circleci_http_proxy" || k_low == "circleci_runner_http_proxy" {
                        lines.push(format!("Environment=\"CIRCLECI_HTTP_PROXY={}\"", proxy_url));
                        continue;
                    } else if k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // Key=value format (/etc/default/circleci-runner)
    let has_kv = content.lines().any(|l| {
        let t = l.trim().to_ascii_lowercase();
        t.starts_with("circleci_http_proxy=") || t.starts_with("circleci_runner_http_proxy=")
    });

    if has_kv {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            if trim.starts_with('#') || trim.starts_with(';') {
                lines.push(line.to_string());
                continue;
            }

            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().trim_matches('"').trim_matches('\'').to_lowercase();
                if key == "circleci_http_proxy" || key == "circleci_runner_http_proxy" || key == "http_proxy" {
                    lines.push(format!("CIRCLECI_HTTP_PROXY=\"{}\"", proxy_url));
                    replaced_http = true;
                    continue;
                } else if key == "circleci_https_proxy" || key == "circleci_runner_https_proxy" || key == "https_proxy" {
                    lines.push(format!("CIRCLECI_HTTPS_PROXY=\"{}\"", proxy_url));
                    replaced_https = true;
                    continue;
                } else if key == "circleci_no_proxy" || key == "no_proxy" {
                    lines.push(format!("CIRCLECI_NO_PROXY=\"{}\"", no_proxy_def));
                    replaced_no = true;
                    continue;
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("CIRCLECI_HTTP_PROXY=\"{}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("CIRCLECI_HTTPS_PROXY=\"{}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("CIRCLECI_NO_PROXY=\"{}\"", no_proxy_def));
        }
        return lines.join("\n") + "\n";
    }

    // YAML format: circleci-runner-config.yaml
    let mut lines = Vec::new();
    let mut in_proxy = false;
    let mut proxy_found = false;
    let mut http_set = false;
    let mut https_set = false;
    let mut no_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') {
            lines.push(line.to_string());
            continue;
        }

        if trim == "proxy:" || trim.starts_with("proxy:") {
            in_proxy = true;
            proxy_found = true;
            lines.push("proxy:".to_string());
            continue;
        }

        if in_proxy && !line.starts_with("  ") && !trim.is_empty() {
            if !http_set {
                lines.push(format!("  http: \"{}\"", proxy_url));
                http_set = true;
            }
            if !https_set {
                lines.push(format!("  https: \"{}\"", proxy_url));
                https_set = true;
            }
            if !no_set {
                lines.push(format!("  no_proxy: \"{}\"", no_proxy_def));
                no_set = true;
            }
            in_proxy = false;
        }

        if in_proxy {
            if let Some((k, _)) = trim.split_once(':') {
                let key = k.trim().to_lowercase();
                if key == "http" || key == "http_proxy" {
                    lines.push(format!("  http: \"{}\"", proxy_url));
                    http_set = true;
                    continue;
                } else if key == "https" || key == "https_proxy" {
                    lines.push(format!("  https: \"{}\"", proxy_url));
                    https_set = true;
                    continue;
                } else if key == "no_proxy" {
                    lines.push(format!("  no_proxy: \"{}\"", no_proxy_def));
                    no_set = true;
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    if in_proxy {
        if !http_set {
            lines.push(format!("  http: \"{}\"", proxy_url));
        }
        if !https_set {
            lines.push(format!("  https: \"{}\"", proxy_url));
        }
        if !no_set {
            lines.push(format!("  no_proxy: \"{}\"", no_proxy_def));
        }
    } else if !proxy_found {
        if !lines.is_empty() && !lines.last().map(|l| l.is_empty()).unwrap_or(true) {
            lines.push(String::new());
        }
        lines.push("proxy:".to_string());
        lines.push(format!("  http: \"{}\"", proxy_url));
        lines.push(format!("  https: \"{}\"", proxy_url));
        lines.push(format!("  no_proxy: \"{}\"", no_proxy_def));
    }

    lines.join("\n") + "\n"
}

pub fn remove_circleci_runner_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;
    let mut in_proxy = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "circleci_http_proxy"
                    || k_low == "circleci_runner_http_proxy"
                    || k_low == "circleci_https_proxy"
                    || k_low == "circleci_no_proxy"
                    || k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if trim == "proxy:" || trim.starts_with("proxy:") {
            in_proxy = true;
            changed = true;
            continue;
        }

        if in_proxy {
            if line.starts_with("  ") || trim.is_empty() {
                changed = true;
                continue;
            } else {
                in_proxy = false;
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_lowercase();
            if key == "circleci_http_proxy"
                || key == "circleci_runner_http_proxy"
                || key == "circleci_https_proxy"
                || key == "circleci_no_proxy"
                || key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_circleci_runner_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_circleci_runner_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_circleci_runner_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_circleci_runner_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_circleci_runner_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_circleci_runner_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_azure_pipelines_agent_proxy(content: &str, proxy_url: &str) -> String {
    let no_proxy_def = "localhost,127.0.0.1,.local";

    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "all_proxy" || k_low == "vsts_http_proxy" || k_low == "azp_http_proxy" {
                        lines.push(format!("Environment=\"VSTS_HTTP_PROXY={}\"", proxy_url));
                        continue;
                    } else if k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // Key-value .env format: contains http_proxy=, https_proxy=, or vsts_http_proxy=
    let has_kv = content.lines().any(|l| {
        let t = l.trim().to_ascii_lowercase();
        t.starts_with("http_proxy=") || t.starts_with("https_proxy=") || t.starts_with("vsts_http_proxy=") || t.starts_with("azp_http_proxy=")
    });

    if has_kv {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            if let Some((k, _)) = trim.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" || k_low == "vsts_http_proxy" {
                    lines.push(format!("http_proxy={}", proxy_url));
                    lines.push(format!("VSTS_HTTP_PROXY={}", proxy_url));
                    replaced_http = true;
                    continue;
                } else if k_low == "https_proxy" || k_low == "vsts_https_proxy" {
                    lines.push(format!("https_proxy={}", proxy_url));
                    lines.push(format!("VSTS_HTTPS_PROXY={}", proxy_url));
                    replaced_https = true;
                    continue;
                } else if k_low == "no_proxy" {
                    lines.push(format!("no_proxy={}", no_proxy_def));
                    replaced_no = true;
                    continue;
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("http_proxy={}", proxy_url));
            lines.push(format!("VSTS_HTTP_PROXY={}", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("https_proxy={}", proxy_url));
            lines.push(format!("VSTS_HTTPS_PROXY={}", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("no_proxy={}", no_proxy_def));
        }
        return lines.join("\n") + "\n";
    }

    // Default Azure Pipelines Agent `.proxy` format: single line proxy URL
    format!("{}\n", proxy_url)
}

pub fn remove_azure_pipelines_agent_proxy(content: &str) -> (String, bool) {
    let trim = content.trim();
    if trim.is_empty() {
        return (String::new(), false);
    }

    // Plain .proxy format
    if trim.starts_with("http://")
        || trim.starts_with("https://")
        || trim.starts_with("socks://")
        || trim.starts_with("socks5://")
    {
        return (String::new(), true);
    }

    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "vsts_http_proxy"
                    || k_low == "vsts_https_proxy"
                    || k_low == "azp_http_proxy"
                    || k_low == "azp_https_proxy"
                    || k_low == "azure_pipelines_http_proxy"
                    || k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_lowercase();
            if key == "vsts_http_proxy"
                || key == "vsts_https_proxy"
                || key == "azp_http_proxy"
                || key == "azp_https_proxy"
                || key == "azure_pipelines_http_proxy"
                || key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_azure_pipelines_agent_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_azure_pipelines_agent_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_azure_pipelines_agent_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    // If writing to .proxy, also generate .proxybypass
    if path.file_name().map(|n| n == ".proxy").unwrap_or(false) {
        if let Some(parent) = path.parent() {
            let bypass_file = parent.join(".proxybypass");
            let _ = std::fs::write(bypass_file, "[\"localhost\", \"127.0.0.1\", \".local\"]\n");
        }
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_azure_pipelines_agent_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_azure_pipelines_agent_config_paths() {
        if path.exists() {
            if path.file_name().map(|n| n == ".proxy").unwrap_or(false) {
                let _ = std::fs::remove_file(&path);
                if let Some(parent) = path.parent() {
                    let bypass_file = parent.join(".proxybypass");
                    if bypass_file.exists() {
                        let _ = std::fs::remove_file(bypass_file);
                    }
                }
                any_removed = true;
            } else if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_azure_pipelines_agent_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_teamcity_agent_proxy(content: &str, proxy_url: &str) -> String {
    let no_proxy_def = "localhost,127.0.0.1,.local";
    let (host, port) = extract_host_port(proxy_url);

    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "all_proxy" || k_low == "teamcity_http_proxy" || k_low == "buildagent_http_proxy" {
                        lines.push(format!("Environment=\"TEAMCITY_HTTP_PROXY={}\"", proxy_url));
                        continue;
                    } else if k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // Default: Java properties file format for buildAgent.properties
    let mut lines = Vec::new();
    let mut env_http_set = false;
    let mut env_https_set = false;
    let mut env_no_set = false;
    let mut host_set = false;
    let mut port_set = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') || trim.starts_with('!') {
            lines.push(line.to_string());
            continue;
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_lowercase();
            if key == "env.http_proxy" {
                lines.push(format!("env.HTTP_PROXY={}", proxy_url));
                env_http_set = true;
                continue;
            } else if key == "env.https_proxy" {
                lines.push(format!("env.HTTPS_PROXY={}", proxy_url));
                env_https_set = true;
                continue;
            } else if key == "env.no_proxy" {
                lines.push(format!("env.NO_PROXY={}", no_proxy_def));
                env_no_set = true;
                continue;
            } else if key == "teamcity.http.proxyhost" || key == "http.proxyhost" {
                lines.push(format!("teamcity.http.proxyHost={}", host));
                host_set = true;
                continue;
            } else if key == "teamcity.http.proxyport" || key == "http.proxyport" {
                lines.push(format!("teamcity.http.proxyPort={}", port));
                port_set = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !env_http_set {
        lines.push(format!("env.HTTP_PROXY={}", proxy_url));
    }
    if !env_https_set {
        lines.push(format!("env.HTTPS_PROXY={}", proxy_url));
    }
    if !env_no_set {
        lines.push(format!("env.NO_PROXY={}", no_proxy_def));
    }
    if !host_set {
        lines.push(format!("teamcity.http.proxyHost={}", host));
    }
    if !port_set {
        lines.push(format!("teamcity.http.proxyPort={}", port));
    }

    lines.join("\n") + "\n"
}

pub fn remove_teamcity_agent_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') || trim.starts_with('!') {
            lines.push(line.to_string());
            continue;
        }

        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "teamcity_http_proxy"
                    || k_low == "buildagent_http_proxy"
                    || k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_lowercase();
            if key == "env.http_proxy"
                || key == "env.https_proxy"
                || key == "env.all_proxy"
                || key == "env.no_proxy"
                || key == "teamcity.http.proxyhost"
                || key == "teamcity.http.proxyport"
                || key == "teamcity.https.proxyhost"
                || key == "teamcity.https.proxyport"
                || key == "teamcity_http_proxy"
                || key == "buildagent_http_proxy"
                || key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_teamcity_agent_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_teamcity_agent_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_teamcity_agent_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_teamcity_agent_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_teamcity_agent_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_teamcity_agent_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_bamboo_agent_proxy(content: &str, proxy_url: &str) -> String {
    let no_proxy_def = "localhost,127.0.0.1,.local";
    let (host, port) = extract_host_port(proxy_url);

    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "all_proxy" || k_low == "bamboo_http_proxy" || k_low == "bamboo_agent_http_proxy" {
                        lines.push(format!("Environment=\"BAMBOO_HTTP_PROXY={}\"", proxy_url));
                        continue;
                    } else if k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // Wrapper.conf format
    if content.lines().any(|l| l.trim().starts_with("wrapper.")) {
        let mut lines = Vec::new();
        let mut max_idx = 0;
        for line in content.lines() {
            let trim = line.trim();
            if trim.contains("-Dhttp.proxyHost")
                || trim.contains("-Dhttp.proxyPort")
                || trim.contains("-Dhttps.proxyHost")
                || trim.contains("-Dhttps.proxyPort")
                || trim.contains("-Dhttp.nonProxyHosts")
            {
                continue;
            }
            if let Some(rest) = trim.strip_prefix("wrapper.java.additional.") {
                if let Some((idx_str, _)) = rest.split_once('=') {
                    if let Ok(idx) = idx_str.trim().parse::<u32>() {
                        if idx > max_idx {
                            max_idx = idx;
                        }
                    }
                }
            }
            lines.push(line.to_string());
        }

        lines.push(format!("wrapper.java.additional.{} = -Dhttp.proxyHost={}", max_idx + 1, host));
        lines.push(format!("wrapper.java.additional.{} = -Dhttp.proxyPort={}", max_idx + 2, port));
        lines.push(format!("wrapper.java.additional.{} = -Dhttps.proxyHost={}", max_idx + 3, host));
        lines.push(format!("wrapper.java.additional.{} = -Dhttps.proxyPort={}", max_idx + 4, port));
        return lines.join("\n") + "\n";
    }

    // XML format: bamboo-agent.cfg.xml
    let is_xml = content.contains("<configuration>") || content.contains("<?xml") || content.trim().is_empty();
    if is_xml {
        if content.contains("<configuration>") {
            let mut lines = Vec::new();
            let mut host_set = false;
            let mut port_set = false;

            for line in content.lines() {
                let trim = line.trim();
                if (trim.starts_with("<httpProxyHost>") && trim.ends_with("</httpProxyHost>"))
                    || (trim.starts_with("<proxyHost>") && trim.ends_with("</proxyHost>"))
                {
                    lines.push(format!("  <httpProxyHost>{}</httpProxyHost>", host));
                    host_set = true;
                    continue;
                }
                if (trim.starts_with("<httpProxyPort>") && trim.ends_with("</httpProxyPort>"))
                    || (trim.starts_with("<proxyPort>") && trim.ends_with("</proxyPort>"))
                {
                    lines.push(format!("  <httpProxyPort>{}</httpProxyPort>", port));
                    port_set = true;
                    continue;
                }
                if trim == "</configuration>" {
                    if !host_set {
                        lines.push(format!("  <httpProxyHost>{}</httpProxyHost>", host));
                    }
                    if !port_set {
                        lines.push(format!("  <httpProxyPort>{}</httpProxyPort>", port));
                    }
                }
                lines.push(line.to_string());
            }
            return lines.join("\n") + "\n";
        } else {
            return format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<configuration>\n  <httpProxyHost>{}</httpProxyHost>\n  <httpProxyPort>{}</httpProxyPort>\n</configuration>\n",
                host, port
            );
        }
    }

    // Default key=value
    format!("BAMBOO_HTTP_PROXY=\"{}\"\n", proxy_url)
}

pub fn remove_bamboo_agent_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "bamboo_http_proxy"
                    || k_low == "bamboo_agent_http_proxy"
                    || k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if (trim.starts_with("<httpProxyHost>") && trim.ends_with("</httpProxyHost>"))
            || (trim.starts_with("<httpProxyPort>") && trim.ends_with("</httpProxyPort>"))
            || (trim.starts_with("<proxyHost>") && trim.ends_with("</proxyHost>"))
            || (trim.starts_with("<proxyPort>") && trim.ends_with("</proxyPort>"))
        {
            changed = true;
            continue;
        }

        if trim.contains("-Dhttp.proxyHost")
            || trim.contains("-Dhttp.proxyPort")
            || trim.contains("-Dhttps.proxyHost")
            || trim.contains("-Dhttps.proxyPort")
            || trim.contains("-Dhttp.nonProxyHosts")
        {
            changed = true;
            continue;
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_lowercase();
            if key == "bamboo_http_proxy"
                || key == "bamboo_agent_http_proxy"
                || key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_bamboo_agent_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_bamboo_agent_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_bamboo_agent_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_bamboo_agent_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_bamboo_agent_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_bamboo_agent_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_tfc_agent_proxy(content: &str, proxy_url: &str) -> String {
    let no_proxy_def = "localhost,127.0.0.1,.local";
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_tfc_http = false;
        let mut replaced_tfc_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "tfc_agent_http_proxy" {
                        lines.push(format!("Environment=\"TFC_AGENT_HTTP_PROXY={}\"", proxy_url));
                        replaced_tfc_http = true;
                        continue;
                    } else if k_low == "tfc_agent_https_proxy" {
                        lines.push(format!("Environment=\"TFC_AGENT_HTTPS_PROXY={}\"", proxy_url));
                        replaced_tfc_https = true;
                        continue;
                    } else if k_low == "no_proxy" || k_low == "tfc_agent_no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_tfc_http {
            lines.push(format!("Environment=\"TFC_AGENT_HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_tfc_https {
            lines.push(format!("Environment=\"TFC_AGENT_HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // Default key=value shell / env format for agent.env
    let mut lines = Vec::new();
    let mut set_http = false;
    let mut set_https = false;
    let mut set_tfc_http = false;
    let mut set_tfc_https = false;
    let mut set_no = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_lowercase();
            if key == "http_proxy" {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                set_http = true;
                continue;
            } else if key == "https_proxy" {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                set_https = true;
                continue;
            } else if key == "tfc_agent_http_proxy" {
                lines.push(format!("TFC_AGENT_HTTP_PROXY=\"{}\"", proxy_url));
                set_tfc_http = true;
                continue;
            } else if key == "tfc_agent_https_proxy" {
                lines.push(format!("TFC_AGENT_HTTPS_PROXY=\"{}\"", proxy_url));
                set_tfc_https = true;
                continue;
            } else if key == "no_proxy" || key == "tfc_agent_no_proxy" {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                set_no = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !set_http {
        lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
    }
    if !set_https {
        lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
    }
    if !set_tfc_http {
        lines.push(format!("TFC_AGENT_HTTP_PROXY=\"{}\"", proxy_url));
    }
    if !set_tfc_https {
        lines.push(format!("TFC_AGENT_HTTPS_PROXY=\"{}\"", proxy_url));
    }
    if !set_no {
        lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
    }

    lines.join("\n") + "\n"
}

pub fn remove_tfc_agent_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "tfc_agent_http_proxy"
                    || k_low == "tfc_agent_https_proxy"
                    || k_low == "tfc_agent_no_proxy"
                    || k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_lowercase();
            if key == "tfc_agent_http_proxy"
                || key == "tfc_agent_https_proxy"
                || key == "tfc_agent_no_proxy"
                || key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_tfc_agent_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_tfc_agent_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_tfc_agent_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_tfc_agent_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_tfc_agent_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_tfc_agent_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_concourse_worker_proxy(content: &str, proxy_url: &str) -> String {
    let no_proxy_def = "localhost,127.0.0.1,.local";
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_concourse_http = false;
        let mut replaced_concourse_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "concourse_http_proxy" {
                        lines.push(format!("Environment=\"CONCOURSE_HTTP_PROXY={}\"", proxy_url));
                        replaced_concourse_http = true;
                        continue;
                    } else if k_low == "concourse_https_proxy" {
                        lines.push(format!("Environment=\"CONCOURSE_HTTPS_PROXY={}\"", proxy_url));
                        replaced_concourse_https = true;
                        continue;
                    } else if k_low == "no_proxy" || k_low == "concourse_no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_concourse_http {
            lines.push(format!("Environment=\"CONCOURSE_HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_concourse_https {
            lines.push(format!("Environment=\"CONCOURSE_HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // Default key=value shell / env format for worker.env
    let mut lines = Vec::new();
    let mut set_http = false;
    let mut set_https = false;
    let mut set_concourse_http = false;
    let mut set_concourse_https = false;
    let mut set_no = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_lowercase();
            if key == "http_proxy" {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                set_http = true;
                continue;
            } else if key == "https_proxy" {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                set_https = true;
                continue;
            } else if key == "concourse_http_proxy" {
                lines.push(format!("CONCOURSE_HTTP_PROXY=\"{}\"", proxy_url));
                set_concourse_http = true;
                continue;
            } else if key == "concourse_https_proxy" {
                lines.push(format!("CONCOURSE_HTTPS_PROXY=\"{}\"", proxy_url));
                set_concourse_https = true;
                continue;
            } else if key == "no_proxy" || key == "concourse_no_proxy" {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                set_no = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !set_http {
        lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
    }
    if !set_https {
        lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
    }
    if !set_concourse_http {
        lines.push(format!("CONCOURSE_HTTP_PROXY=\"{}\"", proxy_url));
    }
    if !set_concourse_https {
        lines.push(format!("CONCOURSE_HTTPS_PROXY=\"{}\"", proxy_url));
    }
    if !set_no {
        lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
    }

    lines.join("\n") + "\n"
}

pub fn remove_concourse_worker_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "concourse_http_proxy"
                    || k_low == "concourse_https_proxy"
                    || k_low == "concourse_no_proxy"
                    || k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_lowercase();
            if key == "concourse_http_proxy"
                || key == "concourse_https_proxy"
                || key == "concourse_no_proxy"
                || key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_concourse_worker_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_concourse_worker_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_concourse_worker_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_concourse_worker_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_concourse_worker_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_concourse_worker_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_tentacle_agent_proxy(content: &str, proxy_url: &str) -> String {
    let (proxy_host, proxy_port) = extract_host_port(proxy_url);
    let no_proxy_def = "localhost,127.0.0.1,.local";

    let is_xml = content.contains("<octopus-settings") || content.contains("<set key=") || content.trim_start().starts_with("<?xml");
    if is_xml {
        let mut lines = Vec::new();
        let mut set_host = false;
        let mut set_port = false;

        for line in content.lines() {
            let trim = line.trim();
            if trim.contains("Tentacle.Services.ProxyHost") {
                lines.push(format!("  <set key=\"Tentacle.Services.ProxyHost\">{}</set>", proxy_host));
                set_host = true;
                continue;
            }
            if trim.contains("Tentacle.Services.ProxyPort") {
                lines.push(format!("  <set key=\"Tentacle.Services.ProxyPort\">{}</set>", proxy_port));
                set_port = true;
                continue;
            }
            lines.push(line.to_string());
        }

        if !set_host || !set_port {
            let mut new_lines = Vec::new();
            let mut inserted = false;
            for line in lines {
                if !inserted && line.trim().eq_ignore_ascii_case("</octopus-settings>") {
                    if !set_host {
                        new_lines.push(format!("  <set key=\"Tentacle.Services.ProxyHost\">{}</set>", proxy_host));
                    }
                    if !set_port {
                        new_lines.push(format!("  <set key=\"Tentacle.Services.ProxyPort\">{}</set>", proxy_port));
                    }
                    inserted = true;
                }
                new_lines.push(line);
            }
            if !inserted {
                if !set_host {
                    new_lines.push(format!("  <set key=\"Tentacle.Services.ProxyHost\">{}</set>", proxy_host));
                }
                if !set_port {
                    new_lines.push(format!("  <set key=\"Tentacle.Services.ProxyPort\">{}</set>", proxy_port));
                }
            }
            return new_lines.join("\n") + "\n";
        }

        return lines.join("\n") + "\n";
    }

    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_octo_host = false;
        let mut replaced_octo_port = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "octopus_proxy_host" || k_low == "tentacle_proxy_host" {
                        lines.push(format!("Environment=\"OCTOPUS_PROXY_HOST={}\"", proxy_host));
                        replaced_octo_host = true;
                        continue;
                    } else if k_low == "octopus_proxy_port" || k_low == "tentacle_proxy_port" {
                        lines.push(format!("Environment=\"OCTOPUS_PROXY_PORT={}\"", proxy_port));
                        replaced_octo_port = true;
                        continue;
                    } else if k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_octo_host {
            lines.push(format!("Environment=\"OCTOPUS_PROXY_HOST={}\"", proxy_host));
        }
        if !replaced_octo_port {
            lines.push(format!("Environment=\"OCTOPUS_PROXY_PORT={}\"", proxy_port));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // Default key=value / env format
    let mut lines = Vec::new();
    let mut set_http = false;
    let mut set_https = false;
    let mut set_octo_host = false;
    let mut set_octo_port = false;
    let mut set_no = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_lowercase();
            if key == "http_proxy" {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                set_http = true;
                continue;
            } else if key == "https_proxy" {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                set_https = true;
                continue;
            } else if key == "octopus_proxy_host" || key == "tentacle_proxy_host" {
                lines.push(format!("OCTOPUS_PROXY_HOST=\"{}\"", proxy_host));
                set_octo_host = true;
                continue;
            } else if key == "octopus_proxy_port" || key == "tentacle_proxy_port" {
                lines.push(format!("OCTOPUS_PROXY_PORT=\"{}\"", proxy_port));
                set_octo_port = true;
                continue;
            } else if key == "no_proxy" {
                lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
                set_no = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !set_http {
        lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
    }
    if !set_https {
        lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
    }
    if !set_octo_host {
        lines.push(format!("OCTOPUS_PROXY_HOST=\"{}\"", proxy_host));
    }
    if !set_octo_port {
        lines.push(format!("OCTOPUS_PROXY_PORT=\"{}\"", proxy_port));
    }
    if !set_no {
        lines.push(format!("NO_PROXY=\"{}\"", no_proxy_def));
    }

    lines.join("\n") + "\n"
}

pub fn remove_tentacle_agent_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        if trim.contains("Tentacle.Services.ProxyHost")
            || trim.contains("Tentacle.Services.ProxyPort")
            || trim.contains("Tentacle.Services.ProxyUsername")
            || trim.contains("Tentacle.Services.ProxyPassword")
        {
            changed = true;
            continue;
        }

        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "octopus_proxy_host"
                    || k_low == "octopus_proxy_port"
                    || k_low == "tentacle_proxy_host"
                    || k_low == "tentacle_proxy_port"
                    || k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_lowercase();
            if key == "octopus_proxy_host"
                || key == "octopus_proxy_port"
                || key == "tentacle_proxy_host"
                || key == "tentacle_proxy_port"
                || key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_tentacle_agent_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_tentacle_agent_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_tentacle_agent_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_tentacle_agent_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_tentacle_agent_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_tentacle_agent_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_puppet_agent_proxy(content: &str, proxy_url: &str) -> String {
    let (proxy_host, proxy_port) = extract_host_port(proxy_url);
    let no_proxy_def = "localhost,127.0.0.1,.local";

    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_host = false;
        let mut replaced_port = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "http_proxy_host" || k_low == "puppet_proxy_host" {
                        lines.push(format!("Environment=\"HTTP_PROXY_HOST={}\"", proxy_host));
                        replaced_host = true;
                        continue;
                    } else if k_low == "http_proxy_port" || k_low == "puppet_proxy_port" {
                        lines.push(format!("Environment=\"HTTP_PROXY_PORT={}\"", proxy_port));
                        replaced_port = true;
                        continue;
                    } else if k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_host {
            lines.push(format!("Environment=\"HTTP_PROXY_HOST={}\"", proxy_host));
        }
        if !replaced_port {
            lines.push(format!("Environment=\"HTTP_PROXY_PORT={}\"", proxy_port));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // puppet.conf INI format
    let has_main_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[main]"));
    let mut lines = Vec::new();
    let mut set_host = false;
    let mut set_port = false;
    let mut set_no = false;
    let mut in_main_section = !has_main_section;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            let sec = trim[1..trim.len() - 1].trim();
            in_main_section = sec.eq_ignore_ascii_case("main");
            lines.push(line.to_string());
            continue;
        }

        if in_main_section {
            if let Some((k, _)) = trim.split_once('=') {
                let key = k.trim().to_lowercase();
                if key == "http_proxy_host" || key == "puppet_proxy_host" {
                    lines.push(format!("http_proxy_host = {}", proxy_host));
                    set_host = true;
                    continue;
                } else if key == "http_proxy_port" || key == "puppet_proxy_port" {
                    lines.push(format!("http_proxy_port = {}", proxy_port));
                    set_port = true;
                    continue;
                } else if key == "no_proxy" {
                    lines.push(format!("no_proxy = {}", no_proxy_def));
                    set_no = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    if has_main_section {
        if !set_host || !set_port || !set_no {
            let mut new_lines = Vec::new();
            let mut inserted = false;
            for line in lines {
                new_lines.push(line.clone());
                if !inserted && line.trim().eq_ignore_ascii_case("[main]") {
                    if !set_host {
                        new_lines.push(format!("http_proxy_host = {}", proxy_host));
                    }
                    if !set_port {
                        new_lines.push(format!("http_proxy_port = {}", proxy_port));
                    }
                    if !set_no {
                        new_lines.push(format!("no_proxy = {}", no_proxy_def));
                    }
                    inserted = true;
                }
            }
            return new_lines.join("\n") + "\n";
        }
        return lines.join("\n") + "\n";
    }

    let mut res = Vec::new();
    res.push("[main]".to_string());
    if !set_host {
        res.push(format!("http_proxy_host = {}", proxy_host));
    }
    if !set_port {
        res.push(format!("http_proxy_port = {}", proxy_port));
    }
    if !set_no {
        res.push(format!("no_proxy = {}", no_proxy_def));
    }
    for line in lines {
        if !line.trim().is_empty() {
            res.push(line);
        }
    }
    res.join("\n") + "\n"
}

pub fn remove_puppet_agent_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        let lower = trim.to_ascii_lowercase();
        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy_host"
                    || k_low == "http_proxy_port"
                    || k_low == "puppet_proxy_host"
                    || k_low == "puppet_proxy_port"
                    || k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "all_proxy"
                    || k_low == "no_proxy"
                {
                    changed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trim.split_once('=') {
            let key = k.trim().to_lowercase();
            if key == "http_proxy_host"
                || key == "http_proxy_port"
                || key == "puppet_proxy_host"
                || key == "puppet_proxy_port"
                || key == "http_proxy_user"
                || key == "http_proxy_password"
                || key == "http_proxy"
                || key == "https_proxy"
                || key == "all_proxy"
                || key == "no_proxy"
            {
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", changed)
}

pub fn set_puppet_agent_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_puppet_agent_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_puppet_agent_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_puppet_agent_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_puppet_agent_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_puppet_agent_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_chef_client_proxy(content: &str, proxy_url: &str) -> String {
    let no_proxy_def = "localhost,127.0.0.1,.local";

    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // Ruby DSL format: http_proxy "...", https_proxy "...", no_proxy "..."
    let mut lines = Vec::new();
    let mut replaced_http = false;
    let mut replaced_https = false;
    let mut replaced_no = false;

    for line in content.lines() {
        let trim = line.trim();
        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("http_proxy") && !lower.starts_with("http_proxy_") {
            lines.push(format!("http_proxy \"{}\"", proxy_url));
            replaced_http = true;
            continue;
        } else if lower.starts_with("https_proxy") && !lower.starts_with("https_proxy_") {
            lines.push(format!("https_proxy \"{}\"", proxy_url));
            replaced_https = true;
            continue;
        } else if lower.starts_with("no_proxy") {
            lines.push(format!("no_proxy \"{}\"", no_proxy_def));
            replaced_no = true;
            continue;
        }
        lines.push(line.to_string());
    }

    if !replaced_http {
        lines.push(format!("http_proxy \"{}\"", proxy_url));
    }
    if !replaced_https {
        lines.push(format!("https_proxy \"{}\"", proxy_url));
    }
    if !replaced_no {
        lines.push(format!("no_proxy \"{}\"", no_proxy_def));
    }

    lines.join("\n") + "\n"
}

pub fn remove_chef_client_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut removed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') || trim.starts_with("//") {
            lines.push(line.to_string());
            continue;
        }

        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" || k_low == "https_proxy" || k_low == "no_proxy" || k_low == "all_proxy" {
                    removed = true;
                    continue;
                }
            }
        }

        if (lower.starts_with("http_proxy") && !lower.starts_with("http_proxy_"))
            || (lower.starts_with("https_proxy") && !lower.starts_with("https_proxy_"))
            || lower.starts_with("no_proxy")
            || lower.starts_with("all_proxy")
        {
            removed = true;
            continue;
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_chef_client_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_chef_client_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_chef_client_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_chef_client_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_chef_client_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_chef_client_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_salt_minion_proxy(content: &str, proxy_url: &str) -> String {
    let (proxy_host, proxy_port) = extract_host_port(proxy_url);
    let no_proxy_def = "localhost,127.0.0.1,.local";

    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_host = false;
        let mut replaced_port = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "proxy_host" || k_low == "salt_proxy_host" {
                        lines.push(format!("Environment=\"PROXY_HOST={}\"", proxy_host));
                        replaced_host = true;
                        continue;
                    } else if k_low == "proxy_port" || k_low == "salt_proxy_port" {
                        lines.push(format!("Environment=\"PROXY_PORT={}\"", proxy_port));
                        replaced_port = true;
                        continue;
                    } else if k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_host {
            lines.push(format!("Environment=\"PROXY_HOST={}\"", proxy_host));
        }
        if !replaced_port {
            lines.push(format!("Environment=\"PROXY_PORT={}\"", proxy_port));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // YAML key: val format in minion / minion.d/proxy.conf
    let mut lines = Vec::new();
    let mut set_host = false;
    let mut set_port = false;
    let mut set_no = false;

    for line in content.lines() {
        let trim = line.trim();
        let pair = if let Some((k, v)) = trim.split_once(':') {
            Some((k, v))
        } else if let Some((k, v)) = trim.split_once('=') {
            Some((k, v))
        } else {
            None
        };

        if let Some((k, _)) = pair {
            let key = k.trim().to_lowercase();
            if key == "proxy_host" || key == "salt_proxy_host" {
                lines.push(format!("proxy_host: {}", proxy_host));
                set_host = true;
                continue;
            } else if key == "proxy_port" || key == "salt_proxy_port" {
                lines.push(format!("proxy_port: {}", proxy_port));
                set_port = true;
                continue;
            } else if key == "no_proxy" {
                lines.push(format!("no_proxy: {}", no_proxy_def));
                set_no = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !set_host {
        lines.push(format!("proxy_host: {}", proxy_host));
    }
    if !set_port {
        lines.push(format!("proxy_port: {}", proxy_port));
    }
    if !set_no {
        lines.push(format!("no_proxy: {}", no_proxy_def));
    }

    lines.join("\n") + "\n"
}

pub fn remove_salt_minion_proxy(content: &str) -> (String, bool) {
    let mut lines = Vec::new();
    let mut removed = false;

    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('#') || trim.starts_with(';') {
            lines.push(line.to_string());
            continue;
        }

        let lower = trim.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trim["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy" || k_low == "https_proxy" || k_low == "proxy_host" || k_low == "proxy_port" || k_low == "no_proxy" || k_low == "all_proxy" {
                    removed = true;
                    continue;
                }
            }
        }

        let pair = if let Some((k, v)) = trim.split_once(':') {
            Some((k, v))
        } else if let Some((k, v)) = trim.split_once('=') {
            Some((k, v))
        } else {
            None
        };

        if let Some((k, _)) = pair {
            let key = k.trim().to_lowercase();
            if key == "proxy_host" || key == "salt_proxy_host" || key == "proxy_port" || key == "salt_proxy_port" || key == "http_proxy" || key == "https_proxy" || key == "no_proxy" || key == "all_proxy" {
                removed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_salt_minion_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_salt_minion_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_salt_minion_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_salt_minion_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_salt_minion_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_salt_minion_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_datadog_agent_proxy(content: &str, proxy_url: &str) -> String {
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let no_proxy_def = "localhost,127.0.0.1,.local";
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_dd_http = false;
        let mut replaced_dd_https = false;
        let mut replaced_no = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "dd_proxy_http" {
                        lines.push(format!("Environment=\"DD_PROXY_HTTP={}\"", proxy_url));
                        replaced_dd_http = true;
                        continue;
                    } else if k_low == "dd_proxy_https" {
                        lines.push(format!("Environment=\"DD_PROXY_HTTPS={}\"", proxy_url));
                        replaced_dd_https = true;
                        continue;
                    } else if k_low == "no_proxy" {
                        lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
                        replaced_no = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_dd_http {
            lines.push(format!("Environment=\"DD_PROXY_HTTP={}\"", proxy_url));
        }
        if !replaced_dd_https {
            lines.push(format!("Environment=\"DD_PROXY_HTTPS={}\"", proxy_url));
        }
        if !replaced_no {
            lines.push(format!("Environment=\"NO_PROXY={}\"", no_proxy_def));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // YAML format in datadog.yaml
    let mut lines = Vec::new();
    let mut in_proxy_block = false;
    let mut proxy_indent = 0;
    let mut replaced_proxy = false;

    let proxy_block = format!(
        "proxy:\n  http: {}\n  https: {}\n  no_proxy:\n    - localhost\n    - 127.0.0.1\n    - .local",
        proxy_url, proxy_url
    );

    for line in content.lines() {
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();

        if (trimmed == "proxy:" || trimmed.starts_with("proxy:")) && !in_proxy_block {
            in_proxy_block = true;
            proxy_indent = indent;
            lines.push(proxy_block.clone());
            replaced_proxy = true;
            continue;
        }

        if in_proxy_block {
            if (indent > proxy_indent) || trimmed.starts_with('-') {
                continue;
            } else {
                in_proxy_block = false;
            }
        }

        lines.push(line.to_string());
    }

    if !replaced_proxy {
        if !lines.is_empty() && !lines.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
            lines.push(String::new());
        }
        lines.push(proxy_block);
    }

    lines.join("\n") + "\n"
}

pub fn remove_datadog_agent_proxy(content: &str) -> (String, bool) {
    let mut removed = false;
    let mut lines = Vec::new();
    let mut in_proxy_block = false;
    let mut proxy_indent = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trimmed["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "dd_proxy_http"
                    || k_low == "dd_proxy_https"
                    || k_low == "no_proxy"
                {
                    removed = true;
                    continue;
                }
            }
        }

        if lower.starts_with("dd_proxy_http=")
            || lower.starts_with("dd_proxy_https=")
            || lower.starts_with("http_proxy=")
            || lower.starts_with("https_proxy=")
        {
            removed = true;
            continue;
        }

        let indent = line.len() - line.trim_start().len();
        if (trimmed == "proxy:" || trimmed.starts_with("proxy:")) && !in_proxy_block {
            in_proxy_block = true;
            proxy_indent = indent;
            removed = true;
            continue;
        }

        if in_proxy_block {
            if (indent > proxy_indent) || trimmed.starts_with('-') {
                removed = true;
                continue;
            } else {
                in_proxy_block = false;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_datadog_agent_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_datadog_agent_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_datadog_agent_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_datadog_agent_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_datadog_agent_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_datadog_agent_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_newrelic_infra_proxy(content: &str, proxy_url: &str) -> String {
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_nria = false;

        for line in content.lines() {
            let trim = line.trim();
            let lower = trim.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trim["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "nria_proxy" {
                        lines.push(format!("Environment=\"NRIA_PROXY={}\"", proxy_url));
                        replaced_nria = true;
                        continue;
                    }
                }
            }
            lines.push(line.to_string());
        }

        if !replaced_nria {
            lines.push(format!("Environment=\"NRIA_PROXY={}\"", proxy_url));
        }
        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }

        let mut res = lines.join("\n");
        if !has_service_section {
            res = format!("[Service]\n{}", res);
        }
        return res + "\n";
    }

    // YAML format in newrelic-infra.yml
    let mut lines = Vec::new();
    let mut replaced_proxy = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some((k, _)) = trimmed.split_once(':') {
            if k.trim().trim_matches('"').trim_matches('\'').eq_ignore_ascii_case("proxy") {
                lines.push(format!("proxy: {}", proxy_url));
                replaced_proxy = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !replaced_proxy {
        if !lines.is_empty() && !lines.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
            lines.push(String::new());
        }
        lines.push(format!("proxy: {}", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_newrelic_infra_proxy(content: &str) -> (String, bool) {
    let mut removed = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trimmed["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "nria_proxy"
                {
                    removed = true;
                    continue;
                }
            }
        }

        if lower.starts_with("nria_proxy=")
            || lower.starts_with("http_proxy=")
            || lower.starts_with("https_proxy=")
        {
            removed = true;
            continue;
        }

        if let Some((k, _)) = trimmed.split_once(':') {
            if k.trim().trim_matches('"').trim_matches('\'').eq_ignore_ascii_case("proxy") {
                removed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_newrelic_infra_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_newrelic_infra_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_newrelic_infra_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_newrelic_infra_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_newrelic_infra_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_newrelic_infra_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_elastic_agent_proxy(content: &str, proxy_url: &str) -> String {
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_ea = false;

        for line in content.lines() {
            let trimmed = line.trim();
            let lower = trimmed.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trimmed["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "elastic_agent_proxy" {
                        lines.push(format!("Environment=\"ELASTIC_AGENT_PROXY={}\"", proxy_url));
                        replaced_ea = true;
                        continue;
                    }
                }
            } else if lower.starts_with("http_proxy=") {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if lower.starts_with("https_proxy=") {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if lower.starts_with("elastic_agent_proxy=") {
                lines.push(format!("ELASTIC_AGENT_PROXY=\"{}\"", proxy_url));
                replaced_ea = true;
                continue;
            }

            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_ea {
            lines.push(format!("Environment=\"ELASTIC_AGENT_PROXY={}\"", proxy_url));
        }

        return lines.join("\n") + "\n";
    }

    // YAML format in elastic-agent.yml
    let mut lines = Vec::new();
    let mut replaced_proxy = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some((k, _)) = trimmed.split_once(':') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_ascii_lowercase();
            if key == "proxy_url" {
                let indent = line.len() - line.trim_start().len();
                lines.push(format!("{:indent$}proxy_url: {}", "", proxy_url, indent = indent));
                replaced_proxy = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !replaced_proxy {
        if !lines.is_empty() && !lines.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
            lines.push(String::new());
        }
        lines.push(format!("proxy_url: {}", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_elastic_agent_proxy(content: &str) -> (String, bool) {
    let mut removed = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trimmed["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "elastic_agent_proxy"
                {
                    removed = true;
                    continue;
                }
            }
        }

        if lower.starts_with("elastic_agent_proxy=")
            || lower.starts_with("http_proxy=")
            || lower.starts_with("https_proxy=")
        {
            removed = true;
            continue;
        }

        if let Some((k, _)) = trimmed.split_once(':') {
            if k.trim().trim_matches('"').trim_matches('\'').eq_ignore_ascii_case("proxy_url") {
                removed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_elastic_agent_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_elastic_agent_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_elastic_agent_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_elastic_agent_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_elastic_agent_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_elastic_agent_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_grafana_alloy_proxy(content: &str, proxy_url: &str) -> String {
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_alloy = false;

        for line in content.lines() {
            let trimmed = line.trim();
            let lower = trimmed.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trimmed["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "alloy_http_proxy" {
                        lines.push(format!("Environment=\"ALLOY_HTTP_PROXY={}\"", proxy_url));
                        replaced_alloy = true;
                        continue;
                    }
                }
            } else if lower.starts_with("http_proxy=") {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if lower.starts_with("https_proxy=") {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if lower.starts_with("alloy_http_proxy=") {
                lines.push(format!("ALLOY_HTTP_PROXY=\"{}\"", proxy_url));
                replaced_alloy = true;
                continue;
            }

            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_alloy {
            lines.push(format!("Environment=\"ALLOY_HTTP_PROXY={}\"", proxy_url));
        }

        return lines.join("\n") + "\n";
    }

    // River / Alloy format or YAML
    let mut lines = Vec::new();
    let mut replaced_proxy = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some((k, _)) = trimmed.split_once('=') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_ascii_lowercase();
            if key == "proxy_url" {
                let indent = line.len() - line.trim_start().len();
                lines.push(format!("{:indent$}proxy_url = \"{}\"", "", proxy_url, indent = indent));
                replaced_proxy = true;
                continue;
            }
        } else if let Some((k, _)) = trimmed.split_once(':') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_ascii_lowercase();
            if key == "proxy_url" {
                let indent = line.len() - line.trim_start().len();
                lines.push(format!("{:indent$}proxy_url: \"{}\"", "", proxy_url, indent = indent));
                replaced_proxy = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !replaced_proxy {
        if !lines.is_empty() && !lines.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
            lines.push(String::new());
        }
        lines.push(format!("proxy_url = \"{}\"", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_grafana_alloy_proxy(content: &str) -> (String, bool) {
    let mut removed = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trimmed["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "alloy_http_proxy"
                {
                    removed = true;
                    continue;
                }
            }
        }

        if lower.starts_with("alloy_http_proxy=")
            || lower.starts_with("http_proxy=")
            || lower.starts_with("https_proxy=")
        {
            removed = true;
            continue;
        }

        if let Some((k, _)) = trimmed.split_once('=') {
            if k.trim().trim_matches('"').trim_matches('\'').eq_ignore_ascii_case("proxy_url") {
                removed = true;
                continue;
            }
        }
        if let Some((k, _)) = trimmed.split_once(':') {
            if k.trim().trim_matches('"').trim_matches('\'').eq_ignore_ascii_case("proxy_url") {
                removed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_grafana_alloy_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_grafana_alloy_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_grafana_alloy_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_grafana_alloy_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_grafana_alloy_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_grafana_alloy_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_otelcol_proxy(content: &str, proxy_url: &str) -> String {
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_otel = false;

        for line in content.lines() {
            let trimmed = line.trim();
            let lower = trimmed.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trimmed["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "otel_exporter_otlp_proxy_url" {
                        lines.push(format!("Environment=\"OTEL_EXPORTER_OTLP_PROXY_URL={}\"", proxy_url));
                        replaced_otel = true;
                        continue;
                    }
                }
            } else if lower.starts_with("http_proxy=") {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if lower.starts_with("https_proxy=") {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if lower.starts_with("otel_exporter_otlp_proxy_url=") {
                lines.push(format!("OTEL_EXPORTER_OTLP_PROXY_URL=\"{}\"", proxy_url));
                replaced_otel = true;
                continue;
            } else if lower.starts_with("otelcol_http_proxy=") {
                lines.push(format!("OTELCOL_HTTP_PROXY=\"{}\"", proxy_url));
                replaced_otel = true;
                continue;
            }

            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_otel {
            lines.push(format!("Environment=\"OTEL_EXPORTER_OTLP_PROXY_URL={}\"", proxy_url));
        }

        return lines.join("\n") + "\n";
    }

    // YAML format in config.yaml
    let mut lines = Vec::new();
    let mut replaced_proxy = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some((k, _)) = trimmed.split_once(':') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_ascii_lowercase();
            if key == "proxy_url" || key == "http_proxy" {
                let indent = line.len() - line.trim_start().len();
                lines.push(format!("{:indent$}proxy_url: \"{}\"", "", proxy_url, indent = indent));
                replaced_proxy = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !replaced_proxy {
        if !lines.is_empty() && !lines.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
            lines.push(String::new());
        }
        lines.push(format!("proxy_url: \"{}\"", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_otelcol_proxy(content: &str) -> (String, bool) {
    let mut removed = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trimmed["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "otel_exporter_otlp_proxy_url"
                    || k_low == "otelcol_http_proxy"
                {
                    removed = true;
                    continue;
                }
            }
        }

        if lower.starts_with("otel_exporter_otlp_proxy_url=")
            || lower.starts_with("otelcol_http_proxy=")
            || lower.starts_with("http_proxy=")
            || lower.starts_with("https_proxy=")
        {
            removed = true;
            continue;
        }

        if let Some((k, _)) = trimmed.split_once(':') {
            let k_low = k.trim().trim_matches('"').trim_matches('\'').to_ascii_lowercase();
            if k_low == "proxy_url" || k_low == "http_proxy" || k_low == "https_proxy" || k_low == "endpoint_proxy" {
                removed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_otelcol_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_otelcol_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_otelcol_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_otelcol_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_otelcol_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_otelcol_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_vector_proxy(content: &str, proxy_url: &str) -> String {
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_vec = false;

        for line in content.lines() {
            let trimmed = line.trim();
            let lower = trimmed.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trimmed["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "vector_proxy" {
                        lines.push(format!("Environment=\"VECTOR_PROXY={}\"", proxy_url));
                        replaced_vec = true;
                        continue;
                    }
                }
            } else if lower.starts_with("http_proxy=") {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if lower.starts_with("https_proxy=") {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if lower.starts_with("vector_proxy=") {
                lines.push(format!("VECTOR_PROXY=\"{}\"", proxy_url));
                replaced_vec = true;
                continue;
            }

            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_vec {
            lines.push(format!("Environment=\"VECTOR_PROXY={}\"", proxy_url));
        }

        return lines.join("\n") + "\n";
    }

    // TOML format
    let has_toml_header = content.lines().any(|l| {
        let t = l.trim();
        t.starts_with('[') && t.ends_with(']')
    });

    if has_toml_header {
        let mut lines = Vec::new();
        let mut in_proxy = false;
        let mut replaced_http = false;
        let mut replaced_https = false;
        let has_proxy_section = content.contains("[proxy]");

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[proxy]" || trimmed.starts_with("[proxy]") {
                in_proxy = true;
                lines.push(line.to_string());
                continue;
            } else if trimmed.starts_with('[') {
                if in_proxy {
                    if !replaced_http {
                        lines.push(format!("http = \"{}\"", proxy_url));
                    }
                    if !replaced_https {
                        lines.push(format!("https = \"{}\"", proxy_url));
                    }
                    in_proxy = false;
                }
                lines.push(line.to_string());
                continue;
            }

            if in_proxy {
                if let Some((k, _)) = trimmed.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http" {
                        lines.push(format!("http = \"{}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https" {
                        lines.push(format!("https = \"{}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    }
                }
            }

            lines.push(line.to_string());
        }

        if in_proxy {
            if !replaced_http {
                lines.push(format!("http = \"{}\"", proxy_url));
            }
            if !replaced_https {
                lines.push(format!("https = \"{}\"", proxy_url));
            }
        } else if !has_proxy_section {
            if !lines.is_empty() && !lines.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
                lines.push(String::new());
            }
            lines.push("[proxy]".to_string());
            lines.push(format!("http = \"{}\"", proxy_url));
            lines.push(format!("https = \"{}\"", proxy_url));
        }

        return lines.join("\n") + "\n";
    }

    // YAML format in vector.yaml
    let mut lines = Vec::new();
    let mut replaced_proxy = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some((k, _)) = trimmed.split_once(':') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_ascii_lowercase();
            if key == "proxy_url" {
                let indent = line.len() - line.trim_start().len();
                lines.push(format!("{:indent$}proxy_url: \"{}\"", "", proxy_url, indent = indent));
                replaced_proxy = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !replaced_proxy {
        if !lines.is_empty() && !lines.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
            lines.push(String::new());
        }
        lines.push("proxy:".to_string());
        lines.push(format!("  http: \"{}\"", proxy_url));
        lines.push(format!("  https: \"{}\"", proxy_url));
    }

    lines.join("\n") + "\n"
}

pub fn remove_vector_proxy(content: &str) -> (String, bool) {
    let mut removed = false;
    let mut lines = Vec::new();
    let mut in_proxy_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trimmed["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "vector_proxy"
                {
                    removed = true;
                    continue;
                }
            }
        }

        if lower.starts_with("vector_proxy=")
            || lower.starts_with("http_proxy=")
            || lower.starts_with("https_proxy=")
        {
            removed = true;
            continue;
        }

        if trimmed == "[proxy]" || trimmed.starts_with("[proxy]") {
            in_proxy_block = true;
            removed = true;
            continue;
        } else if in_proxy_block && trimmed.starts_with('[') {
            in_proxy_block = false;
        }

        if in_proxy_block {
            removed = true;
            continue;
        }

        if trimmed == "proxy:" || trimmed.starts_with("proxy:") {
            removed = true;
            continue;
        }

        if let Some((k, _)) = trimmed.split_once(':') {
            let k_low = k.trim().trim_matches('"').trim_matches('\'').to_ascii_lowercase();
            if k_low == "proxy_url" || k_low == "http" || k_low == "https" {
                if trimmed.contains("http://") || trimmed.contains("https://") || trimmed.contains("socks") {
                    removed = true;
                    continue;
                }
            }
        }

        if let Some((k, _)) = trimmed.split_once('=') {
            let k_low = k.trim().trim_matches('"').trim_matches('\'').to_ascii_lowercase();
            if k_low == "proxy.http" || k_low == "proxy.https" || k_low == "proxy_url" {
                removed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_vector_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_vector_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_vector_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_vector_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_vector_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_vector_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_fluent_bit_proxy(content: &str, proxy_url: &str) -> String {
    let has_systemd = content.lines().any(|l| {
        let lower = l.trim().to_ascii_lowercase();
        lower.starts_with("environment=")
            || lower.starts_with("execstart=")
            || lower == "[unit]"
            || lower == "[install]"
    });

    if has_systemd {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_flb = false;

        for line in content.lines() {
            let trimmed = line.trim();
            let lower = trimmed.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trimmed["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "flb_http_proxy" {
                        lines.push(format!("Environment=\"FLB_HTTP_PROXY={}\"", proxy_url));
                        replaced_flb = true;
                        continue;
                    }
                }
            } else if lower.starts_with("http_proxy=") {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if lower.starts_with("https_proxy=") {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if lower.starts_with("flb_http_proxy=") {
                lines.push(format!("FLB_HTTP_PROXY=\"{}\"", proxy_url));
                replaced_flb = true;
                continue;
            }

            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_flb {
            lines.push(format!("Environment=\"FLB_HTTP_PROXY={}\"", proxy_url));
        }

        return lines.join("\n") + "\n";
    }

    // Classic fluent-bit conf format: check if [SERVICE] exists
    let has_flb_service = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[SERVICE]"));
    if has_flb_service {
        let mut lines = Vec::new();
        let mut in_service = false;
        let mut replaced = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case("[SERVICE]") {
                in_service = true;
                lines.push(line.to_string());
                continue;
            } else if in_service && trimmed.starts_with('[') {
                if !replaced {
                    lines.push(format!("    HTTP_Proxy  {}", proxy_url));
                    replaced = true;
                }
                in_service = false;
                lines.push(line.to_string());
                continue;
            }

            if in_service {
                let first_token = trimmed.split_whitespace().next().unwrap_or("").to_lowercase();
                if first_token == "http_proxy" || first_token == "proxy" {
                    lines.push(format!("    HTTP_Proxy  {}", proxy_url));
                    replaced = true;
                    continue;
                }
            }

            lines.push(line.to_string());
        }

        if in_service && !replaced {
            lines.push(format!("    HTTP_Proxy  {}", proxy_url));
        }

        return lines.join("\n") + "\n";
    }

    // YAML format: check if yaml keys exist
    if content.lines().any(|l| l.trim().starts_with("service:") || l.trim().starts_with("pipeline:")) {
        let mut lines = Vec::new();
        let mut replaced = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if let Some((k, _)) = trimmed.split_once(':') {
                let key = k.trim().trim_matches('"').trim_matches('\'').to_ascii_lowercase();
                if key == "http_proxy" || key == "proxy" {
                    let indent = line.len() - line.trim_start().len();
                    lines.push(format!("{:indent$}http_proxy: \"{}\"", "", proxy_url, indent = indent));
                    replaced = true;
                    continue;
                }
            }
            lines.push(line.to_string());
        }

        if !replaced {
            lines.push("service:".to_string());
            lines.push(format!("  http_proxy: \"{}\"", proxy_url));
        }

        return lines.join("\n") + "\n";
    }

    // Default classic .conf format
    let mut lines = Vec::new();
    lines.push("[SERVICE]".to_string());
    lines.push(format!("    HTTP_Proxy  {}", proxy_url));
    if !content.trim().is_empty() {
        lines.push(String::new());
        lines.push(content.trim().to_string());
    }
    lines.join("\n") + "\n"
}

pub fn remove_fluent_bit_proxy(content: &str) -> (String, bool) {
    let mut removed = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trimmed["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "flb_http_proxy"
                    || k_low == "fluent_bit_http_proxy"
                {
                    removed = true;
                    continue;
                }
            }
        }

        if lower.starts_with("flb_http_proxy=")
            || lower.starts_with("fluent_bit_http_proxy=")
            || lower.starts_with("http_proxy=")
            || lower.starts_with("https_proxy=")
        {
            removed = true;
            continue;
        }

        let first_token = trimmed.split_whitespace().next().unwrap_or("").to_lowercase();
        if first_token == "http_proxy" || first_token == "proxy" {
            removed = true;
            continue;
        }

        if let Some((k, _)) = trimmed.split_once(':') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_ascii_lowercase();
            if key == "http_proxy" || key == "proxy" {
                if trimmed.contains("http://") || trimmed.contains("https://") || trimmed.contains("socks") {
                    removed = true;
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_fluent_bit_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_fluent_bit_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_fluent_bit_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_fluent_bit_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_fluent_bit_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_fluent_bit_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
}

pub fn apply_fluentd_proxy(content: &str, proxy_url: &str) -> String {
    let has_service_section = content.lines().any(|l| l.trim().eq_ignore_ascii_case("[Service]"));
    let has_environment = content.lines().any(|l| l.trim().to_ascii_lowercase().starts_with("environment="));

    if has_service_section || has_environment {
        let mut lines = Vec::new();
        let mut replaced_http = false;
        let mut replaced_https = false;
        let mut replaced_fluent = false;

        for line in content.lines() {
            let trimmed = line.trim();
            let lower = trimmed.to_ascii_lowercase();

            if lower.starts_with("environment=") {
                let env_part = trimmed["environment=".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if let Some((k, _)) = env_part.split_once('=') {
                    let k_low = k.trim().to_ascii_lowercase();
                    if k_low == "http_proxy" {
                        lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
                        replaced_http = true;
                        continue;
                    } else if k_low == "https_proxy" {
                        lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
                        replaced_https = true;
                        continue;
                    } else if k_low == "fluentd_http_proxy" || k_low == "td_agent_http_proxy" {
                        lines.push(format!("Environment=\"FLUENTD_HTTP_PROXY={}\"", proxy_url));
                        replaced_fluent = true;
                        continue;
                    }
                }
            } else if lower.starts_with("http_proxy=") {
                lines.push(format!("HTTP_PROXY=\"{}\"", proxy_url));
                replaced_http = true;
                continue;
            } else if lower.starts_with("https_proxy=") {
                lines.push(format!("HTTPS_PROXY=\"{}\"", proxy_url));
                replaced_https = true;
                continue;
            } else if lower.starts_with("fluentd_http_proxy=") || lower.starts_with("td_agent_http_proxy=") {
                lines.push(format!("FLUENTD_HTTP_PROXY=\"{}\"", proxy_url));
                replaced_fluent = true;
                continue;
            }

            lines.push(line.to_string());
        }

        if !replaced_http {
            lines.push(format!("Environment=\"HTTP_PROXY={}\"", proxy_url));
        }
        if !replaced_https {
            lines.push(format!("Environment=\"HTTPS_PROXY={}\"", proxy_url));
        }
        if !replaced_fluent {
            lines.push(format!("Environment=\"FLUENTD_HTTP_PROXY={}\"", proxy_url));
        }

        return lines.join("\n") + "\n";
    }

    // In fluent.conf
    let has_system_tag = content.lines().any(|l| l.trim().eq_ignore_ascii_case("<system>"));
    if has_system_tag {
        let mut lines = Vec::new();
        let mut in_system = false;
        let mut replaced = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case("<system>") {
                in_system = true;
                lines.push(line.to_string());
                continue;
            } else if in_system && trimmed.eq_ignore_ascii_case("</system>") {
                if !replaced {
                    lines.push(format!("  http_proxy {}", proxy_url));
                    replaced = true;
                }
                in_system = false;
                lines.push(line.to_string());
                continue;
            }

            if in_system {
                let first_token = trimmed.split_whitespace().next().unwrap_or("").to_lowercase();
                if first_token == "http_proxy" {
                    lines.push(format!("  http_proxy {}", proxy_url));
                    replaced = true;
                    continue;
                }
            }

            lines.push(line.to_string());
        }

        return lines.join("\n") + "\n";
    }

    // No <system> tag found: prepend <system>\n  http_proxy ...\n</system>
    let mut lines = Vec::new();
    lines.push("<system>".to_string());
    lines.push(format!("  http_proxy {}", proxy_url));
    lines.push("</system>".to_string());
    if !content.trim().is_empty() {
        lines.push(String::new());
        lines.push(content.trim().to_string());
    }
    lines.join("\n") + "\n"
}

pub fn remove_fluentd_proxy(content: &str) -> (String, bool) {
    let mut removed = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("environment=") {
            let env_part = trimmed["environment=".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((k, _)) = env_part.split_once('=') {
                let k_low = k.trim().to_ascii_lowercase();
                if k_low == "http_proxy"
                    || k_low == "https_proxy"
                    || k_low == "fluentd_http_proxy"
                    || k_low == "td_agent_http_proxy"
                {
                    removed = true;
                    continue;
                }
            }
        }

        if lower.starts_with("fluentd_http_proxy=")
            || lower.starts_with("td_agent_http_proxy=")
            || lower.starts_with("http_proxy=")
            || lower.starts_with("https_proxy=")
        {
            removed = true;
            continue;
        }

        let first_token = trimmed.split_whitespace().next().unwrap_or("").to_lowercase();
        if first_token == "http_proxy" {
            removed = true;
            continue;
        }

        if let Some((k, _)) = trimmed.split_once(':') {
            let key = k.trim().trim_matches('"').trim_matches('\'').to_ascii_lowercase();
            if key == "http_proxy" {
                removed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    (lines.join("\n") + "\n", removed)
}

pub fn set_fluentd_proxy(proxy_url: &str) -> Result<PathBuf> {
    let path = crate::doctor::get_fluentd_write_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = apply_fluentd_proxy(&content, proxy_url);
    std::fs::write(&path, new_content)?;

    #[cfg(target_os = "linux")]
    if path.to_string_lossy().contains("systemd") {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(path)
}

pub fn unset_fluentd_proxy() -> Result<bool> {
    let mut any_removed = false;
    for path in crate::doctor::get_fluentd_config_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let (new_content, removed) = remove_fluentd_proxy(&content);
                if removed {
                    let _ = std::fs::write(&path, new_content);
                    any_removed = true;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if any_removed {
        let _ = Command::new("systemctl").arg("daemon-reload").status();
    }
    Ok(any_removed)
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
    luarocks: bool,
    nimble: bool,
    rebar3: bool,
    scoop: bool,
    bundle: bool,
    gcloud: bool,
    aws: bool,
    azure: bool,
    mamba: bool,
    podman: bool,
    vagrant: bool,
    choco: bool,
    colima: bool,
    minikube: bool,
    spack: bool,
    macports: bool,
    apt: bool,
    dnf: bool,
    zypper: bool,
    pacman: bool,
    pkg: bool,
    portage: bool,
    docker_daemon: bool,
    flatpak: bool,
    snap: bool,
    ollama: bool,
    containerd: bool,
    buildkit: bool,
    k3s: bool,
    microk8s: bool,
    k0s: bool,
    rke2: bool,
    nomad: bool,
    kubelet: bool,
    gitlab_runner: bool,
    actions_runner: bool,
    jenkins: bool,
    buildkite_agent: bool,
    woodpecker_agent: bool,
    drone_runner: bool,
    act_runner: bool,
    circleci_runner: bool,
    azure_pipelines_agent: bool,
    teamcity_agent: bool,
    bamboo_agent: bool,
    tfc_agent: bool,
    concourse_worker: bool,
    tentacle_agent: bool,
    puppet_agent: bool,
    chef_client: bool,
    salt_minion: bool,
    datadog_agent: bool,
    newrelic_infra: bool,
    elastic_agent: bool,
    grafana_alloy: bool,
    otelcol: bool,
    vector: bool,
    fluent_bit: bool,
    fluentd: bool,
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
    let set_luarocks = luarocks || all;
    let set_nimble = nimble || all;
    let set_rebar3 = rebar3 || all;
    let set_scoop = scoop || all;
    let set_bundle = bundle || all;
    let set_gcloud = gcloud || all;
    let set_aws = aws || all;
    let set_azure = azure || all;
    let set_mamba = mamba || all;
    let set_podman = podman || all;
    let set_vagrant = vagrant || all;
    let set_choco = choco || all;
    let set_colima = colima || all;
    let set_minikube = minikube || all;
    let set_spack = spack || all;
    let set_macports = macports || all;
    let set_apt = apt || all;
    let set_dnf = dnf || all;
    let set_zypper = zypper || all;
    let set_pacman = pacman || all;
    let set_pkg = pkg || all;
    let set_portage = portage || all;
    let set_docker_daemon = docker_daemon || all;
    let set_flatpak = flatpak || all;
    let set_snap = snap || all;
    let set_ollama = ollama || all;
    let set_containerd = containerd || all;
    let set_buildkit = buildkit || all;
    let set_k3s = k3s || all;
    let set_microk8s = microk8s || all;
    let set_k0s = k0s || all;
    let set_rke2 = rke2 || all;
    let set_nomad = nomad || all;
    let set_kubelet = kubelet || all;
    let set_gitlab_runner = gitlab_runner || all;
    let set_actions_runner = actions_runner || all;
    let set_jenkins = jenkins || all;
    let set_buildkite_agent = buildkite_agent || all;
    let set_woodpecker_agent = woodpecker_agent || all;
    let set_drone_runner = drone_runner || all;
    let set_act_runner = act_runner || all;
    let set_circleci_runner = circleci_runner || all;
    let set_azure_pipelines_agent = azure_pipelines_agent || all;
    let set_teamcity_agent = teamcity_agent || all;
    let set_bamboo_agent = bamboo_agent || all;
    let set_tfc_agent = tfc_agent || all;
    let set_concourse_worker = concourse_worker || all;
    let set_tentacle_agent = tentacle_agent || all;
    let set_puppet_agent = puppet_agent || all;
    let set_chef_client = chef_client || all;
    let set_salt_minion = salt_minion || all;
    let set_datadog_agent = datadog_agent || all;
    let set_newrelic_infra = newrelic_infra || all;
    let set_elastic_agent = elastic_agent || all;
    let set_grafana_alloy = grafana_alloy || all;
    let set_otelcol = otelcol || all;
    let set_vector = vector || all;
    let set_fluent_bit = fluent_bit || all;
    let set_fluentd = fluentd || all;
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

    if set_luarocks {
        match set_luarocks_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ LuaRocks proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update LuaRocks config.lua: {}", "▲".yellow(), e),
        }
    }

    if set_nimble {
        match set_nimble_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Nimble proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Nimble nimble.ini: {}", "▲".yellow(), e),
        }
    }

    if set_rebar3 {
        match set_rebar3_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Rebar3 proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Rebar3 rebar.config: {}", "▲".yellow(), e),
        }
    }

    if set_scoop {
        match set_scoop_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Scoop proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Scoop config.json: {}", "▲".yellow(), e),
        }
    }

    if set_bundle {
        match set_bundler_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ Ruby Bundler proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update Bundler config: {}", "▲".yellow(), e),
        }
    }

    if set_gcloud {
        match set_gcloud_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Google Cloud SDK (gcloud) proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!(
                "{} Failed to update Google Cloud SDK (gcloud) config: {}",
                "▲".yellow(),
                e
            ),
        }
    }

    if set_aws {
        match set_aws_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!("✔ AWS CLI proxy ({}) set to {}", p.display(), proxy_url).green()
            ),
            Err(e) => eprintln!("{} Failed to update AWS CLI config: {}", "▲".yellow(), e),
        }
    }

    if set_azure {
        match set_azure_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Microsoft Azure CLI (az) proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!(
                "{} Failed to update Microsoft Azure CLI config: {}",
                "▲".yellow(),
                e
            ),
        }
    }

    if set_mamba {
        match set_mamba_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Mamba / Micromamba proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Mamba config: {}", "▲".yellow(), e),
        }
    }

    if set_podman {
        match set_podman_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Podman container engine proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Podman config: {}", "▲".yellow(), e),
        }
    }

    if set_vagrant {
        match set_vagrant_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Vagrant proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Vagrant config: {}", "▲".yellow(), e),
        }
    }

    if set_choco {
        match set_choco_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Chocolatey package manager proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Chocolatey config: {}", "▲".yellow(), e),
        }
    }

    if set_colima {
        match set_colima_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Colima container runtime proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Colima config: {}", "▲".yellow(), e),
        }
    }

    if set_minikube {
        match set_minikube_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Minikube cluster proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Minikube config: {}", "▲".yellow(), e),
        }
    }

    if set_spack {
        match set_spack_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Spack package manager proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Spack config: {}", "▲".yellow(), e),
        }
    }

    if set_macports {
        match set_macports_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ MacPorts package manager proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update MacPorts config: {}", "▲".yellow(), e),
        }
    }

    if set_apt {
        match set_apt_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ APT package manager proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update APT config: {}", "▲".yellow(), e),
        }
    }

    if set_dnf {
        match set_dnf_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ DNF & YUM package manager proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update DNF config: {}", "▲".yellow(), e),
        }
    }

    if set_zypper {
        match set_zypper_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Zypper package manager proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Zypper config: {}", "▲".yellow(), e),
        }
    }

    if set_pacman {
        match set_pacman_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Pacman package manager proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Pacman config: {}", "▲".yellow(), e),
        }
    }

    if set_pkg {
        match set_pkg_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ FreeBSD pkg package manager proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update FreeBSD pkg config: {}", "▲".yellow(), e),
        }
    }

    if set_portage {
        match set_portage_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Portage package manager proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Portage config: {}", "▲".yellow(), e),
        }
    }

    if set_docker_daemon {
        match set_docker_daemon_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Docker Daemon engine proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Docker Daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_flatpak {
        match set_flatpak_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Flatpak package manager proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Flatpak config: {}", "▲".yellow(), e),
        }
    }

    if set_snap {
        match set_snap_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Snap package manager proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Snap config: {}", "▲".yellow(), e),
        }
    }

    if set_ollama {
        match set_ollama_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Ollama local AI runner daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Ollama daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_containerd {
        match set_containerd_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Containerd (Kubernetes/CRI) daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Containerd daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_buildkit {
        match set_buildkit_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ BuildKit (Docker Buildx) daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update BuildKit daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_k3s {
        match set_k3s_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ K3s (Lightweight Kubernetes) daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update K3s daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_microk8s {
        match set_microk8s_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ MicroK8s (Canonical Zero-ops Kubernetes) daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update MicroK8s daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_k0s {
        match set_k0s_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ k0s (Mirantis Zero-friction Kubernetes) daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update k0s daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_rke2 {
        match set_rke2_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ RKE2 (Rancher Kubernetes Engine 2) daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update RKE2 daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_nomad {
        match set_nomad_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Nomad (HashiCorp Nomad Workload Daemon) proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Nomad daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_kubelet {
        match set_kubelet_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Kubelet (Kubernetes Node Daemon) proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Kubelet daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_gitlab_runner {
        match set_gitlab_runner_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ GitLab Runner CI daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update GitLab Runner daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_actions_runner {
        match set_actions_runner_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ GitHub Actions Runner CI daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update GitHub Actions Runner daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_jenkins {
        match set_jenkins_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Jenkins CI automation daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Jenkins daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_buildkite_agent {
        match set_buildkite_agent_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Buildkite Agent CI daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Buildkite Agent daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_woodpecker_agent {
        match set_woodpecker_agent_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Woodpecker CI runner daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Woodpecker CI runner daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_drone_runner {
        match set_drone_runner_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Drone CI runner daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Drone CI runner daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_act_runner {
        match set_act_runner_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Gitea/Forgejo Actions Runner daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Gitea/Forgejo Actions Runner daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_circleci_runner {
        match set_circleci_runner_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ CircleCI runner daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update CircleCI runner daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_azure_pipelines_agent {
        match set_azure_pipelines_agent_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Azure Pipelines Agent daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Azure Pipelines Agent daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_teamcity_agent {
        match set_teamcity_agent_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ TeamCity Agent daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update TeamCity Agent daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_bamboo_agent {
        match set_bamboo_agent_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Bamboo Remote Agent daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Bamboo Remote Agent daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_tfc_agent {
        match set_tfc_agent_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Terraform Cloud Agent daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Terraform Cloud Agent daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_concourse_worker {
        match set_concourse_worker_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Concourse CI Worker daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Concourse CI Worker daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_tentacle_agent {
        match set_tentacle_agent_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Octopus Deploy Tentacle Agent daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Octopus Deploy Tentacle Agent daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_puppet_agent {
        match set_puppet_agent_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Puppet Agent daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Puppet Agent daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_chef_client {
        match set_chef_client_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Chef Infra Client daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Chef Infra Client daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_salt_minion {
        match set_salt_minion_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ SaltStack Salt Minion daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update SaltStack Salt Minion daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_datadog_agent {
        match set_datadog_agent_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Datadog Agent daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Datadog Agent daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_newrelic_infra {
        match set_newrelic_infra_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ New Relic Infrastructure Agent daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update New Relic Infrastructure Agent daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_elastic_agent {
        match set_elastic_agent_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Elastic Agent daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Elastic Agent daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_grafana_alloy {
        match set_grafana_alloy_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Grafana Alloy telemetry proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Grafana Alloy telemetry config: {}", "▲".yellow(), e),
        }
    }

    if set_otelcol {
        match set_otelcol_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ OpenTelemetry Collector daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update OpenTelemetry Collector daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_vector {
        match set_vector_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Vector pipeline daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Vector pipeline daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_fluent_bit {
        match set_fluent_bit_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Fluent Bit telemetry daemon proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Fluent Bit telemetry daemon config: {}", "▲".yellow(), e),
        }
    }

    if set_fluentd {
        match set_fluentd_proxy(&proxy_url) {
            Ok(p) => eprintln!(
                "{}",
                format!(
                    "✔ Fluentd logging framework proxy ({}) set to {}",
                    p.display(),
                    proxy_url
                )
                .green()
            ),
            Err(e) => eprintln!("{} Failed to update Fluentd logging framework config: {}", "▲".yellow(), e),
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
    luarocks: bool,
    nimble: bool,
    rebar3: bool,
    scoop: bool,
    bundle: bool,
    gcloud: bool,
    aws: bool,
    azure: bool,
    mamba: bool,
    podman: bool,
    vagrant: bool,
    choco: bool,
    colima: bool,
    minikube: bool,
    spack: bool,
    macports: bool,
    apt: bool,
    dnf: bool,
    zypper: bool,
    pacman: bool,
    pkg: bool,
    portage: bool,
    docker_daemon: bool,
    flatpak: bool,
    snap: bool,
    ollama: bool,
    containerd: bool,
    buildkit: bool,
    k3s: bool,
    microk8s: bool,
    k0s: bool,
    rke2: bool,
    nomad: bool,
    kubelet: bool,
    gitlab_runner: bool,
    actions_runner: bool,
    jenkins: bool,
    buildkite_agent: bool,
    woodpecker_agent: bool,
    drone_runner: bool,
    act_runner: bool,
    circleci_runner: bool,
    azure_pipelines_agent: bool,
    teamcity_agent: bool,
    bamboo_agent: bool,
    tfc_agent: bool,
    concourse_worker: bool,
    tentacle_agent: bool,
    puppet_agent: bool,
    chef_client: bool,
    salt_minion: bool,
    datadog_agent: bool,
    newrelic_infra: bool,
    elastic_agent: bool,
    grafana_alloy: bool,
    otelcol: bool,
    vector: bool,
    fluent_bit: bool,
    fluentd: bool,
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
    let unset_luarocks = luarocks || all;
    let unset_nimble = nimble || all;
    let unset_rebar3 = rebar3 || all;
    let unset_scoop = scoop || all;
    let unset_bundle = bundle || all;
    let unset_gcloud = gcloud || all;
    let unset_aws = aws || all;
    let unset_azure = azure || all;
    let unset_mamba = mamba || all;
    let unset_podman = podman || all;
    let unset_vagrant = vagrant || all;
    let unset_choco = choco || all;
    let unset_colima = colima || all;
    let unset_minikube = minikube || all;
    let unset_spack = spack || all;
    let unset_macports = macports || all;
    let unset_apt = apt || all;
    let unset_dnf = dnf || all;
    let unset_zypper = zypper || all;
    let unset_pacman = pacman || all;
    let unset_pkg = pkg || all;
    let unset_portage = portage || all;
    let unset_docker_daemon = docker_daemon || all;
    let unset_flatpak = flatpak || all;
    let unset_snap = snap || all;
    let unset_ollama = ollama || all;
    let unset_containerd = containerd || all;
    let unset_buildkit = buildkit || all;
    let unset_k3s = k3s || all;
    let unset_microk8s = microk8s || all;
    let unset_k0s = k0s || all;
    let unset_rke2 = rke2 || all;
    let unset_nomad = nomad || all;
    let unset_kubelet = kubelet || all;
    let unset_gitlab_runner = gitlab_runner || all;
    let unset_actions_runner = actions_runner || all;
    let unset_jenkins = jenkins || all;
    let unset_buildkite_agent = buildkite_agent || all;
    let unset_woodpecker_agent = woodpecker_agent || all;
    let unset_drone_runner = drone_runner || all;
    let unset_act_runner = act_runner || all;
    let unset_circleci_runner = circleci_runner || all;
    let unset_azure_pipelines_agent = azure_pipelines_agent || all;
    let unset_teamcity_agent = teamcity_agent || all;
    let unset_bamboo_agent = bamboo_agent || all;
    let unset_tfc_agent = tfc_agent || all;
    let unset_concourse_worker = concourse_worker || all;
    let unset_tentacle_agent = tentacle_agent || all;
    let unset_puppet_agent = puppet_agent || all;
    let unset_chef_client = chef_client || all;
    let unset_salt_minion = salt_minion || all;
    let unset_datadog_agent = datadog_agent || all;
    let unset_newrelic_infra = newrelic_infra || all;
    let unset_elastic_agent = elastic_agent || all;
    let unset_grafana_alloy = grafana_alloy || all;
    let unset_otelcol = otelcol || all;
    let unset_vector = vector || all;
    let unset_fluent_bit = fluent_bit || all;
    let unset_fluentd = fluentd || all;
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

    if unset_luarocks {
        match unset_luarocks_proxy() {
            Ok(true) => eprintln!("{}", "✔ LuaRocks proxy removed from config.lua".green()),
            Ok(false) => eprintln!("{}", "○ LuaRocks proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset LuaRocks proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_nimble {
        match unset_nimble_proxy() {
            Ok(true) => eprintln!("{}", "✔ Nimble proxy removed from nimble.ini".green()),
            Ok(false) => eprintln!("{}", "○ Nimble proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Nimble proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_rebar3 {
        match unset_rebar3_proxy() {
            Ok(true) => eprintln!("{}", "✔ Rebar3 proxy removed from rebar.config".green()),
            Ok(false) => eprintln!("{}", "○ Rebar3 proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Rebar3 proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_scoop {
        match unset_scoop_proxy() {
            Ok(true) => eprintln!("{}", "✔ Scoop proxy removed from config.json".green()),
            Ok(false) => eprintln!("{}", "○ Scoop proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Scoop proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_bundle {
        match unset_bundler_proxy() {
            Ok(true) => eprintln!("{}", "✔ Ruby Bundler proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Ruby Bundler proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Ruby Bundler proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_gcloud {
        match unset_gcloud_proxy() {
            Ok(true) => eprintln!(
                "{}",
                "✔ Google Cloud SDK (gcloud) proxy removed from config".green()
            ),
            Ok(false) => eprintln!(
                "{}",
                "○ Google Cloud SDK (gcloud) proxy was not set".dimmed()
            ),
            Err(e) => eprintln!(
                "{} Failed to unset Google Cloud SDK (gcloud) proxy: {}",
                "▲".yellow(),
                e
            ),
        }
    }

    if unset_aws {
        match unset_aws_proxy() {
            Ok(true) => eprintln!("{}", "✔ AWS CLI proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ AWS CLI proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset AWS CLI proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_azure {
        match unset_azure_proxy() {
            Ok(true) => eprintln!(
                "{}",
                "✔ Microsoft Azure CLI (az) proxy removed from config".green()
            ),
            Ok(false) => eprintln!(
                "{}",
                "○ Microsoft Azure CLI (az) proxy was not set".dimmed()
            ),
            Err(e) => eprintln!(
                "{} Failed to unset Microsoft Azure CLI proxy: {}",
                "▲".yellow(),
                e
            ),
        }
    }

    if unset_mamba {
        match unset_mamba_proxy() {
            Ok(true) => eprintln!("{}", "✔ Mamba / Micromamba proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Mamba proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Mamba proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_podman {
        match unset_podman_proxy() {
            Ok(true) => eprintln!("{}", "✔ Podman container engine proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Podman proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Podman proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_vagrant {
        match unset_vagrant_proxy() {
            Ok(true) => eprintln!("{}", "✔ Vagrant proxy removed from Vagrantfile".green()),
            Ok(false) => eprintln!("{}", "○ Vagrant proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Vagrant proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_choco {
        match unset_choco_proxy() {
            Ok(true) => eprintln!("{}", "✔ Chocolatey proxy removed from chocolatey.config".green()),
            Ok(false) => eprintln!("{}", "○ Chocolatey proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Chocolatey proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_colima {
        match unset_colima_proxy() {
            Ok(true) => eprintln!("{}", "✔ Colima proxy removed from colima.yaml".green()),
            Ok(false) => eprintln!("{}", "○ Colima proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Colima proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_minikube {
        match unset_minikube_proxy() {
            Ok(true) => eprintln!("{}", "✔ Minikube proxy removed from config.json".green()),
            Ok(false) => eprintln!("{}", "○ Minikube proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Minikube proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_spack {
        match unset_spack_proxy() {
            Ok(true) => eprintln!("{}", "✔ Spack proxy removed from config.yaml".green()),
            Ok(false) => eprintln!("{}", "○ Spack proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Spack proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_macports {
        match unset_macports_proxy() {
            Ok(true) => eprintln!("{}", "✔ MacPorts proxy removed from macports.conf".green()),
            Ok(false) => eprintln!("{}", "○ MacPorts proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset MacPorts proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_apt {
        match unset_apt_proxy() {
            Ok(true) => eprintln!("{}", "✔ APT proxy removed from apt config".green()),
            Ok(false) => eprintln!("{}", "○ APT proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset APT proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_dnf {
        match unset_dnf_proxy() {
            Ok(true) => eprintln!("{}", "✔ DNF / YUM proxy removed from dnf.conf".green()),
            Ok(false) => eprintln!("{}", "○ DNF / YUM proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset DNF / YUM proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_zypper {
        match unset_zypper_proxy() {
            Ok(true) => eprintln!("{}", "✔ Zypper proxy removed from zypp.conf".green()),
            Ok(false) => eprintln!("{}", "○ Zypper proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Zypper proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_pacman {
        match unset_pacman_proxy() {
            Ok(true) => eprintln!("{}", "✔ Pacman proxy removed from pacman.conf".green()),
            Ok(false) => eprintln!("{}", "○ Pacman proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Pacman proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_pkg {
        match unset_pkg_proxy() {
            Ok(true) => eprintln!("{}", "✔ FreeBSD pkg proxy removed from pkg.conf".green()),
            Ok(false) => eprintln!("{}", "○ FreeBSD pkg proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset FreeBSD pkg proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_portage {
        match unset_portage_proxy() {
            Ok(true) => eprintln!("{}", "✔ Portage proxy removed from make.conf".green()),
            Ok(false) => eprintln!("{}", "○ Portage proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Portage proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_docker_daemon {
        match unset_docker_daemon_proxy() {
            Ok(true) => eprintln!("{}", "✔ Docker Daemon proxy removed from daemon.json".green()),
            Ok(false) => eprintln!("{}", "○ Docker Daemon proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Docker Daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_flatpak {
        match unset_flatpak_proxy() {
            Ok(true) => eprintln!("{}", "✔ Flatpak proxy removed from repo/config".green()),
            Ok(false) => eprintln!("{}", "○ Flatpak proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Flatpak proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_snap {
        match unset_snap_proxy() {
            Ok(true) => eprintln!("{}", "✔ Snap proxy removed from snapd config / system settings".green()),
            Ok(false) => eprintln!("{}", "○ Snap proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Snap proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_ollama {
        match unset_ollama_proxy() {
            Ok(true) => eprintln!("{}", "✔ Ollama daemon proxy removed from service/environment config".green()),
            Ok(false) => eprintln!("{}", "○ Ollama proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Ollama daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_containerd {
        match unset_containerd_proxy() {
            Ok(true) => eprintln!("{}", "✔ Containerd daemon proxy removed from service/environment config".green()),
            Ok(false) => eprintln!("{}", "○ Containerd proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Containerd daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_buildkit {
        match unset_buildkit_proxy() {
            Ok(true) => eprintln!("{}", "✔ BuildKit daemon proxy removed from service/environment config".green()),
            Ok(false) => eprintln!("{}", "○ BuildKit proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset BuildKit daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_k3s {
        match unset_k3s_proxy() {
            Ok(true) => eprintln!("{}", "✔ K3s daemon proxy removed from service/environment config".green()),
            Ok(false) => eprintln!("{}", "○ K3s proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset K3s daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_microk8s {
        match unset_microk8s_proxy() {
            Ok(true) => eprintln!("{}", "✔ MicroK8s daemon proxy removed from service/environment config".green()),
            Ok(false) => eprintln!("{}", "○ MicroK8s proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset MicroK8s daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_k0s {
        match unset_k0s_proxy() {
            Ok(true) => eprintln!("{}", "✔ k0s daemon proxy removed from service/environment config".green()),
            Ok(false) => eprintln!("{}", "○ k0s proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset k0s daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_rke2 {
        match unset_rke2_proxy() {
            Ok(true) => eprintln!("{}", "✔ RKE2 daemon proxy removed from service/environment config".green()),
            Ok(false) => eprintln!("{}", "○ RKE2 proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset RKE2 daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_nomad {
        match unset_nomad_proxy() {
            Ok(true) => eprintln!("{}", "✔ Nomad daemon proxy removed from service/environment config".green()),
            Ok(false) => eprintln!("{}", "○ Nomad proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Nomad daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_kubelet {
        match unset_kubelet_proxy() {
            Ok(true) => eprintln!("{}", "✔ Kubelet daemon proxy removed from service/environment config".green()),
            Ok(false) => eprintln!("{}", "○ Kubelet proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Kubelet daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_gitlab_runner {
        match unset_gitlab_runner_proxy() {
            Ok(true) => eprintln!("{}", "✔ GitLab Runner daemon proxy removed from service/environment config".green()),
            Ok(false) => eprintln!("{}", "○ GitLab Runner proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset GitLab Runner daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_actions_runner {
        match unset_actions_runner_proxy() {
            Ok(true) => eprintln!("{}", "✔ GitHub Actions Runner daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ GitHub Actions Runner proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset GitHub Actions Runner daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_jenkins {
        match unset_jenkins_proxy() {
            Ok(true) => eprintln!("{}", "✔ Jenkins CI daemon proxy removed from service/environment config".green()),
            Ok(false) => eprintln!("{}", "○ Jenkins proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Jenkins daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_buildkite_agent {
        match unset_buildkite_agent_proxy() {
            Ok(true) => eprintln!("{}", "✔ Buildkite Agent CI daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Buildkite Agent proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Buildkite Agent daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_woodpecker_agent {
        match unset_woodpecker_agent_proxy() {
            Ok(true) => eprintln!("{}", "✔ Woodpecker CI runner daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Woodpecker CI runner proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Woodpecker CI runner daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_drone_runner {
        match unset_drone_runner_proxy() {
            Ok(true) => eprintln!("{}", "✔ Drone CI runner daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Drone CI runner proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Drone CI runner daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_act_runner {
        match unset_act_runner_proxy() {
            Ok(true) => eprintln!("{}", "✔ Gitea/Forgejo Actions Runner daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Gitea/Forgejo Actions Runner proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Gitea/Forgejo Actions Runner daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_circleci_runner {
        match unset_circleci_runner_proxy() {
            Ok(true) => eprintln!("{}", "✔ CircleCI runner daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ CircleCI runner proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset CircleCI runner daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_azure_pipelines_agent {
        match unset_azure_pipelines_agent_proxy() {
            Ok(true) => eprintln!("{}", "✔ Azure Pipelines Agent daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Azure Pipelines Agent proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Azure Pipelines Agent daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_teamcity_agent {
        match unset_teamcity_agent_proxy() {
            Ok(true) => eprintln!("{}", "✔ TeamCity Agent daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ TeamCity Agent proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset TeamCity Agent daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_bamboo_agent {
        match unset_bamboo_agent_proxy() {
            Ok(true) => eprintln!("{}", "✔ Bamboo Remote Agent daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Bamboo Remote Agent proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Bamboo Remote Agent daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_tfc_agent {
        match unset_tfc_agent_proxy() {
            Ok(true) => eprintln!("{}", "✔ Terraform Cloud Agent daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Terraform Cloud Agent proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Terraform Cloud Agent daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_concourse_worker {
        match unset_concourse_worker_proxy() {
            Ok(true) => eprintln!("{}", "✔ Concourse CI Worker daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Concourse CI Worker proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Concourse CI Worker daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_tentacle_agent {
        match unset_tentacle_agent_proxy() {
            Ok(true) => eprintln!("{}", "✔ Octopus Deploy Tentacle Agent daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Octopus Deploy Tentacle Agent proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Octopus Deploy Tentacle Agent daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_puppet_agent {
        match unset_puppet_agent_proxy() {
            Ok(true) => eprintln!("{}", "✔ Puppet Agent daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Puppet Agent proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Puppet Agent daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_chef_client {
        match unset_chef_client_proxy() {
            Ok(true) => eprintln!("{}", "✔ Chef Infra Client daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Chef Infra Client proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Chef Infra Client daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_salt_minion {
        match unset_salt_minion_proxy() {
            Ok(true) => eprintln!("{}", "✔ SaltStack Salt Minion daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ SaltStack Salt Minion proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset SaltStack Salt Minion daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_datadog_agent {
        match unset_datadog_agent_proxy() {
            Ok(true) => eprintln!("{}", "✔ Datadog Agent daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Datadog Agent proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Datadog Agent daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_newrelic_infra {
        match unset_newrelic_infra_proxy() {
            Ok(true) => eprintln!("{}", "✔ New Relic Infrastructure Agent daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ New Relic Infrastructure Agent proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset New Relic Infrastructure Agent daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_elastic_agent {
        match unset_elastic_agent_proxy() {
            Ok(true) => eprintln!("{}", "✔ Elastic Agent daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Elastic Agent proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Elastic Agent daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_grafana_alloy {
        match unset_grafana_alloy_proxy() {
            Ok(true) => eprintln!("{}", "✔ Grafana Alloy telemetry proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Grafana Alloy telemetry proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Grafana Alloy telemetry proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_otelcol {
        match unset_otelcol_proxy() {
            Ok(true) => eprintln!("{}", "✔ OpenTelemetry Collector daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ OpenTelemetry Collector proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset OpenTelemetry Collector daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_vector {
        match unset_vector_proxy() {
            Ok(true) => eprintln!("{}", "✔ Vector pipeline daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Vector pipeline proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Vector pipeline daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_fluent_bit {
        match unset_fluent_bit_proxy() {
            Ok(true) => eprintln!("{}", "✔ Fluent Bit telemetry daemon proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Fluent Bit telemetry proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Fluent Bit telemetry daemon proxy: {}", "▲".yellow(), e),
        }
    }

    if unset_fluentd {
        match unset_fluentd_proxy() {
            Ok(true) => eprintln!("{}", "✔ Fluentd logging framework proxy removed from config".green()),
            Ok(false) => eprintln!("{}", "○ Fluentd logging framework proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to unset Fluentd logging framework proxy: {}", "▲".yellow(), e),
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

    #[test]
    fn test_apply_and_remove_luarocks_proxy() {
        let initial = "-- LuaRocks configuration\nrocks_servers = {\n   \"https://luarocks.org\"\n}\n";
        let with_proxy = apply_luarocks_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("http_proxy = \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("https_proxy = \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("rocks_servers"));

        // Replace existing
        let updated = apply_luarocks_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("http_proxy = \"http://127.0.0.1:1080\""));
        assert!(updated.contains("https_proxy = \"http://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_luarocks_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("http_proxy"));
        assert!(!cleaned.contains("https_proxy"));
        assert!(cleaned.contains("rocks_servers"));
    }

    #[test]
    fn test_apply_and_remove_nimble_proxy() {
        let initial = "# Nimble configuration\n[Package]\nname = \"mypkg\"\n";
        let with_proxy = apply_nimble_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("http_proxy = \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("https_proxy = \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("name = \"mypkg\""));

        // Replace existing
        let updated = apply_nimble_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("http_proxy = \"http://127.0.0.1:1080\""));
        assert!(updated.contains("https_proxy = \"http://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_nimble_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("http_proxy"));
        assert!(!cleaned.contains("https_proxy"));
        assert!(cleaned.contains("name = \"mypkg\""));
    }

    #[test]
    fn test_apply_and_remove_rebar3_proxy() {
        let initial = "%% Erlang rebar3 config\n{erl_opts, [debug_info]}.\n";
        let with_proxy = apply_rebar3_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("{http_proxy, \"http://127.0.0.1:7890\"}."));
        assert!(with_proxy.contains("{https_proxy, \"http://127.0.0.1:7890\"}."));
        assert!(with_proxy.contains("debug_info"));

        // Replace existing
        let updated = apply_rebar3_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("{http_proxy, \"http://127.0.0.1:1080\"}."));
        assert!(updated.contains("{https_proxy, \"http://127.0.0.1:1080\"}."));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_rebar3_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("http_proxy"));
        assert!(!cleaned.contains("https_proxy"));
        assert!(cleaned.contains("debug_info"));
    }

    #[test]
    fn test_apply_and_remove_scoop_proxy() {
        let initial = r#"{ "root_path": "C:\\Users\\user\\scoop" }"#;
        let with_proxy = apply_scoop_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("\"proxy\": \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("root_path"));

        // Replace existing
        let updated = apply_scoop_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("\"proxy\": \"http://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_scoop_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("\"proxy\""));
        assert!(cleaned.contains("root_path"));
    }

    #[test]
    fn test_apply_and_remove_bundler_proxy() {
        let initial = "---\nBUNDLE_FROZEN: \"true\"\n";
        let with_proxy = apply_bundler_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("BUNDLE_HTTP_PROXY: \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("BUNDLE_HTTPS_PROXY: \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("BUNDLE_FROZEN"));

        // Replace existing
        let updated = apply_bundler_proxy(&with_proxy, "http://127.0.0.1:1080");
        assert!(updated.contains("BUNDLE_HTTP_PROXY: \"http://127.0.0.1:1080\""));
        assert!(updated.contains("BUNDLE_HTTPS_PROXY: \"http://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_bundler_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("BUNDLE_HTTP_PROXY"));
        assert!(!cleaned.contains("BUNDLE_HTTPS_PROXY"));
        assert!(cleaned.contains("BUNDLE_FROZEN"));
    }

    #[test]
    fn test_apply_and_remove_gcloud_config_proxy() {
        let initial = "[core]\nproject = my-project\n";
        let with_proxy = apply_gcloud_config_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("[proxy]"));
        assert!(with_proxy.contains("address = 127.0.0.1"));
        assert!(with_proxy.contains("port = 7890"));
        assert!(with_proxy.contains("type = http"));
        assert!(with_proxy.contains("project = my-project"));

        // Replace existing with socks5
        let updated = apply_gcloud_config_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("port = 1080"));
        assert!(updated.contains("type = socks5"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_gcloud_config_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("[proxy]"));
        assert!(!cleaned.contains("address ="));
        assert!(!cleaned.contains("port ="));
        assert!(cleaned.contains("project = my-project"));
    }

    #[test]
    fn test_apply_and_remove_aws_config_proxy() {
        let initial = "[default]\nregion = us-west-2\n";
        let with_proxy = apply_aws_config_proxy(initial, "http://127.0.0.1:7890", Some("default"));
        assert!(with_proxy.contains("proxy = http://127.0.0.1:7890"));
        assert!(with_proxy.contains("region = us-west-2"));

        // Replace existing
        let updated = apply_aws_config_proxy(&with_proxy, "http://127.0.0.1:1080", Some("default"));
        assert!(updated.contains("proxy = http://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_aws_config_proxy(&updated, Some("default"));
        assert!(removed);
        assert!(!cleaned.contains("proxy ="));
        assert!(cleaned.contains("region = us-west-2"));
    }

    #[test]
    fn test_apply_and_remove_azure_config_proxy() {
        let initial = "[cloud]\nname = AzureCloud\n";
        let with_proxy = apply_azure_config_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("[proxy]"));
        assert!(with_proxy.contains("mode = manual"));
        assert!(with_proxy.contains("http = http://127.0.0.1:7890"));
        assert!(with_proxy.contains("https = http://127.0.0.1:7890"));
        assert!(with_proxy.contains("name = AzureCloud"));

        // Replace existing with socks5
        let updated = apply_azure_config_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("http = socks5://127.0.0.1:1080"));
        assert!(updated.contains("https = socks5://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_azure_config_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("http ="));
        assert!(!cleaned.contains("https ="));
        assert!(!cleaned.contains("mode = manual"));
        assert!(cleaned.contains("name = AzureCloud"));
    }

    #[test]
    fn test_apply_and_remove_podman_proxy() {
        let initial = "[engine]\ncgroup_manager = \"systemd\"\n";
        let with_proxy = apply_podman_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("[engine]"));
        assert!(with_proxy.contains("http_proxy=http://127.0.0.1:7890"));
        assert!(with_proxy.contains("https_proxy=http://127.0.0.1:7890"));
        assert!(with_proxy.contains("cgroup_manager = \"systemd\""));

        // Replace with new proxy
        let updated = apply_podman_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("http_proxy=socks5://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_podman_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("http_proxy"));
        assert!(cleaned.contains("cgroup_manager = \"systemd\""));
    }

    #[test]
    fn test_apply_and_remove_vagrant_proxy() {
        let initial = "Vagrant.configure(\"2\") do |config|\n  config.vm.box = \"ubuntu/focal64\"\nend\n";
        let with_proxy = apply_vagrant_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("config.proxy.http = \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("config.proxy.https = \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("config.vm.box = \"ubuntu/focal64\""));

        let (cleaned, removed) = remove_vagrant_proxy(&with_proxy);
        assert!(removed);
        assert!(!cleaned.contains("config.proxy.http"));
        assert!(!cleaned.contains("config.proxy.https"));
        assert!(cleaned.contains("config.vm.box = \"ubuntu/focal64\""));
    }

    #[test]
    fn test_apply_and_remove_choco_proxy() {
        let initial = r#"<?xml version="1.0" encoding="utf-8"?>
<chocolatey>
  <config>
    <add key="cacheLocation" value="" description="Cache location" />
  </config>
</chocolatey>
"#;
        let with_proxy = apply_choco_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("<add key=\"proxy\" value=\"http://127.0.0.1:7890\""));

        // Replace with new proxy
        let updated = apply_choco_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("value=\"socks5://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_choco_proxy(&updated);
        assert!(removed);
        assert!(cleaned.contains("value=\"\""));
        assert!(!cleaned.contains("socks5://127.0.0.1:1080"));
    }

    #[test]
    fn test_apply_and_remove_colima_proxy() {
        let initial = "cpu: 2\nmemory: 4\n";
        let with_proxy = apply_colima_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("env:"));
        assert!(with_proxy.contains("HTTP_PROXY: \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("HTTPS_PROXY: \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("ALL_PROXY: \"http://127.0.0.1:7890\""));

        let (cleaned, removed) = remove_colima_proxy(&with_proxy);
        assert!(removed);
        assert!(!cleaned.contains("HTTP_PROXY"));
        assert!(!cleaned.contains("HTTPS_PROXY"));
        assert!(!cleaned.contains("ALL_PROXY"));
        assert!(cleaned.contains("cpu: 2"));
    }

    #[test]
    fn test_apply_and_remove_minikube_proxy() {
        let initial = r#"{
  "driver": "docker"
}"#;
        let with_proxy = apply_minikube_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("\"http_proxy\": \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("\"https_proxy\": \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("\"no_proxy\":"));
        assert!(with_proxy.contains("\"driver\": \"docker\""));

        // Replace with new proxy
        let updated = apply_minikube_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("\"http_proxy\": \"socks5://127.0.0.1:1080\""));
        assert!(updated.contains("\"https_proxy\": \"socks5://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_minikube_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("http_proxy"));
        assert!(!cleaned.contains("https_proxy"));
        assert!(cleaned.contains("\"driver\": \"docker\""));
    }

    #[test]
    fn test_apply_and_remove_spack_proxy() {
        let initial = "config:\n  connect_timeout: 10\n";
        let with_proxy = apply_spack_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("proxy: \"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("connect_timeout: 10"));

        // Replace with new proxy
        let updated = apply_spack_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("proxy: \"socks5://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_spack_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("proxy:"));
        assert!(cleaned.contains("connect_timeout: 10"));
    }

    #[test]
    fn test_apply_and_remove_macports_proxy() {
        let initial = "# MacPorts configuration file\nrsync_server rsync.macports.org\n";
        let with_proxy = apply_macports_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("proxy_http              http://127.0.0.1:7890"));
        assert!(with_proxy.contains("proxy_https             http://127.0.0.1:7890"));
        assert!(with_proxy.contains("rsync_server rsync.macports.org"));

        // Replace proxy
        let updated = apply_macports_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("proxy_http              socks5://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_macports_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("proxy_http"));
        assert!(!cleaned.contains("proxy_https"));
        assert!(cleaned.contains("rsync_server rsync.macports.org"));
    }

    #[test]
    fn test_apply_and_remove_apt_proxy() {
        let initial = "// APT configuration\nAPT::Install-Recommends \"0\";\n";
        let with_proxy = apply_apt_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("Acquire::http::Proxy \"http://127.0.0.1:7890\";"));
        assert!(with_proxy.contains("Acquire::https::Proxy \"http://127.0.0.1:7890\";"));
        assert!(with_proxy.contains("APT::Install-Recommends \"0\";"));

        // Replace proxy
        let updated = apply_apt_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("Acquire::http::Proxy \"socks5://127.0.0.1:1080\";"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_apt_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("Acquire::http::Proxy"));
        assert!(!cleaned.contains("Acquire::https::Proxy"));
        assert!(cleaned.contains("APT::Install-Recommends \"0\";"));
    }

    #[test]
    fn test_apply_and_remove_dnf_proxy() {
        let initial = "[main]\ngpgcheck=1\ninstallonly_limit=3\n";
        let with_proxy = apply_dnf_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("proxy=http://127.0.0.1:7890"));
        assert!(with_proxy.contains("gpgcheck=1"));

        // Replace proxy
        let updated = apply_dnf_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("proxy=socks5://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_dnf_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("proxy="));
        assert!(cleaned.contains("gpgcheck=1"));

        // When [main] does not exist initially
        let empty = "";
        let applied_empty = apply_dnf_proxy(empty, "http://127.0.0.1:7890");
        assert!(applied_empty.contains("[main]"));
        assert!(applied_empty.contains("proxy=http://127.0.0.1:7890"));
    }

    #[test]
    fn test_apply_and_remove_zypper_proxy() {
        let initial = "## Zypper config\nsolver.onlyRequires = true\n";
        let with_proxy = apply_zypper_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("proxy = http://127.0.0.1:7890"));
        assert!(with_proxy.contains("solver.onlyRequires = true"));

        // Replace proxy
        let updated = apply_zypper_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("proxy = socks5://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_zypper_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("proxy = "));
        assert!(cleaned.contains("solver.onlyRequires = true"));
    }

    #[test]
    fn test_apply_and_remove_pacman_proxy() {
        let initial = "[options]\nHoldPkg = pacman glibc\n";
        let with_proxy = apply_pacman_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("XferCommand = /usr/bin/curl -g -L -C - --fail --retry 3 --proxy http://127.0.0.1:7890 -o %o %u"));
        assert!(with_proxy.contains("HoldPkg = pacman glibc"));

        // Replace proxy
        let updated = apply_pacman_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("--proxy socks5://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_pacman_proxy(&updated);
        assert!(removed);
        assert!(cleaned.contains("#XferCommand ="));
        assert!(cleaned.contains("HoldPkg = pacman glibc"));

        // Empty file case
        let empty = "";
        let applied_empty = apply_pacman_proxy(empty, "http://127.0.0.1:7890");
        assert!(applied_empty.contains("[options]"));
        assert!(applied_empty.contains("--proxy http://127.0.0.1:7890"));
    }

    #[test]
    fn test_apply_and_remove_pkg_proxy() {
        let initial = "# FreeBSD pkg config\nPKG_DBDIR = \"/var/db/pkg\";\n";
        let with_proxy = apply_pkg_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("http_proxy: \"http://127.0.0.1:7890\";"));
        assert!(with_proxy.contains("PKG_DBDIR = \"/var/db/pkg\";"));

        // Replace proxy
        let updated = apply_pkg_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("http_proxy: \"socks5://127.0.0.1:1080\";"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_pkg_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("http_proxy"));
        assert!(cleaned.contains("PKG_DBDIR = \"/var/db/pkg\";"));
    }

    #[test]
    fn test_remove_docker_daemon_proxy_json() {
        let mut root: serde_json::Value = serde_json::json!({
            "log-level": "info",
            "proxies": {
                "http-proxy": "http://127.0.0.1:7890",
                "https-proxy": "http://127.0.0.1:7890",
                "no-proxy": "localhost"
            }
        });
        assert!(remove_docker_daemon_proxy_json(&mut root));
        assert!(root.get("proxies").is_none());
        assert_eq!(root["log-level"], "info");
    }

    #[test]
    fn test_apply_docker_daemon_proxy_json() {
        let mut root = serde_json::json!({
            "log-level": "warn"
        });
        apply_docker_daemon_proxy_json(&mut root, "http://127.0.0.1:7890");
        assert_eq!(
            root["proxies"]["http-proxy"],
            "http://127.0.0.1:7890"
        );
        assert_eq!(
            root["proxies"]["https-proxy"],
            "http://127.0.0.1:7890"
        );
        assert_eq!(
            root["proxies"]["no-proxy"],
            "localhost,127.0.0.1,::1"
        );
        assert_eq!(root["log-level"], "warn");

        // Overwrite existing proxy
        apply_docker_daemon_proxy_json(&mut root, "socks5://127.0.0.1:1080");
        assert_eq!(
            root["proxies"]["http-proxy"],
            "socks5://127.0.0.1:1080"
        );
        assert_eq!(
            root["proxies"]["https-proxy"],
            "socks5://127.0.0.1:1080"
        );
    }

    #[test]
    fn test_apply_and_remove_portage_proxy() {
        let initial = "# Gentoo make.conf\nCFLAGS=\"-O2 -pipe\"\n";
        let with_proxy = apply_portage_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("http_proxy=\"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("https_proxy=\"http://127.0.0.1:7890\""));
        assert!(with_proxy.contains("CFLAGS=\"-O2 -pipe\""));

        // Replace proxy
        let updated = apply_portage_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("http_proxy=\"socks5://127.0.0.1:1080\""));
        assert!(updated.contains("https_proxy=\"socks5://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_portage_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("http_proxy"));
        assert!(!cleaned.contains("https_proxy"));
        assert!(cleaned.contains("CFLAGS=\"-O2 -pipe\""));
    }

    #[test]
    fn test_apply_and_remove_flatpak_proxy() {
        let initial = "[core]\nrepo_version=1\nmode=bare-user-only\n\n[remote \"flathub\"]\nurl=https://dl.flathub.org/repo/\n";
        let with_proxy = apply_flatpak_proxy(initial, "http://127.0.0.1:7890");
        assert!(with_proxy.contains("proxy = http://127.0.0.1:7890"));
        assert!(with_proxy.contains("mode=bare-user-only"));
        assert!(with_proxy.contains("[remote \"flathub\"]"));

        // Replace proxy
        let updated = apply_flatpak_proxy(&with_proxy, "socks5://127.0.0.1:1080");
        assert!(updated.contains("proxy = socks5://127.0.0.1:1080"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_flatpak_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("proxy ="));
        assert!(cleaned.contains("[remote \"flathub\"]"));

        // Empty file case
        let clean = "";
        let generated = apply_flatpak_proxy(clean, "http://127.0.0.1:7890");
        assert!(generated.contains("[core]"));
        assert!(generated.contains("proxy = http://127.0.0.1:7890"));
    }

    #[test]
    fn test_apply_and_remove_snap_proxy() {
        let initial_systemd = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"HTTPS_PROXY=http://127.0.0.1:7890\"\n";
        let updated = apply_snap_proxy(initial_systemd, "socks5://127.0.0.1:1080");
        assert!(updated.contains("Environment=\"HTTP_PROXY=socks5://127.0.0.1:1080\""));
        assert!(updated.contains("Environment=\"HTTPS_PROXY=socks5://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_snap_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("HTTP_PROXY="));
        assert!(!cleaned.contains("HTTPS_PROXY="));

        // Empty file case creates systemd drop-in format
        let clean = "";
        let generated = apply_snap_proxy(clean, "http://127.0.0.1:7890");
        assert!(generated.contains("[Service]"));
        assert!(generated.contains("Environment=\"HTTP_PROXY=http://127.0.0.1:7890\""));

        // INI system config case
        let initial_ini = "[system]\nproxy.http = http://127.0.0.1:7890\n";
        let with_ini = apply_snap_proxy(initial_ini, "http://127.0.0.1:8888");
        assert!(with_ini.contains("proxy.http = http://127.0.0.1:8888"));
        assert!(with_ini.contains("proxy.https = http://127.0.0.1:8888"));
        let (cleaned_ini, removed_ini) = remove_snap_proxy(&with_ini);
        assert!(removed_ini);
        assert!(!cleaned_ini.contains("proxy.http"));
    }

    #[test]
    fn test_apply_and_remove_ollama_proxy() {
        let initial_systemd = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"HTTPS_PROXY=http://127.0.0.1:7890\"\n";
        let updated = apply_ollama_proxy(initial_systemd, "socks5://127.0.0.1:1080");
        assert!(updated.contains("Environment=\"HTTP_PROXY=socks5://127.0.0.1:1080\""));
        assert!(updated.contains("Environment=\"HTTPS_PROXY=socks5://127.0.0.1:1080\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_ollama_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("HTTP_PROXY="));
        assert!(!cleaned.contains("HTTPS_PROXY="));

        // Empty file case creates systemd drop-in format
        let clean = "";
        let generated = apply_ollama_proxy(clean, "http://127.0.0.1:7890");
        assert!(generated.contains("[Service]"));
        assert!(generated.contains("Environment=\"HTTP_PROXY=http://127.0.0.1:7890\""));

        // Env file format case
        let env_initial = "HTTP_PROXY=\"http://127.0.0.1:7890\"\nHTTPS_PROXY=\"http://127.0.0.1:7890\"\n";
        let env_updated = apply_ollama_proxy(env_initial, "http://127.0.0.1:8888");
        assert!(env_updated.contains("HTTP_PROXY=\"http://127.0.0.1:8888\""));
        let (env_cleaned, env_removed) = remove_ollama_proxy(&env_updated);
        assert!(env_removed);
        assert!(!env_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_containerd_proxy() {
        let initial_systemd = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"HTTPS_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"NO_PROXY=localhost\"\n";
        let updated = apply_containerd_proxy(initial_systemd, "socks5://127.0.0.1:1080");
        assert!(updated.contains("Environment=\"HTTP_PROXY=socks5://127.0.0.1:1080\""));
        assert!(updated.contains("Environment=\"HTTPS_PROXY=socks5://127.0.0.1:1080\""));
        assert!(updated.contains("Environment=\"NO_PROXY=localhost,127.0.0.1"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_containerd_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("HTTP_PROXY="));
        assert!(!cleaned.contains("HTTPS_PROXY="));
        assert!(!cleaned.contains("NO_PROXY="));

        // Empty file case creates systemd drop-in format
        let clean = "";
        let generated = apply_containerd_proxy(clean, "http://127.0.0.1:7890");
        assert!(generated.contains("[Service]"));
        assert!(generated.contains("Environment=\"HTTP_PROXY=http://127.0.0.1:7890\""));
        assert!(generated.contains("Environment=\"NO_PROXY="));

        // Env file format case
        let env_initial = "HTTP_PROXY=\"http://127.0.0.1:7890\"\nHTTPS_PROXY=\"http://127.0.0.1:7890\"\n";
        let env_updated = apply_containerd_proxy(env_initial, "http://127.0.0.1:8888");
        assert!(env_updated.contains("HTTP_PROXY=\"http://127.0.0.1:8888\""));
        assert!(env_updated.contains("NO_PROXY="));
        let (env_cleaned, env_removed) = remove_containerd_proxy(&env_updated);
        assert!(env_removed);
        assert!(!env_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_buildkit_proxy() {
        let initial_systemd = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"HTTPS_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"NO_PROXY=localhost\"\n";
        let updated = apply_buildkit_proxy(initial_systemd, "socks5://127.0.0.1:1080");
        assert!(updated.contains("Environment=\"HTTP_PROXY=socks5://127.0.0.1:1080\""));
        assert!(updated.contains("Environment=\"HTTPS_PROXY=socks5://127.0.0.1:1080\""));
        assert!(updated.contains("Environment=\"NO_PROXY=localhost,127.0.0.1"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_buildkit_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("HTTP_PROXY="));
        assert!(!cleaned.contains("HTTPS_PROXY="));
        assert!(!cleaned.contains("NO_PROXY="));

        // Empty file case creates systemd drop-in format
        let clean = "";
        let generated = apply_buildkit_proxy(clean, "http://127.0.0.1:7890");
        assert!(generated.contains("[Service]"));
        assert!(generated.contains("Environment=\"HTTP_PROXY=http://127.0.0.1:7890\""));
        assert!(generated.contains("Environment=\"NO_PROXY="));

        // Env file format case
        let env_initial = "HTTP_PROXY=\"http://127.0.0.1:7890\"\nHTTPS_PROXY=\"http://127.0.0.1:7890\"\n";
        let env_updated = apply_buildkit_proxy(env_initial, "http://127.0.0.1:8888");
        assert!(env_updated.contains("HTTP_PROXY=\"http://127.0.0.1:8888\""));
        assert!(env_updated.contains("NO_PROXY="));
        let (env_cleaned, env_removed) = remove_buildkit_proxy(&env_updated);
        assert!(env_removed);
        assert!(!env_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_k3s_proxy() {
        let initial_env = "HTTP_PROXY=\"http://127.0.0.1:7890\"\nHTTPS_PROXY=\"http://127.0.0.1:7890\"\nNO_PROXY=\"127.0.0.0/8,10.0.0.0/8\"\n";
        let updated = apply_k3s_proxy(initial_env, "http://127.0.0.1:8080");
        assert!(updated.contains("HTTP_PROXY=\"http://127.0.0.1:8080\""));
        assert!(updated.contains("HTTPS_PROXY=\"http://127.0.0.1:8080\""));
        assert!(updated.contains(".svc,.cluster.local"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_k3s_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("HTTP_PROXY="));
        assert!(!cleaned.contains("HTTPS_PROXY="));
        assert!(!cleaned.contains("NO_PROXY="));

        // Empty file case creates env format
        let clean = "";
        let generated = apply_k3s_proxy(clean, "http://127.0.0.1:7890");
        assert!(generated.contains("HTTP_PROXY=\"http://127.0.0.1:7890\""));
        assert!(generated.contains(".svc,.cluster.local"));

        // Systemd drop-in format case
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"HTTPS_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_k3s_proxy(dropin_initial, "socks5://127.0.0.1:1080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=socks5://127.0.0.1:1080\""));
        let (dropin_cleaned, dropin_removed) = remove_k3s_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_microk8s_proxy() {
        let initial_env = "HTTP_PROXY=http://127.0.0.1:7890\nHTTPS_PROXY=http://127.0.0.1:7890\nNO_PROXY=10.1.0.0/16,10.152.183.0/24\n";
        let updated = apply_microk8s_proxy(initial_env, "http://127.0.0.1:8080");
        assert!(updated.contains("HTTP_PROXY=http://127.0.0.1:8080"));
        assert!(updated.contains("HTTPS_PROXY=http://127.0.0.1:8080"));
        assert!(updated.contains(".cluster.local"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_microk8s_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("HTTP_PROXY="));
        assert!(!cleaned.contains("HTTPS_PROXY="));

        // Empty file case
        let clean = "";
        let generated = apply_microk8s_proxy(clean, "http://127.0.0.1:7890");
        assert!(generated.contains("HTTP_PROXY=http://127.0.0.1:7890"));
        assert!(generated.contains("10.152.183.0/24"));

        // Systemd drop-in format case
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"HTTPS_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_microk8s_proxy(dropin_initial, "socks5://127.0.0.1:1080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=socks5://127.0.0.1:1080\""));
        let (dropin_cleaned, dropin_removed) = remove_microk8s_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_k0s_proxy() {
        let initial_dropin = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"HTTPS_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"NO_PROXY=localhost,127.0.0.1\"\n";
        let updated = apply_k0s_proxy(initial_dropin, "http://127.0.0.1:8080");
        assert!(updated.contains("Environment=\"HTTP_PROXY=http://127.0.0.1:8080\""));
        assert!(updated.contains("Environment=\"HTTPS_PROXY=http://127.0.0.1:8080\""));
        assert!(updated.contains(".svc,.cluster.local"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_k0s_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("HTTP_PROXY="));
        assert!(!cleaned.contains("HTTPS_PROXY="));

        // Empty file case
        let clean = "";
        let generated = apply_k0s_proxy(clean, "http://127.0.0.1:7890");
        assert!(generated.contains("[Service]"));
        assert!(generated.contains("Environment=\"HTTP_PROXY=http://127.0.0.1:7890\""));

        // Default env file format case
        let env_initial = "HTTP_PROXY=\"http://127.0.0.1:7890\"\nHTTPS_PROXY=\"http://127.0.0.1:7890\"\n";
        let env_updated = apply_k0s_proxy(env_initial, "socks5://127.0.0.1:1080");
        assert!(env_updated.contains("HTTP_PROXY=\"socks5://127.0.0.1:1080\""));
        let (env_cleaned, env_removed) = remove_k0s_proxy(&env_updated);
        assert!(env_removed);
        assert!(!env_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_rke2_proxy() {
        let initial_dropin = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"HTTPS_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"NO_PROXY=localhost,127.0.0.1\"\n";
        let updated = apply_rke2_proxy(initial_dropin, "http://127.0.0.1:8080");
        assert!(updated.contains("Environment=\"HTTP_PROXY=http://127.0.0.1:8080\""));
        assert!(updated.contains("Environment=\"HTTPS_PROXY=http://127.0.0.1:8080\""));
        assert!(updated.contains("Environment=\"CONTAINERD_HTTP_PROXY=http://127.0.0.1:8080\""));
        assert!(updated.contains(".svc,.cluster.local"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_rke2_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("HTTP_PROXY="));
        assert!(!cleaned.contains("CONTAINERD_HTTP_PROXY="));

        // Empty file case
        let clean = "";
        let generated = apply_rke2_proxy(clean, "http://127.0.0.1:7890");
        assert!(generated.contains("[Service]"));
        assert!(generated.contains("Environment=\"CONTAINERD_HTTP_PROXY=http://127.0.0.1:7890\""));

        // Default env file format case
        let env_initial = "HTTP_PROXY=\"http://127.0.0.1:7890\"\nCONTAINERD_HTTP_PROXY=\"http://127.0.0.1:7890\"\n";
        let env_updated = apply_rke2_proxy(env_initial, "socks5://127.0.0.1:1080");
        assert!(env_updated.contains("HTTP_PROXY=\"socks5://127.0.0.1:1080\""));
        assert!(env_updated.contains("CONTAINERD_HTTP_PROXY=\"socks5://127.0.0.1:1080\""));
        let (env_cleaned, env_removed) = remove_rke2_proxy(&env_updated);
        assert!(env_removed);
        assert!(!env_cleaned.contains("HTTP_PROXY="));
        assert!(!env_cleaned.contains("CONTAINERD_HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_nomad_proxy() {
        let initial_dropin = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"HTTPS_PROXY=http://127.0.0.1:7890\"\n";
        let updated = apply_nomad_proxy(initial_dropin, "http://127.0.0.1:8080");
        assert!(updated.contains("Environment=\"HTTP_PROXY=http://127.0.0.1:8080\""));
        assert!(updated.contains("Environment=\"HTTPS_PROXY=http://127.0.0.1:8080\""));
        assert!(updated.contains("Environment=\"NO_PROXY=10.0.0.0/8"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_nomad_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("HTTP_PROXY="));

        // Empty file case
        let clean = "";
        let generated = apply_nomad_proxy(clean, "http://127.0.0.1:7890");
        assert!(generated.contains("[Service]"));
        assert!(generated.contains("Environment=\"HTTP_PROXY=http://127.0.0.1:7890\""));

        // Env file format case
        let env_initial = "HTTP_PROXY=\"http://127.0.0.1:7890\"\n";
        let env_updated = apply_nomad_proxy(env_initial, "socks5://127.0.0.1:1080");
        assert!(env_updated.contains("HTTP_PROXY=\"socks5://127.0.0.1:1080\""));
        let (env_cleaned, env_removed) = remove_nomad_proxy(&env_updated);
        assert!(env_removed);
        assert!(!env_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_kubelet_proxy() {
        let initial_dropin = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\nEnvironment=\"HTTPS_PROXY=http://127.0.0.1:7890\"\n";
        let updated = apply_kubelet_proxy(initial_dropin, "http://127.0.0.1:8080");
        assert!(updated.contains("Environment=\"HTTP_PROXY=http://127.0.0.1:8080\""));
        assert!(updated.contains("Environment=\"HTTPS_PROXY=http://127.0.0.1:8080\""));
        assert!(updated.contains("Environment=\"NO_PROXY=10.0.0.0/8"));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_kubelet_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("HTTP_PROXY="));

        // Empty file case
        let clean = "";
        let generated = apply_kubelet_proxy(clean, "http://127.0.0.1:7890");
        assert!(generated.contains("[Service]"));
        assert!(generated.contains("Environment=\"HTTP_PROXY=http://127.0.0.1:7890\""));

        // Env file format case
        let env_initial = "HTTP_PROXY=\"http://127.0.0.1:7890\"\n";
        let env_updated = apply_kubelet_proxy(env_initial, "socks5://127.0.0.1:1080");
        assert!(env_updated.contains("HTTP_PROXY=\"socks5://127.0.0.1:1080\""));
        let (env_cleaned, env_removed) = remove_kubelet_proxy(&env_updated);
        assert!(env_removed);
        assert!(!env_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_gitlab_runner_proxy() {
        let toml_initial = r#"
concurrent = 4
check_interval = 0

[[runners]]
  name = "runner-1"
  executor = "docker"
  environment = ["SOME_VAR=1", "HTTP_PROXY=http://127.0.0.1:7890"]
"#;
        let updated = apply_gitlab_runner_proxy(toml_initial, "http://127.0.0.1:8080");
        assert!(updated.contains("\"HTTP_PROXY=http://127.0.0.1:8080\""));
        assert!(updated.contains("\"HTTPS_PROXY=http://127.0.0.1:8080\""));
        assert!(updated.contains("\"NO_PROXY=localhost,127.0.0.1,.local\""));
        assert!(updated.contains("\"SOME_VAR=1\""));
        assert!(!updated.contains("7890"));

        let (cleaned, removed) = remove_gitlab_runner_proxy(&updated);
        assert!(removed);
        assert!(!cleaned.contains("HTTP_PROXY="));
        assert!(cleaned.contains("\"SOME_VAR=1\""));

        // Empty file case
        let clean = "";
        let generated = apply_gitlab_runner_proxy(clean, "http://127.0.0.1:7890");
        assert!(generated.contains("[[runners]]"));
        assert!(generated.contains("\"HTTP_PROXY=http://127.0.0.1:7890\""));

        // Systemd drop-in format case
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_gitlab_runner_proxy(dropin_initial, "socks5://127.0.0.1:1080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=socks5://127.0.0.1:1080\""));
        let (dropin_cleaned, dropin_removed) = remove_gitlab_runner_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_actions_runner_proxy() {
        // Standard single line .proxy file
        let proxy_initial = "http://127.0.0.1:7890\n";
        let updated = apply_actions_runner_proxy(proxy_initial, "http://127.0.0.1:8080");
        assert_eq!(updated.trim(), "http://127.0.0.1:8080");
        let (cleaned, removed) = remove_actions_runner_proxy(&updated);
        assert!(removed);
        assert!(cleaned.is_empty());

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_actions_runner_proxy(dropin_initial, "socks5://127.0.0.1:1080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=socks5://127.0.0.1:1080\""));
        let (dropin_cleaned, dropin_removed) = remove_actions_runner_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));

        // .env file format
        let env_initial = "http_proxy=http://127.0.0.1:7890\nhttps_proxy=http://127.0.0.1:7890\n";
        let env_updated = apply_actions_runner_proxy(env_initial, "http://127.0.0.1:8118");
        assert!(env_updated.contains("http_proxy=http://127.0.0.1:8118"));
        assert!(env_updated.contains("https_proxy=http://127.0.0.1:8118"));
        let (env_cleaned, env_removed) = remove_actions_runner_proxy(&env_updated);
        assert!(env_removed);
        assert!(!env_cleaned.contains("http_proxy="));
    }

    #[test]
    fn test_apply_and_remove_jenkins_proxy() {
        // XML proxy.xml
        let xml_initial = r#"<?xml version='1.1' encoding='UTF-8'?>
<proxy>
  <name>127.0.0.1</name>
  <port>7890</port>
</proxy>
"#;
        let xml_updated = apply_jenkins_proxy(xml_initial, "http://10.0.0.10:8080");
        assert!(xml_updated.contains("<name>10.0.0.10</name>"));
        assert!(xml_updated.contains("<port>8080</port>"));
        let (xml_cleaned, xml_removed) = remove_jenkins_proxy(&xml_updated);
        assert!(xml_removed);
        assert!(xml_cleaned.contains("<proxy/>"));

        // Java args format
        let args_initial = "JAVA_ARGS=\"-Djava.awt.headless=true -Dhttp.proxyHost=127.0.0.1 -Dhttp.proxyPort=7890\"\n";
        let args_updated = apply_jenkins_proxy(args_initial, "http://127.0.0.1:8118");
        assert!(args_updated.contains("-Dhttp.proxyPort=8118"));
        let (args_cleaned, args_removed) = remove_jenkins_proxy(&args_updated);
        assert!(args_removed);
        assert!(!args_cleaned.contains("-Dhttp.proxyPort="));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_jenkins_proxy(dropin_initial, "http://127.0.0.1:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://127.0.0.1:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_jenkins_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_buildkite_agent_proxy() {
        // Standard buildkite-agent.cfg format
        let cfg_initial = "# buildkite config\ntags=\"queue=default\"\n";
        let cfg_updated = apply_buildkite_agent_proxy(cfg_initial, "http://127.0.0.1:7890");
        assert!(cfg_updated.contains("proxy=\"http://127.0.0.1:7890\""));
        let (cfg_cleaned, cfg_removed) = remove_buildkite_agent_proxy(&cfg_updated);
        assert!(cfg_removed);
        assert!(!cfg_cleaned.contains("proxy="));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:8080\"\n";
        let dropin_updated = apply_buildkite_agent_proxy(dropin_initial, "http://127.0.0.1:9090");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://127.0.0.1:9090\""));
        let (dropin_cleaned, dropin_removed) = remove_buildkite_agent_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_woodpecker_agent_proxy() {
        // Standard agent.conf format
        let conf_initial = "WOODPECKER_SERVER=\"grpc://woodpecker:9000\"\n";
        let conf_updated = apply_woodpecker_agent_proxy(conf_initial, "http://127.0.0.1:8118");
        assert!(conf_updated.contains("WOODPECKER_HTTP_PROXY=\"http://127.0.0.1:8118\""));
        let (conf_cleaned, conf_removed) = remove_woodpecker_agent_proxy(&conf_updated);
        assert!(conf_removed);
        assert!(!conf_cleaned.contains("WOODPECKER_HTTP_PROXY="));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_woodpecker_agent_proxy(dropin_initial, "http://10.0.0.1:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.1:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_woodpecker_agent_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_drone_runner_proxy() {
        // Standard config format
        let conf_initial = "DRONE_RPC_SERVER=\"https://drone.internal\"\n";
        let conf_updated = apply_drone_runner_proxy(conf_initial, "http://127.0.0.1:8118");
        assert!(conf_updated.contains("DRONE_HTTP_PROXY=\"http://127.0.0.1:8118\""));
        let (conf_cleaned, conf_removed) = remove_drone_runner_proxy(&conf_updated);
        assert!(conf_removed);
        assert!(!conf_cleaned.contains("DRONE_HTTP_PROXY="));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_drone_runner_proxy(dropin_initial, "http://10.0.0.1:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.1:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_drone_runner_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_act_runner_proxy() {
        // YAML config format
        let yaml_initial = "runner:\n  file: .runner\n";
        let yaml_updated = apply_act_runner_proxy(yaml_initial, "http://127.0.0.1:7890");
        assert!(yaml_updated.contains("envs:"));
        assert!(yaml_updated.contains("http_proxy: \"http://127.0.0.1:7890\""));
        let (yaml_cleaned, yaml_removed) = remove_act_runner_proxy(&yaml_updated);
        assert!(yaml_removed);
        assert!(!yaml_cleaned.contains("http_proxy:"));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_act_runner_proxy(dropin_initial, "http://10.0.0.1:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.1:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_act_runner_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_circleci_runner_proxy() {
        // YAML config format
        let yaml_initial = "runner:\n  name: test-runner\n";
        let yaml_updated = apply_circleci_runner_proxy(yaml_initial, "http://127.0.0.1:7890");
        assert!(yaml_updated.contains("proxy:"));
        assert!(yaml_updated.contains("http: \"http://127.0.0.1:7890\""));
        let (yaml_cleaned, yaml_removed) = remove_circleci_runner_proxy(&yaml_updated);
        assert!(yaml_removed);
        assert!(!yaml_cleaned.contains("proxy:"));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_circleci_runner_proxy(dropin_initial, "http://10.0.0.1:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.1:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_circleci_runner_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_azure_pipelines_agent_proxy() {
        // Plain .proxy file single-line format
        let proxy_initial = "http://127.0.0.1:7890\n";
        let (proxy_cleaned, proxy_removed) = remove_azure_pipelines_agent_proxy(proxy_initial);
        assert!(proxy_removed);
        assert_eq!(proxy_cleaned, "");

        let proxy_applied = apply_azure_pipelines_agent_proxy("", "http://127.0.0.1:8080");
        assert_eq!(proxy_applied, "http://127.0.0.1:8080\n");

        // Key-value .env format
        let env_initial = "VSTS_HTTP_PROXY=http://127.0.0.1:7890\n";
        let env_updated = apply_azure_pipelines_agent_proxy(env_initial, "http://10.0.0.2:9090");
        assert!(env_updated.contains("http_proxy=http://10.0.0.2:9090"));
        assert!(env_updated.contains("VSTS_HTTP_PROXY=http://10.0.0.2:9090"));
        let (env_cleaned, env_removed) = remove_azure_pipelines_agent_proxy(&env_updated);
        assert!(env_removed);
        assert!(!env_cleaned.contains("VSTS_HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_teamcity_agent_proxy() {
        // Properties format
        let props_initial = "serverUrl=http://teamcity:8111\n";
        let props_updated = apply_teamcity_agent_proxy(props_initial, "http://127.0.0.1:7890");
        assert!(props_updated.contains("env.HTTP_PROXY=http://127.0.0.1:7890"));
        assert!(props_updated.contains("teamcity.http.proxyHost=127.0.0.1"));
        assert!(props_updated.contains("teamcity.http.proxyPort=7890"));
        let (props_cleaned, props_removed) = remove_teamcity_agent_proxy(&props_updated);
        assert!(props_removed);
        assert!(!props_cleaned.contains("env.HTTP_PROXY="));
        assert!(!props_cleaned.contains("teamcity.http.proxyHost="));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_teamcity_agent_proxy(dropin_initial, "http://10.0.0.1:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.1:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_teamcity_agent_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_bamboo_agent_proxy() {
        // XML format
        let xml_initial = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<configuration>\n  <buildDirectory>/test</buildDirectory>\n</configuration>\n";
        let xml_updated = apply_bamboo_agent_proxy(xml_initial, "http://127.0.0.1:7890");
        assert!(xml_updated.contains("<httpProxyHost>127.0.0.1</httpProxyHost>"));
        assert!(xml_updated.contains("<httpProxyPort>7890</httpProxyPort>"));
        let (xml_cleaned, xml_removed) = remove_bamboo_agent_proxy(&xml_updated);
        assert!(xml_removed);
        assert!(!xml_cleaned.contains("<httpProxyHost>"));

        // Wrapper.conf format
        let wrap_initial = "wrapper.java.additional.1=-Dparam=true\n";
        let wrap_updated = apply_bamboo_agent_proxy(wrap_initial, "http://10.0.0.2:8080");
        assert!(wrap_updated.contains("-Dhttp.proxyHost=10.0.0.2"));
        assert!(wrap_updated.contains("-Dhttp.proxyPort=8080"));
        let (wrap_cleaned, wrap_removed) = remove_bamboo_agent_proxy(&wrap_updated);
        assert!(wrap_removed);
        assert!(!wrap_cleaned.contains("-Dhttp.proxyHost="));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_bamboo_agent_proxy(dropin_initial, "http://10.0.0.1:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.1:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_bamboo_agent_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_tfc_agent_proxy() {
        let env_initial = "TFC_AGENT_TOKEN=secret\n";
        let env_updated = apply_tfc_agent_proxy(env_initial, "http://127.0.0.1:7890");
        assert!(env_updated.contains("TFC_AGENT_HTTP_PROXY=\"http://127.0.0.1:7890\""));
        assert!(env_updated.contains("HTTP_PROXY=\"http://127.0.0.1:7890\""));
        let (env_cleaned, env_removed) = remove_tfc_agent_proxy(&env_updated);
        assert!(env_removed);
        assert!(!env_cleaned.contains("TFC_AGENT_HTTP_PROXY="));
        assert!(!env_cleaned.contains("HTTP_PROXY="));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_tfc_agent_proxy(dropin_initial, "http://10.0.0.1:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.1:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_tfc_agent_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_concourse_worker_proxy() {
        let env_initial = "CONCOURSE_GARDEN_NETWORK_POOL=10.254.0.0/16\n";
        let env_updated = apply_concourse_worker_proxy(env_initial, "http://127.0.0.1:7890");
        assert!(env_updated.contains("CONCOURSE_HTTP_PROXY=\"http://127.0.0.1:7890\""));
        assert!(env_updated.contains("HTTP_PROXY=\"http://127.0.0.1:7890\""));
        let (env_cleaned, env_removed) = remove_concourse_worker_proxy(&env_updated);
        assert!(env_removed);
        assert!(!env_cleaned.contains("CONCOURSE_HTTP_PROXY="));
        assert!(!env_cleaned.contains("HTTP_PROXY="));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"CONCOURSE_HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_concourse_worker_proxy(dropin_initial, "http://10.0.0.2:8080");
        assert!(dropin_updated.contains("Environment=\"CONCOURSE_HTTP_PROXY=http://10.0.0.2:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_concourse_worker_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("CONCOURSE_HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_tentacle_agent_proxy() {
        let xml_initial = r#"<?xml version="1.0" encoding="utf-8"?>
<octopus-settings xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <set key="Tentacle.Communication.ListenPort">10933</set>
</octopus-settings>
"#;
        let xml_updated = apply_tentacle_agent_proxy(xml_initial, "http://127.0.0.1:7890");
        assert!(xml_updated.contains("<set key=\"Tentacle.Services.ProxyHost\">127.0.0.1</set>"));
        assert!(xml_updated.contains("<set key=\"Tentacle.Services.ProxyPort\">7890</set>"));
        let (xml_cleaned, xml_removed) = remove_tentacle_agent_proxy(&xml_updated);
        assert!(xml_removed);
        assert!(!xml_cleaned.contains("Tentacle.Services.ProxyHost"));
        assert!(!xml_cleaned.contains("Tentacle.Services.ProxyPort"));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_tentacle_agent_proxy(dropin_initial, "http://10.0.0.1:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.1:8080\""));
        assert!(dropin_updated.contains("Environment=\"OCTOPUS_PROXY_HOST=10.0.0.1\""));
        assert!(dropin_updated.contains("Environment=\"OCTOPUS_PROXY_PORT=8080\""));
        let (dropin_cleaned, dropin_removed) = remove_tentacle_agent_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("OCTOPUS_PROXY_HOST="));
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_puppet_agent_proxy() {
        let ini_initial = r#"
[main]
certname = agent.local
server = puppet.local
"#;
        let ini_updated = apply_puppet_agent_proxy(ini_initial, "http://127.0.0.1:7890");
        assert!(ini_updated.contains("http_proxy_host = 127.0.0.1"));
        assert!(ini_updated.contains("http_proxy_port = 7890"));
        let (ini_cleaned, ini_removed) = remove_puppet_agent_proxy(&ini_updated);
        assert!(ini_removed);
        assert!(!ini_cleaned.contains("http_proxy_host"));
        assert!(!ini_cleaned.contains("http_proxy_port"));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_puppet_agent_proxy(dropin_initial, "http://10.0.0.2:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.2:8080\""));
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY_HOST=10.0.0.2\""));
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY_PORT=8080\""));
        let (dropin_cleaned, dropin_removed) = remove_puppet_agent_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
        assert!(!dropin_cleaned.contains("HTTP_PROXY_HOST="));
    }

    #[test]
    fn test_apply_and_remove_chef_client_proxy() {
        let ruby_initial = r#"
log_level :info
chef_server_url "https://chef.local/organizations/test"
"#;
        let ruby_updated = apply_chef_client_proxy(ruby_initial, "http://127.0.0.1:7890");
        assert!(ruby_updated.contains("http_proxy \"http://127.0.0.1:7890\""));
        assert!(ruby_updated.contains("https_proxy \"http://127.0.0.1:7890\""));
        let (ruby_cleaned, ruby_removed) = remove_chef_client_proxy(&ruby_updated);
        assert!(ruby_removed);
        assert!(!ruby_cleaned.contains("\nhttp_proxy \""));
        assert!(!ruby_cleaned.contains("\nhttps_proxy \""));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_chef_client_proxy(dropin_initial, "http://10.0.0.3:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.3:8080\""));
        assert!(dropin_updated.contains("Environment=\"HTTPS_PROXY=http://10.0.0.3:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_chef_client_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
        assert!(!dropin_cleaned.contains("HTTPS_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_salt_minion_proxy() {
        let yaml_initial = r#"
master: salt.local
id: node-1
"#;
        let yaml_updated = apply_salt_minion_proxy(yaml_initial, "http://127.0.0.1:7890");
        assert!(yaml_updated.contains("proxy_host: 127.0.0.1"));
        assert!(yaml_updated.contains("proxy_port: 7890"));
        let (yaml_cleaned, yaml_removed) = remove_salt_minion_proxy(&yaml_updated);
        assert!(yaml_removed);
        assert!(!yaml_cleaned.contains("proxy_host:"));
        assert!(!yaml_cleaned.contains("proxy_port:"));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_salt_minion_proxy(dropin_initial, "http://10.0.0.4:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.4:8080\""));
        assert!(dropin_updated.contains("Environment=\"PROXY_HOST=10.0.0.4\""));
        assert!(dropin_updated.contains("Environment=\"PROXY_PORT=8080\""));
        let (dropin_cleaned, dropin_removed) = remove_salt_minion_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
        assert!(!dropin_cleaned.contains("PROXY_HOST="));
    }

    #[test]
    fn test_apply_and_remove_datadog_agent_proxy() {
        let yaml_initial = r#"
api_key: 123456abcdef
site: datadoghq.com
"#;
        let yaml_updated = apply_datadog_agent_proxy(yaml_initial, "http://127.0.0.1:7890");
        assert!(yaml_updated.contains("proxy:"));
        assert!(yaml_updated.contains("http: http://127.0.0.1:7890"));
        assert!(yaml_updated.contains("https: http://127.0.0.1:7890"));
        let (yaml_cleaned, yaml_removed) = remove_datadog_agent_proxy(&yaml_updated);
        assert!(yaml_removed);
        assert!(!yaml_cleaned.contains("proxy:"));
        assert!(!yaml_cleaned.contains("http://127.0.0.1:7890"));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_datadog_agent_proxy(dropin_initial, "http://10.0.0.5:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.5:8080\""));
        assert!(dropin_updated.contains("Environment=\"DD_PROXY_HTTP=http://10.0.0.5:8080\""));
        assert!(dropin_updated.contains("Environment=\"DD_PROXY_HTTPS=http://10.0.0.5:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_datadog_agent_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
        assert!(!dropin_cleaned.contains("DD_PROXY_HTTP="));
    }

    #[test]
    fn test_apply_and_remove_newrelic_infra_proxy() {
        let yaml_initial = r#"
license_key: 654321fedcba
log:
  level: info
"#;
        let yaml_updated = apply_newrelic_infra_proxy(yaml_initial, "http://127.0.0.1:7890");
        assert!(yaml_updated.contains("proxy: http://127.0.0.1:7890"));
        let (yaml_cleaned, yaml_removed) = remove_newrelic_infra_proxy(&yaml_updated);
        assert!(yaml_removed);
        assert!(!yaml_cleaned.contains("proxy:"));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_newrelic_infra_proxy(dropin_initial, "http://10.0.0.6:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.6:8080\""));
        assert!(dropin_updated.contains("Environment=\"NRIA_PROXY=http://10.0.0.6:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_newrelic_infra_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
        assert!(!dropin_cleaned.contains("NRIA_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_elastic_agent_proxy() {
        let yaml_initial = r#"
outputs:
  default:
    type: elasticsearch
    hosts: ["https://es:9200"]
"#;
        let yaml_updated = apply_elastic_agent_proxy(yaml_initial, "http://127.0.0.1:7890");
        assert!(yaml_updated.contains("proxy_url: http://127.0.0.1:7890"));
        let (yaml_cleaned, yaml_removed) = remove_elastic_agent_proxy(&yaml_updated);
        assert!(yaml_removed);
        assert!(!yaml_cleaned.contains("proxy_url:"));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_elastic_agent_proxy(dropin_initial, "http://10.0.0.7:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.7:8080\""));
        assert!(dropin_updated.contains("Environment=\"ELASTIC_AGENT_PROXY=http://10.0.0.7:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_elastic_agent_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
        assert!(!dropin_cleaned.contains("ELASTIC_AGENT_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_grafana_alloy_proxy() {
        let river_initial = r#"
prometheus.remote_write "default" {
  endpoint {
    url = "https://prometheus-prod.grafana.net/api/prom/push"
  }
}
"#;
        let river_updated = apply_grafana_alloy_proxy(river_initial, "http://127.0.0.1:7890");
        assert!(river_updated.contains("proxy_url = \"http://127.0.0.1:7890\""));
        let (river_cleaned, river_removed) = remove_grafana_alloy_proxy(&river_updated);
        assert!(river_removed);
        assert!(!river_cleaned.contains("proxy_url"));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_grafana_alloy_proxy(dropin_initial, "http://10.0.0.8:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.8:8080\""));
        assert!(dropin_updated.contains("Environment=\"ALLOY_HTTP_PROXY=http://10.0.0.8:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_grafana_alloy_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
        assert!(!dropin_cleaned.contains("ALLOY_HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_otelcol_proxy() {
        let yaml_initial = r#"
receivers:
  otlp:
    protocols:
      grpc:
exporters:
  otlp:
    endpoint: "otel.example.com:4317"
"#;
        let yaml_updated = apply_otelcol_proxy(yaml_initial, "http://127.0.0.1:7890");
        assert!(yaml_updated.contains("proxy_url: \"http://127.0.0.1:7890\""));
        let (yaml_cleaned, yaml_removed) = remove_otelcol_proxy(&yaml_updated);
        assert!(yaml_removed);
        assert!(!yaml_cleaned.contains("proxy_url"));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_otelcol_proxy(dropin_initial, "http://10.0.0.9:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.9:8080\""));
        assert!(dropin_updated.contains("Environment=\"OTEL_EXPORTER_OTLP_PROXY_URL=http://10.0.0.9:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_otelcol_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
        assert!(!dropin_cleaned.contains("OTEL_EXPORTER_OTLP_PROXY_URL="));
    }

    #[test]
    fn test_apply_and_remove_vector_proxy() {
        let yaml_initial = r#"
sources:
  in:
    type: demo_logs
sinks:
  out:
    type: datadog_logs
"#;
        let yaml_updated = apply_vector_proxy(yaml_initial, "http://127.0.0.1:7890");
        assert!(yaml_updated.contains("http: \"http://127.0.0.1:7890\""));
        assert!(yaml_updated.contains("https: \"http://127.0.0.1:7890\""));
        let (yaml_cleaned, yaml_removed) = remove_vector_proxy(&yaml_updated);
        assert!(yaml_removed);
        assert!(!yaml_cleaned.contains("http: \"http://127.0.0.1:7890\""));

        // TOML format
        let toml_initial = "[api]\nenabled = true\n";
        let toml_updated = apply_vector_proxy(toml_initial, "http://127.0.0.1:7890");
        assert!(toml_updated.contains("http = \"http://127.0.0.1:7890\""));
        let (toml_cleaned, toml_removed) = remove_vector_proxy(&toml_updated);
        assert!(toml_removed);
        assert!(!toml_cleaned.contains("http = \"http://127.0.0.1:7890\""));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_vector_proxy(dropin_initial, "http://10.0.0.10:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.10:8080\""));
        assert!(dropin_updated.contains("Environment=\"VECTOR_PROXY=http://10.0.0.10:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_vector_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
        assert!(!dropin_cleaned.contains("VECTOR_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_fluent_bit_proxy() {
        let conf_initial = r#"
[SERVICE]
    Flush        1
    Log_Level    info
"#;
        let conf_updated = apply_fluent_bit_proxy(conf_initial, "http://127.0.0.1:7890");
        assert!(conf_updated.contains("HTTP_Proxy  http://127.0.0.1:7890"));
        let (conf_cleaned, conf_removed) = remove_fluent_bit_proxy(&conf_updated);
        assert!(conf_removed);
        assert!(!conf_cleaned.contains("HTTP_Proxy"));

        // YAML format
        let yaml_initial = "service:\n  flush: 1\n";
        let yaml_updated = apply_fluent_bit_proxy(yaml_initial, "http://127.0.0.1:7890");
        assert!(yaml_updated.contains("http_proxy: \"http://127.0.0.1:7890\""));
        let (yaml_cleaned, yaml_removed) = remove_fluent_bit_proxy(&yaml_updated);
        assert!(yaml_removed);
        assert!(!yaml_cleaned.contains("http_proxy:"));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_fluent_bit_proxy(dropin_initial, "http://10.0.0.11:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.11:8080\""));
        assert!(dropin_updated.contains("Environment=\"FLB_HTTP_PROXY=http://10.0.0.11:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_fluent_bit_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
        assert!(!dropin_cleaned.contains("FLB_HTTP_PROXY="));
    }

    #[test]
    fn test_apply_and_remove_fluentd_proxy() {
        let conf_initial = r#"
<system>
  workers 2
</system>
"#;
        let conf_updated = apply_fluentd_proxy(conf_initial, "http://127.0.0.1:7890");
        assert!(conf_updated.contains("http_proxy http://127.0.0.1:7890"));
        let (conf_cleaned, conf_removed) = remove_fluentd_proxy(&conf_updated);
        assert!(conf_removed);
        assert!(!conf_cleaned.contains("http_proxy"));

        // Systemd drop-in format
        let dropin_initial = "[Service]\nEnvironment=\"HTTP_PROXY=http://127.0.0.1:7890\"\n";
        let dropin_updated = apply_fluentd_proxy(dropin_initial, "http://10.0.0.12:8080");
        assert!(dropin_updated.contains("Environment=\"HTTP_PROXY=http://10.0.0.12:8080\""));
        assert!(dropin_updated.contains("Environment=\"FLUENTD_HTTP_PROXY=http://10.0.0.12:8080\""));
        let (dropin_cleaned, dropin_removed) = remove_fluentd_proxy(&dropin_updated);
        assert!(dropin_removed);
        assert!(!dropin_cleaned.contains("HTTP_PROXY="));
        assert!(!dropin_cleaned.contains("FLUENTD_HTTP_PROXY="));
    }
}
