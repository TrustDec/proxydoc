use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct TargetResult {
    #[tabled(rename = "Service")]
    name: &'static str,
    #[tabled(rename = "Endpoint")]
    url: &'static str,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Latency")]
    latency: String,
}

const TEST_TARGETS: &[(&str, &str)] = &[
    ("GitHub", "https://api.github.com/zen"),
    ("Google", "https://www.google.com/generate_204"),
    ("Cloudflare", "https://1.1.1.1/cdn-cgi/trace"),
    ("OpenAI API", "https://api.openai.com/v1/models"),
    ("Anthropic API", "https://api.anthropic.com/v1/messages"),
    ("HuggingFace", "https://huggingface.co/api/status"),
];

pub async fn run(custom_proxy: Option<String>) -> Result<()> {
    let proxy_info = match &custom_proxy {
        Some(p) => format!("Custom Proxy: {}", p.yellow()),
        None => match std::env::var("https_proxy").or_else(|_| std::env::var("HTTPS_PROXY")) {
            Ok(p) => format!("Env Proxy: {}", p.yellow()),
            Err(_) => "Direct (No env proxy set)".dimmed().to_string(),
        },
    };

    println!("\n⚡ Testing Global Network & AI API Connectivity...");
    println!("   Active Proxy: {}\n", proxy_info);

    let pb = ProgressBar::new(TEST_TARGETS.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut rows = Vec::new();

    for &(name, url) in TEST_TARGETS {
        pb.set_message(format!("Connecting to {}...", name));

        let mut client_builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .danger_accept_invalid_certs(false);

        if let Some(ref p) = custom_proxy {
            if let Ok(proxy) = reqwest::Proxy::all(p) {
                client_builder = client_builder.proxy(proxy);
            }
        }

        let client = client_builder.build()?;
        let start = Instant::now();
        let res = client.get(url).send().await;
        let elapsed = start.elapsed();

        let (status, latency) = match res {
            Ok(resp) => {
                let code = resp.status().as_u16();
                // 401/403/405 for API endpoints like OpenAI/Anthropic without key means connection succeeded!
                let code_str = if code < 400 || code == 401 || code == 403 || code == 405 {
                    format!("● OK ({})", code).green().bold().to_string()
                } else {
                    format!("▲ HTTP {}", code).yellow().to_string()
                };
                let lat_str = format!("{:.0} ms", elapsed.as_millis());
                (code_str, lat_str)
            }
            Err(e) => {
                let err_desc = if e.is_timeout() {
                    "Timeout (5s)".to_string()
                } else if e.is_connect() {
                    "Connect Failed".to_string()
                } else {
                    "Error".to_string()
                };
                (format!("✕ {}", err_desc).red().bold().to_string(), "-".dimmed().to_string())
            }
        };

        rows.push(TargetResult {
            name,
            url,
            status,
            latency,
        });

        pb.inc(1);
    }

    pb.finish_and_clear();

    let table = Table::new(rows).to_string();
    println!("{}\n", table);

    Ok(())
}
