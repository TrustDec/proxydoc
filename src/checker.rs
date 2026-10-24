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
    (
        "Google Gemini",
        "https://generativelanguage.googleapis.com/v1beta/models",
    ),
    ("DeepSeek API", "https://api.deepseek.com/models"),
    ("xAI (Grok) API", "https://api.x.ai/v1/models"),
    ("SiliconFlow API", "https://api.siliconflow.cn/v1/models"),
    ("Together AI API", "https://api.together.xyz/v1/models"),
    ("Moonshot (Kimi) API", "https://api.moonshot.cn/v1/models"),
    (
        "Aliyun DashScope (Qwen)",
        "https://dashscope.aliyuncs.com/compatible-mode/v1/models",
    ),
    (
        "DeepInfra API",
        "https://api.deepinfra.com/v1/openai/models",
    ),
    ("Cerebras API", "https://api.cerebras.ai/v1/models"),
    (
        "Fireworks AI API",
        "https://api.fireworks.ai/inference/v1/models",
    ),
    (
        "Zhipu AI (GLM) API",
        "https://open.bigmodel.cn/api/paas/v4/models",
    ),
    ("MiniMax API", "https://api.minimax.chat/v1/models"),
    ("HuggingFace", "https://huggingface.co/api/status"),
    ("crates.io", "https://crates.io/api/v1/summary"),
    ("npm Registry", "https://registry.npmjs.org"),
    ("Yarn Registry", "https://registry.yarnpkg.com"),
    ("PyPI Registry", "https://pypi.org/simple/"),
    ("Go Proxy", "https://proxy.golang.org"),
    ("Docker Hub", "https://registry-1.docker.io/v2/"),
    ("Ollama", "https://ollama.com"),
    ("OpenRouter API", "https://openrouter.ai/api/v1/models"),
    ("Groq API", "https://api.groq.com/openai/v1/models"),
    ("Mistral API", "https://api.mistral.ai/v1/models"),
    (
        "Perplexity API",
        "https://api.perplexity.ai/chat/completions",
    ),
    ("Cohere API", "https://api.cohere.com/v1/models"),
    ("GitHub Copilot API", "https://api.githubcopilot.com"),
    ("SambaNova API", "https://api.sambanova.ai/v1/models"),
    ("Replicate API", "https://api.replicate.com/v1/models"),
    (
        "Baidu Qianfan (ERNIE)",
        "https://qianfan.baidubce.com/v2/models",
    ),
    ("DeepL API", "https://api-free.deepl.com/v2/usage"),
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

pub fn build_target_list<'a>(custom_target: Option<&'a str>) -> Vec<(&'a str, &'a str)> {
    let mut targets: Vec<(&'a str, &'a str)> = TEST_TARGETS.to_vec();
    if let Some(t) = custom_target {
        targets.push(("Custom Target", t));
    }
    targets
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

pub async fn test_endpoints(
    client: &reqwest::Client,
    targets: &[(&str, &str)],
    timeout_secs: u64,
) -> (Vec<RawTargetResult>, Option<EgressInfo>) {
    let mut join_set = tokio::task::JoinSet::new();

    for (idx, &(name, url)) in targets.iter().enumerate() {
        let client_clone = client.clone();
        let name_owned = name.to_string();
        let url_owned = url.to_string();
        join_set.spawn(async move {
            let start = Instant::now();
            let res = client_clone.get(&url_owned).send().await;
            let elapsed = start.elapsed();

            let mut trace_body: Option<String> = None;
            let (status, latency_ms, latency_str, success) = match res {
                Ok(resp) => {
                    let code = resp.status().as_u16();
                    if url_owned.contains("cdn-cgi/trace") {
                        if let Ok(text) = resp.text().await {
                            trace_body = Some(text);
                        }
                    }

                    // 401/403/404/405 for API endpoints like OpenAI/Anthropic/DeepSeek without key means connection succeeded!
                    let is_ok =
                        code < 400 || code == 401 || code == 403 || code == 404 || code == 405;
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
                        format!("Timeout ({}s)", timeout_secs)
                    } else {
                        let s = e.to_string().to_lowercase();
                        if s.contains("cert")
                            || s.contains("certificate")
                            || s.contains("tls")
                            || s.contains("ssl")
                            || s.contains("handshake")
                            || s.contains("self-signed")
                        {
                            "TLS / Cert Untrusted".to_string()
                        } else if e.is_connect() {
                            "Connect Failed".to_string()
                        } else {
                            "Error".to_string()
                        }
                    };
                    (
                        format!("✕ {}", err_desc).red().bold().to_string(),
                        None,
                        "-".dimmed().to_string(),
                        false,
                    )
                }
            };

            (
                idx,
                RawTargetResult {
                    name: name_owned,
                    url: url_owned,
                    status,
                    latency_ms,
                    latency_str,
                    success,
                    trace_body,
                },
            )
        });
    }

    let mut ordered: Vec<Option<RawTargetResult>> = vec![None; targets.len()];
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

pub async fn run(
    custom_proxy: Option<String>,
    direct: bool,
    compare: bool,
    json_output: bool,
    custom_target: Option<String>,
    timeout_secs: u64,
) -> Result<()> {
    if compare {
        return run_compare(custom_proxy, json_output, custom_target, timeout_secs).await;
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
        "Direct (Forced --direct, ignoring any proxy)"
            .yellow()
            .to_string()
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
        .user_agent("proxydoc/0.5.0 (https://github.com/TrustDec/proxydoc)")
        .timeout(Duration::from_secs(timeout_secs))
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

    let targets = build_target_list(custom_target.as_deref());
    let (raw_results, egress) = test_endpoints(&client, &targets, timeout_secs).await;

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
        let warp_badge = if eg.warp {
            " (WARP Enabled)".cyan()
        } else {
            "".clear()
        };
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

async fn run_compare(
    custom_proxy: Option<String>,
    json_output: bool,
    custom_target: Option<String>,
    timeout_secs: u64,
) -> Result<()> {
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
        println!(
            "   Comparing performance against proxy: {}\n",
            proxy_target.cyan().bold()
        );
    }

    let direct_client = reqwest::Client::builder()
        .user_agent("proxydoc/0.5.0 (https://github.com/TrustDec/proxydoc)")
        .timeout(Duration::from_secs(timeout_secs))
        .danger_accept_invalid_certs(false)
        .no_proxy()
        .build()?;

    let mut proxy_client_builder = reqwest::Client::builder()
        .user_agent("proxydoc/0.5.0 (https://github.com/TrustDec/proxydoc)")
        .timeout(Duration::from_secs(timeout_secs))
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

    let targets = build_target_list(custom_target.as_deref());
    // Run both test sets concurrently
    let ((direct_results, direct_egress), (proxy_results, proxy_egress)) = tokio::join!(
        test_endpoints(&direct_client, &targets, timeout_secs),
        test_endpoints(&proxy_client, &targets, timeout_secs)
    );

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
        let warp_badge = if p_eg.warp {
            " (WARP Enabled)".cyan()
        } else {
            "".clear()
        };
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

    #[test]
    fn test_build_target_list() {
        let standard = build_target_list(None);
        assert!(standard.iter().any(|(name, _)| *name == "DeepSeek API"));
        assert!(standard.iter().any(|(name, _)| *name == "xAI (Grok) API"));
        assert!(standard.iter().any(|(name, _)| *name == "SiliconFlow API"));
        assert!(standard.iter().any(|(name, _)| *name == "Together AI API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Moonshot (Kimi) API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Aliyun DashScope (Qwen)"));
        assert!(standard.iter().any(|(name, _)| *name == "DeepInfra API"));
        assert!(standard.iter().any(|(name, _)| *name == "Cerebras API"));
        assert!(standard.iter().any(|(name, _)| *name == "Fireworks AI API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Zhipu AI (GLM) API"));
        assert!(standard.iter().any(|(name, _)| *name == "MiniMax API"));
        assert!(standard.iter().any(|(name, _)| *name == "OpenAI API"));
        assert!(standard.iter().any(|(name, _)| *name == "crates.io"));
        assert!(standard.iter().any(|(name, _)| *name == "npm Registry"));
        assert!(standard.iter().any(|(name, _)| *name == "Mistral API"));
        assert!(standard.iter().any(|(name, _)| *name == "Perplexity API"));
        assert!(standard.iter().any(|(name, _)| *name == "Cohere API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "GitHub Copilot API"));
        assert!(standard.iter().any(|(name, _)| *name == "SambaNova API"));
        assert!(standard.iter().any(|(name, _)| *name == "Replicate API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Baidu Qianfan (ERNIE)"));
        assert!(standard.iter().any(|(name, _)| *name == "DeepL API"));
        assert!(standard.iter().any(|(name, _)| *name == "Yarn Registry"));
        assert!(standard.iter().any(|(name, _)| *name == "PyPI Registry"));
        assert!(standard.iter().any(|(name, _)| *name == "Go Proxy"));
        assert!(standard.iter().any(|(name, _)| *name == "Docker Hub"));
        assert!(standard.iter().any(|(name, _)| *name == "Ollama"));
        assert!(standard.iter().any(|(name, _)| *name == "OpenRouter API"));
        assert!(standard.iter().any(|(name, _)| *name == "Groq API"));
        assert!(!standard.iter().any(|(name, _)| *name == "Custom Target"));

        let with_custom = build_target_list(Some("https://api.example.com"));
        assert!(with_custom
            .iter()
            .any(|(name, url)| *name == "Custom Target" && *url == "https://api.example.com"));
    }
}
