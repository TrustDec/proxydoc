use anyhow::Result;
use colored::*;
use std::process::Command;

use crate::doctor::get_macos_system_proxy;
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
    let vars = ["http_proxy", "https_proxy", "all_proxy", "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY"];
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
        "http_proxy", "https_proxy", "all_proxy", "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY",
        "no_proxy", "NO_PROXY",
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
    default_map.insert("httpProxy".to_string(), serde_json::Value::String(proxy_url.to_string()));
    default_map.insert("httpsProxy".to_string(), serde_json::Value::String(proxy_url.to_string()));
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
            h.trim_start_matches("http://").trim_start_matches("https://").to_string()
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
                    lines.push("systemProp.http.nonProxyHosts=localhost|127.0.0.1|*.local".to_string());
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

    format!("<settings>\n  <proxies>\n{}\n  </proxies>\n{}\n</settings>\n", proxy_snippet, content)
}

pub fn remove_maven_proxy_xml(content: &str) -> (String, bool) {
    let mut modified = false;
    let mut res = content.to_string();

    if let Some(doc_idx) = res.find("<id>proxydoc-proxy</id>") {
        if let Some(p_start) = res[..doc_idx].rfind("<proxy>") {
            if let Some(p_end) = res[doc_idx..].find("</proxy>") {
                let end_pos = doc_idx + p_end + 8;
                res = format!("{}{}", &res[..p_start], res[end_pos..].trim_start_matches('\n'));
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
    let _ = Command::new("yarn").args(["config", "set", "proxy", proxy_url]).output();
    let _ = Command::new("yarn").args(["config", "set", "https-proxy", proxy_url]).output();
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
    let _ = Command::new("yarn").args(["config", "delete", "proxy"]).output();
    let _ = Command::new("yarn").args(["config", "delete", "https-proxy"]).output();
    Ok(removed)
}

pub fn apply_condarc_proxy(content: &str, proxy_url: &str) -> String {
    let mut lines = Vec::new();
    let mut in_proxy_servers = false;
    let mut proxy_section_inserted = false;

    let proxy_block = format!("proxy_servers:\n  http: {}\n  https: {}", proxy_url, proxy_url);

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

pub async fn turn_on(
    target: &str,
    git: bool,
    cargo: bool,
    npm: bool,
    yarn: bool,
    pip: bool,
    conda: bool,
    docker: bool,
    gradle: bool,
    maven: bool,
    all: bool,
    shell: Option<&str>,
) -> Result<()> {
    let set_git = git || all;
    let set_cargo = cargo || all;
    let set_npm = npm || all;
    let set_yarn = yarn || all;
    let set_pip = pip || all;
    let set_conda = conda || all;
    let set_docker = docker || all;
    let set_gradle = gradle || all;
    let set_maven = maven || all;

    // Resolve target proxy address
    let proxy_url = if target == "auto" {
        // 1. Try system proxy from scutil
        if let Some(sys) = get_macos_system_proxy() {
            if sys.http_enabled && sys.http_port > 0 {
                format!("http://{}:{}", sys.http_host, sys.http_port)
            } else {
                // 2. Try detected daemon port
                if let Some(p) = scanner::find_first_active_proxy().await {
                    format!("http://127.0.0.1:{}", p.port)
                } else {
                    "http://127.0.0.1:7890".to_string()
                }
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
        let _ = Command::new("git").args(["config", "--global", "http.proxy", &proxy_url]).status();
        let _ = Command::new("git").args(["config", "--global", "https.proxy", &proxy_url]).status();
        eprintln!("{}", format!("✔ Git global proxy set to {}", proxy_url).green());
    }

    if set_cargo {
        match set_cargo_proxy(&proxy_url) {
            Ok(_) => eprintln!("{}", format!("✔ Cargo [http] proxy (~/.cargo/config.toml) set to {}", proxy_url).green()),
            Err(e) => eprintln!("{} Failed to update Cargo config: {}", "▲".yellow(), e),
        }
    }

    if set_npm {
        let _ = Command::new("npm").args(["config", "set", "proxy", &proxy_url]).status();
        let _ = Command::new("npm").args(["config", "set", "https-proxy", &proxy_url]).status();
        eprintln!("{}", format!("✔ NPM proxy set to {}", proxy_url).green());
    }

    if set_yarn {
        match set_yarn_proxy(&proxy_url) {
            Ok(_) => eprintln!("{}", format!("✔ Yarn proxy (~/.yarnrc / ~/.yarnrc.yml) set to {}", proxy_url).green()),
            Err(e) => eprintln!("{} Failed to update Yarn config: {}", "▲".yellow(), e),
        }
    }

    if set_pip {
        let _ = Command::new("pip").args(["config", "set", "global.proxy", &proxy_url]).status();
        eprintln!("{}", format!("✔ Pip global proxy set to {}", proxy_url).green());
    }

    if set_conda {
        match set_conda_proxy(&proxy_url) {
            Ok(_) => eprintln!("{}", format!("✔ Conda proxy (~/.condarc) set to {}", proxy_url).green()),
            Err(e) => eprintln!("{} Failed to update Conda config: {}", "▲".yellow(), e),
        }
    }

    if set_docker {
        match set_docker_proxy(&proxy_url) {
            Ok(_) => eprintln!("{}", format!("✔ Docker client proxy (~/.docker/config.json) set to {}", proxy_url).green()),
            Err(e) => eprintln!("{} Failed to update Docker config: {}", "▲".yellow(), e),
        }
    }

    if set_gradle {
        match set_gradle_proxy(&proxy_url) {
            Ok(_) => eprintln!("{}", format!("✔ Gradle global proxy (~/.gradle/gradle.properties) set to {}", proxy_url).green()),
            Err(e) => eprintln!("{} Failed to update Gradle config: {}", "▲".yellow(), e),
        }
    }

    if set_maven {
        match set_maven_proxy(&proxy_url) {
            Ok(_) => eprintln!("{}", format!("✔ Maven global proxy (~/.m2/settings.xml) set to {}", proxy_url).green()),
            Err(e) => eprintln!("{} Failed to update Maven config: {}", "▲".yellow(), e),
        }
    }

    eprintln!("{}", format!("🟢 Proxy enabled! Target: {}", proxy_url).green().bold());
    eprintln!("{}", "Tip: To apply directly to current terminal, run: eval $(proxydoc on)".dimmed());

    Ok(())
}

pub fn turn_off(
    git: bool,
    cargo: bool,
    npm: bool,
    yarn: bool,
    pip: bool,
    conda: bool,
    docker: bool,
    gradle: bool,
    maven: bool,
    all: bool,
    shell: Option<&str>,
) -> Result<()> {
    let unset_git = git || all;
    let unset_cargo = cargo || all;
    let unset_npm = npm || all;
    let unset_yarn = yarn || all;
    let unset_pip = pip || all;
    let unset_conda = conda || all;
    let unset_docker = docker || all;
    let unset_gradle = gradle || all;
    let unset_maven = maven || all;

    let target_shell = detect_shell(shell);
    for line in generate_shell_unset(target_shell) {
        println!("{}", line);
    }

    if unset_git {
        let _ = Command::new("git").args(["config", "--global", "--unset", "http.proxy"]).status();
        let _ = Command::new("git").args(["config", "--global", "--unset", "https.proxy"]).status();
        eprintln!("{}", "✔ Git global proxy unset".green());
    }

    if unset_cargo {
        match unset_cargo_proxy() {
            Ok(true) => eprintln!("{}", "✔ Cargo proxy unset from ~/.cargo/config.toml".green()),
            Ok(false) => eprintln!("{}", "○ Cargo proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update Cargo config: {}", "▲".yellow(), e),
        }
    }

    if unset_npm {
        let _ = Command::new("npm").args(["config", "delete", "proxy"]).status();
        let _ = Command::new("npm").args(["config", "delete", "https-proxy"]).status();
        eprintln!("{}", "✔ NPM proxy unset".green());
    }

    if unset_yarn {
        match unset_yarn_proxy() {
            Ok(true) => eprintln!("{}", "✔ Cleared Yarn proxy (~/.yarnrc / ~/.yarnrc.yml)".green()),
            Ok(false) => eprintln!("{}", "○ Yarn proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to clear Yarn config: {}", "▲".yellow(), e),
        }
    }

    if unset_pip {
        let _ = Command::new("pip").args(["config", "unset", "global.proxy"]).status();
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
            Ok(true) => eprintln!("{}", "✔ Docker client proxy unset from ~/.docker/config.json".green()),
            Ok(false) => eprintln!("{}", "○ Docker client proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update Docker config: {}", "▲".yellow(), e),
        }
    }

    if unset_gradle {
        match unset_gradle_proxy() {
            Ok(true) => eprintln!("{}", "✔ Gradle global proxy unset from ~/.gradle/gradle.properties".green()),
            Ok(false) => eprintln!("{}", "○ Gradle global proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update Gradle config: {}", "▲".yellow(), e),
        }
    }

    if unset_maven {
        match unset_maven_proxy() {
            Ok(true) => eprintln!("{}", "✔ Maven global proxy unset from ~/.m2/settings.xml".green()),
            Ok(false) => eprintln!("{}", "○ Maven global proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update Maven config: {}", "▲".yellow(), e),
        }
    }

    eprintln!("{}", "🔴 Proxy disabled across terminal session!".yellow().bold());

    Ok(())
}

pub fn print_init_script(shell: Option<&str>) {
    let target_shell = detect_shell(shell);
    match target_shell {
        ShellType::Fish => {
            println!(r#"# proxydoc shell integration for fish
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
"#);
        }
        ShellType::PowerShell => {
            println!(r#"# proxydoc shell integration for PowerShell
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
"#);
        }
        ShellType::Bash => {
            println!(r#"# proxydoc shell integration for bash/zsh
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
"#);
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
        assert!(lines.contains(&"export no_proxy=\"localhost,127.0.0.1,::1,localaddress,.local\"".to_string()));
    }

    #[test]
    fn test_generate_shell_export_fish() {
        let lines = generate_shell_export("http://127.0.0.1:7890", ShellType::Fish);
        assert!(lines.contains(&"set -gx http_proxy \"http://127.0.0.1:7890\";".to_string()));
        assert!(lines.contains(&"set -gx no_proxy \"localhost,127.0.0.1,::1,localaddress,.local\";".to_string()));
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
            root.pointer("/proxies/default/httpProxy").unwrap().as_str().unwrap(),
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
        assert_eq!(extract_host_port("http://127.0.0.1:7890"), ("127.0.0.1".to_string(), 7890));
        assert_eq!(extract_host_port("127.0.0.1:1082"), ("127.0.0.1".to_string(), 1082));
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
}
