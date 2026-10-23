use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tabled::{Table, Tabled};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EgressInfo {
    pub ip: String,
    pub location: String,
    pub colo: String,
    pub warp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct TargetResult {
    #[tabled(rename = "Service")]
    pub name: String,
    #[tabled(rename = "Endpoint")]
    pub url: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Latency")]
    pub latency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckerReport {
    pub proxy_used: Option<String>,
    pub egress: Option<EgressInfo>,
    pub targets: Vec<TargetResult>,
}

pub const TEST_TARGETS: &[(&str, &str)] = &[
    ("GitHub", "https://api.github.com/zen"),
    ("Google", "https://www.google.com/generate_204"),
    ("Cloudflare", "https://1.1.1.1/cdn-cgi/trace"),
    ("OpenAI API", "https://api.openai.com/v1/models"),
    ("Anthropic API", "https://api.anthropic.com/v1/messages"),
    ("HuggingFace", "https://huggingface.co/api/status"),
];

pub fn parse_cf_trace(body: &str) -> EgressInfo {
    let mut ip = "Unknown".to_string();
    let mut location = "Unknown".to_string();
    let mut colo = "Unknown".to_string();
    let mut warp = false;

    for line in body.lines() {
        if let Some((k, v)) = line.split_once('=') {
            match k.trim() {
                "ip" => ip = v.trim().to_string(),
                "loc" => location = v.trim().to_string(),
                "colo" => colo = v.trim().to_string(),
                "warp" => warp = v.trim() == "on",
                _ => {}
            }
        }
    }

    EgressInfo {
        ip,
        location,
        colo,
        warp,
    }
}

pub async fn run(custom_proxy: Option<String>, json_output: bool) -> Result<()> {
    let active_proxy_str = match &custom_proxy {
        Some(p) => Some(p.clone()),
        None => std::env::var("https_proxy")
            .or_else(|_| std::env::var("HTTPS_PROXY"))
            .ok(),
    };

    let proxy_display = match &active_proxy_str {
        Some(p) => format!("Proxy: {}", p.yellow()),
        None => "Direct (No proxy set)".dimmed().to_string(),
    };

    if !json_output {
        println!("\n⚡ Testing Global Network & AI API Connectivity...");
        println!("   Route: {}\n", proxy_display);
    }

    let mut client_builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(false);

    if let Some(ref p) = active_proxy_str {
        if let Ok(proxy) = reqwest::Proxy::all(p) {
            client_builder = client_builder.proxy(proxy);
        }
    }

    let client = client_builder.build()?;

    let pb = if !json_output {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        bar.set_message("Testing endpoints concurrently...");
        bar.enable_steady_tick(Duration::from_millis(80));
        Some(bar)
    } else {
        None
    };

    // Concurrently test all targets
    let mut join_set = tokio::task::JoinSet::new();

    for (idx, &(name, url)) in TEST_TARGETS.iter().enumerate() {
        let client_clone = client.clone();
        join_set.spawn(async move {
            let start = Instant::now();
            let res = client_clone.get(url).send().await;
            let elapsed = start.elapsed();

            let mut trace_body: Option<String> = None;
            let (status, latency) = match res {
                Ok(resp) => {
                    let code = resp.status().as_u16();
                    if url.contains("cdn-cgi/trace") {
                        if let Ok(text) = resp.text().await {
                            trace_body = Some(text);
                        }
                    }

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

            (idx, name.to_string(), url.to_string(), status, latency, trace_body)
        });
    }

    let mut results: Vec<Option<(TargetResult, Option<String>)>> = vec![None; TEST_TARGETS.len()];

    while let Some(res) = join_set.join_next().await {
        if let Ok((idx, name, url, status, latency, trace_body)) = res {
            results[idx] = Some((
                TargetResult {
                    name,
                    url,
                    status,
                    latency,
                },
                trace_body,
            ));
        }
    }

    if let Some(bar) = pb {
        bar.finish_and_clear();
    }

    let mut rows = Vec::new();
    let mut egress: Option<EgressInfo> = None;

    for item in results.into_iter().flatten() {
        let (row, trace) = item;
        if let Some(text) = trace {
            egress = Some(parse_cf_trace(&text));
        }
        rows.push(row);
    }

    if json_output {
        let report = CheckerReport {
            proxy_used: active_proxy_str,
            egress,
            targets: rows,
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    // Display Outbound Egress IP and Location if available
    if let Some(ref eg) = egress {
        let warp_badge = if eg.warp { " (WARP Enabled)".cyan() } else { "".clear() };
        println!(
            "{} {} {} (Country: {}, Datacenter: {}){}\n",
            "🌐".cyan(),
            "Outbound Egress IP:".bold(),
            eg.ip.green().bold(),
            eg.location.yellow().bold(),
            eg.colo.cyan(),
            warp_badge
        );
    }

    let table = Table::new(&rows).to_string();
    println!("{}\n", table);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cf_trace() {
        let raw = "fl=619f83\nh=1.1.1.1\nip=104.28.194.12\nts=1792785716.000\ncolo=LAX\nloc=US\nwarp=on\n";
        let info = parse_cf_trace(raw);
        assert_eq!(info.ip, "104.28.194.12");
        assert_eq!(info.location, "US");
        assert_eq!(info.colo, "LAX");
        assert!(info.warp);
    }

    #[test]
    fn test_parse_cf_trace_defaults() {
        let raw = "";
        let info = parse_cf_trace(raw);
        assert_eq!(info.ip, "Unknown");
        assert_eq!(info.location, "Unknown");
        assert_eq!(info.colo, "Unknown");
        assert!(!info.warp);
    }
}
