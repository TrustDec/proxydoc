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
    (10812, false, "WSTunnel Proxy Daemon (WebSocket/Tunnel)"),
    (10813, false, "Cloak Proxy Client Daemon (Obfuscation/Tunnel)"),
    (10814, false, "Wireproxy Proxy Daemon (WireGuard/Tunnel)"),
    (10815, false, "MTG Proxy Daemon (MTProto/Tunnel)"),
    (10816, false, "Kcptun Proxy Client Daemon (KCP/Tunnel)"),
    (10817, false, "Glider Proxy Daemon (Forward/Multi-protocol)"),
    (10818, false, "Redsocks Transparent Proxy Daemon (Redirector/SOCKS)"),
    (10819, false, "Dante SOCKS Proxy Daemon (SOCKS5/Server)"),
    (8123, false, "Polipo Caching Web Proxy (HTTP)"),
    (10820, false, "3proxy Multi-Protocol Proxy Daemon (HTTP/SOCKS)"),
    (10821, false, "Pproxy Python Proxy Daemon (HTTP/SOCKS)"),
    (10822, true, "MicroSOCKS SOCKS5 Proxy Daemon (SOCKS5)"),
    (53128, false, "Cntlm Authenticating Proxy Daemon (HTTP)"),
    (7777, false, "COW HTTP Proxy Daemon (HTTP)"),
    (8444, false, "Sniproxy SNI Forwarding Proxy Daemon (TLS/HTTP)"),
    (5353, false, "Dnsproxy AdGuard DNS Proxy Daemon (DNS/DoH)"),
    (5335, false, "Mosdns Pluggable DNS Forwarder (DNS/DoH)"),
    (6053, false, "SmartDNS Local DNS Proxy Daemon (DNS/TLS)"),
    (1053, false, "CoreDNS DNS Server & Forwarder (DNS/DoH)"),
    (3000, false, "AdGuard Home DNS & Web Proxy Daemon (HTTP/DNS)"),
    (5354, false, "DNSCrypt-proxy Encrypted DNS Proxy Daemon (DoH/DNSCrypt)"),
    (5336, false, "Unbound Validating Recursive DNS Proxy (DNS/DoT)"),
    (5355, false, "Dnsmasq Lightweight DNS & DHCP Proxy Daemon (DNS)"),
    (4711, false, "Pi-hole FTL DNS & Sinkhole Proxy Daemon (FTL/DNS)"),
    (4000, false, "Blocky Fast DNS & Ad-blocker Proxy Daemon (HTTP/DNS)"),
    (8053, false, "Stubby DNS Privacy DoT Stub Resolver Daemon (DoT/DNS)"),
    (5323, false, "Knot Resolver Validating DNS Resolver Daemon (DNS/DoT)"),
    (5300, false, "PowerDNS Recursor Resolving DNS Cache Proxy Daemon (DNS)"),
    (5199, false, "dnsdist DNS Load Balancer & DoH/DoT Proxy Daemon (Console/DNS)"),
    (953, false, "BIND 9 (named) Domain Name System & RNDC Control Daemon (RNDC/DNS)"),
    (5342, false, "NextDNS Encrypted DNS Client & Proxy Daemon (Control/DNS)"),
    (7053, false, "pdnsd Proxy DNS Daemon with Permanent Caching (DNS)"),
    (5350, false, "MaraDNS / Deadwood Recursive DNS Cache Daemon (DNS)"),
    (4444, false, "i2pd Anonymous Network HTTP Proxy Daemon (HTTP)"),
    (2023, false, "dae / daed eBPF Transparent Proxy & Routing Daemon (Dashboard/API)"),
    (9001, false, "Yggdrasil Encrypted Mesh Routing Daemon (Admin API)"),
    (11234, false, "cjdns Encrypted Mesh Routing Daemon (Admin API)"),
    (655, false, "tinc Mesh VPN Daemon (Control & Peering)"),
    (1194, false, "OpenVPN Secure Tunneling Daemon (Management/Tunnel)"),
    (5555, false, "SoftEther Multi-Protocol VPN Daemon (Control/Tunnel)"),
    (4443, false, "OpenConnect SSL VPN Client Daemon (Local Management / Tunnel)"),
    (3228, false, "strongSwan IPsec IKEv2 Daemon (VICI Management Socket / Daemon)"),
    (51820, false, "WireGuard Secure VPN Tunnel Daemon (Control/Tunnel)"),
    (9087, false, "Headscale Tailscale Coordination & Relay Daemon (Metrics/Control)"),
    (33073, false, "NetBird Zero Trust Mesh VPN Management Daemon (API/gRPC)"),
    (40000, true, "Cloudflare WARP Client Local Proxy Daemon (SOCKS5/HTTP)"),
    (8765, false, "Amnezia VPN Client & Service Daemon (API/IPC)"),
    (10080, false, "OpenZiti Zero Trust Edge Tunnel & Overlay Daemon (Control/IPC)"),
    (50123, false, "Firezone Zero Trust Mesh & WireGuard Daemon (Control/IPC)"),
    (50051, false, "Netmaker WireGuard Mesh Daemon (gRPC/Control)"),
    (9770, false, "Pritunl Enterprise VPN Client Daemon (API/IPC)"),
    (30334, false, "Twingate Zero Trust Connector & Client Daemon (Metrics/IPC)"),
    (11010, false, "EasyTier Decentralized Mesh VPN Daemon (RPC/CLI)"),
    (3080, false, "Teleport Access & Zero Trust Proxy Daemon (Web/Auth)"),
    (5443, false, "Pomerium Identity-Aware & Zero Trust Access Proxy Daemon (HTTPS/Auth)"),
    (42230, false, "Innernet Mesh VPN & WireGuard Overlay Daemon (Control/Tunnel)"),
    (50055, false, "Defguard Enterprise WireGuard Gateway & 2FA Daemon (gRPC/Web)"),
    (8090, false, "Inlets Cloud-Native Tunnel & Reverse Proxy Daemon (Control/Tunnel)"),
    (18080, false, "zrok Peer-to-Peer Sharing & Zero Trust Tunnel Daemon (HTTP/TCP)"),
    (5644, false, "n2n Peer-to-Peer Mesh VPN & Supernode Daemon (UDP/Tunnel)"),
    (9999, false, "Hyprspace Peer-to-Peer Mesh VPN Daemon (libp2p/RPC)"),
    (10000, false, "Fastd Secure Tunneling & Mesh VPN Daemon (Status/Tunnel)"),
    (9000, false, "EdgeVPN Decentralized Immutable P2P Mesh VPN Daemon (libp2p/API)"),
    (9443, false, "boringproxy Reverse Proxy & Automated Tunnel Daemon (HTTPS/Tunnel)"),
    (8443, false, "Ghostunnel Mutual TLS Proxy Daemon (HTTPS/mTLS)"),
    (12000, false, "FreeLAN Peer-to-Peer Mesh VPN Daemon (UDP/Tunnel)"),
    (28080, false, "OmniEdge Decentralized Mesh VPN Daemon (API/Control)"),
    (8787, false, "Lantern Censorship Circumvention Proxy Daemon (HTTP)"),
    (51821, false, "TunSafe WireGuard VPN Client Daemon (Control/Tunnel)"),
    (12300, false, "Sshuttle Transparent SSH Proxy Daemon (Forward/Control)"),
    (8081, false, "Psiphon Censorship Circumvention Daemon (HTTP/Tunnel)"),
    (3999, false, "Localtunnel Localhost Reverse Tunnel Daemon (Client/Control)"),
    (8088, false, "Pagekite Fast Localhost Tunnel & Reverse Proxy Daemon (HTTP/Tunnel)"),
    (8474, false, "Toxiproxy Resiliency & Fault Injection Proxy Daemon (HTTP/Admin)"),
    (9080, false, "Telepresence Kubernetes Local Proxy & Intercept Daemon (Control/Intercept)"),
    (4040, false, "LocalXpose Localhost Reverse Proxy & Tunnel Daemon (HTTP/Admin)"),
    (4005, false, "Packetriot Localhost Reverse Tunnel & Proxy Daemon (Control/Tunnel)"),
    (3333, false, "sish HTTP/TCP Localhost Tunnel & Reverse Proxy Daemon (Tunnel/SSH)"),
    (4300, false, "Pinggy HTTP/TCP Localhost Tunnel & Reverse Proxy Daemon (Control/Tunnel)"),
    (8085, false, "Loophole E2EE Localhost Reverse Proxy & Tunnel Daemon (Control/Tunnel)"),
    (8001, false, "Tunnelmole Localhost Reverse Proxy & Tunnel Daemon (Control/Tunnel)"),
    (10200, false, "Expose Localhost Reverse Proxy & Tunnel Daemon (Control/Tunnel)"),
    (55556, false, "Playit Localhost Reverse Proxy & Tunnel Daemon (Control/Tunnel)"),
    (3320, false, "pgrok Localhost Reverse Proxy & Tunnel Daemon (Control/Tunnel)"),
    (9200, false, "cpolar Localhost Reverse Tunnel & Web UI Daemon (HTTP/Web)"),
    (2222, false, "tmate Terminal Sharing & SSH Reverse Tunnel Daemon (SSH/Tunnel)"),
    (2323, false, "upterm Terminal Sharing & SSH Reverse Tunnel Daemon (Control/Tunnel)"),
    (3993, false, "NATAPP Localhost Reverse Tunnel & Web UI Daemon (HTTP/Web)"),
    (4900, false, "lanproxy Intranet Penetration & Reverse Tunnel Client (Control/Tunnel)"),
    (3022, false, "teleconsole Terminal Sharing & SSH Reverse Tunnel Daemon (SSH/Tunnel)"),
    (8054, false, "tunnelto Localhost Reverse Proxy & Public Tunnel Client (Control/Tunnel)"),
    (8040, false, "Webhook Relay Localhost Tunnel & Webhook Forwarding Daemon (HTTP/Control)"),
    (8299, false, "jprq Localhost Reverse Tunnel & Port Forwarding Daemon (HTTP/TCP)"),
    (8022, false, "mysocketctl Localhost Reverse Tunnel & Border0 Daemon (SSH/Tunnel)"),
    (27183, false, "openp2p Decentralized P2P Reverse Tunnel & Port Forwarding Daemon (HTTP/RPC)"),
    (9990, false, "rport Remote Management & Reverse Tunnel Client Daemon (HTTP/Tunnel)"),
    (3030, false, "mirrord Kubernetes Dev Traffic Mirror & Reverse Proxy Daemon (HTTP/Admin)"),
    (2223, false, "kt-connect Kubernetes Dev Tunnel & SOCKS5 Proxy Daemon (SSH/Tunnel)"),
    (8090, false, "DevSpace Cloud-Native Kubernetes Dev Proxy & UI Daemon (HTTP/Web)"),
    (9909, false, "Geph Censorship-Resistant Proxy & Reverse Tunnel Client Daemon (HTTP/SOCKS5)"),
    (22100, false, "Okteto Cloud-Native Kubernetes Dev Reverse Tunnel & SSH Daemon (SSH/Tunnel)"),
    (50052, false, "Skaffold Cloud-Native Kubernetes Dev Port-Forward & Control Daemon (gRPC/HTTP)"),
    (10350, false, "Tilt Cloud-Native Kubernetes Dev Engine & Web Proxy Daemon (HTTP/Web)"),
    (9777, false, "Garden Cloud-Native Kubernetes Dev Environment & Console Daemon (HTTP/Web)"),
    (3986, false, "Daytona Open-Source Dev Environment Manager & Workspace Tunnel Daemon (HTTP/API)"),
    (8443, false, "vcluster Virtual Kubernetes Cluster Control Plane & Syncer Proxy Daemon (HTTPS/API)"),
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
