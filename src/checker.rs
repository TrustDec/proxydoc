use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tabled::{Table, Tabled};

use crate::scanner;

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

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct ComparisonRow {
    #[tabled(rename = "Service")]
    pub name: String,
    #[tabled(rename = "Direct Status")]
    pub direct_status: String,
    #[tabled(rename = "Direct Latency")]
    pub direct_latency: String,
    #[tabled(rename = "Proxy Status")]
    pub proxy_status: String,
    #[tabled(rename = "Proxy Latency")]
    pub proxy_latency: String,
    #[tabled(rename = "Verdict")]
    pub verdict: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckerReport {
    pub proxy_used: Option<String>,
    pub egress: Option<EgressInfo>,
    pub targets: Vec<TargetResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub proxy_used: String,
    pub direct_egress: Option<EgressInfo>,
    pub proxy_egress: Option<EgressInfo>,
    pub comparison: Vec<ComparisonRow>,
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

#[derive(Debug, Clone)]
pub struct RawTargetResult {
    pub name: String,
    pub url: String,
    pub status: String,
    pub latency_ms: Option<u128>,
    pub latency_str: String,
    pub success: bool,
    pub trace_body: Option<String>,
}

pub async fn test_endpoints(client: &reqwest::Client) -> (Vec<RawTargetResult>, Option<EgressInfo>) {
    let mut join_set = tokio::task::JoinSet::new();

    for (idx, &(name, url)) in TEST_TARGETS.iter().enumerate() {
        let client_clone = client.clone();
        join_set.spawn(async move {
            let start = Instant::now();
            let res = client_clone.get(url).send().await;
            let elapsed = start.elapsed();

            let mut trace_body: Option<String> = None;
            let (status, latency_ms, latency_str, success) = match res {
                Ok(resp) => {
                    let code = resp.status().as_u16();
                    if url.contains("cdn-cgi/trace") {
                        if let Ok(text) = resp.text().await {
                            trace_body = Some(text);
                        }
                    }

                    // 401/403/405 for API endpoints like OpenAI/Anthropic without key means connection succeeded!
                    let is_ok = code < 400 || code == 401 || code == 403 || code == 405;
                    let code_str = if is_ok {
                        format!("● OK ({})", code).green().bold().to_string()
                    } else {
                        format!("▲ HTTP {}", code).yellow().to_string()
                    };
                    let ms = elapsed.as_millis();
                    (code_str, Some(ms), format!("{:.0} ms", ms), is_ok)
                }
                Err(e) => {
                    let err_desc = if e.is_timeout() {
                        "Timeout (5s)".to_string()
                    } else if e.is_connect() {
                        "Connect Failed".to_string()
                    } else {
                        "Error".to_string()
                    };
                    (format!("✕ {}", err_desc).red().bold().to_string(), None, "-".dimmed().to_string(), false)
                }
            };

            (
                idx,
                RawTargetResult {
                    name: name.to_string(),
                    url: url.to_string(),
                    status,
                    latency_ms,
                    latency_str,
                    success,
                    trace_body,
                },
            )
        });
    }

    let mut ordered: Vec<Option<RawTargetResult>> = vec![None; TEST_TARGETS.len()];
    while let Some(res) = join_set.join_next().await {
        if let Ok((idx, raw)) = res {
            ordered[idx] = Some(raw);
        }
    }

    let mut results = Vec::new();
    let mut egress = None;
    for item in ordered.into_iter().flatten() {
        if let Some(ref text) = item.trace_body {
            egress = Some(parse_cf_trace(text));
        }
        results.push(item);
    }

    (results, egress)
}

pub fn compute_verdict(d_ok: bool, d_ms: Option<u128>, p_ok: bool, p_ms: Option<u128>) -> String {
    match (d_ok, p_ok) {
        (false, true) => "🚀 Proxy Unblocks!".green().bold().to_string(),
        (true, false) => "▲ Direct Only".yellow().bold().to_string(),
        (true, true) => {
            if let (Some(d), Some(p)) = (d_ms, p_ms) {
                if p < d {
                    let ratio = (d as f64) / (p.max(1) as f64);
                    format!("🚀 Proxy {:.1}x faster", ratio).green().to_string()
                } else if d < p {
                    let ratio = (p as f64) / (d.max(1) as f64);
                    format!("Direct {:.1}x faster", ratio).cyan().to_string()
                } else {
                    "Equal latency".dimmed().to_string()
                }
            } else {
                "● OK".green().to_string()
            }
        }
        (false, false) => "✕ Both Failed".red().to_string(),
    }
}

pub async fn run(custom_proxy: Option<String>, direct: bool, compare: bool, json_output: bool) -> Result<()> {
    if compare {
        return run_compare(custom_proxy, json_output).await;
    }

    let active_proxy_str = if direct {
        None
    } else {
        match &custom_proxy {
            Some(p) => Some(p.clone()),
            None => std::env::var("https_proxy")
                .or_else(|_| std::env::var("HTTPS_PROXY"))
                .ok(),
        }
    };

    let proxy_display = if direct {
        "Direct (Forced --direct, ignoring any proxy)".yellow().to_string()
    } else {
        match &active_proxy_str {
            Some(p) => format!("Proxy: {}", p.yellow()),
            None => "Direct (No proxy set)".dimmed().to_string(),
        }
    };

    if !json_output {
        println!("\n⚡ Testing Global Network & AI API Connectivity...");
        println!("   Route: {}\n", proxy_display);
    }

    let mut client_builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(false);

    if direct {
        client_builder = client_builder.no_proxy();
    } else if let Some(ref p) = active_proxy_str {
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

    let (raw_results, egress) = test_endpoints(&client).await;

    if let Some(bar) = pb {
        bar.finish_and_clear();
    }

    let rows: Vec<TargetResult> = raw_results
        .into_iter()
        .map(|r| TargetResult {
            name: r.name,
            url: r.url,
            status: r.status,
            latency: r.latency_str,
        })
        .collect();

    if json_output {
        let report = CheckerReport {
            proxy_used: active_proxy_str,
            egress,
            targets: rows,
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

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

async fn run_compare(custom_proxy: Option<String>, json_output: bool) -> Result<()> {
    let proxy_target = match custom_proxy {
        Some(p) => p,
        None => match std::env::var("https_proxy").or_else(|_| std::env::var("HTTPS_PROXY")) {
            Ok(val) => val,
            Err(_) => match scanner::find_first_active_proxy().await {
                Some(p) => format!("http://127.0.0.1:{}", p.port),
                None => "http://127.0.0.1:7890".to_string(),
            },
        },
    };

    if !json_output {
        println!("\n⚡ ================= BENCHMARK: DIRECT VS PROXY =============== ⚡");
        println!("   Comparing performance against proxy: {}\n", proxy_target.cyan().bold());
    }

    let direct_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(false)
        .no_proxy()
        .build()?;

    let mut proxy_client_builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(false);

    if let Ok(proxy) = reqwest::Proxy::all(&proxy_target) {
        proxy_client_builder = proxy_client_builder.proxy(proxy);
    }
    let proxy_client = proxy_client_builder.build()?;

    let pb = if !json_output {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        bar.set_message("Benchmarking Direct vs Proxy concurrently...");
        bar.enable_steady_tick(Duration::from_millis(80));
        Some(bar)
    } else {
        None
    };

    // Run both test sets concurrently
    let ((direct_results, direct_egress), (proxy_results, proxy_egress)) =
        tokio::join!(test_endpoints(&direct_client), test_endpoints(&proxy_client));

    if let Some(bar) = pb {
        bar.finish_and_clear();
    }

    let mut comparison_rows = Vec::new();
    for i in 0..direct_results.len().min(proxy_results.len()) {
        let d = &direct_results[i];
        let p = &proxy_results[i];
        let verdict = compute_verdict(d.success, d.latency_ms, p.success, p.latency_ms);

        comparison_rows.push(ComparisonRow {
            name: d.name.clone(),
            direct_status: d.status.clone(),
            direct_latency: d.latency_str.clone(),
            proxy_status: p.status.clone(),
            proxy_latency: p.latency_str.clone(),
            verdict,
        });
    }

    if json_output {
        let report = ComparisonReport {
            proxy_used: proxy_target,
            direct_egress,
            proxy_egress,
            comparison: comparison_rows,
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    // Show egress comparison
    println!("{}", "📍 Outbound Egress Comparison:".bold());
    if let Some(ref d_eg) = direct_egress {
        println!(
            "  Direct:  {} (Country: {}, Datacenter: {})",
            d_eg.ip.cyan().bold(),
            d_eg.location.yellow(),
            d_eg.colo
        );
    } else {
        println!("  Direct:  {}", "(Unable to detect direct trace)".dimmed());
    }

    if let Some(ref p_eg) = proxy_egress {
        let warp_badge = if p_eg.warp { " (WARP Enabled)".cyan() } else { "".clear() };
        println!(
            "  Proxy:   {} (Country: {}, Datacenter: {}){}",
            p_eg.ip.green().bold(),
            p_eg.location.yellow(),
            p_eg.colo,
            warp_badge
        );
    } else {
        println!("  Proxy:   {}", "(Unable to detect proxy trace)".dimmed());
    }
    println!();

    let table = Table::new(&comparison_rows).to_string();
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

    #[test]
    fn test_compute_verdict_proxy_unblocks() {
        let v = compute_verdict(false, None, true, Some(120));
        assert!(v.contains("Proxy Unblocks"));
    }

    #[test]
    fn test_compute_verdict_direct_only() {
        let v = compute_verdict(true, Some(80), false, None);
        assert!(v.contains("Direct Only"));
    }

    #[test]
    fn test_compute_verdict_speedup() {
        let v = compute_verdict(true, Some(800), true, Some(100));
        assert!(v.contains("Proxy 8.0x faster"));

        let v2 = compute_verdict(true, Some(50), true, Some(150));
        assert!(v2.contains("Direct 3.0x faster"));
    }
}
