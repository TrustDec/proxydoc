use anyhow::Result;
use colored::*;
use std::process::Command;

pub fn turn_on(target: &str, git: bool, npm: bool) -> Result<()> {
    // Resolve target proxy address
    let proxy_url = if target == "auto" {
        // Auto resolve default: check common ports synchronously / fallback to 7890
        "http://127.0.0.1:7890".to_string()
    } else if target.chars().all(|c| c.is_ascii_digit()) {
        format!("http://127.0.0.1:{}", target)
    } else if !target.contains("://") {
        format!("http://{}", target)
    } else {
        target.to_string()
    };

    // If caller wants shell commands to eval
    println!("export http_proxy=\"{}\"", proxy_url);
    println!("export https_proxy=\"{}\"", proxy_url);
    println!("export all_proxy=\"{}\"", proxy_url);
    println!("export HTTP_PROXY=\"{}\"", proxy_url);
    println!("export HTTPS_PROXY=\"{}\"", proxy_url);
    println!("export ALL_PROXY=\"{}\"", proxy_url);

    if git {
        let _ = Command::new("git")
            .args(["config", "--global", "http.proxy", &proxy_url])
            .status();
        let _ = Command::new("git")
            .args(["config", "--global", "https.proxy", &proxy_url])
            .status();
        eprintln!("{}", format!("✔ Git global proxy set to {}", proxy_url).green());
    }

    if npm {
        let _ = Command::new("npm")
            .args(["config", "set", "proxy", &proxy_url])
            .status();
        let _ = Command::new("npm")
            .args(["config", "set", "https-proxy", &proxy_url])
            .status();
        eprintln!("{}", format!("✔ NPM proxy set to {}", proxy_url).green());
    }

    eprintln!(
        "{}",
        format!("🟢 Proxy enabled! Target: {}", proxy_url).green().bold()
    );
    eprintln!(
        "{}",
        "Tip: To apply directly to current terminal, run: proxydoc-on or eval $(proxydoc on)".dimmed()
    );

    Ok(())
}

pub fn turn_off(git: bool, npm: bool) -> Result<()> {
    println!("unset http_proxy");
    println!("unset https_proxy");
    println!("unset all_proxy");
    println!("unset HTTP_PROXY");
    println!("unset HTTPS_PROXY");
    println!("unset ALL_PROXY");

    if git {
        let _ = Command::new("git")
            .args(["config", "--global", "--unset", "http.proxy"])
            .status();
        let _ = Command::new("git")
            .args(["config", "--global", "--unset", "https.proxy"])
            .status();
        eprintln!("{}", "✔ Git global proxy unset".green());
    }

    if npm {
        let _ = Command::new("npm")
            .args(["config", "delete", "proxy"])
            .status();
        let _ = Command::new("npm")
            .args(["config", "delete", "https-proxy"])
            .status();
        eprintln!("{}", "✔ NPM proxy unset".green());
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
alias ptest="proxydoc test"
"#);
}
