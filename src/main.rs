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
        /// Also configure Mihomo daemon proxy (/etc/mihomo/config.yaml / ~/.config/mihomo/config.yaml / /etc/systemd/system/mihomo.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "mihomo-daemon", visible_alias = "clash-meta")]
        mihomo: bool,
        /// Also configure Stunnel daemon proxy (/etc/stunnel/stunnel.conf / ~/.config/stunnel/stunnel.conf / /etc/systemd/system/stunnel.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "stunnel-proxy", visible_alias = "stunnel4")]
        stunnel: bool,
        /// Also configure Shadow-TLS daemon proxy (/etc/shadow-tls/config.json / ~/.config/shadow-tls/config.json / /etc/systemd/system/shadow-tls.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "shadowtls", visible_alias = "shadow-tls-proxy")]
        shadow_tls: bool,
        /// Also configure Tun2socks daemon proxy (/etc/tun2socks/config.yaml / ~/.config/tun2socks/config.yaml / /etc/systemd/system/tun2socks.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tun2socks-daemon", visible_alias = "tun2socks-proxy")]
        tun2socks: bool,
        /// Also configure WSTunnel daemon proxy (/etc/wstunnel/config.yaml / ~/.config/wstunnel/config.yaml / /etc/systemd/system/wstunnel.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "wstunnel-daemon", visible_alias = "wstunnel-proxy")]
        wstunnel: bool,
        /// Also configure Cloak daemon proxy (/etc/cloak/ckclient.json / ~/.config/cloak/ckclient.json / /etc/systemd/system/cloak.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "cloak-proxy", visible_alias = "ck-client")]
        cloak: bool,
        /// Also configure Wireproxy daemon proxy (/etc/wireproxy/config.yaml / ~/.config/wireproxy/config.yaml / /etc/systemd/system/wireproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "wireproxy-daemon", visible_alias = "wireproxy-proxy")]
        wireproxy: bool,
        /// Also configure MTG daemon proxy (/etc/mtg.toml / ~/.config/mtg/config.toml / /etc/systemd/system/mtg.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "mtg-daemon", visible_alias = "mtg-proxy")]
        mtg: bool,
        /// Also configure Kcptun daemon proxy (/etc/kcptun/client.json / ~/.config/kcptun/client.json / /etc/systemd/system/kcptun.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "kcptun-client", visible_alias = "kcptun-proxy")]
        kcptun: bool,
        /// Also configure Glider daemon proxy (/etc/glider/glider.conf / ~/.config/glider/glider.conf / /etc/systemd/system/glider.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "glider-proxy", visible_alias = "glider-daemon")]
        glider: bool,
        /// Also configure Redsocks daemon proxy (/etc/redsocks.conf / ~/.config/redsocks/redsocks.conf / /etc/systemd/system/redsocks.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "redsocks-proxy", visible_alias = "redsocks-daemon")]
        redsocks: bool,
        /// Also configure Dante daemon proxy (/etc/danted.conf / ~/.config/danted.conf / /etc/systemd/system/danted.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "dante-proxy", visible_alias = "danted")]
        dante: bool,
        /// Also configure 3proxy multi-protocol daemon proxy (/etc/3proxy/3proxy.cfg / ~/.3proxy.cfg / ~/.config/3proxy/3proxy.cfg / /etc/systemd/system/3proxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "3proxy", visible_alias = "3proxy-daemon")]
        threeproxy: bool,
        /// Also configure Polipo caching web proxy (/etc/polipo/config / ~/.polipo / ~/.config/polipo/config / /etc/systemd/system/polipo.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "polipo-proxy", visible_alias = "polipo-daemon")]
        polipo: bool,
        /// Also configure Pproxy Python proxy daemon (/etc/pproxy.conf / ~/.config/pproxy/pproxy.conf / ~/.pproxy.conf / /etc/systemd/system/pproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pproxy-daemon", visible_alias = "python-proxy")]
        pproxy: bool,
        /// Also configure MicroSOCKS SOCKS5 proxy daemon (/etc/microsocks.conf / ~/.config/microsocks/microsocks.conf / ~/.config/microsocks/config / /etc/systemd/system/microsocks.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "microsocks-daemon", visible_alias = "microsocks-proxy")]
        microsocks: bool,
        /// Also configure Cntlm authenticating HTTP proxy daemon (/etc/cntlm.conf / ~/.cntlm.conf / /etc/systemd/system/cntlm.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "cntlm-daemon", visible_alias = "cntlm-proxy")]
        cntlm: bool,
        /// Also configure COW HTTP proxy daemon (~/.cow/rc / /etc/cow/rc / /etc/systemd/system/cow.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "cow-daemon", visible_alias = "cow-proxy")]
        cow: bool,
        /// Also configure Sniproxy SNI proxy daemon (/etc/sniproxy.conf / ~/.config/sniproxy/sniproxy.conf / /etc/systemd/system/sniproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "sniproxy-daemon", visible_alias = "sniproxy-proxy")]
        sniproxy: bool,
        /// Also configure Dnsproxy AdGuard DNS proxy daemon (~/.config/dnsproxy/config.yaml / /etc/dnsproxy/config.yaml / /etc/systemd/system/dnsproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "dnsproxy-daemon", visible_alias = "dnsproxy-proxy")]
        dnsproxy: bool,
        /// Also configure Mosdns Pluggable DNS forwarder daemon (~/.config/mosdns/config.yaml / /etc/mosdns/config.yaml / /etc/systemd/system/mosdns.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "mosdns-daemon", visible_alias = "mosdns-proxy")]
        mosdns: bool,
        /// Also configure SmartDNS local DNS proxy daemon (~/.config/smartdns/smartdns.conf / /etc/smartdns/smartdns.conf / /etc/systemd/system/smartdns.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "smartdns-daemon", visible_alias = "smartdns-proxy")]
        smartdns: bool,
        /// Also configure CoreDNS DNS server and forwarder daemon (~/.config/coredns/Corefile / /etc/coredns/Corefile / /etc/systemd/system/coredns.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "coredns-daemon", visible_alias = "coredns-proxy")]
        coredns: bool,
        /// Also configure AdGuard Home DNS and web proxy daemon (~/.config/AdGuardHome/AdGuardHome.yaml / /etc/AdGuardHome/AdGuardHome.yaml / /etc/systemd/system/AdGuardHome.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "adguardhome", visible_alias = "adguard-home", visible_alias = "adguard-daemon")]
        adguard: bool,
        /// Also configure DNSCrypt-proxy Encrypted DNS proxy daemon (~/.config/dnscrypt-proxy/dnscrypt-proxy.toml / /etc/dnscrypt-proxy/dnscrypt-proxy.toml / /etc/systemd/system/dnscrypt-proxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "dnscrypt-daemon", visible_alias = "dnscrypt-proxy")]
        dnscrypt: bool,
        /// Also configure Unbound validating recursive DNS proxy daemon (~/.config/unbound/unbound.conf / /etc/unbound/unbound.conf / /etc/systemd/system/unbound.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "unbound-daemon", visible_alias = "unbound-proxy")]
        unbound: bool,
        /// Also configure Dnsmasq lightweight DNS and DHCP proxy daemon (~/.config/dnsmasq/dnsmasq.conf / /etc/dnsmasq.conf / /etc/systemd/system/dnsmasq.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "dnsmasq-daemon", visible_alias = "dnsmasq-proxy")]
        dnsmasq: bool,
        /// Also configure Pi-hole FTL DNS and sinkhole proxy daemon (~/.config/pihole/pihole-FTL.conf / /etc/pihole/pihole-FTL.conf / /etc/systemd/system/pihole-FTL.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pihole-daemon", visible_alias = "pihole-ftl", visible_alias = "pi-hole")]
        pihole: bool,
        /// Also configure Blocky fast DNS and ad-blocker proxy daemon (~/.config/blocky/config.yml / /etc/blocky/config.yml / /etc/systemd/system/blocky.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "blocky-daemon", visible_alias = "blocky-proxy")]
        blocky: bool,
        /// Also configure Stubby DNS Privacy DNS-over-TLS stub resolver proxy daemon (~/.config/stubby/stubby.yml / /etc/stubby/stubby.yml / /etc/systemd/system/stubby.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "stubby-daemon", visible_alias = "stubby-proxy")]
        stubby: bool,
        /// Also configure Knot Resolver validating DNS resolver proxy daemon (~/.config/knot-resolver/kresd.conf / /etc/knot-resolver/kresd.conf / /etc/systemd/system/kresd@.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "kresd-daemon", visible_alias = "kresd-proxy", visible_alias = "knot-resolver")]
        kresd: bool,
        /// Also configure PowerDNS Recursor resolving DNS cache proxy daemon (~/.config/powerdns/recursor.conf / /etc/powerdns/recursor.conf / /etc/systemd/system/pdns-recursor.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pdns-daemon", visible_alias = "pdns-recursor", visible_alias = "powerdns")]
        pdns: bool,
        /// Also configure dnsdist DNS load balancer & DoH/DoT proxy daemon (~/.config/dnsdist/dnsdist.conf / /etc/dnsdist/dnsdist.conf / /etc/systemd/system/dnsdist.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "dnsdist-daemon", visible_alias = "dnsdist-proxy")]
        dnsdist: bool,
        /// Also configure BIND 9 (named) domain name system & resolver proxy daemon (~/.config/bind/named.conf / /etc/bind/named.conf / /etc/named.conf / /etc/systemd/system/named.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "bind-daemon", visible_alias = "bind9", visible_alias = "named")]
        bind: bool,
        /// Also configure NextDNS encrypted DNS client & proxy daemon (~/.config/nextdns/nextdns.conf / /etc/nextdns.conf / /etc/systemd/system/nextdns.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "nextdns-daemon", visible_alias = "nextdns-proxy")]
        nextdns: bool,
        /// Also configure pdnsd proxy DNS daemon (~/.config/pdnsd/pdnsd.conf / /etc/pdnsd.conf / /etc/systemd/system/pdnsd.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pdnsd-daemon", visible_alias = "pdnsd-proxy")]
        pdnsd: bool,
        /// Also configure MaraDNS / Deadwood recursive DNS cache daemon (~/.mararc / /etc/mararc / ~/.dwood3rc / /etc/dwood3rc / /etc/systemd/system/maradns.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "maradns-daemon", visible_alias = "maradns-proxy", visible_alias = "deadwood", visible_alias = "deadwood-daemon", visible_alias = "deadwood-proxy")]
        maradns: bool,
        /// Also configure i2pd anonymous network HTTP & SOCKS proxy daemon (~/.config/i2pd/i2pd.conf / /etc/i2pd/i2pd.conf / /etc/systemd/system/i2pd.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "i2pd-daemon", visible_alias = "i2pd-proxy", visible_alias = "i2p", visible_alias = "i2p-daemon", visible_alias = "i2p-proxy")]
        i2pd: bool,
        /// Also configure dae eBPF transparent proxy & routing daemon (~/.config/dae/config.dae / /etc/dae/config.dae / /etc/systemd/system/dae.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "dae-daemon", visible_alias = "dae-proxy", visible_alias = "daed", visible_alias = "daed-daemon", visible_alias = "daed-proxy")]
        dae: bool,
        /// Also configure Yggdrasil end-to-end encrypted IPv6 mesh routing daemon (~/.config/yggdrasil/yggdrasil.conf / /etc/yggdrasil.conf / /etc/systemd/system/yggdrasil.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "yggdrasil-daemon", visible_alias = "yggdrasil-proxy", visible_alias = "ygg", visible_alias = "ygg-daemon", visible_alias = "ygg-proxy")]
        yggdrasil: bool,
        /// Also configure cjdns encrypted IPv6 mesh routing daemon (~/.config/cjdns/cjdroute.conf / /etc/cjdroute.conf / /etc/systemd/system/cjdns.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "cjdns-daemon", visible_alias = "cjdns-proxy", visible_alias = "cjdroute", visible_alias = "cjdroute-daemon")]
        cjdns: bool,
        /// Also configure tinc mesh VPN daemon (~/.config/tinc/tinc.conf / /etc/tinc/tinc.conf / /etc/systemd/system/tinc.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tinc-daemon", visible_alias = "tinc-proxy", visible_alias = "tinc-vpn")]
        tinc: bool,
        /// Also configure OpenVPN secure tunneling daemon (/etc/openvpn/client.conf / /etc/openvpn/openvpn.conf / ~/.config/openvpn/openvpn.conf / /etc/systemd/system/openvpn.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "openvpn-daemon", visible_alias = "openvpn-proxy", visible_alias = "ovpn", visible_alias = "ovpn-proxy")]
        openvpn: bool,
        /// Also configure SoftEther multi-protocol VPN daemon (/usr/local/vpnclient/vpn_client.config / /etc/softether/vpnclient.config / ~/.config/softether/vpnclient.config / /etc/systemd/system/vpnclient.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "softether-daemon", visible_alias = "softether-proxy", visible_alias = "vpnclient", visible_alias = "vpnclient-daemon")]
        softether: bool,
        /// Also configure OpenConnect SSL VPN client daemon (/etc/openconnect/openconnect.conf / ~/.config/openconnect/openconnect.conf / /etc/systemd/system/openconnect.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "openconnect-daemon", visible_alias = "openconnect-proxy", visible_alias = "anyconnect", visible_alias = "anyconnect-proxy")]
        openconnect: bool,
        /// Also configure strongSwan IPsec IKEv2 daemon (/etc/strongswan.conf / ~/.config/strongswan/strongswan.conf / /etc/ipsec.conf / /etc/systemd/system/strongswan.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "strongswan-daemon", visible_alias = "strongswan-proxy", visible_alias = "ipsec", visible_alias = "ipsec-daemon", visible_alias = "charon")]
        strongswan: bool,
        /// Also configure WireGuard secure VPN tunnel daemon (/etc/wireguard/wg0.conf / ~/.config/wireguard/wg0.conf / /etc/systemd/system/wg-quick@.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "wireguard-daemon", visible_alias = "wireguard-proxy", visible_alias = "wg", visible_alias = "wg-quick")]
        wireguard: bool,
        /// Also configure Headscale Tailscale control server daemon (/etc/headscale/config.yaml / ~/.config/headscale/config.yaml / /etc/systemd/system/headscale.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "headscale-daemon", visible_alias = "headscale-proxy", visible_alias = "headscale-server")]
        headscale: bool,
        /// Also configure NetBird zero-trust mesh VPN daemon (/etc/netbird/config.json / ~/.config/netbird/config.json / /etc/systemd/system/netbird.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "netbird-daemon", visible_alias = "netbird-proxy", visible_alias = "netbird-vpn")]
        netbird: bool,
        /// Also configure Cloudflare WARP client daemon (/etc/cloudflare-warp/settings.json / ~/.local/share/warp/settings.json / /etc/systemd/system/warp-svc.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "warp-daemon", visible_alias = "warp-proxy", visible_alias = "warp-cli", visible_alias = "warp-svc", visible_alias = "cloudflare-warp")]
        warp: bool,
        /// Also configure Amnezia VPN daemon (/etc/amnezia/amneziavpn/config.json / ~/.config/AmneziaVPN/AmneziaVPN.json / /etc/systemd/system/amnezia-vpn.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "amnezia-daemon", visible_alias = "amnezia-proxy", visible_alias = "amnezia-vpn", visible_alias = "amneziawg")]
        amnezia: bool,
        /// Also configure OpenZiti zero-trust overlay daemon (/etc/ziti/config.json / ~/.config/ziti/config.json / /etc/systemd/system/ziti-edge-tunnel.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "openziti-daemon", visible_alias = "openziti-proxy", visible_alias = "ziti-tunnel", visible_alias = "ziti")]
        openziti: bool,
        /// Also configure Firezone zero-trust mesh & WireGuard daemon (/etc/firezone/client.json / ~/.config/firezone/client.json / /etc/systemd/system/firezone-client.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "firezone-daemon", visible_alias = "firezone-proxy", visible_alias = "firezone-vpn", visible_alias = "firezone-client", visible_alias = "firezoned")]
        firezone: bool,
        /// Also configure Netmaker WireGuard mesh client daemon (/etc/netclient/netclient.yml / ~/.config/netclient/netclient.yml / /etc/systemd/system/netclient.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "netmaker-daemon", visible_alias = "netmaker-proxy", visible_alias = "netclient", visible_alias = "netmaker-vpn")]
        netmaker: bool,
        /// Also configure Pritunl enterprise VPN daemon (/etc/pritunl/pritunl.conf / ~/.config/pritunl/pritunl.conf / /etc/systemd/system/pritunl-client.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pritunl-daemon", visible_alias = "pritunl-proxy", visible_alias = "pritunl-client", visible_alias = "pritunl-vpn")]
        pritunl: bool,
        /// Also configure Twingate zero-trust connector & client daemon (/etc/twingate/connector.conf / ~/.config/twingate/client.conf / /etc/systemd/system/twingate-connector.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "twingate-daemon", visible_alias = "twingate-proxy", visible_alias = "twingate-connector", visible_alias = "twingate-client")]
        twingate: bool,
        /// Also configure EasyTier decentralized mesh VPN daemon (/etc/easytier/config.toml / ~/.config/easytier/config.toml / /etc/systemd/system/easytier.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "easytier-daemon", visible_alias = "easytier-proxy", visible_alias = "easytier-vpn", visible_alias = "easytier-mesh")]
        easytier: bool,
        /// Also configure Teleport access & zero-trust proxy daemon (/etc/teleport.yaml / ~/.config/teleport/teleport.yaml / /etc/systemd/system/teleport.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "teleport-daemon", visible_alias = "teleport-proxy", visible_alias = "teleport-client", visible_alias = "teleport-access")]
        teleport: bool,
        /// Also configure Pomerium access & zero-trust proxy daemon (/etc/pomerium/config.yaml / ~/.config/pomerium/config.yaml / /etc/systemd/system/pomerium.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pomerium-daemon", visible_alias = "pomerium-proxy", visible_alias = "pomerium-access", visible_alias = "pomerium-ztna")]
        pomerium: bool,
        /// Also configure Innernet decentralized mesh VPN daemon (/etc/innernet/innernet-server.conf / ~/.config/innernet/innernet.conf / /etc/systemd/system/innernet-server.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "innernet-daemon", visible_alias = "innernet-proxy", visible_alias = "innernet-vpn", visible_alias = "innernet-mesh")]
        innernet: bool,
        /// Also configure Defguard enterprise WireGuard 2FA & SSO VPN gateway daemon (/etc/defguard/client.toml / ~/.config/defguard/client.toml / /etc/systemd/system/defguard-client.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "defguard-daemon", visible_alias = "defguard-proxy", visible_alias = "defguard-vpn", visible_alias = "defguard-client")]
        defguard: bool,
        /// Also configure Inlets cloud-native tunnel & reverse proxy daemon (/etc/inlets/inlets.conf / ~/.config/inlets/config.yaml / /etc/systemd/system/inlets-pro.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "inlets-tunnel", visible_alias = "inlets-proxy", visible_alias = "inlets-daemon", visible_alias = "inlets-pro")]
        inlets: bool,
        /// Also configure zrok peer-to-peer sharing & zero trust tunnel daemon (~/.zrok/config.json / /etc/zrok/config.json / /etc/systemd/system/zrok.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "zrok-tunnel", visible_alias = "zrok-proxy", visible_alias = "zrok-daemon", visible_alias = "zrok-share")]
        zrok: bool,
        /// Also configure n2n peer-to-peer mesh VPN & supernode daemon (/etc/n2n/edge.conf / ~/.config/n2n/edge.conf / /etc/systemd/system/edge.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "n2n-daemon", visible_alias = "n2n-proxy", visible_alias = "n2n-vpn", visible_alias = "n2n-mesh")]
        n2n: bool,
        /// Also configure Hyprspace peer-to-peer mesh VPN daemon (~/.config/hyprspace/hyprspace.yaml / /etc/hyprspace/hyprspace.yaml / /etc/systemd/system/hyprspace.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "hyprspace-vpn", visible_alias = "hyprspace-mesh", visible_alias = "hyprspace-proxy", visible_alias = "hyprspace-daemon")]
        hyprspace: bool,
        /// Also configure Fastd secure tunneling & mesh VPN daemon (/etc/fastd/fastd.conf / ~/.config/fastd/fastd.conf / /etc/systemd/system/fastd.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "fastd-vpn", visible_alias = "fastd-tunnel", visible_alias = "fastd-proxy", visible_alias = "fastd-daemon")]
        fastd: bool,
        /// Also configure EdgeVPN peer-to-peer mesh VPN daemon (~/.config/edgevpn/config.yaml / /etc/edgevpn/config.yaml / /etc/systemd/system/edgevpn.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "edgevpn-vpn", visible_alias = "edgevpn-mesh", visible_alias = "edgevpn-proxy", visible_alias = "edgevpn-daemon")]
        edgevpn: bool,
        /// Also configure boringproxy reverse proxy & automated tunnel daemon (~/.config/boringproxy/boringproxy.json / /etc/boringproxy/boringproxy.json / /etc/systemd/system/boringproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "boringproxy-tunnel", visible_alias = "boringproxy-proxy", visible_alias = "boringproxy-daemon", visible_alias = "boringproxy-client")]
        boringproxy: bool,
        /// Also configure Ghostunnel mutual TLS proxy daemon (~/.config/ghostunnel/ghostunnel.conf / /etc/ghostunnel/ghostunnel.conf / /etc/systemd/system/ghostunnel.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "ghostunnel-proxy", visible_alias = "ghostunnel-tls", visible_alias = "ghostunnel-daemon", visible_alias = "ghostunnel-client")]
        ghostunnel: bool,
        /// Also configure FreeLAN peer-to-peer mesh VPN daemon (/etc/freelan/freelan.conf / ~/.config/freelan/freelan.conf / /etc/systemd/system/freelan.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "freelan-vpn", visible_alias = "freelan-p2p", visible_alias = "freelan-daemon", visible_alias = "freelan-proxy")]
        freelan: bool,
        /// Also configure OmniEdge decentralized mesh VPN daemon (~/.omniedge/omniedge.json / /etc/omniedge/omniedge.json / /etc/systemd/system/omniedge.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "omniedge-vpn", visible_alias = "omniedge-mesh", visible_alias = "omniedge-daemon", visible_alias = "omniedge-proxy")]
        omniedge: bool,
        /// Also configure Lantern censorship circumvention proxy daemon (~/.lantern/settings.yaml / /etc/lantern/settings.yaml / /etc/systemd/system/lantern.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "lantern-proxy", visible_alias = "lantern-vpn", visible_alias = "lantern-daemon", visible_alias = "lantern-client")]
        lantern: bool,
        /// Also configure TunSafe WireGuard VPN client daemon (/etc/tunsafe/tunsafe.conf / ~/.config/tunsafe/tunsafe.conf / /etc/systemd/system/tunsafe.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tunsafe-vpn", visible_alias = "tunsafe-wg", visible_alias = "tunsafe-daemon", visible_alias = "tunsafe-proxy")]
        tunsafe: bool,
        /// Also configure Sshuttle transparent SSH proxy daemon (~/.config/sshuttle/sshuttle.conf / /etc/sshuttle/sshuttle.conf / /etc/systemd/system/sshuttle.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "sshuttle-proxy", visible_alias = "sshuttle-vpn", visible_alias = "sshuttle-daemon", visible_alias = "sshuttle-ssh")]
        sshuttle: bool,
        /// Also configure Psiphon censorship circumvention proxy daemon (~/.config/psiphon/psiphon.config / /etc/psiphon/psiphon.config / /etc/systemd/system/psiphon.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "psiphon-vpn", visible_alias = "psiphon-tunnel", visible_alias = "psiphon-daemon", visible_alias = "psiphon-proxy")]
        psiphon: bool,
        /// Also configure Localtunnel localhost reverse tunnel daemon (~/.localtunnelrc / ~/.config/localtunnel/config.json / /etc/localtunnel/config.json / /etc/systemd/system/localtunnel.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "localtunnel-proxy", visible_alias = "localtunnel-tunnel", visible_alias = "localtunnel-daemon", visible_alias = "lt")]
        localtunnel: bool,
        /// Also configure Pagekite localhost tunnel & reverse proxy daemon (~/.pagekite.rc / /etc/pagekite.d/pagekite.rc / /etc/systemd/system/pagekite.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pagekite-tunnel", visible_alias = "pagekite-daemon", visible_alias = "pagekite-proxy", visible_alias = "kite")]
        pagekite: bool,
        /// Also configure Toxiproxy resiliency & fault injection proxy daemon (~/.config/toxiproxy/toxiproxy.json / /etc/toxiproxy/toxiproxy.json / /etc/systemd/system/toxiproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "toxiproxy-daemon", visible_alias = "toxiproxy-proxy", visible_alias = "toxi")]
        toxiproxy: bool,
        /// Also configure Telepresence Kubernetes microservice local proxy & intercept daemon (~/.config/telepresence/config.yml / /etc/telepresence/config.yml / /etc/systemd/system/telepresence.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "telepresence-proxy", visible_alias = "telepresence-daemon", visible_alias = "tp")]
        telepresence: bool,
        /// Also configure LocalXpose localhost reverse proxy & tunnel daemon (~/.localxpose/config.yaml / /etc/localxpose/config.yaml / /etc/systemd/system/loclx.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "localxpose-proxy", visible_alias = "localxpose-tunnel", visible_alias = "loclx")]
        localxpose: bool,
        /// Also configure Packetriot localhost reverse tunnel & proxy daemon (~/.pktriot/config.json / /etc/pktriot/config.json / /etc/systemd/system/pktriot.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "packetriot-daemon", visible_alias = "packetriot-proxy", visible_alias = "pktriot")]
        packetriot: bool,
        /// Also configure sish HTTP/TCP localhost tunnel & reverse proxy daemon (~/.sish/config.yaml / /etc/sish/config.yaml / /etc/systemd/system/sish.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "sish-daemon", visible_alias = "sish-proxy", visible_alias = "sish-tunnel")]
        sish: bool,
        /// Also configure Pinggy HTTP/TCP localhost tunnel & reverse proxy daemon (~/.pinggy/config.json / /etc/pinggy/config.json / /etc/systemd/system/pinggy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pinggy-tunnel", visible_alias = "pinggy-proxy", visible_alias = "pinggy-daemon")]
        pinggy: bool,
        /// Also configure Loophole E2EE localhost reverse proxy & tunnel daemon (~/.loophole/config.yaml / /etc/loophole/config.yaml / /etc/systemd/system/loophole.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "loophole-proxy", visible_alias = "loophole-tunnel", visible_alias = "loophole-daemon")]
        loophole: bool,
        /// Also configure Tunnelmole localhost reverse proxy & tunnel daemon (~/.tunnelmole/config.json / /etc/tunnelmole/config.json / /etc/systemd/system/tunnelmole.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tunnelmole-tunnel", visible_alias = "tunnelmole-proxy", visible_alias = "tunnelmole-daemon", visible_alias = "tmole")]
        tunnelmole: bool,
        /// Also configure Expose localhost reverse proxy & tunnel daemon (~/.expose/config.php / ~/.expose/config.json / /etc/systemd/system/expose.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "expose-proxy", visible_alias = "expose-tunnel", visible_alias = "expose-daemon", visible_alias = "beyondcode-expose")]
        expose: bool,
        /// Also configure Playit localhost reverse proxy & tunnel daemon (~/.config/playit/playit.toml / /etc/playit/playit.toml / /etc/systemd/system/playit.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "playit-tunnel", visible_alias = "playit-proxy", visible_alias = "playit-daemon", visible_alias = "playit-gg")]
        playit: bool,
        /// Also configure pgrok localhost reverse proxy & tunnel daemon (~/.pgrok/pgrok.yml / /etc/pgrok/pgrok.yml / /etc/systemd/system/pgrok.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pgrok-proxy", visible_alias = "pgrok-tunnel", visible_alias = "pgrok-daemon")]
        pgrok: bool,
        /// Also configure cpolar localhost reverse proxy & tunnel daemon (~/.cpolar/cpolar.yml / /etc/cpolar/cpolar.yml / /etc/systemd/system/cpolar.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "cpolar-proxy", visible_alias = "cpolar-tunnel", visible_alias = "cpolar-daemon")]
        cpolar: bool,
        /// Also configure tmate terminal sharing & SSH reverse tunnel daemon (~/.tmate.conf / /etc/tmate.conf / /etc/systemd/system/tmate.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tmate-proxy", visible_alias = "tmate-tunnel", visible_alias = "tmate-daemon")]
        tmate: bool,
        /// Also configure upterm terminal sharing & SSH reverse tunnel daemon (~/.config/upterm/config.yml / /etc/upterm/config.yml / /etc/systemd/system/upterm.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "upterm-proxy", visible_alias = "upterm-tunnel", visible_alias = "upterm-daemon")]
        upterm: bool,
        /// Also configure NATAPP localhost reverse proxy & tunnel daemon (~/.natapp/config.ini / /etc/natapp/config.ini / /etc/systemd/system/natapp.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "natapp-proxy", visible_alias = "natapp-tunnel", visible_alias = "natapp-daemon")]
        natapp: bool,
        /// Also configure lanproxy intranet penetration & reverse tunnel client (~/.lanproxy/config.properties / /etc/lanproxy/config.properties / /etc/systemd/system/lanproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "lanproxy-proxy", visible_alias = "lanproxy-tunnel", visible_alias = "lanproxy-client")]
        lanproxy: bool,
        /// Also configure teleconsole terminal sharing & SSH reverse tunnel daemon (~/.teleconsole/config.yaml / /etc/teleconsole/config.yaml / /etc/systemd/system/teleconsole.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "teleconsole-proxy", visible_alias = "teleconsole-tunnel", visible_alias = "teleconsole-daemon")]
        teleconsole: bool,
        /// Also configure tunnelto localhost reverse proxy & public tunnel client (~/.tunnelto/config.toml / /etc/tunnelto/config.toml / /etc/systemd/system/tunnelto.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tunnelto-proxy", visible_alias = "tunnelto-tunnel", visible_alias = "tunnelto-client")]
        tunnelto: bool,
        /// Also configure Webhook Relay localhost tunnel & webhook forwarding daemon (~/.relay/config.json / /etc/webhookrelay/config.json / /etc/systemd/system/webhookrelay.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "webhookrelay-proxy", visible_alias = "webhookrelay-tunnel", visible_alias = "webhookrelay-daemon", visible_alias = "relay")]
        webhookrelay: bool,
        /// Also configure jprq localhost reverse tunnel & port forwarding daemon (~/.jprq/config.json / /etc/jprq/config.json / /etc/systemd/system/jprq.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "jprq-proxy", visible_alias = "jprq-tunnel", visible_alias = "jprq-daemon")]
        jprq: bool,
        /// Also configure mysocketctl / Border0 localhost reverse tunnel & zero-trust client (~/.mysocket/config.json / /etc/mysocketctl/config.json / /etc/systemd/system/mysocketctl.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "mysocketctl-proxy", visible_alias = "mysocketctl-tunnel", visible_alias = "mysocket-proxy", visible_alias = "border0")]
        mysocketctl: bool,
        /// Also configure openp2p decentralized P2P reverse tunnel & port forwarding daemon (~/.openp2p/config.json / /etc/openp2p/config.json / /etc/systemd/system/openp2p.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "openp2p-proxy", visible_alias = "openp2p-tunnel", visible_alias = "openp2p-daemon")]
        openp2p: bool,
        /// Also configure rport remote management & reverse tunnel client daemon (/etc/rport/rport.conf / ~/.rport/rport.conf / /etc/systemd/system/rport.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "rport-proxy", visible_alias = "rport-tunnel", visible_alias = "rport-client", visible_alias = "rport-daemon")]
        rport: bool,
        /// Also configure mirrord Kubernetes dev traffic mirror & reverse proxy client daemon (~/.mirrord/mirrord.json / /etc/mirrord/mirrord.json / /etc/systemd/system/mirrord.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "mirrord-proxy", visible_alias = "mirrord-tunnel", visible_alias = "metalbear")]
        mirrord: bool,
        /// Also configure kt-connect Kubernetes dev reverse tunnel & SOCKS5 proxy daemon (~/.ktctl/config.json / /etc/ktctl/config.json / /etc/systemd/system/ktctl.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "kt-connect-proxy", visible_alias = "kt-connect-tunnel", visible_alias = "ktctl", visible_alias = "kt-connect-daemon")]
        kt_connect: bool,
        /// Also configure DevSpace Cloud-Native Kubernetes dev proxy & port forwarder daemon (~/.devspace/config.json / devspace.yaml / /etc/devspace/config.json / /etc/systemd/system/devspace.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "devspace-proxy", visible_alias = "devspace-tunnel", visible_alias = "loft")]
        devspace: bool,
        /// Also configure Geph censorship-resistant proxy & reverse tunnel client daemon (~/.config/geph4/geph4.json / /etc/geph4/geph4.json / /etc/systemd/system/geph4.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "geph-proxy", visible_alias = "geph-tunnel", visible_alias = "geph4", visible_alias = "geph5")]
        geph: bool,
        /// Also configure Okteto Cloud-Native Kubernetes dev reverse tunnel & SSH daemon (~/.okteto/config.json / okteto.yaml / /etc/okteto/config.json / /etc/systemd/system/okteto.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "okteto-proxy", visible_alias = "okteto-tunnel", visible_alias = "okteto-daemon")]
        okteto: bool,
        /// Also configure Skaffold Cloud-Native Kubernetes dev port forward & control daemon (~/.skaffold/config / skaffold.yaml / /etc/skaffold/config.yaml / /etc/systemd/system/skaffold.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "skaffold-proxy", visible_alias = "skaffold-port-forward", visible_alias = "skaffold-daemon")]
        skaffold: bool,
        /// Also configure git, cargo, npm, yarn, pnpm, bun, pip, conda, docker, gradle, maven, vscode, gh, glab, jetbrains, nuget, gem, svn, hg, uv, go, bazel, composer, conan, sbt, kube, julia, nix, cabal, stack, hex, r, opam, lein, cpan, luarocks, nimble, rebar3, scoop, bundle, gcloud, aws, azure, mamba, podman, vagrant, choco, colima, minikube, spack, macports, apt, dnf, zypper, pacman, pkg, portage, docker daemon, flatpak, snap, ollama, containerd, buildkit, k3s, microk8s, k0s, rke2, nomad, kubelet, gitlab-runner, actions-runner, jenkins, buildkite-agent, woodpecker-agent, drone-runner, act-runner, circleci-runner, azure-pipelines-agent, teamcity-agent, bamboo-agent, tfc-agent, concourse-worker, tentacle-agent, puppet-agent, chef-client, salt-minion, datadog-agent, newrelic-infra, elastic-agent, grafana-alloy, otelcol, vector, fluent-bit, fluentd, telegraf, logstash, prometheus, alertmanager, promtail, vmagent, filebeat, metricbeat, heartbeat, auditbeat, packetbeat, winlogbeat, consul, vault, traefik, caddy, envoy, haproxy, nginx, kong, apisix, squid, privoxy, tinyproxy, mitmproxy, tor, cloudflared, ngrok, frpc, tailscale, zerotier, gost, rathole, chisel, nebula, bore, brook, npc, shadowsocks, v2ray, sing-box, hysteria, trojan, naive, tuic, juicity, snell, leaf, mihomo, stunnel, shadow-tls, tun2socks, wstunnel, cloak, wireproxy, mtg, kcptun, glider, redsocks, dante, 3proxy, polipo, pproxy, microsocks, cntlm, cow, sniproxy, dnsproxy, mosdns, smartdns, coredns, adguard, dnscrypt, unbound, dnsmasq, pihole, blocky, stubby, kresd, pdns, dnsdist, bind, nextdns, pdnsd, maradns, i2pd, dae, yggdrasil, cjdns, tinc, openvpn, softether, openconnect, strongswan, wireguard, headscale, netbird, warp, amnezia, openziti, firezone, netmaker, pritunl, twingate, easytier, teleport, pomerium, innernet, defguard, inlets, zrok, n2n, hyprspace, fastd, edgevpn, boringproxy, ghostunnel, freelan, omniedge, lantern, tunsafe, sshuttle, psiphon, localtunnel, pagekite, toxiproxy, telepresence, localxpose, packetriot, sish, pinggy, loophole, tunnelmole, expose, playit, pgrok, cpolar, tmate, upterm, natapp, lanproxy, teleconsole, tunnelto, webhookrelay, jprq, mysocketctl, openp2p, rport, mirrord, kt-connect, devspace, geph, okteto, and skaffold proxy
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
        /// Also unset Mihomo daemon proxy (/etc/mihomo/config.yaml / ~/.config/mihomo/config.yaml / /etc/systemd/system/mihomo.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "mihomo-daemon", visible_alias = "clash-meta")]
        mihomo: bool,
        /// Also unset Stunnel daemon proxy (/etc/stunnel/stunnel.conf / ~/.config/stunnel/stunnel.conf / /etc/systemd/system/stunnel.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "stunnel-proxy", visible_alias = "stunnel4")]
        stunnel: bool,
        /// Also unset Shadow-TLS daemon proxy (/etc/shadow-tls/config.json / ~/.config/shadow-tls/config.json / /etc/systemd/system/shadow-tls.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "shadowtls", visible_alias = "shadow-tls-proxy")]
        shadow_tls: bool,
        /// Also unset Tun2socks daemon proxy (/etc/tun2socks/config.yaml / ~/.config/tun2socks/config.yaml / /etc/systemd/system/tun2socks.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tun2socks-daemon", visible_alias = "tun2socks-proxy")]
        tun2socks: bool,
        /// Also unset WSTunnel daemon proxy (/etc/wstunnel/config.yaml / ~/.config/wstunnel/config.yaml / /etc/systemd/system/wstunnel.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "wstunnel-daemon", visible_alias = "wstunnel-proxy")]
        wstunnel: bool,
        /// Also unset Cloak daemon proxy (/etc/cloak/ckclient.json / ~/.config/cloak/ckclient.json / /etc/systemd/system/cloak.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "cloak-proxy", visible_alias = "ck-client")]
        cloak: bool,
        /// Also unset Wireproxy daemon proxy (/etc/wireproxy/config.yaml / ~/.config/wireproxy/config.yaml / /etc/systemd/system/wireproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "wireproxy-daemon", visible_alias = "wireproxy-proxy")]
        wireproxy: bool,
        /// Also unset MTG daemon proxy (/etc/mtg.toml / ~/.config/mtg/config.toml / /etc/systemd/system/mtg.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "mtg-daemon", visible_alias = "mtg-proxy")]
        mtg: bool,
        /// Also unset Kcptun daemon proxy (/etc/kcptun/client.json / ~/.config/kcptun/client.json / /etc/systemd/system/kcptun.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "kcptun-client", visible_alias = "kcptun-proxy")]
        kcptun: bool,
        /// Also unset Glider daemon proxy (/etc/glider/glider.conf / ~/.config/glider/glider.conf / /etc/systemd/system/glider.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "glider-proxy", visible_alias = "glider-daemon")]
        glider: bool,
        /// Also unset Redsocks daemon proxy (/etc/redsocks.conf / ~/.config/redsocks/redsocks.conf / /etc/systemd/system/redsocks.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "redsocks-proxy", visible_alias = "redsocks-daemon")]
        redsocks: bool,
        /// Also unset Dante daemon proxy (/etc/danted.conf / ~/.config/danted.conf / /etc/systemd/system/danted.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "dante-proxy", visible_alias = "danted")]
        dante: bool,
        /// Also unset 3proxy multi-protocol daemon proxy (/etc/3proxy/3proxy.cfg / ~/.3proxy.cfg / ~/.config/3proxy/3proxy.cfg / /etc/systemd/system/3proxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "3proxy", visible_alias = "3proxy-daemon")]
        threeproxy: bool,
        /// Also unset Polipo caching web proxy (/etc/polipo/config / ~/.polipo / ~/.config/polipo/config / /etc/systemd/system/polipo.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "polipo-proxy", visible_alias = "polipo-daemon")]
        polipo: bool,
        /// Also unset Pproxy Python proxy daemon (/etc/pproxy.conf / ~/.config/pproxy/pproxy.conf / ~/.pproxy.conf / /etc/systemd/system/pproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pproxy-daemon", visible_alias = "python-proxy")]
        pproxy: bool,
        /// Also unset MicroSOCKS SOCKS5 proxy daemon (/etc/microsocks.conf / ~/.config/microsocks/microsocks.conf / ~/.config/microsocks/config / /etc/systemd/system/microsocks.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "microsocks-daemon", visible_alias = "microsocks-proxy")]
        microsocks: bool,
        /// Also unset Cntlm authenticating HTTP proxy daemon (/etc/cntlm.conf / ~/.cntlm.conf / /etc/systemd/system/cntlm.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "cntlm-daemon", visible_alias = "cntlm-proxy")]
        cntlm: bool,
        /// Also unset COW HTTP proxy daemon (~/.cow/rc / /etc/cow/rc / /etc/systemd/system/cow.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "cow-daemon", visible_alias = "cow-proxy")]
        cow: bool,
        /// Also unset Sniproxy SNI proxy daemon (/etc/sniproxy.conf / ~/.config/sniproxy/sniproxy.conf / /etc/systemd/system/sniproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "sniproxy-daemon", visible_alias = "sniproxy-proxy")]
        sniproxy: bool,
        /// Also unset Dnsproxy AdGuard DNS proxy daemon (~/.config/dnsproxy/config.yaml / /etc/dnsproxy/config.yaml / /etc/systemd/system/dnsproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "dnsproxy-daemon", visible_alias = "dnsproxy-proxy")]
        dnsproxy: bool,
        /// Also unset Mosdns Pluggable DNS forwarder daemon (~/.config/mosdns/config.yaml / /etc/mosdns/config.yaml / /etc/systemd/system/mosdns.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "mosdns-daemon", visible_alias = "mosdns-proxy")]
        mosdns: bool,
        /// Also unset SmartDNS local DNS proxy daemon (~/.config/smartdns/smartdns.conf / /etc/smartdns/smartdns.conf / /etc/systemd/system/smartdns.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "smartdns-daemon", visible_alias = "smartdns-proxy")]
        smartdns: bool,
        /// Also unset CoreDNS DNS server and forwarder daemon (~/.config/coredns/Corefile / /etc/coredns/Corefile / /etc/systemd/system/coredns.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "coredns-daemon", visible_alias = "coredns-proxy")]
        coredns: bool,
        /// Also unset AdGuard Home DNS and web proxy daemon (~/.config/AdGuardHome/AdGuardHome.yaml / /etc/AdGuardHome/AdGuardHome.yaml / /etc/systemd/system/AdGuardHome.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "adguardhome", visible_alias = "adguard-home", visible_alias = "adguard-daemon")]
        adguard: bool,
        /// Also unset DNSCrypt-proxy Encrypted DNS proxy daemon (~/.config/dnscrypt-proxy/dnscrypt-proxy.toml / /etc/dnscrypt-proxy/dnscrypt-proxy.toml / /etc/systemd/system/dnscrypt-proxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "dnscrypt-daemon", visible_alias = "dnscrypt-proxy")]
        dnscrypt: bool,
        /// Also unset Unbound validating recursive DNS proxy daemon (~/.config/unbound/unbound.conf / /etc/unbound/unbound.conf / /etc/systemd/system/unbound.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "unbound-daemon", visible_alias = "unbound-proxy")]
        unbound: bool,
        /// Also unset Dnsmasq lightweight DNS and DHCP proxy daemon (~/.config/dnsmasq/dnsmasq.conf / /etc/dnsmasq.conf / /etc/systemd/system/dnsmasq.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "dnsmasq-daemon", visible_alias = "dnsmasq-proxy")]
        dnsmasq: bool,
        /// Also unset Pi-hole FTL DNS and sinkhole proxy daemon (~/.config/pihole/pihole-FTL.conf / /etc/pihole/pihole-FTL.conf / /etc/systemd/system/pihole-FTL.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pihole-daemon", visible_alias = "pihole-ftl", visible_alias = "pi-hole")]
        pihole: bool,
        /// Also unset Blocky fast DNS and ad-blocker proxy daemon (~/.config/blocky/config.yml / /etc/blocky/config.yml / /etc/systemd/system/blocky.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "blocky-daemon", visible_alias = "blocky-proxy")]
        blocky: bool,
        /// Also unset Stubby DNS Privacy DNS-over-TLS stub resolver proxy daemon (~/.config/stubby/stubby.yml / /etc/stubby/stubby.yml / /etc/systemd/system/stubby.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "stubby-daemon", visible_alias = "stubby-proxy")]
        stubby: bool,
        /// Also unset Knot Resolver validating DNS resolver proxy daemon (~/.config/knot-resolver/kresd.conf / /etc/knot-resolver/kresd.conf / /etc/systemd/system/kresd@.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "kresd-daemon", visible_alias = "kresd-proxy", visible_alias = "knot-resolver")]
        kresd: bool,
        /// Also unset PowerDNS Recursor resolving DNS cache proxy daemon (~/.config/powerdns/recursor.conf / /etc/powerdns/recursor.conf / /etc/systemd/system/pdns-recursor.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pdns-daemon", visible_alias = "pdns-recursor", visible_alias = "powerdns")]
        pdns: bool,
        /// Also unset dnsdist DNS load balancer & DoH/DoT proxy daemon (~/.config/dnsdist/dnsdist.conf / /etc/dnsdist/dnsdist.conf / /etc/systemd/system/dnsdist.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "dnsdist-daemon", visible_alias = "dnsdist-proxy")]
        dnsdist: bool,
        /// Also unset BIND 9 (named) domain name system & resolver proxy daemon (~/.config/bind/named.conf / /etc/bind/named.conf / /etc/named.conf / /etc/systemd/system/named.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "bind-daemon", visible_alias = "bind9", visible_alias = "named")]
        bind: bool,
        /// Also unset NextDNS encrypted DNS client & proxy daemon (~/.config/nextdns/nextdns.conf / /etc/nextdns.conf / /etc/systemd/system/nextdns.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "nextdns-daemon", visible_alias = "nextdns-proxy")]
        nextdns: bool,
        /// Also unset pdnsd proxy DNS daemon (~/.config/pdnsd/pdnsd.conf / /etc/pdnsd.conf / /etc/systemd/system/pdnsd.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pdnsd-daemon", visible_alias = "pdnsd-proxy")]
        pdnsd: bool,
        /// Also unset MaraDNS / Deadwood recursive DNS cache daemon (~/.mararc / /etc/mararc / ~/.dwood3rc / /etc/dwood3rc / /etc/systemd/system/maradns.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "maradns-daemon", visible_alias = "maradns-proxy", visible_alias = "deadwood", visible_alias = "deadwood-daemon", visible_alias = "deadwood-proxy")]
        maradns: bool,
        /// Also unset i2pd anonymous network HTTP & SOCKS proxy daemon (~/.config/i2pd/i2pd.conf / /etc/i2pd/i2pd.conf / /etc/systemd/system/i2pd.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "i2pd-daemon", visible_alias = "i2pd-proxy", visible_alias = "i2p", visible_alias = "i2p-daemon", visible_alias = "i2p-proxy")]
        i2pd: bool,
        /// Also unset dae eBPF transparent proxy & routing daemon (~/.config/dae/config.dae / /etc/dae/config.dae / /etc/systemd/system/dae.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "dae-daemon", visible_alias = "dae-proxy", visible_alias = "daed", visible_alias = "daed-daemon", visible_alias = "daed-proxy")]
        dae: bool,
        /// Also unset Yggdrasil end-to-end encrypted IPv6 mesh routing daemon (~/.config/yggdrasil/yggdrasil.conf / /etc/yggdrasil.conf / /etc/systemd/system/yggdrasil.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "yggdrasil-daemon", visible_alias = "yggdrasil-proxy", visible_alias = "ygg", visible_alias = "ygg-daemon", visible_alias = "ygg-proxy")]
        yggdrasil: bool,
        /// Also unset cjdns encrypted IPv6 mesh routing daemon (~/.config/cjdns/cjdroute.conf / /etc/cjdroute.conf / /etc/systemd/system/cjdns.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "cjdns-daemon", visible_alias = "cjdns-proxy", visible_alias = "cjdroute", visible_alias = "cjdroute-daemon")]
        cjdns: bool,
        /// Also unset tinc mesh VPN daemon (~/.config/tinc/tinc.conf / /etc/tinc/tinc.conf / /etc/systemd/system/tinc.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tinc-daemon", visible_alias = "tinc-proxy", visible_alias = "tinc-vpn")]
        tinc: bool,
        /// Also unset OpenVPN secure tunneling daemon (/etc/openvpn/client.conf / /etc/openvpn/openvpn.conf / ~/.config/openvpn/openvpn.conf / /etc/systemd/system/openvpn.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "openvpn-daemon", visible_alias = "openvpn-proxy", visible_alias = "ovpn", visible_alias = "ovpn-proxy")]
        openvpn: bool,
        /// Also unset SoftEther multi-protocol VPN daemon (/usr/local/vpnclient/vpn_client.config / /etc/softether/vpnclient.config / ~/.config/softether/vpnclient.config / /etc/systemd/system/vpnclient.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "softether-daemon", visible_alias = "softether-proxy", visible_alias = "vpnclient", visible_alias = "vpnclient-daemon")]
        softether: bool,
        /// Also unset OpenConnect SSL VPN client daemon (/etc/openconnect/openconnect.conf / ~/.config/openconnect/openconnect.conf / /etc/systemd/system/openconnect.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "openconnect-daemon", visible_alias = "openconnect-proxy", visible_alias = "anyconnect", visible_alias = "anyconnect-proxy")]
        openconnect: bool,
        /// Also unset strongSwan IPsec IKEv2 daemon (/etc/strongswan.conf / ~/.config/strongswan/strongswan.conf / /etc/ipsec.conf / /etc/systemd/system/strongswan.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "strongswan-daemon", visible_alias = "strongswan-proxy", visible_alias = "ipsec", visible_alias = "ipsec-daemon", visible_alias = "charon")]
        strongswan: bool,
        /// Also unset WireGuard secure VPN tunnel daemon (/etc/wireguard/wg0.conf / ~/.config/wireguard/wg0.conf / /etc/systemd/system/wg-quick@.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "wireguard-daemon", visible_alias = "wireguard-proxy", visible_alias = "wg", visible_alias = "wg-quick")]
        wireguard: bool,
        /// Also unset Headscale Tailscale control server daemon (/etc/headscale/config.yaml / ~/.config/headscale/config.yaml / /etc/systemd/system/headscale.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "headscale-daemon", visible_alias = "headscale-proxy", visible_alias = "headscale-server")]
        headscale: bool,
        /// Also unset NetBird zero-trust mesh VPN daemon (/etc/netbird/config.json / ~/.config/netbird/config.json / /etc/systemd/system/netbird.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "netbird-daemon", visible_alias = "netbird-proxy", visible_alias = "netbird-vpn")]
        netbird: bool,
        /// Also unset Cloudflare WARP client daemon (/etc/cloudflare-warp/settings.json / ~/.local/share/warp/settings.json / /etc/systemd/system/warp-svc.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "warp-daemon", visible_alias = "warp-proxy", visible_alias = "warp-cli", visible_alias = "warp-svc", visible_alias = "cloudflare-warp")]
        warp: bool,
        /// Also unset Amnezia VPN daemon (/etc/amnezia/amneziavpn/config.json / ~/.config/AmneziaVPN/AmneziaVPN.json / /etc/systemd/system/amnezia-vpn.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "amnezia-daemon", visible_alias = "amnezia-proxy", visible_alias = "amnezia-vpn", visible_alias = "amneziawg")]
        amnezia: bool,
        /// Also unset OpenZiti zero-trust overlay daemon (/etc/ziti/config.json / ~/.config/ziti/config.json / /etc/systemd/system/ziti-edge-tunnel.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "openziti-daemon", visible_alias = "openziti-proxy", visible_alias = "ziti-tunnel", visible_alias = "ziti")]
        openziti: bool,
        /// Also unset Firezone zero-trust mesh & WireGuard daemon (/etc/firezone/client.json / ~/.config/firezone/client.json / /etc/systemd/system/firezone-client.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "firezone-daemon", visible_alias = "firezone-proxy", visible_alias = "firezone-vpn", visible_alias = "firezone-client", visible_alias = "firezoned")]
        firezone: bool,
        /// Also unset Netmaker WireGuard mesh client daemon (/etc/netclient/netclient.yml / ~/.config/netclient/netclient.yml / /etc/systemd/system/netclient.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "netmaker-daemon", visible_alias = "netmaker-proxy", visible_alias = "netclient", visible_alias = "netmaker-vpn")]
        netmaker: bool,
        /// Also unset Pritunl enterprise VPN daemon (/etc/pritunl/pritunl.conf / ~/.config/pritunl/pritunl.conf / /etc/systemd/system/pritunl-client.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pritunl-daemon", visible_alias = "pritunl-proxy", visible_alias = "pritunl-client", visible_alias = "pritunl-vpn")]
        pritunl: bool,
        /// Also unset Twingate zero-trust connector & client daemon (/etc/twingate/connector.conf / ~/.config/twingate/client.conf / /etc/systemd/system/twingate-connector.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "twingate-daemon", visible_alias = "twingate-proxy", visible_alias = "twingate-connector", visible_alias = "twingate-client")]
        twingate: bool,
        /// Also unset EasyTier decentralized mesh VPN daemon (/etc/easytier/config.toml / ~/.config/easytier/config.toml / /etc/systemd/system/easytier.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "easytier-daemon", visible_alias = "easytier-proxy", visible_alias = "easytier-vpn", visible_alias = "easytier-mesh")]
        easytier: bool,
        /// Also unset Teleport access & zero-trust proxy daemon (/etc/teleport.yaml / ~/.config/teleport/teleport.yaml / /etc/systemd/system/teleport.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "teleport-daemon", visible_alias = "teleport-proxy", visible_alias = "teleport-client", visible_alias = "teleport-access")]
        teleport: bool,
        /// Also unset Pomerium access & zero-trust proxy daemon (/etc/pomerium/config.yaml / ~/.config/pomerium/config.yaml / /etc/systemd/system/pomerium.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pomerium-daemon", visible_alias = "pomerium-proxy", visible_alias = "pomerium-access", visible_alias = "pomerium-ztna")]
        pomerium: bool,
        /// Also unset Innernet decentralized mesh VPN daemon (/etc/innernet/innernet-server.conf / ~/.config/innernet/innernet.conf / /etc/systemd/system/innernet-server.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "innernet-daemon", visible_alias = "innernet-proxy", visible_alias = "innernet-vpn", visible_alias = "innernet-mesh")]
        innernet: bool,
        /// Also unset Defguard enterprise WireGuard 2FA & SSO VPN gateway daemon (/etc/defguard/client.toml / ~/.config/defguard/client.toml / /etc/systemd/system/defguard-client.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "defguard-daemon", visible_alias = "defguard-proxy", visible_alias = "defguard-vpn", visible_alias = "defguard-client")]
        defguard: bool,
        /// Also unset Inlets cloud-native tunnel & reverse proxy daemon (/etc/inlets/inlets.conf / ~/.config/inlets/config.yaml / /etc/systemd/system/inlets-pro.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "inlets-tunnel", visible_alias = "inlets-proxy", visible_alias = "inlets-daemon", visible_alias = "inlets-pro")]
        inlets: bool,
        /// Also unset zrok peer-to-peer sharing & zero trust tunnel daemon (~/.zrok/config.json / /etc/zrok/config.json / /etc/systemd/system/zrok.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "zrok-tunnel", visible_alias = "zrok-proxy", visible_alias = "zrok-daemon", visible_alias = "zrok-share")]
        zrok: bool,
        /// Also unset n2n peer-to-peer mesh VPN & supernode daemon (/etc/n2n/edge.conf / ~/.config/n2n/edge.conf / /etc/systemd/system/edge.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "n2n-daemon", visible_alias = "n2n-proxy", visible_alias = "n2n-vpn", visible_alias = "n2n-mesh")]
        n2n: bool,
        /// Also unset Hyprspace peer-to-peer mesh VPN daemon (~/.config/hyprspace/hyprspace.yaml / /etc/hyprspace/hyprspace.yaml / /etc/systemd/system/hyprspace.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "hyprspace-vpn", visible_alias = "hyprspace-mesh", visible_alias = "hyprspace-proxy", visible_alias = "hyprspace-daemon")]
        hyprspace: bool,
        /// Also unset Fastd secure tunneling & mesh VPN daemon (/etc/fastd/fastd.conf / ~/.config/fastd/fastd.conf / /etc/systemd/system/fastd.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "fastd-vpn", visible_alias = "fastd-tunnel", visible_alias = "fastd-proxy", visible_alias = "fastd-daemon")]
        fastd: bool,
        /// Also unset EdgeVPN peer-to-peer mesh VPN daemon (~/.config/edgevpn/config.yaml / /etc/edgevpn/config.yaml / /etc/systemd/system/edgevpn.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "edgevpn-vpn", visible_alias = "edgevpn-mesh", visible_alias = "edgevpn-proxy", visible_alias = "edgevpn-daemon")]
        edgevpn: bool,
        /// Also unset boringproxy reverse proxy & automated tunnel daemon (~/.config/boringproxy/boringproxy.json / /etc/boringproxy/boringproxy.json / /etc/systemd/system/boringproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "boringproxy-tunnel", visible_alias = "boringproxy-proxy", visible_alias = "boringproxy-daemon", visible_alias = "boringproxy-client")]
        boringproxy: bool,
        /// Also unset Ghostunnel mutual TLS proxy daemon (~/.config/ghostunnel/ghostunnel.conf / /etc/ghostunnel/ghostunnel.conf / /etc/systemd/system/ghostunnel.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "ghostunnel-proxy", visible_alias = "ghostunnel-tls", visible_alias = "ghostunnel-daemon", visible_alias = "ghostunnel-client")]
        ghostunnel: bool,
        /// Also unset FreeLAN peer-to-peer mesh VPN daemon (/etc/freelan/freelan.conf / ~/.config/freelan/freelan.conf / /etc/systemd/system/freelan.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "freelan-vpn", visible_alias = "freelan-p2p", visible_alias = "freelan-daemon", visible_alias = "freelan-proxy")]
        freelan: bool,
        /// Also unset OmniEdge decentralized mesh VPN daemon (~/.omniedge/omniedge.json / /etc/omniedge/omniedge.json / /etc/systemd/system/omniedge.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "omniedge-vpn", visible_alias = "omniedge-mesh", visible_alias = "omniedge-daemon", visible_alias = "omniedge-proxy")]
        omniedge: bool,
        /// Also unset Lantern censorship circumvention proxy daemon (~/.lantern/settings.yaml / /etc/lantern/settings.yaml / /etc/systemd/system/lantern.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "lantern-proxy", visible_alias = "lantern-vpn", visible_alias = "lantern-daemon", visible_alias = "lantern-client")]
        lantern: bool,
        /// Also unset TunSafe WireGuard VPN client daemon (/etc/tunsafe/tunsafe.conf / ~/.config/tunsafe/tunsafe.conf / /etc/systemd/system/tunsafe.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tunsafe-vpn", visible_alias = "tunsafe-wg", visible_alias = "tunsafe-daemon", visible_alias = "tunsafe-proxy")]
        tunsafe: bool,
        /// Also unset Sshuttle transparent SSH proxy daemon (~/.config/sshuttle/sshuttle.conf / /etc/sshuttle/sshuttle.conf / /etc/systemd/system/sshuttle.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "sshuttle-proxy", visible_alias = "sshuttle-vpn", visible_alias = "sshuttle-daemon", visible_alias = "sshuttle-ssh")]
        sshuttle: bool,
        /// Also unset Psiphon censorship circumvention proxy daemon (~/.config/psiphon/psiphon.config / /etc/psiphon/psiphon.config / /etc/systemd/system/psiphon.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "psiphon-vpn", visible_alias = "psiphon-tunnel", visible_alias = "psiphon-daemon", visible_alias = "psiphon-proxy")]
        psiphon: bool,
        /// Also unset Localtunnel localhost reverse tunnel daemon (~/.localtunnelrc / ~/.config/localtunnel/config.json / /etc/localtunnel/config.json / /etc/systemd/system/localtunnel.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "localtunnel-proxy", visible_alias = "localtunnel-tunnel", visible_alias = "localtunnel-daemon", visible_alias = "lt")]
        localtunnel: bool,
        /// Also unset Pagekite localhost tunnel & reverse proxy daemon (~/.pagekite.rc / /etc/pagekite.d/pagekite.rc / /etc/systemd/system/pagekite.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pagekite-tunnel", visible_alias = "pagekite-daemon", visible_alias = "pagekite-proxy", visible_alias = "kite")]
        pagekite: bool,
        /// Also unset Toxiproxy resiliency & fault injection proxy daemon (~/.config/toxiproxy/toxiproxy.json / /etc/toxiproxy/toxiproxy.json / /etc/systemd/system/toxiproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "toxiproxy-daemon", visible_alias = "toxiproxy-proxy", visible_alias = "toxi")]
        toxiproxy: bool,
        /// Also unset Telepresence Kubernetes microservice local proxy & intercept daemon (~/.config/telepresence/config.yml / /etc/telepresence/config.yml / /etc/systemd/system/telepresence.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "telepresence-proxy", visible_alias = "telepresence-daemon", visible_alias = "tp")]
        telepresence: bool,
        /// Also unset LocalXpose localhost reverse proxy & tunnel daemon (~/.localxpose/config.yaml / /etc/localxpose/config.yaml / /etc/systemd/system/loclx.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "localxpose-proxy", visible_alias = "localxpose-tunnel", visible_alias = "loclx")]
        localxpose: bool,
        /// Also unset Packetriot localhost reverse tunnel & proxy daemon (~/.pktriot/config.json / /etc/pktriot/config.json / /etc/systemd/system/pktriot.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "packetriot-daemon", visible_alias = "packetriot-proxy", visible_alias = "pktriot")]
        packetriot: bool,
        /// Also unset sish HTTP/TCP localhost tunnel & reverse proxy daemon (~/.sish/config.yaml / /etc/sish/config.yaml / /etc/systemd/system/sish.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "sish-daemon", visible_alias = "sish-proxy", visible_alias = "sish-tunnel")]
        sish: bool,
        /// Also unset Pinggy HTTP/TCP localhost tunnel & reverse proxy daemon (~/.pinggy/config.json / /etc/pinggy/config.json / /etc/systemd/system/pinggy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pinggy-tunnel", visible_alias = "pinggy-proxy", visible_alias = "pinggy-daemon")]
        pinggy: bool,
        /// Also unset Loophole E2EE localhost reverse proxy & tunnel daemon (~/.loophole/config.yaml / /etc/loophole/config.yaml / /etc/systemd/system/loophole.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "loophole-proxy", visible_alias = "loophole-tunnel", visible_alias = "loophole-daemon")]
        loophole: bool,
        /// Also unset Tunnelmole localhost reverse proxy & tunnel daemon (~/.tunnelmole/config.json / /etc/tunnelmole/config.json / /etc/systemd/system/tunnelmole.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tunnelmole-tunnel", visible_alias = "tunnelmole-proxy", visible_alias = "tunnelmole-daemon", visible_alias = "tmole")]
        tunnelmole: bool,
        /// Also unset Expose localhost reverse proxy & tunnel daemon (~/.expose/config.php / ~/.expose/config.json / /etc/systemd/system/expose.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "expose-proxy", visible_alias = "expose-tunnel", visible_alias = "expose-daemon", visible_alias = "beyondcode-expose")]
        expose: bool,
        /// Also unset Playit localhost reverse proxy & tunnel daemon (~/.config/playit/playit.toml / /etc/playit/playit.toml / /etc/systemd/system/playit.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "playit-tunnel", visible_alias = "playit-proxy", visible_alias = "playit-daemon", visible_alias = "playit-gg")]
        playit: bool,
        /// Also unset pgrok localhost reverse proxy & tunnel daemon (~/.pgrok/pgrok.yml / /etc/pgrok/pgrok.yml / /etc/systemd/system/pgrok.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "pgrok-proxy", visible_alias = "pgrok-tunnel", visible_alias = "pgrok-daemon")]
        pgrok: bool,
        /// Also unset cpolar localhost reverse proxy & tunnel daemon (~/.cpolar/cpolar.yml / /etc/cpolar/cpolar.yml / /etc/systemd/system/cpolar.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "cpolar-proxy", visible_alias = "cpolar-tunnel", visible_alias = "cpolar-daemon")]
        cpolar: bool,
        /// Also unset tmate terminal sharing & SSH reverse tunnel daemon (~/.tmate.conf / /etc/tmate.conf / /etc/systemd/system/tmate.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tmate-proxy", visible_alias = "tmate-tunnel", visible_alias = "tmate-daemon")]
        tmate: bool,
        /// Also unset upterm terminal sharing & SSH reverse tunnel daemon (~/.config/upterm/config.yml / /etc/upterm/config.yml / /etc/systemd/system/upterm.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "upterm-proxy", visible_alias = "upterm-tunnel", visible_alias = "upterm-daemon")]
        upterm: bool,
        /// Also unset NATAPP localhost reverse proxy & tunnel daemon (~/.natapp/config.ini / /etc/natapp/config.ini / /etc/systemd/system/natapp.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "natapp-proxy", visible_alias = "natapp-tunnel", visible_alias = "natapp-daemon")]
        natapp: bool,
        /// Also unset lanproxy intranet penetration & reverse tunnel client (~/.lanproxy/config.properties / /etc/lanproxy/config.properties / /etc/systemd/system/lanproxy.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "lanproxy-proxy", visible_alias = "lanproxy-tunnel", visible_alias = "lanproxy-client")]
        lanproxy: bool,
        /// Also unset teleconsole terminal sharing & SSH reverse tunnel daemon (~/.teleconsole/config.yaml / /etc/teleconsole/config.yaml / /etc/systemd/system/teleconsole.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "teleconsole-proxy", visible_alias = "teleconsole-tunnel", visible_alias = "teleconsole-daemon")]
        teleconsole: bool,
        /// Also unset tunnelto localhost reverse proxy & public tunnel client (~/.tunnelto/config.toml / /etc/tunnelto/config.toml / /etc/systemd/system/tunnelto.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "tunnelto-proxy", visible_alias = "tunnelto-tunnel", visible_alias = "tunnelto-client")]
        tunnelto: bool,
        /// Also unset Webhook Relay localhost tunnel & webhook forwarding daemon (~/.relay/config.json / /etc/webhookrelay/config.json / /etc/systemd/system/webhookrelay.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "webhookrelay-proxy", visible_alias = "webhookrelay-tunnel", visible_alias = "webhookrelay-daemon", visible_alias = "relay")]
        webhookrelay: bool,
        /// Also unset jprq localhost reverse tunnel & port forwarding daemon (~/.jprq/config.json / /etc/jprq/config.json / /etc/systemd/system/jprq.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "jprq-proxy", visible_alias = "jprq-tunnel", visible_alias = "jprq-daemon")]
        jprq: bool,
        /// Also unset mysocketctl / Border0 localhost reverse tunnel & zero-trust client (~/.mysocket/config.json / /etc/mysocketctl/config.json / /etc/systemd/system/mysocketctl.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "mysocketctl-proxy", visible_alias = "mysocketctl-tunnel", visible_alias = "mysocket-proxy", visible_alias = "border0")]
        mysocketctl: bool,
        /// Also unset openp2p decentralized P2P reverse tunnel & port forwarding daemon (~/.openp2p/config.json / /etc/openp2p/config.json / /etc/systemd/system/openp2p.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "openp2p-proxy", visible_alias = "openp2p-tunnel", visible_alias = "openp2p-daemon")]
        openp2p: bool,
        /// Also unset rport remote management & reverse tunnel client daemon (/etc/rport/rport.conf / ~/.rport/rport.conf / /etc/systemd/system/rport.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "rport-proxy", visible_alias = "rport-tunnel", visible_alias = "rport-client", visible_alias = "rport-daemon")]
        rport: bool,
        /// Also unset mirrord Kubernetes dev traffic mirror & reverse proxy client daemon (~/.mirrord/mirrord.json / /etc/mirrord/mirrord.json / /etc/systemd/system/mirrord.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "mirrord-proxy", visible_alias = "mirrord-tunnel", visible_alias = "metalbear")]
        mirrord: bool,
        /// Also unset kt-connect Kubernetes dev reverse tunnel & SOCKS5 proxy daemon (~/.ktctl/config.json / /etc/ktctl/config.json / /etc/systemd/system/ktctl.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "kt-connect-proxy", visible_alias = "kt-connect-tunnel", visible_alias = "ktctl", visible_alias = "kt-connect-daemon")]
        kt_connect: bool,
        /// Also unset DevSpace Cloud-Native Kubernetes dev proxy & port forwarder daemon (~/.devspace/config.json / devspace.yaml / /etc/devspace/config.json / /etc/systemd/system/devspace.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "devspace-proxy", visible_alias = "devspace-tunnel", visible_alias = "loft")]
        devspace: bool,
        /// Also unset Geph censorship-resistant proxy & reverse tunnel client daemon (~/.config/geph4/geph4.json / /etc/geph4/geph4.json / /etc/systemd/system/geph4.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "geph-proxy", visible_alias = "geph-tunnel", visible_alias = "geph4", visible_alias = "geph5")]
        geph: bool,
        /// Also unset Okteto Cloud-Native Kubernetes dev reverse tunnel & SSH daemon (~/.okteto/config.json / okteto.yaml / /etc/okteto/config.json / /etc/systemd/system/okteto.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "okteto-proxy", visible_alias = "okteto-tunnel", visible_alias = "okteto-daemon")]
        okteto: bool,
        /// Also unset Skaffold Cloud-Native Kubernetes dev port forward & control daemon (~/.skaffold/config / skaffold.yaml / /etc/skaffold/config.yaml / /etc/systemd/system/skaffold.service.d/http-proxy.conf)
        #[arg(long, visible_alias = "skaffold-proxy", visible_alias = "skaffold-port-forward", visible_alias = "skaffold-daemon")]
        skaffold: bool,
        /// Also unset git, cargo, npm, yarn, pnpm, bun, pip, conda, docker, gradle, maven, vscode, gh, glab, jetbrains, nuget, gem, svn, hg, uv, go, bazel, composer, conan, sbt, kube, julia, nix, cabal, stack, hex, r, opam, lein, cpan, luarocks, nimble, rebar3, scoop, bundle, gcloud, aws, azure, mamba, podman, vagrant, choco, colima, minikube, spack, macports, apt, dnf, zypper, pacman, pkg, portage, docker daemon, flatpak, snap, ollama, containerd, buildkit, k3s, microk8s, k0s, rke2, nomad, kubelet, gitlab-runner, actions-runner, jenkins, buildkite-agent, woodpecker-agent, drone-runner, act-runner, circleci-runner, azure-pipelines-agent, teamcity-agent, bamboo-agent, tfc-agent, concourse-worker, tentacle-agent, puppet-agent, chef-client, salt-minion, datadog-agent, newrelic-infra, elastic-agent, grafana-alloy, otelcol, vector, fluent-bit, fluentd, telegraf, logstash, prometheus, alertmanager, promtail, vmagent, filebeat, metricbeat, heartbeat, auditbeat, packetbeat, winlogbeat, consul, vault, traefik, caddy, envoy, haproxy, nginx, kong, apisix, squid, privoxy, tinyproxy, mitmproxy, tor, cloudflared, ngrok, frpc, tailscale, zerotier, gost, rathole, chisel, nebula, bore, brook, npc, shadowsocks, v2ray, sing-box, hysteria, trojan, naive, tuic, juicity, snell, leaf, mihomo, stunnel, shadow-tls, tun2socks, wstunnel, cloak, wireproxy, mtg, kcptun, glider, redsocks, dante, 3proxy, polipo, pproxy, microsocks, cntlm, cow, sniproxy, dnsproxy, mosdns, smartdns, coredns, adguard, dnscrypt, unbound, dnsmasq, pihole, blocky, stubby, kresd, pdns, dnsdist, bind, nextdns, pdnsd, maradns, i2pd, dae, yggdrasil, cjdns, tinc, openvpn, softether, openconnect, strongswan, wireguard, headscale, netbird, warp, amnezia, openziti, firezone, netmaker, pritunl, twingate, easytier, teleport, pomerium, innernet, defguard, inlets, zrok, n2n, hyprspace, fastd, edgevpn, boringproxy, ghostunnel, freelan, omniedge, lantern, tunsafe, sshuttle, psiphon, localtunnel, pagekite, toxiproxy, telepresence, localxpose, packetriot, sish, pinggy, loophole, tunnelmole, expose, playit, pgrok, cpolar, tmate, upterm, natapp, lanproxy, teleconsole, tunnelto, webhookrelay, jprq, mysocketctl, openp2p, rport, mirrord, kt-connect, devspace, geph, okteto, and skaffold proxy
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
            mihomo,
            stunnel,
            shadow_tls,
            tun2socks,
            wstunnel,
            cloak,
            wireproxy,
            mtg,
            kcptun,
            glider,
            redsocks,
            dante,
            threeproxy,
            polipo,
            pproxy,
            microsocks,
            cntlm,
            cow,
            sniproxy,
            dnsproxy,
            mosdns,
            smartdns,
            coredns,
            adguard,
            dnscrypt,
            unbound,
            dnsmasq,
            pihole,
            blocky,
            stubby,
            kresd,
            pdns,
            dnsdist,
            bind,
            nextdns,
            pdnsd,
            maradns,
            i2pd,
            dae,
            yggdrasil,
            cjdns,
            tinc,
            openvpn,
            softether,
            openconnect,
            strongswan,
            wireguard,
            headscale,
            netbird,
            warp,
            amnezia,
            openziti,
            firezone,
            netmaker,
            pritunl,
            twingate,
            easytier,
            teleport,
            pomerium,
            innernet,
            defguard,
            inlets,
            zrok,
            n2n,
            hyprspace,
            fastd,
            edgevpn,
            boringproxy,
            ghostunnel,
            freelan,
            omniedge,
            lantern,
            tunsafe,
            sshuttle,
            psiphon,
            localtunnel,
            pagekite,
            toxiproxy,
            telepresence,
            localxpose,
            packetriot,
            sish,
            pinggy,
            loophole,
            tunnelmole,
            expose,
            playit,
            pgrok,
            cpolar,
            tmate,
            upterm,
            natapp,
            lanproxy,
            teleconsole,
            tunnelto,
            webhookrelay,
            jprq,
            mysocketctl,
            openp2p,
            rport,
            mirrord,
            kt_connect,
            devspace,
            geph,
            okteto,
            skaffold,
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
                mihomo,
                stunnel,
                shadow_tls,
                tun2socks,
                wstunnel,
                cloak,
                wireproxy,
                mtg,
                kcptun,
                glider,
                redsocks,
                dante,
                threeproxy,
                polipo,
                pproxy,
                microsocks,
                cntlm,
                cow,
                sniproxy,
                dnsproxy,
                mosdns,
                smartdns,
                coredns,
                adguard,
                dnscrypt,
                unbound,
                dnsmasq,
                pihole,
                blocky,
                stubby,
                kresd,
                pdns,
                dnsdist,
                bind,
                nextdns,
                pdnsd,
                maradns,
                i2pd,
                dae,
                yggdrasil,
                cjdns,
                tinc,
                openvpn,
                softether,
                openconnect,
                strongswan,
                wireguard,
                headscale,
                netbird,
                warp,
                amnezia,
                openziti,
                firezone,
                netmaker,
                pritunl,
                twingate,
                easytier,
                teleport,
                pomerium,
                innernet,
                defguard,
                inlets,
                zrok,
                n2n,
                hyprspace,
                fastd,
                edgevpn,
                boringproxy,
                ghostunnel,
                freelan,
                omniedge,
                lantern,
                tunsafe,
                sshuttle,
                psiphon,
                localtunnel,
                pagekite,
                toxiproxy,
                telepresence,
                localxpose,
                packetriot,
                sish,
                pinggy,
                loophole,
                tunnelmole,
                expose,
                playit,
                pgrok,
                cpolar,
                tmate,
                upterm,
                natapp,
                lanproxy,
                teleconsole,
                tunnelto,
                webhookrelay,
                jprq,
                mysocketctl,
                openp2p,
                rport,
                mirrord,
                kt_connect,
                devspace,
                geph,
                okteto,
                skaffold,
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
            mihomo,
            stunnel,
            shadow_tls,
            tun2socks,
            wstunnel,
            cloak,
            wireproxy,
            mtg,
            kcptun,
            glider,
            redsocks,
            dante,
            threeproxy,
            polipo,
            pproxy,
            microsocks,
            cntlm,
            cow,
            sniproxy,
            dnsproxy,
            mosdns,
            smartdns,
            coredns,
            adguard,
            dnscrypt,
            unbound,
            dnsmasq,
            pihole,
            blocky,
            stubby,
            kresd,
            pdns,
            dnsdist,
            bind,
            nextdns,
            pdnsd,
            maradns,
            i2pd,
            dae,
            yggdrasil,
            cjdns,
            tinc,
            openvpn,
            softether,
            openconnect,
            strongswan,
            wireguard,
            headscale,
            netbird,
            warp,
            amnezia,
            openziti,
            firezone,
            netmaker,
            pritunl,
            twingate,
            easytier,
            teleport,
            pomerium,
            innernet,
            defguard,
            inlets,
            zrok,
            n2n,
            hyprspace,
            fastd,
            edgevpn,
            boringproxy,
            ghostunnel,
            freelan,
            omniedge,
            lantern,
            tunsafe,
            sshuttle,
            psiphon,
            localtunnel,
            pagekite,
            toxiproxy,
            telepresence,
            localxpose,
            packetriot,
            sish,
            pinggy,
            loophole,
            tunnelmole,
            expose,
            playit,
            pgrok,
            cpolar,
            tmate,
            upterm,
            natapp,
            lanproxy,
            teleconsole,
            tunnelto,
            webhookrelay,
            jprq,
            mysocketctl,
            openp2p,
            rport,
            mirrord,
            kt_connect,
            devspace,
            geph,
            okteto,
            skaffold,
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
                mihomo,
                stunnel,
                shadow_tls,
                tun2socks,
                wstunnel,
                cloak,
                wireproxy,
                mtg,
                kcptun,
                glider,
                redsocks,
                dante,
                threeproxy,
                polipo,
                pproxy,
                microsocks,
                cntlm,
                cow,
                sniproxy,
                dnsproxy,
                mosdns,
                smartdns,
                coredns,
                adguard,
                dnscrypt,
                unbound,
                dnsmasq,
                pihole,
                blocky,
                stubby,
                kresd,
                pdns,
                dnsdist,
                bind,
                nextdns,
                pdnsd,
                maradns,
                i2pd,
                dae,
                yggdrasil,
                cjdns,
                tinc,
                openvpn,
                softether,
                openconnect,
                strongswan,
                wireguard,
                headscale,
                netbird,
                warp,
                amnezia,
                openziti,
                firezone,
                netmaker,
                pritunl,
                twingate,
                easytier,
                teleport,
                pomerium,
                innernet,
                defguard,
                inlets,
                zrok,
                n2n,
                hyprspace,
                fastd,
                edgevpn,
                boringproxy,
                ghostunnel,
                freelan,
                omniedge,
                lantern,
                tunsafe,
                sshuttle,
                psiphon,
                localtunnel,
                pagekite,
                toxiproxy,
                telepresence,
                localxpose,
                packetriot,
                sish,
                pinggy,
                loophole,
                tunnelmole,
                expose,
                playit,
                pgrok,
                cpolar,
                tmate,
                upterm,
                natapp,
                lanproxy,
                teleconsole,
                tunnelto,
                webhookrelay,
                jprq,
                mysocketctl,
                openp2p,
                rport,
                mirrord,
                kt_connect,
                devspace,
                geph,
                okteto,
                skaffold,
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

    #[test]
    fn test_cli_parse_on_mihomo() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--mihomo"]).unwrap();
        match cli.command {
            Some(Commands::On { mihomo, .. }) => {
                assert!(mihomo);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--clash-meta"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { mihomo, .. }) => {
                assert!(mihomo);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_mihomo() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--mihomo"]).unwrap();
        match cli.command {
            Some(Commands::Off { mihomo, .. }) => {
                assert!(mihomo);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_stunnel() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--stunnel"]).unwrap();
        match cli.command {
            Some(Commands::On { stunnel, .. }) => {
                assert!(stunnel);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--stunnel4"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { stunnel, .. }) => {
                assert!(stunnel);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_stunnel() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--stunnel"]).unwrap();
        match cli.command {
            Some(Commands::Off { stunnel, .. }) => {
                assert!(stunnel);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_shadow_tls() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--shadow-tls"]).unwrap();
        match cli.command {
            Some(Commands::On { shadow_tls, .. }) => {
                assert!(shadow_tls);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--shadowtls"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { shadow_tls, .. }) => {
                assert!(shadow_tls);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_shadow_tls() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--shadow-tls"]).unwrap();
        match cli.command {
            Some(Commands::Off { shadow_tls, .. }) => {
                assert!(shadow_tls);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_tun2socks() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--tun2socks"]).unwrap();
        match cli.command {
            Some(Commands::On { tun2socks, .. }) => {
                assert!(tun2socks);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--tun2socks-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { tun2socks, .. }) => {
                assert!(tun2socks);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_tun2socks() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--tun2socks"]).unwrap();
        match cli.command {
            Some(Commands::Off { tun2socks, .. }) => {
                assert!(tun2socks);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_wstunnel() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--wstunnel"]).unwrap();
        match cli.command {
            Some(Commands::On { wstunnel, .. }) => {
                assert!(wstunnel);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--wstunnel-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { wstunnel, .. }) => {
                assert!(wstunnel);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_wstunnel() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--wstunnel"]).unwrap();
        match cli.command {
            Some(Commands::Off { wstunnel, .. }) => {
                assert!(wstunnel);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_cloak() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--cloak"]).unwrap();
        match cli.command {
            Some(Commands::On { cloak, .. }) => {
                assert!(cloak);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--ck-client"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { cloak, .. }) => {
                assert!(cloak);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_cloak() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--cloak"]).unwrap();
        match cli.command {
            Some(Commands::Off { cloak, .. }) => {
                assert!(cloak);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_wireproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--wireproxy"]).unwrap();
        match cli.command {
            Some(Commands::On { wireproxy, .. }) => {
                assert!(wireproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--wireproxy-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { wireproxy, .. }) => {
                assert!(wireproxy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_wireproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--wireproxy"]).unwrap();
        match cli.command {
            Some(Commands::Off { wireproxy, .. }) => {
                assert!(wireproxy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_mtg() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--mtg"]).unwrap();
        match cli.command {
            Some(Commands::On { mtg, .. }) => {
                assert!(mtg);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--mtg-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { mtg, .. }) => {
                assert!(mtg);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_mtg() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--mtg"]).unwrap();
        match cli.command {
            Some(Commands::Off { mtg, .. }) => {
                assert!(mtg);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_kcptun() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--kcptun"]).unwrap();
        match cli.command {
            Some(Commands::On { kcptun, .. }) => {
                assert!(kcptun);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--kcptun-client"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { kcptun, .. }) => {
                assert!(kcptun);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_kcptun() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--kcptun"]).unwrap();
        match cli.command {
            Some(Commands::Off { kcptun, .. }) => {
                assert!(kcptun);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_glider() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--glider"]).unwrap();
        match cli.command {
            Some(Commands::On { glider, .. }) => {
                assert!(glider);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--glider-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { glider, .. }) => {
                assert!(glider);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_glider() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--glider"]).unwrap();
        match cli.command {
            Some(Commands::Off { glider, .. }) => {
                assert!(glider);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_redsocks() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--redsocks"]).unwrap();
        match cli.command {
            Some(Commands::On { redsocks, .. }) => {
                assert!(redsocks);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--redsocks-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { redsocks, .. }) => {
                assert!(redsocks);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_redsocks() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--redsocks"]).unwrap();
        match cli.command {
            Some(Commands::Off { redsocks, .. }) => {
                assert!(redsocks);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_dante() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--dante"]).unwrap();
        match cli.command {
            Some(Commands::On { dante, .. }) => {
                assert!(dante);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--danted"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { dante, .. }) => {
                assert!(dante);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_dante() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--dante"]).unwrap();
        match cli.command {
            Some(Commands::Off { dante, .. }) => {
                assert!(dante);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_threeproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--threeproxy"]).unwrap();
        match cli.command {
            Some(Commands::On { threeproxy, .. }) => {
                assert!(threeproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--3proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { threeproxy, .. }) => {
                assert!(threeproxy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_threeproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--threeproxy"]).unwrap();
        match cli.command {
            Some(Commands::Off { threeproxy, .. }) => {
                assert!(threeproxy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_polipo() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--polipo"]).unwrap();
        match cli.command {
            Some(Commands::On { polipo, .. }) => {
                assert!(polipo);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--polipo-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { polipo, .. }) => {
                assert!(polipo);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_polipo() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--polipo"]).unwrap();
        match cli.command {
            Some(Commands::Off { polipo, .. }) => {
                assert!(polipo);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pproxy"]).unwrap();
        match cli.command {
            Some(Commands::On { pproxy, .. }) => {
                assert!(pproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--python-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { pproxy, .. }) => {
                assert!(pproxy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pproxy"]).unwrap();
        match cli.command {
            Some(Commands::Off { pproxy, .. }) => {
                assert!(pproxy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_microsocks() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--microsocks"]).unwrap();
        match cli.command {
            Some(Commands::On { microsocks, .. }) => {
                assert!(microsocks);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--microsocks-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { microsocks, .. }) => {
                assert!(microsocks);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_microsocks() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--microsocks"]).unwrap();
        match cli.command {
            Some(Commands::Off { microsocks, .. }) => {
                assert!(microsocks);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_cntlm() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--cntlm"]).unwrap();
        match cli.command {
            Some(Commands::On { cntlm, .. }) => {
                assert!(cntlm);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--cntlm-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { cntlm, .. }) => {
                assert!(cntlm);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_cntlm() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--cntlm"]).unwrap();
        match cli.command {
            Some(Commands::Off { cntlm, .. }) => {
                assert!(cntlm);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_cow() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--cow"]).unwrap();
        match cli.command {
            Some(Commands::On { cow, .. }) => {
                assert!(cow);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--cow-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { cow, .. }) => {
                assert!(cow);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_cow() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--cow"]).unwrap();
        match cli.command {
            Some(Commands::Off { cow, .. }) => {
                assert!(cow);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_sniproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--sniproxy"]).unwrap();
        match cli.command {
            Some(Commands::On { sniproxy, .. }) => {
                assert!(sniproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--sniproxy-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { sniproxy, .. }) => {
                assert!(sniproxy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_sniproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--sniproxy"]).unwrap();
        match cli.command {
            Some(Commands::Off { sniproxy, .. }) => {
                assert!(sniproxy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_dnsproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--dnsproxy"]).unwrap();
        match cli.command {
            Some(Commands::On { dnsproxy, .. }) => {
                assert!(dnsproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--dnsproxy-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { dnsproxy, .. }) => {
                assert!(dnsproxy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_dnsproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--dnsproxy"]).unwrap();
        match cli.command {
            Some(Commands::Off { dnsproxy, .. }) => {
                assert!(dnsproxy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_mosdns() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--mosdns"]).unwrap();
        match cli.command {
            Some(Commands::On { mosdns, .. }) => {
                assert!(mosdns);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--mosdns-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { mosdns, .. }) => {
                assert!(mosdns);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_mosdns() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--mosdns"]).unwrap();
        match cli.command {
            Some(Commands::Off { mosdns, .. }) => {
                assert!(mosdns);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_smartdns() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--smartdns"]).unwrap();
        match cli.command {
            Some(Commands::On { smartdns, .. }) => {
                assert!(smartdns);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--smartdns-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { smartdns, .. }) => {
                assert!(smartdns);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_smartdns() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--smartdns"]).unwrap();
        match cli.command {
            Some(Commands::Off { smartdns, .. }) => {
                assert!(smartdns);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_coredns() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--coredns"]).unwrap();
        match cli.command {
            Some(Commands::On { coredns, .. }) => {
                assert!(coredns);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--coredns-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { coredns, .. }) => {
                assert!(coredns);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_coredns() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--coredns"]).unwrap();
        match cli.command {
            Some(Commands::Off { coredns, .. }) => {
                assert!(coredns);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_adguard() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--adguard"]).unwrap();
        match cli.command {
            Some(Commands::On { adguard, .. }) => {
                assert!(adguard);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--adguardhome"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { adguard, .. }) => {
                assert!(adguard);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_adguard() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--adguard"]).unwrap();
        match cli.command {
            Some(Commands::Off { adguard, .. }) => {
                assert!(adguard);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_dnscrypt() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--dnscrypt"]).unwrap();
        match cli.command {
            Some(Commands::On { dnscrypt, .. }) => {
                assert!(dnscrypt);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--dnscrypt-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { dnscrypt, .. }) => {
                assert!(dnscrypt);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--dnscrypt-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { dnscrypt, .. }) => {
                assert!(dnscrypt);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_dnscrypt() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--dnscrypt"]).unwrap();
        match cli.command {
            Some(Commands::Off { dnscrypt, .. }) => {
                assert!(dnscrypt);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_unbound() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--unbound"]).unwrap();
        match cli.command {
            Some(Commands::On { unbound, .. }) => {
                assert!(unbound);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--unbound-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { unbound, .. }) => {
                assert!(unbound);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--unbound-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { unbound, .. }) => {
                assert!(unbound);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_unbound() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--unbound"]).unwrap();
        match cli.command {
            Some(Commands::Off { unbound, .. }) => {
                assert!(unbound);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_dnsmasq() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--dnsmasq"]).unwrap();
        match cli.command {
            Some(Commands::On { dnsmasq, .. }) => {
                assert!(dnsmasq);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--dnsmasq-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { dnsmasq, .. }) => {
                assert!(dnsmasq);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--dnsmasq-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { dnsmasq, .. }) => {
                assert!(dnsmasq);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_dnsmasq() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--dnsmasq"]).unwrap();
        match cli.command {
            Some(Commands::Off { dnsmasq, .. }) => {
                assert!(dnsmasq);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pihole() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pihole"]).unwrap();
        match cli.command {
            Some(Commands::On { pihole, .. }) => {
                assert!(pihole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--pihole-ftl"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { pihole, .. }) => {
                assert!(pihole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--pihole-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { pihole, .. }) => {
                assert!(pihole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--pi-hole"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { pihole, .. }) => {
                assert!(pihole);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pihole() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pihole"]).unwrap();
        match cli.command {
            Some(Commands::Off { pihole, .. }) => {
                assert!(pihole);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_blocky() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--blocky"]).unwrap();
        match cli.command {
            Some(Commands::On { blocky, .. }) => {
                assert!(blocky);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--blocky-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { blocky, .. }) => {
                assert!(blocky);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--blocky-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { blocky, .. }) => {
                assert!(blocky);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_blocky() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--blocky"]).unwrap();
        match cli.command {
            Some(Commands::Off { blocky, .. }) => {
                assert!(blocky);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_stubby() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--stubby"]).unwrap();
        match cli.command {
            Some(Commands::On { stubby, .. }) => {
                assert!(stubby);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--stubby-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { stubby, .. }) => {
                assert!(stubby);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--stubby-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { stubby, .. }) => {
                assert!(stubby);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_stubby() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--stubby"]).unwrap();
        match cli.command {
            Some(Commands::Off { stubby, .. }) => {
                assert!(stubby);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_kresd() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--kresd"]).unwrap();
        match cli.command {
            Some(Commands::On { kresd, .. }) => {
                assert!(kresd);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--kresd-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { kresd, .. }) => {
                assert!(kresd);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--knot-resolver"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { kresd, .. }) => {
                assert!(kresd);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_kresd() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--kresd"]).unwrap();
        match cli.command {
            Some(Commands::Off { kresd, .. }) => {
                assert!(kresd);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pdns() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pdns"]).unwrap();
        match cli.command {
            Some(Commands::On { pdns, .. }) => {
                assert!(pdns);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--pdns-recursor"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { pdns, .. }) => {
                assert!(pdns);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--powerdns"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { pdns, .. }) => {
                assert!(pdns);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pdns() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pdns"]).unwrap();
        match cli.command {
            Some(Commands::Off { pdns, .. }) => {
                assert!(pdns);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_dnsdist() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--dnsdist"]).unwrap();
        match cli.command {
            Some(Commands::On { dnsdist, .. }) => {
                assert!(dnsdist);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--dnsdist-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { dnsdist, .. }) => {
                assert!(dnsdist);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--dnsdist-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { dnsdist, .. }) => {
                assert!(dnsdist);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_dnsdist() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--dnsdist"]).unwrap();
        match cli.command {
            Some(Commands::Off { dnsdist, .. }) => {
                assert!(dnsdist);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_bind() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--bind"]).unwrap();
        match cli.command {
            Some(Commands::On { bind, .. }) => {
                assert!(bind);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--bind9"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { bind, .. }) => {
                assert!(bind);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--named"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { bind, .. }) => {
                assert!(bind);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_bind() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--bind"]).unwrap();
        match cli.command {
            Some(Commands::Off { bind, .. }) => {
                assert!(bind);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_nextdns() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--nextdns"]).unwrap();
        match cli.command {
            Some(Commands::On { nextdns, .. }) => {
                assert!(nextdns);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--nextdns-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { nextdns, .. }) => {
                assert!(nextdns);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--nextdns-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { nextdns, .. }) => {
                assert!(nextdns);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_nextdns() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--nextdns"]).unwrap();
        match cli.command {
            Some(Commands::Off { nextdns, .. }) => {
                assert!(nextdns);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pdnsd() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pdnsd"]).unwrap();
        match cli.command {
            Some(Commands::On { pdnsd, .. }) => {
                assert!(pdnsd);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--pdnsd-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { pdnsd, .. }) => {
                assert!(pdnsd);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--pdnsd-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { pdnsd, .. }) => {
                assert!(pdnsd);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pdnsd() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pdnsd"]).unwrap();
        match cli.command {
            Some(Commands::Off { pdnsd, .. }) => {
                assert!(pdnsd);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_maradns() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--maradns"]).unwrap();
        match cli.command {
            Some(Commands::On { maradns, .. }) => {
                assert!(maradns);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--deadwood"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { maradns, .. }) => {
                assert!(maradns);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--maradns-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { maradns, .. }) => {
                assert!(maradns);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_maradns() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--maradns"]).unwrap();
        match cli.command {
            Some(Commands::Off { maradns, .. }) => {
                assert!(maradns);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_i2pd() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--i2pd"]).unwrap();
        match cli.command {
            Some(Commands::On { i2pd, .. }) => {
                assert!(i2pd);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--i2p"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { i2pd, .. }) => {
                assert!(i2pd);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--i2pd-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { i2pd, .. }) => {
                assert!(i2pd);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_i2pd() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--i2pd"]).unwrap();
        match cli.command {
            Some(Commands::Off { i2pd, .. }) => {
                assert!(i2pd);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_dae() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--dae"]).unwrap();
        match cli.command {
            Some(Commands::On { dae, .. }) => {
                assert!(dae);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--daed"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { dae, .. }) => {
                assert!(dae);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--dae-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { dae, .. }) => {
                assert!(dae);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_dae() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--dae"]).unwrap();
        match cli.command {
            Some(Commands::Off { dae, .. }) => {
                assert!(dae);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_yggdrasil() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--yggdrasil"]).unwrap();
        match cli.command {
            Some(Commands::On { yggdrasil, .. }) => {
                assert!(yggdrasil);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--ygg"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { yggdrasil, .. }) => {
                assert!(yggdrasil);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--yggdrasil-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { yggdrasil, .. }) => {
                assert!(yggdrasil);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_yggdrasil() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--yggdrasil"]).unwrap();
        match cli.command {
            Some(Commands::Off { yggdrasil, .. }) => {
                assert!(yggdrasil);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_cjdns() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--cjdns"]).unwrap();
        match cli.command {
            Some(Commands::On { cjdns, .. }) => {
                assert!(cjdns);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--cjdroute"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { cjdns, .. }) => {
                assert!(cjdns);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--cjdns-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { cjdns, .. }) => {
                assert!(cjdns);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_cjdns() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--cjdns"]).unwrap();
        match cli.command {
            Some(Commands::Off { cjdns, .. }) => {
                assert!(cjdns);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_tinc() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--tinc"]).unwrap();
        match cli.command {
            Some(Commands::On { tinc, .. }) => {
                assert!(tinc);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--tinc-vpn"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { tinc, .. }) => {
                assert!(tinc);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--tinc-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { tinc, .. }) => {
                assert!(tinc);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_tinc() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--tinc"]).unwrap();
        match cli.command {
            Some(Commands::Off { tinc, .. }) => {
                assert!(tinc);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_openvpn() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--openvpn"]).unwrap();
        match cli.command {
            Some(Commands::On { openvpn, .. }) => {
                assert!(openvpn);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--ovpn"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { openvpn, .. }) => {
                assert!(openvpn);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--openvpn-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { openvpn, .. }) => {
                assert!(openvpn);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_openvpn() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--openvpn"]).unwrap();
        match cli.command {
            Some(Commands::Off { openvpn, .. }) => {
                assert!(openvpn);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_softether() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--softether"]).unwrap();
        match cli.command {
            Some(Commands::On { softether, .. }) => {
                assert!(softether);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--vpnclient"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { softether, .. }) => {
                assert!(softether);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--softether-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { softether, .. }) => {
                assert!(softether);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_softether() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--softether"]).unwrap();
        match cli.command {
            Some(Commands::Off { softether, .. }) => {
                assert!(softether);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_openconnect() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--openconnect"]).unwrap();
        match cli.command {
            Some(Commands::On { openconnect, .. }) => {
                assert!(openconnect);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--anyconnect"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { openconnect, .. }) => {
                assert!(openconnect);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--openconnect-daemon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { openconnect, .. }) => {
                assert!(openconnect);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_openconnect() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--openconnect"]).unwrap();
        match cli.command {
            Some(Commands::Off { openconnect, .. }) => {
                assert!(openconnect);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_strongswan() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--strongswan"]).unwrap();
        match cli.command {
            Some(Commands::On { strongswan, .. }) => {
                assert!(strongswan);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--ipsec"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { strongswan, .. }) => {
                assert!(strongswan);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--charon"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { strongswan, .. }) => {
                assert!(strongswan);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_strongswan() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--strongswan"]).unwrap();
        match cli.command {
            Some(Commands::Off { strongswan, .. }) => {
                assert!(strongswan);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_wireguard() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--wireguard"]).unwrap();
        match cli.command {
            Some(Commands::On { wireguard, .. }) => {
                assert!(wireguard);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--wg"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { wireguard, .. }) => {
                assert!(wireguard);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--wg-quick"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { wireguard, .. }) => {
                assert!(wireguard);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_wireguard() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--wireguard"]).unwrap();
        match cli.command {
            Some(Commands::Off { wireguard, .. }) => {
                assert!(wireguard);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_headscale() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--headscale"]).unwrap();
        match cli.command {
            Some(Commands::On { headscale, .. }) => {
                assert!(headscale);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--headscale-server"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { headscale, .. }) => {
                assert!(headscale);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--headscale-proxy"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { headscale, .. }) => {
                assert!(headscale);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_headscale() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--headscale"]).unwrap();
        match cli.command {
            Some(Commands::Off { headscale, .. }) => {
                assert!(headscale);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_netbird() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--netbird"]).unwrap();
        match cli.command {
            Some(Commands::On { netbird, .. }) => {
                assert!(netbird);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--netbird-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { netbird, .. }) => {
                assert!(netbird);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--netbird-vpn"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { netbird, .. }) => {
                assert!(netbird);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_netbird() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--netbird"]).unwrap();
        match cli.command {
            Some(Commands::Off { netbird, .. }) => {
                assert!(netbird);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_warp() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--warp"]).unwrap();
        match cli.command {
            Some(Commands::On { warp, .. }) => {
                assert!(warp);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--warp-cli"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { warp, .. }) => {
                assert!(warp);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--cloudflare-warp"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { warp, .. }) => {
                assert!(warp);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_warp() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--warp"]).unwrap();
        match cli.command {
            Some(Commands::Off { warp, .. }) => {
                assert!(warp);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_amnezia() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--amnezia"]).unwrap();
        match cli.command {
            Some(Commands::On { amnezia, .. }) => {
                assert!(amnezia);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--amnezia-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { amnezia, .. }) => {
                assert!(amnezia);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--amneziawg"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { amnezia, .. }) => {
                assert!(amnezia);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_amnezia() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--amnezia"]).unwrap();
        match cli.command {
            Some(Commands::Off { amnezia, .. }) => {
                assert!(amnezia);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_openziti() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--openziti"]).unwrap();
        match cli.command {
            Some(Commands::On { openziti, .. }) => {
                assert!(openziti);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--ziti"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { openziti, .. }) => {
                assert!(openziti);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--ziti-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { openziti, .. }) => {
                assert!(openziti);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_openziti() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--openziti"]).unwrap();
        match cli.command {
            Some(Commands::Off { openziti, .. }) => {
                assert!(openziti);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_firezone() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--firezone"]).unwrap();
        match cli.command {
            Some(Commands::On { firezone, .. }) => {
                assert!(firezone);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--firezone-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { firezone, .. }) => {
                assert!(firezone);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--firezone-client"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { firezone, .. }) => {
                assert!(firezone);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--firezoned"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { firezone, .. }) => {
                assert!(firezone);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_firezone() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--firezone"]).unwrap();
        match cli.command {
            Some(Commands::Off { firezone, .. }) => {
                assert!(firezone);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_netmaker() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--netmaker"]).unwrap();
        match cli.command {
            Some(Commands::On { netmaker, .. }) => {
                assert!(netmaker);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--netmaker-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { netmaker, .. }) => {
                assert!(netmaker);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--netclient"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { netmaker, .. }) => {
                assert!(netmaker);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_netmaker() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--netmaker"]).unwrap();
        match cli.command {
            Some(Commands::Off { netmaker, .. }) => {
                assert!(netmaker);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pritunl() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pritunl"]).unwrap();
        match cli.command {
            Some(Commands::On { pritunl, .. }) => {
                assert!(pritunl);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--pritunl-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { pritunl, .. }) => {
                assert!(pritunl);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--pritunl-client"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { pritunl, .. }) => {
                assert!(pritunl);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pritunl() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pritunl"]).unwrap();
        match cli.command {
            Some(Commands::Off { pritunl, .. }) => {
                assert!(pritunl);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_twingate() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--twingate"]).unwrap();
        match cli.command {
            Some(Commands::On { twingate, .. }) => {
                assert!(twingate);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--twingate-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { twingate, .. }) => {
                assert!(twingate);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--twingate-connector"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { twingate, .. }) => {
                assert!(twingate);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_twingate() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--twingate"]).unwrap();
        match cli.command {
            Some(Commands::Off { twingate, .. }) => {
                assert!(twingate);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_easytier() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--easytier"]).unwrap();
        match cli.command {
            Some(Commands::On { easytier, .. }) => {
                assert!(easytier);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--easytier-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { easytier, .. }) => {
                assert!(easytier);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--easytier-vpn"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { easytier, .. }) => {
                assert!(easytier);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_easytier() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--easytier"]).unwrap();
        match cli.command {
            Some(Commands::Off { easytier, .. }) => {
                assert!(easytier);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_teleport() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--teleport"]).unwrap();
        match cli.command {
            Some(Commands::On { teleport, .. }) => {
                assert!(teleport);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--teleport-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { teleport, .. }) => {
                assert!(teleport);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--teleport-client"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { teleport, .. }) => {
                assert!(teleport);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_teleport() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--teleport"]).unwrap();
        match cli.command {
            Some(Commands::Off { teleport, .. }) => {
                assert!(teleport);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pomerium() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pomerium"]).unwrap();
        match cli.command {
            Some(Commands::On { pomerium, .. }) => {
                assert!(pomerium);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--pomerium-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { pomerium, .. }) => {
                assert!(pomerium);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--pomerium-ztna"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { pomerium, .. }) => {
                assert!(pomerium);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pomerium() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pomerium"]).unwrap();
        match cli.command {
            Some(Commands::Off { pomerium, .. }) => {
                assert!(pomerium);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_innernet() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--innernet"]).unwrap();
        match cli.command {
            Some(Commands::On { innernet, .. }) => {
                assert!(innernet);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--innernet-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { innernet, .. }) => {
                assert!(innernet);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--innernet-mesh"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { innernet, .. }) => {
                assert!(innernet);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_innernet() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--innernet"]).unwrap();
        match cli.command {
            Some(Commands::Off { innernet, .. }) => {
                assert!(innernet);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_defguard() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--defguard"]).unwrap();
        match cli.command {
            Some(Commands::On { defguard, .. }) => {
                assert!(defguard);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--defguard-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { defguard, .. }) => {
                assert!(defguard);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--defguard-vpn"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { defguard, .. }) => {
                assert!(defguard);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_defguard() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--defguard"]).unwrap();
        match cli.command {
            Some(Commands::Off { defguard, .. }) => {
                assert!(defguard);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_inlets() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--inlets"]).unwrap();
        match cli.command {
            Some(Commands::On { inlets, .. }) => {
                assert!(inlets);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--inlets-tunnel"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { inlets, .. }) => {
                assert!(inlets);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--inlets-pro"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { inlets, .. }) => {
                assert!(inlets);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_inlets() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--inlets"]).unwrap();
        match cli.command {
            Some(Commands::Off { inlets, .. }) => {
                assert!(inlets);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_zrok() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--zrok"]).unwrap();
        match cli.command {
            Some(Commands::On { zrok, .. }) => {
                assert!(zrok);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--zrok-tunnel"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { zrok, .. }) => {
                assert!(zrok);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--zrok-share"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { zrok, .. }) => {
                assert!(zrok);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_zrok() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--zrok"]).unwrap();
        match cli.command {
            Some(Commands::Off { zrok, .. }) => {
                assert!(zrok);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_n2n() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--n2n"]).unwrap();
        match cli.command {
            Some(Commands::On { n2n, .. }) => {
                assert!(n2n);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--n2n-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { n2n, .. }) => {
                assert!(n2n);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--n2n-vpn"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { n2n, .. }) => {
                assert!(n2n);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_n2n() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--n2n"]).unwrap();
        match cli.command {
            Some(Commands::Off { n2n, .. }) => {
                assert!(n2n);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_hyprspace() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--hyprspace"]).unwrap();
        match cli.command {
            Some(Commands::On { hyprspace, .. }) => {
                assert!(hyprspace);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--hyprspace-vpn"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { hyprspace, .. }) => {
                assert!(hyprspace);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--hyprspace-mesh"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { hyprspace, .. }) => {
                assert!(hyprspace);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_hyprspace() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--hyprspace"]).unwrap();
        match cli.command {
            Some(Commands::Off { hyprspace, .. }) => {
                assert!(hyprspace);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_fastd() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--fastd"]).unwrap();
        match cli.command {
            Some(Commands::On { fastd, .. }) => {
                assert!(fastd);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--fastd-vpn"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { fastd, .. }) => {
                assert!(fastd);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--fastd-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { fastd, .. }) => {
                assert!(fastd);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_fastd() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--fastd"]).unwrap();
        match cli.command {
            Some(Commands::Off { fastd, .. }) => {
                assert!(fastd);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_edgevpn() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--edgevpn"]).unwrap();
        match cli.command {
            Some(Commands::On { edgevpn, .. }) => {
                assert!(edgevpn);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--edgevpn-vpn"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { edgevpn, .. }) => {
                assert!(edgevpn);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--edgevpn-mesh"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { edgevpn, .. }) => {
                assert!(edgevpn);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_edgevpn() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--edgevpn"]).unwrap();
        match cli.command {
            Some(Commands::Off { edgevpn, .. }) => {
                assert!(edgevpn);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_boringproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--boringproxy"]).unwrap();
        match cli.command {
            Some(Commands::On { boringproxy, .. }) => {
                assert!(boringproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--boringproxy-tunnel"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { boringproxy, .. }) => {
                assert!(boringproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--boringproxy-client"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { boringproxy, .. }) => {
                assert!(boringproxy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_boringproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--boringproxy"]).unwrap();
        match cli.command {
            Some(Commands::Off { boringproxy, .. }) => {
                assert!(boringproxy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_ghostunnel() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--ghostunnel"]).unwrap();
        match cli.command {
            Some(Commands::On { ghostunnel, .. }) => {
                assert!(ghostunnel);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--ghostunnel-tls"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { ghostunnel, .. }) => {
                assert!(ghostunnel);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--ghostunnel-proxy"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { ghostunnel, .. }) => {
                assert!(ghostunnel);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_ghostunnel() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--ghostunnel"]).unwrap();
        match cli.command {
            Some(Commands::Off { ghostunnel, .. }) => {
                assert!(ghostunnel);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_freelan() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--freelan"]).unwrap();
        match cli.command {
            Some(Commands::On { freelan, .. }) => {
                assert!(freelan);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--freelan-vpn"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { freelan, .. }) => {
                assert!(freelan);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--freelan-p2p"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { freelan, .. }) => {
                assert!(freelan);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_freelan() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--freelan"]).unwrap();
        match cli.command {
            Some(Commands::Off { freelan, .. }) => {
                assert!(freelan);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_omniedge() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--omniedge"]).unwrap();
        match cli.command {
            Some(Commands::On { omniedge, .. }) => {
                assert!(omniedge);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--omniedge-vpn"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { omniedge, .. }) => {
                assert!(omniedge);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--omniedge-mesh"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { omniedge, .. }) => {
                assert!(omniedge);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_omniedge() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--omniedge"]).unwrap();
        match cli.command {
            Some(Commands::Off { omniedge, .. }) => {
                assert!(omniedge);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_lantern() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--lantern"]).unwrap();
        match cli.command {
            Some(Commands::On { lantern, .. }) => {
                assert!(lantern);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--lantern-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { lantern, .. }) => {
                assert!(lantern);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--lantern-vpn"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { lantern, .. }) => {
                assert!(lantern);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_lantern() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--lantern"]).unwrap();
        match cli.command {
            Some(Commands::Off { lantern, .. }) => {
                assert!(lantern);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_tunsafe() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--tunsafe"]).unwrap();
        match cli.command {
            Some(Commands::On { tunsafe, .. }) => {
                assert!(tunsafe);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--tunsafe-vpn"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { tunsafe, .. }) => {
                assert!(tunsafe);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--tunsafe-wg"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { tunsafe, .. }) => {
                assert!(tunsafe);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_tunsafe() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--tunsafe"]).unwrap();
        match cli.command {
            Some(Commands::Off { tunsafe, .. }) => {
                assert!(tunsafe);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_sshuttle() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--sshuttle"]).unwrap();
        match cli.command {
            Some(Commands::On { sshuttle, .. }) => {
                assert!(sshuttle);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--sshuttle-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { sshuttle, .. }) => {
                assert!(sshuttle);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--sshuttle-vpn"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { sshuttle, .. }) => {
                assert!(sshuttle);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_sshuttle() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--sshuttle"]).unwrap();
        match cli.command {
            Some(Commands::Off { sshuttle, .. }) => {
                assert!(sshuttle);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_psiphon() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--psiphon"]).unwrap();
        match cli.command {
            Some(Commands::On { psiphon, .. }) => {
                assert!(psiphon);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--psiphon-vpn"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { psiphon, .. }) => {
                assert!(psiphon);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--psiphon-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { psiphon, .. }) => {
                assert!(psiphon);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_psiphon() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--psiphon"]).unwrap();
        match cli.command {
            Some(Commands::Off { psiphon, .. }) => {
                assert!(psiphon);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_localtunnel() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--localtunnel"]).unwrap();
        match cli.command {
            Some(Commands::On { localtunnel, .. }) => {
                assert!(localtunnel);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--localtunnel-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { localtunnel, .. }) => {
                assert!(localtunnel);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--lt"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { localtunnel, .. }) => {
                assert!(localtunnel);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_localtunnel() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--localtunnel"]).unwrap();
        match cli.command {
            Some(Commands::Off { localtunnel, .. }) => {
                assert!(localtunnel);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pagekite() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pagekite"]).unwrap();
        match cli.command {
            Some(Commands::On { pagekite, .. }) => {
                assert!(pagekite);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--pagekite-tunnel"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { pagekite, .. }) => {
                assert!(pagekite);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--kite"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { pagekite, .. }) => {
                assert!(pagekite);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pagekite() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pagekite"]).unwrap();
        match cli.command {
            Some(Commands::Off { pagekite, .. }) => {
                assert!(pagekite);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_toxiproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--toxiproxy"]).unwrap();
        match cli.command {
            Some(Commands::On { toxiproxy, .. }) => {
                assert!(toxiproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--toxiproxy-daemon"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { toxiproxy, .. }) => {
                assert!(toxiproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--toxi"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { toxiproxy, .. }) => {
                assert!(toxiproxy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_toxiproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--toxiproxy"]).unwrap();
        match cli.command {
            Some(Commands::Off { toxiproxy, .. }) => {
                assert!(toxiproxy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_telepresence() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--telepresence"]).unwrap();
        match cli.command {
            Some(Commands::On { telepresence, .. }) => {
                assert!(telepresence);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--telepresence-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { telepresence, .. }) => {
                assert!(telepresence);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--tp"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { telepresence, .. }) => {
                assert!(telepresence);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_telepresence() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--telepresence"]).unwrap();
        match cli.command {
            Some(Commands::Off { telepresence, .. }) => {
                assert!(telepresence);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_localxpose() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--localxpose"]).unwrap();
        match cli.command {
            Some(Commands::On { localxpose, .. }) => {
                assert!(localxpose);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--localxpose-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { localxpose, .. }) => {
                assert!(localxpose);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--loclx"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { localxpose, .. }) => {
                assert!(localxpose);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_localxpose() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--localxpose"]).unwrap();
        match cli.command {
            Some(Commands::Off { localxpose, .. }) => {
                assert!(localxpose);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_packetriot() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--packetriot"]).unwrap();
        match cli.command {
            Some(Commands::On { packetriot, .. }) => {
                assert!(packetriot);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--packetriot-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { packetriot, .. }) => {
                assert!(packetriot);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--pktriot"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { packetriot, .. }) => {
                assert!(packetriot);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_packetriot() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--packetriot"]).unwrap();
        match cli.command {
            Some(Commands::Off { packetriot, .. }) => {
                assert!(packetriot);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_sish() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--sish"]).unwrap();
        match cli.command {
            Some(Commands::On { sish, .. }) => {
                assert!(sish);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--sish-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { sish, .. }) => {
                assert!(sish);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--sish-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { sish, .. }) => {
                assert!(sish);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_sish() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--sish"]).unwrap();
        match cli.command {
            Some(Commands::Off { sish, .. }) => {
                assert!(sish);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pinggy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pinggy"]).unwrap();
        match cli.command {
            Some(Commands::On { pinggy, .. }) => {
                assert!(pinggy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--pinggy-tunnel"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { pinggy, .. }) => {
                assert!(pinggy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--pinggy-proxy"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { pinggy, .. }) => {
                assert!(pinggy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--pinggy-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { pinggy, .. }) => {
                assert!(pinggy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pinggy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pinggy"]).unwrap();
        match cli.command {
            Some(Commands::Off { pinggy, .. }) => {
                assert!(pinggy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_loophole() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--loophole"]).unwrap();
        match cli.command {
            Some(Commands::On { loophole, .. }) => {
                assert!(loophole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--loophole-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { loophole, .. }) => {
                assert!(loophole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--loophole-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { loophole, .. }) => {
                assert!(loophole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--loophole-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { loophole, .. }) => {
                assert!(loophole);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_loophole() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--loophole"]).unwrap();
        match cli.command {
            Some(Commands::Off { loophole, .. }) => {
                assert!(loophole);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_tunnelmole() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--tunnelmole"]).unwrap();
        match cli.command {
            Some(Commands::On { tunnelmole, .. }) => {
                assert!(tunnelmole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--tunnelmole-tunnel"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { tunnelmole, .. }) => {
                assert!(tunnelmole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--tunnelmole-proxy"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { tunnelmole, .. }) => {
                assert!(tunnelmole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--tunnelmole-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { tunnelmole, .. }) => {
                assert!(tunnelmole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias4 = Cli::try_parse_from(["proxydoc", "on", "--tmole"]).unwrap();
        match cli_alias4.command {
            Some(Commands::On { tunnelmole, .. }) => {
                assert!(tunnelmole);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_tunnelmole() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--tunnelmole"]).unwrap();
        match cli.command {
            Some(Commands::Off { tunnelmole, .. }) => {
                assert!(tunnelmole);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_expose() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--expose"]).unwrap();
        match cli.command {
            Some(Commands::On { expose, .. }) => {
                assert!(expose);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--expose-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { expose, .. }) => {
                assert!(expose);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--expose-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { expose, .. }) => {
                assert!(expose);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--expose-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { expose, .. }) => {
                assert!(expose);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias4 = Cli::try_parse_from(["proxydoc", "on", "--beyondcode-expose"]).unwrap();
        match cli_alias4.command {
            Some(Commands::On { expose, .. }) => {
                assert!(expose);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_expose() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--expose"]).unwrap();
        match cli.command {
            Some(Commands::Off { expose, .. }) => {
                assert!(expose);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_playit() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--playit"]).unwrap();
        match cli.command {
            Some(Commands::On { playit, .. }) => {
                assert!(playit);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--playit-tunnel"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { playit, .. }) => {
                assert!(playit);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--playit-proxy"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { playit, .. }) => {
                assert!(playit);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--playit-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { playit, .. }) => {
                assert!(playit);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias4 = Cli::try_parse_from(["proxydoc", "on", "--playit-gg"]).unwrap();
        match cli_alias4.command {
            Some(Commands::On { playit, .. }) => {
                assert!(playit);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_playit() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--playit"]).unwrap();
        match cli.command {
            Some(Commands::Off { playit, .. }) => {
                assert!(playit);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_pgrok() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--pgrok"]).unwrap();
        match cli.command {
            Some(Commands::On { pgrok, .. }) => {
                assert!(pgrok);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--pgrok-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { pgrok, .. }) => {
                assert!(pgrok);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--pgrok-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { pgrok, .. }) => {
                assert!(pgrok);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--pgrok-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { pgrok, .. }) => {
                assert!(pgrok);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_pgrok() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--pgrok"]).unwrap();
        match cli.command {
            Some(Commands::Off { pgrok, .. }) => {
                assert!(pgrok);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_cpolar() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--cpolar"]).unwrap();
        match cli.command {
            Some(Commands::On { cpolar, .. }) => {
                assert!(cpolar);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--cpolar-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { cpolar, .. }) => {
                assert!(cpolar);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--cpolar-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { cpolar, .. }) => {
                assert!(cpolar);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--cpolar-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { cpolar, .. }) => {
                assert!(cpolar);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_cpolar() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--cpolar"]).unwrap();
        match cli.command {
            Some(Commands::Off { cpolar, .. }) => {
                assert!(cpolar);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_tmate() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--tmate"]).unwrap();
        match cli.command {
            Some(Commands::On { tmate, .. }) => {
                assert!(tmate);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--tmate-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { tmate, .. }) => {
                assert!(tmate);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--tmate-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { tmate, .. }) => {
                assert!(tmate);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--tmate-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { tmate, .. }) => {
                assert!(tmate);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_tmate() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--tmate"]).unwrap();
        match cli.command {
            Some(Commands::Off { tmate, .. }) => {
                assert!(tmate);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_upterm() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--upterm"]).unwrap();
        match cli.command {
            Some(Commands::On { upterm, .. }) => {
                assert!(upterm);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--upterm-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { upterm, .. }) => {
                assert!(upterm);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--upterm-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { upterm, .. }) => {
                assert!(upterm);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--upterm-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { upterm, .. }) => {
                assert!(upterm);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_upterm() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--upterm"]).unwrap();
        match cli.command {
            Some(Commands::Off { upterm, .. }) => {
                assert!(upterm);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_natapp() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--natapp"]).unwrap();
        match cli.command {
            Some(Commands::On { natapp, .. }) => {
                assert!(natapp);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--natapp-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { natapp, .. }) => {
                assert!(natapp);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--natapp-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { natapp, .. }) => {
                assert!(natapp);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--natapp-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { natapp, .. }) => {
                assert!(natapp);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_natapp() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--natapp"]).unwrap();
        match cli.command {
            Some(Commands::Off { natapp, .. }) => {
                assert!(natapp);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_lanproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--lanproxy"]).unwrap();
        match cli.command {
            Some(Commands::On { lanproxy, .. }) => {
                assert!(lanproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--lanproxy-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { lanproxy, .. }) => {
                assert!(lanproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--lanproxy-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { lanproxy, .. }) => {
                assert!(lanproxy);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--lanproxy-client"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { lanproxy, .. }) => {
                assert!(lanproxy);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_lanproxy() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--lanproxy"]).unwrap();
        match cli.command {
            Some(Commands::Off { lanproxy, .. }) => {
                assert!(lanproxy);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_teleconsole() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--teleconsole"]).unwrap();
        match cli.command {
            Some(Commands::On { teleconsole, .. }) => {
                assert!(teleconsole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--teleconsole-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { teleconsole, .. }) => {
                assert!(teleconsole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--teleconsole-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { teleconsole, .. }) => {
                assert!(teleconsole);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--teleconsole-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { teleconsole, .. }) => {
                assert!(teleconsole);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_teleconsole() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--teleconsole"]).unwrap();
        match cli.command {
            Some(Commands::Off { teleconsole, .. }) => {
                assert!(teleconsole);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_tunnelto() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--tunnelto"]).unwrap();
        match cli.command {
            Some(Commands::On { tunnelto, .. }) => {
                assert!(tunnelto);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--tunnelto-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { tunnelto, .. }) => {
                assert!(tunnelto);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--tunnelto-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { tunnelto, .. }) => {
                assert!(tunnelto);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--tunnelto-client"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { tunnelto, .. }) => {
                assert!(tunnelto);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_tunnelto() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--tunnelto"]).unwrap();
        match cli.command {
            Some(Commands::Off { tunnelto, .. }) => {
                assert!(tunnelto);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_webhookrelay() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--webhookrelay"]).unwrap();
        match cli.command {
            Some(Commands::On { webhookrelay, .. }) => {
                assert!(webhookrelay);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--webhookrelay-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { webhookrelay, .. }) => {
                assert!(webhookrelay);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--webhookrelay-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { webhookrelay, .. }) => {
                assert!(webhookrelay);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--webhookrelay-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { webhookrelay, .. }) => {
                assert!(webhookrelay);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias4 = Cli::try_parse_from(["proxydoc", "on", "--relay"]).unwrap();
        match cli_alias4.command {
            Some(Commands::On { webhookrelay, .. }) => {
                assert!(webhookrelay);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_webhookrelay() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--webhookrelay"]).unwrap();
        match cli.command {
            Some(Commands::Off { webhookrelay, .. }) => {
                assert!(webhookrelay);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_jprq() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--jprq"]).unwrap();
        match cli.command {
            Some(Commands::On { jprq, .. }) => {
                assert!(jprq);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--jprq-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { jprq, .. }) => {
                assert!(jprq);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--jprq-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { jprq, .. }) => {
                assert!(jprq);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--jprq-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { jprq, .. }) => {
                assert!(jprq);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_jprq() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--jprq"]).unwrap();
        match cli.command {
            Some(Commands::Off { jprq, .. }) => {
                assert!(jprq);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_mysocketctl() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--mysocketctl"]).unwrap();
        match cli.command {
            Some(Commands::On { mysocketctl, .. }) => {
                assert!(mysocketctl);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--mysocketctl-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { mysocketctl, .. }) => {
                assert!(mysocketctl);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--mysocketctl-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { mysocketctl, .. }) => {
                assert!(mysocketctl);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--mysocket-proxy"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { mysocketctl, .. }) => {
                assert!(mysocketctl);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias4 = Cli::try_parse_from(["proxydoc", "on", "--border0"]).unwrap();
        match cli_alias4.command {
            Some(Commands::On { mysocketctl, .. }) => {
                assert!(mysocketctl);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_mysocketctl() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--mysocketctl"]).unwrap();
        match cli.command {
            Some(Commands::Off { mysocketctl, .. }) => {
                assert!(mysocketctl);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_openp2p() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--openp2p"]).unwrap();
        match cli.command {
            Some(Commands::On { openp2p, .. }) => {
                assert!(openp2p);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--openp2p-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { openp2p, .. }) => {
                assert!(openp2p);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--openp2p-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { openp2p, .. }) => {
                assert!(openp2p);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--openp2p-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { openp2p, .. }) => {
                assert!(openp2p);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_openp2p() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--openp2p"]).unwrap();
        match cli.command {
            Some(Commands::Off { openp2p, .. }) => {
                assert!(openp2p);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_rport() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--rport"]).unwrap();
        match cli.command {
            Some(Commands::On { rport, .. }) => {
                assert!(rport);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--rport-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { rport, .. }) => {
                assert!(rport);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--rport-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { rport, .. }) => {
                assert!(rport);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--rport-client"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { rport, .. }) => {
                assert!(rport);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias4 = Cli::try_parse_from(["proxydoc", "on", "--rport-daemon"]).unwrap();
        match cli_alias4.command {
            Some(Commands::On { rport, .. }) => {
                assert!(rport);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_rport() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--rport"]).unwrap();
        match cli.command {
            Some(Commands::Off { rport, .. }) => {
                assert!(rport);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_mirrord() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--mirrord"]).unwrap();
        match cli.command {
            Some(Commands::On { mirrord, .. }) => {
                assert!(mirrord);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--mirrord-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { mirrord, .. }) => {
                assert!(mirrord);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--mirrord-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { mirrord, .. }) => {
                assert!(mirrord);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--metalbear"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { mirrord, .. }) => {
                assert!(mirrord);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_mirrord() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--mirrord"]).unwrap();
        match cli.command {
            Some(Commands::Off { mirrord, .. }) => {
                assert!(mirrord);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_kt_connect() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--kt-connect"]).unwrap();
        match cli.command {
            Some(Commands::On { kt_connect, .. }) => {
                assert!(kt_connect);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--kt-connect-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { kt_connect, .. }) => {
                assert!(kt_connect);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--kt-connect-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { kt_connect, .. }) => {
                assert!(kt_connect);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--ktctl"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { kt_connect, .. }) => {
                assert!(kt_connect);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias4 = Cli::try_parse_from(["proxydoc", "on", "--kt-connect-daemon"]).unwrap();
        match cli_alias4.command {
            Some(Commands::On { kt_connect, .. }) => {
                assert!(kt_connect);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_kt_connect() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--kt-connect"]).unwrap();
        match cli.command {
            Some(Commands::Off { kt_connect, .. }) => {
                assert!(kt_connect);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_devspace() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--devspace"]).unwrap();
        match cli.command {
            Some(Commands::On { devspace, .. }) => {
                assert!(devspace);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--devspace-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { devspace, .. }) => {
                assert!(devspace);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--devspace-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { devspace, .. }) => {
                assert!(devspace);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--loft"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { devspace, .. }) => {
                assert!(devspace);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_devspace() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--devspace"]).unwrap();
        match cli.command {
            Some(Commands::Off { devspace, .. }) => {
                assert!(devspace);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_geph() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--geph"]).unwrap();
        match cli.command {
            Some(Commands::On { geph, .. }) => {
                assert!(geph);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--geph-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { geph, .. }) => {
                assert!(geph);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--geph-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { geph, .. }) => {
                assert!(geph);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--geph4"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { geph, .. }) => {
                assert!(geph);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias4 = Cli::try_parse_from(["proxydoc", "on", "--geph5"]).unwrap();
        match cli_alias4.command {
            Some(Commands::On { geph, .. }) => {
                assert!(geph);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_geph() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--geph"]).unwrap();
        match cli.command {
            Some(Commands::Off { geph, .. }) => {
                assert!(geph);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_okteto() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--okteto"]).unwrap();
        match cli.command {
            Some(Commands::On { okteto, .. }) => {
                assert!(okteto);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--okteto-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { okteto, .. }) => {
                assert!(okteto);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--okteto-tunnel"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { okteto, .. }) => {
                assert!(okteto);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--okteto-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { okteto, .. }) => {
                assert!(okteto);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_okteto() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--okteto"]).unwrap();
        match cli.command {
            Some(Commands::Off { okteto, .. }) => {
                assert!(okteto);
            }
            _ => panic!("Expected Off command"),
        }
    }

    #[test]
    fn test_cli_parse_on_skaffold() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "--skaffold"]).unwrap();
        match cli.command {
            Some(Commands::On { skaffold, .. }) => {
                assert!(skaffold);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias = Cli::try_parse_from(["proxydoc", "on", "--skaffold-proxy"]).unwrap();
        match cli_alias.command {
            Some(Commands::On { skaffold, .. }) => {
                assert!(skaffold);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias2 = Cli::try_parse_from(["proxydoc", "on", "--skaffold-port-forward"]).unwrap();
        match cli_alias2.command {
            Some(Commands::On { skaffold, .. }) => {
                assert!(skaffold);
            }
            _ => panic!("Expected On command"),
        }

        let cli_alias3 = Cli::try_parse_from(["proxydoc", "on", "--skaffold-daemon"]).unwrap();
        match cli_alias3.command {
            Some(Commands::On { skaffold, .. }) => {
                assert!(skaffold);
            }
            _ => panic!("Expected On command"),
        }
    }

    #[test]
    fn test_cli_parse_off_skaffold() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "--skaffold"]).unwrap();
        match cli.command {
            Some(Commands::Off { skaffold, .. }) => {
                assert!(skaffold);
            }
            _ => panic!("Expected Off command"),
        }
    }
}
