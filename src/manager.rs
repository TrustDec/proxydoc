use anyhow::Result;
use colored::*;
use std::process::Command;

use crate::doctor::get_macos_system_proxy;
use crate::scanner;

pub async fn turn_on(target: &str, git: bool, npm: bool, pip: bool, all: bool) -> Result<()> {
    let set_git = git || all;
    let set_npm = npm || all;
    let set_pip = pip || all;

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

    println!("export http_proxy=\"{}\"", proxy_url);
    println!("export https_proxy=\"{}\"", proxy_url);
    println!("export all_proxy=\"{}\"", proxy_url);
    println!("export HTTP_PROXY=\"{}\"", proxy_url);
    println!("export HTTPS_PROXY=\"{}\"", proxy_url);
    println!("export ALL_PROXY=\"{}\"", proxy_url);
    println!("export no_proxy=\"localhost,127.0.0.1,::1,localaddress,.local\"");
    println!("export NO_PROXY=\"localhost,127.0.0.1,::1,localaddress,.local\"");

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

    eprintln!("{}", format!("🟢 Proxy enabled! Target: {}", proxy_url).green().bold());
    eprintln!("{}", "Tip: To apply directly to current terminal, run: eval $(proxydoc on)".dimmed());

    Ok(())
}

pub fn turn_off(git: bool, npm: bool, pip: bool, all: bool) -> Result<()> {
    let unset_git = git || all;
    let unset_npm = npm || all;
    let unset_pip = pip || all;

    println!("unset http_proxy");
    println!("unset https_proxy");
    println!("unset all_proxy");
    println!("unset HTTP_PROXY");
    println!("unset HTTPS_PROXY");
    println!("unset ALL_PROXY");
    println!("unset no_proxy");
    println!("unset NO_PROXY");

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

    eprintln!("{}", "🔴 Proxy disabled across terminal session!".yellow().bold());

    Ok(())
}

pub fn print_init_script() {
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
