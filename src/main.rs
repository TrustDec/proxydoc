use anyhow::Result;
use clap::{Parser, Subcommand};

mod checker;
mod doctor;
mod fixer;
mod manager;
mod scanner;

#[derive(Parser)]
#[command(
    name = "proxydoc",
    author = "TrustDec <https://github.com/TrustDec>",
    version = clap::crate_version!(),
    about = "🩺 Deep developer proxy diagnostics, DNS poisoning checker & one-click auto-repair CLI.",
    long_about = "proxydoc is a modern, lightweight CLI tool to diagnose root causes of broken developer network environments, detect DNS poisoning on GitHub/AI APIs, sync system proxies to terminal, and repair broken configurations automatically."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 🩺 Deep diagnostics (System vs Terminal sync, DNS poisoning, Git/NPM/Cargo/Pip/cURL/Docker dead ports)
    Doctor {
        /// Show verbose diagnostic details
        #[arg(short, long)]
        verbose: bool,
        /// Output diagnostics in JSON format for automation / IDEs
        #[arg(long)]
        json: bool,
    },
    /// 🩹 One-click automatic repair for detected network and proxy issues
    Fix,
    /// ⚡ Concurrently test latency to GitHub, OpenAI, Anthropic, DeepSeek, Google & inspect Egress IP
    Test {
        /// Custom proxy URL to test (e.g. http://127.0.0.1:7890 or socks5://127.0.0.1:1082)
        #[arg(short, long)]
        proxy: Option<String>,
        /// Force direct connection ignoring any active proxy
        #[arg(short, long)]
        direct: bool,
        /// Benchmark and compare Direct vs Proxy side-by-side
        #[arg(short, long)]
        compare: bool,
        /// Output test results and egress info in JSON format
        #[arg(long)]
        json: bool,
        /// Additional custom endpoint URL to benchmark (e.g. https://api.deepseek.com or internal gateway)
        #[arg(short, long)]
        target: Option<String>,
        /// Timeout in seconds per endpoint request (default: 5)
        #[arg(long, default_value_t = 5)]
        timeout: u64,
    },
    /// 🔍 Scan active local proxy daemon ports (Clash, Shadowrocket, Sing-box, V2Ray, CPA)
    Scan {
        /// Output scan results in JSON format
        #[arg(long)]
        json: bool,
    },
    /// 🟢 Output shell export commands to turn ON proxy for current session
    On {
        /// Proxy port or URL (defaults to auto-detected system or daemon port)
        #[arg(default_value = "auto")]
        target: String,
        /// Also configure git global proxy
        #[arg(long)]
        git: bool,
        /// Also configure cargo proxy (~/.cargo/config.toml)
        #[arg(long)]
        cargo: bool,
        /// Also configure npm proxy
        #[arg(long)]
        npm: bool,
        /// Also configure Yarn proxy (~/.yarnrc / ~/.yarnrc.yml)
        #[arg(long)]
        yarn: bool,
        /// Also configure pnpm proxy (~/.pnpmrc)
        #[arg(long)]
        pnpm: bool,
        /// Also configure Bun proxy (~/.bunfig.toml)
        #[arg(long)]
        bun: bool,
        /// Also configure pip global proxy
        #[arg(long)]
        pip: bool,
        /// Also configure Conda proxy (~/.condarc)
        #[arg(long)]
        conda: bool,
        /// Also configure Docker client proxy (~/.docker/config.json)
        #[arg(long)]
        docker: bool,
        /// Also configure Gradle global proxy (~/.gradle/gradle.properties)
        #[arg(long)]
        gradle: bool,
        /// Also configure Maven global proxy (~/.m2/settings.xml)
        #[arg(long)]
        maven: bool,
        /// Also configure VS Code and Cursor IDE proxy (settings.json)
        #[arg(long)]
        vscode: bool,
        /// Also configure GitHub CLI (gh) proxy (~/.config/gh/config.yml)
        #[arg(long)]
        gh: bool,
        /// Also configure GitLab CLI (glab) proxy (~/.config/glab-cli/config.yml)
        #[arg(long)]
        glab: bool,
        /// Also configure JetBrains & Android Studio IDE proxy (proxy.settings.xml)
        #[arg(long)]
        jetbrains: bool,
        /// Also configure NuGet (.NET) proxy (NuGet.Config)
        #[arg(long)]
        nuget: bool,
        /// Also configure cURL proxy (~/.curlrc)
        #[arg(long)]
        curl: bool,
        /// Also configure Wget proxy (~/.wgetrc)
        #[arg(long)]
        wget: bool,
        /// Also configure Gem (Ruby) proxy (~/.gemrc)
        #[arg(long)]
        gem: bool,
        /// Also configure Subversion (SVN) proxy (~/.subversion/servers)
        #[arg(long)]
        svn: bool,
        /// Also configure Mercurial (Hg) proxy (~/.hgrc)
        #[arg(long)]
        hg: bool,
        /// Also configure Astral uv proxy (~/.config/uv/uv.toml)
        #[arg(long)]
        uv: bool,
        /// Also configure Go modules proxy (go/env & GOPROXY)
        #[arg(long)]
        go: bool,
        /// Also configure Bazel proxy (~/.bazelrc action_env)
        #[arg(long)]
        bazel: bool,
        /// Also configure PHP Composer proxy (config.json)
        #[arg(long)]
        composer: bool,
        /// Also configure Conan C/C++ proxy (global.conf / conan.conf)
        #[arg(long)]
        conan: bool,
        /// Also configure Scala Build Tool (sbt) proxy (~/.sbt/sbtopts)
        #[arg(long)]
        sbt: bool,
        /// Also configure Kubernetes (kubectl) cluster proxy (~/.kube/config proxy-url)
        #[arg(long)]
        kube: bool,
        /// Also configure Julia proxy (~/.julia/config/startup.jl)
        #[arg(long)]
        julia: bool,
        /// Also configure Nix package manager proxy (~/.config/nix/nix.conf)
        #[arg(long)]
        nix: bool,
        /// Also configure Haskell Cabal proxy (~/.cabal/config)
        #[arg(long)]
        cabal: bool,
        /// Also configure Haskell Stack proxy (~/.stack/config.yaml)
        #[arg(long)]
        stack: bool,
        /// Also configure Elixir / Erlang / Gleam Hex proxy (~/.hex/hex.config)
        #[arg(long)]
        hex: bool,
        /// Also configure R package manager proxy (~/.Renviron)
        #[arg(long)]
        r: bool,
        /// Also configure OCaml OPAM proxy (~/.opam/config)
        #[arg(long)]
        opam: bool,
        /// Also configure Clojure Leiningen proxy (~/.lein/profiles.clj)
        #[arg(long)]
        lein: bool,
        /// Also configure Perl CPAN package manager proxy (~/.cpan/CPAN/MyConfig.pm)
        #[arg(long)]
        cpan: bool,
        /// Also configure Lua (LuaRocks) package manager proxy (~/.luarocks/config.lua)
        #[arg(long)]
        luarocks: bool,
        /// Also configure Nim (Nimble) package manager proxy (~/.config/nimble/nimble.ini)
        #[arg(long)]
        nimble: bool,
        /// Also configure Erlang (Rebar3) package manager proxy (~/.config/rebar3/rebar.config)
        #[arg(long)]
        rebar3: bool,
        /// Also configure Scoop (Windows Package Manager) proxy (~/.config/scoop/config.json)
        #[arg(long)]
        scoop: bool,
        /// Also configure Ruby Bundler proxy (~/.bundle/config)
        #[arg(long, visible_alias = "bundler")]
        bundle: bool,
        /// Also configure Google Cloud SDK (gcloud) proxy (~/.config/gcloud/configurations/config_*)
        #[arg(long)]
        gcloud: bool,
        /// Also configure AWS CLI proxy (~/.aws/config)
        #[arg(long)]
        aws: bool,
        /// Also configure Microsoft Azure CLI (az) proxy (~/.azure/config)
        #[arg(long, visible_alias = "az")]
        azure: bool,
        /// Also configure Mamba / Micromamba proxy (~/.mambarc)
        #[arg(long, visible_alias = "micromamba")]
        mamba: bool,
        /// Also configure Podman container engine proxy (~/.config/containers/containers.conf)
        #[arg(long)]
        podman: bool,
        /// Also configure Vagrant proxy (~/.vagrant.d/Vagrantfile)
        #[arg(long)]
        vagrant: bool,
        /// Also configure Chocolatey package manager proxy (chocolatey.config)
        #[arg(long, visible_alias = "chocolatey")]
        choco: bool,
        /// Also configure Colima container runtime proxy (~/.colima/default/colima.yaml)
        #[arg(long)]
        colima: bool,
        /// Also configure Minikube cluster proxy (~/.minikube/config/config.json)
        #[arg(long)]
        minikube: bool,
        /// Also configure Spack scientific package manager proxy (~/.spack/config.yaml)
        #[arg(long)]
        spack: bool,
        /// Also configure MacPorts package manager proxy (~/.macports/macports.conf)
        #[arg(long)]
        macports: bool,
        /// Also configure APT package manager proxy (/etc/apt/apt.conf.d/99proxydoc.conf)
        #[arg(long)]
        apt: bool,
        /// Also configure DNF & YUM package manager proxy (/etc/dnf/dnf.conf)
        #[arg(long)]
        dnf: bool,
        /// Also configure Zypper package manager proxy (/etc/zypp/zypp.conf)
        #[arg(long)]
        zypper: bool,
        /// Also configure Pacman package manager proxy (/etc/pacman.conf)
        #[arg(long)]
        pacman: bool,
        /// Also configure FreeBSD pkg package manager proxy (/usr/local/etc/pkg.conf)
        #[arg(long)]
        pkg: bool,
        /// Also configure Portage package manager proxy (/etc/portage/make.conf)
        #[arg(long, visible_alias = "emerge")]
        portage: bool,
        /// Also configure Docker Daemon engine proxy (/etc/docker/daemon.json)
        #[arg(long, visible_alias = "daemon")]
        docker_daemon: bool,
        /// Also configure Flatpak package manager proxy (~/.local/share/flatpak/repo/config / /var/lib/flatpak/repo/config)
        #[arg(long, visible_alias = "ostree")]
        flatpak: bool,
        /// Also configure Snap package manager proxy (/etc/systemd/system/snapd.service.d/http-proxy.conf / snap set system proxy.http)
        #[arg(long, visible_alias = "snapd")]
        snap: bool,
        /// Also configure Ollama local AI runner daemon proxy (/etc/systemd/system/ollama.service.d/http-proxy.conf / ~/.ollama/environment)
        #[arg(long, visible_alias = "ollamad")]
        ollama: bool,
        /// Also configure Containerd & CRI-O container runtime daemon proxy (/etc/systemd/system/containerd.service.d/http-proxy.conf / /etc/default/containerd)
        #[arg(long, visible_alias = "crictl", visible_alias = "cri")]
        containerd: bool,
        /// Also configure BuildKit (Docker Buildx / buildctl) container builder daemon proxy (/etc/systemd/system/buildkit.service.d/http-proxy.conf / /etc/default/buildkit)
        #[arg(long, visible_alias = "buildkitd", visible_alias = "buildx")]
        buildkit: bool,
        /// Also configure K3s lightweight Kubernetes node daemon proxy (/etc/systemd/system/k3s.service.env / /etc/systemd/system/k3s.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "k3s-agent", visible_alias = "rancher")]
        k3s: bool,
        /// Also configure MicroK8s lightweight Kubernetes daemon proxy (/var/snap/microk8s/current/args/containerd-env / /var/snap/microk8s/current/args/microk8s-env)
        #[arg(long, visible_alias = "canonical-k8s")]
        microk8s: bool,
        /// Also configure k0s zero-friction Kubernetes daemon proxy (/etc/systemd/system/k0scontroller.service.d/http-proxy.conf / /etc/default/k0scontroller)
        #[arg(long, visible_alias = "mirantis-k8s")]
        k0s: bool,
        /// Also configure RKE2 (Rancher Kubernetes Engine 2) daemon proxy (/etc/rancher/rke2/rke2.env / /etc/systemd/system/rke2-server.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "rke2-server", visible_alias = "rke2-agent")]
        rke2: bool,
        /// Also configure Nomad (HashiCorp Nomad Workload Daemon) proxy (/etc/systemd/system/nomad.service.d/http-proxy.conf / /etc/default/nomad)
        #[arg(long, visible_alias = "nomadd")]
        nomad: bool,
        /// Also configure Kubelet Kubernetes node daemon proxy (/etc/systemd/system/kubelet.service.d/http-proxy.conf / /etc/default/kubelet)
        #[arg(long, visible_alias = "k8s-node")]
        kubelet: bool,
        /// Also configure GitLab Runner (CI/CD Workload Daemon) proxy (/etc/gitlab-runner/config.toml / /etc/systemd/system/gitlab-runner.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "gitlab-ci", visible_alias = "gl-runner")]
        gitlab_runner: bool,
        /// Also configure GitHub Actions Runner (CI/CD Daemon) proxy (~/.actions-runner/.proxy / .env / /etc/systemd/system/actions-runner.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "runner", visible_alias = "actions-runner", visible_alias = "gh-runner")]
        actions_runner: bool,
        /// Also configure Jenkins CI Automation daemon proxy (~/.jenkins/proxy.xml / /etc/default/jenkins / /etc/systemd/system/jenkins.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "jenkins-ci", visible_alias = "jenkins-agent")]
        jenkins: bool,
        /// Also configure Buildkite Agent (CI/CD Hybrid Runner Daemon) proxy (~/.buildkite-agent/buildkite-agent.cfg / /etc/buildkite-agent/buildkite-agent.cfg / /etc/systemd/system/buildkite-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "buildkite", visible_alias = "bk-agent")]
        buildkite_agent: bool,
        /// Also configure Woodpecker CI Runner (CI Pipeline Daemon) proxy (/etc/woodpecker/agent.conf / /etc/default/woodpecker-agent / /etc/systemd/system/woodpecker-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "woodpecker", visible_alias = "wp-agent")]
        woodpecker_agent: bool,
        /// Also configure Drone CI Runner (CI Pipeline Daemon) proxy (/etc/drone-runner/config / /etc/default/drone-runner / /etc/systemd/system/drone-runner.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "drone", visible_alias = "drone-ci")]
        drone_runner: bool,
        /// Also configure Gitea & Forgejo Actions Runner (act_runner CI daemon) proxy (/etc/act_runner/config.yaml / ~/.act_runner/config.yaml / /etc/systemd/system/act-runner.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "act", visible_alias = "gitea-runner", visible_alias = "forgejo-runner")]
        act_runner: bool,
        /// Also configure CircleCI Runner (Self-hosted CI Runner Daemon) proxy (/etc/circleci-runner/circleci-runner-config.yaml / ~/.circleci-runner/circleci-runner-config.yaml / /etc/systemd/system/circleci-runner.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "circleci", visible_alias = "cci-runner")]
        circleci_runner: bool,
        /// Also configure Azure Pipelines Agent (vsts-agent / azp-agent CI runner daemon) proxy (~/azure-pipelines-agent/.proxy / /etc/azure-pipelines-agent/.proxy / /etc/systemd/system/azure-pipelines-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "azure-pipelines", visible_alias = "vsts-agent", visible_alias = "azp-agent", visible_alias = "azure-agent")]
        azure_pipelines_agent: bool,
        /// Also configure JetBrains TeamCity Agent (buildAgent CI runner daemon) proxy (~/buildAgent/conf/buildAgent.properties / /opt/buildagent/conf/buildAgent.properties / /etc/systemd/system/teamcity-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "teamcity", visible_alias = "buildagent", visible_alias = "tc-agent")]
        teamcity_agent: bool,
        /// Also configure Atlassian Bamboo Remote Agent (bamboo-agent CI runner daemon) proxy (~/bamboo-agent-home/bamboo-agent.cfg.xml / /opt/bamboo-agent/conf/wrapper.conf / /etc/systemd/system/bamboo-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "bamboo", visible_alias = "atlassian-agent", visible_alias = "bamboo-agent")]
        bamboo_agent: bool,
        /// Also configure HashiCorp Terraform Cloud / Enterprise Agent (tfc-agent daemon) proxy (/etc/tfc-agent/agent.env / ~/.tfc-agent/agent.env / /etc/systemd/system/tfc-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tfc", visible_alias = "tfc-agent", visible_alias = "terraform-agent")]
        tfc_agent: bool,
        /// Also configure Concourse CI Worker (concourse-worker daemon) proxy (/etc/concourse/worker.env / ~/.concourse/worker.env / /etc/systemd/system/concourse-worker.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "concourse", visible_alias = "concourse-worker")]
        concourse_worker: bool,
        /// Also configure Octopus Deploy Tentacle Agent (tentacle CI/CD deployment runner daemon) proxy (~/octopus/Tentacle/tentacle.config / /etc/octopus/Tentacle/tentacle.config / /etc/systemd/system/tentacle.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tentacle", visible_alias = "octopus", visible_alias = "octopus-tentacle", visible_alias = "tentacle-agent")]
        tentacle_agent: bool,
        /// Also configure Puppet Agent (puppet-agent daemon) proxy (/etc/puppetlabs/puppet/puppet.conf / ~/.puppetlabs/etc/puppet/puppet.conf / /etc/systemd/system/puppet.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "puppet", visible_alias = "puppet-agent")]
        puppet_agent: bool,
        /// Also configure Chef Infra Client (chef-client daemon) proxy (/etc/chef/client.rb / ~/.chef/client.rb / /etc/systemd/system/chef-client.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "chef", visible_alias = "chef-infra", visible_alias = "chef-agent")]
        chef_client: bool,
        /// Also configure SaltStack Salt Minion (salt-minion daemon) proxy (/etc/salt/minion / /etc/salt/minion.d/proxy.conf / /etc/systemd/system/salt-minion.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "salt", visible_alias = "saltstack", visible_alias = "salt-agent")]
        salt_minion: bool,
        /// Also configure Datadog Agent (datadog-agent daemon) proxy (/etc/datadog-agent/datadog.yaml / ~/.datadog-agent/datadog.yaml / /etc/systemd/system/datadog-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "datadog", visible_alias = "datadog-agent", visible_alias = "dd-agent")]
        datadog_agent: bool,
        /// Also configure New Relic Infrastructure Agent (newrelic-infra daemon) proxy (/etc/newrelic-infra.yml / /etc/systemd/system/newrelic-infra.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "newrelic", visible_alias = "newrelic-infra", visible_alias = "nr-agent")]
        newrelic_infra: bool,
        /// Also configure Elastic Agent (Fleet & Beats observability daemon) proxy (/etc/elastic-agent/elastic-agent.yml / ~/.elastic-agent/elastic-agent.yml / /etc/systemd/system/elastic-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "elastic", visible_alias = "fleet-agent", visible_alias = "elastic-agent")]
        elastic_agent: bool,
        /// Also configure Grafana Alloy (Grafana Agent successor telemetry collector daemon) proxy (/etc/alloy/config.alloy / /etc/default/alloy / /etc/systemd/system/alloy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "alloy", visible_alias = "grafana-agent", visible_alias = "grafana-alloy")]
        grafana_alloy: bool,
        /// Also configure OpenTelemetry Collector (otelcol / otelcol-contrib daemon) proxy (/etc/otelcol/config.yaml / /etc/default/otelcol / /etc/systemd/system/otelcol.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "otel", visible_alias = "opentelemetry", visible_alias = "otel-collector")]
        otelcol: bool,
        /// Also configure Vector (vector telemetry & pipeline daemon) proxy (/etc/vector/vector.yaml / ~/.config/vector/vector.yaml / /etc/systemd/system/vector.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "timber-vector")]
        vector: bool,
        /// Also configure Fluent Bit (fluent-bit telemetry & log processor daemon) proxy (/etc/fluent-bit/fluent-bit.conf / /etc/fluent-bit/fluent-bit.yaml / /etc/systemd/system/fluent-bit.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "flb", visible_alias = "fluentbit")]
        fluent_bit: bool,
        /// Also configure Fluentd (fluentd / td-agent / fluent-package logging framework daemon) proxy (/etc/fluentd/fluent.conf / /etc/td-agent/td-agent.conf / /etc/systemd/system/fluentd.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "td-agent", visible_alias = "fluent-package")]
        fluentd: bool,
        /// Also configure Telegraf (InfluxData telegraf metrics collection agent daemon) proxy (/etc/telegraf/telegraf.conf / ~/.config/telegraf/telegraf.conf / /etc/systemd/system/telegraf.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "influx-telegraf")]
        telegraf: bool,
        /// Also configure Logstash (logstash log processing & ingestion pipeline daemon) proxy (/etc/logstash/logstash.yml / /etc/logstash/jvm.options / /etc/systemd/system/logstash.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "elastic-logstash")]
        logstash: bool,
        /// Also configure Prometheus (Prometheus monitoring system & time series database daemon) proxy (/etc/prometheus/prometheus.yml / ~/.config/prometheus/prometheus.yml / /etc/systemd/system/prometheus.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "prom", visible_alias = "prometheus-server")]
        prometheus: bool,
        /// Also configure Alertmanager (Prometheus Alertmanager alert routing & notification daemon) proxy (/etc/alertmanager/alertmanager.yml / ~/.config/alertmanager/alertmanager.yml / /etc/systemd/system/alertmanager.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "alert-manager", visible_alias = "prom-alertmanager")]
        alertmanager: bool,
        /// Also configure Promtail (Grafana Promtail log collection & shipping daemon) proxy (/etc/promtail/config.yml / ~/.config/promtail/config.yml / /etc/systemd/system/promtail.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "prom-tail", visible_alias = "loki-promtail")]
        promtail: bool,
        /// Also configure VictoriaMetrics Agent (vmagent Prometheus-compatible metric scraper & forwarder daemon) proxy (/etc/vmagent/vmagent.yml / ~/.config/vmagent/vmagent.yml / /etc/systemd/system/vmagent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "victoria-agent", visible_alias = "victoria-metrics-agent")]
        vmagent: bool,
        /// Also configure Filebeat (Elastic Filebeat log harvester & shipping daemon) proxy (/etc/filebeat/filebeat.yml / ~/.filebeat/filebeat.yml / /etc/systemd/system/filebeat.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "file-beat", visible_alias = "elastic-filebeat")]
        filebeat: bool,
        /// Also configure Metricbeat (Elastic Metricbeat system & service metrics shipping daemon) proxy (/etc/metricbeat/metricbeat.yml / ~/.metricbeat/metricbeat.yml / /etc/systemd/system/metricbeat.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "metric-beat", visible_alias = "elastic-metricbeat")]
        metricbeat: bool,
        /// Also configure Heartbeat (Elastic Heartbeat uptime & synthetic monitoring daemon) proxy (/etc/heartbeat/heartbeat.yml / ~/.heartbeat/heartbeat.yml / /etc/systemd/system/heartbeat.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "heart-beat", visible_alias = "elastic-heartbeat")]
        heartbeat: bool,
        /// Also configure Auditbeat (Elastic Auditbeat audit data & security monitoring daemon) proxy (/etc/auditbeat/auditbeat.yml / ~/.auditbeat/auditbeat.yml / /etc/systemd/system/auditbeat.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "audit-beat", visible_alias = "elastic-auditbeat")]
        auditbeat: bool,
        /// Also configure Packetbeat (Elastic Packetbeat network packet analyzer & wire data shipping daemon) proxy (/etc/packetbeat/packetbeat.yml / ~/.packetbeat/packetbeat.yml / /etc/systemd/system/packetbeat.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "packet-beat", visible_alias = "elastic-packetbeat")]
        packetbeat: bool,
        /// Also configure Winlogbeat (Elastic Winlogbeat Windows event log shipping daemon) proxy (/etc/winlogbeat/winlogbeat.yml / ~/.winlogbeat/winlogbeat.yml / /etc/systemd/system/winlogbeat.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "winlog-beat", visible_alias = "elastic-winlogbeat")]
        winlogbeat: bool,
        /// Also configure HashiCorp Consul service mesh & discovery daemon proxy (/etc/consul.d/consul.hcl / ~/.consul.d/consul.hcl / /etc/systemd/system/consul.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "consul-agent", visible_alias = "hashicorp-consul")]
        consul: bool,
        /// Also configure HashiCorp Vault secret management daemon proxy (/etc/vault.d/vault.hcl / ~/.vault.d/vault.hcl / /etc/systemd/system/vault.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "vault-agent", visible_alias = "hashicorp-vault")]
        vault: bool,
        /// Also configure Traefik edge router & ingress daemon proxy (/etc/traefik/traefik.yml / ~/.config/traefik/traefik.yml / /etc/systemd/system/traefik.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "traefik-proxy", visible_alias = "traefik-ingress")]
        traefik: bool,
        /// Also configure Caddy enterprise web server daemon proxy (/etc/caddy/Caddyfile / ~/.config/caddy/Caddyfile / /etc/systemd/system/caddy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "caddy-server", visible_alias = "caddyserver")]
        caddy: bool,
        /// Also configure Envoy cloud-native service proxy (/etc/envoy/envoy.yaml / ~/.config/envoy/envoy.yaml / /etc/systemd/system/envoy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "envoy-proxy", visible_alias = "envoyproxy")]
        envoy: bool,
        /// Also configure HAProxy load balancer daemon proxy (/etc/haproxy/haproxy.cfg / ~/.config/haproxy/haproxy.cfg / /etc/systemd/system/haproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "ha-proxy", visible_alias = "haproxy-server")]
        haproxy: bool,
        /// Also configure NGINX web server & reverse proxy daemon (/etc/nginx/nginx.conf / /etc/nginx/conf.d/proxy.conf / /etc/systemd/system/nginx.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "nginx-server", visible_alias = "nginx-proxy")]
        nginx: bool,
        /// Also configure Kong API gateway daemon proxy (/etc/kong/kong.conf / ~/.kong/kong.conf / /etc/systemd/system/kong.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "kong-gateway", visible_alias = "kong-api")]
        kong: bool,
        /// Also configure Apache APISIX API gateway daemon proxy (/etc/apisix/config.yaml / /usr/local/apisix/conf/config.yaml / /etc/systemd/system/apisix.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "apisix-gateway", visible_alias = "apisix-proxy")]
        apisix: bool,
        /// Also configure Squid caching proxy daemon (/etc/squid/squid.conf / /etc/squid/conf.d/proxy.conf / /etc/systemd/system/squid.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "squid-proxy", visible_alias = "squid-cache")]
        squid: bool,
        /// Also configure Privoxy forwarding proxy daemon (/etc/privoxy/config / /usr/local/etc/privoxy/config / /opt/homebrew/etc/privoxy/config / /etc/systemd/system/privoxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "privoxy-proxy", visible_alias = "privoxy-daemon")]
        privoxy: bool,
        /// Also configure Tinyproxy HTTP proxy daemon (/etc/tinyproxy/tinyproxy.conf / /etc/tinyproxy.conf / /usr/local/etc/tinyproxy.conf / /etc/systemd/system/tinyproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tinyproxy-proxy", visible_alias = "tinyproxy-daemon")]
        tinyproxy: bool,
        /// Also configure Mitmproxy HTTPS intercepting proxy daemon (~/.mitmproxy/config.yaml / /etc/mitmproxy/config.yaml / /etc/systemd/system/mitmproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "mitmproxy-daemon", visible_alias = "mitmdump")]
        mitmproxy: bool,
        /// Also configure Tor anonymity network proxy daemon (/etc/tor/torrc / /usr/local/etc/tor/torrc / ~/.torrc / /etc/systemd/system/tor.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tor-daemon", visible_alias = "torrc")]
        tor: bool,
        /// Also configure Cloudflare Tunnel daemon proxy (~/.cloudflared/config.yml / /etc/cloudflared/config.yml / /etc/systemd/system/cloudflared.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "cf-tunnel", visible_alias = "tunnel")]
        cloudflared: bool,
        /// Also configure Ngrok tunnel daemon proxy (~/.config/ngrok/ngrok.yml / ~/Library/Application Support/ngrok/ngrok.yml / /etc/systemd/system/ngrok.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "ngrok-tunnel", visible_alias = "ngrok-agent")]
        ngrok: bool,
        /// Also configure FRP (Fast Reverse Proxy) client daemon (/etc/frp/frpc.toml / ~/.frp/frpc.toml / /etc/systemd/system/frpc.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "frp", visible_alias = "frp-client")]
        frpc: bool,
        /// Also configure Tailscale daemon proxy (/etc/default/tailscaled / /etc/sysconfig/tailscaled / /etc/systemd/system/tailscaled.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tailscaled", visible_alias = "ts")]
        tailscale: bool,
        /// Also configure ZeroTier One daemon proxy (/var/lib/zerotier-one/local.conf / /Library/Application Support/ZeroTier/One/local.conf / /etc/systemd/system/zerotier-one.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "zerotier-one", visible_alias = "zt")]
        zerotier: bool,
        /// Also configure Gost (GO Simple Tunnel) daemon proxy (/etc/gost/gost.json / ~/.config/gost/config.json / /etc/systemd/system/gost.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "go-gost")]
        gost: bool,
        /// Also configure Rathole NAT traversal daemon proxy (/etc/rathole/rathole.toml / ~/.config/rathole/rathole.toml / /etc/systemd/system/rathole.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "rat")]
        rathole: bool,
        /// Also configure Chisel HTTP tunnel daemon proxy (/etc/chisel/config.json / ~/.config/chisel/config.json / /etc/systemd/system/chisel.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "chisel-tunnel")]
        chisel: bool,
        /// Also configure Nebula mesh overlay daemon proxy (/etc/nebula/config.yaml / ~/.config/nebula/config.yaml / /etc/systemd/system/nebula.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "neb", visible_alias = "slack-nebula")]
        nebula: bool,
        /// Also configure Bore modern tunnel daemon proxy (/etc/bore/bore.toml / ~/.config/bore/bore.toml / /etc/systemd/system/bore.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "bore-tunnel")]
        bore: bool,
        /// Also configure Brook proxy daemon proxy (/etc/brook/config.json / ~/.config/brook/config.json / /etc/systemd/system/brook.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "brook-proxy")]
        brook: bool,
        /// Also configure NPC (NPS Client) daemon proxy (/etc/npc/conf/npc.conf / ~/.npc/npc.conf / /etc/systemd/system/npc.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "nps-npc", visible_alias = "nps")]
        npc: bool,
        /// Also configure Shadowsocks daemon proxy (/etc/shadowsocks-rust/config.json / ~/.config/shadowsocks-rust/config.json / /etc/systemd/system/shadowsocks-rust.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "shadowsocks-proxy", visible_alias = "ss")]
        shadowsocks: bool,
        /// Also configure V2Ray & Xray daemon proxy (/etc/v2ray/config.json / ~/.config/v2ray/config.json / /etc/systemd/system/v2ray.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "v2ray-proxy", visible_alias = "xray")]
        v2ray: bool,
        /// Also configure Sing-box daemon proxy (/etc/sing-box/config.json / ~/.config/sing-box/config.json / /etc/systemd/system/sing-box.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "singbox", visible_alias = "sb")]
        sing_box: bool,
        /// Also configure Hysteria & Hysteria 2 daemon proxy (/etc/hysteria/config.yaml / ~/.config/hysteria/config.yaml / /etc/systemd/system/hysteria.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "hysteria2", visible_alias = "hy2")]
        hysteria: bool,
        /// Also configure Trojan & Trojan-Go daemon proxy (/etc/trojan/config.json / ~/.config/trojan/config.json / /etc/systemd/system/trojan.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "trojan-go", visible_alias = "tg")]
        trojan: bool,
        /// Also configure NaïveProxy daemon proxy (/etc/naiveproxy/config.json / ~/.config/naiveproxy/config.json / /etc/systemd/system/naiveproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "naiveproxy", visible_alias = "naive-proxy")]
        naive: bool,
        /// Also configure Tuic daemon proxy (/etc/tuic/client.json / ~/.config/tuic/client.json / /etc/systemd/system/tuic.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tuic-client")]
        tuic: bool,
        /// Also configure Juicity daemon proxy (/etc/juicity/client.json / ~/.config/juicity/client.json / /etc/systemd/system/juicity.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "juicity-client")]
        juicity: bool,
        /// Also configure Snell daemon proxy (/etc/snell/snell.conf / ~/.config/snell/snell.conf / /etc/systemd/system/snell.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "snell-server", visible_alias = "snell-client")]
        snell: bool,
        /// Also configure Leaf daemon proxy (/etc/leaf/leaf.conf / ~/.config/leaf/leaf.conf / /etc/systemd/system/leaf.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "leaf-proxy", visible_alias = "leaf-daemon")]
        leaf: bool,
        /// Also configure git, cargo, npm, yarn, pnpm, bun, pip, conda, docker, gradle, maven, vscode, gh, glab, jetbrains, nuget, gem, svn, hg, uv, go, bazel, composer, conan, sbt, kube, julia, nix, cabal, stack, hex, r, opam, lein, cpan, luarocks, nimble, rebar3, scoop, bundle, gcloud, aws, azure, mamba, podman, vagrant, choco, colima, minikube, spack, macports, apt, dnf, zypper, pacman, pkg, portage, docker daemon, flatpak, snap, ollama, containerd, buildkit, k3s, microk8s, k0s, rke2, nomad, kubelet, gitlab-runner, actions-runner, jenkins, buildkite-agent, woodpecker-agent, drone-runner, act-runner, circleci-runner, azure-pipelines-agent, teamcity-agent, bamboo-agent, tfc-agent, concourse-worker, tentacle-agent, puppet-agent, chef-client, salt-minion, datadog-agent, newrelic-infra, elastic-agent, grafana-alloy, otelcol, vector, fluent-bit, fluentd, telegraf, logstash, prometheus, alertmanager, promtail, vmagent, filebeat, metricbeat, heartbeat, auditbeat, packetbeat, winlogbeat, consul, vault, traefik, caddy, envoy, haproxy, nginx, kong, apisix, squid, privoxy, tinyproxy, mitmproxy, tor, cloudflared, ngrok, frpc, tailscale, zerotier, gost, rathole, chisel, nebula, bore, brook, npc, shadowsocks, v2ray, sing-box, hysteria, trojan, naive, tuic, juicity, snell, and leaf proxy
        #[arg(short, long)]
        all: bool,
        /// Shell format for environment variables: bash, zsh, fish, pwsh (auto-detected if omitted)
        #[arg(short, long)]
        shell: Option<String>,
    },
    /// 🔴 Output shell unset commands to turn OFF proxy for current session
    Off {
        /// Also unset git global proxy
        #[arg(long)]
        git: bool,
        /// Also unset cargo proxy (~/.cargo/config.toml)
        #[arg(long)]
        cargo: bool,
        /// Also unset npm proxy
        #[arg(long)]
        npm: bool,
        /// Also unset Yarn proxy (~/.yarnrc / ~/.yarnrc.yml)
        #[arg(long)]
        yarn: bool,
        /// Also unset pnpm proxy (~/.pnpmrc)
        #[arg(long)]
        pnpm: bool,
        /// Also unset Bun proxy (~/.bunfig.toml)
        #[arg(long)]
        bun: bool,
        /// Also unset pip global proxy
        #[arg(long)]
        pip: bool,
        /// Also unset Conda proxy (~/.condarc)
        #[arg(long)]
        conda: bool,
        /// Also unset Docker client proxy (~/.docker/config.json)
        #[arg(long)]
        docker: bool,
        /// Also unset Gradle global proxy (~/.gradle/gradle.properties)
        #[arg(long)]
        gradle: bool,
        /// Also unset Maven global proxy (~/.m2/settings.xml)
        #[arg(long)]
        maven: bool,
        /// Also unset VS Code and Cursor IDE proxy (settings.json)
        #[arg(long)]
        vscode: bool,
        /// Also unset GitHub CLI (gh) proxy (~/.config/gh/config.yml)
        #[arg(long)]
        gh: bool,
        /// Also unset GitLab CLI (glab) proxy (~/.config/glab-cli/config.yml)
        #[arg(long)]
        glab: bool,
        /// Also unset JetBrains & Android Studio IDE proxy (proxy.settings.xml)
        #[arg(long)]
        jetbrains: bool,
        /// Also unset NuGet (.NET) proxy (NuGet.Config)
        #[arg(long)]
        nuget: bool,
        /// Also unset cURL proxy (~/.curlrc)
        #[arg(long)]
        curl: bool,
        /// Also unset Wget proxy (~/.wgetrc)
        #[arg(long)]
        wget: bool,
        /// Also unset Gem (Ruby) proxy (~/.gemrc)
        #[arg(long)]
        gem: bool,
        /// Also unset Subversion (SVN) proxy (~/.subversion/servers)
        #[arg(long)]
        svn: bool,
        /// Also unset Mercurial (Hg) proxy (~/.hgrc)
        #[arg(long)]
        hg: bool,
        /// Also unset Astral uv proxy (~/.config/uv/uv.toml)
        #[arg(long)]
        uv: bool,
        /// Also unset Go modules proxy (go/env & GOPROXY)
        #[arg(long)]
        go: bool,
        /// Also unset Bazel proxy (~/.bazelrc action_env)
        #[arg(long)]
        bazel: bool,
        /// Also unset PHP Composer proxy (config.json)
        #[arg(long)]
        composer: bool,
        /// Also unset Conan C/C++ proxy (global.conf / conan.conf)
        #[arg(long)]
        conan: bool,
        /// Also unset Scala Build Tool (sbt) proxy (~/.sbt/sbtopts)
        #[arg(long)]
        sbt: bool,
        /// Also unset Kubernetes (kubectl) cluster proxy (~/.kube/config proxy-url)
        #[arg(long)]
        kube: bool,
        /// Also unset Julia proxy (~/.julia/config/startup.jl)
        #[arg(long)]
        julia: bool,
        /// Also unset Nix package manager proxy (~/.config/nix/nix.conf)
        #[arg(long)]
        nix: bool,
        /// Also unset Haskell Cabal proxy (~/.cabal/config)
        #[arg(long)]
        cabal: bool,
        /// Also unset Haskell Stack proxy (~/.stack/config.yaml)
        #[arg(long)]
        stack: bool,
        /// Also unset Elixir / Erlang / Gleam Hex proxy (~/.hex/hex.config)
        #[arg(long)]
        hex: bool,
        /// Also unset R package manager proxy (~/.Renviron)
        #[arg(long)]
        r: bool,
        /// Also unset OCaml OPAM proxy (~/.opam/config)
        #[arg(long)]
        opam: bool,
        /// Also unset Clojure Leiningen proxy (~/.lein/profiles.clj)
        #[arg(long)]
        lein: bool,
        /// Also unset Perl CPAN package manager proxy (~/.cpan/CPAN/MyConfig.pm)
        #[arg(long)]
        cpan: bool,
        /// Also unset Lua (LuaRocks) package manager proxy (~/.luarocks/config.lua)
        #[arg(long)]
        luarocks: bool,
        /// Also unset Nim (Nimble) package manager proxy (~/.config/nimble/nimble.ini)
        #[arg(long)]
        nimble: bool,
        /// Also unset Erlang (Rebar3) package manager proxy (~/.config/rebar3/rebar.config)
        #[arg(long)]
        rebar3: bool,
        /// Also unset Scoop (Windows Package Manager) proxy (~/.config/scoop/config.json)
        #[arg(long)]
        scoop: bool,
        /// Also unset Ruby Bundler proxy (~/.bundle/config)
        #[arg(long, visible_alias = "bundler")]
        bundle: bool,
        /// Also unset Google Cloud SDK (gcloud) proxy (~/.config/gcloud/configurations/config_*)
        #[arg(long)]
        gcloud: bool,
        /// Also unset AWS CLI proxy (~/.aws/config)
        #[arg(long)]
        aws: bool,
        /// Also unset Microsoft Azure CLI (az) proxy (~/.azure/config)
        #[arg(long, visible_alias = "az")]
        azure: bool,
        /// Also unset Mamba / Micromamba proxy (~/.mambarc)
        #[arg(long, visible_alias = "micromamba")]
        mamba: bool,
        /// Also unset Podman container engine proxy (~/.config/containers/containers.conf)
        #[arg(long)]
        podman: bool,
        /// Also unset Vagrant proxy (~/.vagrant.d/Vagrantfile)
        #[arg(long)]
        vagrant: bool,
        /// Also unset Chocolatey package manager proxy (chocolatey.config)
        #[arg(long, visible_alias = "chocolatey")]
        choco: bool,
        /// Also unset Colima container runtime proxy (~/.colima/default/colima.yaml)
        #[arg(long)]
        colima: bool,
        /// Also unset Minikube cluster proxy (~/.minikube/config/config.json)
        #[arg(long)]
        minikube: bool,
        /// Also unset Spack scientific package manager proxy (~/.spack/config.yaml)
        #[arg(long)]
        spack: bool,
        /// Also unset MacPorts package manager proxy (~/.macports/macports.conf)
        #[arg(long)]
        macports: bool,
        /// Also unset APT package manager proxy (/etc/apt/apt.conf.d/99proxydoc.conf)
        #[arg(long)]
        apt: bool,
        /// Also unset DNF & YUM package manager proxy (/etc/dnf/dnf.conf)
        #[arg(long)]
        dnf: bool,
        /// Also unset Zypper package manager proxy (/etc/zypp/zypp.conf)
        #[arg(long)]
        zypper: bool,
        /// Also unset Pacman package manager proxy (/etc/pacman.conf)
        #[arg(long)]
        pacman: bool,
        /// Also unset FreeBSD pkg package manager proxy (/usr/local/etc/pkg.conf)
        #[arg(long)]
        pkg: bool,
        /// Also unset Portage package manager proxy (/etc/portage/make.conf)
        #[arg(long, visible_alias = "emerge")]
        portage: bool,
        /// Also unset Docker Daemon engine proxy (/etc/docker/daemon.json)
        #[arg(long, visible_alias = "daemon")]
        docker_daemon: bool,
        /// Also unset Flatpak package manager proxy (~/.local/share/flatpak/repo/config / /var/lib/flatpak/repo/config)
        #[arg(long, visible_alias = "ostree")]
        flatpak: bool,
        /// Also unset Snap package manager proxy (/etc/systemd/system/snapd.service.d/http-proxy.conf / snap set system proxy.http)
        #[arg(long, visible_alias = "snapd")]
        snap: bool,
        /// Also unset Ollama local AI runner daemon proxy (/etc/systemd/system/ollama.service.d/http-proxy.conf / ~/.ollama/environment)
        #[arg(long, visible_alias = "ollamad")]
        ollama: bool,
        /// Also unset Containerd & CRI-O container runtime daemon proxy (/etc/systemd/system/containerd.service.d/http-proxy.conf / /etc/default/containerd)
        #[arg(long, visible_alias = "crictl", visible_alias = "cri")]
        containerd: bool,
        /// Also unset BuildKit (Docker Buildx / buildctl) container builder daemon proxy (/etc/systemd/system/buildkit.service.d/http-proxy.conf / /etc/default/buildkit)
        #[arg(long, visible_alias = "buildkitd", visible_alias = "buildx")]
        buildkit: bool,
        /// Also unset K3s lightweight Kubernetes node daemon proxy (/etc/systemd/system/k3s.service.env / /etc/systemd/system/k3s.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "k3s-agent", visible_alias = "rancher")]
        k3s: bool,
        /// Also unset MicroK8s lightweight Kubernetes daemon proxy (/var/snap/microk8s/current/args/containerd-env / /var/snap/microk8s/current/args/microk8s-env)
        #[arg(long, visible_alias = "canonical-k8s")]
        microk8s: bool,
        /// Also unset k0s zero-friction Kubernetes daemon proxy (/etc/systemd/system/k0scontroller.service.d/http-proxy.conf / /etc/default/k0scontroller)
        #[arg(long, visible_alias = "mirantis-k8s")]
        k0s: bool,
        /// Also unset RKE2 (Rancher Kubernetes Engine 2) daemon proxy (/etc/rancher/rke2/rke2.env / /etc/systemd/system/rke2-server.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "rke2-server", visible_alias = "rke2-agent")]
        rke2: bool,
        /// Also unset Nomad (HashiCorp Nomad Workload Daemon) proxy (/etc/systemd/system/nomad.service.d/http-proxy.conf / /etc/default/nomad)
        #[arg(long, visible_alias = "nomadd")]
        nomad: bool,
        /// Also unset Kubelet Kubernetes node daemon proxy (/etc/systemd/system/kubelet.service.d/http-proxy.conf / /etc/default/kubelet)
        #[arg(long, visible_alias = "k8s-node")]
        kubelet: bool,
        /// Also unset GitLab Runner (CI/CD Workload Daemon) proxy (/etc/gitlab-runner/config.toml / /etc/systemd/system/gitlab-runner.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "gitlab-ci", visible_alias = "gl-runner")]
        gitlab_runner: bool,
        /// Also unset GitHub Actions Runner (CI/CD Daemon) proxy (~/.actions-runner/.proxy / .env / /etc/systemd/system/actions-runner.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "runner", visible_alias = "actions-runner", visible_alias = "gh-runner")]
        actions_runner: bool,
        /// Also unset Jenkins CI Automation daemon proxy (~/.jenkins/proxy.xml / /etc/default/jenkins / /etc/systemd/system/jenkins.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "jenkins-ci", visible_alias = "jenkins-agent")]
        jenkins: bool,
        /// Also unset Buildkite Agent (CI/CD Hybrid Runner Daemon) proxy (~/.buildkite-agent/buildkite-agent.cfg / /etc/buildkite-agent/buildkite-agent.cfg / /etc/systemd/system/buildkite-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "buildkite", visible_alias = "bk-agent")]
        buildkite_agent: bool,
        /// Also unset Woodpecker CI Runner (CI Pipeline Daemon) proxy (/etc/woodpecker/agent.conf / /etc/default/woodpecker-agent / /etc/systemd/system/woodpecker-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "woodpecker", visible_alias = "wp-agent")]
        woodpecker_agent: bool,
        /// Also unset Drone CI Runner (CI Pipeline Daemon) proxy (/etc/drone-runner/config / /etc/default/drone-runner / /etc/systemd/system/drone-runner.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "drone", visible_alias = "drone-ci")]
        drone_runner: bool,
        /// Also unset Gitea & Forgejo Actions Runner (act_runner CI daemon) proxy (/etc/act_runner/config.yaml / ~/.act_runner/config.yaml / /etc/systemd/system/act-runner.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "act", visible_alias = "gitea-runner", visible_alias = "forgejo-runner")]
        act_runner: bool,
        /// Also unset CircleCI Runner (Self-hosted CI Runner Daemon) proxy (/etc/circleci-runner/circleci-runner-config.yaml / ~/.circleci-runner/circleci-runner-config.yaml / /etc/systemd/system/circleci-runner.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "circleci", visible_alias = "cci-runner")]
        circleci_runner: bool,
        /// Also unset Azure Pipelines Agent (vsts-agent / azp-agent CI runner daemon) proxy (~/azure-pipelines-agent/.proxy / /etc/azure-pipelines-agent/.proxy / /etc/systemd/system/azure-pipelines-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "azure-pipelines", visible_alias = "vsts-agent", visible_alias = "azp-agent", visible_alias = "azure-agent")]
        azure_pipelines_agent: bool,
        /// Also unset JetBrains TeamCity Agent (buildAgent CI runner daemon) proxy (~/buildAgent/conf/buildAgent.properties / /opt/buildagent/conf/buildAgent.properties / /etc/systemd/system/teamcity-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "teamcity", visible_alias = "buildagent", visible_alias = "tc-agent")]
        teamcity_agent: bool,
        /// Also unset Atlassian Bamboo Remote Agent (bamboo-agent CI runner daemon) proxy (~/bamboo-agent-home/bamboo-agent.cfg.xml / /opt/bamboo-agent/conf/wrapper.conf / /etc/systemd/system/bamboo-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "bamboo", visible_alias = "atlassian-agent", visible_alias = "bamboo-agent")]
        bamboo_agent: bool,
        /// Also unset HashiCorp Terraform Cloud / Enterprise Agent (tfc-agent daemon) proxy (/etc/tfc-agent/agent.env / ~/.tfc-agent/agent.env / /etc/systemd/system/tfc-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tfc", visible_alias = "tfc-agent", visible_alias = "terraform-agent")]
        tfc_agent: bool,
        /// Also unset Concourse CI Worker (concourse-worker daemon) proxy (/etc/concourse/worker.env / ~/.concourse/worker.env / /etc/systemd/system/concourse-worker.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "concourse", visible_alias = "concourse-worker")]
        concourse_worker: bool,
        /// Also unset Octopus Deploy Tentacle Agent (tentacle CI/CD deployment runner daemon) proxy (~/octopus/Tentacle/tentacle.config / /etc/octopus/Tentacle/tentacle.config / /etc/systemd/system/tentacle.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tentacle", visible_alias = "octopus", visible_alias = "octopus-tentacle", visible_alias = "tentacle-agent")]
        tentacle_agent: bool,
        /// Also unset Puppet Agent (puppet-agent daemon) proxy (/etc/puppetlabs/puppet/puppet.conf / ~/.puppetlabs/etc/puppet/puppet.conf / /etc/systemd/system/puppet.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "puppet", visible_alias = "puppet-agent")]
        puppet_agent: bool,
        /// Also unset Chef Infra Client (chef-client daemon) proxy (/etc/chef/client.rb / ~/.chef/client.rb / /etc/systemd/system/chef-client.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "chef", visible_alias = "chef-infra", visible_alias = "chef-agent")]
        chef_client: bool,
        /// Also unset SaltStack Salt Minion (salt-minion daemon) proxy (/etc/salt/minion / /etc/salt/minion.d/proxy.conf / /etc/systemd/system/salt-minion.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "salt", visible_alias = "saltstack", visible_alias = "salt-agent")]
        salt_minion: bool,
        /// Also unset Datadog Agent (datadog-agent daemon) proxy (/etc/datadog-agent/datadog.yaml / ~/.datadog-agent/datadog.yaml / /etc/systemd/system/datadog-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "datadog", visible_alias = "datadog-agent", visible_alias = "dd-agent")]
        datadog_agent: bool,
        /// Also unset New Relic Infrastructure Agent (newrelic-infra daemon) proxy (/etc/newrelic-infra.yml / /etc/systemd/system/newrelic-infra.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "newrelic", visible_alias = "newrelic-infra", visible_alias = "nr-agent")]
        newrelic_infra: bool,
        /// Also unset Elastic Agent (Fleet & Beats observability daemon) proxy (/etc/elastic-agent/elastic-agent.yml / ~/.elastic-agent/elastic-agent.yml / /etc/systemd/system/elastic-agent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "elastic", visible_alias = "fleet-agent", visible_alias = "elastic-agent")]
        elastic_agent: bool,
        /// Also unset Grafana Alloy (Grafana Agent successor telemetry collector daemon) proxy (/etc/alloy/config.alloy / /etc/default/alloy / /etc/systemd/system/alloy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "alloy", visible_alias = "grafana-agent", visible_alias = "grafana-alloy")]
        grafana_alloy: bool,
        /// Also unset OpenTelemetry Collector (otelcol / otelcol-contrib daemon) proxy (/etc/otelcol/config.yaml / /etc/default/otelcol / /etc/systemd/system/otelcol.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "otel", visible_alias = "opentelemetry", visible_alias = "otel-collector")]
        otelcol: bool,
        /// Also unset Vector (vector telemetry & pipeline daemon) proxy (/etc/vector/vector.yaml / ~/.config/vector/vector.yaml / /etc/systemd/system/vector.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "timber-vector")]
        vector: bool,
        /// Also unset Fluent Bit (fluent-bit telemetry & log processor daemon) proxy (/etc/fluent-bit/fluent-bit.conf / /etc/fluent-bit/fluent-bit.yaml / /etc/systemd/system/fluent-bit.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "flb", visible_alias = "fluentbit")]
        fluent_bit: bool,
        /// Also unset Fluentd (fluentd / td-agent / fluent-package logging framework daemon) proxy (/etc/fluentd/fluent.conf / /etc/td-agent/td-agent.conf / /etc/systemd/system/fluentd.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "td-agent", visible_alias = "fluent-package")]
        fluentd: bool,
        /// Also unset Telegraf (InfluxData telegraf metrics collection agent daemon) proxy (/etc/telegraf/telegraf.conf / ~/.config/telegraf/telegraf.conf / /etc/systemd/system/telegraf.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "influx-telegraf")]
        telegraf: bool,
        /// Also unset Logstash (logstash log processing & ingestion pipeline daemon) proxy (/etc/logstash/logstash.yml / /etc/logstash/jvm.options / /etc/systemd/system/logstash.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "elastic-logstash")]
        logstash: bool,
        /// Also unset Prometheus (Prometheus monitoring system & time series database daemon) proxy (/etc/prometheus/prometheus.yml / ~/.config/prometheus/prometheus.yml / /etc/systemd/system/prometheus.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "prom", visible_alias = "prometheus-server")]
        prometheus: bool,
        /// Also unset Alertmanager (Prometheus Alertmanager alert routing & notification daemon) proxy (/etc/alertmanager/alertmanager.yml / ~/.config/alertmanager/alertmanager.yml / /etc/systemd/system/alertmanager.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "alert-manager", visible_alias = "prom-alertmanager")]
        alertmanager: bool,
        /// Also unset Promtail (Grafana Promtail log collection & shipping daemon) proxy (/etc/promtail/config.yml / ~/.config/promtail/config.yml / /etc/systemd/system/promtail.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "prom-tail", visible_alias = "loki-promtail")]
        promtail: bool,
        /// Also unset VictoriaMetrics Agent (vmagent Prometheus-compatible metric scraper & forwarder daemon) proxy (/etc/vmagent/vmagent.yml / ~/.config/vmagent/vmagent.yml / /etc/systemd/system/vmagent.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "victoria-agent", visible_alias = "victoria-metrics-agent")]
        vmagent: bool,
        /// Also unset Filebeat (Elastic Filebeat log harvester & shipping daemon) proxy (/etc/filebeat/filebeat.yml / ~/.filebeat/filebeat.yml / /etc/systemd/system/filebeat.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "file-beat", visible_alias = "elastic-filebeat")]
        filebeat: bool,
        /// Also unset Metricbeat (Elastic Metricbeat system & service metrics shipping daemon) proxy (/etc/metricbeat/metricbeat.yml / ~/.metricbeat/metricbeat.yml / /etc/systemd/system/metricbeat.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "metric-beat", visible_alias = "elastic-metricbeat")]
        metricbeat: bool,
        /// Also unset Heartbeat (Elastic Heartbeat uptime & synthetic monitoring daemon) proxy (/etc/heartbeat/heartbeat.yml / ~/.heartbeat/heartbeat.yml / /etc/systemd/system/heartbeat.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "heart-beat", visible_alias = "elastic-heartbeat")]
        heartbeat: bool,
        /// Also unset Auditbeat (Elastic Auditbeat audit data & security monitoring daemon) proxy (/etc/auditbeat/auditbeat.yml / ~/.auditbeat/auditbeat.yml / /etc/systemd/system/auditbeat.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "audit-beat", visible_alias = "elastic-auditbeat")]
        auditbeat: bool,
        /// Also unset Packetbeat (Elastic Packetbeat network packet analyzer & wire data shipping daemon) proxy (/etc/packetbeat/packetbeat.yml / ~/.packetbeat/packetbeat.yml / /etc/systemd/system/packetbeat.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "packet-beat", visible_alias = "elastic-packetbeat")]
        packetbeat: bool,
        /// Also unset Winlogbeat (Elastic Winlogbeat Windows event log shipping daemon) proxy (/etc/winlogbeat/winlogbeat.yml / ~/.winlogbeat/winlogbeat.yml / /etc/systemd/system/winlogbeat.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "winlog-beat", visible_alias = "elastic-winlogbeat")]
        winlogbeat: bool,
        /// Also unset HashiCorp Consul service mesh & discovery daemon proxy (/etc/consul.d/consul.hcl / ~/.consul.d/consul.hcl / /etc/systemd/system/consul.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "consul-agent", visible_alias = "hashicorp-consul")]
        consul: bool,
        /// Also unset HashiCorp Vault secret management daemon proxy (/etc/vault.d/vault.hcl / ~/.vault.d/vault.hcl / /etc/systemd/system/vault.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "vault-agent", visible_alias = "hashicorp-vault")]
        vault: bool,
        /// Also unset Traefik edge router & ingress daemon proxy (/etc/traefik/traefik.yml / ~/.config/traefik/traefik.yml / /etc/systemd/system/traefik.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "traefik-proxy", visible_alias = "traefik-ingress")]
        traefik: bool,
        /// Also unset Caddy enterprise web server daemon proxy (/etc/caddy/Caddyfile / ~/.config/caddy/Caddyfile / /etc/systemd/system/caddy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "caddy-server", visible_alias = "caddyserver")]
        caddy: bool,
        /// Also unset Envoy cloud-native service proxy (/etc/envoy/envoy.yaml / ~/.config/envoy/envoy.yaml / /etc/systemd/system/envoy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "envoy-proxy", visible_alias = "envoyproxy")]
        envoy: bool,
        /// Also unset HAProxy load balancer daemon proxy (/etc/haproxy/haproxy.cfg / ~/.config/haproxy/haproxy.cfg / /etc/systemd/system/haproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "ha-proxy", visible_alias = "haproxy-server")]
        haproxy: bool,
        /// Also unset NGINX web server & reverse proxy daemon (/etc/nginx/nginx.conf / /etc/nginx/conf.d/proxy.conf / /etc/systemd/system/nginx.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "nginx-server", visible_alias = "nginx-proxy")]
        nginx: bool,
        /// Also unset Kong API gateway daemon proxy (/etc/kong/kong.conf / ~/.kong/kong.conf / /etc/systemd/system/kong.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "kong-gateway", visible_alias = "kong-api")]
        kong: bool,
        /// Also unset Apache APISIX API gateway daemon proxy (/etc/apisix/config.yaml / /usr/local/apisix/conf/config.yaml / /etc/systemd/system/apisix.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "apisix-gateway", visible_alias = "apisix-proxy")]
        apisix: bool,
        /// Also unset Squid caching proxy daemon (/etc/squid/squid.conf / /etc/squid/conf.d/proxy.conf / /etc/systemd/system/squid.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "squid-proxy", visible_alias = "squid-cache")]
        squid: bool,
        /// Also unset Privoxy forwarding proxy daemon (/etc/privoxy/config / /usr/local/etc/privoxy/config / /opt/homebrew/etc/privoxy/config / /etc/systemd/system/privoxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "privoxy-proxy", visible_alias = "privoxy-daemon")]
        privoxy: bool,
        /// Also unset Tinyproxy HTTP proxy daemon (/etc/tinyproxy/tinyproxy.conf / /etc/tinyproxy.conf / /usr/local/etc/tinyproxy.conf / /etc/systemd/system/tinyproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tinyproxy-proxy", visible_alias = "tinyproxy-daemon")]
        tinyproxy: bool,
        /// Also unset Mitmproxy HTTPS intercepting proxy daemon (~/.mitmproxy/config.yaml / /etc/mitmproxy/config.yaml / /etc/systemd/system/mitmproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "mitmproxy-daemon", visible_alias = "mitmdump")]
        mitmproxy: bool,
        /// Also unset Tor anonymity network proxy daemon (/etc/tor/torrc / /usr/local/etc/tor/torrc / ~/.torrc / /etc/systemd/system/tor.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tor-daemon", visible_alias = "torrc")]
        tor: bool,
        /// Also unset Cloudflare Tunnel daemon proxy (~/.cloudflared/config.yml / /etc/cloudflared/config.yml / /etc/systemd/system/cloudflared.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "cf-tunnel", visible_alias = "tunnel")]
        cloudflared: bool,
        /// Also unset Ngrok tunnel daemon proxy (~/.config/ngrok/ngrok.yml / ~/Library/Application Support/ngrok/ngrok.yml / /etc/systemd/system/ngrok.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "ngrok-tunnel", visible_alias = "ngrok-agent")]
        ngrok: bool,
        /// Also unset FRP (Fast Reverse Proxy) client daemon (/etc/frp/frpc.toml / ~/.frp/frpc.toml / /etc/systemd/system/frpc.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "frp", visible_alias = "frp-client")]
        frpc: bool,
        /// Also unset Tailscale daemon proxy (/etc/default/tailscaled / /etc/sysconfig/tailscaled / /etc/systemd/system/tailscaled.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tailscaled", visible_alias = "ts")]
        tailscale: bool,
        /// Also unset ZeroTier One daemon proxy (/var/lib/zerotier-one/local.conf / /Library/Application Support/ZeroTier/One/local.conf / /etc/systemd/system/zerotier-one.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "zerotier-one", visible_alias = "zt")]
        zerotier: bool,
        /// Also unset Gost (GO Simple Tunnel) daemon proxy (/etc/gost/gost.json / ~/.config/gost/config.json / /etc/systemd/system/gost.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "go-gost")]
        gost: bool,
        /// Also unset Rathole NAT traversal daemon proxy (/etc/rathole/rathole.toml / ~/.config/rathole/rathole.toml / /etc/systemd/system/rathole.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "rat")]
        rathole: bool,
        /// Also unset Chisel HTTP tunnel daemon proxy (/etc/chisel/config.json / ~/.config/chisel/config.json / /etc/systemd/system/chisel.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "chisel-tunnel")]
        chisel: bool,
        /// Also unset Nebula mesh overlay daemon proxy (/etc/nebula/config.yaml / ~/.config/nebula/config.yaml / /etc/systemd/system/nebula.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "neb", visible_alias = "slack-nebula")]
        nebula: bool,
        /// Also unset Bore modern tunnel daemon proxy (/etc/bore/bore.toml / ~/.config/bore/bore.toml / /etc/systemd/system/bore.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "bore-tunnel")]
        bore: bool,
        /// Also unset Brook proxy daemon proxy (/etc/brook/config.json / ~/.config/brook/config.json / /etc/systemd/system/brook.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "brook-proxy")]
        brook: bool,
        /// Also unset NPC (NPS Client) daemon proxy (/etc/npc/conf/npc.conf / ~/.npc/npc.conf / /etc/systemd/system/npc.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "nps-npc", visible_alias = "nps")]
        npc: bool,
        /// Also unset Shadowsocks daemon proxy (/etc/shadowsocks-rust/config.json / ~/.config/shadowsocks-rust/config.json / /etc/systemd/system/shadowsocks-rust.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "shadowsocks-proxy", visible_alias = "ss")]
        shadowsocks: bool,
        /// Also unset V2Ray & Xray daemon proxy (/etc/v2ray/config.json / ~/.config/v2ray/config.json / /etc/systemd/system/v2ray.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "v2ray-proxy", visible_alias = "xray")]
        v2ray: bool,
        /// Also unset Sing-box daemon proxy (/etc/sing-box/config.json / ~/.config/sing-box/config.json / /etc/systemd/system/sing-box.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "singbox", visible_alias = "sb")]
        sing_box: bool,
        /// Also unset Hysteria & Hysteria 2 daemon proxy (/etc/hysteria/config.yaml / ~/.config/hysteria/config.yaml / /etc/systemd/system/hysteria.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "hysteria2", visible_alias = "hy2")]
        hysteria: bool,
        /// Also unset Trojan & Trojan-Go daemon proxy (/etc/trojan/config.json / ~/.config/trojan/config.json / /etc/systemd/system/trojan.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "trojan-go", visible_alias = "tg")]
        trojan: bool,
        /// Also unset NaïveProxy daemon proxy (/etc/naiveproxy/config.json / ~/.config/naiveproxy/config.json / /etc/systemd/system/naiveproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "naiveproxy", visible_alias = "naive-proxy")]
        naive: bool,
        /// Also unset Tuic daemon proxy (/etc/tuic/client.json / ~/.config/tuic/client.json / /etc/systemd/system/tuic.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tuic-client")]
        tuic: bool,
        /// Also unset Juicity daemon proxy (/etc/juicity/client.json / ~/.config/juicity/client.json / /etc/systemd/system/juicity.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "juicity-client")]
        juicity: bool,
        /// Also unset Snell daemon proxy (/etc/snell/snell.conf / ~/.config/snell/snell.conf / /etc/systemd/system/snell.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "snell-server", visible_alias = "snell-client")]
        snell: bool,
        /// Also unset Leaf daemon proxy (/etc/leaf/leaf.conf / ~/.config/leaf/leaf.conf / /etc/systemd/system/leaf.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "leaf-proxy", visible_alias = "leaf-daemon")]
        leaf: bool,
        /// Also unset git, cargo, npm, yarn, pnpm, bun, pip, conda, docker, gradle, maven, vscode, gh, glab, jetbrains, nuget, gem, svn, hg, uv, go, bazel, composer, conan, sbt, kube, julia, nix, cabal, stack, hex, r, opam, lein, cpan, luarocks, nimble, rebar3, scoop, bundle, gcloud, aws, azure, mamba, podman, vagrant, choco, colima, minikube, spack, macports, apt, dnf, zypper, pacman, pkg, portage, docker daemon, flatpak, snap, ollama, containerd, buildkit, k3s, microk8s, k0s, rke2, nomad, kubelet, gitlab-runner, actions-runner, jenkins, buildkite-agent, woodpecker-agent, drone-runner, act-runner, circleci-runner, azure-pipelines-agent, teamcity-agent, bamboo-agent, tfc-agent, concourse-worker, tentacle-agent, puppet-agent, chef-client, salt-minion, datadog-agent, newrelic-infra, elastic-agent, grafana-alloy, otelcol, vector, fluent-bit, fluentd, telegraf, logstash, prometheus, alertmanager, promtail, vmagent, filebeat, metricbeat, heartbeat, auditbeat, packetbeat, winlogbeat, consul, vault, traefik, caddy, envoy, haproxy, nginx, kong, apisix, squid, privoxy, tinyproxy, mitmproxy, tor, cloudflared, ngrok, frpc, tailscale, zerotier, gost, rathole, chisel, nebula, bore, brook, npc, shadowsocks, v2ray, sing-box, hysteria, trojan, naive, tuic, juicity, snell, and leaf proxy
        #[arg(short, long)]
        all: bool,
        /// Shell format for environment variables: bash, zsh, fish, pwsh (auto-detected if omitted)
        #[arg(short, long)]
        shell: Option<String>,
    },
    /// 🐚 Print shell integration helper script for eval: `eval "$(proxydoc init)"`
    Init {
        /// Target shell syntax: bash, zsh, fish, pwsh (auto-detected if omitted)
        #[arg(short, long)]
        shell: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            doctor::run(false, false).await?;
        }
        Some(Commands::Doctor { verbose, json }) => {
            doctor::run(verbose, json).await?;
        }
        Some(Commands::Fix) => {
            fixer::run().await?;
        }
        Some(Commands::Test {
            proxy,
            direct,
            compare,
            json,
            target,
            timeout,
        }) => {
            checker::run(proxy, direct, compare, json, target, timeout).await?;
        }
        Some(Commands::Scan { json }) => {
            scanner::run(json).await?;
        }
        Some(Commands::On {
            target,
            git,
            cargo,
            npm,
            yarn,
            pnpm,
            bun,
            pip,
            conda,
            docker,
            gradle,
            maven,
            vscode,
            gh,
            glab,
            jetbrains,
            nuget,
            curl,
            wget,
            gem,
            svn,
            hg,
            uv,
            go,
            bazel,
            composer,
            conan,
            sbt,
            kube,
            julia,
            nix,
            cabal,
            stack,
            hex,
            r,
            opam,
            lein,
            cpan,
            luarocks,
            nimble,
            rebar3,
            scoop,
            bundle,
            gcloud,
            aws,
            azure,
            mamba,
            podman,
            vagrant,
            choco,
            colima,
            minikube,
            spack,
            macports,
            apt,
            dnf,
            zypper,
            pacman,
            pkg,
            portage,
            docker_daemon,
            flatpak,
            snap,
            ollama,
            containerd,
            buildkit,
            k3s,
            microk8s,
            k0s,
            rke2,
            nomad,
            kubelet,
            gitlab_runner,
            actions_runner,
            jenkins,
            buildkite_agent,
            woodpecker_agent,
            drone_runner,
            act_runner,
            circleci_runner,
            azure_pipelines_agent,
            teamcity_agent,
            bamboo_agent,
            tfc_agent,
            concourse_worker,
            tentacle_agent,
            puppet_agent,
            chef_client,
            salt_minion,
            datadog_agent,
            newrelic_infra,
            elastic_agent,
            grafana_alloy,
            otelcol,
            vector,
            fluent_bit,
            fluentd,
            telegraf,
            logstash,
            prometheus,
            alertmanager,
            promtail,
            vmagent,
            filebeat,
            metricbeat,
            heartbeat,
            auditbeat,
            packetbeat,
            winlogbeat,
            consul,
            vault,
            traefik,
            caddy,
            envoy,
            haproxy,
            nginx,
            kong,
            apisix,
            squid,
            privoxy,
            tinyproxy,
            mitmproxy,
            tor,
            cloudflared,
            ngrok,
            frpc,
            tailscale,
            zerotier,
            gost,
            rathole,
            chisel,
            nebula,
            bore,
            brook,
            npc,
            shadowsocks,
            v2ray,
            sing_box,
            hysteria,
            trojan,
            naive,
            tuic,
            juicity,
            snell,
            leaf,
            all,
            shell,
        }) => {
            manager::turn_on(
                &target,
                git,
                cargo,
                npm,
                yarn,
                pnpm,
                bun,
                pip,
                conda,
                docker,
                gradle,
                maven,
                vscode,
                gh,
                glab,
                jetbrains,
                nuget,
                curl,
                wget,
                gem,
                svn,
                hg,
                uv,
                go,
                bazel,
                composer,
                conan,
                sbt,
                kube,
                julia,
                nix,
                cabal,
                stack,
                hex,
                r,
                opam,
                lein,
                cpan,
                luarocks,
                nimble,
                rebar3,
                scoop,
                bundle,
                gcloud,
                aws,
                azure,
                mamba,
                podman,
                vagrant,
                choco,
                colima,
                minikube,
                spack,
                macports,
                apt,
                dnf,
                zypper,
                pacman,
                pkg,
                portage,
                docker_daemon,
                flatpak,
                snap,
                ollama,
                containerd,
                buildkit,
                k3s,
                microk8s,
                k0s,
                rke2,
                nomad,
                kubelet,
                gitlab_runner,
                actions_runner,
                jenkins,
                buildkite_agent,
                woodpecker_agent,
                drone_runner,
                act_runner,
                circleci_runner,
                azure_pipelines_agent,
                teamcity_agent,
                bamboo_agent,
                tfc_agent,
                concourse_worker,
                tentacle_agent,
                puppet_agent,
                chef_client,
                salt_minion,
                datadog_agent,
                newrelic_infra,
                elastic_agent,
                grafana_alloy,
                otelcol,
                vector,
                fluent_bit,
                fluentd,
                telegraf,
                logstash,
                prometheus,
                alertmanager,
                promtail,
                vmagent,
                filebeat,
                metricbeat,
                heartbeat,
                auditbeat,
                packetbeat,
                winlogbeat,
                consul,
                vault,
                traefik,
                caddy,
                envoy,
                haproxy,
                nginx,
                kong,
                apisix,
                squid,
                privoxy,
                tinyproxy,
                mitmproxy,
                tor,
                cloudflared,
                ngrok,
                frpc,
                tailscale,
                zerotier,
                gost,
                rathole,
                chisel,
                nebula,
                bore,
                brook,
                npc,
                shadowsocks,
                v2ray,
                sing_box,
                hysteria,
                trojan,
                naive,
                tuic,
                juicity,
                snell,
                leaf,
                all,
                shell.as_deref(),
            )
            .await?;
        }
        Some(Commands::Off {
            git,
            cargo,
            npm,
            yarn,
            pnpm,
            bun,
            pip,
            conda,
            docker,
            gradle,
            maven,
            vscode,
            gh,
            glab,
            jetbrains,
            nuget,
            curl,
            wget,
            gem,
            svn,
            hg,
            uv,
            go,
            bazel,
            composer,
            conan,
            sbt,
            kube,
            julia,
            nix,
            cabal,
            stack,
            hex,
            r,
            opam,
            lein,
            cpan,
            luarocks,
            nimble,
            rebar3,
            scoop,
            bundle,
            gcloud,
            aws,
            azure,
            mamba,
            podman,
            vagrant,
            choco,
            colima,
            minikube,
            spack,
            macports,
            apt,
            dnf,
            zypper,
            pacman,
            pkg,
            portage,
            docker_daemon,
            flatpak,
            snap,
            ollama,
            containerd,
            buildkit,
            k3s,
            microk8s,
            k0s,
            rke2,
            nomad,
            kubelet,
            gitlab_runner,
            actions_runner,
            jenkins,
            buildkite_agent,
            woodpecker_agent,
            drone_runner,
            act_runner,
            circleci_runner,
            azure_pipelines_agent,
            teamcity_agent,
            bamboo_agent,
            tfc_agent,
            concourse_worker,
            tentacle_agent,
            puppet_agent,
            chef_client,
            salt_minion,
            datadog_agent,
            newrelic_infra,
            elastic_agent,
            grafana_alloy,
            otelcol,
            vector,
            fluent_bit,
            fluentd,
            telegraf,
            logstash,
            prometheus,
            alertmanager,
            promtail,
            vmagent,
            filebeat,
            metricbeat,
            heartbeat,
            auditbeat,
            packetbeat,
            winlogbeat,
            consul,
            vault,
            traefik,
            caddy,
            envoy,
            haproxy,
            nginx,
            kong,
            apisix,
            squid,
            privoxy,
            tinyproxy,
            mitmproxy,
            tor,
            cloudflared,
            ngrok,
            frpc,
            tailscale,
            zerotier,
            gost,
            rathole,
            chisel,
            nebula,
            bore,
            brook,
            npc,
            shadowsocks,
            v2ray,
            sing_box,
            hysteria,
            trojan,
            naive,
            tuic,
            juicity,
            snell,
            leaf,
            all,
            shell,
        }) => {
            manager::turn_off(
                git,
                cargo,
                npm,
                yarn,
                pnpm,
                bun,
                pip,
                conda,
                docker,
                gradle,
                maven,
                vscode,
                gh,
                glab,
                jetbrains,
                nuget,
                curl,
                wget,
                gem,
                svn,
                hg,
                uv,
                go,
                bazel,
                composer,
                conan,
                sbt,
                kube,
                julia,
                nix,
                cabal,
                stack,
                hex,
                r,
                opam,
                lein,
                cpan,
                luarocks,
                nimble,
                rebar3,
                scoop,
                bundle,
                gcloud,
                aws,
                azure,
                mamba,
                podman,
                vagrant,
                choco,
                colima,
                minikube,
                spack,
                macports,
                apt,
                dnf,
                zypper,
                pacman,
                pkg,
                portage,
                docker_daemon,
                flatpak,
                snap,
                ollama,
                containerd,
                buildkit,
                k3s,
                microk8s,
                k0s,
                rke2,
                nomad,
                kubelet,
                gitlab_runner,
                actions_runner,
                jenkins,
                buildkite_agent,
                woodpecker_agent,
                drone_runner,
                act_runner,
                circleci_runner,
                azure_pipelines_agent,
                teamcity_agent,
                bamboo_agent,
                tfc_agent,
                concourse_worker,
                tentacle_agent,
                puppet_agent,
                chef_client,
                salt_minion,
                datadog_agent,
                newrelic_infra,
                elastic_agent,
                grafana_alloy,
                otelcol,
                vector,
                fluent_bit,
                fluentd,
                telegraf,
                logstash,
                prometheus,
                alertmanager,
                promtail,
                vmagent,
                filebeat,
                metricbeat,
                heartbeat,
                auditbeat,
                packetbeat,
                winlogbeat,
                consul,
                vault,
                traefik,
                caddy,
                envoy,
                haproxy,
                nginx,
                kong,
                apisix,
                squid,
                privoxy,
                tinyproxy,
                mitmproxy,
                tor,
                cloudflared,
                ngrok,
                frpc,
                tailscale,
                zerotier,
                gost,
                rathole,
                chisel,
                nebula,
                bore,
                brook,
                npc,
                shadowsocks,
                v2ray,
                sing_box,
                hysteria,
                trojan,
                naive,
                tuic,
                juicity,
                snell,
                leaf,
                all,
                shell.as_deref(),
            )?;
        }
        Some(Commands::Init { shell }) => {
            manager::print_init_script(shell.as_deref());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_doctor_json() {
        let cli = Cli::try_parse_from(["proxydoc", "doctor", "--json"]).unwrap();
        match cli.command {
            Some(Commands::Doctor { verbose, json }) => {
                assert!(!verbose);
                assert!(json);
            }
            _ => panic!("Expected Doctor command"),
        }
    }

    #[test]
    fn test_cli_parse_test_options() {
        let cli = Cli::try_parse_from([
            "proxydoc",
            "test",
            "--proxy",
            "http://127.0.0.1:7890",
            "--json",
        ])
        .unwrap();
        match cli.command {
            Some(Commands::Test {
                proxy,
                direct,
                compare,
                json,
                ..
            }) => {
                assert_eq!(proxy, Some("http://127.0.0.1:7890".to_string()));
                assert!(!direct);
                assert!(!compare);
                assert!(json);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_test_compare() {
        let cli = Cli::try_parse_from(["proxydoc", "test", "--compare"]).unwrap();
        match cli.command {
            Some(Commands::Test {
                compare, direct, ..
            }) => {
                assert!(compare);
                assert!(!direct);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_test_direct() {
        let cli = Cli::try_parse_from(["proxydoc", "test", "-d"]).unwrap();
        match cli.command {
            Some(Commands::Test {
                direct, compare, ..
            }) => {
                assert!(direct);
                assert!(!compare);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_on_all() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "7890", "--all"]).unwrap();
        match cli.command {
            Some(Commands::On {
                target, all, pip, ..
            }) => {
                assert_eq!(target, "7890");
                assert!(all);
                assert!(!pip);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pip() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pip"]).unwrap();
        match cli.command {
            Some(Commands::On { pip, .. }) => {
                assert!(pip);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_test_target_and_timeout() {
        let cli = Cli::try_parse_from([
            "proxydoc",
            "test",
            "-t",
            "https://api.deepseek.com/models",
            "--timeout",
            "10",
        ])
        .unwrap();
        match cli.command {
            Some(Commands::Test {
                target, timeout, ..
            }) => {
                assert_eq!(target, Some("https://api.deepseek.com/models".to_string()));
                assert_eq!(timeout, 10);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_on_docker_and_shell() {
        let cli =
            Cli::try_parse_from(["proxydoc", "on", "7897", "--docker", "--shell", "fish"]).unwrap();
        match cli.command {
            Some(Commands::On {
                target,
                docker,
                shell,
                ..
            }) => {
                assert_eq!(target, "7897");
                assert!(docker);
                assert_eq!(shell, Some("fish".to_string()));
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_docker() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--docker"]).unwrap();
        match cli.command {
            Some(Commands::Off { docker, .. }) => {
                assert!(docker);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_init_shell() {
        let cli = Cli::try_parse_from(["proxydoc", "init", "--shell", "pwsh"]).unwrap();
        match cli.command {
            Some(Commands::Init { shell }) => {
                assert_eq!(shell, Some("pwsh".to_string()));
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_cli_parse_off_all() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "-a"]).unwrap();
        match cli.command {
            Some(Commands::Off { all, .. }) => {
                assert!(all);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_gradle() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "7890", "--gradle"]).unwrap();
        match cli.command {
            Some(Commands::On { target, gradle, .. }) => {
                assert_eq!(target, "7890");
                assert!(gradle);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_on_cargo() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "7890", "--cargo"]).unwrap();
        match cli.command {
            Some(Commands::On { target, cargo, .. }) => {
                assert_eq!(target, "7890");
                assert!(cargo);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_cargo() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--cargo"]).unwrap();
        match cli.command {
            Some(Commands::Off { cargo, .. }) => {
                assert!(cargo);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_off_gradle() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--gradle"]).unwrap();
        match cli.command {
            Some(Commands::Off { gradle, .. }) => {
                assert!(gradle);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_yarn() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--yarn"]).unwrap();
        match cli.command {
            Some(Commands::On { yarn, .. }) => {
                assert!(yarn);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_yarn() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--yarn"]).unwrap();
        match cli.command {
            Some(Commands::Off { yarn, .. }) => {
                assert!(yarn);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_conda() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--conda"]).unwrap();
        match cli.command {
            Some(Commands::On { conda, .. }) => {
                assert!(conda);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_conda() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--conda"]).unwrap();
        match cli.command {
            Some(Commands::Off { conda, .. }) => {
                assert!(conda);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pnpm() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pnpm"]).unwrap();
        match cli.command {
            Some(Commands::On { pnpm, .. }) => {
                assert!(pnpm);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pnpm() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pnpm"]).unwrap();
        match cli.command {
            Some(Commands::Off { pnpm, .. }) => {
                assert!(pnpm);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_bun() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--bun"]).unwrap();
        match cli.command {
            Some(Commands::On { bun, .. }) => {
                assert!(bun);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_bun() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--bun"]).unwrap();
        match cli.command {
            Some(Commands::Off { bun, .. }) => {
                assert!(bun);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_vscode() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--vscode"]).unwrap();
        match cli.command {
            Some(Commands::On { vscode, .. }) => {
                assert!(vscode);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_vscode() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--vscode"]).unwrap();
        match cli.command {
            Some(Commands::Off { vscode, .. }) => {
                assert!(vscode);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_gh() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--gh"]).unwrap();
        match cli.command {
            Some(Commands::On { gh, .. }) => {
                assert!(gh);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_gh() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--gh"]).unwrap();
        match cli.command {
            Some(Commands::Off { gh, .. }) => {
                assert!(gh);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_jetbrains() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--jetbrains"]).unwrap();
        match cli.command {
            Some(Commands::On { jetbrains, .. }) => {
                assert!(jetbrains);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_jetbrains() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--jetbrains"]).unwrap();
        match cli.command {
            Some(Commands::Off { jetbrains, .. }) => {
                assert!(jetbrains);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_nuget() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--nuget"]).unwrap();
        match cli.command {
            Some(Commands::On { nuget, .. }) => {
                assert!(nuget);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_nuget() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--nuget"]).unwrap();
        match cli.command {
            Some(Commands::Off { nuget, .. }) => {
                assert!(nuget);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_curl() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--curl"]).unwrap();
        match cli.command {
            Some(Commands::On { curl, .. }) => {
                assert!(curl);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_curl() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--curl"]).unwrap();
        match cli.command {
            Some(Commands::Off { curl, .. }) => {
                assert!(curl);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_wget() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--wget"]).unwrap();
        match cli.command {
            Some(Commands::On { wget, .. }) => {
                assert!(wget);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_wget() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--wget"]).unwrap();
        match cli.command {
            Some(Commands::Off { wget, .. }) => {
                assert!(wget);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_gem() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--gem"]).unwrap();
        match cli.command {
            Some(Commands::On { gem, .. }) => {
                assert!(gem);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_gem() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--gem"]).unwrap();
        match cli.command {
            Some(Commands::Off { gem, .. }) => {
                assert!(gem);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_svn() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--svn"]).unwrap();
        match cli.command {
            Some(Commands::On { svn, .. }) => {
                assert!(svn);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_svn() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--svn"]).unwrap();
        match cli.command {
            Some(Commands::Off { svn, .. }) => {
                assert!(svn);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_hg() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--hg"]).unwrap();
        match cli.command {
            Some(Commands::On { hg, .. }) => {
                assert!(hg);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_hg() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--hg"]).unwrap();
        match cli.command {
            Some(Commands::Off { hg, .. }) => {
                assert!(hg);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_uv() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--uv"]).unwrap();
        match cli.command {
            Some(Commands::On { uv, .. }) => {
                assert!(uv);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_uv() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--uv"]).unwrap();
        match cli.command {
            Some(Commands::Off { uv, .. }) => {
                assert!(uv);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_go() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--go"]).unwrap();
        match cli.command {
            Some(Commands::On { go, .. }) => {
                assert!(go);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_go() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--go"]).unwrap();
        match cli.command {
            Some(Commands::Off { go, .. }) => {
                assert!(go);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_bazel() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--bazel"]).unwrap();
        match cli.command {
            Some(Commands::On { bazel, .. }) => {
                assert!(bazel);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_bazel() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--bazel"]).unwrap();
        match cli.command {
            Some(Commands::Off { bazel, .. }) => {
                assert!(bazel);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_composer() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--composer"]).unwrap();
        match cli.command {
            Some(Commands::On { composer, .. }) => {
                assert!(composer);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_composer() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--composer"]).unwrap();
        match cli.command {
            Some(Commands::Off { composer, .. }) => {
                assert!(composer);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_conan() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--conan"]).unwrap();
        match cli.command {
            Some(Commands::On { conan, .. }) => {
                assert!(conan);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_conan() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--conan"]).unwrap();
        match cli.command {
            Some(Commands::Off { conan, .. }) => {
                assert!(conan);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_sbt() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--sbt"]).unwrap();
        match cli.command {
            Some(Commands::On { sbt, .. }) => {
                assert!(sbt);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_sbt() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--sbt"]).unwrap();
        match cli.command {
            Some(Commands::Off { sbt, .. }) => {
                assert!(sbt);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_kube() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--kube"]).unwrap();
        match cli.command {
            Some(Commands::On { kube, .. }) => {
                assert!(kube);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_kube() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--kube"]).unwrap();
        match cli.command {
            Some(Commands::Off { kube, .. }) => {
                assert!(kube);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_glab() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--glab"]).unwrap();
        match cli.command {
            Some(Commands::On { glab, .. }) => {
                assert!(glab);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_glab() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--glab"]).unwrap();
        match cli.command {
            Some(Commands::Off { glab, .. }) => {
                assert!(glab);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_julia() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--julia"]).unwrap();
        match cli.command {
            Some(Commands::On { julia, .. }) => {
                assert!(julia);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_julia() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--julia"]).unwrap();
        match cli.command {
            Some(Commands::Off { julia, .. }) => {
                assert!(julia);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_nix() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--nix"]).unwrap();
        match cli.command {
            Some(Commands::On { nix, .. }) => {
                assert!(nix);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_nix() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--nix"]).unwrap();
        match cli.command {
            Some(Commands::Off { nix, .. }) => {
                assert!(nix);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_cabal() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--cabal"]).unwrap();
        match cli.command {
            Some(Commands::On { cabal, .. }) => {
                assert!(cabal);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_cabal() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--cabal"]).unwrap();
        match cli.command {
            Some(Commands::Off { cabal, .. }) => {
                assert!(cabal);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_stack() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--stack"]).unwrap();
        match cli.command {
            Some(Commands::On { stack, .. }) => {
                assert!(stack);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_stack() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--stack"]).unwrap();
        match cli.command {
            Some(Commands::Off { stack, .. }) => {
                assert!(stack);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_hex() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--hex"]).unwrap();
        match cli.command {
            Some(Commands::On { hex, .. }) => {
                assert!(hex);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_hex() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--hex"]).unwrap();
        match cli.command {
            Some(Commands::Off { hex, .. }) => {
                assert!(hex);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_r() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--r"]).unwrap();
        match cli.command {
            Some(Commands::On { r, .. }) => {
                assert!(r);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_r() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--r"]).unwrap();
        match cli.command {
            Some(Commands::Off { r, .. }) => {
                assert!(r);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_opam() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--opam"]).unwrap();
        match cli.command {
            Some(Commands::On { opam, .. }) => {
                assert!(opam);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_opam() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--opam"]).unwrap();
        match cli.command {
            Some(Commands::Off { opam, .. }) => {
                assert!(opam);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_lein() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--lein"]).unwrap();
        match cli.command {
            Some(Commands::On { lein, .. }) => {
                assert!(lein);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_lein() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--lein"]).unwrap();
        match cli.command {
            Some(Commands::Off { lein, .. }) => {
                assert!(lein);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_cpan() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--cpan"]).unwrap();
        match cli.command {
            Some(Commands::On { cpan, .. }) => {
                assert!(cpan);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_cpan() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--cpan"]).unwrap();
        match cli.command {
            Some(Commands::Off { cpan, .. }) => {
                assert!(cpan);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_luarocks() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--luarocks"]).unwrap();
        match cli.command {
            Some(Commands::On { luarocks, .. }) => {
                assert!(luarocks);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_luarocks() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--luarocks"]).unwrap();
        match cli.command {
            Some(Commands::Off { luarocks, .. }) => {
                assert!(luarocks);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_nimble() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--nimble"]).unwrap();
        match cli.command {
            Some(Commands::On { nimble, .. }) => {
                assert!(nimble);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_nimble() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--nimble"]).unwrap();
        match cli.command {
            Some(Commands::Off { nimble, .. }) => {
                assert!(nimble);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_rebar3() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--rebar3"]).unwrap();
        match cli.command {
            Some(Commands::On { rebar3, .. }) => {
                assert!(rebar3);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_rebar3() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--rebar3"]).unwrap();
        match cli.command {
            Some(Commands::Off { rebar3, .. }) => {
                assert!(rebar3);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_scoop() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--scoop"]).unwrap();
        match cli.command {
            Some(Commands::On { scoop, .. }) => {
                assert!(scoop);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_scoop() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--scoop"]).unwrap();
        match cli.command {
            Some(Commands::Off { scoop, .. }) => {
                assert!(scoop);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_bundle() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--bundle"]).unwrap();
        match cli.command {
            Some(Commands::On { bundle, .. }) => {
                assert!(bundle);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_bundle() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--bundle"]).unwrap();
        match cli.command {
            Some(Commands::Off { bundle, .. }) => {
                assert!(bundle);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_gcloud() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--gcloud"]).unwrap();
        match cli.command {
            Some(Commands::On { gcloud, .. }) => {
                assert!(gcloud);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_gcloud() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--gcloud"]).unwrap();
        match cli.command {
            Some(Commands::Off { gcloud, .. }) => {
                assert!(gcloud);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_aws() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--aws"]).unwrap();
        match cli.command {
            Some(Commands::On { aws, .. }) => {
                assert!(aws);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_aws() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--aws"]).unwrap();
        match cli.command {
            Some(Commands::Off { aws, .. }) => {
                assert!(aws);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_azure() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--azure"]).unwrap();
        match cli.command {
            Some(Commands::On { azure, .. }) => {
                assert!(azure);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--az"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { azure, .. }) => {
                assert!(azure);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_azure() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--azure"]).unwrap();
        match cli.command {
            Some(Commands::Off { azure, .. }) => {
                assert!(azure);
            }
            _ => panic!("Expected Off command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "off", "--az"]).unwrap();
        match cli_alias.command {
            Some(Commands::Off { azure, .. }) => {
                assert!(azure);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_mamba() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--mamba"]).unwrap();
        match cli.command {
            Some(Commands::On { mamba, .. }) => {
                assert!(mamba);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--micromamba"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { mamba, .. }) => {
                assert!(mamba);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_mamba() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--mamba"]).unwrap();
        match cli.command {
            Some(Commands::Off { mamba, .. }) => {
                assert!(mamba);
            }
            _ => panic!("Expected Off command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "off", "--micromamba"]).unwrap();
        match cli_alias.command {
            Some(Commands::Off { mamba, .. }) => {
                assert!(mamba);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_podman() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--podman"]).unwrap();
        match cli.command {
            Some(Commands::On { podman, .. }) => {
                assert!(podman);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_podman() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--podman"]).unwrap();
        match cli.command {
            Some(Commands::Off { podman, .. }) => {
                assert!(podman);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_vagrant() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--vagrant"]).unwrap();
        match cli.command {
            Some(Commands::On { vagrant, .. }) => {
                assert!(vagrant);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_vagrant() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--vagrant"]).unwrap();
        match cli.command {
            Some(Commands::Off { vagrant, .. }) => {
                assert!(vagrant);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_choco() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--choco"]).unwrap();
        match cli.command {
            Some(Commands::On { choco, .. }) => {
                assert!(choco);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--chocolatey"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { choco, .. }) => {
                assert!(choco);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_choco() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--choco"]).unwrap();
        match cli.command {
            Some(Commands::Off { choco, .. }) => {
                assert!(choco);
            }
            _ => panic!("Expected Off command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "off", "--chocolatey"]).unwrap();
        match cli_alias.command {
            Some(Commands::Off { choco, .. }) => {
                assert!(choco);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_colima() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--colima"]).unwrap();
        match cli.command {
            Some(Commands::On { colima, .. }) => {
                assert!(colima);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_colima() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--colima"]).unwrap();
        match cli.command {
            Some(Commands::Off { colima, .. }) => {
                assert!(colima);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_minikube() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--minikube"]).unwrap();
        match cli.command {
            Some(Commands::On { minikube, .. }) => {
                assert!(minikube);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_minikube() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--minikube"]).unwrap();
        match cli.command {
            Some(Commands::Off { minikube, .. }) => {
                assert!(minikube);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_spack() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--spack"]).unwrap();
        match cli.command {
            Some(Commands::On { spack, .. }) => {
                assert!(spack);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_spack() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--spack"]).unwrap();
        match cli.command {
            Some(Commands::Off { spack, .. }) => {
                assert!(spack);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_macports() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--macports"]).unwrap();
        match cli.command {
            Some(Commands::On { macports, .. }) => {
                assert!(macports);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_macports() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--macports"]).unwrap();
        match cli.command {
            Some(Commands::Off { macports, .. }) => {
                assert!(macports);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_apt() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--apt"]).unwrap();
        match cli.command {
            Some(Commands::On { apt, .. }) => {
                assert!(apt);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_apt() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--apt"]).unwrap();
        match cli.command {
            Some(Commands::Off { apt, .. }) => {
                assert!(apt);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_dnf() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--dnf"]).unwrap();
        match cli.command {
            Some(Commands::On { dnf, .. }) => {
                assert!(dnf);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_dnf() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--dnf"]).unwrap();
        match cli.command {
            Some(Commands::Off { dnf, .. }) => {
                assert!(dnf);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_zypper() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--zypper"]).unwrap();
        match cli.command {
            Some(Commands::On { zypper, .. }) => {
                assert!(zypper);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_zypper() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--zypper"]).unwrap();
        match cli.command {
            Some(Commands::Off { zypper, .. }) => {
                assert!(zypper);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pacman() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pacman"]).unwrap();
        match cli.command {
            Some(Commands::On { pacman, .. }) => {
                assert!(pacman);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pacman() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pacman"]).unwrap();
        match cli.command {
            Some(Commands::Off { pacman, .. }) => {
                assert!(pacman);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pkg() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pkg"]).unwrap();
        match cli.command {
            Some(Commands::On { pkg, .. }) => {
                assert!(pkg);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pkg() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pkg"]).unwrap();
        match cli.command {
            Some(Commands::Off { pkg, .. }) => {
                assert!(pkg);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_portage() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--portage"]).unwrap();
        match cli.command {
            Some(Commands::On { portage, .. }) => {
                assert!(portage);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_portage() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--emerge"]).unwrap();
        match cli.command {
            Some(Commands::Off { portage, .. }) => {
                assert!(portage);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_docker_daemon() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--docker-daemon"]).unwrap();
        match cli.command {
            Some(Commands::On { docker_daemon, .. }) => {
                assert!(docker_daemon);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_docker_daemon() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--daemon"]).unwrap();
        match cli.command {
            Some(Commands::Off { docker_daemon, .. }) => {
                assert!(docker_daemon);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_flatpak() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--flatpak"]).unwrap();
        match cli.command {
            Some(Commands::On { flatpak, .. }) => {
                assert!(flatpak);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_flatpak() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--ostree"]).unwrap();
        match cli.command {
            Some(Commands::Off { flatpak, .. }) => {
                assert!(flatpak);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_snap() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--snap"]).unwrap();
        match cli.command {
            Some(Commands::On { snap, .. }) => {
                assert!(snap);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_snap() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--snapd"]).unwrap();
        match cli.command {
            Some(Commands::Off { snap, .. }) => {
                assert!(snap);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_ollama() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--ollama"]).unwrap();
        match cli.command {
            Some(Commands::On { ollama, .. }) => {
                assert!(ollama);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_ollama() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--ollamad"]).unwrap();
        match cli.command {
            Some(Commands::Off { ollama, .. }) => {
                assert!(ollama);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_containerd() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--containerd"]).unwrap();
        match cli.command {
            Some(Commands::On { containerd, .. }) => {
                assert!(containerd);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_containerd() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--crictl"]).unwrap();
        match cli.command {
            Some(Commands::Off { containerd, .. }) => {
                assert!(containerd);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_buildkit() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--buildkit"]).unwrap();
        match cli.command {
            Some(Commands::On { buildkit, .. }) => {
                assert!(buildkit);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--buildx"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { buildkit, .. }) => {
                assert!(buildkit);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_buildkit() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--buildkitd"]).unwrap();
        match cli.command {
            Some(Commands::Off { buildkit, .. }) => {
                assert!(buildkit);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_k3s() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--k3s"]).unwrap();
        match cli.command {
            Some(Commands::On { k3s, .. }) => {
                assert!(k3s);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--rancher"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { k3s, .. }) => {
                assert!(k3s);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_k3s() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--k3s-agent"]).unwrap();
        match cli.command {
            Some(Commands::Off { k3s, .. }) => {
                assert!(k3s);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_microk8s() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--microk8s"]).unwrap();
        match cli.command {
            Some(Commands::On { microk8s, .. }) => {
                assert!(microk8s);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--canonical-k8s"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { microk8s, .. }) => {
                assert!(microk8s);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_microk8s() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--microk8s"]).unwrap();
        match cli.command {
            Some(Commands::Off { microk8s, .. }) => {
                assert!(microk8s);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_k0s() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--k0s"]).unwrap();
        match cli.command {
            Some(Commands::On { k0s, .. }) => {
                assert!(k0s);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--mirantis-k8s"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { k0s, .. }) => {
                assert!(k0s);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_k0s() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--k0s"]).unwrap();
        match cli.command {
            Some(Commands::Off { k0s, .. }) => {
                assert!(k0s);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_rke2() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--rke2"]).unwrap();
        match cli.command {
            Some(Commands::On { rke2, .. }) => {
                assert!(rke2);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--rke2-server"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { rke2, .. }) => {
                assert!(rke2);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_rke2() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--rke2"]).unwrap();
        match cli.command {
            Some(Commands::Off { rke2, .. }) => {
                assert!(rke2);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_nomad() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--nomad"]).unwrap();
        match cli.command {
            Some(Commands::On { nomad, .. }) => {
                assert!(nomad);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--nomadd"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { nomad, .. }) => {
                assert!(nomad);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_nomad() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--nomad"]).unwrap();
        match cli.command {
            Some(Commands::Off { nomad, .. }) => {
                assert!(nomad);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_kubelet() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--kubelet"]).unwrap();
        match cli.command {
            Some(Commands::On { kubelet, .. }) => {
                assert!(kubelet);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--k8s-node"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { kubelet, .. }) => {
                assert!(kubelet);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_kubelet() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--kubelet"]).unwrap();
        match cli.command {
            Some(Commands::Off { kubelet, .. }) => {
                assert!(kubelet);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_gitlab_runner() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--gitlab-runner"]).unwrap();
        match cli.command {
            Some(Commands::On { gitlab_runner, .. }) => {
                assert!(gitlab_runner);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--gitlab-ci"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { gitlab_runner, .. }) => {
                assert!(gitlab_runner);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--gl-runner"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { gitlab_runner, .. }) => {
                assert!(gitlab_runner);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_gitlab_runner() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--gitlab-runner"]).unwrap();
        match cli.command {
            Some(Commands::Off { gitlab_runner, .. }) => {
                assert!(gitlab_runner);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_actions_runner() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--actions-runner"]).unwrap();
        match cli.command {
            Some(Commands::On { actions_runner, .. }) => {
                assert!(actions_runner);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--runner"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { actions_runner, .. }) => {
                assert!(actions_runner);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--gh-runner"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { actions_runner, .. }) => {
                assert!(actions_runner);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_actions_runner() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--actions-runner"]).unwrap();
        match cli.command {
            Some(Commands::Off { actions_runner, .. }) => {
                assert!(actions_runner);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_jenkins() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--jenkins"]).unwrap();
        match cli.command {
            Some(Commands::On { jenkins, .. }) => {
                assert!(jenkins);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--jenkins-ci"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { jenkins, .. }) => {
                assert!(jenkins);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_jenkins() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--jenkins"]).unwrap();
        match cli.command {
            Some(Commands::Off { jenkins, .. }) => {
                assert!(jenkins);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_buildkite_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--buildkite-agent"]).unwrap();
        match cli.command {
            Some(Commands::On { buildkite_agent, .. }) => {
                assert!(buildkite_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--buildkite"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { buildkite_agent, .. }) => {
                assert!(buildkite_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--bk-agent"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { buildkite_agent, .. }) => {
                assert!(buildkite_agent);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_buildkite_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--buildkite-agent"]).unwrap();
        match cli.command {
            Some(Commands::Off { buildkite_agent, .. }) => {
                assert!(buildkite_agent);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_woodpecker_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--woodpecker-agent"]).unwrap();
        match cli.command {
            Some(Commands::On { woodpecker_agent, .. }) => {
                assert!(woodpecker_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--woodpecker"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { woodpecker_agent, .. }) => {
                assert!(woodpecker_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--wp-agent"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { woodpecker_agent, .. }) => {
                assert!(woodpecker_agent);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_woodpecker_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--woodpecker-agent"]).unwrap();
        match cli.command {
            Some(Commands::Off { woodpecker_agent, .. }) => {
                assert!(woodpecker_agent);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_drone_runner() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--drone-runner"]).unwrap();
        match cli.command {
            Some(Commands::On { drone_runner, .. }) => {
                assert!(drone_runner);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--drone"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { drone_runner, .. }) => {
                assert!(drone_runner);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--drone-ci"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { drone_runner, .. }) => {
                assert!(drone_runner);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_drone_runner() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--drone-runner"]).unwrap();
        match cli.command {
            Some(Commands::Off { drone_runner, .. }) => {
                assert!(drone_runner);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_act_runner() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--act-runner"]).unwrap();
        match cli.command {
            Some(Commands::On { act_runner, .. }) => {
                assert!(act_runner);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--act"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { act_runner, .. }) => {
                assert!(act_runner);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--gitea-runner"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { act_runner, .. }) => {
                assert!(act_runner);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--forgejo-runner"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { act_runner, .. }) => {
                assert!(act_runner);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_act_runner() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--act-runner"]).unwrap();
        match cli.command {
            Some(Commands::Off { act_runner, .. }) => {
                assert!(act_runner);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_circleci_runner() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--circleci-runner"]).unwrap();
        match cli.command {
            Some(Commands::On { circleci_runner, .. }) => {
                assert!(circleci_runner);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--circleci"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { circleci_runner, .. }) => {
                assert!(circleci_runner);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--cci-runner"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { circleci_runner, .. }) => {
                assert!(circleci_runner);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_circleci_runner() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--circleci-runner"]).unwrap();
        match cli.command {
            Some(Commands::Off { circleci_runner, .. }) => {
                assert!(circleci_runner);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_azure_pipelines_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--azure-pipelines-agent"]).unwrap();
        match cli.command {
            Some(Commands::On { azure_pipelines_agent, .. }) => {
                assert!(azure_pipelines_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--vsts-agent"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { azure_pipelines_agent, .. }) => {
                assert!(azure_pipelines_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--azp-agent"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { azure_pipelines_agent, .. }) => {
                assert!(azure_pipelines_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--azure-agent"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { azure_pipelines_agent, .. }) => {
                assert!(azure_pipelines_agent);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_azure_pipelines_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--azure-pipelines-agent"]).unwrap();
        match cli.command {
            Some(Commands::Off { azure_pipelines_agent, .. }) => {
                assert!(azure_pipelines_agent);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_teamcity_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--teamcity-agent"]).unwrap();
        match cli.command {
            Some(Commands::On { teamcity_agent, .. }) => {
                assert!(teamcity_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--teamcity"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { teamcity_agent, .. }) => {
                assert!(teamcity_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--buildagent"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { teamcity_agent, .. }) => {
                assert!(teamcity_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--tc-agent"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { teamcity_agent, .. }) => {
                assert!(teamcity_agent);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_teamcity_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--teamcity-agent"]).unwrap();
        match cli.command {
            Some(Commands::Off { teamcity_agent, .. }) => {
                assert!(teamcity_agent);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_bamboo_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--bamboo-agent"]).unwrap();
        match cli.command {
            Some(Commands::On { bamboo_agent, .. }) => {
                assert!(bamboo_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--bamboo"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { bamboo_agent, .. }) => {
                assert!(bamboo_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--atlassian-agent"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { bamboo_agent, .. }) => {
                assert!(bamboo_agent);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_bamboo_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--bamboo-agent"]).unwrap();
        match cli.command {
            Some(Commands::Off { bamboo_agent, .. }) => {
                assert!(bamboo_agent);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_tfc_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--tfc-agent"]).unwrap();
        match cli.command {
            Some(Commands::On { tfc_agent, .. }) => {
                assert!(tfc_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--tfc"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { tfc_agent, .. }) => {
                assert!(tfc_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--terraform-agent"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { tfc_agent, .. }) => {
                assert!(tfc_agent);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_tfc_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--tfc-agent"]).unwrap();
        match cli.command {
            Some(Commands::Off { tfc_agent, .. }) => {
                assert!(tfc_agent);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_concourse_worker() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--concourse-worker"]).unwrap();
        match cli.command {
            Some(Commands::On { concourse_worker, .. }) => {
                assert!(concourse_worker);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--concourse"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { concourse_worker, .. }) => {
                assert!(concourse_worker);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_concourse_worker() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--concourse-worker"]).unwrap();
        match cli.command {
            Some(Commands::Off { concourse_worker, .. }) => {
                assert!(concourse_worker);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_tentacle_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--tentacle-agent"]).unwrap();
        match cli.command {
            Some(Commands::On { tentacle_agent, .. }) => {
                assert!(tentacle_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--tentacle"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { tentacle_agent, .. }) => {
                assert!(tentacle_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--octopus"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { tentacle_agent, .. }) => {
                assert!(tentacle_agent);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_tentacle_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--tentacle-agent"]).unwrap();
        match cli.command {
            Some(Commands::Off { tentacle_agent, .. }) => {
                assert!(tentacle_agent);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_puppet_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--puppet-agent"]).unwrap();
        match cli.command {
            Some(Commands::On { puppet_agent, .. }) => {
                assert!(puppet_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--puppet"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { puppet_agent, .. }) => {
                assert!(puppet_agent);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_puppet_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--puppet-agent"]).unwrap();
        match cli.command {
            Some(Commands::Off { puppet_agent, .. }) => {
                assert!(puppet_agent);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_chef_client() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--chef-client"]).unwrap();
        match cli.command {
            Some(Commands::On { chef_client, .. }) => {
                assert!(chef_client);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--chef"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { chef_client, .. }) => {
                assert!(chef_client);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_chef_client() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--chef-client"]).unwrap();
        match cli.command {
            Some(Commands::Off { chef_client, .. }) => {
                assert!(chef_client);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_salt_minion() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--salt-minion"]).unwrap();
        match cli.command {
            Some(Commands::On { salt_minion, .. }) => {
                assert!(salt_minion);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--salt"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { salt_minion, .. }) => {
                assert!(salt_minion);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_salt_minion() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--salt-minion"]).unwrap();
        match cli.command {
            Some(Commands::Off { salt_minion, .. }) => {
                assert!(salt_minion);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_datadog_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--datadog-agent"]).unwrap();
        match cli.command {
            Some(Commands::On { datadog_agent, .. }) => {
                assert!(datadog_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--datadog"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { datadog_agent, .. }) => {
                assert!(datadog_agent);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_datadog_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--datadog-agent"]).unwrap();
        match cli.command {
            Some(Commands::Off { datadog_agent, .. }) => {
                assert!(datadog_agent);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_newrelic_infra() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--newrelic-infra"]).unwrap();
        match cli.command {
            Some(Commands::On { newrelic_infra, .. }) => {
                assert!(newrelic_infra);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--newrelic"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { newrelic_infra, .. }) => {
                assert!(newrelic_infra);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_newrelic_infra() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--newrelic-infra"]).unwrap();
        match cli.command {
            Some(Commands::Off { newrelic_infra, .. }) => {
                assert!(newrelic_infra);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_elastic_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--elastic-agent"]).unwrap();
        match cli.command {
            Some(Commands::On { elastic_agent, .. }) => {
                assert!(elastic_agent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--elastic"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { elastic_agent, .. }) => {
                assert!(elastic_agent);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_elastic_agent() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--elastic-agent"]).unwrap();
        match cli.command {
            Some(Commands::Off { elastic_agent, .. }) => {
                assert!(elastic_agent);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_grafana_alloy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--grafana-alloy"]).unwrap();
        match cli.command {
            Some(Commands::On { grafana_alloy, .. }) => {
                assert!(grafana_alloy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--alloy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { grafana_alloy, .. }) => {
                assert!(grafana_alloy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_grafana_alloy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--grafana-alloy"]).unwrap();
        match cli.command {
            Some(Commands::Off { grafana_alloy, .. }) => {
                assert!(grafana_alloy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_otelcol() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--otelcol"]).unwrap();
        match cli.command {
            Some(Commands::On { otelcol, .. }) => {
                assert!(otelcol);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--otel"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { otelcol, .. }) => {
                assert!(otelcol);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_otelcol() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--otelcol"]).unwrap();
        match cli.command {
            Some(Commands::Off { otelcol, .. }) => {
                assert!(otelcol);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_vector() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--vector"]).unwrap();
        match cli.command {
            Some(Commands::On { vector, .. }) => {
                assert!(vector);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--timber-vector"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { vector, .. }) => {
                assert!(vector);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_vector() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--vector"]).unwrap();
        match cli.command {
            Some(Commands::Off { vector, .. }) => {
                assert!(vector);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_fluent_bit() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--fluent-bit"]).unwrap();
        match cli.command {
            Some(Commands::On { fluent_bit, .. }) => {
                assert!(fluent_bit);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--flb"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { fluent_bit, .. }) => {
                assert!(fluent_bit);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_fluent_bit() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--fluent-bit"]).unwrap();
        match cli.command {
            Some(Commands::Off { fluent_bit, .. }) => {
                assert!(fluent_bit);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_fluentd() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--fluentd"]).unwrap();
        match cli.command {
            Some(Commands::On { fluentd, .. }) => {
                assert!(fluentd);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--td-agent"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { fluentd, .. }) => {
                assert!(fluentd);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_fluentd() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--fluentd"]).unwrap();
        match cli.command {
            Some(Commands::Off { fluentd, .. }) => {
                assert!(fluentd);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_telegraf() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--telegraf"]).unwrap();
        match cli.command {
            Some(Commands::On { telegraf, .. }) => {
                assert!(telegraf);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--influx-telegraf"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { telegraf, .. }) => {
                assert!(telegraf);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_telegraf() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--telegraf"]).unwrap();
        match cli.command {
            Some(Commands::Off { telegraf, .. }) => {
                assert!(telegraf);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_logstash() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--logstash"]).unwrap();
        match cli.command {
            Some(Commands::On { logstash, .. }) => {
                assert!(logstash);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--elastic-logstash"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { logstash, .. }) => {
                assert!(logstash);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_logstash() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--logstash"]).unwrap();
        match cli.command {
            Some(Commands::Off { logstash, .. }) => {
                assert!(logstash);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_prometheus() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--prometheus"]).unwrap();
        match cli.command {
            Some(Commands::On { prometheus, .. }) => {
                assert!(prometheus);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--prom"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { prometheus, .. }) => {
                assert!(prometheus);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_prometheus() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--prometheus"]).unwrap();
        match cli.command {
            Some(Commands::Off { prometheus, .. }) => {
                assert!(prometheus);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_alertmanager() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--alertmanager"]).unwrap();
        match cli.command {
            Some(Commands::On { alertmanager, .. }) => {
                assert!(alertmanager);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--alert-manager"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { alertmanager, .. }) => {
                assert!(alertmanager);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_alertmanager() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--alertmanager"]).unwrap();
        match cli.command {
            Some(Commands::Off { alertmanager, .. }) => {
                assert!(alertmanager);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_promtail() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--promtail"]).unwrap();
        match cli.command {
            Some(Commands::On { promtail, .. }) => {
                assert!(promtail);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--loki-promtail"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { promtail, .. }) => {
                assert!(promtail);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_promtail() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--promtail"]).unwrap();
        match cli.command {
            Some(Commands::Off { promtail, .. }) => {
                assert!(promtail);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_vmagent() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--vmagent"]).unwrap();
        match cli.command {
            Some(Commands::On { vmagent, .. }) => {
                assert!(vmagent);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--victoria-agent"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { vmagent, .. }) => {
                assert!(vmagent);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_vmagent() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--vmagent"]).unwrap();
        match cli.command {
            Some(Commands::Off { vmagent, .. }) => {
                assert!(vmagent);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_filebeat() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--filebeat"]).unwrap();
        match cli.command {
            Some(Commands::On { filebeat, .. }) => {
                assert!(filebeat);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--elastic-filebeat"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { filebeat, .. }) => {
                assert!(filebeat);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_filebeat() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--filebeat"]).unwrap();
        match cli.command {
            Some(Commands::Off { filebeat, .. }) => {
                assert!(filebeat);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_metricbeat() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--metricbeat"]).unwrap();
        match cli.command {
            Some(Commands::On { metricbeat, .. }) => {
                assert!(metricbeat);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--elastic-metricbeat"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { metricbeat, .. }) => {
                assert!(metricbeat);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_metricbeat() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--metricbeat"]).unwrap();
        match cli.command {
            Some(Commands::Off { metricbeat, .. }) => {
                assert!(metricbeat);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_heartbeat() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--heartbeat"]).unwrap();
        match cli.command {
            Some(Commands::On { heartbeat, .. }) => {
                assert!(heartbeat);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--elastic-heartbeat"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { heartbeat, .. }) => {
                assert!(heartbeat);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_heartbeat() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--heartbeat"]).unwrap();
        match cli.command {
            Some(Commands::Off { heartbeat, .. }) => {
                assert!(heartbeat);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_auditbeat() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--auditbeat"]).unwrap();
        match cli.command {
            Some(Commands::On { auditbeat, .. }) => {
                assert!(auditbeat);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--elastic-auditbeat"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { auditbeat, .. }) => {
                assert!(auditbeat);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_auditbeat() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--auditbeat"]).unwrap();
        match cli.command {
            Some(Commands::Off { auditbeat, .. }) => {
                assert!(auditbeat);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_packetbeat() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--packetbeat"]).unwrap();
        match cli.command {
            Some(Commands::On { packetbeat, .. }) => {
                assert!(packetbeat);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--elastic-packetbeat"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { packetbeat, .. }) => {
                assert!(packetbeat);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_packetbeat() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--packetbeat"]).unwrap();
        match cli.command {
            Some(Commands::Off { packetbeat, .. }) => {
                assert!(packetbeat);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_winlogbeat() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--winlogbeat"]).unwrap();
        match cli.command {
            Some(Commands::On { winlogbeat, .. }) => {
                assert!(winlogbeat);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--elastic-winlogbeat"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { winlogbeat, .. }) => {
                assert!(winlogbeat);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_winlogbeat() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--winlogbeat"]).unwrap();
        match cli.command {
            Some(Commands::Off { winlogbeat, .. }) => {
                assert!(winlogbeat);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_consul() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--consul"]).unwrap();
        match cli.command {
            Some(Commands::On { consul, .. }) => {
                assert!(consul);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--consul-agent"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { consul, .. }) => {
                assert!(consul);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_consul() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--consul"]).unwrap();
        match cli.command {
            Some(Commands::Off { consul, .. }) => {
                assert!(consul);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_vault() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--vault"]).unwrap();
        match cli.command {
            Some(Commands::On { vault, .. }) => {
                assert!(vault);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--vault-agent"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { vault, .. }) => {
                assert!(vault);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_vault() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--vault"]).unwrap();
        match cli.command {
            Some(Commands::Off { vault, .. }) => {
                assert!(vault);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_traefik() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--traefik"]).unwrap();
        match cli.command {
            Some(Commands::On { traefik, .. }) => {
                assert!(traefik);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--traefik-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { traefik, .. }) => {
                assert!(traefik);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_traefik() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--traefik"]).unwrap();
        match cli.command {
            Some(Commands::Off { traefik, .. }) => {
                assert!(traefik);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_caddy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--caddy"]).unwrap();
        match cli.command {
            Some(Commands::On { caddy, .. }) => {
                assert!(caddy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--caddy-server"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { caddy, .. }) => {
                assert!(caddy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_caddy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--caddy"]).unwrap();
        match cli.command {
            Some(Commands::Off { caddy, .. }) => {
                assert!(caddy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_envoy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--envoy"]).unwrap();
        match cli.command {
            Some(Commands::On { envoy, .. }) => {
                assert!(envoy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--envoy-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { envoy, .. }) => {
                assert!(envoy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_envoy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--envoy"]).unwrap();
        match cli.command {
            Some(Commands::Off { envoy, .. }) => {
                assert!(envoy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_haproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--haproxy"]).unwrap();
        match cli.command {
            Some(Commands::On { haproxy, .. }) => {
                assert!(haproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--ha-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { haproxy, .. }) => {
                assert!(haproxy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_haproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--haproxy"]).unwrap();
        match cli.command {
            Some(Commands::Off { haproxy, .. }) => {
                assert!(haproxy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_nginx() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--nginx"]).unwrap();
        match cli.command {
            Some(Commands::On { nginx, .. }) => {
                assert!(nginx);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--nginx-server"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { nginx, .. }) => {
                assert!(nginx);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_nginx() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--nginx"]).unwrap();
        match cli.command {
            Some(Commands::Off { nginx, .. }) => {
                assert!(nginx);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_kong() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--kong"]).unwrap();
        match cli.command {
            Some(Commands::On { kong, .. }) => {
                assert!(kong);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--kong-gateway"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { kong, .. }) => {
                assert!(kong);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_kong() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--kong"]).unwrap();
        match cli.command {
            Some(Commands::Off { kong, .. }) => {
                assert!(kong);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_apisix() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--apisix"]).unwrap();
        match cli.command {
            Some(Commands::On { apisix, .. }) => {
                assert!(apisix);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--apisix-gateway"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { apisix, .. }) => {
                assert!(apisix);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_apisix() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--apisix"]).unwrap();
        match cli.command {
            Some(Commands::Off { apisix, .. }) => {
                assert!(apisix);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_squid() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--squid"]).unwrap();
        match cli.command {
            Some(Commands::On { squid, .. }) => {
                assert!(squid);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--squid-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { squid, .. }) => {
                assert!(squid);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_squid() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--squid"]).unwrap();
        match cli.command {
            Some(Commands::Off { squid, .. }) => {
                assert!(squid);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_privoxy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--privoxy"]).unwrap();
        match cli.command {
            Some(Commands::On { privoxy, .. }) => {
                assert!(privoxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--privoxy-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { privoxy, .. }) => {
                assert!(privoxy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_privoxy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--privoxy"]).unwrap();
        match cli.command {
            Some(Commands::Off { privoxy, .. }) => {
                assert!(privoxy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_tinyproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--tinyproxy"]).unwrap();
        match cli.command {
            Some(Commands::On { tinyproxy, .. }) => {
                assert!(tinyproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--tinyproxy-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { tinyproxy, .. }) => {
                assert!(tinyproxy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_tinyproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--tinyproxy"]).unwrap();
        match cli.command {
            Some(Commands::Off { tinyproxy, .. }) => {
                assert!(tinyproxy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_mitmproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--mitmproxy"]).unwrap();
        match cli.command {
            Some(Commands::On { mitmproxy, .. }) => {
                assert!(mitmproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--mitmproxy-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { mitmproxy, .. }) => {
                assert!(mitmproxy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_mitmproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--mitmproxy"]).unwrap();
        match cli.command {
            Some(Commands::Off { mitmproxy, .. }) => {
                assert!(mitmproxy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_tor() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--tor"]).unwrap();
        match cli.command {
            Some(Commands::On { tor, .. }) => {
                assert!(tor);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--tor-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { tor, .. }) => {
                assert!(tor);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_tor() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--tor"]).unwrap();
        match cli.command {
            Some(Commands::Off { tor, .. }) => {
                assert!(tor);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_cloudflared() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--cloudflared"]).unwrap();
        match cli.command {
            Some(Commands::On { cloudflared, .. }) => {
                assert!(cloudflared);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--cf-tunnel"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { cloudflared, .. }) => {
                assert!(cloudflared);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_cloudflared() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--cloudflared"]).unwrap();
        match cli.command {
            Some(Commands::Off { cloudflared, .. }) => {
                assert!(cloudflared);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_ngrok() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--ngrok"]).unwrap();
        match cli.command {
            Some(Commands::On { ngrok, .. }) => {
                assert!(ngrok);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--ngrok-agent"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { ngrok, .. }) => {
                assert!(ngrok);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_ngrok() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--ngrok"]).unwrap();
        match cli.command {
            Some(Commands::Off { ngrok, .. }) => {
                assert!(ngrok);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_frpc() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--frpc"]).unwrap();
        match cli.command {
            Some(Commands::On { frpc, .. }) => {
                assert!(frpc);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--frp"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { frpc, .. }) => {
                assert!(frpc);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_frpc() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--frpc"]).unwrap();
        match cli.command {
            Some(Commands::Off { frpc, .. }) => {
                assert!(frpc);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_tailscale() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--tailscale"]).unwrap();
        match cli.command {
            Some(Commands::On { tailscale, .. }) => {
                assert!(tailscale);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--tailscaled"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { tailscale, .. }) => {
                assert!(tailscale);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_tailscale() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--tailscale"]).unwrap();
        match cli.command {
            Some(Commands::Off { tailscale, .. }) => {
                assert!(tailscale);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_zerotier() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--zerotier"]).unwrap();
        match cli.command {
            Some(Commands::On { zerotier, .. }) => {
                assert!(zerotier);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--zt"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { zerotier, .. }) => {
                assert!(zerotier);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_zerotier() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--zerotier"]).unwrap();
        match cli.command {
            Some(Commands::Off { zerotier, .. }) => {
                assert!(zerotier);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_gost() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--gost"]).unwrap();
        match cli.command {
            Some(Commands::On { gost, .. }) => {
                assert!(gost);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--go-gost"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { gost, .. }) => {
                assert!(gost);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_gost() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--gost"]).unwrap();
        match cli.command {
            Some(Commands::Off { gost, .. }) => {
                assert!(gost);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_rathole() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--rathole"]).unwrap();
        match cli.command {
            Some(Commands::On { rathole, .. }) => {
                assert!(rathole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--rat"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { rathole, .. }) => {
                assert!(rathole);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_rathole() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--rathole"]).unwrap();
        match cli.command {
            Some(Commands::Off { rathole, .. }) => {
                assert!(rathole);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_chisel() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--chisel"]).unwrap();
        match cli.command {
            Some(Commands::On { chisel, .. }) => {
                assert!(chisel);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--chisel-tunnel"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { chisel, .. }) => {
                assert!(chisel);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_chisel() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--chisel"]).unwrap();
        match cli.command {
            Some(Commands::Off { chisel, .. }) => {
                assert!(chisel);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_nebula() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--nebula"]).unwrap();
        match cli.command {
            Some(Commands::On { nebula, .. }) => {
                assert!(nebula);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--neb"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { nebula, .. }) => {
                assert!(nebula);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--slack-nebula"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { nebula, .. }) => {
                assert!(nebula);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_nebula() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--nebula"]).unwrap();
        match cli.command {
            Some(Commands::Off { nebula, .. }) => {
                assert!(nebula);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_bore() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--bore"]).unwrap();
        match cli.command {
            Some(Commands::On { bore, .. }) => {
                assert!(bore);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--bore-tunnel"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { bore, .. }) => {
                assert!(bore);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_bore() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--bore"]).unwrap();
        match cli.command {
            Some(Commands::Off { bore, .. }) => {
                assert!(bore);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_brook() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--brook"]).unwrap();
        match cli.command {
            Some(Commands::On { brook, .. }) => {
                assert!(brook);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--brook-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { brook, .. }) => {
                assert!(brook);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_brook() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--brook"]).unwrap();
        match cli.command {
            Some(Commands::Off { brook, .. }) => {
                assert!(brook);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_npc() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--npc"]).unwrap();
        match cli.command {
            Some(Commands::On { npc, .. }) => {
                assert!(npc);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--nps-npc"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { npc, .. }) => {
                assert!(npc);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--nps"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { npc, .. }) => {
                assert!(npc);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_npc() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--npc"]).unwrap();
        match cli.command {
            Some(Commands::Off { npc, .. }) => {
                assert!(npc);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_shadowsocks() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--shadowsocks"]).unwrap();
        match cli.command {
            Some(Commands::On { shadowsocks, .. }) => {
                assert!(shadowsocks);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--shadowsocks-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { shadowsocks, .. }) => {
                assert!(shadowsocks);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--ss"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { shadowsocks, .. }) => {
                assert!(shadowsocks);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_shadowsocks() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--shadowsocks"]).unwrap();
        match cli.command {
            Some(Commands::Off { shadowsocks, .. }) => {
                assert!(shadowsocks);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_v2ray() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--v2ray"]).unwrap();
        match cli.command {
            Some(Commands::On { v2ray, .. }) => {
                assert!(v2ray);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--v2ray-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { v2ray, .. }) => {
                assert!(v2ray);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--xray"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { v2ray, .. }) => {
                assert!(v2ray);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_v2ray() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--v2ray"]).unwrap();
        match cli.command {
            Some(Commands::Off { v2ray, .. }) => {
                assert!(v2ray);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_sing_box() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--sing-box"]).unwrap();
        match cli.command {
            Some(Commands::On { sing_box, .. }) => {
                assert!(sing_box);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--singbox"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { sing_box, .. }) => {
                assert!(sing_box);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--sb"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { sing_box, .. }) => {
                assert!(sing_box);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_sing_box() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--sing-box"]).unwrap();
        match cli.command {
            Some(Commands::Off { sing_box, .. }) => {
                assert!(sing_box);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_hysteria() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--hysteria"]).unwrap();
        match cli.command {
            Some(Commands::On { hysteria, .. }) => {
                assert!(hysteria);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--hysteria2"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { hysteria, .. }) => {
                assert!(hysteria);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--hy2"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { hysteria, .. }) => {
                assert!(hysteria);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_hysteria() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--hysteria"]).unwrap();
        match cli.command {
            Some(Commands::Off { hysteria, .. }) => {
                assert!(hysteria);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_trojan() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--trojan"]).unwrap();
        match cli.command {
            Some(Commands::On { trojan, .. }) => {
                assert!(trojan);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--trojan-go"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { trojan, .. }) => {
                assert!(trojan);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--tg"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { trojan, .. }) => {
                assert!(trojan);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_trojan() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--trojan"]).unwrap();
        match cli.command {
            Some(Commands::Off { trojan, .. }) => {
                assert!(trojan);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_naive() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--naive"]).unwrap();
        match cli.command {
            Some(Commands::On { naive, .. }) => {
                assert!(naive);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--naiveproxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { naive, .. }) => {
                assert!(naive);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--naive-proxy"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { naive, .. }) => {
                assert!(naive);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_naive() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--naive"]).unwrap();
        match cli.command {
            Some(Commands::Off { naive, .. }) => {
                assert!(naive);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_tuic() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--tuic"]).unwrap();
        match cli.command {
            Some(Commands::On { tuic, .. }) => {
                assert!(tuic);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--tuic-client"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { tuic, .. }) => {
                assert!(tuic);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_tuic() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--tuic"]).unwrap();
        match cli.command {
            Some(Commands::Off { tuic, .. }) => {
                assert!(tuic);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_juicity() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--juicity"]).unwrap();
        match cli.command {
            Some(Commands::On { juicity, .. }) => {
                assert!(juicity);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--juicity-client"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { juicity, .. }) => {
                assert!(juicity);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_juicity() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--juicity"]).unwrap();
        match cli.command {
            Some(Commands::Off { juicity, .. }) => {
                assert!(juicity);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_snell() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--snell"]).unwrap();
        match cli.command {
            Some(Commands::On { snell, .. }) => {
                assert!(snell);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--snell-server"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { snell, .. }) => {
                assert!(snell);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_snell() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--snell"]).unwrap();
        match cli.command {
            Some(Commands::Off { snell, .. }) => {
                assert!(snell);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_leaf() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--leaf"]).unwrap();
        match cli.command {
            Some(Commands::On { leaf, .. }) => {
                assert!(leaf);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--leaf-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { leaf, .. }) => {
                assert!(leaf);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_leaf() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--leaf"]).unwrap();
        match cli.command {
            Some(Commands::Off { leaf, .. }) => {
                assert!(leaf);
            }
            _ => panic!("Expected Off command"),
        }
    }
}
