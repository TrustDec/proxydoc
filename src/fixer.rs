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
    get_mosdns_proxy, get_smartdns_proxy, get_coredns_proxy, get_adguardhome_proxy,
    get_dnscrypt_proxy, get_unbound_proxy,
    get_dnsmasq_proxy, get_pihole_proxy,
    get_blocky_proxy, get_stubby_proxy,
    get_kresd_proxy, get_pdns_proxy,
    get_dnsdist_proxy, get_bind_proxy,
    get_nextdns_proxy, get_pdnsd_proxy,
    get_maradns_proxy, get_i2pd_proxy,
    get_dae_proxy, get_yggdrasil_proxy,
    get_cjdns_proxy, get_tinc_proxy,
    get_openvpn_proxy, get_softether_proxy,
    get_openconnect_proxy, get_strongswan_proxy,
    get_wireguard_proxy, get_headscale_proxy,
    get_netbird_proxy, get_warp_proxy,
    get_amnezia_proxy, get_openziti_proxy,
    get_firezone_proxy, get_netmaker_proxy,
    get_pritunl_proxy, get_twingate_proxy,
    get_easytier_proxy, get_teleport_proxy,
    get_pomerium_proxy, get_innernet_proxy,
    get_defguard_proxy, get_inlets_proxy,
    get_zrok_proxy, get_n2n_proxy,
    get_hyprspace_proxy, get_fastd_proxy,
    get_edgevpn_proxy, get_boringproxy_proxy,
    get_ghostunnel_proxy, get_freelan_proxy,
    get_omniedge_proxy, get_lantern_proxy,
    get_tunsafe_proxy, get_sshuttle_proxy,
    get_psiphon_proxy, get_localtunnel_proxy,
    get_pagekite_proxy, get_toxiproxy_proxy,
    get_telepresence_proxy, get_localxpose_proxy,
    get_packetriot_proxy, get_sish_proxy,
    get_pinggy_proxy, get_loophole_proxy,
    get_tunnelmole_proxy, get_expose_proxy,
    get_playit_proxy, get_pgrok_proxy,
    get_cpolar_proxy, get_tmate_proxy,
    get_upterm_proxy, get_natapp_proxy,
    get_lanproxy_proxy, get_teleconsole_proxy,
    get_tunnelto_proxy, get_webhookrelay_proxy,
    get_jprq_proxy, get_mysocketctl_proxy,
    get_openp2p_proxy, get_rport_proxy,
    get_mirrord_proxy, get_kt_connect_proxy,
    get_devspace_proxy, get_geph_proxy,
    get_okteto_proxy, get_skaffold_proxy,
    get_tilt_proxy, get_garden_proxy,
    get_daytona_proxy, get_vcluster_proxy,
    get_coder_proxy, get_devpod_proxy,
    get_gitpod_proxy, get_devcontainer_proxy,
    get_mise_proxy, get_pixi_proxy,
    get_proto_proxy, get_flox_proxy,
    get_devenv_proxy, get_devbox_proxy,
    get_aqua_proxy, get_hermit_proxy,
    get_asdf_proxy, get_vfox_proxy,
    get_sdkman_proxy, get_rustup_proxy,
    get_fnm_proxy, get_volta_proxy,
    get_pyenv_proxy, get_nvm_proxy,
    get_rbenv_proxy, get_gvm_proxy,
    get_nodenv_proxy, get_jenv_proxy,
    get_goenv_proxy, get_phpenv_proxy,
    get_swiftenv_proxy, get_perlbrew_proxy,
    get_luaver_proxy, get_ghcup_proxy,
    get_zvm_proxy, get_juliaup_proxy,
    get_elan_proxy, get_choosenim_proxy,
    get_coursier_proxy, get_kerl_proxy,
    get_tenv_proxy, get_tfswitch_proxy,
    get_tgswitch_proxy, get_tfenv_proxy,
    get_tgenv_proxy, get_pkenv_proxy,
    get_tofuenv_proxy, get_helmenv_proxy,
    get_kbenv_proxy, get_sopsenv_proxy,
    get_ocenv_proxy, get_sbtenv_proxy,
    get_scalaenv_proxy, get_cljenv_proxy,
    get_crenv_proxy, get_haxenv_proxy,
    get_plenv_proxy, get_exenv_proxy,
    get_luaenv_proxy, get_erlenv_proxy,
    get_dartenv_proxy, get_renv_proxy,
    get_kotenv_proxy, get_denoenv_proxy,
    get_bunenv_proxy, get_pgenv_proxy,
    get_rkenv_proxy, get_gdenv_proxy,
    get_anyenv_proxy, get_jlenv_proxy,
    get_zenv_proxy, get_vlenv_proxy,
    get_glenv_proxy, get_nimenv_proxy,
    get_tkenv_proxy, get_hsenv_proxy,
    get_kzenv_proxy, get_solenv_proxy,
    get_bzenv_proxy, get_ctenv_proxy,
    get_cmenv_proxy, get_bufenv_proxy,
    get_pklenv_proxy, get_opaenv_proxy,
    get_dbtenv_proxy, get_cueenv_proxy,
    get_wasmenv_proxy, get_hfenv_proxy,
    get_kopsenv_proxy, get_tflintenv_proxy,
    get_argoenv_proxy, get_istioenv_proxy,
    get_ciliumenv_proxy, get_linkerdenv_proxy,
    get_fluxenv_proxy, get_kindenv_proxy,
    get_k3denv_proxy, get_cosignenv_proxy,
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
    unset_coredns_proxy, unset_adguardhome_proxy,
    unset_dnscrypt_proxy, unset_unbound_proxy,
    unset_dnsmasq_proxy, unset_pihole_proxy,
    unset_blocky_proxy, unset_stubby_proxy,
    unset_kresd_proxy, unset_pdns_proxy,
    unset_dnsdist_proxy, unset_bind_proxy,
    unset_nextdns_proxy, unset_pdnsd_proxy,
    unset_maradns_proxy, unset_i2pd_proxy,
    unset_dae_proxy, unset_yggdrasil_proxy,
    unset_cjdns_proxy, unset_tinc_proxy,
    unset_openvpn_proxy, unset_softether_proxy,
    unset_openconnect_proxy, unset_strongswan_proxy,
    unset_wireguard_proxy, unset_headscale_proxy,
    unset_netbird_proxy, unset_warp_proxy,
    unset_amnezia_proxy, unset_openziti_proxy,
    unset_firezone_proxy, unset_netmaker_proxy,
    unset_pritunl_proxy, unset_twingate_proxy,
    unset_easytier_proxy, unset_teleport_proxy,
    unset_pomerium_proxy, unset_innernet_proxy,
    unset_defguard_proxy, unset_inlets_proxy,
    unset_zrok_proxy, unset_n2n_proxy,
    unset_hyprspace_proxy, unset_fastd_proxy,
    unset_edgevpn_proxy, unset_boringproxy_proxy,
    unset_ghostunnel_proxy, unset_freelan_proxy,
    unset_omniedge_proxy, unset_lantern_proxy,
    unset_tunsafe_proxy, unset_sshuttle_proxy,
    unset_psiphon_proxy, unset_localtunnel_proxy,
    unset_pagekite_proxy, unset_toxiproxy_proxy,
    unset_telepresence_proxy, unset_localxpose_proxy,
    unset_packetriot_proxy, unset_sish_proxy,
    unset_pinggy_proxy, unset_loophole_proxy,
    unset_tunnelmole_proxy, unset_expose_proxy,
    unset_playit_proxy, unset_pgrok_proxy,
    unset_cpolar_proxy, unset_tmate_proxy,
    unset_upterm_proxy, unset_natapp_proxy,
    unset_lanproxy_proxy, unset_teleconsole_proxy,
    unset_tunnelto_proxy, unset_webhookrelay_proxy,
    unset_jprq_proxy, unset_mysocketctl_proxy,
    unset_openp2p_proxy, unset_rport_proxy,
    unset_mirrord_proxy, unset_kt_connect_proxy,
    unset_devspace_proxy, unset_geph_proxy,
    unset_okteto_proxy, unset_skaffold_proxy,
    unset_tilt_proxy, unset_garden_proxy,
    unset_daytona_proxy, unset_vcluster_proxy,
    unset_coder_proxy, unset_devpod_proxy,
    unset_gitpod_proxy, unset_devcontainer_proxy,
    unset_mise_proxy, unset_pixi_proxy,
    unset_proto_proxy, unset_flox_proxy,
    unset_devenv_proxy, unset_devbox_proxy,
    unset_aqua_proxy, unset_hermit_proxy,
    unset_asdf_proxy, unset_vfox_proxy,
    unset_sdkman_proxy, unset_rustup_proxy,
    unset_fnm_proxy, unset_volta_proxy,
    unset_pyenv_proxy, unset_nvm_proxy,
    unset_rbenv_proxy, unset_gvm_proxy,
    unset_nodenv_proxy, unset_jenv_proxy,
    unset_goenv_proxy, unset_phpenv_proxy,
    unset_swiftenv_proxy, unset_perlbrew_proxy,
    unset_luaver_proxy, unset_ghcup_proxy,
    unset_zvm_proxy, unset_juliaup_proxy,
    unset_elan_proxy, unset_choosenim_proxy,
    unset_coursier_proxy, unset_kerl_proxy,
    unset_tenv_proxy, unset_tfswitch_proxy,
    unset_tgswitch_proxy, unset_tfenv_proxy,
    unset_tgenv_proxy, unset_pkenv_proxy,
    unset_tofuenv_proxy, unset_helmenv_proxy,
    unset_kbenv_proxy, unset_sopsenv_proxy,
    unset_ocenv_proxy, unset_sbtenv_proxy,
    unset_scalaenv_proxy, unset_cljenv_proxy,
    unset_crenv_proxy, unset_haxenv_proxy,
    unset_plenv_proxy, unset_exenv_proxy,
    unset_luaenv_proxy, unset_erlenv_proxy,
    unset_dartenv_proxy, unset_renv_proxy,
    unset_kotenv_proxy, unset_denoenv_proxy,
    unset_bunenv_proxy, unset_pgenv_proxy,
    unset_rkenv_proxy, unset_gdenv_proxy,
    unset_anyenv_proxy, unset_jlenv_proxy,
    unset_zenv_proxy, unset_vlenv_proxy,
    unset_glenv_proxy, unset_nimenv_proxy,
    unset_tkenv_proxy, unset_hsenv_proxy,
    unset_kzenv_proxy, unset_solenv_proxy,
    unset_bzenv_proxy, unset_ctenv_proxy,
    unset_cmenv_proxy, unset_bufenv_proxy,
    unset_pklenv_proxy, unset_opaenv_proxy,
    unset_dbtenv_proxy, unset_cueenv_proxy,
    unset_wasmenv_proxy, unset_hfenv_proxy,
    unset_kopsenv_proxy, unset_tflintenv_proxy,
    unset_argoenv_proxy, unset_istioenv_proxy,
    unset_ciliumenv_proxy, unset_linkerdenv_proxy,
    unset_fluxenv_proxy, unset_kindenv_proxy,
    unset_k3denv_proxy, unset_cosignenv_proxy,
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
            println!("  [165/175] Sniproxy proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [165/175] Sniproxy proxy: {}", "Clean or not set".dimmed());
    }

    // 166. Clean dead Dnsproxy AdGuard DNS proxy daemon in config / systemd
    if let Some((dnsproxy_proxy, dnsproxy_path)) = get_dnsproxy_proxy() {
        if !verify_proxy_url(&dnsproxy_proxy).await {
            println!(
                "  [166/175] Found dead Dnsproxy proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [166/175] Dnsproxy proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [166/175] Dnsproxy proxy: {}", "Clean or not set".dimmed());
    }

    // 167. Clean dead Mosdns Pluggable DNS forwarder daemon in config / systemd
    if let Some((mosdns_proxy, mosdns_path)) = get_mosdns_proxy() {
        if !verify_proxy_url(&mosdns_proxy).await {
            println!(
                "  [167/175] Found dead Mosdns proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [167/175] Mosdns proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [167/175] Mosdns proxy: {}", "Clean or not set".dimmed());
    }

    // 168. Clean dead SmartDNS local DNS proxy daemon in config / systemd
    if let Some((smartdns_proxy, smartdns_path)) = get_smartdns_proxy() {
        if !verify_proxy_url(&smartdns_proxy).await {
            println!(
                "  [168/175] Found dead SmartDNS proxy (`{}` in {}), cleaning with backup...",
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
            println!("  [168/175] SmartDNS proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [168/175] SmartDNS proxy: {}", "Clean or not set".dimmed());
    }

    // 169. Clean dead CoreDNS DNS server and forwarder daemon in config / systemd
    if let Some((coredns_proxy, coredns_path)) = get_coredns_proxy() {
        if !verify_proxy_url(&coredns_proxy).await {
            println!(
                "  [169/175] Found dead CoreDNS proxy (`{}` in {}), cleaning with backup...",
                coredns_proxy.yellow(),
                coredns_path.display()
            );
            if coredns_path.exists() {
                let backup = coredns_path.with_extension("bak");
                let _ = std::fs::copy(&coredns_path, &backup);
            }
            if let Ok(true) = unset_coredns_proxy() {
                println!(
                    "  {} Cleared broken CoreDNS proxy in {} (backup saved)",
                    "✔".green(),
                    coredns_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [169/187] CoreDNS proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [169/187] CoreDNS proxy: {}", "Clean or not set".dimmed());
    }

    // 170. Clean dead AdGuard Home DNS & web proxy daemon in config / systemd
    if let Some((adguard_proxy, adguard_path)) = get_adguardhome_proxy() {
        if !verify_proxy_url(&adguard_proxy).await {
            println!(
                "  [170/187] Found dead AdGuard Home proxy (`{}` in {}), cleaning with backup...",
                adguard_proxy.yellow(),
                adguard_path.display()
            );
            if adguard_path.exists() {
                let backup = adguard_path.with_extension("bak");
                let _ = std::fs::copy(&adguard_path, &backup);
            }
            if let Ok(true) = unset_adguardhome_proxy() {
                println!(
                    "  {} Cleared broken AdGuard Home proxy in {} (backup saved)",
                    "✔".green(),
                    adguard_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [170/187] AdGuard Home proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [170/187] AdGuard Home proxy: {}", "Clean or not set".dimmed());
    }

    // 171. Clean dead DNSCrypt-proxy Encrypted DNS proxy daemon in config / systemd
    if let Some((dnscrypt_proxy, dnscrypt_path)) = get_dnscrypt_proxy() {
        if !verify_proxy_url(&dnscrypt_proxy).await {
            println!(
                "  [171/187] Found dead DNSCrypt-proxy proxy (`{}` in {}), cleaning with backup...",
                dnscrypt_proxy.yellow(),
                dnscrypt_path.display()
            );
            if dnscrypt_path.exists() {
                let backup = dnscrypt_path.with_extension("bak");
                let _ = std::fs::copy(&dnscrypt_path, &backup);
            }
            if let Ok(true) = unset_dnscrypt_proxy() {
                println!(
                    "  {} Cleared broken DNSCrypt-proxy proxy in {} (backup saved)",
                    "✔".green(),
                    dnscrypt_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [171/187] DNSCrypt-proxy proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [171/187] DNSCrypt-proxy proxy: {}", "Clean or not set".dimmed());
    }

    // 172. Clean dead Unbound validating recursive DNS proxy daemon in config / systemd
    if let Some((unbound_proxy, unbound_path)) = get_unbound_proxy() {
        if !verify_proxy_url(&unbound_proxy).await {
            println!(
                "  [172/187] Found dead Unbound proxy (`{}` in {}), cleaning with backup...",
                unbound_proxy.yellow(),
                unbound_path.display()
            );
            if unbound_path.exists() {
                let backup = unbound_path.with_extension("bak");
                let _ = std::fs::copy(&unbound_path, &backup);
            }
            if let Ok(true) = unset_unbound_proxy() {
                println!(
                    "  {} Cleared broken Unbound proxy in {} (backup saved)",
                    "✔".green(),
                    unbound_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [172/187] Unbound proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [172/187] Unbound proxy: {}", "Clean or not set".dimmed());
    }

    // 173. Clean dead Dnsmasq lightweight DNS & DHCP proxy daemon in config / systemd
    if let Some((dnsmasq_proxy, dnsmasq_path)) = get_dnsmasq_proxy() {
        if !verify_proxy_url(&dnsmasq_proxy).await {
            println!(
                "  [173/187] Found dead Dnsmasq proxy (`{}` in {}), cleaning with backup...",
                dnsmasq_proxy.yellow(),
                dnsmasq_path.display()
            );
            if dnsmasq_path.exists() {
                let backup = dnsmasq_path.with_extension("bak");
                let _ = std::fs::copy(&dnsmasq_path, &backup);
            }
            if let Ok(true) = unset_dnsmasq_proxy() {
                println!(
                    "  {} Cleared broken Dnsmasq proxy in {} (backup saved)",
                    "✔".green(),
                    dnsmasq_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [173/187] Dnsmasq proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [173/187] Dnsmasq proxy: {}", "Clean or not set".dimmed());
    }

    // 174. Clean dead Pi-hole FTL DNS & sinkhole proxy daemon in config / systemd
    if let Some((pihole_proxy, pihole_path)) = get_pihole_proxy() {
        if !verify_proxy_url(&pihole_proxy).await {
            println!(
                "  [174/187] Found dead Pi-hole proxy (`{}` in {}), cleaning with backup...",
                pihole_proxy.yellow(),
                pihole_path.display()
            );
            if pihole_path.exists() {
                let backup = pihole_path.with_extension("bak");
                let _ = std::fs::copy(&pihole_path, &backup);
            }
            if let Ok(true) = unset_pihole_proxy() {
                println!(
                    "  {} Cleared broken Pi-hole proxy in {} (backup saved)",
                    "✔".green(),
                    pihole_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [174/187] Pi-hole proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [174/187] Pi-hole proxy: {}", "Clean or not set".dimmed());
    }

    // 175. Clean dead Blocky fast DNS & ad-blocker proxy daemon in config / systemd
    if let Some((blocky_proxy, blocky_path)) = get_blocky_proxy() {
        if !verify_proxy_url(&blocky_proxy).await {
            println!(
                "  [175/187] Found dead Blocky proxy (`{}` in {}), cleaning with backup...",
                blocky_proxy.yellow(),
                blocky_path.display()
            );
            if blocky_path.exists() {
                let backup = blocky_path.with_extension("bak");
                let _ = std::fs::copy(&blocky_path, &backup);
            }
            if let Ok(true) = unset_blocky_proxy() {
                println!(
                    "  {} Cleared broken Blocky proxy in {} (backup saved)",
                    "✔".green(),
                    blocky_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [175/187] Blocky proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [175/187] Blocky proxy: {}", "Clean or not set".dimmed());
    }

    // 176. Clean dead Stubby DNS Privacy DoT stub resolver proxy daemon in config / systemd
    if let Some((stubby_proxy, stubby_path)) = get_stubby_proxy() {
        if !verify_proxy_url(&stubby_proxy).await {
            println!(
                "  [176/187] Found dead Stubby proxy (`{}` in {}), cleaning with backup...",
                stubby_proxy.yellow(),
                stubby_path.display()
            );
            if stubby_path.exists() {
                let backup = stubby_path.with_extension("bak");
                let _ = std::fs::copy(&stubby_path, &backup);
            }
            if let Ok(true) = unset_stubby_proxy() {
                println!(
                    "  {} Cleared broken Stubby proxy in {} (backup saved)",
                    "✔".green(),
                    stubby_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [176/187] Stubby proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [176/187] Stubby proxy: {}", "Clean or not set".dimmed());
    }

    // 177. Clean dead Knot Resolver validating recursive DNS proxy daemon in config / systemd
    if let Some((kresd_proxy, kresd_path)) = get_kresd_proxy() {
        if !verify_proxy_url(&kresd_proxy).await {
            println!(
                "  [177/187] Found dead Knot Resolver proxy (`{}` in {}), cleaning with backup...",
                kresd_proxy.yellow(),
                kresd_path.display()
            );
            if kresd_path.exists() {
                let backup = kresd_path.with_extension("bak");
                let _ = std::fs::copy(&kresd_path, &backup);
            }
            if let Ok(true) = unset_kresd_proxy() {
                println!(
                    "  {} Cleared broken Knot Resolver proxy in {} (backup saved)",
                    "✔".green(),
                    kresd_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [177/187] Knot Resolver proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [177/187] Knot Resolver proxy: {}", "Clean or not set".dimmed());
    }

    // 178. Clean dead PowerDNS Recursor resolving DNS cache proxy daemon in config / systemd
    if let Some((pdns_proxy, pdns_path)) = get_pdns_proxy() {
        if !verify_proxy_url(&pdns_proxy).await {
            println!(
                "  [178/187] Found dead PowerDNS Recursor proxy (`{}` in {}), cleaning with backup...",
                pdns_proxy.yellow(),
                pdns_path.display()
            );
            if pdns_path.exists() {
                let backup = pdns_path.with_extension("bak");
                let _ = std::fs::copy(&pdns_path, &backup);
            }
            if let Ok(true) = unset_pdns_proxy() {
                println!(
                    "  {} Cleared broken PowerDNS Recursor proxy in {} (backup saved)",
                    "✔".green(),
                    pdns_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [178/187] PowerDNS Recursor proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [178/187] PowerDNS Recursor proxy: {}", "Clean or not set".dimmed());
    }

    // 179. Clean dead dnsdist DNS load balancer & proxy daemon in config / systemd
    if let Some((dnsdist_proxy, dnsdist_path)) = get_dnsdist_proxy() {
        if !verify_proxy_url(&dnsdist_proxy).await {
            println!(
                "  [179/187] Found dead dnsdist proxy (`{}` in {}), cleaning with backup...",
                dnsdist_proxy.yellow(),
                dnsdist_path.display()
            );
            if dnsdist_path.exists() {
                let backup = dnsdist_path.with_extension("bak");
                let _ = std::fs::copy(&dnsdist_path, &backup);
            }
            if let Ok(true) = unset_dnsdist_proxy() {
                println!(
                    "  {} Cleared broken dnsdist proxy in {} (backup saved)",
                    "✔".green(),
                    dnsdist_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [179/187] dnsdist proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [179/187] dnsdist proxy: {}", "Clean or not set".dimmed());
    }

    // 180. Clean dead BIND 9 (named) domain name system & resolver proxy daemon in config / systemd
    if let Some((bind_proxy, bind_path)) = get_bind_proxy() {
        if !verify_proxy_url(&bind_proxy).await {
            println!(
                "  [180/187] Found dead BIND 9 (named) proxy (`{}` in {}), cleaning with backup...",
                bind_proxy.yellow(),
                bind_path.display()
            );
            if bind_path.exists() {
                let backup = bind_path.with_extension("bak");
                let _ = std::fs::copy(&bind_path, &backup);
            }
            if let Ok(true) = unset_bind_proxy() {
                println!(
                    "  {} Cleared broken BIND 9 (named) proxy in {} (backup saved)",
                    "✔".green(),
                    bind_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [180/187] BIND 9 (named) proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [180/187] BIND 9 (named) proxy: {}", "Clean or not set".dimmed());
    }

    // 181. Clean dead NextDNS encrypted DNS client & proxy daemon in config / systemd
    if let Some((nextdns_proxy, nextdns_path)) = get_nextdns_proxy() {
        if !verify_proxy_url(&nextdns_proxy).await {
            println!(
                "  [181/187] Found dead NextDNS proxy (`{}` in {}), cleaning with backup...",
                nextdns_proxy.yellow(),
                nextdns_path.display()
            );
            if nextdns_path.exists() {
                let backup = nextdns_path.with_extension("bak");
                let _ = std::fs::copy(&nextdns_path, &backup);
            }
            if let Ok(true) = unset_nextdns_proxy() {
                println!(
                    "  {} Cleared broken NextDNS proxy in {} (backup saved)",
                    "✔".green(),
                    nextdns_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [181/187] NextDNS proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [181/187] NextDNS proxy: {}", "Clean or not set".dimmed());
    }

    // 182. Clean dead pdnsd proxy DNS daemon in config / systemd
    if let Some((pdnsd_proxy, pdnsd_path)) = get_pdnsd_proxy() {
        if !verify_proxy_url(&pdnsd_proxy).await {
            println!(
                "  [182/199] Found dead pdnsd proxy (`{}` in {}), cleaning with backup...",
                pdnsd_proxy.yellow(),
                pdnsd_path.display()
            );
            if pdnsd_path.exists() {
                let backup = pdnsd_path.with_extension("bak");
                let _ = std::fs::copy(&pdnsd_path, &backup);
            }
            if let Ok(true) = unset_pdnsd_proxy() {
                println!(
                    "  {} Cleared broken pdnsd proxy in {} (backup saved)",
                    "✔".green(),
                    pdnsd_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [182/199] pdnsd proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [182/199] pdnsd proxy: {}", "Clean or not set".dimmed());
    }

    // 183. Clean dead MaraDNS / Deadwood recursive DNS cache daemon in config / systemd
    if let Some((maradns_proxy, maradns_path)) = get_maradns_proxy() {
        if !verify_proxy_url(&maradns_proxy).await {
            println!(
                "  [183/199] Found dead MaraDNS / Deadwood proxy (`{}` in {}), cleaning with backup...",
                maradns_proxy.yellow(),
                maradns_path.display()
            );
            if maradns_path.exists() {
                let backup = maradns_path.with_extension("bak");
                let _ = std::fs::copy(&maradns_path, &backup);
            }
            if let Ok(true) = unset_maradns_proxy() {
                println!(
                    "  {} Cleared broken MaraDNS / Deadwood proxy in {} (backup saved)",
                    "✔".green(),
                    maradns_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [183/199] MaraDNS / Deadwood proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [183/199] MaraDNS / Deadwood proxy: {}", "Clean or not set".dimmed());
    }

    // 184. Clean dead i2pd anonymous network HTTP & SOCKS proxy daemon in config / systemd
    if let Some((i2pd_proxy, i2pd_path)) = get_i2pd_proxy() {
        if !verify_proxy_url(&i2pd_proxy).await {
            println!(
                "  [184/199] Found dead i2pd proxy (`{}` in {}), cleaning with backup...",
                i2pd_proxy.yellow(),
                i2pd_path.display()
            );
            if i2pd_path.exists() {
                let backup = i2pd_path.with_extension("bak");
                let _ = std::fs::copy(&i2pd_path, &backup);
            }
            if let Ok(true) = unset_i2pd_proxy() {
                println!(
                    "  {} Cleared broken i2pd proxy in {} (backup saved)",
                    "✔".green(),
                    i2pd_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [184/199] i2pd proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [184/199] i2pd proxy: {}", "Clean or not set".dimmed());
    }

    // 185. Clean dead dae eBPF transparent proxy daemon in config / systemd
    if let Some((dae_proxy, dae_path)) = get_dae_proxy() {
        if !verify_proxy_url(&dae_proxy).await {
            println!(
                "  [185/199] Found dead dae proxy (`{}` in {}), cleaning with backup...",
                dae_proxy.yellow(),
                dae_path.display()
            );
            if dae_path.exists() {
                let backup = dae_path.with_extension("bak");
                let _ = std::fs::copy(&dae_path, &backup);
            }
            if let Ok(true) = unset_dae_proxy() {
                println!(
                    "  {} Cleared broken dae proxy in {} (backup saved)",
                    "✔".green(),
                    dae_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [185/199] dae proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [185/199] dae proxy: {}", "Clean or not set".dimmed());
    }

    // 186. Clean dead Yggdrasil mesh routing proxy daemon in config / systemd
    if let Some((ygg_proxy, ygg_path)) = get_yggdrasil_proxy() {
        if !verify_proxy_url(&ygg_proxy).await {
            println!(
                "  [186/199] Found dead Yggdrasil proxy (`{}` in {}), cleaning with backup...",
                ygg_proxy.yellow(),
                ygg_path.display()
            );
            if ygg_path.exists() {
                let backup = ygg_path.with_extension("bak");
                let _ = std::fs::copy(&ygg_path, &backup);
            }
            if let Ok(true) = unset_yggdrasil_proxy() {
                println!(
                    "  {} Cleared broken Yggdrasil proxy in {} (backup saved)",
                    "✔".green(),
                    ygg_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [186/199] Yggdrasil proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [186/199] Yggdrasil proxy: {}", "Clean or not set".dimmed());
    }

    // 187. Clean dead cjdns encrypted mesh routing daemon in config / systemd
    if let Some((cjdns_proxy, cjdns_path)) = get_cjdns_proxy() {
        if !verify_proxy_url(&cjdns_proxy).await {
            println!(
                "  [187/199] Found dead cjdns proxy (`{}` in {}), cleaning with backup...",
                cjdns_proxy.yellow(),
                cjdns_path.display()
            );
            if cjdns_path.exists() {
                let backup = cjdns_path.with_extension("bak");
                let _ = std::fs::copy(&cjdns_path, &backup);
            }
            if let Ok(true) = unset_cjdns_proxy() {
                println!(
                    "  {} Cleared broken cjdns proxy in {} (backup saved)",
                    "✔".green(),
                    cjdns_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [187/199] cjdns proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [187/199] cjdns proxy: {}", "Clean or not set".dimmed());
    }

    // 188. Clean dead tinc mesh VPN daemon in config / systemd
    if let Some((tinc_proxy, tinc_path)) = get_tinc_proxy() {
        if !verify_proxy_url(&tinc_proxy).await {
            println!(
                "  [188/199] Found dead tinc proxy (`{}` in {}), cleaning with backup...",
                tinc_proxy.yellow(),
                tinc_path.display()
            );
            if tinc_path.exists() {
                let backup = tinc_path.with_extension("bak");
                let _ = std::fs::copy(&tinc_path, &backup);
            }
            if let Ok(true) = unset_tinc_proxy() {
                println!(
                    "  {} Cleared broken tinc proxy in {} (backup saved)",
                    "✔".green(),
                    tinc_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [188/199] tinc proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [188/199] tinc proxy: {}", "Clean or not set".dimmed());
    }

    // 189. Clean dead OpenVPN secure tunneling daemon in config / systemd
    if let Some((openvpn_proxy, openvpn_path)) = get_openvpn_proxy() {
        if !verify_proxy_url(&openvpn_proxy).await {
            println!(
                "  [189/199] Found dead OpenVPN proxy (`{}` in {}), cleaning with backup...",
                openvpn_proxy.yellow(),
                openvpn_path.display()
            );
            if openvpn_path.exists() {
                let backup = openvpn_path.with_extension("bak");
                let _ = std::fs::copy(&openvpn_path, &backup);
            }
            if let Ok(true) = unset_openvpn_proxy() {
                println!(
                    "  {} Cleared broken OpenVPN proxy in {} (backup saved)",
                    "✔".green(),
                    openvpn_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [189/199] OpenVPN proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [189/199] OpenVPN proxy: {}", "Clean or not set".dimmed());
    }

    // 190. Clean dead SoftEther multi-protocol VPN daemon in config / systemd
    if let Some((softether_proxy, softether_path)) = get_softether_proxy() {
        if !verify_proxy_url(&softether_proxy).await {
            println!(
                "  [190/199] Found dead SoftEther proxy (`{}` in {}), cleaning with backup...",
                softether_proxy.yellow(),
                softether_path.display()
            );
            if softether_path.exists() {
                let backup = softether_path.with_extension("bak");
                let _ = std::fs::copy(&softether_path, &backup);
            }
            if let Ok(true) = unset_softether_proxy() {
                println!(
                    "  {} Cleared broken SoftEther proxy in {} (backup saved)",
                    "✔".green(),
                    softether_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [190/199] SoftEther proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [190/199] SoftEther proxy: {}", "Clean or not set".dimmed());
    }

    // 191. Clean dead OpenConnect SSL VPN client daemon in config / systemd
    if let Some((openconnect_proxy, openconnect_path)) = get_openconnect_proxy() {
        if !verify_proxy_url(&openconnect_proxy).await {
            println!(
                "  [191/199] Found dead OpenConnect proxy (`{}` in {}), cleaning with backup...",
                openconnect_proxy.yellow(),
                openconnect_path.display()
            );
            if openconnect_path.exists() {
                let backup = openconnect_path.with_extension("bak");
                let _ = std::fs::copy(&openconnect_path, &backup);
            }
            if let Ok(true) = unset_openconnect_proxy() {
                println!(
                    "  {} Cleared broken OpenConnect proxy in {} (backup saved)",
                    "✔".green(),
                    openconnect_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [191/199] OpenConnect proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [191/199] OpenConnect proxy: {}", "Clean or not set".dimmed());
    }

    // 192. Clean dead strongSwan IPsec IKEv2 daemon in config / systemd
    if let Some((strongswan_proxy, strongswan_path)) = get_strongswan_proxy() {
        if !verify_proxy_url(&strongswan_proxy).await {
            println!(
                "  [192/199] Found dead strongSwan proxy (`{}` in {}), cleaning with backup...",
                strongswan_proxy.yellow(),
                strongswan_path.display()
            );
            if strongswan_path.exists() {
                let backup = strongswan_path.with_extension("bak");
                let _ = std::fs::copy(&strongswan_path, &backup);
            }
            if let Ok(true) = unset_strongswan_proxy() {
                println!(
                    "  {} Cleared broken strongSwan proxy in {} (backup saved)",
                    "✔".green(),
                    strongswan_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [192/199] strongSwan proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [192/199] strongSwan proxy: {}", "Clean or not set".dimmed());
    }

    // 193. Clean dead WireGuard secure VPN tunnel daemon in config / systemd
    if let Some((wireguard_proxy, wireguard_path)) = get_wireguard_proxy() {
        if !verify_proxy_url(&wireguard_proxy).await {
            println!(
                "  [193/205] Found dead WireGuard proxy (`{}` in {}), cleaning with backup...",
                wireguard_proxy.yellow(),
                wireguard_path.display()
            );
            if wireguard_path.exists() {
                let backup = wireguard_path.with_extension("bak");
                let _ = std::fs::copy(&wireguard_path, &backup);
            }
            if let Ok(true) = unset_wireguard_proxy() {
                println!(
                    "  {} Cleared broken WireGuard proxy in {} (backup saved)",
                    "✔".green(),
                    wireguard_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [193/205] WireGuard proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [193/205] WireGuard proxy: {}", "Clean or not set".dimmed());
    }

    // 194. Clean dead Headscale Tailscale control server daemon in config / systemd
    if let Some((headscale_proxy, headscale_path)) = get_headscale_proxy() {
        if !verify_proxy_url(&headscale_proxy).await {
            println!(
                "  [194/205] Found dead Headscale proxy (`{}` in {}), cleaning with backup...",
                headscale_proxy.yellow(),
                headscale_path.display()
            );
            if headscale_path.exists() {
                let backup = headscale_path.with_extension("bak");
                let _ = std::fs::copy(&headscale_path, &backup);
            }
            if let Ok(true) = unset_headscale_proxy() {
                println!(
                    "  {} Cleared broken Headscale proxy in {} (backup saved)",
                    "✔".green(),
                    headscale_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [194/205] Headscale proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [194/205] Headscale proxy: {}", "Clean or not set".dimmed());
    }

    // 195. Clean dead NetBird Zero Trust Mesh VPN daemon in config / systemd
    if let Some((netbird_proxy, netbird_path)) = get_netbird_proxy() {
        if !verify_proxy_url(&netbird_proxy).await {
            println!(
                "  [195/205] Found dead NetBird proxy (`{}` in {}), cleaning with backup...",
                netbird_proxy.yellow(),
                netbird_path.display()
            );
            if netbird_path.exists() {
                let backup = netbird_path.with_extension("bak");
                let _ = std::fs::copy(&netbird_path, &backup);
            }
            if let Ok(true) = unset_netbird_proxy() {
                println!(
                    "  {} Cleared broken NetBird proxy in {} (backup saved)",
                    "✔".green(),
                    netbird_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [195/205] NetBird proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [195/205] NetBird proxy: {}", "Clean or not set".dimmed());
    }

    // 196. Clean dead Cloudflare WARP client daemon in config / systemd
    if let Some((warp_proxy, warp_path)) = get_warp_proxy() {
        if !verify_proxy_url(&warp_proxy).await {
            println!(
                "  [196/205] Found dead Cloudflare WARP proxy (`{}` in {}), cleaning with backup...",
                warp_proxy.yellow(),
                warp_path.display()
            );
            if warp_path.exists() {
                let backup = warp_path.with_extension("bak");
                let _ = std::fs::copy(&warp_path, &backup);
            }
            if let Ok(true) = unset_warp_proxy() {
                println!(
                    "  {} Cleared broken Cloudflare WARP proxy in {} (backup saved)",
                    "✔".green(),
                    warp_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [196/205] Cloudflare WARP proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [196/205] Cloudflare WARP proxy: {}", "Clean or not set".dimmed());
    }

    // 197. Clean dead Amnezia VPN daemon in config / systemd
    if let Some((amnezia_proxy, amnezia_path)) = get_amnezia_proxy() {
        if !verify_proxy_url(&amnezia_proxy).await {
            println!(
                "  [197/205] Found dead Amnezia VPN proxy (`{}` in {}), cleaning with backup...",
                amnezia_proxy.yellow(),
                amnezia_path.display()
            );
            if amnezia_path.exists() {
                let backup = amnezia_path.with_extension("bak");
                let _ = std::fs::copy(&amnezia_path, &backup);
            }
            if let Ok(true) = unset_amnezia_proxy() {
                println!(
                    "  {} Cleared broken Amnezia VPN proxy in {} (backup saved)",
                    "✔".green(),
                    amnezia_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [197/205] Amnezia VPN proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [197/205] Amnezia VPN proxy: {}", "Clean or not set".dimmed());
    }

    // 198. Clean dead OpenZiti Edge Tunnel & Overlay daemon in config / systemd
    if let Some((openziti_proxy, openziti_path)) = get_openziti_proxy() {
        if !verify_proxy_url(&openziti_proxy).await {
            println!(
                "  [198/205] Found dead OpenZiti proxy (`{}` in {}), cleaning with backup...",
                openziti_proxy.yellow(),
                openziti_path.display()
            );
            if openziti_path.exists() {
                let backup = openziti_path.with_extension("bak");
                let _ = std::fs::copy(&openziti_path, &backup);
            }
            if let Ok(true) = unset_openziti_proxy() {
                println!(
                    "  {} Cleared broken OpenZiti proxy in {} (backup saved)",
                    "✔".green(),
                    openziti_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [198/205] OpenZiti proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [198/205] OpenZiti proxy: {}", "Clean or not set".dimmed());
    }

    // 199. Clean dead Firezone Zero Trust Mesh & WireGuard daemon in config / systemd
    if let Some((firezone_proxy, firezone_path)) = get_firezone_proxy() {
        if !verify_proxy_url(&firezone_proxy).await {
            println!(
                "  [199/207] Found dead Firezone proxy (`{}` in {}), cleaning with backup...",
                firezone_proxy.yellow(),
                firezone_path.display()
            );
            if firezone_path.exists() {
                let backup = firezone_path.with_extension("bak");
                let _ = std::fs::copy(&firezone_path, &backup);
            }
            if let Ok(true) = unset_firezone_proxy() {
                println!(
                    "  {} Cleared broken Firezone proxy in {} (backup saved)",
                    "✔".green(),
                    firezone_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [199/207] Firezone proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [199/207] Firezone proxy: {}", "Clean or not set".dimmed());
    }

    // 200. Clean dead Netmaker WireGuard Mesh daemon in config / systemd
    if let Some((netmaker_proxy, netmaker_path)) = get_netmaker_proxy() {
        if !verify_proxy_url(&netmaker_proxy).await {
            println!(
                "  [200/207] Found dead Netmaker proxy (`{}` in {}), cleaning with backup...",
                netmaker_proxy.yellow(),
                netmaker_path.display()
            );
            if netmaker_path.exists() {
                let backup = netmaker_path.with_extension("bak");
                let _ = std::fs::copy(&netmaker_path, &backup);
            }
            if let Ok(true) = unset_netmaker_proxy() {
                println!(
                    "  {} Cleared broken Netmaker proxy in {} (backup saved)",
                    "✔".green(),
                    netmaker_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [200/207] Netmaker proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [200/207] Netmaker proxy: {}", "Clean or not set".dimmed());
    }

    // 201. Clean dead Pritunl Zero Trust & Enterprise VPN daemon in config / systemd
    if let Some((pritunl_proxy, pritunl_path)) = get_pritunl_proxy() {
        if !verify_proxy_url(&pritunl_proxy).await {
            println!(
                "  [201/213] Found dead Pritunl proxy (`{}` in {}), cleaning with backup...",
                pritunl_proxy.yellow(),
                pritunl_path.display()
            );
            if pritunl_path.exists() {
                let backup = pritunl_path.with_extension("bak");
                let _ = std::fs::copy(&pritunl_path, &backup);
            }
            if let Ok(true) = unset_pritunl_proxy() {
                println!(
                    "  {} Cleared broken Pritunl proxy in {} (backup saved)",
                    "✔".green(),
                    pritunl_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [201/213] Pritunl proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [201/213] Pritunl proxy: {}", "Clean or not set".dimmed());
    }

    // 202. Clean dead Twingate Zero Trust Network Access connector & client daemon in config / systemd
    if let Some((twingate_proxy, twingate_path)) = get_twingate_proxy() {
        if !verify_proxy_url(&twingate_proxy).await {
            println!(
                "  [202/213] Found dead Twingate proxy (`{}` in {}), cleaning with backup...",
                twingate_proxy.yellow(),
                twingate_path.display()
            );
            if twingate_path.exists() {
                let backup = twingate_path.with_extension("bak");
                let _ = std::fs::copy(&twingate_path, &backup);
            }
            if let Ok(true) = unset_twingate_proxy() {
                println!(
                    "  {} Cleared broken Twingate proxy in {} (backup saved)",
                    "✔".green(),
                    twingate_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [202/213] Twingate proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [202/213] Twingate proxy: {}", "Clean or not set".dimmed());
    }

    // 203. Clean dead EasyTier Decentralized Mesh VPN daemon in config / systemd
    if let Some((easytier_proxy, easytier_path)) = get_easytier_proxy() {
        if !verify_proxy_url(&easytier_proxy).await {
            println!(
                "  [203/213] Found dead EasyTier proxy (`{}` in {}), cleaning with backup...",
                easytier_proxy.yellow(),
                easytier_path.display()
            );
            if easytier_path.exists() {
                let backup = easytier_path.with_extension("bak");
                let _ = std::fs::copy(&easytier_path, &backup);
            }
            if let Ok(true) = unset_easytier_proxy() {
                println!(
                    "  {} Cleared broken EasyTier proxy in {} (backup saved)",
                    "✔".green(),
                    easytier_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [203/213] EasyTier proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [203/213] EasyTier proxy: {}", "Clean or not set".dimmed());
    }

    // 204. Clean dead Teleport Access & Zero Trust Proxy daemon in config / systemd
    if let Some((teleport_proxy, teleport_path)) = get_teleport_proxy() {
        if !verify_proxy_url(&teleport_proxy).await {
            println!(
                "  [204/213] Found dead Teleport proxy (`{}` in {}), cleaning with backup...",
                teleport_proxy.yellow(),
                teleport_path.display()
            );
            if teleport_path.exists() {
                let backup = teleport_path.with_extension("bak");
                let _ = std::fs::copy(&teleport_path, &backup);
            }
            if let Ok(true) = unset_teleport_proxy() {
                println!(
                    "  {} Cleared broken Teleport proxy in {} (backup saved)",
                    "✔".green(),
                    teleport_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [204/213] Teleport proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [204/213] Teleport proxy: {}", "Clean or not set".dimmed());
    }

    // 205. Clean dead Pomerium Identity-Aware & Zero Trust Access Proxy daemon in config / systemd
    if let Some((pomerium_proxy, pomerium_path)) = get_pomerium_proxy() {
        if !verify_proxy_url(&pomerium_proxy).await {
            println!(
                "  [205/213] Found dead Pomerium proxy (`{}` in {}), cleaning with backup...",
                pomerium_proxy.yellow(),
                pomerium_path.display()
            );
            if pomerium_path.exists() {
                let backup = pomerium_path.with_extension("bak");
                let _ = std::fs::copy(&pomerium_path, &backup);
            }
            if let Ok(true) = unset_pomerium_proxy() {
                println!(
                    "  {} Cleared broken Pomerium proxy in {} (backup saved)",
                    "✔".green(),
                    pomerium_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [205/213] Pomerium proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [205/213] Pomerium proxy: {}", "Clean or not set".dimmed());
    }

    // 206. Clean dead Innernet Mesh VPN & WireGuard Overlay daemon in config / systemd
    if let Some((innernet_proxy, innernet_path)) = get_innernet_proxy() {
        if !verify_proxy_url(&innernet_proxy).await {
            println!(
                "  [206/213] Found dead Innernet proxy (`{}` in {}), cleaning with backup...",
                innernet_proxy.yellow(),
                innernet_path.display()
            );
            if innernet_path.exists() {
                let backup = innernet_path.with_extension("bak");
                let _ = std::fs::copy(&innernet_path, &backup);
            }
            if let Ok(true) = unset_innernet_proxy() {
                println!(
                    "  {} Cleared broken Innernet proxy in {} (backup saved)",
                    "✔".green(),
                    innernet_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [206/213] Innernet proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [206/213] Innernet proxy: {}", "Clean or not set".dimmed());
    }

    // 207. Clean dead Defguard Enterprise WireGuard 2FA & SSO VPN Gateway daemon in config / systemd
    if let Some((defguard_proxy, defguard_path)) = get_defguard_proxy() {
        if !verify_proxy_url(&defguard_proxy).await {
            println!(
                "  [207/223] Found dead Defguard proxy (`{}` in {}), cleaning with backup...",
                defguard_proxy.yellow(),
                defguard_path.display()
            );
            if defguard_path.exists() {
                let backup = defguard_path.with_extension("bak");
                let _ = std::fs::copy(&defguard_path, &backup);
            }
            if let Ok(true) = unset_defguard_proxy() {
                println!(
                    "  {} Cleared broken Defguard proxy in {} (backup saved)",
                    "✔".green(),
                    defguard_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [207/223] Defguard proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [207/223] Defguard proxy: {}", "Clean or not set".dimmed());
    }

    // 208. Clean dead Inlets Cloud-Native Tunnel & Reverse Proxy daemon in config / systemd
    if let Some((inlets_proxy, inlets_path)) = get_inlets_proxy() {
        if !verify_proxy_url(&inlets_proxy).await {
            println!(
                "  [208/223] Found dead Inlets proxy (`{}` in {}), cleaning with backup...",
                inlets_proxy.yellow(),
                inlets_path.display()
            );
            if inlets_path.exists() {
                let backup = inlets_path.with_extension("bak");
                let _ = std::fs::copy(&inlets_path, &backup);
            }
            if let Ok(true) = unset_inlets_proxy() {
                println!(
                    "  {} Cleared broken Inlets proxy in {} (backup saved)",
                    "✔".green(),
                    inlets_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [208/223] Inlets proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [208/223] Inlets proxy: {}", "Clean or not set".dimmed());
    }

    // 209. Clean dead zrok Peer-to-Peer Sharing & Zero Trust Tunnel daemon in config / systemd
    if let Some((zrok_proxy, zrok_path)) = get_zrok_proxy() {
        if !verify_proxy_url(&zrok_proxy).await {
            println!(
                "  [209/223] Found dead zrok proxy (`{}` in {}), cleaning with backup...",
                zrok_proxy.yellow(),
                zrok_path.display()
            );
            if zrok_path.exists() {
                let backup = zrok_path.with_extension("bak");
                let _ = std::fs::copy(&zrok_path, &backup);
            }
            if let Ok(true) = unset_zrok_proxy() {
                println!(
                    "  {} Cleared broken zrok proxy in {} (backup saved)",
                    "✔".green(),
                    zrok_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [209/223] zrok proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [209/223] zrok proxy: {}", "Clean or not set".dimmed());
    }

    // 210. Clean dead n2n Peer-to-Peer Mesh VPN & Supernode daemon in config / systemd
    if let Some((n2n_proxy, n2n_path)) = get_n2n_proxy() {
        if !verify_proxy_url(&n2n_proxy).await {
            println!(
                "  [210/223] Found dead n2n proxy (`{}` in {}), cleaning with backup...",
                n2n_proxy.yellow(),
                n2n_path.display()
            );
            if n2n_path.exists() {
                let backup = n2n_path.with_extension("bak");
                let _ = std::fs::copy(&n2n_path, &backup);
            }
            if let Ok(true) = unset_n2n_proxy() {
                println!(
                    "  {} Cleared broken n2n proxy in {} (backup saved)",
                    "✔".green(),
                    n2n_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [210/223] n2n proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [210/223] n2n proxy: {}", "Clean or not set".dimmed());
    }

    // 211. Clean dead Hyprspace Peer-to-Peer Mesh VPN daemon in config / systemd
    if let Some((hyprspace_proxy, hyprspace_path)) = get_hyprspace_proxy() {
        if !verify_proxy_url(&hyprspace_proxy).await {
            println!(
                "  [211/223] Found dead Hyprspace proxy (`{}` in {}), cleaning with backup...",
                hyprspace_proxy.yellow(),
                hyprspace_path.display()
            );
            if hyprspace_path.exists() {
                let backup = hyprspace_path.with_extension("bak");
                let _ = std::fs::copy(&hyprspace_path, &backup);
            }
            if let Ok(true) = unset_hyprspace_proxy() {
                println!(
                    "  {} Cleared broken Hyprspace proxy in {} (backup saved)",
                    "✔".green(),
                    hyprspace_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [211/223] Hyprspace proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [211/223] Hyprspace proxy: {}", "Clean or not set".dimmed());
    }

    // 212. Clean dead Fastd Secure Tunneling & Mesh VPN daemon in config / systemd
    if let Some((fastd_proxy, fastd_path)) = get_fastd_proxy() {
        if !verify_proxy_url(&fastd_proxy).await {
            println!(
                "  [212/223] Found dead Fastd proxy (`{}` in {}), cleaning with backup...",
                fastd_proxy.yellow(),
                fastd_path.display()
            );
            if fastd_path.exists() {
                let backup = fastd_path.with_extension("bak");
                let _ = std::fs::copy(&fastd_path, &backup);
            }
            if let Ok(true) = unset_fastd_proxy() {
                println!(
                    "  {} Cleared broken Fastd proxy in {} (backup saved)",
                    "✔".green(),
                    fastd_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [212/223] Fastd proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [212/223] Fastd proxy: {}", "Clean or not set".dimmed());
    }

    // 213. Clean dead EdgeVPN Decentralized Immutable P2P Mesh VPN daemon in config / systemd
    if let Some((edgevpn_proxy, edgevpn_path)) = get_edgevpn_proxy() {
        if !verify_proxy_url(&edgevpn_proxy).await {
            println!(
                "  [213/223] Found dead EdgeVPN proxy (`{}` in {}), cleaning with backup...",
                edgevpn_proxy.yellow(),
                edgevpn_path.display()
            );
            if edgevpn_path.exists() {
                let backup = edgevpn_path.with_extension("bak");
                let _ = std::fs::copy(&edgevpn_path, &backup);
            }
            if let Ok(true) = unset_edgevpn_proxy() {
                println!(
                    "  {} Cleared broken EdgeVPN proxy in {} (backup saved)",
                    "✔".green(),
                    edgevpn_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [213/223] EdgeVPN proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [213/223] EdgeVPN proxy: {}", "Clean or not set".dimmed());
    }

    // 214. Clean dead boringproxy Reverse Proxy & Automated Tunnel daemon in config / systemd
    if let Some((boringproxy_proxy, boringproxy_path)) = get_boringproxy_proxy() {
        if !verify_proxy_url(&boringproxy_proxy).await {
            println!(
                "  [214/223] Found dead boringproxy proxy (`{}` in {}), cleaning with backup...",
                boringproxy_proxy.yellow(),
                boringproxy_path.display()
            );
            if boringproxy_path.exists() {
                let backup = boringproxy_path.with_extension("bak");
                let _ = std::fs::copy(&boringproxy_path, &backup);
            }
            if let Ok(true) = unset_boringproxy_proxy() {
                println!(
                    "  {} Cleared broken boringproxy proxy in {} (backup saved)",
                    "✔".green(),
                    boringproxy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [214/223] boringproxy proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [214/223] boringproxy proxy: {}", "Clean or not set".dimmed());
    }

    // 215. Clean dead Ghostunnel Mutual TLS proxy daemon in config / systemd
    if let Some((ghostunnel_proxy, ghostunnel_path)) = get_ghostunnel_proxy() {
        if !verify_proxy_url(&ghostunnel_proxy).await {
            println!(
                "  [215/223] Found dead Ghostunnel proxy (`{}` in {}), cleaning with backup...",
                ghostunnel_proxy.yellow(),
                ghostunnel_path.display()
            );
            if ghostunnel_path.exists() {
                let backup = ghostunnel_path.with_extension("bak");
                let _ = std::fs::copy(&ghostunnel_path, &backup);
            }
            if let Ok(true) = unset_ghostunnel_proxy() {
                println!(
                    "  {} Cleared broken Ghostunnel proxy in {} (backup saved)",
                    "✔".green(),
                    ghostunnel_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [215/223] Ghostunnel proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [215/223] Ghostunnel proxy: {}", "Clean or not set".dimmed());
    }

    // 216. Clean dead FreeLAN Peer-to-Peer Mesh VPN daemon in config / systemd
    if let Some((freelan_proxy, freelan_path)) = get_freelan_proxy() {
        if !verify_proxy_url(&freelan_proxy).await {
            println!(
                "  [216/223] Found dead FreeLAN proxy (`{}` in {}), cleaning with backup...",
                freelan_proxy.yellow(),
                freelan_path.display()
            );
            if freelan_path.exists() {
                let backup = freelan_path.with_extension("bak");
                let _ = std::fs::copy(&freelan_path, &backup);
            }
            if let Ok(true) = unset_freelan_proxy() {
                println!(
                    "  {} Cleared broken FreeLAN proxy in {} (backup saved)",
                    "✔".green(),
                    freelan_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [216/223] FreeLAN proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [216/223] FreeLAN proxy: {}", "Clean or not set".dimmed());
    }

    // 217. Clean dead OmniEdge Decentralized Mesh VPN daemon in config / systemd
    if let Some((omniedge_proxy, omniedge_path)) = get_omniedge_proxy() {
        if !verify_proxy_url(&omniedge_proxy).await {
            println!(
                "  [217/225] Found dead OmniEdge proxy (`{}` in {}), cleaning with backup...",
                omniedge_proxy.yellow(),
                omniedge_path.display()
            );
            if omniedge_path.exists() {
                let backup = omniedge_path.with_extension("bak");
                let _ = std::fs::copy(&omniedge_path, &backup);
            }
            if let Ok(true) = unset_omniedge_proxy() {
                println!(
                    "  {} Cleared broken OmniEdge proxy in {} (backup saved)",
                    "✔".green(),
                    omniedge_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [217/225] OmniEdge proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [217/225] OmniEdge proxy: {}", "Clean or not set".dimmed());
    }

    // 218. Clean dead Lantern Censorship Circumvention proxy daemon in config / systemd
    if let Some((lantern_proxy, lantern_path)) = get_lantern_proxy() {
        if !verify_proxy_url(&lantern_proxy).await {
            println!(
                "  [218/225] Found dead Lantern proxy (`{}` in {}), cleaning with backup...",
                lantern_proxy.yellow(),
                lantern_path.display()
            );
            if lantern_path.exists() {
                let backup = lantern_path.with_extension("bak");
                let _ = std::fs::copy(&lantern_path, &backup);
            }
            if let Ok(true) = unset_lantern_proxy() {
                println!(
                    "  {} Cleared broken Lantern proxy in {} (backup saved)",
                    "✔".green(),
                    lantern_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [218/225] Lantern proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [218/225] Lantern proxy: {}", "Clean or not set".dimmed());
    }

    // 219. Clean dead TunSafe WireGuard VPN daemon in config / systemd
    if let Some((tunsafe_proxy, tunsafe_path)) = get_tunsafe_proxy() {
        if !verify_proxy_url(&tunsafe_proxy).await {
            println!(
                "  [219/225] Found dead TunSafe proxy (`{}` in {}), cleaning with backup...",
                tunsafe_proxy.yellow(),
                tunsafe_path.display()
            );
            if tunsafe_path.exists() {
                let backup = tunsafe_path.with_extension("bak");
                let _ = std::fs::copy(&tunsafe_path, &backup);
            }
            if let Ok(true) = unset_tunsafe_proxy() {
                println!(
                    "  {} Cleared broken TunSafe proxy in {} (backup saved)",
                    "✔".green(),
                    tunsafe_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [219/225] TunSafe proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [219/225] TunSafe proxy: {}", "Clean or not set".dimmed());
    }

    // 220. Clean dead Sshuttle Transparent SSH proxy daemon in config / systemd
    if let Some((sshuttle_proxy, sshuttle_path)) = get_sshuttle_proxy() {
        if !verify_proxy_url(&sshuttle_proxy).await {
            println!(
                "  [220/227] Found dead Sshuttle proxy (`{}` in {}), cleaning with backup...",
                sshuttle_proxy.yellow(),
                sshuttle_path.display()
            );
            if sshuttle_path.exists() {
                let backup = sshuttle_path.with_extension("bak");
                let _ = std::fs::copy(&sshuttle_path, &backup);
            }
            if let Ok(true) = unset_sshuttle_proxy() {
                println!(
                    "  {} Cleared broken Sshuttle proxy in {} (backup saved)",
                    "✔".green(),
                    sshuttle_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [220/227] Sshuttle proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [220/227] Sshuttle proxy: {}", "Clean or not set".dimmed());
    }

    // 221. Clean dead Psiphon Censorship Circumvention daemon in config / systemd
    if let Some((psiphon_proxy, psiphon_path)) = get_psiphon_proxy() {
        if !verify_proxy_url(&psiphon_proxy).await {
            println!(
                "  [221/229] Found dead Psiphon proxy (`{}` in {}), cleaning with backup...",
                psiphon_proxy.yellow(),
                psiphon_path.display()
            );
            if psiphon_path.exists() {
                let backup = psiphon_path.with_extension("bak");
                let _ = std::fs::copy(&psiphon_path, &backup);
            }
            if let Ok(true) = unset_psiphon_proxy() {
                println!(
                    "  {} Cleared broken Psiphon proxy in {} (backup saved)",
                    "✔".green(),
                    psiphon_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [221/229] Psiphon proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [221/229] Psiphon proxy: {}", "Clean or not set".dimmed());
    }

    // 222. Clean dead Localtunnel Localhost Reverse Tunnel daemon in config / systemd
    if let Some((localtunnel_proxy, localtunnel_path)) = get_localtunnel_proxy() {
        if !verify_proxy_url(&localtunnel_proxy).await {
            println!(
                "  [222/229] Found dead Localtunnel proxy (`{}` in {}), cleaning with backup...",
                localtunnel_proxy.yellow(),
                localtunnel_path.display()
            );
            if localtunnel_path.exists() {
                let backup = localtunnel_path.with_extension("bak");
                let _ = std::fs::copy(&localtunnel_path, &backup);
            }
            if let Ok(true) = unset_localtunnel_proxy() {
                println!(
                    "  {} Cleared broken Localtunnel proxy in {} (backup saved)",
                    "✔".green(),
                    localtunnel_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [222/229] Localtunnel proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [222/229] Localtunnel proxy: {}", "Clean or not set".dimmed());
    }

    // 223. Clean dead Pagekite Localhost Tunnel & Reverse Proxy daemon in config / systemd
    if let Some((pagekite_proxy, pagekite_path)) = get_pagekite_proxy() {
        if !verify_proxy_url(&pagekite_proxy).await {
            println!(
                "  [223/229] Found dead Pagekite proxy (`{}` in {}), cleaning with backup...",
                pagekite_proxy.yellow(),
                pagekite_path.display()
            );
            if pagekite_path.exists() {
                let backup = pagekite_path.with_extension("bak");
                let _ = std::fs::copy(&pagekite_path, &backup);
            }
            if let Ok(true) = unset_pagekite_proxy() {
                println!(
                    "  {} Cleared broken Pagekite proxy in {} (backup saved)",
                    "✔".green(),
                    pagekite_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [223/229] Pagekite proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [223/229] Pagekite proxy: {}", "Clean or not set".dimmed());
    }

    // 224. Clean dead Toxiproxy Resiliency & Fault Injection daemon in config / systemd
    if let Some((toxiproxy_proxy, toxiproxy_path)) = get_toxiproxy_proxy() {
        if !verify_proxy_url(&toxiproxy_proxy).await {
            println!(
                "  [224/229] Found dead Toxiproxy proxy (`{}` in {}), cleaning with backup...",
                toxiproxy_proxy.yellow(),
                toxiproxy_path.display()
            );
            if toxiproxy_path.exists() {
                let backup = toxiproxy_path.with_extension("bak");
                let _ = std::fs::copy(&toxiproxy_path, &backup);
            }
            if let Ok(true) = unset_toxiproxy_proxy() {
                println!(
                    "  {} Cleared broken Toxiproxy proxy in {} (backup saved)",
                    "✔".green(),
                    toxiproxy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [224/233] Toxiproxy proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [224/233] Toxiproxy proxy: {}", "Clean or not set".dimmed());
    }

    // 225. Clean dead Telepresence Kubernetes Local Proxy & Intercept daemon in config / systemd
    if let Some((telepresence_proxy, telepresence_path)) = get_telepresence_proxy() {
        if !verify_proxy_url(&telepresence_proxy).await {
            println!(
                "  [225/233] Found dead Telepresence proxy (`{}` in {}), cleaning with backup...",
                telepresence_proxy.yellow(),
                telepresence_path.display()
            );
            if telepresence_path.exists() {
                let backup = telepresence_path.with_extension("bak");
                let _ = std::fs::copy(&telepresence_path, &backup);
            }
            if let Ok(true) = unset_telepresence_proxy() {
                println!(
                    "  {} Cleared broken Telepresence proxy in {} (backup saved)",
                    "✔".green(),
                    telepresence_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [225/233] Telepresence proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [225/233] Telepresence proxy: {}", "Clean or not set".dimmed());
    }

    // 226. Clean dead LocalXpose Localhost Reverse Proxy & Tunnel daemon in config / systemd
    if let Some((localxpose_proxy, localxpose_path)) = get_localxpose_proxy() {
        if !verify_proxy_url(&localxpose_proxy).await {
            println!(
                "  [226/233] Found dead LocalXpose proxy (`{}` in {}), cleaning with backup...",
                localxpose_proxy.yellow(),
                localxpose_path.display()
            );
            if localxpose_path.exists() {
                let backup = localxpose_path.with_extension("bak");
                let _ = std::fs::copy(&localxpose_path, &backup);
            }
            if let Ok(true) = unset_localxpose_proxy() {
                println!(
                    "  {} Cleared broken LocalXpose proxy in {} (backup saved)",
                    "✔".green(),
                    localxpose_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [226/233] LocalXpose proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [226/233] LocalXpose proxy: {}", "Clean or not set".dimmed());
    }

    // 227. Clean dead Packetriot Localhost Reverse Tunnel & Proxy daemon in config / systemd
    if let Some((packetriot_proxy, packetriot_path)) = get_packetriot_proxy() {
        if !verify_proxy_url(&packetriot_proxy).await {
            println!(
                "  [227/263] Found dead Packetriot proxy (`{}` in {}), cleaning with backup...",
                packetriot_proxy.yellow(),
                packetriot_path.display()
            );
            if packetriot_path.exists() {
                let backup = packetriot_path.with_extension("bak");
                let _ = std::fs::copy(&packetriot_path, &backup);
            }
            if let Ok(true) = unset_packetriot_proxy() {
                println!(
                    "  {} Cleared broken Packetriot proxy in {} (backup saved)",
                    "✔".green(),
                    packetriot_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [227/263] Packetriot proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [227/263] Packetriot proxy: {}", "Clean or not set".dimmed());
    }

    // 228. Clean dead sish HTTP/TCP Localhost Tunnel & Reverse Proxy daemon in config / systemd
    if let Some((sish_proxy, sish_path)) = get_sish_proxy() {
        if !verify_proxy_url(&sish_proxy).await {
            println!(
                "  [228/263] Found dead sish proxy (`{}` in {}), cleaning with backup...",
                sish_proxy.yellow(),
                sish_path.display()
            );
            if sish_path.exists() {
                let backup = sish_path.with_extension("bak");
                let _ = std::fs::copy(&sish_path, &backup);
            }
            if let Ok(true) = unset_sish_proxy() {
                println!(
                    "  {} Cleared broken sish proxy in {} (backup saved)",
                    "✔".green(),
                    sish_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [228/263] sish proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [228/263] sish proxy: {}", "Clean or not set".dimmed());
    }

    // 229. Clean dead Pinggy HTTP/TCP Localhost Tunnel & Reverse Proxy daemon in config / systemd
    if let Some((pinggy_proxy, pinggy_path)) = get_pinggy_proxy() {
        if !verify_proxy_url(&pinggy_proxy).await {
            println!(
                "  [229/263] Found dead Pinggy proxy (`{}` in {}), cleaning with backup...",
                pinggy_proxy.yellow(),
                pinggy_path.display()
            );
            if pinggy_path.exists() {
                let backup = pinggy_path.with_extension("bak");
                let _ = std::fs::copy(&pinggy_path, &backup);
            }
            if let Ok(true) = unset_pinggy_proxy() {
                println!(
                    "  {} Cleared broken Pinggy proxy in {} (backup saved)",
                    "✔".green(),
                    pinggy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [229/263] Pinggy proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [229/263] Pinggy proxy: {}", "Clean or not set".dimmed());
    }

    // 230. Clean dead Loophole E2EE Localhost Reverse Proxy & Tunnel daemon in config / systemd
    if let Some((loophole_proxy, loophole_path)) = get_loophole_proxy() {
        if !verify_proxy_url(&loophole_proxy).await {
            println!(
                "  [230/263] Found dead Loophole proxy (`{}` in {}), cleaning with backup...",
                loophole_proxy.yellow(),
                loophole_path.display()
            );
            if loophole_path.exists() {
                let backup = loophole_path.with_extension("bak");
                let _ = std::fs::copy(&loophole_path, &backup);
            }
            if let Ok(true) = unset_loophole_proxy() {
                println!(
                    "  {} Cleared broken Loophole proxy in {} (backup saved)",
                    "✔".green(),
                    loophole_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [230/263] Loophole proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [230/263] Loophole proxy: {}", "Clean or not set".dimmed());
    }

    // 231. Clean dead Tunnelmole Localhost Reverse Proxy & Tunnel daemon in config / systemd
    if let Some((tunnelmole_proxy, tunnelmole_path)) = get_tunnelmole_proxy() {
        if !verify_proxy_url(&tunnelmole_proxy).await {
            println!(
                "  [231/263] Found dead Tunnelmole proxy (`{}` in {}), cleaning with backup...",
                tunnelmole_proxy.yellow(),
                tunnelmole_path.display()
            );
            if tunnelmole_path.exists() {
                let backup = tunnelmole_path.with_extension("bak");
                let _ = std::fs::copy(&tunnelmole_path, &backup);
            }
            if let Ok(true) = unset_tunnelmole_proxy() {
                println!(
                    "  {} Cleared broken Tunnelmole proxy in {} (backup saved)",
                    "✔".green(),
                    tunnelmole_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [231/263] Tunnelmole proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [231/263] Tunnelmole proxy: {}", "Clean or not set".dimmed());
    }

    // 232. Clean dead Expose Localhost Reverse Proxy & Tunnel daemon in config / systemd
    if let Some((expose_proxy, expose_path)) = get_expose_proxy() {
        if !verify_proxy_url(&expose_proxy).await {
            println!(
                "  [232/263] Found dead Expose proxy (`{}` in {}), cleaning with backup...",
                expose_proxy.yellow(),
                expose_path.display()
            );
            if expose_path.exists() {
                let backup = expose_path.with_extension("bak");
                let _ = std::fs::copy(&expose_path, &backup);
            }
            if let Ok(true) = unset_expose_proxy() {
                println!(
                    "  {} Cleared broken Expose proxy in {} (backup saved)",
                    "✔".green(),
                    expose_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [232/263] Expose proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [232/263] Expose proxy: {}", "Clean or not set".dimmed());
    }

    // 233. Clean dead Playit Localhost Reverse Proxy & Tunnel daemon in config / systemd
    if let Some((playit_proxy, playit_path)) = get_playit_proxy() {
        if !verify_proxy_url(&playit_proxy).await {
            println!(
                "  [233/263] Found dead Playit proxy (`{}` in {}), cleaning with backup...",
                playit_proxy.yellow(),
                playit_path.display()
            );
            if playit_path.exists() {
                let backup = playit_path.with_extension("bak");
                let _ = std::fs::copy(&playit_path, &backup);
            }
            if let Ok(true) = unset_playit_proxy() {
                println!(
                    "  {} Cleared broken Playit proxy in {} (backup saved)",
                    "✔".green(),
                    playit_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [233/263] Playit proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [233/263] Playit proxy: {}", "Clean or not set".dimmed());
    }

    // 234. Clean dead pgrok Localhost Reverse Proxy & Tunnel daemon in config / systemd
    if let Some((pgrok_proxy, pgrok_path)) = get_pgrok_proxy() {
        if !verify_proxy_url(&pgrok_proxy).await {
            println!(
                "  [234/263] Found dead pgrok proxy (`{}` in {}), cleaning with backup...",
                pgrok_proxy.yellow(),
                pgrok_path.display()
            );
            if pgrok_path.exists() {
                let backup = pgrok_path.with_extension("bak");
                let _ = std::fs::copy(&pgrok_path, &backup);
            }
            if let Ok(true) = unset_pgrok_proxy() {
                println!(
                    "  {} Cleared broken pgrok proxy in {} (backup saved)",
                    "✔".green(),
                    pgrok_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [234/263] pgrok proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [234/263] pgrok proxy: {}", "Clean or not set".dimmed());
    }

    // 235. Clean dead cpolar Localhost Reverse Tunnel & Web UI daemon in config / systemd
    if let Some((cpolar_proxy, cpolar_path)) = get_cpolar_proxy() {
        if !verify_proxy_url(&cpolar_proxy).await {
            println!(
                "  [235/263] Found dead cpolar proxy (`{}` in {}), cleaning with backup...",
                cpolar_proxy.yellow(),
                cpolar_path.display()
            );
            if cpolar_path.exists() {
                let backup = cpolar_path.with_extension("bak");
                let _ = std::fs::copy(&cpolar_path, &backup);
            }
            if let Ok(true) = unset_cpolar_proxy() {
                println!(
                    "  {} Cleared broken cpolar proxy in {} (backup saved)",
                    "✔".green(),
                    cpolar_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [235/263] cpolar proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [235/263] cpolar proxy: {}", "Clean or not set".dimmed());
    }

    // 236. Clean dead tmate Terminal Sharing & SSH Reverse Tunnel daemon in config / systemd
    if let Some((tmate_proxy, tmate_path)) = get_tmate_proxy() {
        if !verify_proxy_url(&tmate_proxy).await {
            println!(
                "  [236/263] Found dead tmate proxy (`{}` in {}), cleaning with backup...",
                tmate_proxy.yellow(),
                tmate_path.display()
            );
            if tmate_path.exists() {
                let backup = tmate_path.with_extension("bak");
                let _ = std::fs::copy(&tmate_path, &backup);
            }
            if let Ok(true) = unset_tmate_proxy() {
                println!(
                    "  {} Cleared broken tmate proxy in {} (backup saved)",
                    "✔".green(),
                    tmate_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [236/263] tmate proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [236/263] tmate proxy: {}", "Clean or not set".dimmed());
    }

    // 237. Clean dead upterm Terminal Sharing & SSH Reverse Tunnel daemon in config / systemd
    if let Some((upterm_proxy, upterm_path)) = get_upterm_proxy() {
        if !verify_proxy_url(&upterm_proxy).await {
            println!(
                "  [237/263] Found dead upterm proxy (`{}` in {}), cleaning with backup...",
                upterm_proxy.yellow(),
                upterm_path.display()
            );
            if upterm_path.exists() {
                let backup = upterm_path.with_extension("bak");
                let _ = std::fs::copy(&upterm_path, &backup);
            }
            if let Ok(true) = unset_upterm_proxy() {
                println!(
                    "  {} Cleared broken upterm proxy in {} (backup saved)",
                    "✔".green(),
                    upterm_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [237/263] upterm proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [237/263] upterm proxy: {}", "Clean or not set".dimmed());
    }

    // 238. Clean dead NATAPP Localhost Reverse Tunnel & Web UI daemon in config / systemd
    if let Some((natapp_proxy, natapp_path)) = get_natapp_proxy() {
        if !verify_proxy_url(&natapp_proxy).await {
            println!(
                "  [238/263] Found dead NATAPP proxy (`{}` in {}), cleaning with backup...",
                natapp_proxy.yellow(),
                natapp_path.display()
            );
            if natapp_path.exists() {
                let backup = natapp_path.with_extension("bak");
                let _ = std::fs::copy(&natapp_path, &backup);
            }
            if let Ok(true) = unset_natapp_proxy() {
                println!(
                    "  {} Cleared broken NATAPP proxy in {} (backup saved)",
                    "✔".green(),
                    natapp_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [238/263] NATAPP proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [238/263] NATAPP proxy: {}", "Clean or not set".dimmed());
    }

    // 239. Clean dead lanproxy Intranet Penetration & Reverse Tunnel Client in config / systemd
    if let Some((lanproxy_proxy, lanproxy_path)) = get_lanproxy_proxy() {
        if !verify_proxy_url(&lanproxy_proxy).await {
            println!(
                "  [239/263] Found dead lanproxy proxy (`{}` in {}), cleaning with backup...",
                lanproxy_proxy.yellow(),
                lanproxy_path.display()
            );
            if lanproxy_path.exists() {
                let backup = lanproxy_path.with_extension("bak");
                let _ = std::fs::copy(&lanproxy_path, &backup);
            }
            if let Ok(true) = unset_lanproxy_proxy() {
                println!(
                    "  {} Cleared broken lanproxy proxy in {} (backup saved)",
                    "✔".green(),
                    lanproxy_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [239/263] lanproxy proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [239/263] lanproxy proxy: {}", "Clean or not set".dimmed());
    }

    // 240. Clean dead teleconsole Terminal Sharing & SSH Reverse Tunnel daemon in config / systemd
    if let Some((teleconsole_proxy, teleconsole_path)) = get_teleconsole_proxy() {
        if !verify_proxy_url(&teleconsole_proxy).await {
            println!(
                "  [240/263] Found dead teleconsole proxy (`{}` in {}), cleaning with backup...",
                teleconsole_proxy.yellow(),
                teleconsole_path.display()
            );
            if teleconsole_path.exists() {
                let backup = teleconsole_path.with_extension("bak");
                let _ = std::fs::copy(&teleconsole_path, &backup);
            }
            if let Ok(true) = unset_teleconsole_proxy() {
                println!(
                    "  {} Cleared broken teleconsole proxy in {} (backup saved)",
                    "✔".green(),
                    teleconsole_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [240/263] teleconsole proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [240/263] teleconsole proxy: {}", "Clean or not set".dimmed());
    }

    // 241. Clean dead tunnelto Localhost Reverse Proxy & Public Tunnel Client in config / systemd
    if let Some((tunnelto_proxy, tunnelto_path)) = get_tunnelto_proxy() {
        if !verify_proxy_url(&tunnelto_proxy).await {
            println!(
                "  [241/263] Found dead tunnelto proxy (`{}` in {}), cleaning with backup...",
                tunnelto_proxy.yellow(),
                tunnelto_path.display()
            );
            if tunnelto_path.exists() {
                let backup = tunnelto_path.with_extension("bak");
                let _ = std::fs::copy(&tunnelto_path, &backup);
            }
            if let Ok(true) = unset_tunnelto_proxy() {
                println!(
                    "  {} Cleared broken tunnelto proxy in {} (backup saved)",
                    "✔".green(),
                    tunnelto_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [241/263] tunnelto proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [241/263] tunnelto proxy: {}", "Clean or not set".dimmed());
    }

    // 242. Clean dead Webhook Relay Localhost Tunnel & Webhook Forwarding Daemon in config / systemd
    if let Some((webhookrelay_proxy, webhookrelay_path)) = get_webhookrelay_proxy() {
        if !verify_proxy_url(&webhookrelay_proxy).await {
            println!(
                "  [242/263] Found dead Webhook Relay proxy (`{}` in {}), cleaning with backup...",
                webhookrelay_proxy.yellow(),
                webhookrelay_path.display()
            );
            if webhookrelay_path.exists() {
                let backup = webhookrelay_path.with_extension("bak");
                let _ = std::fs::copy(&webhookrelay_path, &backup);
            }
            if let Ok(true) = unset_webhookrelay_proxy() {
                println!(
                    "  {} Cleared broken Webhook Relay proxy in {} (backup saved)",
                    "✔".green(),
                    webhookrelay_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [242/263] Webhook Relay proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [242/263] Webhook Relay proxy: {}", "Clean or not set".dimmed());
    }


    // 243. Clean dead jprq Localhost Reverse Tunnel & Port Forwarding daemon in config / systemd
    if let Some((jprq_proxy, jprq_path)) = get_jprq_proxy() {
        if !verify_proxy_url(&jprq_proxy).await {
            println!(
                "  [243/263] Found dead jprq proxy (`{}` in {}), cleaning with backup...",
                jprq_proxy.yellow(),
                jprq_path.display()
            );
            if jprq_path.exists() {
                let backup = jprq_path.with_extension("bak");
                let _ = std::fs::copy(&jprq_path, &backup);
            }
            if let Ok(true) = unset_jprq_proxy() {
                println!(
                    "  {} Cleared broken jprq proxy in {} (backup saved)",
                    "✔".green(),
                    jprq_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [243/263] jprq proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [243/263] jprq proxy: {}", "Clean or not set".dimmed());
    }

    // 244. Clean dead mysocketctl Localhost Reverse Tunnel & Zero-Trust daemon in config / systemd
    if let Some((mysocketctl_proxy, mysocketctl_path)) = get_mysocketctl_proxy() {
        if !verify_proxy_url(&mysocketctl_proxy).await {
            println!(
                "  [244/263] Found dead mysocketctl proxy (`{}` in {}), cleaning with backup...",
                mysocketctl_proxy.yellow(),
                mysocketctl_path.display()
            );
            if mysocketctl_path.exists() {
                let backup = mysocketctl_path.with_extension("bak");
                let _ = std::fs::copy(&mysocketctl_path, &backup);
            }
            if let Ok(true) = unset_mysocketctl_proxy() {
                println!(
                    "  {} Cleared broken mysocketctl proxy in {} (backup saved)",
                    "✔".green(),
                    mysocketctl_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [244/263] mysocketctl proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [244/263] mysocketctl proxy: {}", "Clean or not set".dimmed());
    }

    // 245. Clean dead openp2p Localhost Reverse Tunnel & Port Forwarding daemon in config / systemd
    if let Some((openp2p_proxy, openp2p_path)) = get_openp2p_proxy() {
        if !verify_proxy_url(&openp2p_proxy).await {
            println!(
                "  [245/263] Found dead openp2p proxy (`{}` in {}), cleaning with backup...",
                openp2p_proxy.yellow(),
                openp2p_path.display()
            );
            if openp2p_path.exists() {
                let backup = openp2p_path.with_extension("bak");
                let _ = std::fs::copy(&openp2p_path, &backup);
            }
            if let Ok(true) = unset_openp2p_proxy() {
                println!(
                    "  {} Cleared broken openp2p proxy in {} (backup saved)",
                    "✔".green(),
                    openp2p_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [245/263] openp2p proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [245/263] openp2p proxy: {}", "Clean or not set".dimmed());
    }

    // 246. Clean dead rport Remote Management & Reverse Tunnel client daemon in config / systemd
    if let Some((rport_proxy, rport_path)) = get_rport_proxy() {
        if !verify_proxy_url(&rport_proxy).await {
            println!(
                "  [246/263] Found dead rport proxy (`{}` in {}), cleaning with backup...",
                rport_proxy.yellow(),
                rport_path.display()
            );
            if rport_path.exists() {
                let backup = rport_path.with_extension("bak");
                let _ = std::fs::copy(&rport_path, &backup);
            }
            if let Ok(true) = unset_rport_proxy() {
                println!(
                    "  {} Cleared broken rport proxy in {} (backup saved)",
                    "✔".green(),
                    rport_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [246/263] rport proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [246/263] rport proxy: {}", "Clean or not set".dimmed());
    }

    // 247. Clean dead mirrord Kubernetes Dev Traffic Mirror & Reverse Proxy daemon in config / systemd
    if let Some((mirrord_proxy, mirrord_path)) = get_mirrord_proxy() {
        if !verify_proxy_url(&mirrord_proxy).await {
            println!(
                "  [247/263] Found dead mirrord proxy (`{}` in {}), cleaning with backup...",
                mirrord_proxy.yellow(),
                mirrord_path.display()
            );
            if mirrord_path.exists() {
                let backup = mirrord_path.with_extension("bak");
                let _ = std::fs::copy(&mirrord_path, &backup);
            }
            if let Ok(true) = unset_mirrord_proxy() {
                println!(
                    "  {} Cleared broken mirrord proxy in {} (backup saved)",
                    "✔".green(),
                    mirrord_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [247/263] mirrord proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [247/263] mirrord proxy: {}", "Clean or not set".dimmed());
    }

    // 248. Clean dead kt-connect Kubernetes Dev Tunnel & SOCKS5 Proxy daemon in config / systemd
    if let Some((kt_connect_proxy, kt_connect_path)) = get_kt_connect_proxy() {
        if !verify_proxy_url(&kt_connect_proxy).await {
            println!(
                "  [248/263] Found dead kt-connect proxy (`{}` in {}), cleaning with backup...",
                kt_connect_proxy.yellow(),
                kt_connect_path.display()
            );
            if kt_connect_path.exists() {
                let backup = kt_connect_path.with_extension("bak");
                let _ = std::fs::copy(&kt_connect_path, &backup);
            }
            if let Ok(true) = unset_kt_connect_proxy() {
                println!(
                    "  {} Cleared broken kt-connect proxy in {} (backup saved)",
                    "✔".green(),
                    kt_connect_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [248/263] kt-connect proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [248/263] kt-connect proxy: {}", "Clean or not set".dimmed());
    }

    // 249. Clean dead DevSpace Cloud-Native Kubernetes Dev Proxy daemon in config / systemd
    if let Some((devspace_proxy, devspace_path)) = get_devspace_proxy() {
        if !verify_proxy_url(&devspace_proxy).await {
            println!(
                "  [249/263] Found dead DevSpace proxy (`{}` in {}), cleaning with backup...",
                devspace_proxy.yellow(),
                devspace_path.display()
            );
            if devspace_path.exists() {
                let backup = devspace_path.with_extension("bak");
                let _ = std::fs::copy(&devspace_path, &backup);
            }
            if let Ok(true) = unset_devspace_proxy() {
                println!(
                    "  {} Cleared broken DevSpace proxy in {} (backup saved)",
                    "✔".green(),
                    devspace_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [249/263] DevSpace proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [249/263] DevSpace proxy: {}", "Clean or not set".dimmed());
    }

    // 250. Clean dead Geph Censorship-Resistant Proxy & Reverse Tunnel daemon in config / systemd
    if let Some((geph_proxy, geph_path)) = get_geph_proxy() {
        if !verify_proxy_url(&geph_proxy).await {
            println!(
                "  [250/263] Found dead Geph proxy (`{}` in {}), cleaning with backup...",
                geph_proxy.yellow(),
                geph_path.display()
            );
            if geph_path.exists() {
                let backup = geph_path.with_extension("bak");
                let _ = std::fs::copy(&geph_path, &backup);
            }
            if let Ok(true) = unset_geph_proxy() {
                println!(
                    "  {} Cleared broken Geph proxy in {} (backup saved)",
                    "✔".green(),
                    geph_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [250/263] Geph proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [250/263] Geph proxy: {}", "Clean or not set".dimmed());
    }

    // 251. Clean dead Okteto Cloud-Native Kubernetes Dev Reverse Tunnel & SSH daemon in config / systemd
    if let Some((okteto_proxy, okteto_path)) = get_okteto_proxy() {
        if !verify_proxy_url(&okteto_proxy).await {
            println!(
                "  [251/263] Found dead Okteto proxy (`{}` in {}), cleaning with backup...",
                okteto_proxy.yellow(),
                okteto_path.display()
            );
            if okteto_path.exists() {
                let backup = okteto_path.with_extension("bak");
                let _ = std::fs::copy(&okteto_path, &backup);
            }
            if let Ok(true) = unset_okteto_proxy() {
                println!(
                    "  {} Cleared broken Okteto proxy in {} (backup saved)",
                    "✔".green(),
                    okteto_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [251/263] Okteto proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [251/263] Okteto proxy: {}", "Clean or not set".dimmed());
    }

    // 252. Clean dead Skaffold Cloud-Native Kubernetes Dev Port-Forward & Control daemon in config / systemd
    if let Some((skaffold_proxy, skaffold_path)) = get_skaffold_proxy() {
        if !verify_proxy_url(&skaffold_proxy).await {
            println!(
                "  [252/263] Found dead Skaffold proxy (`{}` in {}), cleaning with backup...",
                skaffold_proxy.yellow(),
                skaffold_path.display()
            );
            if skaffold_path.exists() {
                let backup = skaffold_path.with_extension("bak");
                let _ = std::fs::copy(&skaffold_path, &backup);
            }
            if let Ok(true) = unset_skaffold_proxy() {
                println!(
                    "  {} Cleared broken Skaffold proxy in {} (backup saved)",
                    "✔".green(),
                    skaffold_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [252/263] Skaffold proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [252/263] Skaffold proxy: {}", "Clean or not set".dimmed());
    }

    // 253. Clean dead Tilt Cloud-Native Kubernetes Dev Engine & Live Update Proxy daemon in config / systemd
    if let Some((tilt_proxy, tilt_path)) = get_tilt_proxy() {
        if !verify_proxy_url(&tilt_proxy).await {
            println!(
                "  [253/263] Found dead Tilt proxy (`{}` in {}), cleaning with backup...",
                tilt_proxy.yellow(),
                tilt_path.display()
            );
            if tilt_path.exists() {
                let backup = tilt_path.with_extension("bak");
                let _ = std::fs::copy(&tilt_path, &backup);
            }
            if let Ok(true) = unset_tilt_proxy() {
                println!(
                    "  {} Cleared broken Tilt proxy in {} (backup saved)",
                    "✔".green(),
                    tilt_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [253/263] Tilt proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [253/263] Tilt proxy: {}", "Clean or not set".dimmed());
    }

    // 254. Clean dead Garden Cloud-Native Kubernetes Dev Environment & Port-Forwarding Proxy daemon in config / systemd
    if let Some((garden_proxy, garden_path)) = get_garden_proxy() {
        if !verify_proxy_url(&garden_proxy).await {
            println!(
                "  [254/263] Found dead Garden proxy (`{}` in {}), cleaning with backup...",
                garden_proxy.yellow(),
                garden_path.display()
            );
            if garden_path.exists() {
                let backup = garden_path.with_extension("bak");
                let _ = std::fs::copy(&garden_path, &backup);
            }
            if let Ok(true) = unset_garden_proxy() {
                println!(
                    "  {} Cleared broken Garden proxy in {} (backup saved)",
                    "✔".green(),
                    garden_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [254/263] Garden proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [254/263] Garden proxy: {}", "Clean or not set".dimmed());
    }

    // 255. Clean dead Daytona Open-Source Dev Environment Manager & Workspace Tunnel Daemon proxy in config / systemd
    if let Some((daytona_proxy, daytona_path)) = get_daytona_proxy() {
        if !verify_proxy_url(&daytona_proxy).await {
            println!(
                "  [255/263] Found dead Daytona proxy (`{}` in {}), cleaning with backup...",
                daytona_proxy.yellow(),
                daytona_path.display()
            );
            if daytona_path.exists() {
                let backup = daytona_path.with_extension("bak");
                let _ = std::fs::copy(&daytona_path, &backup);
            }
            if let Ok(true) = unset_daytona_proxy() {
                println!(
                    "  {} Cleared broken Daytona proxy in {} (backup saved)",
                    "✔".green(),
                    daytona_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [255/263] Daytona proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [255/263] Daytona proxy: {}", "Clean or not set".dimmed());
    }

    // 256. Clean dead vcluster Virtual Kubernetes Cluster Control Plane & Syncer Proxy Daemon in config / systemd
    if let Some((vcluster_proxy, vcluster_path)) = get_vcluster_proxy() {
        if !verify_proxy_url(&vcluster_proxy).await {
            println!(
                "  [256/263] Found dead vcluster proxy (`{}` in {}), cleaning with backup...",
                vcluster_proxy.yellow(),
                vcluster_path.display()
            );
            if vcluster_path.exists() {
                let backup = vcluster_path.with_extension("bak");
                let _ = std::fs::copy(&vcluster_path, &backup);
            }
            if let Ok(true) = unset_vcluster_proxy() {
                println!(
                    "  {} Cleared broken vcluster proxy in {} (backup saved)",
                    "✔".green(),
                    vcluster_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [256/263] vcluster proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [256/263] vcluster proxy: {}", "Clean or not set".dimmed());
    }

    // 257. Clean dead Coder Open-Source Cloud Development Environment & Workspace Daemon in config / systemd
    if let Some((coder_proxy, coder_path)) = get_coder_proxy() {
        if !verify_proxy_url(&coder_proxy).await {
            println!(
                "  [257/263] Found dead Coder proxy (`{}` in {}), cleaning with backup...",
                coder_proxy.yellow(),
                coder_path.display()
            );
            if coder_path.exists() {
                let backup = coder_path.with_extension("bak");
                let _ = std::fs::copy(&coder_path, &backup);
            }
            if let Ok(true) = unset_coder_proxy() {
                println!(
                    "  {} Cleared broken Coder proxy in {} (backup saved)",
                    "✔".green(),
                    coder_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [257/263] Coder proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [257/263] Coder proxy: {}", "Clean or not set".dimmed());
    }

    // 258. Clean dead DevPod Client-Only Dev Environment Manager & Container Daemon in config / systemd
    if let Some((devpod_proxy, devpod_path)) = get_devpod_proxy() {
        if !verify_proxy_url(&devpod_proxy).await {
            println!(
                "  [258/263] Found dead DevPod proxy (`{}` in {}), cleaning with backup...",
                devpod_proxy.yellow(),
                devpod_path.display()
            );
            if devpod_path.exists() {
                let backup = devpod_path.with_extension("bak");
                let _ = std::fs::copy(&devpod_path, &backup);
            }
            if let Ok(true) = unset_devpod_proxy() {
                println!(
                    "  {} Cleared broken DevPod proxy in {} (backup saved)",
                    "✔".green(),
                    devpod_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [258/265] DevPod proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [258/265] DevPod proxy: {}", "Clean or not set".dimmed());
    }

    // 259. Clean dead Gitpod Automated Cloud Development Environment & Workspace Tunnel Daemon in config / systemd
    if let Some((gitpod_proxy, gitpod_path)) = get_gitpod_proxy() {
        if !verify_proxy_url(&gitpod_proxy).await {
            println!(
                "  [259/265] Found dead Gitpod proxy (`{}` in {}), cleaning with backup...",
                gitpod_proxy.yellow(),
                gitpod_path.display()
            );
            if gitpod_path.exists() {
                let backup = gitpod_path.with_extension("bak");
                let _ = std::fs::copy(&gitpod_path, &backup);
            }
            if let Ok(true) = unset_gitpod_proxy() {
                println!(
                    "  {} Cleared broken Gitpod proxy in {} (backup saved)",
                    "✔".green(),
                    gitpod_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [259/265] Gitpod proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [259/265] Gitpod proxy: {}", "Clean or not set".dimmed());
    }

    // 260. Clean dead Dev Container Open-Source Spec CLI & Workspace Container Daemon in config / systemd
    if let Some((devcontainer_proxy, devcontainer_path)) = get_devcontainer_proxy() {
        if !verify_proxy_url(&devcontainer_proxy).await {
            println!(
                "  [260/273] Found dead Dev Container proxy (`{}` in {}), cleaning with backup...",
                devcontainer_proxy.yellow(),
                devcontainer_path.display()
            );
            if devcontainer_path.exists() {
                let backup = devcontainer_path.with_extension("bak");
                let _ = std::fs::copy(&devcontainer_path, &backup);
            }
            if let Ok(true) = unset_devcontainer_proxy() {
                println!(
                    "  {} Cleared broken Dev Container proxy in {} (backup saved)",
                    "✔".green(),
                    devcontainer_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [260/273] Dev Container proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [260/273] Dev Container proxy: {}", "Clean or not set".dimmed());
    }

    // 261. Clean dead Mise-en-place Polyglot Dev Environment Manager in config / systemd
    if let Some((mise_proxy, mise_path)) = get_mise_proxy() {
        if !verify_proxy_url(&mise_proxy).await {
            println!(
                "  [261/273] Found dead Mise dev environment proxy (`{}` in {}), cleaning with backup...",
                mise_proxy.yellow(),
                mise_path.display()
            );
            if mise_path.exists() {
                let backup = mise_path.with_extension("bak");
                let _ = std::fs::copy(&mise_path, &backup);
            }
            if let Ok(true) = unset_mise_proxy() {
                println!(
                    "  {} Cleared broken Mise dev environment proxy in {} (backup saved)",
                    "✔".green(),
                    mise_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [261/273] Mise dev environment proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [261/273] Mise dev environment proxy: {}", "Clean or not set".dimmed());
    }

    // 262. Clean dead Pixi Fast Conda Dev Environment & Package Manager in config / systemd
    if let Some((pixi_proxy, pixi_path)) = get_pixi_proxy() {
        if !verify_proxy_url(&pixi_proxy).await {
            println!(
                "  [262/273] Found dead Pixi dev environment proxy (`{}` in {}), cleaning with backup...",
                pixi_proxy.yellow(),
                pixi_path.display()
            );
            if pixi_path.exists() {
                let backup = pixi_path.with_extension("bak");
                let _ = std::fs::copy(&pixi_path, &backup);
            }
            if let Ok(true) = unset_pixi_proxy() {
                println!(
                    "  {} Cleared broken Pixi dev environment proxy in {} (backup saved)",
                    "✔".green(),
                    pixi_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [262/273] Pixi dev environment proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [262/273] Pixi dev environment proxy: {}", "Clean or not set".dimmed());
    }

    // 263. Clean dead Proto Polyglot Toolchain Manager & Dev Environment in config / systemd
    if let Some((proto_proxy, proto_path)) = get_proto_proxy() {
        if !verify_proxy_url(&proto_proxy).await {
            println!(
                "  [263/273] Found dead Proto dev environment proxy (`{}` in {}), cleaning with backup...",
                proto_proxy.yellow(),
                proto_path.display()
            );
            if proto_path.exists() {
                let backup = proto_path.with_extension("bak");
                let _ = std::fs::copy(&proto_path, &backup);
            }
            if let Ok(true) = unset_proto_proxy() {
                println!(
                    "  {} Cleared broken Proto dev environment proxy in {} (backup saved)",
                    "✔".green(),
                    proto_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [263/273] Proto dev environment proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [263/273] Proto dev environment proxy: {}", "Clean or not set".dimmed());
    }

    // 264. Clean dead Flox Nix-Based Virtual Dev Environment & Package Manager in config / systemd
    if let Some((flox_proxy, flox_path)) = get_flox_proxy() {
        if !verify_proxy_url(&flox_proxy).await {
            println!(
                "  [264/273] Found dead Flox dev environment proxy (`{}` in {}), cleaning with backup...",
                flox_proxy.yellow(),
                flox_path.display()
            );
            if flox_path.exists() {
                let backup = flox_path.with_extension("bak");
                let _ = std::fs::copy(&flox_path, &backup);
            }
            if let Ok(true) = unset_flox_proxy() {
                println!(
                    "  {} Cleared broken Flox dev environment proxy in {} (backup saved)",
                    "✔".green(),
                    flox_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [264/273] Flox dev environment proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [264/273] Flox dev environment proxy: {}", "Clean or not set".dimmed());
    }

    // 265. Clean dead Devenv Fast Nix-Based Dev Environment Manager in config / systemd
    if let Some((devenv_proxy, devenv_path)) = get_devenv_proxy() {
        if !verify_proxy_url(&devenv_proxy).await {
            println!(
                "  [265/273] Found dead Devenv dev environment proxy (`{}` in {}), cleaning with backup...",
                devenv_proxy.yellow(),
                devenv_path.display()
            );
            if devenv_path.exists() {
                let backup = devenv_path.with_extension("bak");
                let _ = std::fs::copy(&devenv_path, &backup);
            }
            if let Ok(true) = unset_devenv_proxy() {
                println!(
                    "  {} Cleared broken Devenv dev environment proxy in {} (backup saved)",
                    "✔".green(),
                    devenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [265/273] Devenv dev environment proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [265/273] Devenv dev environment proxy: {}", "Clean or not set".dimmed());
    }

    // 266. Clean dead Devbox Portable Nix-Based Isolated Dev Environment in config / systemd
    if let Some((devbox_proxy, devbox_path)) = get_devbox_proxy() {
        if !verify_proxy_url(&devbox_proxy).await {
            println!(
                "  [266/273] Found dead Devbox dev environment proxy (`{}` in {}), cleaning with backup...",
                devbox_proxy.yellow(),
                devbox_path.display()
            );
            if devbox_path.exists() {
                let backup = devbox_path.with_extension("bak");
                let _ = std::fs::copy(&devbox_path, &backup);
            }
            if let Ok(true) = unset_devbox_proxy() {
                println!(
                    "  {} Cleared broken Devbox dev environment proxy in {} (backup saved)",
                    "✔".green(),
                    devbox_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [266/273] Devbox dev environment proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [266/273] Devbox dev environment proxy: {}", "Clean or not set".dimmed());
    }

    // 267. Clean dead Aqua Declarative CLI Version Manager in config / systemd
    if let Some((aqua_proxy, aqua_path)) = get_aqua_proxy() {
        if !verify_proxy_url(&aqua_proxy).await {
            println!(
                "  [267/279] Found dead Aqua CLI toolchain proxy (`{}` in {}), cleaning with backup...",
                aqua_proxy.yellow(),
                aqua_path.display()
            );
            if aqua_path.exists() {
                let backup = aqua_path.with_extension("bak");
                let _ = std::fs::copy(&aqua_path, &backup);
            }
            if let Ok(true) = unset_aqua_proxy() {
                println!(
                    "  {} Cleared broken Aqua CLI toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    aqua_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [267/279] Aqua CLI toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [267/279] Aqua CLI toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 268. Clean dead Hermit Isolated & Reproducible Dev Environment in config / systemd
    if let Some((hermit_proxy, hermit_path)) = get_hermit_proxy() {
        if !verify_proxy_url(&hermit_proxy).await {
            println!(
                "  [268/279] Found dead Hermit dev environment proxy (`{}` in {}), cleaning with backup...",
                hermit_proxy.yellow(),
                hermit_path.display()
            );
            if hermit_path.exists() {
                let backup = hermit_path.with_extension("bak");
                let _ = std::fs::copy(&hermit_path, &backup);
            }
            if let Ok(true) = unset_hermit_proxy() {
                println!(
                    "  {} Cleared broken Hermit dev environment proxy in {} (backup saved)",
                    "✔".green(),
                    hermit_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [268/279] Hermit dev environment proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [268/279] Hermit dev environment proxy: {}", "Clean or not set".dimmed());
    }

    // 269. Clean dead asdf Polyglot Runtime Version Manager in config / systemd
    if let Some((asdf_proxy, asdf_path)) = get_asdf_proxy() {
        if !verify_proxy_url(&asdf_proxy).await {
            println!(
                "  [269/279] Found dead asdf polyglot version manager proxy (`{}` in {}), cleaning with backup...",
                asdf_proxy.yellow(),
                asdf_path.display()
            );
            if asdf_path.exists() {
                let backup = asdf_path.with_extension("bak");
                let _ = std::fs::copy(&asdf_path, &backup);
            }
            if let Ok(true) = unset_asdf_proxy() {
                println!(
                    "  {} Cleared broken asdf polyglot version manager proxy in {} (backup saved)",
                    "✔".green(),
                    asdf_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [269/279] asdf polyglot version manager proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [269/279] asdf polyglot version manager proxy: {}", "Clean or not set".dimmed());
    }

    // 270. Clean dead vfox Extensible Multi-Platform Toolchain Manager in config / systemd
    if let Some((vfox_proxy, vfox_path)) = get_vfox_proxy() {
        if !verify_proxy_url(&vfox_proxy).await {
            println!(
                "  [270/279] Found dead vfox toolchain manager proxy (`{}` in {}), cleaning with backup...",
                vfox_proxy.yellow(),
                vfox_path.display()
            );
            if vfox_path.exists() {
                let backup = vfox_path.with_extension("bak");
                let _ = std::fs::copy(&vfox_path, &backup);
            }
            if let Ok(true) = unset_vfox_proxy() {
                println!(
                    "  {} Cleared broken vfox toolchain manager proxy in {} (backup saved)",
                    "✔".green(),
                    vfox_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [270/279] vfox toolchain manager proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [270/279] vfox toolchain manager proxy: {}", "Clean or not set".dimmed());
    }

    // 271. Clean dead SDKMAN! toolchain proxy in config / systemd
    if let Some((sdkman_proxy, sdkman_path)) = get_sdkman_proxy() {
        if !verify_proxy_url(&sdkman_proxy).await {
            println!(
                "  [271/279] Found dead SDKMAN! toolchain proxy (`{}` in {}), cleaning with backup...",
                sdkman_proxy.yellow(),
                sdkman_path.display()
            );
            if sdkman_path.exists() {
                let backup = sdkman_path.with_extension("bak");
                let _ = std::fs::copy(&sdkman_path, &backup);
            }
            if let Ok(true) = unset_sdkman_proxy() {
                println!(
                    "  {} Cleared broken SDKMAN! toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    sdkman_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [271/279] SDKMAN! toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [271/279] SDKMAN! toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 272. Clean dead Rustup toolchain proxy in config / systemd
    if let Some((rustup_proxy, rustup_path)) = get_rustup_proxy() {
        if !verify_proxy_url(&rustup_proxy).await {
            println!(
                "  [272/279] Found dead Rustup toolchain proxy (`{}` in {}), cleaning with backup...",
                rustup_proxy.yellow(),
                rustup_path.display()
            );
            if rustup_path.exists() {
                let backup = rustup_path.with_extension("bak");
                let _ = std::fs::copy(&rustup_path, &backup);
            }
            if let Ok(true) = unset_rustup_proxy() {
                println!(
                    "  {} Cleared broken Rustup toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    rustup_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [272/279] Rustup toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [272/279] Rustup toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 273. Clean dead fnm Fast Node Manager toolchain proxy in config / systemd
    if let Some((fnm_proxy, fnm_path)) = get_fnm_proxy() {
        if !verify_proxy_url(&fnm_proxy).await {
            println!(
                "  [273/279] Found dead fnm toolchain proxy (`{}` in {}), cleaning with backup...",
                fnm_proxy.yellow(),
                fnm_path.display()
            );
            if fnm_path.exists() {
                let backup = fnm_path.with_extension("bak");
                let _ = std::fs::copy(&fnm_path, &backup);
            }
            if let Ok(true) = unset_fnm_proxy() {
                println!(
                    "  {} Cleared broken fnm toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    fnm_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [273/279] fnm toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [273/279] fnm toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 274. Clean dead Volta JavaScript toolchain proxy in config / systemd
    if let Some((volta_proxy, volta_path)) = get_volta_proxy() {
        if !verify_proxy_url(&volta_proxy).await {
            println!(
                "  [274/323] Found dead Volta toolchain proxy (`{}` in {}), cleaning with backup...",
                volta_proxy.yellow(),
                volta_path.display()
            );
            if volta_path.exists() {
                let backup = volta_path.with_extension("bak");
                let _ = std::fs::copy(&volta_path, &backup);
            }
            if let Ok(true) = unset_volta_proxy() {
                println!(
                    "  {} Cleared broken Volta toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    volta_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [274/323] Volta toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [274/323] Volta toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 275. Clean dead pyenv Python toolchain proxy in config / systemd
    if let Some((pyenv_proxy, pyenv_path)) = get_pyenv_proxy() {
        if !verify_proxy_url(&pyenv_proxy).await {
            println!(
                "  [275/323] Found dead pyenv toolchain proxy (`{}` in {}), cleaning with backup...",
                pyenv_proxy.yellow(),
                pyenv_path.display()
            );
            if pyenv_path.exists() {
                let backup = pyenv_path.with_extension("bak");
                let _ = std::fs::copy(&pyenv_path, &backup);
            }
            if let Ok(true) = unset_pyenv_proxy() {
                println!(
                    "  {} Cleared broken pyenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    pyenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [275/323] pyenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [275/323] pyenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 276. Clean dead nvm Node Version Manager toolchain proxy in config / systemd
    if let Some((nvm_proxy, nvm_path)) = get_nvm_proxy() {
        if !verify_proxy_url(&nvm_proxy).await {
            println!(
                "  [276/323] Found dead nvm toolchain proxy (`{}` in {}), cleaning with backup...",
                nvm_proxy.yellow(),
                nvm_path.display()
            );
            if nvm_path.exists() {
                let backup = nvm_path.with_extension("bak");
                let _ = std::fs::copy(&nvm_path, &backup);
            }
            if let Ok(true) = unset_nvm_proxy() {
                println!(
                    "  {} Cleared broken nvm toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    nvm_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [276/323] nvm toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [276/323] nvm toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 277. Clean dead rbenv Ruby toolchain proxy in config / systemd
    if let Some((rbenv_proxy, rbenv_path)) = get_rbenv_proxy() {
        if !verify_proxy_url(&rbenv_proxy).await {
            println!(
                "  [277/323] Found dead rbenv toolchain proxy (`{}` in {}), cleaning with backup...",
                rbenv_proxy.yellow(),
                rbenv_path.display()
            );
            if rbenv_path.exists() {
                let backup = rbenv_path.with_extension("bak");
                let _ = std::fs::copy(&rbenv_path, &backup);
            }
            if let Ok(true) = unset_rbenv_proxy() {
                println!(
                    "  {} Cleared broken rbenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    rbenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [277/323] rbenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [277/323] rbenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 278. Clean dead gvm Go toolchain proxy in config / systemd
    if let Some((gvm_proxy, gvm_path)) = get_gvm_proxy() {
        if !verify_proxy_url(&gvm_proxy).await {
            println!(
                "  [278/323] Found dead gvm toolchain proxy (`{}` in {}), cleaning with backup...",
                gvm_proxy.yellow(),
                gvm_path.display()
            );
            if gvm_path.exists() {
                let backup = gvm_path.with_extension("bak");
                let _ = std::fs::copy(&gvm_path, &backup);
            }
            if let Ok(true) = unset_gvm_proxy() {
                println!(
                    "  {} Cleared broken gvm toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    gvm_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [278/323] gvm toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [278/323] gvm toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 279. Clean dead nodenv Node.js toolchain proxy in config / systemd
    if let Some((nodenv_proxy, nodenv_path)) = get_nodenv_proxy() {
        if !verify_proxy_url(&nodenv_proxy).await {
            println!(
                "  [279/323] Found dead nodenv toolchain proxy (`{}` in {}), cleaning with backup...",
                nodenv_proxy.yellow(),
                nodenv_path.display()
            );
            if nodenv_path.exists() {
                let backup = nodenv_path.with_extension("bak");
                let _ = std::fs::copy(&nodenv_path, &backup);
            }
            if let Ok(true) = unset_nodenv_proxy() {
                println!(
                    "  {} Cleared broken nodenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    nodenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [279/323] nodenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [279/323] nodenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 280. Clean dead jenv Java environment proxy in config / systemd
    if let Some((jenv_proxy, jenv_path)) = get_jenv_proxy() {
        if !verify_proxy_url(&jenv_proxy).await {
            println!(
                "  [280/323] Found dead jenv toolchain proxy (`{}` in {}), cleaning with backup...",
                jenv_proxy.yellow(),
                jenv_path.display()
            );
            if jenv_path.exists() {
                let backup = jenv_path.with_extension("bak");
                let _ = std::fs::copy(&jenv_path, &backup);
            }
            if let Ok(true) = unset_jenv_proxy() {
                println!(
                    "  {} Cleared broken jenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    jenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [280/323] jenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [280/323] jenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 281. Clean dead goenv Go toolchain proxy in config / systemd
    if let Some((goenv_proxy, goenv_path)) = get_goenv_proxy() {
        if !verify_proxy_url(&goenv_proxy).await {
            println!(
                "  [281/323] Found dead goenv toolchain proxy (`{}` in {}), cleaning with backup...",
                goenv_proxy.yellow(),
                goenv_path.display()
            );
            if goenv_path.exists() {
                let backup = goenv_path.with_extension("bak");
                let _ = std::fs::copy(&goenv_path, &backup);
            }
            if let Ok(true) = unset_goenv_proxy() {
                println!(
                    "  {} Cleared broken goenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    goenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [281/323] goenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [281/323] goenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 282. Clean dead phpenv PHP toolchain proxy in config / systemd
    if let Some((phpenv_proxy, phpenv_path)) = get_phpenv_proxy() {
        if !verify_proxy_url(&phpenv_proxy).await {
            println!(
                "  [282/323] Found dead phpenv toolchain proxy (`{}` in {}), cleaning with backup...",
                phpenv_proxy.yellow(),
                phpenv_path.display()
            );
            if phpenv_path.exists() {
                let backup = phpenv_path.with_extension("bak");
                let _ = std::fs::copy(&phpenv_path, &backup);
            }
            if let Ok(true) = unset_phpenv_proxy() {
                println!(
                    "  {} Cleared broken phpenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    phpenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [282/323] phpenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [282/323] phpenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 283. Clean dead swiftenv Swift toolchain proxy in config / systemd
    if let Some((swiftenv_proxy, swiftenv_path)) = get_swiftenv_proxy() {
        if !verify_proxy_url(&swiftenv_proxy).await {
            println!(
                "  [283/323] Found dead swiftenv toolchain proxy (`{}` in {}), cleaning with backup...",
                swiftenv_proxy.yellow(),
                swiftenv_path.display()
            );
            if swiftenv_path.exists() {
                let backup = swiftenv_path.with_extension("bak");
                let _ = std::fs::copy(&swiftenv_path, &backup);
            }
            if let Ok(true) = unset_swiftenv_proxy() {
                println!(
                    "  {} Cleared broken swiftenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    swiftenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [283/323] swiftenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [283/323] swiftenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 284. Clean dead perlbrew Perl toolchain proxy in config / systemd
    if let Some((perlbrew_proxy, perlbrew_path)) = get_perlbrew_proxy() {
        if !verify_proxy_url(&perlbrew_proxy).await {
            println!(
                "  [284/323] Found dead perlbrew toolchain proxy (`{}` in {}), cleaning with backup...",
                perlbrew_proxy.yellow(),
                perlbrew_path.display()
            );
            if perlbrew_path.exists() {
                let backup = perlbrew_path.with_extension("bak");
                let _ = std::fs::copy(&perlbrew_path, &backup);
            }
            if let Ok(true) = unset_perlbrew_proxy() {
                println!(
                    "  {} Cleared broken perlbrew toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    perlbrew_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [284/323] perlbrew toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [284/323] perlbrew toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 285. Clean dead luaver Lua toolchain proxy in config / systemd
    if let Some((luaver_proxy, luaver_path)) = get_luaver_proxy() {
        if !verify_proxy_url(&luaver_proxy).await {
            println!(
                "  [285/323] Found dead luaver toolchain proxy (`{}` in {}), cleaning with backup...",
                luaver_proxy.yellow(),
                luaver_path.display()
            );
            if luaver_path.exists() {
                let backup = luaver_path.with_extension("bak");
                let _ = std::fs::copy(&luaver_path, &backup);
            }
            if let Ok(true) = unset_luaver_proxy() {
                println!(
                    "  {} Cleared broken luaver toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    luaver_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [285/323] luaver toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [285/323] luaver toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 286. Clean dead ghcup Haskell toolchain proxy in config / systemd
    if let Some((ghcup_proxy, ghcup_path)) = get_ghcup_proxy() {
        if !verify_proxy_url(&ghcup_proxy).await {
            println!(
                "  [286/323] Found dead ghcup toolchain proxy (`{}` in {}), cleaning with backup...",
                ghcup_proxy.yellow(),
                ghcup_path.display()
            );
            if ghcup_path.exists() {
                let backup = ghcup_path.with_extension("bak");
                let _ = std::fs::copy(&ghcup_path, &backup);
            }
            if let Ok(true) = unset_ghcup_proxy() {
                println!(
                    "  {} Cleared broken ghcup toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    ghcup_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [286/323] ghcup toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [286/323] ghcup toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 287. Clean dead zvm Zig toolchain proxy in config / systemd
    if let Some((zvm_proxy, zvm_path)) = get_zvm_proxy() {
        if !verify_proxy_url(&zvm_proxy).await {
            println!(
                "  [287/323] Found dead zvm toolchain proxy (`{}` in {}), cleaning with backup...",
                zvm_proxy.yellow(),
                zvm_path.display()
            );
            if zvm_path.exists() {
                let backup = zvm_path.with_extension("bak");
                let _ = std::fs::copy(&zvm_path, &backup);
            }
            if let Ok(true) = unset_zvm_proxy() {
                println!(
                    "  {} Cleared broken zvm toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    zvm_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [287/323] zvm toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [287/323] zvm toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 288. Clean dead juliaup Julia toolchain proxy in config / systemd
    if let Some((juliaup_proxy, juliaup_path)) = get_juliaup_proxy() {
        if !verify_proxy_url(&juliaup_proxy).await {
            println!(
                "  [288/323] Found dead juliaup toolchain proxy (`{}` in {}), cleaning with backup...",
                juliaup_proxy.yellow(),
                juliaup_path.display()
            );
            if juliaup_path.exists() {
                let backup = juliaup_path.with_extension("bak");
                let _ = std::fs::copy(&juliaup_path, &backup);
            }
            if let Ok(true) = unset_juliaup_proxy() {
                println!(
                    "  {} Cleared broken juliaup toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    juliaup_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [288/323] juliaup toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [288/323] juliaup toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 289. Clean dead elan Lean toolchain proxy in config / systemd
    if let Some((elan_proxy, elan_path)) = get_elan_proxy() {
        if !verify_proxy_url(&elan_proxy).await {
            println!(
                "  [289/323] Found dead elan toolchain proxy (`{}` in {}), cleaning with backup...",
                elan_proxy.yellow(),
                elan_path.display()
            );
            if elan_path.exists() {
                let backup = elan_path.with_extension("bak");
                let _ = std::fs::copy(&elan_path, &backup);
            }
            if let Ok(true) = unset_elan_proxy() {
                println!(
                    "  {} Cleared broken elan toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    elan_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [289/323] elan toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [289/323] elan toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 290. Clean dead choosenim Nim toolchain proxy in config / systemd
    if let Some((choosenim_proxy, choosenim_path)) = get_choosenim_proxy() {
        if !verify_proxy_url(&choosenim_proxy).await {
            println!(
                "  [290/323] Found dead choosenim toolchain proxy (`{}` in {}), cleaning with backup...",
                choosenim_proxy.yellow(),
                choosenim_path.display()
            );
            if choosenim_path.exists() {
                let backup = choosenim_path.with_extension("bak");
                let _ = std::fs::copy(&choosenim_path, &backup);
            }
            if let Ok(true) = unset_choosenim_proxy() {
                println!(
                    "  {} Cleared broken choosenim toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    choosenim_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [290/323] choosenim toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [290/323] choosenim toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 291. Clean dead coursier Scala toolchain proxy in config / systemd
    if let Some((coursier_proxy, coursier_path)) = get_coursier_proxy() {
        if !verify_proxy_url(&coursier_proxy).await {
            println!(
                "  [291/323] Found dead coursier toolchain proxy (`{}` in {}), cleaning with backup...",
                coursier_proxy.yellow(),
                coursier_path.display()
            );
            if coursier_path.exists() {
                let backup = coursier_path.with_extension("bak");
                let _ = std::fs::copy(&coursier_path, &backup);
            }
            if let Ok(true) = unset_coursier_proxy() {
                println!(
                    "  {} Cleared broken coursier toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    coursier_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [291/323] coursier toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [291/323] coursier toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 292. Clean dead kerl Erlang/OTP toolchain proxy in config / systemd
    if let Some((kerl_proxy, kerl_path)) = get_kerl_proxy() {
        if !verify_proxy_url(&kerl_proxy).await {
            println!(
                "  [292/323] Found dead kerl toolchain proxy (`{}` in {}), cleaning with backup...",
                kerl_proxy.yellow(),
                kerl_path.display()
            );
            if kerl_path.exists() {
                let backup = kerl_path.with_extension("bak");
                let _ = std::fs::copy(&kerl_path, &backup);
            }
            if let Ok(true) = unset_kerl_proxy() {
                println!(
                    "  {} Cleared broken kerl toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    kerl_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [292/323] kerl toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [292/323] kerl toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 293. Clean dead tenv toolchain proxy in config / systemd
    if let Some((tenv_proxy, tenv_path)) = get_tenv_proxy() {
        if !verify_proxy_url(&tenv_proxy).await {
            println!(
                "  [293/323] Found dead tenv toolchain proxy (`{}` in {}), cleaning with backup...",
                tenv_proxy.yellow(),
                tenv_path.display()
            );
            if tenv_path.exists() {
                let backup = tenv_path.with_extension("bak");
                let _ = std::fs::copy(&tenv_path, &backup);
            }
            if let Ok(true) = unset_tenv_proxy() {
                println!(
                    "  {} Cleared broken tenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    tenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [293/323] tenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [293/323] tenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 294. Clean dead tfswitch toolchain proxy in config / systemd
    if let Some((tfswitch_proxy, tfswitch_path)) = get_tfswitch_proxy() {
        if !verify_proxy_url(&tfswitch_proxy).await {
            println!(
                "  [294/323] Found dead tfswitch toolchain proxy (`{}` in {}), cleaning with backup...",
                tfswitch_proxy.yellow(),
                tfswitch_path.display()
            );
            if tfswitch_path.exists() {
                let backup = tfswitch_path.with_extension("bak");
                let _ = std::fs::copy(&tfswitch_path, &backup);
            }
            if let Ok(true) = unset_tfswitch_proxy() {
                println!(
                    "  {} Cleared broken tfswitch toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    tfswitch_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [294/323] tfswitch toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [294/323] tfswitch toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 295. Clean dead tgswitch toolchain proxy in config / systemd
    if let Some((tgswitch_proxy, tgswitch_path)) = get_tgswitch_proxy() {
        if !verify_proxy_url(&tgswitch_proxy).await {
            println!(
                "  [295/323] Found dead tgswitch toolchain proxy (`{}` in {}), cleaning with backup...",
                tgswitch_proxy.yellow(),
                tgswitch_path.display()
            );
            if tgswitch_path.exists() {
                let backup = tgswitch_path.with_extension("bak");
                let _ = std::fs::copy(&tgswitch_path, &backup);
            }
            if let Ok(true) = unset_tgswitch_proxy() {
                println!(
                    "  {} Cleared broken tgswitch toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    tgswitch_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [295/323] tgswitch toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [295/323] tgswitch toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 296. Clean dead tfenv toolchain proxy in config / systemd
    if let Some((tfenv_proxy, tfenv_path)) = get_tfenv_proxy() {
        if !verify_proxy_url(&tfenv_proxy).await {
            println!(
                "  [296/323] Found dead tfenv toolchain proxy (`{}` in {}), cleaning with backup...",
                tfenv_proxy.yellow(),
                tfenv_path.display()
            );
            if tfenv_path.exists() {
                let backup = tfenv_path.with_extension("bak");
                let _ = std::fs::copy(&tfenv_path, &backup);
            }
            if let Ok(true) = unset_tfenv_proxy() {
                println!(
                    "  {} Cleared broken tfenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    tfenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [296/323] tfenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [296/323] tfenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 297. Clean dead tgenv toolchain proxy in config / systemd
    if let Some((tgenv_proxy, tgenv_path)) = get_tgenv_proxy() {
        if !verify_proxy_url(&tgenv_proxy).await {
            println!(
                "  [297/323] Found dead tgenv toolchain proxy (`{}` in {}), cleaning with backup...",
                tgenv_proxy.yellow(),
                tgenv_path.display()
            );
            if tgenv_path.exists() {
                let backup = tgenv_path.with_extension("bak");
                let _ = std::fs::copy(&tgenv_path, &backup);
            }
            if let Ok(true) = unset_tgenv_proxy() {
                println!(
                    "  {} Cleared broken tgenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    tgenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [297/323] tgenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [297/323] tgenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 298. Clean dead pkenv toolchain proxy in config / systemd
    if let Some((pkenv_proxy, pkenv_path)) = get_pkenv_proxy() {
        if !verify_proxy_url(&pkenv_proxy).await {
            println!(
                "  [298/323] Found dead pkenv toolchain proxy (`{}` in {}), cleaning with backup...",
                pkenv_proxy.yellow(),
                pkenv_path.display()
            );
            if pkenv_path.exists() {
                let backup = pkenv_path.with_extension("bak");
                let _ = std::fs::copy(&pkenv_path, &backup);
            }
            if let Ok(true) = unset_pkenv_proxy() {
                println!(
                    "  {} Cleared broken pkenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    pkenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [298/323] pkenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [298/323] pkenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 299. Clean dead tofuenv toolchain proxy in config / systemd
    if let Some((tofuenv_proxy, tofuenv_path)) = get_tofuenv_proxy() {
        if !verify_proxy_url(&tofuenv_proxy).await {
            println!(
                "  [299/323] Found dead tofuenv toolchain proxy (`{}` in {}), cleaning with backup...",
                tofuenv_proxy.yellow(),
                tofuenv_path.display()
            );
            if tofuenv_path.exists() {
                let backup = tofuenv_path.with_extension("bak");
                let _ = std::fs::copy(&tofuenv_path, &backup);
            }
            if let Ok(true) = unset_tofuenv_proxy() {
                println!(
                    "  {} Cleared broken tofuenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    tofuenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [299/323] tofuenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [299/323] tofuenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 300. Clean dead helmenv toolchain proxy in config / systemd
    if let Some((helmenv_proxy, helmenv_path)) = get_helmenv_proxy() {
        if !verify_proxy_url(&helmenv_proxy).await {
            println!(
                "  [300/323] Found dead helmenv toolchain proxy (`{}` in {}), cleaning with backup...",
                helmenv_proxy.yellow(),
                helmenv_path.display()
            );
            if helmenv_path.exists() {
                let backup = helmenv_path.with_extension("bak");
                let _ = std::fs::copy(&helmenv_path, &backup);
            }
            if let Ok(true) = unset_helmenv_proxy() {
                println!(
                    "  {} Cleared broken helmenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    helmenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [300/323] helmenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [300/323] helmenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 301. Clean dead kbenv toolchain proxy in config / systemd
    if let Some((kbenv_proxy, kbenv_path)) = get_kbenv_proxy() {
        if !verify_proxy_url(&kbenv_proxy).await {
            println!(
                "  [301/323] Found dead kbenv toolchain proxy (`{}` in {}), cleaning with backup...",
                kbenv_proxy.yellow(),
                kbenv_path.display()
            );
            if kbenv_path.exists() {
                let backup = kbenv_path.with_extension("bak");
                let _ = std::fs::copy(&kbenv_path, &backup);
            }
            if let Ok(true) = unset_kbenv_proxy() {
                println!(
                    "  {} Cleared broken kbenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    kbenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [301/323] kbenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [301/323] kbenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 302. Clean dead sopsenv toolchain proxy in config / systemd
    if let Some((sopsenv_proxy, sopsenv_path)) = get_sopsenv_proxy() {
        if !verify_proxy_url(&sopsenv_proxy).await {
            println!(
                "  [302/323] Found dead sopsenv toolchain proxy (`{}` in {}), cleaning with backup...",
                sopsenv_proxy.yellow(),
                sopsenv_path.display()
            );
            if sopsenv_path.exists() {
                let backup = sopsenv_path.with_extension("bak");
                let _ = std::fs::copy(&sopsenv_path, &backup);
            }
            if let Ok(true) = unset_sopsenv_proxy() {
                println!(
                    "  {} Cleared broken sopsenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    sopsenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [302/323] sopsenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [302/323] sopsenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 303. Clean dead ocenv toolchain proxy in config / systemd
    if let Some((ocenv_proxy, ocenv_path)) = get_ocenv_proxy() {
        if !verify_proxy_url(&ocenv_proxy).await {
            println!(
                "  [303/323] Found dead ocenv toolchain proxy (`{}` in {}), cleaning with backup...",
                ocenv_proxy.yellow(),
                ocenv_path.display()
            );
            if ocenv_path.exists() {
                let backup = ocenv_path.with_extension("bak");
                let _ = std::fs::copy(&ocenv_path, &backup);
            }
            if let Ok(true) = unset_ocenv_proxy() {
                println!(
                    "  {} Cleared broken ocenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    ocenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [303/323] ocenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [303/323] ocenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 304. Clean dead sbtenv toolchain proxy in config / systemd
    if let Some((sbtenv_proxy, sbtenv_path)) = get_sbtenv_proxy() {
        if !verify_proxy_url(&sbtenv_proxy).await {
            println!(
                "  [304/323] Found dead sbtenv toolchain proxy (`{}` in {}), cleaning with backup...",
                sbtenv_proxy.yellow(),
                sbtenv_path.display()
            );
            if sbtenv_path.exists() {
                let backup = sbtenv_path.with_extension("bak");
                let _ = std::fs::copy(&sbtenv_path, &backup);
            }
            if let Ok(true) = unset_sbtenv_proxy() {
                println!(
                    "  {} Cleared broken sbtenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    sbtenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [304/323] sbtenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [304/323] sbtenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 305. Clean dead scalaenv toolchain proxy in config / systemd
    if let Some((scalaenv_proxy, scalaenv_path)) = get_scalaenv_proxy() {
        if !verify_proxy_url(&scalaenv_proxy).await {
            println!(
                "  [305/323] Found dead scalaenv toolchain proxy (`{}` in {}), cleaning with backup...",
                scalaenv_proxy.yellow(),
                scalaenv_path.display()
            );
            if scalaenv_path.exists() {
                let backup = scalaenv_path.with_extension("bak");
                let _ = std::fs::copy(&scalaenv_path, &backup);
            }
            if let Ok(true) = unset_scalaenv_proxy() {
                println!(
                    "  {} Cleared broken scalaenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    scalaenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [305/323] scalaenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [305/323] scalaenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 306. Clean dead cljenv toolchain proxy in config / systemd
    if let Some((cljenv_proxy, cljenv_path)) = get_cljenv_proxy() {
        if !verify_proxy_url(&cljenv_proxy).await {
            println!(
                "  [306/323] Found dead cljenv toolchain proxy (`{}` in {}), cleaning with backup...",
                cljenv_proxy.yellow(),
                cljenv_path.display()
            );
            if cljenv_path.exists() {
                let backup = cljenv_path.with_extension("bak");
                let _ = std::fs::copy(&cljenv_path, &backup);
            }
            if let Ok(true) = unset_cljenv_proxy() {
                println!(
                    "  {} Cleared broken cljenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    cljenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [306/323] cljenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [306/323] cljenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 307. Clean dead crenv toolchain proxy in config / systemd
    if let Some((crenv_proxy, crenv_path)) = get_crenv_proxy() {
        if !verify_proxy_url(&crenv_proxy).await {
            println!(
                "  [307/323] Found dead crenv toolchain proxy (`{}` in {}), cleaning with backup...",
                crenv_proxy.yellow(),
                crenv_path.display()
            );
            if crenv_path.exists() {
                let backup = crenv_path.with_extension("bak");
                let _ = std::fs::copy(&crenv_path, &backup);
            }
            if let Ok(true) = unset_crenv_proxy() {
                println!(
                    "  {} Cleared broken crenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    crenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [307/323] crenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [307/323] crenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 308. Clean dead haxenv toolchain proxy in config / systemd
    if let Some((haxenv_proxy, haxenv_path)) = get_haxenv_proxy() {
        if !verify_proxy_url(&haxenv_proxy).await {
            println!(
                "  [308/323] Found dead haxenv toolchain proxy (`{}` in {}), cleaning with backup...",
                haxenv_proxy.yellow(),
                haxenv_path.display()
            );
            if haxenv_path.exists() {
                let backup = haxenv_path.with_extension("bak");
                let _ = std::fs::copy(&haxenv_path, &backup);
            }
            if let Ok(true) = unset_haxenv_proxy() {
                println!(
                    "  {} Cleared broken haxenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    haxenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [308/323] haxenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [308/323] haxenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 309. Clean dead plenv toolchain proxy in config / systemd
    if let Some((plenv_proxy, plenv_path)) = get_plenv_proxy() {
        if !verify_proxy_url(&plenv_proxy).await {
            println!(
                "  [309/323] Found dead plenv toolchain proxy (`{}` in {}), cleaning with backup...",
                plenv_proxy.yellow(),
                plenv_path.display()
            );
            if plenv_path.exists() {
                let backup = plenv_path.with_extension("bak");
                let _ = std::fs::copy(&plenv_path, &backup);
            }
            if let Ok(true) = unset_plenv_proxy() {
                println!(
                    "  {} Cleared broken plenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    plenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [309/323] plenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [309/323] plenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 310. Clean dead exenv toolchain proxy in config / systemd
    if let Some((exenv_proxy, exenv_path)) = get_exenv_proxy() {
        if !verify_proxy_url(&exenv_proxy).await {
            println!(
                "  [310/323] Found dead exenv toolchain proxy (`{}` in {}), cleaning with backup...",
                exenv_proxy.yellow(),
                exenv_path.display()
            );
            if exenv_path.exists() {
                let backup = exenv_path.with_extension("bak");
                let _ = std::fs::copy(&exenv_path, &backup);
            }
            if let Ok(true) = unset_exenv_proxy() {
                println!(
                    "  {} Cleared broken exenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    exenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [310/323] exenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [310/323] exenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 311. Clean dead luaenv toolchain proxy in config / systemd
    if let Some((luaenv_proxy, luaenv_path)) = get_luaenv_proxy() {
        if !verify_proxy_url(&luaenv_proxy).await {
            println!(
                "  [311/323] Found dead luaenv toolchain proxy (`{}` in {}), cleaning with backup...",
                luaenv_proxy.yellow(),
                luaenv_path.display()
            );
            if luaenv_path.exists() {
                let backup = luaenv_path.with_extension("bak");
                let _ = std::fs::copy(&luaenv_path, &backup);
            }
            if let Ok(true) = unset_luaenv_proxy() {
                println!(
                    "  {} Cleared broken luaenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    luaenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [311/323] luaenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [311/323] luaenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 312. Clean dead erlenv toolchain proxy in config / systemd
    if let Some((erlenv_proxy, erlenv_path)) = get_erlenv_proxy() {
        if !verify_proxy_url(&erlenv_proxy).await {
            println!(
                "  [312/323] Found dead erlenv toolchain proxy (`{}` in {}), cleaning with backup...",
                erlenv_proxy.yellow(),
                erlenv_path.display()
            );
            if erlenv_path.exists() {
                let backup = erlenv_path.with_extension("bak");
                let _ = std::fs::copy(&erlenv_path, &backup);
            }
            if let Ok(true) = unset_erlenv_proxy() {
                println!(
                    "  {} Cleared broken erlenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    erlenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [312/323] erlenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [312/323] erlenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 313. Clean dead dartenv toolchain proxy in config / systemd
    if let Some((dartenv_proxy, dartenv_path)) = get_dartenv_proxy() {
        if !verify_proxy_url(&dartenv_proxy).await {
            println!(
                "  [313/323] Found dead dartenv toolchain proxy (`{}` in {}), cleaning with backup...",
                dartenv_proxy.yellow(),
                dartenv_path.display()
            );
            if dartenv_path.exists() {
                let backup = dartenv_path.with_extension("bak");
                let _ = std::fs::copy(&dartenv_path, &backup);
            }
            if let Ok(true) = unset_dartenv_proxy() {
                println!(
                    "  {} Cleared broken dartenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    dartenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [313/323] dartenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [313/323] dartenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 314. Clean dead renv toolchain proxy in config / systemd
    if let Some((renv_proxy, renv_path)) = get_renv_proxy() {
        if !verify_proxy_url(&renv_proxy).await {
            println!(
                "  [314/323] Found dead renv toolchain proxy (`{}` in {}), cleaning with backup...",
                renv_proxy.yellow(),
                renv_path.display()
            );
            if renv_path.exists() {
                let backup = renv_path.with_extension("bak");
                let _ = std::fs::copy(&renv_path, &backup);
            }
            if let Ok(true) = unset_renv_proxy() {
                println!(
                    "  {} Cleared broken renv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    renv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [314/323] renv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [314/323] renv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 315. Clean dead kotenv toolchain proxy in config / systemd
    if let Some((kotenv_proxy, kotenv_path)) = get_kotenv_proxy() {
        if !verify_proxy_url(&kotenv_proxy).await {
            println!(
                "  [315/323] Found dead kotenv toolchain proxy (`{}` in {}), cleaning with backup...",
                kotenv_proxy.yellow(),
                kotenv_path.display()
            );
            if kotenv_path.exists() {
                let backup = kotenv_path.with_extension("bak");
                let _ = std::fs::copy(&kotenv_path, &backup);
            }
            if let Ok(true) = unset_kotenv_proxy() {
                println!(
                    "  {} Cleared broken kotenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    kotenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [315/323] kotenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [315/323] kotenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 316. Clean dead denoenv toolchain proxy in config / systemd
    if let Some((denoenv_proxy, denoenv_path)) = get_denoenv_proxy() {
        if !verify_proxy_url(&denoenv_proxy).await {
            println!(
                "  [316/323] Found dead denoenv toolchain proxy (`{}` in {}), cleaning with backup...",
                denoenv_proxy.yellow(),
                denoenv_path.display()
            );
            if denoenv_path.exists() {
                let backup = denoenv_path.with_extension("bak");
                let _ = std::fs::copy(&denoenv_path, &backup);
            }
            if let Ok(true) = unset_denoenv_proxy() {
                println!(
                    "  {} Cleared broken denoenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    denoenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [316/323] denoenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [316/323] denoenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 317. Clean dead bunenv toolchain proxy in config / systemd
    if let Some((bunenv_proxy, bunenv_path)) = get_bunenv_proxy() {
        if !verify_proxy_url(&bunenv_proxy).await {
            println!(
                "  [317/331] Found dead bunenv toolchain proxy (`{}` in {}), cleaning with backup...",
                bunenv_proxy.yellow(),
                bunenv_path.display()
            );
            if bunenv_path.exists() {
                let backup = bunenv_path.with_extension("bak");
                let _ = std::fs::copy(&bunenv_path, &backup);
            }
            if let Ok(true) = unset_bunenv_proxy() {
                println!(
                    "  {} Cleared broken bunenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    bunenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [317/331] bunenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [317/331] bunenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 318. Clean dead pgenv toolchain proxy in config / systemd
    if let Some((pgenv_proxy, pgenv_path)) = get_pgenv_proxy() {
        if !verify_proxy_url(&pgenv_proxy).await {
            println!(
                "  [318/331] Found dead pgenv toolchain proxy (`{}` in {}), cleaning with backup...",
                pgenv_proxy.yellow(),
                pgenv_path.display()
            );
            if pgenv_path.exists() {
                let backup = pgenv_path.with_extension("bak");
                let _ = std::fs::copy(&pgenv_path, &backup);
            }
            if let Ok(true) = unset_pgenv_proxy() {
                println!(
                    "  {} Cleared broken pgenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    pgenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [318/331] pgenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [318/331] pgenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 319. Clean dead rkenv toolchain proxy in config / systemd
    if let Some((rkenv_proxy, rkenv_path)) = get_rkenv_proxy() {
        if !verify_proxy_url(&rkenv_proxy).await {
            println!(
                "  [319/331] Found dead rkenv toolchain proxy (`{}` in {}), cleaning with backup...",
                rkenv_proxy.yellow(),
                rkenv_path.display()
            );
            if rkenv_path.exists() {
                let backup = rkenv_path.with_extension("bak");
                let _ = std::fs::copy(&rkenv_path, &backup);
            }
            if let Ok(true) = unset_rkenv_proxy() {
                println!(
                    "  {} Cleared broken rkenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    rkenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [319/331] rkenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [319/331] rkenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 320. Clean dead gdenv toolchain proxy in config / systemd
    if let Some((gdenv_proxy, gdenv_path)) = get_gdenv_proxy() {
        if !verify_proxy_url(&gdenv_proxy).await {
            println!(
                "  [320/331] Found dead gdenv toolchain proxy (`{}` in {}), cleaning with backup...",
                gdenv_proxy.yellow(),
                gdenv_path.display()
            );
            if gdenv_path.exists() {
                let backup = gdenv_path.with_extension("bak");
                let _ = std::fs::copy(&gdenv_path, &backup);
            }
            if let Ok(true) = unset_gdenv_proxy() {
                println!(
                    "  {} Cleared broken gdenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    gdenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [320/331] gdenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [320/331] gdenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 321. Clean dead anyenv toolchain proxy in config / systemd
    if let Some((anyenv_proxy, anyenv_path)) = get_anyenv_proxy() {
        if !verify_proxy_url(&anyenv_proxy).await {
            println!(
                "  [321/331] Found dead anyenv toolchain proxy (`{}` in {}), cleaning with backup...",
                anyenv_proxy.yellow(),
                anyenv_path.display()
            );
            if anyenv_path.exists() {
                let backup = anyenv_path.with_extension("bak");
                let _ = std::fs::copy(&anyenv_path, &backup);
            }
            if let Ok(true) = unset_anyenv_proxy() {
                println!(
                    "  {} Cleared broken anyenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    anyenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [321/331] anyenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [321/331] anyenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 322. Clean dead jlenv toolchain proxy in config / systemd
    if let Some((jlenv_proxy, jlenv_path)) = get_jlenv_proxy() {
        if !verify_proxy_url(&jlenv_proxy).await {
            println!(
                "  [322/331] Found dead jlenv toolchain proxy (`{}` in {}), cleaning with backup...",
                jlenv_proxy.yellow(),
                jlenv_path.display()
            );
            if jlenv_path.exists() {
                let backup = jlenv_path.with_extension("bak");
                let _ = std::fs::copy(&jlenv_path, &backup);
            }
            if let Ok(true) = unset_jlenv_proxy() {
                println!(
                    "  {} Cleared broken jlenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    jlenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [322/331] jlenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [322/331] jlenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 323. Clean dead zenv toolchain proxy in config / systemd
    if let Some((zenv_proxy, zenv_path)) = get_zenv_proxy() {
        if !verify_proxy_url(&zenv_proxy).await {
            println!(
                "  [323/331] Found dead zenv toolchain proxy (`{}` in {}), cleaning with backup...",
                zenv_proxy.yellow(),
                zenv_path.display()
            );
            if zenv_path.exists() {
                let backup = zenv_path.with_extension("bak");
                let _ = std::fs::copy(&zenv_path, &backup);
            }
            if let Ok(true) = unset_zenv_proxy() {
                println!(
                    "  {} Cleared broken zenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    zenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [323/331] zenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [323/331] zenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 324. Clean dead vlenv toolchain proxy in config / systemd
    if let Some((vlenv_proxy, vlenv_path)) = get_vlenv_proxy() {
        if !verify_proxy_url(&vlenv_proxy).await {
            println!(
                "  [324/331] Found dead vlenv toolchain proxy (`{}` in {}), cleaning with backup...",
                vlenv_proxy.yellow(),
                vlenv_path.display()
            );
            if vlenv_path.exists() {
                let backup = vlenv_path.with_extension("bak");
                let _ = std::fs::copy(&vlenv_path, &backup);
            }
            if let Ok(true) = unset_vlenv_proxy() {
                println!(
                    "  {} Cleared broken vlenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    vlenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [324/331] vlenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [324/331] vlenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 325. Clean dead glenv toolchain proxy in config / systemd
    if let Some((glenv_proxy, glenv_path)) = get_glenv_proxy() {
        if !verify_proxy_url(&glenv_proxy).await {
            println!(
                "  [325/331] Found dead glenv toolchain proxy (`{}` in {}), cleaning with backup...",
                glenv_proxy.yellow(),
                glenv_path.display()
            );
            if glenv_path.exists() {
                let backup = glenv_path.with_extension("bak");
                let _ = std::fs::copy(&glenv_path, &backup);
            }
            if let Ok(true) = unset_glenv_proxy() {
                println!(
                    "  {} Cleared broken glenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    glenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [325/331] glenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [325/331] glenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 326. Clean dead nimenv toolchain proxy in config / systemd
    if let Some((nimenv_proxy, nimenv_path)) = get_nimenv_proxy() {
        if !verify_proxy_url(&nimenv_proxy).await {
            println!(
                "  [326/333] Found dead nimenv toolchain proxy (`{}` in {}), cleaning with backup...",
                nimenv_proxy.yellow(),
                nimenv_path.display()
            );
            if nimenv_path.exists() {
                let backup = nimenv_path.with_extension("bak");
                let _ = std::fs::copy(&nimenv_path, &backup);
            }
            if let Ok(true) = unset_nimenv_proxy() {
                println!(
                    "  {} Cleared broken nimenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    nimenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [326/335] nimenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [326/335] nimenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 327. Clean dead tkenv toolchain proxy in config / systemd
    if let Some((tkenv_proxy, tkenv_path)) = get_tkenv_proxy() {
        if !verify_proxy_url(&tkenv_proxy).await {
            println!(
                "  [327/335] Found dead tkenv toolchain proxy (`{}` in {}), cleaning with backup...",
                tkenv_proxy.yellow(),
                tkenv_path.display()
            );
            if tkenv_path.exists() {
                let backup = tkenv_path.with_extension("bak");
                let _ = std::fs::copy(&tkenv_path, &backup);
            }
            if let Ok(true) = unset_tkenv_proxy() {
                println!(
                    "  {} Cleared broken tkenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    tkenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [327/335] tkenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [327/335] tkenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 328. Clean dead hsenv toolchain proxy in config / systemd
    if let Some((hsenv_proxy, hsenv_path)) = get_hsenv_proxy() {
        if !verify_proxy_url(&hsenv_proxy).await {
            println!(
                "  [328/335] Found dead hsenv toolchain proxy (`{}` in {}), cleaning with backup...",
                hsenv_proxy.yellow(),
                hsenv_path.display()
            );
            if hsenv_path.exists() {
                let backup = hsenv_path.with_extension("bak");
                let _ = std::fs::copy(&hsenv_path, &backup);
            }
            if let Ok(true) = unset_hsenv_proxy() {
                println!(
                    "  {} Cleared broken hsenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    hsenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [328/335] hsenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [328/335] hsenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 329. Clean dead kzenv toolchain proxy in config / systemd
    if let Some((kzenv_proxy, kzenv_path)) = get_kzenv_proxy() {
        if !verify_proxy_url(&kzenv_proxy).await {
            println!(
                "  [329/355] Found dead kzenv toolchain proxy (`{}` in {}), cleaning with backup...",
                kzenv_proxy.yellow(),
                kzenv_path.display()
            );
            if kzenv_path.exists() {
                let backup = kzenv_path.with_extension("bak");
                let _ = std::fs::copy(&kzenv_path, &backup);
            }
            if let Ok(true) = unset_kzenv_proxy() {
                println!(
                    "  {} Cleared broken kzenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    kzenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [329/355] kzenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [329/355] kzenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 330. Clean dead solenv toolchain proxy in config / systemd
    if let Some((solenv_proxy, solenv_path)) = get_solenv_proxy() {
        if !verify_proxy_url(&solenv_proxy).await {
            println!(
                "  [330/355] Found dead solenv toolchain proxy (`{}` in {}), cleaning with backup...",
                solenv_proxy.yellow(),
                solenv_path.display()
            );
            if solenv_path.exists() {
                let backup = solenv_path.with_extension("bak");
                let _ = std::fs::copy(&solenv_path, &backup);
            }
            if let Ok(true) = unset_solenv_proxy() {
                println!(
                    "  {} Cleared broken solenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    solenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [330/355] solenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [330/355] solenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 331. Clean dead bzenv toolchain proxy in config / systemd
    if let Some((bzenv_proxy, bzenv_path)) = get_bzenv_proxy() {
        if !verify_proxy_url(&bzenv_proxy).await {
            println!(
                "  [331/355] Found dead bzenv toolchain proxy (`{}` in {}), cleaning with backup...",
                bzenv_proxy.yellow(),
                bzenv_path.display()
            );
            if bzenv_path.exists() {
                let backup = bzenv_path.with_extension("bak");
                let _ = std::fs::copy(&bzenv_path, &backup);
            }
            if let Ok(true) = unset_bzenv_proxy() {
                println!(
                    "  {} Cleared broken bzenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    bzenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [331/355] bzenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [331/355] bzenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 332. Clean dead ctenv toolchain proxy in config / systemd
    if let Some((ctenv_proxy, ctenv_path)) = get_ctenv_proxy() {
        if !verify_proxy_url(&ctenv_proxy).await {
            println!(
                "  [332/355] Found dead ctenv toolchain proxy (`{}` in {}), cleaning with backup...",
                ctenv_proxy.yellow(),
                ctenv_path.display()
            );
            if ctenv_path.exists() {
                let backup = ctenv_path.with_extension("bak");
                let _ = std::fs::copy(&ctenv_path, &backup);
            }
            if let Ok(true) = unset_ctenv_proxy() {
                println!(
                    "  {} Cleared broken ctenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    ctenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [332/355] ctenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [332/355] ctenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 333. Clean dead cmenv toolchain proxy in config / systemd
    if let Some((cmenv_proxy, cmenv_path)) = get_cmenv_proxy() {
        if !verify_proxy_url(&cmenv_proxy).await {
            println!(
                "  [333/355] Found dead cmenv toolchain proxy (`{}` in {}), cleaning with backup...",
                cmenv_proxy.yellow(),
                cmenv_path.display()
            );
            if cmenv_path.exists() {
                let backup = cmenv_path.with_extension("bak");
                let _ = std::fs::copy(&cmenv_path, &backup);
            }
            if let Ok(true) = unset_cmenv_proxy() {
                println!(
                    "  {} Cleared broken cmenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    cmenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [333/355] cmenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [333/355] cmenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 334. Clean dead bufenv toolchain proxy in config / systemd
    if let Some((bufenv_proxy, bufenv_path)) = get_bufenv_proxy() {
        if !verify_proxy_url(&bufenv_proxy).await {
            println!(
                "  [334/355] Found dead bufenv toolchain proxy (`{}` in {}), cleaning with backup...",
                bufenv_proxy.yellow(),
                bufenv_path.display()
            );
            if bufenv_path.exists() {
                let backup = bufenv_path.with_extension("bak");
                let _ = std::fs::copy(&bufenv_path, &backup);
            }
            if let Ok(true) = unset_bufenv_proxy() {
                println!(
                    "  {} Cleared broken bufenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    bufenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [334/355] bufenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [334/355] bufenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 335. Clean dead pklenv toolchain proxy in config / systemd
    if let Some((pklenv_proxy, pklenv_path)) = get_pklenv_proxy() {
        if !verify_proxy_url(&pklenv_proxy).await {
            println!(
                "  [335/355] Found dead pklenv toolchain proxy (`{}` in {}), cleaning with backup...",
                pklenv_proxy.yellow(),
                pklenv_path.display()
            );
            if pklenv_path.exists() {
                let backup = pklenv_path.with_extension("bak");
                let _ = std::fs::copy(&pklenv_path, &backup);
            }
            if let Ok(true) = unset_pklenv_proxy() {
                println!(
                    "  {} Cleared broken pklenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    pklenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [335/355] pklenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [335/355] pklenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 336. Clean dead opaenv toolchain proxy in config / systemd
    if let Some((opaenv_proxy, opaenv_path)) = get_opaenv_proxy() {
        if !verify_proxy_url(&opaenv_proxy).await {
            println!(
                "  [336/355] Found dead opaenv toolchain proxy (`{}` in {}), cleaning with backup...",
                opaenv_proxy.yellow(),
                opaenv_path.display()
            );
            if opaenv_path.exists() {
                let backup = opaenv_path.with_extension("bak");
                let _ = std::fs::copy(&opaenv_path, &backup);
            }
            if let Ok(true) = unset_opaenv_proxy() {
                println!(
                    "  {} Cleared broken opaenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    opaenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [336/355] opaenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [336/355] opaenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 337. Clean dead dbtenv toolchain proxy in config / systemd
    if let Some((dbtenv_proxy, dbtenv_path)) = get_dbtenv_proxy() {
        if !verify_proxy_url(&dbtenv_proxy).await {
            println!(
                "  [337/355] Found dead dbtenv toolchain proxy (`{}` in {}), cleaning with backup...",
                dbtenv_proxy.yellow(),
                dbtenv_path.display()
            );
            if dbtenv_path.exists() {
                let backup = dbtenv_path.with_extension("bak");
                let _ = std::fs::copy(&dbtenv_path, &backup);
            }
            if let Ok(true) = unset_dbtenv_proxy() {
                println!(
                    "  {} Cleared broken dbtenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    dbtenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [337/355] dbtenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [337/355] dbtenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 338. Clean dead cueenv toolchain proxy in config / systemd
    if let Some((cueenv_proxy, cueenv_path)) = get_cueenv_proxy() {
        if !verify_proxy_url(&cueenv_proxy).await {
            println!(
                "  [338/355] Found dead cueenv toolchain proxy (`{}` in {}), cleaning with backup...",
                cueenv_proxy.yellow(),
                cueenv_path.display()
            );
            if cueenv_path.exists() {
                let backup = cueenv_path.with_extension("bak");
                let _ = std::fs::copy(&cueenv_path, &backup);
            }
            if let Ok(true) = unset_cueenv_proxy() {
                println!(
                    "  {} Cleared broken cueenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    cueenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [338/355] cueenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [338/355] cueenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 339. Clean dead wasmenv toolchain proxy in config / systemd
    if let Some((wasmenv_proxy, wasmenv_path)) = get_wasmenv_proxy() {
        if !verify_proxy_url(&wasmenv_proxy).await {
            println!(
                "  [339/355] Found dead wasmenv toolchain proxy (`{}` in {}), cleaning with backup...",
                wasmenv_proxy.yellow(),
                wasmenv_path.display()
            );
            if wasmenv_path.exists() {
                let backup = wasmenv_path.with_extension("bak");
                let _ = std::fs::copy(&wasmenv_path, &backup);
            }
            if let Ok(true) = unset_wasmenv_proxy() {
                println!(
                    "  {} Cleared broken wasmenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    wasmenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [339/355] wasmenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [339/355] wasmenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 340. Clean dead hfenv toolchain proxy in config / systemd
    if let Some((hfenv_proxy, hfenv_path)) = get_hfenv_proxy() {
        if !verify_proxy_url(&hfenv_proxy).await {
            println!(
                "  [340/355] Found dead hfenv toolchain proxy (`{}` in {}), cleaning with backup...",
                hfenv_proxy.yellow(),
                hfenv_path.display()
            );
            if hfenv_path.exists() {
                let backup = hfenv_path.with_extension("bak");
                let _ = std::fs::copy(&hfenv_path, &backup);
            }
            if let Ok(true) = unset_hfenv_proxy() {
                println!(
                    "  {} Cleared broken hfenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    hfenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [340/355] hfenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [340/355] hfenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 341. Clean dead kopsenv toolchain proxy in config / systemd
    if let Some((kopsenv_proxy, kopsenv_path)) = get_kopsenv_proxy() {
        if !verify_proxy_url(&kopsenv_proxy).await {
            println!(
                "  [341/355] Found dead kopsenv toolchain proxy (`{}` in {}), cleaning with backup...",
                kopsenv_proxy.yellow(),
                kopsenv_path.display()
            );
            if kopsenv_path.exists() {
                let backup = kopsenv_path.with_extension("bak");
                let _ = std::fs::copy(&kopsenv_path, &backup);
            }
            if let Ok(true) = unset_kopsenv_proxy() {
                println!(
                    "  {} Cleared broken kopsenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    kopsenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [341/355] kopsenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [341/355] kopsenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 342. Clean dead tflintenv toolchain proxy in config / systemd
    if let Some((tflintenv_proxy, tflintenv_path)) = get_tflintenv_proxy() {
        if !verify_proxy_url(&tflintenv_proxy).await {
            println!(
                "  [342/355] Found dead tflintenv toolchain proxy (`{}` in {}), cleaning with backup...",
                tflintenv_proxy.yellow(),
                tflintenv_path.display()
            );
            if tflintenv_path.exists() {
                let backup = tflintenv_path.with_extension("bak");
                let _ = std::fs::copy(&tflintenv_path, &backup);
            }
            if let Ok(true) = unset_tflintenv_proxy() {
                println!(
                    "  {} Cleared broken tflintenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    tflintenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [342/355] tflintenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [342/355] tflintenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 343. Clean dead argoenv toolchain proxy in config / systemd
    if let Some((argoenv_proxy, argoenv_path)) = get_argoenv_proxy() {
        if !verify_proxy_url(&argoenv_proxy).await {
            println!(
                "  [343/355] Found dead argoenv toolchain proxy (`{}` in {}), cleaning with backup...",
                argoenv_proxy.yellow(),
                argoenv_path.display()
            );
            if argoenv_path.exists() {
                let backup = argoenv_path.with_extension("bak");
                let _ = std::fs::copy(&argoenv_path, &backup);
            }
            if let Ok(true) = unset_argoenv_proxy() {
                println!(
                    "  {} Cleared broken argoenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    argoenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [343/355] argoenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [343/355] argoenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 344. Clean dead istioenv toolchain proxy in config / systemd
    if let Some((istioenv_proxy, istioenv_path)) = get_istioenv_proxy() {
        if !verify_proxy_url(&istioenv_proxy).await {
            println!(
                "  [344/355] Found dead istioenv toolchain proxy (`{}` in {}), cleaning with backup...",
                istioenv_proxy.yellow(),
                istioenv_path.display()
            );
            if istioenv_path.exists() {
                let backup = istioenv_path.with_extension("bak");
                let _ = std::fs::copy(&istioenv_path, &backup);
            }
            if let Ok(true) = unset_istioenv_proxy() {
                println!(
                    "  {} Cleared broken istioenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    istioenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [344/355] istioenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [344/355] istioenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 345. Clean dead ciliumenv toolchain proxy in config / systemd
    if let Some((ciliumenv_proxy, ciliumenv_path)) = get_ciliumenv_proxy() {
        if !verify_proxy_url(&ciliumenv_proxy).await {
            println!(
                "  [345/355] Found dead ciliumenv toolchain proxy (`{}` in {}), cleaning with backup...",
                ciliumenv_proxy.yellow(),
                ciliumenv_path.display()
            );
            if ciliumenv_path.exists() {
                let backup = ciliumenv_path.with_extension("bak");
                let _ = std::fs::copy(&ciliumenv_path, &backup);
            }
            if let Ok(true) = unset_ciliumenv_proxy() {
                println!(
                    "  {} Cleared broken ciliumenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    ciliumenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [345/355] ciliumenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [345/355] ciliumenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 346. Clean dead linkerdenv toolchain proxy in config / systemd
    if let Some((linkerdenv_proxy, linkerdenv_path)) = get_linkerdenv_proxy() {
        if !verify_proxy_url(&linkerdenv_proxy).await {
            println!(
                "  [346/355] Found dead linkerdenv toolchain proxy (`{}` in {}), cleaning with backup...",
                linkerdenv_proxy.yellow(),
                linkerdenv_path.display()
            );
            if linkerdenv_path.exists() {
                let backup = linkerdenv_path.with_extension("bak");
                let _ = std::fs::copy(&linkerdenv_path, &backup);
            }
            if let Ok(true) = unset_linkerdenv_proxy() {
                println!(
                    "  {} Cleared broken linkerdenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    linkerdenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [346/355] linkerdenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [346/355] linkerdenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 347. Clean dead fluxenv toolchain proxy in config / systemd
    if let Some((fluxenv_proxy, fluxenv_path)) = get_fluxenv_proxy() {
        if !verify_proxy_url(&fluxenv_proxy).await {
            println!(
                "  [347/355] Found dead fluxenv toolchain proxy (`{}` in {}), cleaning with backup...",
                fluxenv_proxy.yellow(),
                fluxenv_path.display()
            );
            if fluxenv_path.exists() {
                let backup = fluxenv_path.with_extension("bak");
                let _ = std::fs::copy(&fluxenv_path, &backup);
            }
            if let Ok(true) = unset_fluxenv_proxy() {
                println!(
                    "  {} Cleared broken fluxenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    fluxenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [347/355] fluxenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [347/355] fluxenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 348. Clean dead kindenv toolchain proxy in config / systemd
    if let Some((kindenv_proxy, kindenv_path)) = get_kindenv_proxy() {
        if !verify_proxy_url(&kindenv_proxy).await {
            println!(
                "  [348/355] Found dead kindenv toolchain proxy (`{}` in {}), cleaning with backup...",
                kindenv_proxy.yellow(),
                kindenv_path.display()
            );
            if kindenv_path.exists() {
                let backup = kindenv_path.with_extension("bak");
                let _ = std::fs::copy(&kindenv_path, &backup);
            }
            if let Ok(true) = unset_kindenv_proxy() {
                println!(
                    "  {} Cleared broken kindenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    kindenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [348/355] kindenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [348/355] kindenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 349. Clean dead k3denv toolchain proxy in config / systemd
    if let Some((k3denv_proxy, k3denv_path)) = get_k3denv_proxy() {
        if !verify_proxy_url(&k3denv_proxy).await {
            println!(
                "  [349/355] Found dead k3denv toolchain proxy (`{}` in {}), cleaning with backup...",
                k3denv_proxy.yellow(),
                k3denv_path.display()
            );
            if k3denv_path.exists() {
                let backup = k3denv_path.with_extension("bak");
                let _ = std::fs::copy(&k3denv_path, &backup);
            }
            if let Ok(true) = unset_k3denv_proxy() {
                println!(
                    "  {} Cleared broken k3denv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    k3denv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [349/355] k3denv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [349/355] k3denv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 350. Clean dead cosignenv toolchain proxy in config / systemd
    if let Some((cosignenv_proxy, cosignenv_path)) = get_cosignenv_proxy() {
        if !verify_proxy_url(&cosignenv_proxy).await {
            println!(
                "  [350/355] Found dead cosignenv toolchain proxy (`{}` in {}), cleaning with backup...",
                cosignenv_proxy.yellow(),
                cosignenv_path.display()
            );
            if cosignenv_path.exists() {
                let backup = cosignenv_path.with_extension("bak");
                let _ = std::fs::copy(&cosignenv_path, &backup);
            }
            if let Ok(true) = unset_cosignenv_proxy() {
                println!(
                    "  {} Cleared broken cosignenv toolchain proxy in {} (backup saved)",
                    "✔".green(),
                    cosignenv_path.display()
                );
                fixed_count += 1;
            }
        } else {
            println!("  [350/355] cosignenv toolchain proxy: {}", "Reachable".green());
        }
    } else {
        println!("  [350/355] cosignenv toolchain proxy: {}", "Clean or not set".dimmed());
    }

    // 351. Clean dead Alpine Linux apk package manager repositories in /etc/apk/repositories
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
                        "  [351/355] Found {} dead Alpine apk repository mirror(s) in {}, cleaning with backup...",
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
                    println!("  [351/355] Alpine Linux apk repositories: {}", "Reachable".green());
                } else {
                    println!("  [351/355] Alpine Linux apk repositories: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [351/355] Alpine Linux apk repositories: {}", "Clean or not set".dimmed());
    }

    // 352. Clean dead vcpkg registries/mirrors in vcpkg-configuration.json
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
                        "  [352/355] Found {} dead vcpkg registry mirror(s) in {}, cleaning with backup...",
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
                    println!("  [352/355] vcpkg configuration registries: {}", "Reachable".green());
                } else {
                    println!("  [352/355] vcpkg configuration registries: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [352/355] vcpkg configuration registries: {}", "Clean or not set".dimmed());
    }

    // 353. Clean dead Kubernetes Helm chart repositories in repositories.yaml
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
                        "  [353/355] Found {} dead Helm chart repository mirror(s) in {}, cleaning with backup...",
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
                    println!("  [353/355] Kubernetes Helm chart repositories: {}", "Reachable".green());
                } else {
                    println!("  [353/355] Kubernetes Helm chart repositories: {}", "Clean or empty".dimmed());
                }
            }
        }
    } else {
        println!("  [353/355] Kubernetes Helm chart repositories: {}", "Clean or not set".dimmed());
    }

    // 354. Sync active system/daemon proxy to terminal with proper NO_PROXY bypass
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
            "  [354/355] Detected healthy active proxy on port {}. Generating Shell sync...",
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
            "  [354/355] No active local proxy client found. Ensure your proxy software is open."
        );
    }

    // 355. Flush local DNS cache
    println!("  [355/355] Flushing local DNS cache to fix resolution stalls...");
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
