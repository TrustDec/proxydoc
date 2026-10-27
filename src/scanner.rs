use colored::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use tabled::{Table, Tabled};
use tokio::net::TcpStream;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct PortInfo {
    #[tabled(rename = "Port")]
    pub port: u16,
    #[tabled(rename = "Protocol")]
    pub protocol: &'static str,
    #[tabled(rename = "Common Clients")]
    pub clients: &'static str,
    #[tabled(rename = "Status")]
    pub status: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedProxy {
    pub port: u16,
    pub is_socks: bool,
    pub client_hint: &'static str,
}

pub const COMMON_PORTS: &[(u16, bool, &'static str)] = &[
    (7890, false, "Clash / Clash Verge / Mihomo (Mixed/HTTP)"),
    (7897, false, "Clash Verge Rev default (Mixed/HTTP)"),
    (1080, true, "Shadowsocks / V2Ray / Sing-box / Trojan (SOCKS5)"),
    (1082, true, "Shadowrocket Mac / Sing-box (SOCKS5)"),
    (10808, true, "V2RayN / Xray (SOCKS5)"),
    (10809, false, "V2RayN / Xray (HTTP)"),
    (1086, true, "ShadowsocksX-NG (SOCKS5)"),
    (1087, false, "ShadowsocksX-NG (HTTP)"),
    (8317, false, "CLI-Proxy-API / CPA Gateway (HTTP)"),
    (9090, false, "Clash RESTful Controller"),
    (2080, false, "Surge Mac (HTTP)"),
    (6152, true, "Surge Mac (SOCKS5)"),
    (6153, false, "Surge Mac (HTTP)"),
    (8080, false, "Mitmproxy / Burp Suite (HTTP)"),
    (9050, true, "Tor SOCKS daemon (SOCKS5)"),
    (8118, false, "Privoxy forwarding daemon (HTTP)"),
    (8888, false, "Tinyproxy / Fiddler (HTTP)"),
    (3128, false, "Squid caching proxy (HTTP)"),
    (4040, false, "Ngrok Web Inspection (HTTP)"),
    (2024, false, "Cloudflare Tunnel Metrics (HTTP)"),
    (7400, false, "FRP Web Admin / Dashboard (HTTP)"),
    (1055, true, "Tailscale SOCKS5 Proxy (SOCKS5)"),
    (9993, false, "ZeroTier One Service (UDP/HTTP API)"),
    (10800, false, "GO Simple Tunnel / Gost (HTTP)"),
    (2333, false, "Rathole NAT Traversal (Default)"),
    (4959, false, "Chisel HTTP Tunnel Service (HTTP)"),
    (4242, false, "Nebula Mesh Overlay Network (Control/UDP)"),
    (7835, false, "Bore Modern Tunnel Service (TCP)"),
    (1081, false, "Brook Proxy Daemon (HTTP)"),
    (8024, false, "NPS / NPC Intranet Penetration Client (Control)"),
    (8388, false, "Shadowsocks Service / Daemon (TCP/UDP)"),
    (10085, false, "V2Ray / Xray gRPC API Daemon (gRPC)"),
    (6450, false, "Sing-box Universal Proxy Daemon (Control/Mixed)"),
    (5678, false, "Hysteria Proxy Client Daemon (HTTP/SOCKS5)"),
    (10801, false, "Trojan-Go Local Forwarder (HTTP)"),
    (8443, false, "NaïveProxy / Trojan HTTPS Ingress (HTTPS/Mixed)"),
    (10802, false, "Tuic Proxy Daemon (HTTP/Mixed)"),
    (10803, false, "Juicity Proxy Daemon (HTTP/Mixed)"),
    (10804, false, "Snell Proxy Daemon (HTTP/Mixed)"),
    (10805, false, "Leaf Proxy Daemon (HTTP/SOCKS5)"),
    (10806, false, "Mihomo Proxy Daemon (Mixed/HTTP)"),
    (10807, false, "Stunnel Proxy Daemon (TLS/Tunnel)"),
    (10810, false, "Shadow-TLS Proxy Daemon (TLS/Tunnel)"),
    (10811, false, "Tun2socks Proxy Daemon (TUN/SOCKS5)"),
];

pub async fn check_host_port(host: &str, port: u16) -> bool {
    let target = match host {
        "localhost" => format!("127.0.0.1:{}", port),
        _ => format!("{}:{}", host, port),
    };

    if let Ok(addr) = target.parse::<SocketAddr>() {
        return match timeout(Duration::from_millis(200), TcpStream::connect(&addr)).await {
            Ok(Ok(_)) => true,
            _ => false,
        };
    }

    if let Ok(mut addrs) = tokio::net::lookup_host(&target).await {
        if let Some(addr) = addrs.next() {
            return match timeout(Duration::from_millis(200), TcpStream::connect(&addr)).await {
                Ok(Ok(_)) => true,
                _ => false,
            };
        }
    }

    false
}

pub async fn check_port(port: u16) -> bool {
    check_host_port("127.0.0.1", port).await
}

pub async fn find_first_active_proxy() -> Option<DetectedProxy> {
    for &(port, is_socks, hint) in COMMON_PORTS {
        if check_port(port).await {
            // ignore 9090 as it is usually controller, not proxy
            if port != 9090 {
                return Some(DetectedProxy {
                    port,
                    is_socks,
                    client_hint: hint,
                });
            }
        }
    }
    None
}

pub async fn run(json_output: bool) -> anyhow::Result<()> {
    if !json_output {
        println!(
            "{}",
            "🔍 Scanning common local proxy ports (127.0.0.1)..."
                .cyan()
                .bold()
        );
    }

    let mut rows = Vec::new();
    let mut found_count = 0;

    for &(port, is_socks, clients) in COMMON_PORTS {
        let active = check_port(port).await;
        if active {
            found_count += 1;
        }

        let status = if active {
            "● OPEN (Active)".green().bold().to_string()
        } else {
            "○ closed".dimmed().to_string()
        };

        rows.push(PortInfo {
            port,
            protocol: if is_socks { "SOCKS5" } else { "HTTP/Mixed" },
            clients,
            status,
            active,
        });
    }

    if json_output {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }

    let table = Table::new(&rows).to_string();
    println!("\n{}\n", table);

    if found_count > 0 {
        println!(
            "{}",
            format!(
                "✅ Found {} active local proxy port(s) ready to use.",
                found_count
            )
            .green()
            .bold()
        );
    } else {
        println!(
            "{}",
            "⚠️ No common proxy ports detected. Ensure your proxy client (Clash/Shadowrocket/Sing-box) is running."
                .yellow()
                .bold()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_host_port_active_and_closed() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        assert!(check_host_port("127.0.0.1", port).await);
        assert!(check_host_port("localhost", port).await);

        drop(listener);
        // Closed port check should return false
        assert!(!check_host_port("127.0.0.1", 0).await);
    }
}
