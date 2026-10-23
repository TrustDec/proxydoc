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

pub async fn turn_on(
    target: &str,
    git: bool,
    npm: bool,
    pip: bool,
    docker: bool,
    all: bool,
    shell: Option<&str>,
) -> Result<()> {
    let set_git = git || all;
    let set_npm = npm || all;
    let set_pip = pip || all;
    let set_docker = docker || all;

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

    if set_npm {
        let _ = Command::new("npm").args(["config", "set", "proxy", &proxy_url]).status();
        let _ = Command::new("npm").args(["config", "set", "https-proxy", &proxy_url]).status();
        eprintln!("{}", format!("✔ NPM proxy set to {}", proxy_url).green());
    }

    if set_pip {
        let _ = Command::new("pip").args(["config", "set", "global.proxy", &proxy_url]).status();
        eprintln!("{}", format!("✔ Pip global proxy set to {}", proxy_url).green());
    }

    if set_docker {
        match set_docker_proxy(&proxy_url) {
            Ok(_) => eprintln!("{}", format!("✔ Docker client proxy (~/.docker/config.json) set to {}", proxy_url).green()),
            Err(e) => eprintln!("{} Failed to update Docker config: {}", "▲".yellow(), e),
        }
    }

    eprintln!("{}", format!("🟢 Proxy enabled! Target: {}", proxy_url).green().bold());
    eprintln!("{}", "Tip: To apply directly to current terminal, run: eval $(proxydoc on)".dimmed());

    Ok(())
}

pub fn turn_off(
    git: bool,
    npm: bool,
    pip: bool,
    docker: bool,
    all: bool,
    shell: Option<&str>,
) -> Result<()> {
    let unset_git = git || all;
    let unset_npm = npm || all;
    let unset_pip = pip || all;
    let unset_docker = docker || all;

    let target_shell = detect_shell(shell);
    for line in generate_shell_unset(target_shell) {
        println!("{}", line);
    }

    if unset_git {
        let _ = Command::new("git").args(["config", "--global", "--unset", "http.proxy"]).status();
        let _ = Command::new("git").args(["config", "--global", "--unset", "https.proxy"]).status();
        eprintln!("{}", "✔ Git global proxy unset".green());
    }

    if unset_npm {
        let _ = Command::new("npm").args(["config", "delete", "proxy"]).status();
        let _ = Command::new("npm").args(["config", "delete", "https-proxy"]).status();
        eprintln!("{}", "✔ NPM proxy unset".green());
    }

    if unset_pip {
        let _ = Command::new("pip").args(["config", "unset", "global.proxy"]).status();
        eprintln!("{}", "✔ Pip global proxy unset".green());
    }

    if unset_docker {
        match unset_docker_proxy() {
            Ok(true) => eprintln!("{}", "✔ Docker client proxy unset from ~/.docker/config.json".green()),
            Ok(false) => eprintln!("{}", "○ Docker client proxy was not set".dimmed()),
            Err(e) => eprintln!("{} Failed to update Docker config: {}", "▲".yellow(), e),
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
}
