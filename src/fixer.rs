use anyhow::Result;
use colored::*;
use std::process::Command;

use crate::doctor::{
    clean_helm_repositories, clean_jsonc, clean_swiftpm_mirrors, clean_terraform_network_mirrors,
    clean_vcpkg_configuration_registries, get_bazel_proxy, get_bun_proxy, get_cabal_proxy,
    get_cargo_proxy, get_composer_proxy, get_conan_proxy, get_conda_proxy, get_cpan_proxy,
    get_curlrc_proxy, get_docker_proxy, get_gem_proxy, get_gemrc_path, get_gh_proxy,
    get_glab_proxy, get_goproxy, get_gradle_proxy, get_helm_repositories_paths, get_hex_proxy,
    get_hg_proxy, get_hgrc_path, get_ide_proxies, get_jetbrains_proxies, get_julia_startup_proxy,
    get_kubeconfig_path, get_lein_proxy, get_luarocks_proxy, get_maven_proxy, get_nimble_proxy,
    get_nix_proxy, get_npm_config, get_npm_registries, get_nuget_proxy, get_opam_proxy,
    get_pip_proxy, get_pnpm_proxy, get_rebar3_proxy, get_renviron_proxy, get_sbt_proxy,
    get_scoop_proxy, get_ssh_config_proxies, get_stack_proxy, get_svn_proxy,
    get_svn_servers_path, get_swiftpm_mirrors_path, get_system_proxy, get_terraform_config_paths,
    get_uv_config_path, get_uv_proxy, get_vcpkg_configuration_paths, get_wgetrc_proxy,
    get_yarn_proxy, parse_helm_repositories, parse_kubeconfig_proxies, parse_swiftpm_mirrors,
    parse_terraform_network_mirrors, parse_vcpkg_configuration_registries, verify_proxy_url,
    get_bundler_proxy, get_gcloud_proxy, get_aws_proxy, get_azure_proxy, get_podman_proxy,
    get_vagrant_proxy, get_choco_proxy, get_colima_proxy, get_minikube_proxy, get_spack_proxy,
    get_macports_proxy, get_apt_proxy, get_dnf_proxy, get_zypper_proxy,
    get_pacman_proxy, get_pkg_proxy, get_portage_proxy, get_docker_daemon_proxy,
    get_flatpak_proxy, get_snap_proxy, get_ollama_proxy, get_containerd_proxy, get_buildkit_proxy, get_k3s_proxy, get_microk8s_proxy, get_k0s_proxy, get_rke2_proxy, get_nomad_proxy, get_kubelet_proxy, get_gitlab_runner_proxy, get_actions_runner_proxy, get_jenkins_proxy, get_buildkite_agent_proxy, get_woodpecker_agent_proxy, get_drone_runner_proxy, get_act_runner_proxy, get_circleci_runner_proxy, get_azure_pipelines_agent_proxy, get_teamcity_agent_proxy, get_bamboo_agent_proxy, get_tfc_agent_proxy, get_concourse_worker_proxy, get_tentacle_agent_proxy, get_puppet_agent_proxy, get_chef_client_proxy, get_salt_minion_proxy, get_datadog_agent_proxy, get_newrelic_infra_proxy, get_elastic_agent_proxy, get_grafana_alloy_proxy, get_otelcol_proxy, get_vector_proxy, get_fluent_bit_proxy, get_fluentd_proxy, get_telegraf_proxy, get_logstash_proxy, get_prometheus_proxy, get_alertmanager_proxy, get_promtail_proxy, get_vmagent_proxy, get_filebeat_proxy, get_metricbeat_proxy, get_heartbeat_proxy, get_auditbeat_proxy, get_packetbeat_proxy, get_winlogbeat_proxy, get_consul_proxy, get_vault_proxy, get_traefik_proxy, get_caddy_proxy, get_envoy_proxy, get_haproxy_proxy, get_nginx_proxy, get_kong_proxy, get_apisix_proxy, get_squid_proxy, get_privoxy_proxy, get_tinyproxy_proxy, get_mitmproxy_proxy, get_tor_proxy, get_cloudflared_proxy, get_ngrok_proxy, get_frpc_proxy, get_tailscale_proxy, get_zerotier_proxy, get_gost_proxy, get_rathole_proxy, get_chisel_proxy, get_nebula_proxy, get_bore_proxy, get_brook_proxy, get_npc_proxy, get_shadowsocks_proxy, get_v2ray_proxy, get_sing_box_proxy, get_hysteria_proxy, get_trojan_proxy, get_naive_proxy, get_tuic_proxy, get_juicity_proxy, get_snell_proxy, get_leaf_proxy, get_mihomo_proxy, get_stunnel_proxy, get_shadow_tls_proxy, get_tun2socks_proxy, get_wstunnel_proxy, get_cloak_proxy, get_wireproxy_proxy, get_mtg_proxy, get_kcptun_proxy, get_glider_proxy, get_redsocks_proxy, get_dante_proxy, get_threeproxy_proxy, get_polipo_proxy, get_pproxy_proxy, get_microsocks_proxy, get_cntlm_proxy, get_cow_proxy, get_sniproxy_proxy, get_dnsproxy_proxy, get_apk_repositories_paths,
    parse_apk_repositories, clean_apk_repositories,
    get_mosdns_proxy, get_smartdns_proxy,
};
use crate::manager::{
    deactivate_jetbrains_proxy_xml, remove_vscode_proxy_json, unset_bazel_proxy, unset_bun_proxy,
    unset_cabal_proxy, unset_composer_proxy, unset_conan_proxy, unset_conda_proxy,
    unset_cpan_proxy, unset_docker_proxy, unset_gem_proxy, unset_gh_proxy, unset_glab_proxy,
    unset_go_proxy, unset_gradle_proxy, unset_hex_proxy, unset_hg_proxy, unset_julia_proxy,
    unset_kube_proxy, unset_lein_proxy, unset_luarocks_proxy, unset_maven_proxy,
    unset_nimble_proxy, unset_nix_proxy, unset_nuget_proxy, unset_opam_proxy, unset_pnpm_proxy,
    unset_rebar3_proxy, unset_renviron_proxy, unset_sbt_proxy, unset_scoop_proxy,
    unset_stack_proxy, unset_svn_proxy, unset_uv_proxy, unset_yarn_proxy, unset_bundler_proxy,
    unset_gcloud_proxy, unset_aws_proxy, unset_azure_proxy, unset_mamba_proxy, unset_podman_proxy,
    unset_vagrant_proxy, unset_choco_proxy, unset_colima_proxy, unset_minikube_proxy,
    unset_spack_proxy, unset_macports_proxy, unset_apt_proxy, unset_dnf_proxy, unset_zypper_proxy,
    unset_pacman_proxy, unset_pkg_proxy, unset_portage_proxy, unset_docker_daemon_proxy,
    unset_flatpak_proxy, unset_snap_proxy, unset_ollama_proxy, unset_containerd_proxy,
    unset_buildkit_proxy, unset_k3s_proxy, unset_microk8s_proxy, unset_k0s_proxy,
    unset_rke2_proxy, unset_nomad_proxy, unset_kubelet_proxy, unset_gitlab_runner_proxy,
    unset_actions_runner_proxy, unset_jenkins_proxy, unset_buildkite_agent_proxy, unset_woodpecker_agent_proxy, unset_drone_runner_proxy, unset_act_runner_proxy, unset_circleci_runner_proxy, unset_azure_pipelines_agent_proxy, unset_teamcity_agent_proxy, unset_bamboo_agent_proxy, unset_tfc_agent_proxy, unset_concourse_worker_proxy, unset_tentacle_agent_proxy, unset_puppet_agent_proxy, unset_chef_client_proxy, unset_salt_minion_proxy, unset_datadog_agent_proxy, unset_newrelic_infra_proxy, unset_elastic_agent_proxy, unset_grafana_alloy_proxy, unset_otelcol_proxy, unset_vector_proxy, unset_fluent_bit_proxy, unset_fluentd_proxy, unset_telegraf_proxy, unset_logstash_proxy, unset_prometheus_proxy, unset_alertmanager_proxy, unset_promtail_proxy, unset_vmagent_proxy, unset_filebeat_proxy, unset_metricbeat_proxy, unset_heartbeat_proxy, unset_auditbeat_proxy, unset_packetbeat_proxy, unset_winlogbeat_proxy, unset_consul_proxy, unset_vault_proxy, unset_traefik_proxy, unset_caddy_proxy, unset_envoy_proxy, unset_haproxy_proxy, unset_nginx_proxy, unset_kong_proxy, unset_apisix_proxy, unset_squid_proxy, unset_privoxy_proxy, unset_tinyproxy_proxy, unset_mitmproxy_proxy, unset_tor_proxy, unset_cloudflared_proxy, unset_ngrok_proxy, unset_frpc_proxy, unset_tailscale_proxy, unset_zerotier_proxy, unset_gost_proxy, unset_rathole_proxy, unset_chisel_proxy, unset_nebula_proxy, unset_bore_proxy, unset_brook_proxy, unset_npc_proxy, unset_shadowsocks_proxy, unset_v2ray_proxy, unset_sing_box_proxy, unset_hysteria_proxy, unset_trojan_proxy, unset_naive_proxy, unset_tuic_proxy, unset_juicity_proxy, unset_snell_proxy, unset_leaf_proxy, unset_mihomo_proxy, unset_stunnel_proxy, unset_shadow_tls_proxy, unset_tun2socks_proxy, unset_wstunnel_proxy, unset_cloak_proxy, unset_wireproxy_proxy, unset_mtg_proxy, unset_kcptun_proxy, unset_glider_proxy, unset_redsocks_proxy, unset_dante_proxy, unset_threeproxy_proxy, unset_polipo_proxy, unset_pproxy_proxy, unset_microsocks_proxy, unset_cntlm_proxy, unset_cow_proxy, unset_sniproxy_proxy, unset_dnsproxy_proxy,
    unset_mosdns_proxy, unset_smartdns_proxy,
};
use crate::scanner;

pub fn comment_out_dead_lines(
    content: &str,
    is_dead_proxy_line: impl Fn(&str) -> bool,
) -> (String, bool) {
    let mut modified = false;
    let mut out_lines = Vec::new();

    for line in content.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') && is_dead_proxy_line(trim) {
            out_lines.push(format!("# [proxydoc disabled dead proxy]: {}", line));
            modified = true;
        } else {
            out_lines.push(line.to_string());
        }
    }

    (out_lines.join("\n") + "\n", modified)
}

pub async fn run() -> Result<()> {
    println!(
        "\n{}",
        "🩹 ==================== PROXY AUTO-REPAIR ==================== 🩹"
            .bold()
            .cyan()
    );
    println!(
        "{}",
        "   Detecting anomalies and automatically applying fixes...\n".dimmed()
    );

    let mut fixed_count = 0;

    // 1. Clean dead Git proxy
    let git_http = get_git_config("http.proxy");
    let git_https = get_git_config("https.proxy");
    let mut git_cleaned = false;

    if let Some(val) = git_http {
        if !verify_proxy_url(&val).await {
            println!(
                "  [1/125] Found dead Git http.proxy (`{}`), cleaning...",
                val.yellow()
            );
            let _ = Command::new("git")
                .args(["config", "--global", "--unset", "http.proxy"])
                .status();
            git_cleaned = true;
            fixed_count += 1;
        }
    }
    if let Some(val) = git_https {
        if !verify_proxy_url(&val).await {
            println!(
                "  [1/125] Found dead Git https.proxy (`{}`), cleaning...",
                val.yellow()
            );
            let _ = Command::new("git")
                .args(["config", "--global", "--unset", "https.proxy"])
                .status();
            git_cleaned = true;
            fixed_count += 1;
        }
    }

    if let Some(val) = get_git_config("http.sslCAInfo") {
        if !std::path::Path::new(&val).exists() {
            println!("  [1/125] Found broken Git http.sslCAInfo pointing to missing file (`{}`), cleaning...", val.yellow());
            let _ = Command::new("git")
                .args(["config", "--global", "--unset", "http.sslCAInfo"])
                .status();
            git_cleaned = true;
            fixed_count += 1;
        }
    }

    if git_cleaned {
        println!("  {} Cleared broken Git global proxy.", "✔".green());
    } else {
        println!("  [1/125] Git global proxy: {}", "Clean or healthy".dimmed());
    }

    // 2. Clean dead NPM proxy
    let npm_http = get_npm_config("proxy");
    let npm_https = get_npm_config("https-proxy");
    let mut npm_cleaned = false;

    if let Some(val) = npm_http {
        if !verify_proxy_url(&val).await {
            println!(
                "  [2/125] Found dead NPM proxy (`{}`), cleaning...",
                val.yellow()
            );
            let _ = Command::new("npm")
                .args(["config", "delete", "proxy"])
                .status();
            npm_cleaned = true;
            fixed_count += 1;
        }
    }
    if let Some(val) = npm_https {
        if !verify_proxy_url(&val).await {
            println!(
                "  [2/125] Found dead NPM https-proxy (`{}`), cleaning...",
                val.yellow()
            );
            let _ = Command::new("npm")
                .args(["config", "delete", "https-proxy"])
                .status();
            npm_cleaned = true;
            fixed_count += 1;
        }
    }

    if npm_cleaned {
        println!("  {} Cleared broken NPM proxy.", "✔".green());
    } else {
        println!("  [2/125] NPM proxy: {}", "Clean or healthy".dimmed());
    }

    // 3. Clean dead Yarn proxy (~/.yarnrc / ~/.yarnrc.yml)
    if let Some(cfg) = get_yarn_proxy() {
        if !verify_proxy_url(&cfg.proxy).await {
            println!(
                "  [3/125] Found dead Yarn proxy (`{}` in {}), cleaning...",
                cfg.proxy.yellow(),
                cfg.source
            );
            if let Ok(true) = unset_yarn_proxy() {
                println!("  {} Cleared broken Yarn global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [3/125] Yarn proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [3/125] Yarn proxy: {}", "Clean or not set".dimmed());
    }

    // 4. Clean dead pnpm proxy (~/.pnpmrc)
    if let Some(val) = get_pnpm_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [4/125] Found dead pnpm proxy (`{}` in ~/.pnpmrc), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_pnpm_proxy() {
                println!("  {} Cleared broken pnpm proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [4/125] pnpm proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [4/125] pnpm proxy: {}", "Clean or not set".dimmed());
    }

    // 5. Clean dead Bun proxy (~/.bunfig.toml)
    if let Some(val) = get_bun_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [5/125] Found dead Bun proxy (`{}` in ~/.bunfig.toml), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_bun_proxy() {
                println!("  {} Cleared broken Bun proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [5/125] Bun proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [5/125] Bun proxy: {}", "Clean or not set".dimmed());
    }

    // 6. Clean dead Pip proxy
    if let Some(val) = get_pip_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [6/125] Found dead Pip global.proxy (`{}`), cleaning...",
                val.yellow()
            );
            let _ = Command::new("pip")
                .args(["config", "unset", "global.proxy"])
                .status();
            println!("  {} Cleared broken Pip global proxy.", "✔".green());
            fixed_count += 1;
        } else {
            println!("  [6/125] Pip global proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [6/125] Pip global proxy: {}", "Clean or not set".dimmed());
    }

    // 7. Clean dead Conda proxy (~/.condarc)
    if let Some(cfg) = get_conda_proxy() {
        let mut dead = false;
        if let Some(ref http) = cfg.http {
            if !verify_proxy_url(http).await {
                dead = true;
            }
        }
        if let Some(ref https) = cfg.https {
            if !verify_proxy_url(https).await {
                dead = true;
            }
        }

        if dead {
            println!("  [7/125] Found dead Conda proxy in ~/.condarc, cleaning...");
            if let Ok(true) = unset_conda_proxy() {
                println!(
                    "  {} Cleared broken Conda proxy in ~/.condarc.",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [7/125] Conda proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [7/125] Conda proxy: {}", "Clean or not set".dimmed());
    }

    // 8. Clean dead Docker client proxy (~/.docker/config.json)
    if let Some(val) = get_docker_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [8/125] Found dead Docker proxy (`{}`), cleaning ~/.docker/config.json...",
                val.yellow()
            );
            if let Ok(true) = unset_docker_proxy() {
                println!("  {} Cleared broken Docker client proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [8/125] Docker client proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [8/125] Docker client proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 9. Clean dead Gradle proxy (~/.gradle/gradle.properties)
    if let Some(cfg) = get_gradle_proxy() {
        if !scanner::check_host_port(&cfg.host, cfg.port).await {
            println!(
                "  [9/125] Found dead Gradle proxy (`{}:{}`), removing from ~/.gradle/gradle.properties...",
                cfg.host.yellow(),
                cfg.port
            );
            if let Ok(true) = unset_gradle_proxy() {
                println!("  {} Cleared broken Gradle global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [9/125] Gradle global proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [9/125] Gradle global proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 10. Clean dead Maven proxy (~/.m2/settings.xml)
    if let Some(cfg) = get_maven_proxy() {
        if !scanner::check_host_port(&cfg.host, cfg.port).await {
            println!(
                "  [10/125] Found dead Maven proxy (`{}:{}`), deactivating in ~/.m2/settings.xml...",
                cfg.host.yellow(),
                cfg.port
            );
            if let Ok(true) = unset_maven_proxy() {
                println!("  {} Deactivated broken Maven global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [10/125] Maven global proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [10/125] Maven global proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 11. Clean dead NuGet (.NET) proxy
    if let Some((val, path)) = get_nuget_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [11/125] Found dead NuGet proxy (`{}` in {}), cleaning...",
                val.yellow(),
                path.display()
            );
            if let Ok(true) = unset_nuget_proxy() {
                println!("  {} Cleared broken NuGet global proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!(
                "  [11/125] NuGet (.NET) global proxy: {}",
                "Reachable".green()
            );
        }
    } else {
        println!(
            "  [11/125] NuGet (.NET) global proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 12. Clean dead VS Code / Cursor IDE proxies
    let ide_proxies = get_ide_proxies();
    let mut ide_cleaned = false;
    for info in ide_proxies {
        if let Some(ref val) = info.proxy {
            if !verify_proxy_url(val).await {
                println!(
                    "  [12/125] Found dead {} proxy (`{}`), cleaning {}...",
                    info.ide_name,
                    val.yellow(),
                    info.path.display()
                );
                let bak = info.path.with_extension("json.bak");
                let _ = std::fs::copy(&info.path, bak);

                if let Ok(content) = std::fs::read_to_string(&info.path) {
                    let cleaned = clean_jsonc(&content);
                    if let Ok(mut root) = serde_json::from_str::<serde_json::Value>(&cleaned) {
                        if remove_vscode_proxy_json(&mut root) {
                            if let Ok(out) = serde_json::to_string_pretty(&root) {
                                let _ = std::fs::write(&info.path, out);
                                ide_cleaned = true;
                                fixed_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    if ide_cleaned {
        println!("  {} Cleared broken IDE proxy settings.", "✔".green());
    } else {
        println!(
            "  [12/125] VS Code & Cursor IDE settings: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 12. Clean dead JetBrains / Android Studio IDE proxies
    let jb_proxies = get_jetbrains_proxies();
    let mut jb_cleaned = false;
    for info in jb_proxies {
        let is_dead = if info.use_proxy {
            if let (Some(ref host), Some(port)) = (&info.host, info.port) {
                !scanner::check_host_port(host, port).await
            } else {
                false
            }
        } else if info.use_pac {
            if let Some(ref pac) = info.pac_url {
                !verify_proxy_url(pac).await
            } else {
                false
            }
        } else {
            false
        };

        if is_dead {
            println!(
                "  [13/125] Found dead {} proxy in {}, deactivating...",
                info.ide_name,
                info.path.display()
            );
            let bak = info.path.with_extension("xml.bak");
            let _ = std::fs::copy(&info.path, bak);

            if let Ok(content) = std::fs::read_to_string(&info.path) {
                let (new_c, changed) = deactivate_jetbrains_proxy_xml(&content);
                if changed {
                    let _ = std::fs::write(&info.path, new_c);
                    jb_cleaned = true;
                    fixed_count += 1;
                }
            }
        }
    }

    if jb_cleaned {
        println!(
            "  {} Deactivated broken JetBrains / Android Studio proxy settings.",
            "✔".green()
        );
    } else {
        println!(
            "  [13/125] JetBrains & Android Studio settings: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 13. Clean dead GitHub CLI (gh) proxy
    if let Some(val) = get_gh_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [14/125] Found dead GitHub CLI (gh) proxy (`{}`), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_gh_proxy() {
                println!("  {} Cleared broken GitHub CLI (gh) proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [14/125] GitHub CLI (gh) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [14/125] GitHub CLI (gh) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 14. Clean dead cURL (~/.curlrc), Wget (~/.wgetrc) & Cargo proxy
    let mut static_fixed = false;
    if let Some(home) = dirs::home_dir() {
        let curlrc_path = home.join(".curlrc");
        if let Some(val) = get_curlrc_proxy() {
            if !verify_proxy_url(&val).await {
                if let Ok(content) = std::fs::read_to_string(&curlrc_path) {
                    println!("  [15/125] Found dead proxy `{}` in ~/.curlrc, commenting out with backup...", val.yellow());
                    let (new_c, changed) = comment_out_dead_lines(&content, |l| {
                        l.starts_with("proxy") || l.starts_with("-x") || l.starts_with("--proxy")
                    });
                    if changed {
                        let _ = std::fs::copy(&curlrc_path, home.join(".curlrc.bak"));
                        let _ = std::fs::write(&curlrc_path, new_c);
                        println!(
                            "  {} Disabled dead proxy in ~/.curlrc (backup saved to ~/.curlrc.bak)",
                            "✔".green()
                        );
                        static_fixed = true;
                        fixed_count += 1;
                    }
                }
            }
        }

        let wgetrc_path = home.join(".wgetrc");
        if let Some(val) = get_wgetrc_proxy() {
            if !verify_proxy_url(&val).await {
                if let Ok(content) = std::fs::read_to_string(&wgetrc_path) {
                    println!("  [15/125] Found dead proxy `{}` in ~/.wgetrc, commenting out with backup...", val.yellow());
                    let (new_c, changed) = comment_out_dead_lines(&content, |l| {
                        l.to_ascii_lowercase().starts_with("http_proxy")
                            || l.to_ascii_lowercase().starts_with("https_proxy")
                    });
                    if changed {
                        let _ = std::fs::copy(&wgetrc_path, home.join(".wgetrc.bak"));
                        let _ = std::fs::write(&wgetrc_path, new_c);
                        println!(
                            "  {} Disabled dead proxy in ~/.wgetrc (backup saved to ~/.wgetrc.bak)",
                            "✔".green()
                        );
                        static_fixed = true;
                        fixed_count += 1;
                    }
                }
            }
        }
    }

    if let Some(val) = get_cargo_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [15/125] Found dead Cargo proxy (`{}`), cleaning ~/.cargo/config.toml...",
                val.yellow()
            );
            if let Ok(true) = crate::manager::unset_cargo_proxy() {
                println!("  {} Cleared broken Cargo global proxy.", "✔".green());
                static_fixed = true;
                fixed_count += 1;
            } else {
                println!(
                    "  [15/125] {} Cargo config has dead proxy `{}`! Please remove `proxy` in ~/.cargo/config.toml",
                    "▲".yellow(),
                    val.yellow()
                );
                static_fixed = true;
            }
        }
    }

    if !static_fixed {
        println!(
            "  [15/125] cURL, Wget, and Cargo configs: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 16. Clean dead Gem (~/.gemrc) proxy
    if let Some(val) = get_gem_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [16/125] Found dead Gem proxy (`{}` in ~/.gemrc), cleaning with backup...",
                val.yellow()
            );
            let path = get_gemrc_path();
            if path.exists() {
                let _ = std::fs::copy(&path, path.with_extension("gemrc.bak"));
            }
            if let Ok(true) = unset_gem_proxy() {
                println!(
                    "  {} Cleared broken Gem proxy in ~/.gemrc (backup saved to ~/.gemrc.bak)",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [16/125] Gem (Ruby) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [16/125] Gem (Ruby) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 17. Clean dead Subversion (~/.subversion/servers) proxy
    if let Some(val) = get_svn_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [17/125] Found dead Subversion proxy (`{}` in servers), cleaning with backup...",
                val.yellow()
            );
            let path = get_svn_servers_path();
            if path.exists() {
                let _ = std::fs::copy(&path, path.with_extension("servers.bak"));
            }
            if let Ok(true) = unset_svn_proxy() {
                println!(
                    "  {} Cleared broken Subversion proxy in servers config (backup saved)",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [17/125] Subversion (SVN) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [17/125] Subversion (SVN) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 18. Clean dead Mercurial (~/.hgrc / Mercurial.ini) proxy
    if let Some(val) = get_hg_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [18/125] Found dead Mercurial proxy (`{}` in ~/.hgrc), cleaning with backup...",
                val.yellow()
            );
            let path = get_hgrc_path();
            if path.exists() {
                let _ = std::fs::copy(&path, path.with_extension("hgrc.bak"));
            }
            if let Ok(true) = unset_hg_proxy() {
                println!(
                    "  {} Cleared broken Mercurial proxy in ~/.hgrc (backup saved)",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [18/125] Mercurial (Hg) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [18/125] Mercurial (Hg) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 19. Migrate deprecated registry.npm.taobao.org in NPM / Yarn / pnpm configs
    let node_registries = get_npm_registries();
    let mut taobao_migrated = false;
    for reg in node_registries {
        if reg.url.contains("registry.npm.taobao.org") {
            if let Some(ref path) = reg.path {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if content.contains("registry.npm.taobao.org") {
                        let backup = path.with_extension("registry.bak");
                        let _ = std::fs::write(&backup, &content);
                        let updated =
                            content.replace("registry.npm.taobao.org", "registry.npmmirror.com");
                        if std::fs::write(path, updated).is_ok() {
                            println!(
                                "  {} Migrated deprecated 'registry.npm.taobao.org' -> 'https://registry.npmmirror.com' in {}",
                                "✔".green(),
                                path.display()
                            );
                            taobao_migrated = true;
                            fixed_count += 1;
                        }
                    }
                }
            }
        }
    }
    if !taobao_migrated {
        println!(
            "  [19/125] Node / JS Registry mirrors: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 20. Inspect SSH config (~/.ssh/config) ProxyCommands
    let ssh_proxies = get_ssh_config_proxies();
    let mut ssh_has_dead = false;
    for cmd in &ssh_proxies {
        if let Some(ref target) = cmd.target {
            if !verify_proxy_url(target).await {
                println!(
                    "  [20/125] {} ~/.ssh/config Host `{}` has dead ProxyCommand target `{}`! (Git clone over SSH will hang)",
                    "▲".red().bold(),
                    cmd.host_pattern.yellow(),
                    target.yellow()
                );
                ssh_has_dead = true;
            }
        }
    }
    if !ssh_has_dead {
        println!(
            "  [20/125] SSH ProxyCommand configs: {}",
            "Clean or healthy".dimmed()
        );
    }

    // 21. Clean dead Astral uv (~/.config/uv/uv.toml) proxy
    if let Some(val) = get_uv_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [21/125] Found dead Astral uv proxy (`{}` in uv.toml), cleaning with backup...",
                val.yellow()
            );
            let path = get_uv_config_path();
            if path.exists() {
                let _ = std::fs::copy(&path, path.with_extension("toml.bak"));
            }
            if let Ok(true) = unset_uv_proxy() {
                println!(
                    "  {} Cleared broken Astral uv proxy in uv.toml (backup saved)",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [21/125] Astral uv proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [21/125] Astral uv proxy: {}", "Clean or not set".dimmed());
    }

    // 22. Clean dead Go GOPROXY mirror
    if let Some(gp) = get_goproxy() {
        let mirrors = crate::doctor::parse_goproxy(&gp);
        let mut dead_found = false;
        for m in &mirrors {
            if !verify_proxy_url(m).await {
                dead_found = true;
                break;
            }
        }
        if dead_found {
            println!(
                "  [22/125] Found dead GOPROXY mirror (`{}`), cleaning / resetting to default...",
                gp.yellow()
            );
            if let Ok(true) = unset_go_proxy() {
                println!(
                    "  {} Reset broken GOPROXY to default (https://proxy.golang.org,direct)",
                    "✔".green()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [22/125] Go GOPROXY: {}", "Reachable or direct".green());
        }
    } else {
        println!("  [22/125] Go GOPROXY: {}", "Clean or default".dimmed());
    }

    // 23. Clean dead Bazel (~/.bazelrc) action_env proxy
    if let Some((val, path)) = get_bazel_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [23/125] Found dead Bazel action_env proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bazelrc.bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_bazel_proxy() {
                println!(
                    "  {} Cleared broken Bazel action_env proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [23/125] Bazel proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [23/125] Bazel proxy: {}", "Clean or not set".dimmed());
    }

    // 24. Clean dead PHP Composer (config.json) proxy
    if let Some((val, path)) = get_composer_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [24/125] Found dead PHP Composer proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("composer.bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_composer_proxy() {
                println!(
                    "  {} Cleared broken PHP Composer proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [24/125] PHP Composer proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [24/125] PHP Composer proxy: {}", "Clean or not set".dimmed());
    }

    // 25. Clean dead Conan C/C++ (global.conf / conan.conf) proxy
    if let Some((val, path)) = get_conan_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [25/125] Found dead Conan C/C++ proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("conan.bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_conan_proxy() {
                println!(
                    "  {} Cleared broken Conan C/C++ proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [25/125] Conan C/C++ proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [25/125] Conan C/C++ proxy: {}", "Clean or not set".dimmed());
    }

    // 26. Clean dead Scala Build Tool (~/.sbt/sbtopts) proxy
    if let Some((val, path)) = get_sbt_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [26/125] Found dead Scala (sbt) proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("sbtopts.bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_sbt_proxy() {
                println!(
                    "  {} Cleared broken Scala (sbt) proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [26/125] Scala (sbt) proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [26/125] Scala (sbt) proxy: {}", "Clean or not set".dimmed());
    }

    // 27. Clean dead Kubernetes (~/.kube/config) cluster proxy-url
    if let Some(kpath) = get_kubeconfig_path() {
        if let Ok(content) = std::fs::read_to_string(&kpath) {
            let kube_proxies = parse_kubeconfig_proxies(&content);
            let mut dead_cluster = false;
            for (_cluster, purl) in &kube_proxies {
                if !verify_proxy_url(purl).await {
                    dead_cluster = true;
                    break;
                }
            }
            if dead_cluster {
                println!(
                    "  [27/125] Found dead cluster proxy-url in {}, cleaning with backup...",
                    kpath.display()
                );
                let backup = kpath.with_extension("bak");
                let _ = std::fs::copy(&kpath, &backup);
                if let Ok(true) = unset_kube_proxy() {
                    println!(
                        "  {} Cleared broken Kubernetes proxy-url in {} (backup saved)",
                        "✔".green(),
                        kpath.display()
                    );
                    fixed_count += 1;
                }
            } else if !kube_proxies.is_empty() {
                println!("  [27/125] Kubernetes (kubectl) proxy-url: {}", "Reachable".green());
            } else {
                println!("  [27/125] Kubernetes (kubectl) proxy-url: {}", "Clean or not set".dimmed());
            }
        }
    } else {
        println!("  [27/125] Kubernetes (kubectl) proxy-url: {}", "Clean or not set".dimmed());
    }

    // 28. Clean dead Julia startup (~/.julia/config/startup.jl) proxy
    if let Some((val, path)) = get_julia_startup_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [28/125] Found dead Julia startup proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("startup.bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_julia_proxy() {
                println!(
                    "  {} Cleared broken Julia startup proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [28/125] Julia startup proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [28/125] Julia startup proxy: {}", "Clean or not set".dimmed());
    }

    // 29. Clean dead GitLab CLI (~/.config/glab-cli/config.yml) proxy
    if let Some(val) = get_glab_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [29/125] Found dead GitLab CLI (glab) proxy (`{}`), cleaning...",
                val.yellow()
            );
            if let Ok(true) = unset_glab_proxy() {
                println!("  {} Cleared broken GitLab CLI (glab) proxy.", "✔".green());
                fixed_count += 1;
            }
        } else {
            println!("  [29/125] GitLab CLI (glab) proxy: {}", "Reachable".green());
        }
    } else {
        println!(
            "  [29/125] GitLab CLI (glab) proxy: {}",
            "Clean or not set".dimmed()
        );
    }

    // 30. Clean dead Nix (~/.config/nix/nix.conf) http-proxy
    if let Some((val, path)) = get_nix_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [30/125] Found dead Nix http-proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_nix_proxy() {
                println!(
                    "  {} Cleared broken Nix http-proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [30/125] Nix (nix.conf) http-proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [30/125] Nix (nix.conf) http-proxy: {}", "Clean or not set".dimmed());
    }

    // 31. Clean dead Haskell Cabal (~/.cabal/config) proxy
    if let Some((val, path)) = get_cabal_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [31/125] Found dead Haskell Cabal proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_cabal_proxy() {
                println!(
                    "  {} Cleared broken Haskell Cabal proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [31/125] Haskell Cabal proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [31/125] Haskell Cabal proxy: {}", "Clean or not set".dimmed());
    }

    // 32. Clean dead Haskell Stack (~/.stack/config.yaml) proxy
    if let Some((val, path)) = get_stack_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [32/125] Found dead Haskell Stack proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_stack_proxy() {
                println!(
                    "  {} Cleared broken Haskell Stack proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [32/125] Haskell Stack proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [32/125] Haskell Stack proxy: {}", "Clean or not set".dimmed());
    }

    // 33. Clean dead Elixir / Erlang / Gleam Hex (~/.hex/hex.config) proxy
    if let Some((val, path)) = get_hex_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [33/125] Found dead Hex package proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_hex_proxy() {
                println!(
                    "  {} Cleared broken Hex proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [33/125] Hex (Elixir/Gleam) proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [33/125] Hex (Elixir/Gleam) proxy: {}", "Clean or not set".dimmed());
    }

    // 34. Clean dead SwiftPM (~/.swiftpm/configuration/mirrors.json) package mirrors
    if let Some(mpath) = get_swiftpm_mirrors_path() {
        if let Ok(content) = std::fs::read_to_string(&mpath) {
            let mirrors = parse_swiftpm_mirrors(&content);
            let mut dead_mirrors = Vec::new();
            for m in &mirrors {
                if !verify_proxy_url(m).await {
                    dead_mirrors.push(m.as_str());
                }
            }
            if !dead_mirrors.is_empty() {
                println!(
                    "  [34/125] Found {} dead SwiftPM mirror(s) in {}, cleaning with backup...",
                    dead_mirrors.len().to_string().yellow(),
                    mpath.display()
                );
                let (new_json, modified) = clean_swiftpm_mirrors(&content, &dead_mirrors);
                if modified {
                    let backup = mpath.with_extension("bak");
                    let _ = std::fs::copy(&mpath, &backup);
                    let _ = std::fs::write(&mpath, new_json);
                    println!(
                        "  {} Pruned dead SwiftPM mirror(s) from {} (backup saved)",
                        "✔".green(),
                        mpath.display()
                    );
                    fixed_count += 1;
                }
            } else if !mirrors.is_empty() {
                println!("  [34/125] SwiftPM mirrors: {}", "Reachable".green());
            } else {
                println!("  [34/125] SwiftPM mirrors: {}", "Clean or empty".dimmed());
            }
        }
    } else {
        println!("  [34/125] SwiftPM mirrors: {}", "Clean or not set".dimmed());
    }

    // 35. Clean dead R package manager (~/.Renviron) proxy
    if let Some((val, path)) = get_renviron_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [35/125] Found dead R proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_renviron_proxy() {
                println!(
                    "  {} Cleared broken R proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [35/125] R (~/.Renviron) proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [35/125] R (~/.Renviron) proxy: {}", "Clean or not set".dimmed());
    }

    // 36. Clean dead OCaml OPAM (~/.opam/config) proxy
    if let Some((val, path)) = get_opam_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [36/125] Found dead OCaml OPAM proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_opam_proxy() {
                println!(
                    "  {} Cleared broken OCaml OPAM proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [36/125] OCaml OPAM proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [36/125] OCaml OPAM proxy: {}", "Clean or not set".dimmed());
    }

    // 37. Clean dead Terraform / OpenTofu (~/.terraformrc / ~/.tofurc) network mirrors
    let tf_paths = get_terraform_config_paths();
    if !tf_paths.is_empty() {
        for tpath in tf_paths {
            if let Ok(content) = std::fs::read_to_string(&tpath) {
                let mirrors = parse_terraform_network_mirrors(&content);
                let mut dead_mirrors = Vec::new();
                for mirror in &mirrors {
                    if !verify_proxy_url(mirror).await {
                        dead_mirrors.push(mirror.as_str());
                    }
                }
                if !dead_mirrors.is_empty() {
                    println!(
                        "  [37/125] Found {} dead Terraform/OpenTofu mirror(s) in {}, cleaning with backup...",
                        dead_mirrors.len().to_string().yellow(),
                        tpath.display()
                    );
                    let (new_content, modified) =
                        clean_terraform_network_mirrors(&content, &dead_mirrors);
                    if modified {
                        let backup = tpath.with_extension("bak");
                        let _ = std::fs::copy(&tpath, &backup);
                        let _ = std::fs::write(&tpath, new_content);
                        println!(
                            "  {} Disabled dead Terraform/OpenTofu mirror(s) in {} (backup saved)",
                            "✔".green(),
                            tpath.display()
                        );
                        fixed_count += 1;
                    }
                } else if !mirrors.is_empty() {
                    println!("  [37/125] Terraform/OpenTofu network mirrors: {}", "Reachable".green());
                } else {
                    println!("  [37/125] Terraform/OpenTofu network mirrors: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [37/125] Terraform/OpenTofu network mirrors: {}", "Clean or not set".dimmed());
    }

    // 38. Clean dead Clojure Leiningen (~/.lein/profiles.clj) proxy
    if let Some((val, path)) = get_lein_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [38/125] Found dead Clojure Leiningen proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_lein_proxy() {
                println!(
                    "  {} Cleared broken Clojure Leiningen proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [38/125] Clojure Leiningen proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [38/125] Clojure Leiningen proxy: {}", "Clean or not set".dimmed());
    }

    // 39. Clean dead Perl CPAN (~/.cpan/CPAN/MyConfig.pm) proxy
    if let Some((val, path)) = get_cpan_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [39/125] Found dead Perl CPAN proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_cpan_proxy() {
                println!(
                    "  {} Cleared broken Perl CPAN proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [39/125] Perl CPAN proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [39/125] Perl CPAN proxy: {}", "Clean or not set".dimmed());
    }

    // 40. Clean dead LuaRocks (~/.luarocks/config.lua) proxy
    if let Some((val, path)) = get_luarocks_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [40/125] Found dead LuaRocks proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_luarocks_proxy() {
                println!(
                    "  {} Cleared broken LuaRocks proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [40/125] LuaRocks proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [40/125] LuaRocks proxy: {}", "Clean or not set".dimmed());
    }

    // 41. Clean dead Nim (Nimble, ~/.config/nimble/nimble.ini) proxy
    if let Some((val, path)) = get_nimble_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [41/125] Found dead Nimble proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_nimble_proxy() {
                println!(
                    "  {} Cleared broken Nimble proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [41/125] Nimble proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [41/125] Nimble proxy: {}", "Clean or not set".dimmed());
    }

    // 42. Clean dead Erlang (Rebar3, ~/.config/rebar3/rebar.config) proxy
    if let Some((val, path)) = get_rebar3_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [42/125] Found dead Rebar3 proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_rebar3_proxy() {
                println!(
                    "  {} Cleared broken Rebar3 proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [42/125] Rebar3 proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [42/125] Rebar3 proxy: {}", "Clean or not set".dimmed());
    }

    // 43. Clean dead Scoop (~/.config/scoop/config.json) proxy
    if let Some((val, path)) = get_scoop_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [43/125] Found dead Scoop proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_scoop_proxy() {
                println!(
                    "  {} Cleared broken Scoop proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [43/125] Scoop proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [43/125] Scoop proxy: {}", "Clean or not set".dimmed());
    }

    // 44. Clean dead Ruby Bundler (~/.bundle/config) proxy
    if let Some((val, path)) = get_bundler_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [44/125] Found dead Bundler proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_bundler_proxy() {
                println!(
                    "  {} Cleared broken Bundler proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [44/125] Bundler proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [44/125] Bundler proxy: {}", "Clean or not set".dimmed());
    }

    // 45. Clean dead Google Cloud SDK (gcloud) proxy (~/.config/gcloud/configurations/config_*)
    if let Some((val, path)) = get_gcloud_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [45/125] Found dead Google Cloud SDK (gcloud) proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_gcloud_proxy() {
                println!(
                    "  {} Cleared broken Google Cloud SDK (gcloud) proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [45/125] Google Cloud SDK (gcloud) proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [45/125] Google Cloud SDK (gcloud) proxy: {}", "Clean or not set".dimmed());
    }

    // 46. Clean dead AWS CLI proxy (~/.aws/config)
    if let Some((val, path)) = get_aws_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [46/125] Found dead AWS CLI proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_aws_proxy() {
                println!(
                    "  {} Cleared broken AWS CLI proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [46/125] AWS CLI proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [46/125] AWS CLI proxy: {}", "Clean or not set".dimmed());
    }

    // 47. Clean dead Microsoft Azure CLI proxy (~/.azure/config)
    if let Some((val, path)) = get_azure_proxy() {
        if !verify_proxy_url(&val).await {
            println!(
                "  [47/125] Found dead Azure CLI proxy (`{}` in {}), cleaning with backup...",
                val.yellow(),
                path.display()
            );
            let backup = path.with_extension("bak");
            let _ = std::fs::copy(&path, &backup);
            if let Ok(true) = unset_azure_proxy() {
                println!(
                    "  {} Cleared broken Azure CLI proxy in {} (backup saved)",
                    "✔".green(),
                    path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [47/125] Azure CLI proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [47/125] Azure CLI proxy: {}", "Clean or not set".dimmed());
    }

    // 48. Clean dead Mamba / Micromamba proxy (~/.mambarc)
    if let Some(mamba_path) = crate::doctor::get_mambarc_path() {
        if mamba_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&mamba_path) {
                if let Some(cfg) = crate::doctor::parse_condarc_proxy(&content) {
                    let mut dead = false;
                    if let Some(ref http) = cfg.http {
                        if !verify_proxy_url(http).await {
                            dead = true;
                        }
                    }
                    if let Some(ref https) = cfg.https {
                        if !verify_proxy_url(https).await {
                            dead = true;
                        }
                    }
                    if dead {
                        println!(
                            "  [48/125] Found dead Mamba proxy in {}, cleaning with backup...",
                            mamba_path.display()
                        );
                        let backup = mamba_path.with_extension("bak");
                        let _ = std::fs::copy(&mamba_path, &backup);
                        if let Ok(true) = unset_mamba_proxy() {
                            println!(
                                "  {} Cleared broken Mamba proxy in {} (backup saved)",
                                "✔".green(),
                                mamba_path.display()
                            );
                            fixed_count += 1;
                        }
                    } else {
                        println!("  [48/125] Mamba proxy: {}", "Reachable".green());
                    }
                } else {
                    println!("  [48/125] Mamba proxy: {}", "Clean or not set".dimmed());
                }
            } else {
                println!("  [48/125] Mamba proxy: {}", "Clean or not set".dimmed());
            }
        } else {
            println!("  [48/125] Mamba proxy: {}", "Clean or not set".dimmed());
        }
    } else {
        println!("  [48/125] Mamba proxy: {}", "Clean or not set".dimmed());
    }

    // 49. Clean dead Podman proxy (~/.config/containers/containers.conf)
    if let Some((podproxy, podpath)) = get_podman_proxy() {
        if !verify_proxy_url(&podproxy).await {
            println!(
                "  [49/125] Found dead Podman proxy (`{}` in {}), cleaning with backup...",
                podproxy.yellow(),
                podpath.display()
            );
            let backup = podpath.with_extension("bak");
            let _ = std::fs::copy(&podpath, &backup);
            if let Ok(true) = unset_podman_proxy() {
                println!(
                    "  {} Cleared broken Podman proxy in {} (backup saved)",
                    "✔".green(),
                    podpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [49/125] Podman proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [49/125] Podman proxy: {}", "Clean or not set".dimmed());
    }

    // 50. Clean dead Vagrant proxy (~/.vagrant.d/Vagrantfile)
    if let Some((vagproxy, vagpath)) = get_vagrant_proxy() {
        if !verify_proxy_url(&vagproxy).await {
            println!(
                "  [50/125] Found dead Vagrant proxy (`{}` in {}), cleaning with backup...",
                vagproxy.yellow(),
                vagpath.display()
            );
            let backup = vagpath.with_extension("bak");
            let _ = std::fs::copy(&vagpath, &backup);
            if let Ok(true) = unset_vagrant_proxy() {
                println!(
                    "  {} Cleared broken Vagrant proxy in {} (backup saved)",
                    "✔".green(),
                    vagpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [50/125] Vagrant proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [50/125] Vagrant proxy: {}", "Clean or not set".dimmed());
    }

    // 51. Clean dead Chocolatey proxy (chocolatey.config)
    if let Some((chocoproxy, chocopath)) = get_choco_proxy() {
        if !verify_proxy_url(&chocoproxy).await {
            println!(
                "  [51/125] Found dead Chocolatey proxy (`{}` in {}), cleaning with backup...",
                chocoproxy.yellow(),
                chocopath.display()
            );
            let backup = chocopath.with_extension("bak");
            let _ = std::fs::copy(&chocopath, &backup);
            if let Ok(true) = unset_choco_proxy() {
                println!(
                    "  {} Cleared broken Chocolatey proxy in {} (backup saved)",
                    "✔".green(),
                    chocopath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [51/125] Chocolatey proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [51/125] Chocolatey proxy: {}", "Clean or not set".dimmed());
    }

    // 52. Clean dead Colima proxy (~/.colima/default/colima.yaml)
    if let Some((colimaproxy, colimapath)) = get_colima_proxy() {
        if !verify_proxy_url(&colimaproxy).await {
            println!(
                "  [52/125] Found dead Colima proxy (`{}` in {}), cleaning with backup...",
                colimaproxy.yellow(),
                colimapath.display()
            );
            let backup = colimapath.with_extension("bak");
            let _ = std::fs::copy(&colimapath, &backup);
            if let Ok(true) = unset_colima_proxy() {
                println!(
                    "  {} Cleared broken Colima proxy in {} (backup saved)",
                    "✔".green(),
                    colimapath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [52/125] Colima proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [52/125] Colima proxy: {}", "Clean or not set".dimmed());
    }

    // 53. Clean dead Minikube proxy (~/.minikube/config/config.json)
    if let Some((miniproxy, minipath)) = get_minikube_proxy() {
        if !verify_proxy_url(&miniproxy).await {
            println!(
                "  [53/125] Found dead Minikube proxy (`{}` in {}), cleaning with backup...",
                miniproxy.yellow(),
                minipath.display()
            );
            let backup = minipath.with_extension("bak");
            let _ = std::fs::copy(&minipath, &backup);
            if let Ok(true) = unset_minikube_proxy() {
                println!(
                    "  {} Cleared broken Minikube proxy in {} (backup saved)",
                    "✔".green(),
                    minipath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [53/125] Minikube proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [53/125] Minikube proxy: {}", "Clean or not set".dimmed());
    }

    // 54. Clean dead Spack proxy (~/.spack/config.yaml)
    if let Some((spackproxy, spackpath)) = get_spack_proxy() {
        if !verify_proxy_url(&spackproxy).await {
            println!(
                "  [54/125] Found dead Spack proxy (`{}` in {}), cleaning with backup...",
                spackproxy.yellow(),
                spackpath.display()
            );
            let backup = spackpath.with_extension("bak");
            let _ = std::fs::copy(&spackpath, &backup);
            if let Ok(true) = unset_spack_proxy() {
                println!(
                    "  {} Cleared broken Spack proxy in {} (backup saved)",
                    "✔".green(),
                    spackpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [54/125] Spack proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [54/125] Spack proxy: {}", "Clean or not set".dimmed());
    }

    // 55. Clean dead MacPorts proxy (~/.macports/macports.conf / /opt/local/etc/macports/macports.conf)
    if let Some((mpproxy, mppath)) = get_macports_proxy() {
        if !verify_proxy_url(&mpproxy).await {
            println!(
                "  [55/125] Found dead MacPorts proxy (`{}` in {}), cleaning with backup...",
                mpproxy.yellow(),
                mppath.display()
            );
            let backup = mppath.with_extension("bak");
            let _ = std::fs::copy(&mppath, &backup);
            if let Ok(true) = unset_macports_proxy() {
                println!(
                    "  {} Cleared broken MacPorts proxy in {} (backup saved)",
                    "✔".green(),
                    mppath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [55/125] MacPorts proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [55/125] MacPorts proxy: {}", "Clean or not set".dimmed());
    }

    // 56. Clean dead APT proxy (/etc/apt/apt.conf.d/* / ~/.apt/apt.conf)
    if let Some((aptproxy, aptpath)) = get_apt_proxy() {
        if !verify_proxy_url(&aptproxy).await {
            println!(
                "  [56/125] Found dead APT proxy (`{}` in {}), cleaning with backup...",
                aptproxy.yellow(),
                aptpath.display()
            );
            let backup = aptpath.with_extension("bak");
            let _ = std::fs::copy(&aptpath, &backup);
            if let Ok(true) = unset_apt_proxy() {
                println!(
                    "  {} Cleared broken APT proxy in {} (backup saved)",
                    "✔".green(),
                    aptpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [56/125] APT proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [56/125] APT proxy: {}", "Clean or not set".dimmed());
    }

    // 57. Clean dead DNF & YUM proxy (/etc/dnf/dnf.conf / ~/.config/dnf/dnf.conf / /etc/yum.conf)
    if let Some((dnfproxy, dnfpath)) = get_dnf_proxy() {
        if !verify_proxy_url(&dnfproxy).await {
            println!(
                "  [57/125] Found dead DNF/YUM proxy (`{}` in {}), cleaning with backup...",
                dnfproxy.yellow(),
                dnfpath.display()
            );
            let backup = dnfpath.with_extension("bak");
            let _ = std::fs::copy(&dnfpath, &backup);
            if let Ok(true) = unset_dnf_proxy() {
                println!(
                    "  {} Cleared broken DNF/YUM proxy in {} (backup saved)",
                    "✔".green(),
                    dnfpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [57/125] DNF/YUM proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [57/125] DNF/YUM proxy: {}", "Clean or not set".dimmed());
    }

    // 58. Clean dead Zypper proxy (/etc/zypp/zypp.conf / ~/.zypp/zypp.conf)
    if let Some((zyppproxy, zypppath)) = get_zypper_proxy() {
        if !verify_proxy_url(&zyppproxy).await {
            println!(
                "  [58/125] Found dead Zypper proxy (`{}` in {}), cleaning with backup...",
                zyppproxy.yellow(),
                zypppath.display()
            );
            let backup = zypppath.with_extension("bak");
            let _ = std::fs::copy(&zypppath, &backup);
            if let Ok(true) = unset_zypper_proxy() {
                println!(
                    "  {} Cleared broken Zypper proxy in {} (backup saved)",
                    "✔".green(),
                    zypppath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [58/125] Zypper proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [58/125] Zypper proxy: {}", "Clean or not set".dimmed());
    }

    // 59. Clean dead Pacman package manager proxy in /etc/pacman.conf
    if let Some((pacproxy, pacpath)) = get_pacman_proxy() {
        if !verify_proxy_url(&pacproxy).await {
            println!(
                "  [59/125] Found dead Pacman proxy (`{}` in {}), cleaning with backup...",
                pacproxy.yellow(),
                pacpath.display()
            );
            let backup = pacpath.with_extension("bak");
            let _ = std::fs::copy(&pacpath, &backup);
            if let Ok(true) = unset_pacman_proxy() {
                println!(
                    "  {} Cleared broken Pacman proxy in {} (backup saved)",
                    "✔".green(),
                    pacpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [59/125] Pacman proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [59/125] Pacman proxy: {}", "Clean or not set".dimmed());
    }

    // 60. Clean dead FreeBSD pkg package manager proxy in /usr/local/etc/pkg.conf
    if let Some((pkgproxy, pkgpath)) = get_pkg_proxy() {
        if !verify_proxy_url(&pkgproxy).await {
            println!(
                "  [60/125] Found dead FreeBSD pkg proxy (`{}` in {}), cleaning with backup...",
                pkgproxy.yellow(),
                pkgpath.display()
            );
            let backup = pkgpath.with_extension("bak");
            let _ = std::fs::copy(&pkgpath, &backup);
            if let Ok(true) = unset_pkg_proxy() {
                println!(
                    "  {} Cleared broken FreeBSD pkg proxy in {} (backup saved)",
                    "✔".green(),
                    pkgpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [60/125] FreeBSD pkg proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [60/125] FreeBSD pkg proxy: {}", "Clean or not set".dimmed());
    }

    // 61. Clean dead Portage package manager proxy in make.conf
    if let Some((portproxy, portpath)) = get_portage_proxy() {
        if !verify_proxy_url(&portproxy).await {
            println!(
                "  [61/125] Found dead Portage proxy (`{}` in {}), cleaning with backup...",
                portproxy.yellow(),
                portpath.display()
            );
            let backup = portpath.with_extension("bak");
            let _ = std::fs::copy(&portpath, &backup);
            if let Ok(true) = unset_portage_proxy() {
                println!(
                    "  {} Cleared broken Portage proxy in {} (backup saved)",
                    "✔".green(),
                    portpath.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [61/125] Portage proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [61/125] Portage proxy: {}", "Clean or not set".dimmed());
    }

    // 62. Clean dead Docker Daemon engine proxy in daemon.json
    if let Some((daemon_proxy, daemon_path)) = get_docker_daemon_proxy() {
        if !verify_proxy_url(&daemon_proxy).await {
            println!(
                "  [62/125] Found dead Docker Daemon proxy (`{}` in {}), cleaning with backup...",
                daemon_proxy.yellow(),
                daemon_path.display()
            );
            let backup = daemon_path.with_extension("bak");
            let _ = std::fs::copy(&daemon_path, &backup);
            if let Ok(true) = unset_docker_daemon_proxy() {
                println!(
                    "  {} Cleared broken Docker Daemon proxy in {} (backup saved)",
                    "✔".green(),
                    daemon_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [62/125] Docker Daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [62/125] Docker Daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 63. Clean dead Flatpak package manager proxy in repo/config
    if let Some((flatpak_proxy, flatpak_path)) = get_flatpak_proxy() {
        if !verify_proxy_url(&flatpak_proxy).await {
            println!(
                "  [63/125] Found dead Flatpak proxy (`{}` in {}), cleaning with backup...",
                flatpak_proxy.yellow(),
                flatpak_path.display()
            );
            let backup = flatpak_path.with_extension("bak");
            let _ = std::fs::copy(&flatpak_path, &backup);
            if let Ok(true) = unset_flatpak_proxy() {
                println!(
                    "  {} Cleared broken Flatpak proxy in {} (backup saved)",
                    "✔".green(),
                    flatpak_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [63/125] Flatpak proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [63/125] Flatpak proxy: {}", "Clean or not set".dimmed());
    }

    // 64. Clean dead Snap package manager proxy in snapd config / system settings
    if let Some((snap_proxy, snap_path)) = get_snap_proxy() {
        if !verify_proxy_url(&snap_proxy).await {
            println!(
                "  [64/125] Found dead Snap proxy (`{}` in {}), cleaning with backup...",
                snap_proxy.yellow(),
                snap_path.display()
            );
            if snap_path.exists() {
                let backup = snap_path.with_extension("bak");
                let _ = std::fs::copy(&snap_path, &backup);
            }
            if let Ok(true) = unset_snap_proxy() {
                println!(
                    "  {} Cleared broken Snap proxy in {} (backup saved)",
                    "✔".green(),
                    snap_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [64/125] Snap proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [64/125] Snap proxy: {}", "Clean or not set".dimmed());
    }

    // 65. Clean dead Ollama local AI runner daemon proxy in systemd / config
    if let Some((ollama_proxy, ollama_path)) = get_ollama_proxy() {
        if !verify_proxy_url(&ollama_proxy).await {
            println!(
                "  [65/125] Found dead Ollama daemon proxy (`{}` in {}), cleaning with backup...",
                ollama_proxy.yellow(),
                ollama_path.display()
            );
            if ollama_path.exists() {
                let backup = ollama_path.with_extension("bak");
                let _ = std::fs::copy(&ollama_path, &backup);
            }
            if let Ok(true) = unset_ollama_proxy() {
                println!(
                    "  {} Cleared broken Ollama proxy in {} (backup saved)",
                    "✔".green(),
                    ollama_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [65/125] Ollama daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [65/125] Ollama daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 66. Clean dead Containerd (Kubernetes / CRI) container runtime daemon proxy in systemd / config
    if let Some((containerd_proxy, containerd_path)) = get_containerd_proxy() {
        if !verify_proxy_url(&containerd_proxy).await {
            println!(
                "  [66/125] Found dead Containerd daemon proxy (`{}` in {}), cleaning with backup...",
                containerd_proxy.yellow(),
                containerd_path.display()
            );
            if containerd_path.exists() {
                let backup = containerd_path.with_extension("bak");
                let _ = std::fs::copy(&containerd_path, &backup);
            }
            if let Ok(true) = unset_containerd_proxy() {
                println!(
                    "  {} Cleared broken Containerd proxy in {} (backup saved)",
                    "✔".green(),
                    containerd_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [66/125] Containerd daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [66/125] Containerd daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 67. Clean dead BuildKit (Docker Buildx / buildctl) container builder daemon proxy in systemd / config
    if let Some((buildkit_proxy, buildkit_path)) = get_buildkit_proxy() {
        if !verify_proxy_url(&buildkit_proxy).await {
            println!(
                "  [67/125] Found dead BuildKit daemon proxy (`{}` in {}), cleaning with backup...",
                buildkit_proxy.yellow(),
                buildkit_path.display()
            );
            if buildkit_path.exists() {
                let backup = buildkit_path.with_extension("bak");
                let _ = std::fs::copy(&buildkit_path, &backup);
            }
            if let Ok(true) = unset_buildkit_proxy() {
                println!(
                    "  {} Cleared broken BuildKit proxy in {} (backup saved)",
                    "✔".green(),
                    buildkit_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [67/125] BuildKit daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [67/125] BuildKit daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 68. Clean dead K3s (Lightweight Kubernetes / k3s-agent) node daemon proxy in systemd / env
    if let Some((k3s_proxy, k3s_path)) = get_k3s_proxy() {
        if !verify_proxy_url(&k3s_proxy).await {
            println!(
                "  [68/125] Found dead K3s daemon proxy (`{}` in {}), cleaning with backup...",
                k3s_proxy.yellow(),
                k3s_path.display()
            );
            if k3s_path.exists() {
                let backup = k3s_path.with_extension("bak");
                let _ = std::fs::copy(&k3s_path, &backup);
            }
            if let Ok(true) = unset_k3s_proxy() {
                println!(
                    "  {} Cleared broken K3s proxy in {} (backup saved)",
                    "✔".green(),
                    k3s_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [68/125] K3s daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [68/125] K3s daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 69. Clean dead MicroK8s (Canonical Zero-ops Kubernetes) node daemon proxy in /var/snap/microk8s/current/args/containerd-env
    if let Some((mk8s_proxy, mk8s_path)) = get_microk8s_proxy() {
        if !verify_proxy_url(&mk8s_proxy).await {
            println!(
                "  [69/125] Found dead MicroK8s daemon proxy (`{}` in {}), cleaning with backup...",
                mk8s_proxy.yellow(),
                mk8s_path.display()
            );
            if mk8s_path.exists() {
                let backup = mk8s_path.with_extension("bak");
                let _ = std::fs::copy(&mk8s_path, &backup);
            }
            if let Ok(true) = unset_microk8s_proxy() {
                println!(
                    "  {} Cleared broken MicroK8s proxy in {} (backup saved)",
                    "✔".green(),
                    mk8s_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [69/125] MicroK8s daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [69/125] MicroK8s daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 70. Clean dead k0s (Mirantis Zero-friction Kubernetes) node daemon proxy in systemd / default
    if let Some((k0s_proxy, k0s_path)) = get_k0s_proxy() {
        if !verify_proxy_url(&k0s_proxy).await {
            println!(
                "  [70/125] Found dead k0s daemon proxy (`{}` in {}), cleaning with backup...",
                k0s_proxy.yellow(),
                k0s_path.display()
            );
            if k0s_path.exists() {
                let backup = k0s_path.with_extension("bak");
                let _ = std::fs::copy(&k0s_path, &backup);
            }
            if let Ok(true) = unset_k0s_proxy() {
                println!(
                    "  {} Cleared broken k0s proxy in {} (backup saved)",
                    "✔".green(),
                    k0s_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [70/125] k0s daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [70/125] k0s daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 71. Clean dead RKE2 (Rancher Kubernetes Engine 2) node daemon proxy in /etc/rancher/rke2/rke2.env / systemd
    if let Some((rke2_proxy, rke2_path)) = get_rke2_proxy() {
        if !verify_proxy_url(&rke2_proxy).await {
            println!(
                "  [71/125] Found dead RKE2 daemon proxy (`{}` in {}), cleaning with backup...",
                rke2_proxy.yellow(),
                rke2_path.display()
            );
            if rke2_path.exists() {
                let backup = rke2_path.with_extension("bak");
                let _ = std::fs::copy(&rke2_path, &backup);
            }
            if let Ok(true) = unset_rke2_proxy() {
                println!(
                    "  {} Cleared broken RKE2 proxy in {} (backup saved)",
                    "✔".green(),
                    rke2_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [71/125] RKE2 daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [71/125] RKE2 daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 72. Clean dead Nomad (HashiCorp Nomad Workload Daemon) proxy in systemd / default
    if let Some((nomad_proxy, nomad_path)) = get_nomad_proxy() {
        if !verify_proxy_url(&nomad_proxy).await {
            println!(
                "  [72/125] Found dead Nomad daemon proxy (`{}` in {}), cleaning with backup...",
                nomad_proxy.yellow(),
                nomad_path.display()
            );
            if nomad_path.exists() {
                let backup = nomad_path.with_extension("bak");
                let _ = std::fs::copy(&nomad_path, &backup);
            }
            if let Ok(true) = unset_nomad_proxy() {
                println!(
                    "  {} Cleared broken Nomad proxy in {} (backup saved)",
                    "✔".green(),
                    nomad_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [72/125] Nomad daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [72/125] Nomad daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 73. Clean dead Kubelet (Kubernetes Node Daemon) proxy in systemd / default
    if let Some((kubelet_proxy, kubelet_path)) = get_kubelet_proxy() {
        if !verify_proxy_url(&kubelet_proxy).await {
            println!(
                "  [73/125] Found dead Kubelet daemon proxy (`{}` in {}), cleaning with backup...",
                kubelet_proxy.yellow(),
                kubelet_path.display()
            );
            if kubelet_path.exists() {
                let backup = kubelet_path.with_extension("bak");
                let _ = std::fs::copy(&kubelet_path, &backup);
            }
            if let Ok(true) = unset_kubelet_proxy() {
                println!(
                    "  {} Cleared broken Kubelet proxy in {} (backup saved)",
                    "✔".green(),
                    kubelet_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [73/125] Kubelet daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [73/125] Kubelet daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 74. Clean dead GitLab Runner (CI/CD Workload Daemon) proxy in config.toml / systemd
    if let Some((gl_proxy, gl_path)) = get_gitlab_runner_proxy() {
        if !verify_proxy_url(&gl_proxy).await {
            println!(
                "  [74/125] Found dead GitLab Runner daemon proxy (`{}` in {}), cleaning with backup...",
                gl_proxy.yellow(),
                gl_path.display()
            );
            if gl_path.exists() {
                let backup = gl_path.with_extension("bak");
                let _ = std::fs::copy(&gl_path, &backup);
            }
            if let Ok(true) = unset_gitlab_runner_proxy() {
                println!(
                    "  {} Cleared broken GitLab Runner proxy in {} (backup saved)",
                    "✔".green(),
                    gl_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [74/125] GitLab Runner daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [74/125] GitLab Runner daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 75. Clean dead GitHub Actions Runner (CI/CD Daemon) proxy in .proxy / .env / systemd
    if let Some((runner_proxy, runner_path)) = get_actions_runner_proxy() {
        if !verify_proxy_url(&runner_proxy).await {
            println!(
                "  [75/125] Found dead GitHub Actions Runner daemon proxy (`{}` in {}), cleaning with backup...",
                runner_proxy.yellow(),
                runner_path.display()
            );
            if runner_path.exists() {
                let backup = runner_path.with_extension("bak");
                let _ = std::fs::copy(&runner_path, &backup);
            }
            if let Ok(true) = unset_actions_runner_proxy() {
                println!(
                    "  {} Cleared broken GitHub Actions Runner proxy in {} (backup saved)",
                    "✔".green(),
                    runner_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [75/125] GitHub Actions Runner daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [75/125] GitHub Actions Runner daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 76. Clean dead Jenkins CI Automation daemon proxy in proxy.xml / default / systemd
    if let Some((jenkins_proxy, jenkins_path)) = get_jenkins_proxy() {
        if !verify_proxy_url(&jenkins_proxy).await {
            println!(
                "  [76/125] Found dead Jenkins CI daemon proxy (`{}` in {}), cleaning with backup...",
                jenkins_proxy.yellow(),
                jenkins_path.display()
            );
            if jenkins_path.exists() {
                let backup = jenkins_path.with_extension("bak");
                let _ = std::fs::copy(&jenkins_path, &backup);
            }
            if let Ok(true) = unset_jenkins_proxy() {
                println!(
                    "  {} Cleared broken Jenkins CI proxy in {} (backup saved)",
                    "✔".green(),
                    jenkins_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [76/125] Jenkins CI daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [76/125] Jenkins CI daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 77. Clean dead Buildkite Agent CI/CD Daemon proxy in buildkite-agent.cfg / default / systemd
    if let Some((bk_proxy, bk_path)) = get_buildkite_agent_proxy() {
        if !verify_proxy_url(&bk_proxy).await {
            println!(
                "  [77/125] Found dead Buildkite Agent daemon proxy (`{}` in {}), cleaning with backup...",
                bk_proxy.yellow(),
                bk_path.display()
            );
            if bk_path.exists() {
                let backup = bk_path.with_extension("bak");
                let _ = std::fs::copy(&bk_path, &backup);
            }
            if let Ok(true) = unset_buildkite_agent_proxy() {
                println!(
                    "  {} Cleared broken Buildkite Agent proxy in {} (backup saved)",
                    "✔".green(),
                    bk_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [77/125] Buildkite Agent daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [77/125] Buildkite Agent daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 78. Clean dead Woodpecker CI Runner daemon proxy in agent.conf / default / systemd
    if let Some((wp_proxy, wp_path)) = get_woodpecker_agent_proxy() {
        if !verify_proxy_url(&wp_proxy).await {
            println!(
                "  [78/125] Found dead Woodpecker CI runner daemon proxy (`{}` in {}), cleaning with backup...",
                wp_proxy.yellow(),
                wp_path.display()
            );
            if wp_path.exists() {
                let backup = wp_path.with_extension("bak");
                let _ = std::fs::copy(&wp_path, &backup);
            }
            if let Ok(true) = unset_woodpecker_agent_proxy() {
                println!(
                    "  {} Cleared broken Woodpecker CI runner proxy in {} (backup saved)",
                    "✔".green(),
                    wp_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [78/125] Woodpecker CI runner daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [78/125] Woodpecker CI runner daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 79. Clean dead Drone CI Runner daemon proxy in config / default / systemd
    if let Some((drone_proxy, drone_path)) = get_drone_runner_proxy() {
        if !verify_proxy_url(&drone_proxy).await {
            println!(
                "  [79/125] Found dead Drone CI runner daemon proxy (`{}` in {}), cleaning with backup...",
                drone_proxy.yellow(),
                drone_path.display()
            );
            if drone_path.exists() {
                let backup = drone_path.with_extension("bak");
                let _ = std::fs::copy(&drone_path, &backup);
            }
            if let Ok(true) = unset_drone_runner_proxy() {
                println!(
                    "  {} Cleared broken Drone CI runner proxy in {} (backup saved)",
                    "✔".green(),
                    drone_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [79/125] Drone CI runner daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [79/125] Drone CI runner daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 80. Clean dead Gitea & Forgejo Actions Runner (act_runner) daemon proxy in config.yaml / default / systemd
    if let Some((act_proxy, act_path)) = get_act_runner_proxy() {
        if !verify_proxy_url(&act_proxy).await {
            println!(
                "  [80/125] Found dead Gitea/Forgejo Actions Runner daemon proxy (`{}` in {}), cleaning with backup...",
                act_proxy.yellow(),
                act_path.display()
            );
            if act_path.exists() {
                let backup = act_path.with_extension("bak");
                let _ = std::fs::copy(&act_path, &backup);
            }
            if let Ok(true) = unset_act_runner_proxy() {
                println!(
                    "  {} Cleared broken Gitea/Forgejo Actions Runner proxy in {} (backup saved)",
                    "✔".green(),
                    act_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [80/125] Gitea/Forgejo Actions Runner daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [80/125] Gitea/Forgejo Actions Runner daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 81. Clean dead CircleCI Runner daemon proxy in config.yaml / default / systemd
    if let Some((cci_proxy, cci_path)) = get_circleci_runner_proxy() {
        if !verify_proxy_url(&cci_proxy).await {
            println!(
                "  [81/125] Found dead CircleCI runner daemon proxy (`{}` in {}), cleaning with backup...",
                cci_proxy.yellow(),
                cci_path.display()
            );
            if cci_path.exists() {
                let backup = cci_path.with_extension("bak");
                let _ = std::fs::copy(&cci_path, &backup);
            }
            if let Ok(true) = unset_circleci_runner_proxy() {
                println!(
                    "  {} Cleared broken CircleCI runner proxy in {} (backup saved)",
                    "✔".green(),
                    cci_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [81/125] CircleCI runner daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [81/125] CircleCI runner daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 82. Clean dead Azure Pipelines Agent (vsts-agent / azp-agent) daemon proxy in .proxy / .env / systemd
    if let Some((azp_proxy, azp_path)) = get_azure_pipelines_agent_proxy() {
        if !verify_proxy_url(&azp_proxy).await {
            println!(
                "  [82/125] Found dead Azure Pipelines Agent daemon proxy (`{}` in {}), cleaning with backup...",
                azp_proxy.yellow(),
                azp_path.display()
            );
            if azp_path.exists() {
                let backup = azp_path.with_extension("bak");
                let _ = std::fs::copy(&azp_path, &backup);
            }
            if let Ok(true) = unset_azure_pipelines_agent_proxy() {
                println!(
                    "  {} Cleared broken Azure Pipelines Agent proxy in {} (backup saved)",
                    "✔".green(),
                    azp_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [82/125] Azure Pipelines Agent daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [82/125] Azure Pipelines Agent daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 83. Clean dead TeamCity Agent daemon proxy in buildAgent.properties / default / systemd
    if let Some((tc_proxy, tc_path)) = get_teamcity_agent_proxy() {
        if !verify_proxy_url(&tc_proxy).await {
            println!(
                "  [83/125] Found dead TeamCity Agent daemon proxy (`{}` in {}), cleaning with backup...",
                tc_proxy.yellow(),
                tc_path.display()
            );
            if tc_path.exists() {
                let backup = tc_path.with_extension("bak");
                let _ = std::fs::copy(&tc_path, &backup);
            }
            if let Ok(true) = unset_teamcity_agent_proxy() {
                println!(
                    "  {} Cleared broken TeamCity Agent proxy in {} (backup saved)",
                    "✔".green(),
                    tc_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [83/125] TeamCity Agent daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [83/125] TeamCity Agent daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 84. Clean dead Bamboo Remote Agent daemon proxy in bamboo-agent.cfg.xml / wrapper.conf / systemd
    if let Some((bam_proxy, bam_path)) = get_bamboo_agent_proxy() {
        if !verify_proxy_url(&bam_proxy).await {
            println!(
                "  [84/125] Found dead Bamboo Remote Agent daemon proxy (`{}` in {}), cleaning with backup...",
                bam_proxy.yellow(),
                bam_path.display()
            );
            if bam_path.exists() {
                let backup = bam_path.with_extension("bak");
                let _ = std::fs::copy(&bam_path, &backup);
            }
            if let Ok(true) = unset_bamboo_agent_proxy() {
                println!(
                    "  {} Cleared broken Bamboo Remote Agent proxy in {} (backup saved)",
                    "✔".green(),
                    bam_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [84/125] Bamboo Remote Agent daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [84/125] Bamboo Remote Agent daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 85. Clean dead Terraform Cloud / Enterprise Agent daemon proxy in agent.env / default / systemd
    if let Some((tfc_proxy, tfc_path)) = get_tfc_agent_proxy() {
        if !verify_proxy_url(&tfc_proxy).await {
            println!(
                "  [85/125] Found dead Terraform Cloud Agent daemon proxy (`{}` in {}), cleaning with backup...",
                tfc_proxy.yellow(),
                tfc_path.display()
            );
            if tfc_path.exists() {
                let backup = tfc_path.with_extension("bak");
                let _ = std::fs::copy(&tfc_path, &backup);
            }
            if let Ok(true) = unset_tfc_agent_proxy() {
                println!(
                    "  {} Cleared broken Terraform Cloud Agent proxy in {} (backup saved)",
                    "✔".green(),
                    tfc_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [85/125] Terraform Cloud Agent daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [85/125] Terraform Cloud Agent daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 86. Clean dead Concourse CI Worker daemon proxy in worker.env / default / systemd
    if let Some((con_proxy, con_path)) = get_concourse_worker_proxy() {
        if !verify_proxy_url(&con_proxy).await {
            println!(
                "  [86/125] Found dead Concourse CI Worker daemon proxy (`{}` in {}), cleaning with backup...",
                con_proxy.yellow(),
                con_path.display()
            );
            if con_path.exists() {
                let backup = con_path.with_extension("bak");
                let _ = std::fs::copy(&con_path, &backup);
            }
            if let Ok(true) = unset_concourse_worker_proxy() {
                println!(
                    "  {} Cleared broken Concourse CI Worker proxy in {} (backup saved)",
                    "✔".green(),
                    con_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [86/125] Concourse CI Worker daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [86/125] Concourse CI Worker daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 87. Clean dead Octopus Deploy Tentacle Agent CI/CD runner daemon proxy in tentacle.config / default / systemd
    if let Some((tentacle_proxy, tentacle_path)) = get_tentacle_agent_proxy() {
        if !verify_proxy_url(&tentacle_proxy).await {
            println!(
                "  [87/125] Found dead Octopus Deploy Tentacle Agent daemon proxy (`{}` in {}), cleaning with backup...",
                tentacle_proxy.yellow(),
                tentacle_path.display()
            );
            if tentacle_path.exists() {
                let backup = tentacle_path.with_extension("bak");
                let _ = std::fs::copy(&tentacle_path, &backup);
            }
            if let Ok(true) = unset_tentacle_agent_proxy() {
                println!(
                    "  {} Cleared broken Octopus Deploy Tentacle Agent proxy in {} (backup saved)",
                    "✔".green(),
                    tentacle_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [87/125] Octopus Deploy Tentacle Agent daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [87/125] Octopus Deploy Tentacle Agent daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 88. Clean dead Puppet Agent infrastructure daemon proxy in puppet.conf / default / systemd
    if let Some((puppet_proxy, puppet_path)) = get_puppet_agent_proxy() {
        if !verify_proxy_url(&puppet_proxy).await {
            println!(
                "  [88/125] Found dead Puppet Agent daemon proxy (`{}` in {}), cleaning with backup...",
                puppet_proxy.yellow(),
                puppet_path.display()
            );
            if puppet_path.exists() {
                let backup = puppet_path.with_extension("bak");
                let _ = std::fs::copy(&puppet_path, &backup);
            }
            if let Ok(true) = unset_puppet_agent_proxy() {
                println!(
                    "  {} Cleared broken Puppet Agent proxy in {} (backup saved)",
                    "✔".green(),
                    puppet_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [88/125] Puppet Agent daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [88/125] Puppet Agent daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 89. Clean dead Chef Infra Client daemon proxy in client.rb / default / systemd
    if let Some((chef_proxy, chef_path)) = get_chef_client_proxy() {
        if !verify_proxy_url(&chef_proxy).await {
            println!(
                "  [89/125] Found dead Chef Infra Client daemon proxy (`{}` in {}), cleaning with backup...",
                chef_proxy.yellow(),
                chef_path.display()
            );
            if chef_path.exists() {
                let backup = chef_path.with_extension("bak");
                let _ = std::fs::copy(&chef_path, &backup);
            }
            if let Ok(true) = unset_chef_client_proxy() {
                println!(
                    "  {} Cleared broken Chef Infra Client proxy in {} (backup saved)",
                    "✔".green(),
                    chef_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [89/125] Chef Infra Client daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [89/125] Chef Infra Client daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 90. Clean dead SaltStack Salt Minion infrastructure daemon proxy in minion / minion.d / default / systemd
    if let Some((salt_proxy, salt_path)) = get_salt_minion_proxy() {
        if !verify_proxy_url(&salt_proxy).await {
            println!(
                "  [90/125] Found dead SaltStack Salt Minion daemon proxy (`{}` in {}), cleaning with backup...",
                salt_proxy.yellow(),
                salt_path.display()
            );
            if salt_path.exists() {
                let backup = salt_path.with_extension("bak");
                let _ = std::fs::copy(&salt_path, &backup);
            }
            if let Ok(true) = unset_salt_minion_proxy() {
                println!(
                    "  {} Cleared broken SaltStack Salt Minion proxy in {} (backup saved)",
                    "✔".green(),
                    salt_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [90/125] SaltStack Salt Minion daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [90/125] SaltStack Salt Minion daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 91. Clean dead Datadog Agent infrastructure daemon proxy in datadog.yaml / default / systemd
    if let Some((dd_proxy, dd_path)) = get_datadog_agent_proxy() {
        if !verify_proxy_url(&dd_proxy).await {
            println!(
                "  [91/125] Found dead Datadog Agent daemon proxy (`{}` in {}), cleaning with backup...",
                dd_proxy.yellow(),
                dd_path.display()
            );
            if dd_path.exists() {
                let backup = dd_path.with_extension("bak");
                let _ = std::fs::copy(&dd_path, &backup);
            }
            if let Ok(true) = unset_datadog_agent_proxy() {
                println!(
                    "  {} Cleared broken Datadog Agent proxy in {} (backup saved)",
                    "✔".green(),
                    dd_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [91/125] Datadog Agent daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [91/125] Datadog Agent daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 92. Clean dead New Relic Infrastructure Agent daemon proxy in newrelic-infra.yml / default / systemd
    if let Some((nr_proxy, nr_path)) = get_newrelic_infra_proxy() {
        if !verify_proxy_url(&nr_proxy).await {
            println!(
                "  [92/125] Found dead New Relic Infrastructure Agent daemon proxy (`{}` in {}), cleaning with backup...",
                nr_proxy.yellow(),
                nr_path.display()
            );
            if nr_path.exists() {
                let backup = nr_path.with_extension("bak");
                let _ = std::fs::copy(&nr_path, &backup);
            }
            if let Ok(true) = unset_newrelic_infra_proxy() {
                println!(
                    "  {} Cleared broken New Relic Infrastructure Agent proxy in {} (backup saved)",
                    "✔".green(),
                    nr_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [92/125] New Relic Infrastructure Agent daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [92/125] New Relic Infrastructure Agent daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 93. Clean dead Elastic Agent daemon proxy in elastic-agent.yml / default / systemd
    if let Some((ea_proxy, ea_path)) = get_elastic_agent_proxy() {
        if !verify_proxy_url(&ea_proxy).await {
            println!(
                "  [93/125] Found dead Elastic Agent daemon proxy (`{}` in {}), cleaning with backup...",
                ea_proxy.yellow(),
                ea_path.display()
            );
            if ea_path.exists() {
                let backup = ea_path.with_extension("bak");
                let _ = std::fs::copy(&ea_path, &backup);
            }
            if let Ok(true) = unset_elastic_agent_proxy() {
                println!(
                    "  {} Cleared broken Elastic Agent proxy in {} (backup saved)",
                    "✔".green(),
                    ea_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [93/125] Elastic Agent daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [93/125] Elastic Agent daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 94. Clean dead Grafana Alloy telemetry daemon proxy in config.alloy / default / systemd
    if let Some((alloy_proxy, alloy_path)) = get_grafana_alloy_proxy() {
        if !verify_proxy_url(&alloy_proxy).await {
            println!(
                "  [94/125] Found dead Grafana Alloy telemetry proxy (`{}` in {}), cleaning with backup...",
                alloy_proxy.yellow(),
                alloy_path.display()
            );
            if alloy_path.exists() {
                let backup = alloy_path.with_extension("bak");
                let _ = std::fs::copy(&alloy_path, &backup);
            }
            if let Ok(true) = unset_grafana_alloy_proxy() {
                println!(
                    "  {} Cleared broken Grafana Alloy telemetry proxy in {} (backup saved)",
                    "✔".green(),
                    alloy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [94/125] Grafana Alloy telemetry proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [94/125] Grafana Alloy telemetry proxy: {}", "Clean or not set".dimmed());
    }

    // 95. Clean dead OpenTelemetry Collector telemetry daemon proxy in config.yaml / default / systemd
    if let Some((otel_proxy, otel_path)) = get_otelcol_proxy() {
        if !verify_proxy_url(&otel_proxy).await {
            println!(
                "  [95/125] Found dead OpenTelemetry Collector daemon proxy (`{}` in {}), cleaning with backup...",
                otel_proxy.yellow(),
                otel_path.display()
            );
            if otel_path.exists() {
                let backup = otel_path.with_extension("bak");
                let _ = std::fs::copy(&otel_path, &backup);
            }
            if let Ok(true) = unset_otelcol_proxy() {
                println!(
                    "  {} Cleared broken OpenTelemetry Collector proxy in {} (backup saved)",
                    "✔".green(),
                    otel_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [95/125] OpenTelemetry Collector daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [95/125] OpenTelemetry Collector daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 96. Clean dead Vector pipeline daemon proxy in vector.yaml / default / systemd
    if let Some((vector_proxy, vector_path)) = get_vector_proxy() {
        if !verify_proxy_url(&vector_proxy).await {
            println!(
                "  [96/125] Found dead Vector pipeline daemon proxy (`{}` in {}), cleaning with backup...",
                vector_proxy.yellow(),
                vector_path.display()
            );
            if vector_path.exists() {
                let backup = vector_path.with_extension("bak");
                let _ = std::fs::copy(&vector_path, &backup);
            }
            if let Ok(true) = unset_vector_proxy() {
                println!(
                    "  {} Cleared broken Vector pipeline proxy in {} (backup saved)",
                    "✔".green(),
                    vector_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [96/125] Vector pipeline daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [96/125] Vector pipeline daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 97. Clean dead Fluent Bit telemetry daemon proxy in fluent-bit.conf / default / systemd
    if let Some((flb_proxy, flb_path)) = get_fluent_bit_proxy() {
        if !verify_proxy_url(&flb_proxy).await {
            println!(
                "  [97/125] Found dead Fluent Bit daemon proxy (`{}` in {}), cleaning with backup...",
                flb_proxy.yellow(),
                flb_path.display()
            );
            if flb_path.exists() {
                let backup = flb_path.with_extension("bak");
                let _ = std::fs::copy(&flb_path, &backup);
            }
            if let Ok(true) = unset_fluent_bit_proxy() {
                println!(
                    "  {} Cleared broken Fluent Bit proxy in {} (backup saved)",
                    "✔".green(),
                    flb_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [97/125] Fluent Bit telemetry proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [97/125] Fluent Bit telemetry proxy: {}", "Clean or not set".dimmed());
    }

    // 98. Clean dead Fluentd logging framework proxy in fluent.conf / default / systemd
    if let Some((fluentd_proxy, fluentd_path)) = get_fluentd_proxy() {
        if !verify_proxy_url(&fluentd_proxy).await {
            println!(
                "  [98/125] Found dead Fluentd daemon proxy (`{}` in {}), cleaning with backup...",
                fluentd_proxy.yellow(),
                fluentd_path.display()
            );
            if fluentd_path.exists() {
                let backup = fluentd_path.with_extension("bak");
                let _ = std::fs::copy(&fluentd_path, &backup);
            }
            if let Ok(true) = unset_fluentd_proxy() {
                println!(
                    "  {} Cleared broken Fluentd proxy in {} (backup saved)",
                    "✔".green(),
                    fluentd_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [98/125] Fluentd logging proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [98/125] Fluentd logging proxy: {}", "Clean or not set".dimmed());
    }

    // 99. Clean dead Telegraf metrics agent proxy in telegraf.conf / default / systemd
    if let Some((telegraf_proxy, telegraf_path)) = get_telegraf_proxy() {
        if !verify_proxy_url(&telegraf_proxy).await {
            println!(
                "  [99/125] Found dead Telegraf metrics daemon proxy (`{}` in {}), cleaning with backup...",
                telegraf_proxy.yellow(),
                telegraf_path.display()
            );
            if telegraf_path.exists() {
                let backup = telegraf_path.with_extension("bak");
                let _ = std::fs::copy(&telegraf_path, &backup);
            }
            if let Ok(true) = unset_telegraf_proxy() {
                println!(
                    "  {} Cleared broken Telegraf proxy in {} (backup saved)",
                    "✔".green(),
                    telegraf_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [99/125] Telegraf metrics proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [99/125] Telegraf metrics proxy: {}", "Clean or not set".dimmed());
    }

    // 100. Clean dead Logstash pipeline ingestion proxy in logstash.yml / jvm.options / default / systemd
    if let Some((logstash_proxy, logstash_path)) = get_logstash_proxy() {
        if !verify_proxy_url(&logstash_proxy).await {
            println!(
                "  [100/125] Found dead Logstash pipeline daemon proxy (`{}` in {}), cleaning with backup...",
                logstash_proxy.yellow(),
                logstash_path.display()
            );
            if logstash_path.exists() {
                let backup = logstash_path.with_extension("bak");
                let _ = std::fs::copy(&logstash_path, &backup);
            }
            if let Ok(true) = unset_logstash_proxy() {
                println!(
                    "  {} Cleared broken Logstash proxy in {} (backup saved)",
                    "✔".green(),
                    logstash_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [100/125] Logstash pipeline proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [100/125] Logstash pipeline proxy: {}", "Clean or not set".dimmed());
    }

    // 101. Clean dead Prometheus monitoring daemon proxy in prometheus.yml / default / systemd
    if let Some((prometheus_proxy, prometheus_path)) = get_prometheus_proxy() {
        if !verify_proxy_url(&prometheus_proxy).await {
            println!(
                "  [101/125] Found dead Prometheus monitoring daemon proxy (`{}` in {}), cleaning with backup...",
                prometheus_proxy.yellow(),
                prometheus_path.display()
            );
            if prometheus_path.exists() {
                let backup = prometheus_path.with_extension("bak");
                let _ = std::fs::copy(&prometheus_path, &backup);
            }
            if let Ok(true) = unset_prometheus_proxy() {
                println!(
                    "  {} Cleared broken Prometheus proxy in {} (backup saved)",
                    "✔".green(),
                    prometheus_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [101/125] Prometheus monitoring proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [101/125] Prometheus monitoring proxy: {}", "Clean or not set".dimmed());
    }

    // 102. Clean dead Prometheus Alertmanager alert routing proxy in alertmanager.yml / default / systemd
    if let Some((alertmanager_proxy, alertmanager_path)) = get_alertmanager_proxy() {
        if !verify_proxy_url(&alertmanager_proxy).await {
            println!(
                "  [102/125] Found dead Alertmanager notification daemon proxy (`{}` in {}), cleaning with backup...",
                alertmanager_proxy.yellow(),
                alertmanager_path.display()
            );
            if alertmanager_path.exists() {
                let backup = alertmanager_path.with_extension("bak");
                let _ = std::fs::copy(&alertmanager_path, &backup);
            }
            if let Ok(true) = unset_alertmanager_proxy() {
                println!(
                    "  {} Cleared broken Alertmanager proxy in {} (backup saved)",
                    "✔".green(),
                    alertmanager_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [102/125] Alertmanager notification proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [102/125] Alertmanager notification proxy: {}", "Clean or not set".dimmed());
    }

    // 103. Clean dead Promtail log collector daemon proxy in promtail.yml / config.yml / default / systemd
    if let Some((promtail_proxy, promtail_path)) = get_promtail_proxy() {
        if !verify_proxy_url(&promtail_proxy).await {
            println!(
                "  [103/125] Found dead Promtail log collector daemon proxy (`{}` in {}), cleaning with backup...",
                promtail_proxy.yellow(),
                promtail_path.display()
            );
            if promtail_path.exists() {
                let backup = promtail_path.with_extension("bak");
                let _ = std::fs::copy(&promtail_path, &backup);
            }
            if let Ok(true) = unset_promtail_proxy() {
                println!(
                    "  {} Cleared broken Promtail proxy in {} (backup saved)",
                    "✔".green(),
                    promtail_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [103/125] Promtail log collector proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [103/125] Promtail log collector proxy: {}", "Clean or not set".dimmed());
    }

    // 104. Clean dead VictoriaMetrics Agent (vmagent) daemon proxy in vmagent.yml / default / systemd
    if let Some((vmagent_proxy, vmagent_path)) = get_vmagent_proxy() {
        if !verify_proxy_url(&vmagent_proxy).await {
            println!(
                "  [104/125] Found dead VictoriaMetrics Agent daemon proxy (`{}` in {}), cleaning with backup...",
                vmagent_proxy.yellow(),
                vmagent_path.display()
            );
            if vmagent_path.exists() {
                let backup = vmagent_path.with_extension("bak");
                let _ = std::fs::copy(&vmagent_path, &backup);
            }
            if let Ok(true) = unset_vmagent_proxy() {
                println!(
                    "  {} Cleared broken VictoriaMetrics Agent proxy in {} (backup saved)",
                    "✔".green(),
                    vmagent_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [104/125] VictoriaMetrics Agent proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [104/125] VictoriaMetrics Agent proxy: {}", "Clean or not set".dimmed());
    }

    // 105. Clean dead Filebeat log shipper daemon proxy in filebeat.yml / default / systemd
    if let Some((filebeat_proxy, filebeat_path)) = get_filebeat_proxy() {
        if !verify_proxy_url(&filebeat_proxy).await {
            println!(
                "  [105/125] Found dead Filebeat log shipper daemon proxy (`{}` in {}), cleaning with backup...",
                filebeat_proxy.yellow(),
                filebeat_path.display()
            );
            if filebeat_path.exists() {
                let backup = filebeat_path.with_extension("bak");
                let _ = std::fs::copy(&filebeat_path, &backup);
            }
            if let Ok(true) = unset_filebeat_proxy() {
                println!(
                    "  {} Cleared broken Filebeat proxy in {} (backup saved)",
                    "✔".green(),
                    filebeat_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [105/125] Filebeat log shipper proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [105/125] Filebeat log shipper proxy: {}", "Clean or not set".dimmed());
    }

    // 106. Clean dead Metricbeat metrics shipper daemon proxy in metricbeat.yml / default / systemd
    if let Some((metricbeat_proxy, metricbeat_path)) = get_metricbeat_proxy() {
        if !verify_proxy_url(&metricbeat_proxy).await {
            println!(
                "  [106/125] Found dead Metricbeat metrics shipper daemon proxy (`{}` in {}), cleaning with backup...",
                metricbeat_proxy.yellow(),
                metricbeat_path.display()
            );
            if metricbeat_path.exists() {
                let backup = metricbeat_path.with_extension("bak");
                let _ = std::fs::copy(&metricbeat_path, &backup);
            }
            if let Ok(true) = unset_metricbeat_proxy() {
                println!(
                    "  {} Cleared broken Metricbeat proxy in {} (backup saved)",
                    "✔".green(),
                    metricbeat_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [106/125] Metricbeat metrics shipper proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [106/125] Metricbeat metrics shipper proxy: {}", "Clean or not set".dimmed());
    }

    // 107. Clean dead Heartbeat uptime monitor daemon proxy in heartbeat.yml / default / systemd
    if let Some((heartbeat_proxy, heartbeat_path)) = get_heartbeat_proxy() {
        if !verify_proxy_url(&heartbeat_proxy).await {
            println!(
                "  [107/125] Found dead Heartbeat uptime monitor daemon proxy (`{}` in {}), cleaning with backup...",
                heartbeat_proxy.yellow(),
                heartbeat_path.display()
            );
            if heartbeat_path.exists() {
                let backup = heartbeat_path.with_extension("bak");
                let _ = std::fs::copy(&heartbeat_path, &backup);
            }
            if let Ok(true) = unset_heartbeat_proxy() {
                println!(
                    "  {} Cleared broken Heartbeat proxy in {} (backup saved)",
                    "✔".green(),
                    heartbeat_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [107/125] Heartbeat uptime monitor proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [107/125] Heartbeat uptime monitor proxy: {}", "Clean or not set".dimmed());
    }

    // 108. Clean dead Auditbeat security auditor daemon proxy in auditbeat.yml / default / systemd
    if let Some((auditbeat_proxy, auditbeat_path)) = get_auditbeat_proxy() {
        if !verify_proxy_url(&auditbeat_proxy).await {
            println!(
                "  [108/125] Found dead Auditbeat security auditor daemon proxy (`{}` in {}), cleaning with backup...",
                auditbeat_proxy.yellow(),
                auditbeat_path.display()
            );
            if auditbeat_path.exists() {
                let backup = auditbeat_path.with_extension("bak");
                let _ = std::fs::copy(&auditbeat_path, &backup);
            }
            if let Ok(true) = unset_auditbeat_proxy() {
                println!(
                    "  {} Cleared broken Auditbeat proxy in {} (backup saved)",
                    "✔".green(),
                    auditbeat_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [108/125] Auditbeat security auditor proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [108/125] Auditbeat security auditor proxy: {}", "Clean or not set".dimmed());
    }

    // 109. Clean dead Packetbeat network packet analyzer daemon proxy in packetbeat.yml / default / systemd
    if let Some((packetbeat_proxy, packetbeat_path)) = get_packetbeat_proxy() {
        if !verify_proxy_url(&packetbeat_proxy).await {
            println!(
                "  [109/125] Found dead Packetbeat network packet analyzer daemon proxy (`{}` in {}), cleaning with backup...",
                packetbeat_proxy.yellow(),
                packetbeat_path.display()
            );
            if packetbeat_path.exists() {
                let backup = packetbeat_path.with_extension("bak");
                let _ = std::fs::copy(&packetbeat_path, &backup);
            }
            if let Ok(true) = unset_packetbeat_proxy() {
                println!(
                    "  {} Cleared broken Packetbeat proxy in {} (backup saved)",
                    "✔".green(),
                    packetbeat_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [109/125] Packetbeat network analyzer proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [109/125] Packetbeat network analyzer proxy: {}", "Clean or not set".dimmed());
    }

    // 110. Clean dead Winlogbeat event log shipper daemon proxy in winlogbeat.yml / default / systemd
    if let Some((winlogbeat_proxy, winlogbeat_path)) = get_winlogbeat_proxy() {
        if !verify_proxy_url(&winlogbeat_proxy).await {
            println!(
                "  [110/125] Found dead Winlogbeat event log shipper daemon proxy (`{}` in {}), cleaning with backup...",
                winlogbeat_proxy.yellow(),
                winlogbeat_path.display()
            );
            if winlogbeat_path.exists() {
                let backup = winlogbeat_path.with_extension("bak");
                let _ = std::fs::copy(&winlogbeat_path, &backup);
            }
            if let Ok(true) = unset_winlogbeat_proxy() {
                println!(
                    "  {} Cleared broken Winlogbeat proxy in {} (backup saved)",
                    "✔".green(),
                    winlogbeat_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [110/125] Winlogbeat event shipper proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [110/125] Winlogbeat event shipper proxy: {}", "Clean or not set".dimmed());
    }

    // 111. Clean dead HashiCorp Consul service mesh daemon proxy in consul.hcl / default / systemd
    if let Some((consul_proxy, consul_path)) = get_consul_proxy() {
        if !verify_proxy_url(&consul_proxy).await {
            println!(
                "  [111/125] Found dead Consul service mesh daemon proxy (`{}` in {}), cleaning with backup...",
                consul_proxy.yellow(),
                consul_path.display()
            );
            if consul_path.exists() {
                let backup = consul_path.with_extension("bak");
                let _ = std::fs::copy(&consul_path, &backup);
            }
            if let Ok(true) = unset_consul_proxy() {
                println!(
                    "  {} Cleared broken Consul proxy in {} (backup saved)",
                    "✔".green(),
                    consul_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [111/125] Consul service mesh daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [111/125] Consul service mesh daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 112. Clean dead HashiCorp Vault secret management daemon proxy in vault.hcl / default / systemd
    if let Some((vault_proxy, vault_path)) = get_vault_proxy() {
        if !verify_proxy_url(&vault_proxy).await {
            println!(
                "  [112/125] Found dead Vault secret management daemon proxy (`{}` in {}), cleaning with backup...",
                vault_proxy.yellow(),
                vault_path.display()
            );
            if vault_path.exists() {
                let backup = vault_path.with_extension("bak");
                let _ = std::fs::copy(&vault_path, &backup);
            }
            if let Ok(true) = unset_vault_proxy() {
                println!(
                    "  {} Cleared broken Vault proxy in {} (backup saved)",
                    "✔".green(),
                    vault_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [112/125] Vault secret management daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [112/125] Vault secret management daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 113. Clean dead Traefik edge router daemon proxy in traefik.yml / default / systemd
    if let Some((traefik_proxy, traefik_path)) = get_traefik_proxy() {
        if !verify_proxy_url(&traefik_proxy).await {
            println!(
                "  [113/125] Found dead Traefik edge router daemon proxy (`{}` in {}), cleaning with backup...",
                traefik_proxy.yellow(),
                traefik_path.display()
            );
            if traefik_path.exists() {
                let backup = traefik_path.with_extension("bak");
                let _ = std::fs::copy(&traefik_path, &backup);
            }
            if let Ok(true) = unset_traefik_proxy() {
                println!(
                    "  {} Cleared broken Traefik proxy in {} (backup saved)",
                    "✔".green(),
                    traefik_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [113/125] Traefik edge router daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [113/125] Traefik edge router daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 114. Clean dead Caddy enterprise web server daemon proxy in Caddyfile / default / systemd
    if let Some((caddy_proxy, caddy_path)) = get_caddy_proxy() {
        if !verify_proxy_url(&caddy_proxy).await {
            println!(
                "  [114/125] Found dead Caddy web server daemon proxy (`{}` in {}), cleaning with backup...",
                caddy_proxy.yellow(),
                caddy_path.display()
            );
            if caddy_path.exists() {
                let backup = caddy_path.with_extension("bak");
                let _ = std::fs::copy(&caddy_path, &backup);
            }
            if let Ok(true) = unset_caddy_proxy() {
                println!(
                    "  {} Cleared broken Caddy proxy in {} (backup saved)",
                    "✔".green(),
                    caddy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [114/125] Caddy web server daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [114/125] Caddy web server daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 115. Clean dead Envoy cloud-native service proxy in envoy.yaml / default / systemd
    if let Some((envoy_proxy, envoy_path)) = get_envoy_proxy() {
        if !verify_proxy_url(&envoy_proxy).await {
            println!(
                "  [115/125] Found dead Envoy service proxy daemon proxy (`{}` in {}), cleaning with backup...",
                envoy_proxy.yellow(),
                envoy_path.display()
            );
            if envoy_path.exists() {
                let backup = envoy_path.with_extension("bak");
                let _ = std::fs::copy(&envoy_path, &backup);
            }
            if let Ok(true) = unset_envoy_proxy() {
                println!(
                    "  {} Cleared broken Envoy proxy in {} (backup saved)",
                    "✔".green(),
                    envoy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [115/125] Envoy service proxy daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [115/125] Envoy service proxy daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 116. Clean dead HAProxy load balancer daemon proxy in haproxy.cfg / default / systemd
    if let Some((haproxy_proxy, haproxy_path)) = get_haproxy_proxy() {
        if !verify_proxy_url(&haproxy_proxy).await {
            println!(
                "  [116/125] Found dead HAProxy load balancer daemon proxy (`{}` in {}), cleaning with backup...",
                haproxy_proxy.yellow(),
                haproxy_path.display()
            );
            if haproxy_path.exists() {
                let backup = haproxy_path.with_extension("bak");
                let _ = std::fs::copy(&haproxy_path, &backup);
            }
            if let Ok(true) = unset_haproxy_proxy() {
                println!(
                    "  {} Cleared broken HAProxy proxy in {} (backup saved)",
                    "✔".green(),
                    haproxy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [116/125] HAProxy load balancer daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [116/125] HAProxy load balancer daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 117. Clean dead NGINX web server & reverse proxy daemon proxy in nginx.conf / conf.d / default / systemd
    if let Some((nginx_proxy, nginx_path)) = get_nginx_proxy() {
        if !verify_proxy_url(&nginx_proxy).await {
            println!(
                "  [117/125] Found dead NGINX web server daemon proxy (`{}` in {}), cleaning with backup...",
                nginx_proxy.yellow(),
                nginx_path.display()
            );
            if nginx_path.exists() {
                let backup = nginx_path.with_extension("bak");
                let _ = std::fs::copy(&nginx_path, &backup);
            }
            if let Ok(true) = unset_nginx_proxy() {
                println!(
                    "  {} Cleared broken NGINX proxy in {} (backup saved)",
                    "✔".green(),
                    nginx_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [117/125] NGINX web server daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [117/125] NGINX web server daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 118. Clean dead Kong API gateway daemon proxy in kong.conf / default / systemd
    if let Some((kong_proxy, kong_path)) = get_kong_proxy() {
        if !verify_proxy_url(&kong_proxy).await {
            println!(
                "  [118/125] Found dead Kong API gateway daemon proxy (`{}` in {}), cleaning with backup...",
                kong_proxy.yellow(),
                kong_path.display()
            );
            if kong_path.exists() {
                let backup = kong_path.with_extension("bak");
                let _ = std::fs::copy(&kong_path, &backup);
            }
            if let Ok(true) = unset_kong_proxy() {
                println!(
                    "  {} Cleared broken Kong proxy in {} (backup saved)",
                    "✔".green(),
                    kong_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [118/125] Kong API gateway daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [118/125] Kong API gateway daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 119. Clean dead Apache APISIX API gateway daemon proxy in config.yaml / default / systemd
    if let Some((apisix_proxy, apisix_path)) = get_apisix_proxy() {
        if !verify_proxy_url(&apisix_proxy).await {
            println!(
                "  [119/125] Found dead Apache APISIX API gateway daemon proxy (`{}` in {}), cleaning with backup...",
                apisix_proxy.yellow(),
                apisix_path.display()
            );
            if apisix_path.exists() {
                let backup = apisix_path.with_extension("bak");
                let _ = std::fs::copy(&apisix_path, &backup);
            }
            if let Ok(true) = unset_apisix_proxy() {
                println!(
                    "  {} Cleared broken Apache APISIX proxy in {} (backup saved)",
                    "✔".green(),
                    apisix_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [119/125] Apache APISIX gateway daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [119/125] Apache APISIX gateway daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 120. Clean dead Squid caching proxy daemon in squid.conf / conf.d / default / systemd
    if let Some((squid_proxy, squid_path)) = get_squid_proxy() {
        if !verify_proxy_url(&squid_proxy).await {
            println!(
                "  [120/127] Found dead Squid caching proxy daemon (`{}` in {}), cleaning with backup...",
                squid_proxy.yellow(),
                squid_path.display()
            );
            if squid_path.exists() {
                let backup = squid_path.with_extension("bak");
                let _ = std::fs::copy(&squid_path, &backup);
            }
            if let Ok(true) = unset_squid_proxy() {
                println!(
                    "  {} Cleared broken Squid proxy in {} (backup saved)",
                    "✔".green(),
                    squid_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [120/127] Squid caching proxy daemon: {}", "Reachable".green());
        }
    } else {
        println!("  [120/127] Squid caching proxy daemon: {}", "Clean or not set".dimmed());
    }

    // 121. Clean dead Privoxy forwarding proxy daemon in config / default / systemd
    if let Some((privoxy_proxy, privoxy_path)) = get_privoxy_proxy() {
        if !verify_proxy_url(&privoxy_proxy).await {
            println!(
                "  [121/133] Found dead Privoxy forwarding proxy daemon (`{}` in {}), cleaning with backup...",
                privoxy_proxy.yellow(),
                privoxy_path.display()
            );
            if privoxy_path.exists() {
                let backup = privoxy_path.with_extension("bak");
                let _ = std::fs::copy(&privoxy_path, &backup);
            }
            if let Ok(true) = unset_privoxy_proxy() {
                println!(
                    "  {} Cleared broken Privoxy proxy in {} (backup saved)",
                    "✔".green(),
                    privoxy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [121/133] Privoxy forwarding proxy daemon: {}", "Reachable".green());
        }
    } else {
        println!("  [121/133] Privoxy forwarding proxy daemon: {}", "Clean or not set".dimmed());
    }

    // 122. Clean dead Tinyproxy HTTP proxy daemon in tinyproxy.conf / default / systemd
    if let Some((tinyproxy_proxy, tinyproxy_path)) = get_tinyproxy_proxy() {
        if !verify_proxy_url(&tinyproxy_proxy).await {
            println!(
                "  [122/133] Found dead Tinyproxy HTTP proxy daemon (`{}` in {}), cleaning with backup...",
                tinyproxy_proxy.yellow(),
                tinyproxy_path.display()
            );
            if tinyproxy_path.exists() {
                let backup = tinyproxy_path.with_extension("bak");
                let _ = std::fs::copy(&tinyproxy_path, &backup);
            }
            if let Ok(true) = unset_tinyproxy_proxy() {
                println!(
                    "  {} Cleared broken Tinyproxy proxy in {} (backup saved)",
                    "✔".green(),
                    tinyproxy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [122/133] Tinyproxy HTTP proxy daemon: {}", "Reachable".green());
        }
    } else {
        println!("  [122/133] Tinyproxy HTTP proxy daemon: {}", "Clean or not set".dimmed());
    }

    // 123. Clean dead Mitmproxy HTTPS intercepting proxy daemon in config.yaml / default / systemd
    if let Some((mitmproxy_proxy, mitmproxy_path)) = get_mitmproxy_proxy() {
        if !verify_proxy_url(&mitmproxy_proxy).await {
            println!(
                "  [123/133] Found dead Mitmproxy HTTPS intercepting proxy daemon (`{}` in {}), cleaning with backup...",
                mitmproxy_proxy.yellow(),
                mitmproxy_path.display()
            );
            if mitmproxy_path.exists() {
                let backup = mitmproxy_path.with_extension("bak");
                let _ = std::fs::copy(&mitmproxy_path, &backup);
            }
            if let Ok(true) = unset_mitmproxy_proxy() {
                println!(
                    "  {} Cleared broken Mitmproxy proxy in {} (backup saved)",
                    "✔".green(),
                    mitmproxy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [123/133] Mitmproxy HTTPS intercepting proxy daemon: {}", "Reachable".green());
        }
    } else {
        println!("  [123/133] Mitmproxy HTTPS intercepting proxy daemon: {}", "Clean or not set".dimmed());
    }

    // 124. Clean dead Tor anonymity network proxy daemon in torrc / default / systemd
    if let Some((tor_proxy, tor_path)) = get_tor_proxy() {
        if !verify_proxy_url(&tor_proxy).await {
            println!(
                "  [124/133] Found dead Tor anonymity network proxy daemon (`{}` in {}), cleaning with backup...",
                tor_proxy.yellow(),
                tor_path.display()
            );
            if tor_path.exists() {
                let backup = tor_path.with_extension("bak");
                let _ = std::fs::copy(&tor_path, &backup);
            }
            if let Ok(true) = unset_tor_proxy() {
                println!(
                    "  {} Cleared broken Tor proxy in {} (backup saved)",
                    "✔".green(),
                    tor_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [124/133] Tor anonymity network proxy daemon: {}", "Reachable".green());
        }
    } else {
        println!("  [124/133] Tor anonymity network proxy daemon: {}", "Clean or not set".dimmed());
    }

    // 125. Clean dead Cloudflare Tunnel daemon proxy in config.yml / default / systemd
    if let Some((cf_proxy, cf_path)) = get_cloudflared_proxy() {
        if !verify_proxy_url(&cf_proxy).await {
            println!(
                "  [125/133] Found dead Cloudflare Tunnel daemon proxy (`{}` in {}), cleaning with backup...",
                cf_proxy.yellow(),
                cf_path.display()
            );
            if cf_path.exists() {
                let backup = cf_path.with_extension("bak");
                let _ = std::fs::copy(&cf_path, &backup);
            }
            if let Ok(true) = unset_cloudflared_proxy() {
                println!(
                    "  {} Cleared broken Cloudflare Tunnel proxy in {} (backup saved)",
                    "✔".green(),
                    cf_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [125/133] Cloudflare Tunnel daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [125/133] Cloudflare Tunnel daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 126. Clean dead Ngrok tunnel daemon proxy in ngrok.yml / default / systemd
    if let Some((ngrok_proxy, ngrok_path)) = get_ngrok_proxy() {
        if !verify_proxy_url(&ngrok_proxy).await {
            println!(
                "  [126/133] Found dead Ngrok tunnel daemon proxy (`{}` in {}), cleaning with backup...",
                ngrok_proxy.yellow(),
                ngrok_path.display()
            );
            if ngrok_path.exists() {
                let backup = ngrok_path.with_extension("bak");
                let _ = std::fs::copy(&ngrok_path, &backup);
            }
            if let Ok(true) = unset_ngrok_proxy() {
                println!(
                    "  {} Cleared broken Ngrok proxy in {} (backup saved)",
                    "✔".green(),
                    ngrok_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [126/133] Ngrok tunnel daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [126/133] Ngrok tunnel daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 127. Clean dead FRP client daemon proxy in frpc.toml / default / systemd
    if let Some((frp_proxy, frp_path)) = get_frpc_proxy() {
        if !verify_proxy_url(&frp_proxy).await {
            println!(
                "  [127/141] Found dead FRP client daemon proxy (`{}` in {}), cleaning with backup...",
                frp_proxy.yellow(),
                frp_path.display()
            );
            if frp_path.exists() {
                let backup = frp_path.with_extension("bak");
                let _ = std::fs::copy(&frp_path, &backup);
            }
            if let Ok(true) = unset_frpc_proxy() {
                println!(
                    "  {} Cleared broken FRP client proxy in {} (backup saved)",
                    "✔".green(),
                    frp_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [127/141] FRP client daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [127/141] FRP client daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 128. Clean dead Tailscale daemon proxy in /etc/default/tailscaled / systemd
    if let Some((ts_proxy, ts_path)) = get_tailscale_proxy() {
        if !verify_proxy_url(&ts_proxy).await {
            println!(
                "  [128/141] Found dead Tailscale daemon proxy (`{}` in {}), cleaning with backup...",
                ts_proxy.yellow(),
                ts_path.display()
            );
            if ts_path.exists() {
                let backup = ts_path.with_extension("bak");
                let _ = std::fs::copy(&ts_path, &backup);
            }
            if let Ok(true) = unset_tailscale_proxy() {
                println!(
                    "  {} Cleared broken Tailscale proxy in {} (backup saved)",
                    "✔".green(),
                    ts_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [128/141] Tailscale daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [128/141] Tailscale daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 129. Clean dead ZeroTier One daemon proxy in local.conf / default / systemd
    if let Some((zt_proxy, zt_path)) = get_zerotier_proxy() {
        if !verify_proxy_url(&zt_proxy).await {
            println!(
                "  [129/141] Found dead ZeroTier daemon proxy (`{}` in {}), cleaning with backup...",
                zt_proxy.yellow(),
                zt_path.display()
            );
            if zt_path.exists() {
                let backup = zt_path.with_extension("bak");
                let _ = std::fs::copy(&zt_path, &backup);
            }
            if let Ok(true) = unset_zerotier_proxy() {
                println!(
                    "  {} Cleared broken ZeroTier proxy in {} (backup saved)",
                    "✔".green(),
                    zt_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [129/141] ZeroTier daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [129/141] ZeroTier daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 130. Clean dead Gost (GO Simple Tunnel) daemon proxy in gost.json / config.json / systemd
    if let Some((gost_proxy, gost_path)) = get_gost_proxy() {
        if !verify_proxy_url(&gost_proxy).await {
            println!(
                "  [130/141] Found dead Gost daemon proxy (`{}` in {}), cleaning with backup...",
                gost_proxy.yellow(),
                gost_path.display()
            );
            if gost_path.exists() {
                let backup = gost_path.with_extension("bak");
                let _ = std::fs::copy(&gost_path, &backup);
            }
            if let Ok(true) = unset_gost_proxy() {
                println!(
                    "  {} Cleared broken Gost proxy in {} (backup saved)",
                    "✔".green(),
                    gost_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [130/141] Gost daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [130/141] Gost daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 131. Clean dead Rathole NAT traversal daemon proxy in rathole.toml / client.toml / systemd
    if let Some((rathole_proxy, rathole_path)) = get_rathole_proxy() {
        if !verify_proxy_url(&rathole_proxy).await {
            println!(
                "  [131/141] Found dead Rathole daemon proxy (`{}` in {}), cleaning with backup...",
                rathole_proxy.yellow(),
                rathole_path.display()
            );
            if rathole_path.exists() {
                let backup = rathole_path.with_extension("bak");
                let _ = std::fs::copy(&rathole_path, &backup);
            }
            if let Ok(true) = unset_rathole_proxy() {
                println!(
                    "  {} Cleared broken Rathole proxy in {} (backup saved)",
                    "✔".green(),
                    rathole_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [131/141] Rathole daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [131/141] Rathole daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 132. Clean dead Chisel HTTP tunnel daemon proxy in config.json / chisel.json / systemd
    if let Some((chisel_proxy, chisel_path)) = get_chisel_proxy() {
        if !verify_proxy_url(&chisel_proxy).await {
            println!(
                "  [132/141] Found dead Chisel daemon proxy (`{}` in {}), cleaning with backup...",
                chisel_proxy.yellow(),
                chisel_path.display()
            );
            if chisel_path.exists() {
                let backup = chisel_path.with_extension("bak");
                let _ = std::fs::copy(&chisel_path, &backup);
            }
            if let Ok(true) = unset_chisel_proxy() {
                println!(
                    "  {} Cleared broken Chisel proxy in {} (backup saved)",
                    "✔".green(),
                    chisel_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [132/141] Chisel daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [132/141] Chisel daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 133. Clean dead Nebula mesh overlay daemon proxy in config.yaml / config.yml / systemd
    if let Some((nebula_proxy, nebula_path)) = get_nebula_proxy() {
        if !verify_proxy_url(&nebula_proxy).await {
            println!(
                "  [133/141] Found dead Nebula daemon proxy (`{}` in {}), cleaning with backup...",
                nebula_proxy.yellow(),
                nebula_path.display()
            );
            if nebula_path.exists() {
                let backup = nebula_path.with_extension("bak");
                let _ = std::fs::copy(&nebula_path, &backup);
            }
            if let Ok(true) = unset_nebula_proxy() {
                println!(
                    "  {} Cleared broken Nebula proxy in {} (backup saved)",
                    "✔".green(),
                    nebula_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [133/141] Nebula daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [133/141] Nebula daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 134. Clean dead Bore modern tunnel daemon proxy in bore.toml / config.toml / systemd
    if let Some((bore_proxy, bore_path)) = get_bore_proxy() {
        if !verify_proxy_url(&bore_proxy).await {
            println!(
                "  [134/141] Found dead Bore daemon proxy (`{}` in {}), cleaning with backup...",
                bore_proxy.yellow(),
                bore_path.display()
            );
            if bore_path.exists() {
                let backup = bore_path.with_extension("bak");
                let _ = std::fs::copy(&bore_path, &backup);
            }
            if let Ok(true) = unset_bore_proxy() {
                println!(
                    "  {} Cleared broken Bore proxy in {} (backup saved)",
                    "✔".green(),
                    bore_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [134/141] Bore daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [134/141] Bore daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 135. Clean dead Brook proxy daemon proxy in config.json / brook.conf / systemd
    if let Some((brook_proxy, brook_path)) = get_brook_proxy() {
        if !verify_proxy_url(&brook_proxy).await {
            println!(
                "  [135/145] Found dead Brook daemon proxy (`{}` in {}), cleaning with backup...",
                brook_proxy.yellow(),
                brook_path.display()
            );
            if brook_path.exists() {
                let backup = brook_path.with_extension("bak");
                let _ = std::fs::copy(&brook_path, &backup);
            }
            if let Ok(true) = unset_brook_proxy() {
                println!(
                    "  {} Cleared broken Brook proxy in {} (backup saved)",
                    "✔".green(),
                    brook_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [135/145] Brook daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [135/145] Brook daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 136. Clean dead NPC (NPS Client) daemon proxy in npc.conf / conf/npc.conf / systemd
    if let Some((npc_proxy, npc_path)) = get_npc_proxy() {
        if !verify_proxy_url(&npc_proxy).await {
            println!(
                "  [136/145] Found dead NPC daemon proxy (`{}` in {}), cleaning with backup...",
                npc_proxy.yellow(),
                npc_path.display()
            );
            if npc_path.exists() {
                let backup = npc_path.with_extension("bak");
                let _ = std::fs::copy(&npc_path, &backup);
            }
            if let Ok(true) = unset_npc_proxy() {
                println!(
                    "  {} Cleared broken NPC proxy in {} (backup saved)",
                    "✔".green(),
                    npc_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [136/145] NPC daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [136/145] NPC daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 137. Clean dead Shadowsocks daemon proxy in config.json / systemd
    if let Some((ss_proxy, ss_path)) = get_shadowsocks_proxy() {
        if !verify_proxy_url(&ss_proxy).await {
            println!(
                "  [137/145] Found dead Shadowsocks daemon proxy (`{}` in {}), cleaning with backup...",
                ss_proxy.yellow(),
                ss_path.display()
            );
            if ss_path.exists() {
                let backup = ss_path.with_extension("bak");
                let _ = std::fs::copy(&ss_path, &backup);
            }
            if let Ok(true) = unset_shadowsocks_proxy() {
                println!(
                    "  {} Cleared broken Shadowsocks proxy in {} (backup saved)",
                    "✔".green(),
                    ss_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [137/145] Shadowsocks daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [137/145] Shadowsocks daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 138. Clean dead V2Ray & Xray daemon proxy in config.json / systemd
    if let Some((v2ray_proxy, v2ray_path)) = get_v2ray_proxy() {
        if !verify_proxy_url(&v2ray_proxy).await {
            println!(
                "  [138/145] Found dead V2Ray daemon proxy (`{}` in {}), cleaning with backup...",
                v2ray_proxy.yellow(),
                v2ray_path.display()
            );
            if v2ray_path.exists() {
                let backup = v2ray_path.with_extension("bak");
                let _ = std::fs::copy(&v2ray_path, &backup);
            }
            if let Ok(true) = unset_v2ray_proxy() {
                println!(
                    "  {} Cleared broken V2Ray proxy in {} (backup saved)",
                    "✔".green(),
                    v2ray_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [138/145] V2Ray daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [138/145] V2Ray daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 139. Clean dead Sing-box daemon proxy in config.json / systemd
    if let Some((sb_proxy, sb_path)) = get_sing_box_proxy() {
        if !verify_proxy_url(&sb_proxy).await {
            println!(
                "  [139/145] Found dead Sing-box daemon proxy (`{}` in {}), cleaning with backup...",
                sb_proxy.yellow(),
                sb_path.display()
            );
            if sb_path.exists() {
                let backup = sb_path.with_extension("bak");
                let _ = std::fs::copy(&sb_path, &backup);
            }
            if let Ok(true) = unset_sing_box_proxy() {
                println!(
                    "  {} Cleared broken Sing-box proxy in {} (backup saved)",
                    "✔".green(),
                    sb_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [139/145] Sing-box daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [139/145] Sing-box daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 140. Clean dead Hysteria daemon proxy in config.yaml / config.json / systemd
    if let Some((hy_proxy, hy_path)) = get_hysteria_proxy() {
        if !verify_proxy_url(&hy_proxy).await {
            println!(
                "  [140/151] Found dead Hysteria daemon proxy (`{}` in {}), cleaning with backup...",
                hy_proxy.yellow(),
                hy_path.display()
            );
            if hy_path.exists() {
                let backup = hy_path.with_extension("bak");
                let _ = std::fs::copy(&hy_path, &backup);
            }
            if let Ok(true) = unset_hysteria_proxy() {
                println!(
                    "  {} Cleared broken Hysteria proxy in {} (backup saved)",
                    "✔".green(),
                    hy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [140/151] Hysteria daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [140/151] Hysteria daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 141. Clean dead Trojan daemon proxy in config.json / systemd
    if let Some((tj_proxy, tj_path)) = get_trojan_proxy() {
        if !verify_proxy_url(&tj_proxy).await {
            println!(
                "  [141/151] Found dead Trojan daemon proxy (`{}` in {}), cleaning with backup...",
                tj_proxy.yellow(),
                tj_path.display()
            );
            if tj_path.exists() {
                let backup = tj_path.with_extension("bak");
                let _ = std::fs::copy(&tj_path, &backup);
            }
            if let Ok(true) = unset_trojan_proxy() {
                println!(
                    "  {} Cleared broken Trojan proxy in {} (backup saved)",
                    "✔".green(),
                    tj_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [141/151] Trojan daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [141/151] Trojan daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 142. Clean dead NaïveProxy daemon proxy in config.json / systemd
    if let Some((nv_proxy, nv_path)) = get_naive_proxy() {
        if !verify_proxy_url(&nv_proxy).await {
            println!(
                "  [142/151] Found dead NaïveProxy daemon proxy (`{}` in {}), cleaning with backup...",
                nv_proxy.yellow(),
                nv_path.display()
            );
            if nv_path.exists() {
                let backup = nv_path.with_extension("bak");
                let _ = std::fs::copy(&nv_path, &backup);
            }
            if let Ok(true) = unset_naive_proxy() {
                println!(
                    "  {} Cleared broken NaïveProxy proxy in {} (backup saved)",
                    "✔".green(),
                    nv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [142/151] NaïveProxy daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [142/151] NaïveProxy daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 143. Clean dead Tuic daemon proxy in client.json / systemd
    if let Some((tu_proxy, tu_path)) = get_tuic_proxy() {
        if !verify_proxy_url(&tu_proxy).await {
            println!(
                "  [143/151] Found dead Tuic daemon proxy (`{}` in {}), cleaning with backup...",
                tu_proxy.yellow(),
                tu_path.display()
            );
            if tu_path.exists() {
                let backup = tu_path.with_extension("bak");
                let _ = std::fs::copy(&tu_path, &backup);
            }
            if let Ok(true) = unset_tuic_proxy() {
                println!(
                    "  {} Cleared broken Tuic proxy in {} (backup saved)",
                    "✔".green(),
                    tu_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [143/151] Tuic daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [143/151] Tuic daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 144. Clean dead Juicity daemon proxy in client.json / systemd
    if let Some((ju_proxy, ju_path)) = get_juicity_proxy() {
        if !verify_proxy_url(&ju_proxy).await {
            println!(
                "  [144/151] Found dead Juicity daemon proxy (`{}` in {}), cleaning with backup...",
                ju_proxy.yellow(),
                ju_path.display()
            );
            if ju_path.exists() {
                let backup = ju_path.with_extension("bak");
                let _ = std::fs::copy(&ju_path, &backup);
            }
            if let Ok(true) = unset_juicity_proxy() {
                println!(
                    "  {} Cleared broken Juicity proxy in {} (backup saved)",
                    "✔".green(),
                    ju_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [144/151] Juicity daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [144/151] Juicity daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 145. Clean dead Snell daemon proxy in snell.conf / systemd
    if let Some((sn_proxy, sn_path)) = get_snell_proxy() {
        if !verify_proxy_url(&sn_proxy).await {
            println!(
                "  [145/153] Found dead Snell daemon proxy (`{}` in {}), cleaning with backup...",
                sn_proxy.yellow(),
                sn_path.display()
            );
            if sn_path.exists() {
                let backup = sn_path.with_extension("bak");
                let _ = std::fs::copy(&sn_path, &backup);
            }
            if let Ok(true) = unset_snell_proxy() {
                println!(
                    "  {} Cleared broken Snell proxy in {} (backup saved)",
                    "✔".green(),
                    sn_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [145/153] Snell daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [145/153] Snell daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 146. Clean dead Leaf daemon proxy in leaf.conf / systemd
    if let Some((lf_proxy, lf_path)) = get_leaf_proxy() {
        if !verify_proxy_url(&lf_proxy).await {
            println!(
                "  [146/153] Found dead Leaf daemon proxy (`{}` in {}), cleaning with backup...",
                lf_proxy.yellow(),
                lf_path.display()
            );
            if lf_path.exists() {
                let backup = lf_path.with_extension("bak");
                let _ = std::fs::copy(&lf_path, &backup);
            }
            if let Ok(true) = unset_leaf_proxy() {
                println!(
                    "  {} Cleared broken Leaf proxy in {} (backup saved)",
                    "✔".green(),
                    lf_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [146/153] Leaf daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [146/153] Leaf daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 147. Clean dead Mihomo daemon proxy in config.yaml / systemd
    if let Some((mh_proxy, mh_path)) = get_mihomo_proxy() {
        if !verify_proxy_url(&mh_proxy).await {
            println!(
                "  [147/155] Found dead Mihomo daemon proxy (`{}` in {}), cleaning with backup...",
                mh_proxy.yellow(),
                mh_path.display()
            );
            if mh_path.exists() {
                let backup = mh_path.with_extension("bak");
                let _ = std::fs::copy(&mh_path, &backup);
            }
            if let Ok(true) = unset_mihomo_proxy() {
                println!(
                    "  {} Cleared broken Mihomo proxy in {} (backup saved)",
                    "✔".green(),
                    mh_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [147/155] Mihomo daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [147/155] Mihomo daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 148. Clean dead Stunnel daemon proxy in stunnel.conf / systemd
    if let Some((st_proxy, st_path)) = get_stunnel_proxy() {
        if !verify_proxy_url(&st_proxy).await {
            println!(
                "  [148/155] Found dead Stunnel daemon proxy (`{}` in {}), cleaning with backup...",
                st_proxy.yellow(),
                st_path.display()
            );
            if st_path.exists() {
                let backup = st_path.with_extension("bak");
                let _ = std::fs::copy(&st_path, &backup);
            }
            if let Ok(true) = unset_stunnel_proxy() {
                println!(
                    "  {} Cleared broken Stunnel proxy in {} (backup saved)",
                    "✔".green(),
                    st_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [148/155] Stunnel daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [148/155] Stunnel daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 149. Clean dead Shadow-TLS daemon proxy in config.json / config.yaml / systemd
    if let Some((sh_proxy, sh_path)) = get_shadow_tls_proxy() {
        if !verify_proxy_url(&sh_proxy).await {
            println!(
                "  [149/157] Found dead Shadow-TLS daemon proxy (`{}` in {}), cleaning with backup...",
                sh_proxy.yellow(),
                sh_path.display()
            );
            if sh_path.exists() {
                let backup = sh_path.with_extension("bak");
                let _ = std::fs::copy(&sh_path, &backup);
            }
            if let Ok(true) = unset_shadow_tls_proxy() {
                println!(
                    "  {} Cleared broken Shadow-TLS proxy in {} (backup saved)",
                    "✔".green(),
                    sh_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [149/157] Shadow-TLS daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [149/157] Shadow-TLS daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 150. Clean dead Tun2socks daemon proxy in config.yaml / config.json / systemd
    if let Some((t2s_proxy, t2s_path)) = get_tun2socks_proxy() {
        if !verify_proxy_url(&t2s_proxy).await {
            println!(
                "  [150/157] Found dead Tun2socks daemon proxy (`{}` in {}), cleaning with backup...",
                t2s_proxy.yellow(),
                t2s_path.display()
            );
            if t2s_path.exists() {
                let backup = t2s_path.with_extension("bak");
                let _ = std::fs::copy(&t2s_path, &backup);
            }
            if let Ok(true) = unset_tun2socks_proxy() {
                println!(
                    "  {} Cleared broken Tun2socks proxy in {} (backup saved)",
                    "✔".green(),
                    t2s_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [150/157] Tun2socks daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [150/157] Tun2socks daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 151. Clean dead WSTunnel daemon proxy in config.yaml / config.json / systemd
    if let Some((ws_proxy, ws_path)) = get_wstunnel_proxy() {
        if !verify_proxy_url(&ws_proxy).await {
            println!(
                "  [151/157] Found dead WSTunnel daemon proxy (`{}` in {}), cleaning with backup...",
                ws_proxy.yellow(),
                ws_path.display()
            );
            if ws_path.exists() {
                let backup = ws_path.with_extension("bak");
                let _ = std::fs::copy(&ws_path, &backup);
            }
            if let Ok(true) = unset_wstunnel_proxy() {
                println!(
                    "  {} Cleared broken WSTunnel proxy in {} (backup saved)",
                    "✔".green(),
                    ws_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [151/157] WSTunnel daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [151/157] WSTunnel daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 152. Clean dead Cloak daemon proxy in ckclient.json / config.json / systemd
    if let Some((ck_proxy, ck_path)) = get_cloak_proxy() {
        if !verify_proxy_url(&ck_proxy).await {
            println!(
                "  [152/157] Found dead Cloak daemon proxy (`{}` in {}), cleaning with backup...",
                ck_proxy.yellow(),
                ck_path.display()
            );
            if ck_path.exists() {
                let backup = ck_path.with_extension("bak");
                let _ = std::fs::copy(&ck_path, &backup);
            }
            if let Ok(true) = unset_cloak_proxy() {
                println!(
                    "  {} Cleared broken Cloak proxy in {} (backup saved)",
                    "✔".green(),
                    ck_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [152/165] Cloak daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [152/165] Cloak daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 153. Clean dead Wireproxy daemon proxy in config.yaml / wireproxy.conf / systemd
    if let Some((wire_proxy, wire_path)) = get_wireproxy_proxy() {
        if !verify_proxy_url(&wire_proxy).await {
            println!(
                "  [153/165] Found dead Wireproxy daemon proxy (`{}` in {}), cleaning with backup...",
                wire_proxy.yellow(),
                wire_path.display()
            );
            if wire_path.exists() {
                let backup = wire_path.with_extension("bak");
                let _ = std::fs::copy(&wire_path, &backup);
            }
            if let Ok(true) = unset_wireproxy_proxy() {
                println!(
                    "  {} Cleared broken Wireproxy proxy in {} (backup saved)",
                    "✔".green(),
                    wire_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [153/165] Wireproxy daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [153/165] Wireproxy daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 154. Clean dead MTG daemon proxy in config.toml / config.yaml / systemd
    if let Some((mtg_proxy, mtg_path)) = get_mtg_proxy() {
        if !verify_proxy_url(&mtg_proxy).await {
            println!(
                "  [154/165] Found dead MTG daemon proxy (`{}` in {}), cleaning with backup...",
                mtg_proxy.yellow(),
                mtg_path.display()
            );
            if mtg_path.exists() {
                let backup = mtg_path.with_extension("bak");
                let _ = std::fs::copy(&mtg_path, &backup);
            }
            if let Ok(true) = unset_mtg_proxy() {
                println!(
                    "  {} Cleared broken MTG proxy in {} (backup saved)",
                    "✔".green(),
                    mtg_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [154/165] MTG daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [154/165] MTG daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 155. Clean dead Kcptun daemon proxy in client.json / kcptun.json / systemd
    if let Some((kcptun_proxy, kcptun_path)) = get_kcptun_proxy() {
        if !verify_proxy_url(&kcptun_proxy).await {
            println!(
                "  [155/165] Found dead Kcptun daemon proxy (`{}` in {}), cleaning with backup...",
                kcptun_proxy.yellow(),
                kcptun_path.display()
            );
            if kcptun_path.exists() {
                let backup = kcptun_path.with_extension("bak");
                let _ = std::fs::copy(&kcptun_path, &backup);
            }
            if let Ok(true) = unset_kcptun_proxy() {
                println!(
                    "  {} Cleared broken Kcptun proxy in {} (backup saved)",
                    "✔".green(),
                    kcptun_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [155/165] Kcptun daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [155/165] Kcptun daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 156. Clean dead Glider daemon proxy in glider.conf / systemd
    if let Some((glider_proxy, glider_path)) = get_glider_proxy() {
        if !verify_proxy_url(&glider_proxy).await {
            println!(
                "  [156/165] Found dead Glider daemon proxy (`{}` in {}), cleaning with backup...",
                glider_proxy.yellow(),
                glider_path.display()
            );
            if glider_path.exists() {
                let backup = glider_path.with_extension("bak");
                let _ = std::fs::copy(&glider_path, &backup);
            }
            if let Ok(true) = unset_glider_proxy() {
                println!(
                    "  {} Cleared broken Glider proxy in {} (backup saved)",
                    "✔".green(),
                    glider_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [156/165] Glider daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [156/165] Glider daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 157. Clean dead Redsocks daemon proxy in redsocks.conf / systemd
    if let Some((redsocks_proxy, redsocks_path)) = get_redsocks_proxy() {
        if !verify_proxy_url(&redsocks_proxy).await {
            println!(
                "  [157/165] Found dead Redsocks daemon proxy (`{}` in {}), cleaning with backup...",
                redsocks_proxy.yellow(),
                redsocks_path.display()
            );
            if redsocks_path.exists() {
                let backup = redsocks_path.with_extension("bak");
                let _ = std::fs::copy(&redsocks_path, &backup);
            }
            if let Ok(true) = unset_redsocks_proxy() {
                println!(
                    "  {} Cleared broken Redsocks proxy in {} (backup saved)",
                    "✔".green(),
                    redsocks_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [157/165] Redsocks daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [157/165] Redsocks daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 158. Clean dead Dante daemon proxy in danted.conf / systemd
    if let Some((dante_proxy, dante_path)) = get_dante_proxy() {
        if !verify_proxy_url(&dante_proxy).await {
            println!(
                "  [158/165] Found dead Dante daemon proxy (`{}` in {}), cleaning with backup...",
                dante_proxy.yellow(),
                dante_path.display()
            );
            if dante_path.exists() {
                let backup = dante_path.with_extension("bak");
                let _ = std::fs::copy(&dante_path, &backup);
            }
            if let Ok(true) = unset_dante_proxy() {
                println!(
                    "  {} Cleared broken Dante proxy in {} (backup saved)",
                    "✔".green(),
                    dante_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [158/165] Dante daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [158/165] Dante daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 159. Clean dead 3proxy daemon proxy in 3proxy.cfg / systemd
    if let Some((threeproxy_proxy, threeproxy_path)) = get_threeproxy_proxy() {
        if !verify_proxy_url(&threeproxy_proxy).await {
            println!(
                "  [159/165] Found dead 3proxy daemon proxy (`{}` in {}), cleaning with backup...",
                threeproxy_proxy.yellow(),
                threeproxy_path.display()
            );
            if threeproxy_path.exists() {
                let backup = threeproxy_path.with_extension("bak");
                let _ = std::fs::copy(&threeproxy_path, &backup);
            }
            if let Ok(true) = unset_threeproxy_proxy() {
                println!(
                    "  {} Cleared broken 3proxy proxy in {} (backup saved)",
                    "✔".green(),
                    threeproxy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [159/165] 3proxy daemon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [159/165] 3proxy daemon proxy: {}", "Clean or not set".dimmed());
    }

    // 160. Clean dead Polipo caching web proxy in config / systemd
    if let Some((polipo_proxy, polipo_path)) = get_polipo_proxy() {
        if !verify_proxy_url(&polipo_proxy).await {
            println!(
                "  [160/167] Found dead Polipo caching web proxy (`{}` in {}), cleaning with backup...",
                polipo_proxy.yellow(),
                polipo_path.display()
            );
            if polipo_path.exists() {
                let backup = polipo_path.with_extension("bak");
                let _ = std::fs::copy(&polipo_path, &backup);
            }
            if let Ok(true) = unset_polipo_proxy() {
                println!(
                    "  {} Cleared broken Polipo proxy in {} (backup saved)",
                    "✔".green(),
                    polipo_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [160/169] Polipo caching web proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [160/169] Polipo caching web proxy: {}", "Clean or not set".dimmed());
    }

    // 161. Clean dead Pproxy Python proxy daemon in config / systemd
    if let Some((pproxy_proxy, pproxy_path)) = get_pproxy_proxy() {
        if !verify_proxy_url(&pproxy_proxy).await {
            println!(
                "  [161/169] Found dead Pproxy Python proxy (`{}` in {}), cleaning with backup...",
                pproxy_proxy.yellow(),
                pproxy_path.display()
            );
            if pproxy_path.exists() {
                let backup = pproxy_path.with_extension("bak");
                let _ = std::fs::copy(&pproxy_path, &backup);
            }
            if let Ok(true) = unset_pproxy_proxy() {
                println!(
                    "  {} Cleared broken Pproxy proxy in {} (backup saved)",
                    "✔".green(),
                    pproxy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [161/169] Pproxy Python proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [161/169] Pproxy Python proxy: {}", "Clean or not set".dimmed());
    }

    // 162. Clean dead MicroSOCKS SOCKS5 proxy daemon in config / systemd
    if let Some((microsocks_proxy, microsocks_path)) = get_microsocks_proxy() {
        if !verify_proxy_url(&microsocks_proxy).await {
            println!(
                "  [162/169] Found dead MicroSOCKS proxy (`{}` in {}), cleaning with backup...",
                microsocks_proxy.yellow(),
                microsocks_path.display()
            );
            if microsocks_path.exists() {
                let backup = microsocks_path.with_extension("bak");
                let _ = std::fs::copy(&microsocks_path, &backup);
            }
            if let Ok(true) = unset_microsocks_proxy() {
                println!(
                    "  {} Cleared broken MicroSOCKS proxy in {} (backup saved)",
                    "✔".green(),
                    microsocks_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [162/171] MicroSOCKS proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [162/171] MicroSOCKS proxy: {}", "Clean or not set".dimmed());
    }

    // 163. Clean dead Cntlm Authenticating proxy daemon in config / systemd
    if let Some((cntlm_proxy, cntlm_path)) = get_cntlm_proxy() {
        if !verify_proxy_url(&cntlm_proxy).await {
            println!(
                "  [163/171] Found dead Cntlm proxy (`{}` in {}), cleaning with backup...",
                cntlm_proxy.yellow(),
                cntlm_path.display()
            );
            if cntlm_path.exists() {
                let backup = cntlm_path.with_extension("bak");
                let _ = std::fs::copy(&cntlm_path, &backup);
            }
            if let Ok(true) = unset_cntlm_proxy() {
                println!(
                    "  {} Cleared broken Cntlm proxy in {} (backup saved)",
                    "✔".green(),
                    cntlm_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [163/171] Cntlm proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [163/171] Cntlm proxy: {}", "Clean or not set".dimmed());
    }

    // 164. Clean dead COW HTTP proxy daemon in config / systemd
    if let Some((cow_proxy, cow_path)) = get_cow_proxy() {
        if !verify_proxy_url(&cow_proxy).await {
            println!(
                "  [164/171] Found dead COW proxy (`{}` in {}), cleaning with backup...",
                cow_proxy.yellow(),
                cow_path.display()
            );
            if cow_path.exists() {
                let backup = cow_path.with_extension("bak");
                let _ = std::fs::copy(&cow_path, &backup);
            }
            if let Ok(true) = unset_cow_proxy() {
                println!(
                    "  {} Cleared broken COW proxy in {} (backup saved)",
                    "✔".green(),
                    cow_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [164/171] COW proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [164/171] COW proxy: {}", "Clean or not set".dimmed());
    }

    // 165. Clean dead Sniproxy SNI proxy daemon in config / systemd
    if let Some((sniproxy_proxy, sniproxy_path)) = get_sniproxy_proxy() {
        if !verify_proxy_url(&sniproxy_proxy).await {
            println!(
                "  [165/173] Found dead Sniproxy proxy (`{}` in {}), cleaning with backup...",
                sniproxy_proxy.yellow(),
                sniproxy_path.display()
            );
            if sniproxy_path.exists() {
                let backup = sniproxy_path.with_extension("bak");
                let _ = std::fs::copy(&sniproxy_path, &backup);
            }
            if let Ok(true) = unset_sniproxy_proxy() {
                println!(
                    "  {} Cleared broken Sniproxy proxy in {} (backup saved)",
                    "✔".green(),
                    sniproxy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [165/173] Sniproxy proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [165/173] Sniproxy proxy: {}", "Clean or not set".dimmed());
    }

    // 166. Clean dead Dnsproxy AdGuard DNS proxy daemon in config / systemd
    if let Some((dnsproxy_proxy, dnsproxy_path)) = get_dnsproxy_proxy() {
        if !verify_proxy_url(&dnsproxy_proxy).await {
            println!(
                "  [166/173] Found dead Dnsproxy proxy (`{}` in {}), cleaning with backup...",
                dnsproxy_proxy.yellow(),
                dnsproxy_path.display()
            );
            if dnsproxy_path.exists() {
                let backup = dnsproxy_path.with_extension("bak");
                let _ = std::fs::copy(&dnsproxy_path, &backup);
            }
            if let Ok(true) = unset_dnsproxy_proxy() {
                println!(
                    "  {} Cleared broken Dnsproxy proxy in {} (backup saved)",
                    "✔".green(),
                    dnsproxy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [166/173] Dnsproxy proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [166/173] Dnsproxy proxy: {}", "Clean or not set".dimmed());
    }

    // 167. Clean dead Mosdns Pluggable DNS forwarder daemon in config / systemd
    if let Some((mosdns_proxy, mosdns_path)) = get_mosdns_proxy() {
        if !verify_proxy_url(&mosdns_proxy).await {
            println!(
                "  [167/173] Found dead Mosdns proxy (`{}` in {}), cleaning with backup...",
                mosdns_proxy.yellow(),
                mosdns_path.display()
            );
            if mosdns_path.exists() {
                let backup = mosdns_path.with_extension("bak");
                let _ = std::fs::copy(&mosdns_path, &backup);
            }
            if let Ok(true) = unset_mosdns_proxy() {
                println!(
                    "  {} Cleared broken Mosdns proxy in {} (backup saved)",
                    "✔".green(),
                    mosdns_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [167/173] Mosdns proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [167/173] Mosdns proxy: {}", "Clean or not set".dimmed());
    }

    // 168. Clean dead SmartDNS local DNS proxy daemon in config / systemd
    if let Some((smartdns_proxy, smartdns_path)) = get_smartdns_proxy() {
        if !verify_proxy_url(&smartdns_proxy).await {
            println!(
                "  [168/173] Found dead SmartDNS proxy (`{}` in {}), cleaning with backup...",
                smartdns_proxy.yellow(),
                smartdns_path.display()
            );
            if smartdns_path.exists() {
                let backup = smartdns_path.with_extension("bak");
                let _ = std::fs::copy(&smartdns_path, &backup);
            }
            if let Ok(true) = unset_smartdns_proxy() {
                println!(
                    "  {} Cleared broken SmartDNS proxy in {} (backup saved)",
                    "✔".green(),
                    smartdns_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [168/173] SmartDNS proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [168/173] SmartDNS proxy: {}", "Clean or not set".dimmed());
    }

    // 169. Clean dead Alpine Linux apk package manager repositories in /etc/apk/repositories
    let apk_paths = get_apk_repositories_paths();
    if !apk_paths.is_empty() {
        for apath in apk_paths {
            if let Ok(content) = std::fs::read_to_string(&apath) {
                let repos = parse_apk_repositories(&content);
                let mut dead_repos = Vec::new();
                for repo in &repos {
                    if !verify_proxy_url(repo).await {
                        dead_repos.push(repo.as_str());
                    }
                }
                if !dead_repos.is_empty() {
                    println!(
                        "  [169/173] Found {} dead Alpine apk repository mirror(s) in {}, cleaning with backup...",
                        dead_repos.len().to_string().yellow(),
                        apath.display()
                    );
                    let (new_content, modified) = clean_apk_repositories(&content, &dead_repos);
                    if modified {
                        let backup = apath.with_extension("bak");
                        let _ = std::fs::copy(&apath, &backup);
                        let _ = std::fs::write(&apath, new_content);
                        println!(
                            "  {} Cleaned dead Alpine apk repository mirror(s) in {} (backup saved)",
                            "✔".green(),
                            apath.display()
                        );
                        fixed_count += 1;
                    }
                } else if !repos.is_empty() {
                    println!("  [169/173] Alpine Linux apk repositories: {}", "Reachable".green());
                } else {
                    println!("  [169/173] Alpine Linux apk repositories: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [169/173] Alpine Linux apk repositories: {}", "Clean or not set".dimmed());
    }

    // 170. Clean dead vcpkg registries/mirrors in vcpkg-configuration.json
    let vcpkg_paths = get_vcpkg_configuration_paths();
    if !vcpkg_paths.is_empty() {
        for vpath in vcpkg_paths {
            if let Ok(content) = std::fs::read_to_string(&vpath) {
                let regs = parse_vcpkg_configuration_registries(&content);
                let mut dead_regs = Vec::new();
                for reg in &regs {
                    if !verify_proxy_url(reg).await {
                        dead_regs.push(reg.as_str());
                    }
                }
                if !dead_regs.is_empty() {
                    println!(
                        "  [170/173] Found {} dead vcpkg registry mirror(s) in {}, cleaning with backup...",
                        dead_regs.len().to_string().yellow(),
                        vpath.display()
                    );
                    let (new_content, modified) =
                        clean_vcpkg_configuration_registries(&content, &dead_regs);
                    if modified {
                        let backup = vpath.with_extension("bak");
                        let _ = std::fs::copy(&vpath, &backup);
                        let _ = std::fs::write(&vpath, new_content);
                        println!(
                            "  {} Cleaned dead vcpkg registry mirror(s) in {} (backup saved)",
                            "✔".green(),
                            vpath.display()
                        );
                        fixed_count += 1;
                    }
                } else if !regs.is_empty() {
                    println!("  [170/173] vcpkg configuration registries: {}", "Reachable".green());
                } else {
                    println!("  [170/173] vcpkg configuration registries: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [170/173] vcpkg configuration registries: {}", "Clean or not set".dimmed());
    }

    // 171. Clean dead Kubernetes Helm chart repositories in repositories.yaml
    let helm_paths = get_helm_repositories_paths();
    if !helm_paths.is_empty() {
        for hpath in helm_paths {
            if let Ok(content) = std::fs::read_to_string(&hpath) {
                let repos = parse_helm_repositories(&content);
                let mut dead_repos = Vec::new();
                for repo in &repos {
                    if !verify_proxy_url(repo).await {
                        dead_repos.push(repo.as_str());
                    }
                }
                if !dead_repos.is_empty() {
                    println!(
                        "  [171/173] Found {} dead Helm chart repository mirror(s) in {}, cleaning with backup...",
                        dead_repos.len().to_string().yellow(),
                        hpath.display()
                    );
                    let (new_content, modified) =
                        clean_helm_repositories(&content, &dead_repos);
                    if modified {
                        let backup = hpath.with_extension("bak");
                        let _ = std::fs::copy(&hpath, &backup);
                        let _ = std::fs::write(&hpath, new_content);
                        println!(
                            "  {} Cleaned dead Helm chart repository mirror(s) in {} (backup saved)",
                            "✔".green(),
                            hpath.display()
                        );
                        fixed_count += 1;
                    }
                } else if !repos.is_empty() {
                    println!("  [171/173] Kubernetes Helm chart repositories: {}", "Reachable".green());
                } else {
                    println!("  [171/173] Kubernetes Helm chart repositories: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [171/173] Kubernetes Helm chart repositories: {}", "Clean or not set".dimmed());
    }

    // 172. Sync active system/daemon proxy to terminal with proper NO_PROXY bypass
    let active_port = match get_system_proxy() {
        Some(sys_info) if sys_info.state.http_enabled && sys_info.state.http_port > 0 => {
            if scanner::check_host_port(&sys_info.state.http_host, sys_info.state.http_port).await {
                Some(sys_info.state.http_port)
            } else {
                None
            }
        }
        _ => None,
    };

    let target_port = match active_port {
        Some(p) => Some(p),
        None => scanner::find_first_active_proxy().await.map(|p| p.port),
    };

    if let Some(port) = target_port {
        println!(
            "  [172/173] Detected healthy active proxy on port {}. Generating Shell sync...",
            port.to_string().green().bold()
        );
        let proxy_url = format!("http://127.0.0.1:{}", port);
        println!(
            "  {} Recommended shell injection (run `eval $(proxydoc on)`):",
            "✔".green()
        );
        println!(
            "\n    {}",
            format!("export http_proxy=\"{}\"", proxy_url).cyan()
        );
        println!(
            "    {}",
            format!("export https_proxy=\"{}\"", proxy_url).cyan()
        );
        println!(
            "    {}",
            format!("export all_proxy=\"{}\"", proxy_url).cyan()
        );
        println!(
            "    {}",
            "export no_proxy=\"localhost,127.0.0.1,::1,localaddress,.local\"".cyan()
        );
        println!();
        fixed_count += 1;
    } else {
        println!(
            "  [172/173] No active local proxy client found. Ensure your proxy software is open."
        );
    }

    // 173. Flush local DNS cache
    println!("  [173/173] Flushing local DNS cache to fix resolution stalls...");
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("dscacheutil").arg("-flushcache").status();
        let _ = Command::new("killall")
            .args(["-HUP", "mDNSResponder"])
            .status();
        println!("  {} macOS DNS cache flushed.", "✔".green());
        fixed_count += 1;
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("ipconfig").arg("/flushdns").status();
        println!("  {} Windows DNS cache flushed.", "✔".green());
        fixed_count += 1;
    }
    #[cfg(target_os = "linux")]
    {
        let flushed = Command::new("resolvectl")
            .args(["flush-caches"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
            || Command::new("systemd-resolve")
                .args(["--flush-caches"])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
        if flushed {
            println!("  {} Linux DNS cache flushed.", "✔".green());
            fixed_count += 1;
        } else {
            println!(
                "  {} Linux DNS cache: No systemd-resolved detected or flush skipped.",
                "○".dimmed()
            );
        }
    }

    println!(
        "\n{}",
        format!("🎉 Auto-repair complete! ({} action(s) taken)", fixed_count)
            .green()
            .bold()
    );
    println!("Run `proxydoc doctor` to verify the new state.\n");

    Ok(())
}

fn get_git_config(key: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["config", "--global", "--get", key])
        .output()
        .ok()?;
    if out.status.success() {
        let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !val.is_empty() {
            return Some(val);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_out_dead_lines() {
        let content = "verbose\nproxy = http://127.0.0.1:9999\nsilent\n";
        let (res, modified) = comment_out_dead_lines(content, |l| l.starts_with("proxy"));
        assert!(modified);
        assert!(res.contains("# [proxydoc disabled dead proxy]: proxy = http://127.0.0.1:9999"));
        assert!(res.contains("verbose\n"));
        assert!(res.contains("silent\n"));

        let clean_content = "verbose\n# already commented\n";
        let (_, clean_mod) = comment_out_dead_lines(clean_content, |l| l.starts_with("proxy"));
        assert!(!clean_mod);
    }
}
