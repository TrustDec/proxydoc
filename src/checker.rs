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
    ("StepFun (Step-2) API", "https://api.stepfun.com/v1/models"),
    ("01.AI (Yi) API", "https://api.lingyiwanwu.com/v1/models"),
    ("Voyage AI API", "https://api.voyageai.com/v1/models"),
    ("Novita AI API", "https://api.novita.ai/v3/openai/models"),
    ("Hyperbolic API", "https://api.hyperbolic.xyz/v1/models"),
    ("Lambda Labs API", "https://api.lambdalabs.com/v1/models"),
    ("Upstage (Solar) API", "https://api.upstage.ai/v1/solar/models"),
    ("Ai21 Labs (Jamba) API", "https://api.ai21.com/studio/v1/models"),
    ("Baichuan AI API", "https://api.baichuan-ai.com/v1/models"),
    (
        "SenseNova (SenseTime) API",
        "https://api.sensenova.cn/v1/llm/models",
    ),
    (
        "Tencent Hunyuan API",
        "https://api.hunyuan.cloud.tencent.com/v1/models",
    ),
    ("Baseten API", "https://api.baseten.co/v1/models"),
    ("FriendliAI API", "https://api.friendli.ai/v1/models"),
    ("Lepton AI API", "https://api.lepton.ai/v1/models"),
    (
        "Nebius AI Studio API",
        "https://api.studio.nebius.ai/v1/models",
    ),
    ("Chutes AI API", "https://api.chutes.ai/v1/models"),
    (
        "Infini-AI API",
        "https://cloud.infini-ai.com/api/maas/v1/models",
    ),
    ("AIML API", "https://api.aimlapi.com/v1/models"),
    (
        "Crusoe Cloud API",
        "https://api.crusoecloud.com/v1/models",
    ),
    ("Modal", "https://modal.com"),
    ("NVIDIA NIM API", "https://integrate.api.nvidia.com/v1/models"),
    (
        "ByteDance Doubao (Ark) API",
        "https://ark.cn-beijing.volces.com/api/v3/models",
    ),
    ("Fal.ai", "https://queue.fal.run"),
    ("RunPod API", "https://api.runpod.ai/v2"),
    ("Helicone AI Gateway", "https://api.helicone.ai/v1/models"),
    ("Kling AI API", "https://api.klingai.com/v1/models"),
    ("ElevenLabs API", "https://api.elevenlabs.io/v1/models"),
    ("Cartesia Voice AI API", "https://api.cartesia.ai"),
    ("Portkey AI Gateway", "https://api.portkey.ai/v1/models"),
    ("Langfuse API", "https://cloud.langfuse.com/api/public/health"),
    ("Stackage (Haskell)", "https://www.stackage.org"),
    ("Hex.pm Registry", "https://hex.pm/api/packages"),
    ("LiveKit Cloud", "https://cloud.livekit.io"),
    ("Deepgram Speech AI API", "https://api.deepgram.com/v1/projects"),
    ("Weights & Biases (W&B) API", "https://api.wandb.ai/viewer"),
    ("Pinecone Vector DB", "https://api.pinecone.io/indexes"),
    ("CRAN (R Archive)", "https://cran.r-project.org"),
    ("OPAM Registry (OCaml)", "https://opam.ocaml.org"),
    ("Tavily AI Search API", "https://api.tavily.com/search"),
    ("Exa AI Search API", "https://api.exa.ai/search"),
    ("Qdrant Cloud", "https://cloud.qdrant.io"),
    ("AssemblyAI Speech API", "https://api.assemblyai.com/v2/transcript"),
    ("Clojars (Clojure)", "https://clojars.org"),
    ("MetaCPAN (Perl)", "https://fastapi.metacpan.org/v1"),
    ("Jina AI API", "https://r.jina.ai"),
    ("Firecrawl API", "https://api.firecrawl.dev"),
    ("Weaviate Cloud", "https://api.weaviate.io"),
    ("Chroma Vector DB", "https://api.trychroma.com"),
    ("LuaRocks Registry", "https://luarocks.org"),
    ("Perplexity AI API", "https://api.perplexity.ai"),
    ("Codestral (Mistral) API", "https://codestral.mistral.ai/v1/models"),
    ("Zilliz Cloud (Milvus)", "https://cloud.zilliz.com"),
    ("Nimble Directory", "https://nimble.directory"),
    ("Hex PM Registry (Erlang)", "https://repo.hex.pm"),
    ("RubyGems Registry", "https://rubygems.org/api/v1/gems"),
    ("Cursor AI API", "https://api2.cursor.sh"),
    ("LanceDB Cloud", "https://api.lancedb.com"),
    ("Maven Central", "https://repo1.maven.org/maven2"),
    ("NuGet Gallery", "https://api.nuget.org/v3/index.json"),
    ("Packagist", "https://repo.packagist.org/packages.json"),
    ("Vapi AI API", "https://api.vapi.ai"),
    ("Braintrust API", "https://api.braintrust.dev"),
    ("LlamaIndex Cloud", "https://api.cloud.llamaindex.ai"),
    ("Mem0 AI API", "https://api.mem0.ai"),
    ("Supabase API", "https://api.supabase.com"),
    ("Dify Cloud API", "https://api.dify.ai"),
    ("Unstructured.io API", "https://api.unstructuredapp.io"),
    ("Upstash API", "https://api.upstash.com"),
    ("Vercel Cloud API", "https://api.vercel.com/v2/user"),
    ("Resend API", "https://api.resend.com"),
    ("E2B Sandbox API", "https://api.e2b.dev"),
    ("PostHog Analytics API", "https://us.i.posthog.com"),
    ("Pydantic Logfire API", "https://logfire-api.pydantic.dev"),
    ("Railway Cloud API", "https://backboard.railway.com"),
    ("Sentry API", "https://sentry.io/api/0/"),
    ("Neon Database API", "https://console.neon.tech/api/v2"),
    ("Render Cloud API", "https://api.render.com"),
    ("Fly.io Machines API", "https://api.machines.dev/v1"),
    ("Turso Database API", "https://api.turso.tech/v1"),
    ("Clerk Auth API", "https://api.clerk.com/v1"),
    ("Convex Cloud", "https://api.convex.dev"),
    ("Inngest API", "https://api.inngest.com/v1/events"),
    ("Trigger.dev Cloud", "https://api.trigger.dev/healthcheck"),
    ("Civitai API", "https://civitai.com/api/v1/models"),
    ("Axiom API", "https://api.axiom.co/v1/datasets"),
    ("Stripe API", "https://api.stripe.com/v1/charges"),
    (
        "Cloudflare API",
        "https://api.cloudflare.com/client/v4/user/tokens/verify",
    ),
    ("Dub.co API", "https://api.dub.co"),
    ("Koyeb Cloud API", "https://api.koyeb.com/v1/account"),
    ("Novu API", "https://api.novu.co/v1/health-check"),
    ("Linear API", "https://api.linear.app/graphql"),
    ("Codeium API", "https://api.codeium.com"),
    (
        "Datadog API",
        "https://api.datadoghq.com/api/v1/validate",
    ),
    ("Sanity API", "https://api.sanity.io/v1"),
    ("Meilisearch Cloud", "https://cloud.meilisearch.com"),
    ("PlanetScale API", "https://api.planetscale.com/v1"),
    ("Better Stack API", "https://uptime.betterstack.com/api/v2"),
    ("Mintlify API", "https://api.mintlify.com"),
    ("Val Town API", "https://api.val.town/v1"),
    (
        "Hetzner Cloud API",
        "https://api.hetzner.cloud/v1",
    ),
    (
        "GitLab API",
        "https://gitlab.com/api/v4/version",
    ),
    (
        "DigitalOcean API",
        "https://api.digitalocean.com/v2/account",
    ),
    (
        "Netlify API",
        "https://api.netlify.com/api/v1",
    ),
    (
        "Scaleway Cloud API",
        "https://api.scaleway.com",
    ),
    (
        "Linode (Akamai) API",
        "https://api.linode.com/v4/profile",
    ),
    (
        "Bitbucket API",
        "https://api.bitbucket.org/2.0/user",
    ),
    (
        "Vultr Cloud API",
        "https://api.vultr.com/v2/account",
    ),
    (
        "Fastly API",
        "https://api.fastly.com/public-ip-list",
    ),
    (
        "Gitee API",
        "https://gitee.com/api/v5/swagger",
    ),
    (
        "Deno Deploy API",
        "https://api.deno.com/v1/user",
    ),
    (
        "Ollama Registry",
        "https://registry.ollama.ai/v2/",
    ),
    (
        "GHCR (GitHub Packages)",
        "https://ghcr.io/v2/",
    ),
    (
        "GitHub Releases (Objects)",
        "https://objects.githubusercontent.com",
    ),
    (
        "Tailscale API",
        "https://api.tailscale.com/api/v2",
    ),
    (
        "Replit API",
        "https://api.replit.com",
    ),
    (
        "Quay.io (Red Hat Registry)",
        "https://quay.io/v2/",
    ),
    (
        "Terraform Registry",
        "https://registry.terraform.io/v1/providers",
    ),
    (
        "Ngrok API",
        "https://api.ngrok.com",
    ),
    (
        "Grafana Cloud API",
        "https://grafana.com/api/plugins",
    ),
    (
        "Postman API",
        "https://api.getpostman.com",
    ),
    (
        "GitHub Raw (User Content)",
        "https://raw.githubusercontent.com",
    ),
    (
        "Stability AI API",
        "https://api.stability.ai/v1/user/account",
    ),
    (
        "Apify API",
        "https://api.apify.com/v2/acts",
    ),
    (
        "Anyscale Endpoints API",
        "https://api.endpoints.anyscale.com/v1/models",
    ),
    (
        "Kaggle API",
        "https://www.kaggle.com/api/v1",
    ),
    (
        "Rust Toolchain (rustup)",
        "https://static.rust-lang.org/dist/channel-rust-stable.toml",
    ),
    (
        "HashiCorp Releases",
        "https://releases.hashicorp.com",
    ),
    (
        "Alpine Linux CDN",
        "https://dl-cdn.alpinelinux.org/alpine/",
    ),
    (
        "Gradle Distributions",
        "https://services.gradle.org/distributions",
    ),
    (
        "Conan Center (JFrog)",
        "https://center.conan.io",
    ),
    (
        "PyPI Files CDN",
        "https://files.pythonhosted.org",
    ),
    (
        "Node.js Dist",
        "https://nodejs.org/dist/index.json",
    ),
    (
        "Homebrew Formulae",
        "https://formulae.brew.sh/api/formula.json",
    ),
    (
        "Debian Archive CDN",
        "https://deb.debian.org",
    ),
    (
        "Ubuntu Archive CDN",
        "https://archive.ubuntu.com",
    ),
    (
        "Flathub CDN",
        "https://dl.flathub.org/repo/flathub.flatpakrepo",
    ),
    (
        "Snapcraft API",
        "https://api.snapcraft.io",
    ),
    (
        "Fedora Mirror CDN",
        "https://mirrors.fedoraproject.org",
    ),
    (
        "Arch Linux Mirrors",
        "https://archlinux.org/mirrorlist/all/",
    ),
    (
        "GitHub Actions Runners",
        "https://github.com/actions/runner/releases",
    ),
    (
        "Amazon ECR Public",
        "https://public.ecr.aws",
    ),
    (
        "Google Container Registry (GCR)",
        "https://gcr.io/v2/",
    ),
    (
        "Microsoft Container Registry (MCR)",
        "https://mcr.microsoft.com/v2/",
    ),
    (
        "Rocky Linux CDN",
        "https://download.rockylinux.org",
    ),
    (
        "Jenkins Update Center",
        "https://updates.jenkins.io/update-center.json",
    ),
    (
        "Buildkite API",
        "https://api.buildkite.com/v2",
    ),
    (
        "AlmaLinux CDN",
        "https://repo.almalinux.org",
    ),
    (
        "Oracle Linux CDN",
        "https://yum.oracle.com",
    ),
    (
        "Woodpecker CI",
        "https://woodpecker-ci.org",
    ),
    (
        "OpenSUSE CDN",
        "https://download.opensuse.org",
    ),
    (
        "Drone CI",
        "https://drone.io",
    ),
    (
        "Codeberg API",
        "https://codeberg.org/api/v1/version",
    ),
    (
        "Gitea Releases",
        "https://dl.gitea.com",
    ),
    (
        "Void Linux CDN",
        "https://repo-default.voidlinux.org",
    ),
    (
        "FreeBSD Pkg CDN",
        "https://pkg.freebsd.org",
    ),
    (
        "CircleCI API",
        "https://circleci.com/api/v2/me",
    ),
    (
        "Azure DevOps API",
        "https://dev.azure.com",
    ),
    (
        "Gentoo Linux CDN",
        "https://distfiles.gentoo.org",
    ),
    (
        "OpenBSD CDN",
        "https://cdn.openbsd.org/pub/OpenBSD/",
    ),
    (
        "NixOS Binary Cache",
        "https://cache.nixos.org/nix-cache-info",
    ),
    (
        "JetBrains Marketplace & Plugins",
        "https://plugins.jetbrains.com",
    ),
    (
        "Atlassian API & Bitbucket",
        "https://api.bitbucket.org",
    ),
    (
        "NetBSD CDN",
        "https://cdn.netbsd.org/pub/NetBSD/",
    ),
    (
        "HashiCorp Terraform Cloud API",
        "https://app.terraform.io/api/v2/ping",
    ),
    (
        "Concourse CI",
        "https://concourse-ci.org",
    ),
    (
        "GitLab Package Registry CDN",
        "https://packages.gitlab.com",
    ),
    (
        "JFrog Releases",
        "https://releases.jfrog.io",
    ),
    (
        "DragonFly BSD CDN",
        "https://mirror-master.dragonflybsd.org",
    ),
    (
        "Octopus Deploy API & Portal",
        "https://octopus.com",
    ),
    (
        "Puppet Forge API",
        "https://forgeapi.puppet.com/v3/releases",
    ),
    (
        "Ansible Galaxy API",
        "https://galaxy.ansible.com/api/",
    ),
    (
        "CentOS Stream Mirror CDN",
        "https://mirror.stream.centos.org",
    ),
    (
        "Debian Security CDN",
        "https://security.debian.org",
    ),
    (
        "Chef Supermarket API",
        "https://supermarket.chef.io",
    ),
    (
        "Salt Project Package CDN",
        "https://repo.saltproject.io",
    ),
    (
        "Kali Linux Package CDN",
        "https://http.kali.org",
    ),
    (
        "Arch Linux Archive CDN",
        "https://archive.archlinux.org",
    ),
    (
        "Amazon Linux CDN",
        "https://cdn.amazonlinux.com",
    ),
    (
        "New Relic Infrastructure Intake API",
        "https://infra-api.newrelic.com",
    ),
    (
        "New Relic Package CDN",
        "https://download.newrelic.com",
    ),
    (
        "Datadog Package CDN",
        "https://apt.datadoghq.com",
    ),
    (
        "Linux Mint Package CDN",
        "https://packages.linuxmint.com",
    ),
    (
        "Raspberry Pi OS Archive CDN",
        "https://archive.raspberrypi.com",
    ),
    (
        "Elastic Package Registry CDN",
        "https://epr.elastic.co",
    ),
    (
        "Elastic Artifacts CDN",
        "https://artifacts.elastic.co",
    ),
    (
        "Grafana Labs Package CDN",
        "https://packages.grafana.com",
    ),
    (
        "OpenTelemetry CDN",
        "https://opentelemetry.io",
    ),
    (
        "Manjaro Linux Package CDN",
        "https://repo.manjaro.org",
    ),
    (
        "Vector Package CDN",
        "https://packages.timber.io",
    ),
    (
        "Cloudflare cdnjs CDN",
        "https://cdnjs.cloudflare.com",
    ),
    (
        "jsDelivr CDN",
        "https://cdn.jsdelivr.net",
    ),
    (
        "unpkg CDN",
        "https://unpkg.com",
    ),
    (
        "Fluent Bit Package CDN",
        "https://packages.fluentbit.io",
    ),
    (
        "Fluentd Package CDN",
        "https://packages.treasuredata.com",
    ),
    (
        "Fastly CDN",
        "https://www.fastly.com",
    ),
    (
        "Statically CDN",
        "https://cdn.statically.io",
    ),
    (
        "InfluxData Package CDN",
        "https://repos.influxdata.com",
    ),
    (
        "InfluxData Cloud API",
        "https://cloud2.influxdata.com",
    ),
    (
        "esm.sh CDN",
        "https://esm.sh",
    ),
    (
        "Skypack CDN",
        "https://cdn.skypack.dev",
    ),
    (
        "Prometheus CDN",
        "https://prometheus.io",
    ),
    (
        "Thanos Monitoring CDN",
        "https://thanos.io",
    ),
    (
        "Cortex Metrics CDN",
        "https://cortexmetrics.io",
    ),
    (
        "PagerDuty Events API",
        "https://events.pagerduty.com",
    ),
    (
        "OpsGenie Alert API",
        "https://api.opsgenie.com",
    ),
    (
        "VictoriaMetrics CDN",
        "https://victoriametrics.com",
    ),
    (
        "Grafana Loki CDN",
        "https://grafana.com",
    ),
    (
        "Jaeger Tracing CDN",
        "https://www.jaegertracing.io",
    ),
    (
        "Netdata Cloud API",
        "https://api.netdata.cloud",
    ),
    (
        "Netdata Monitoring CDN",
        "https://www.netdata.cloud",
    ),
    (
        "Elastic Cloud API",
        "https://api.elastic.co",
    ),
    (
        "Grafana Pyroscope CDN",
        "https://pyroscope.io",
    ),
    (
        "OpenSearch CDN",
        "https://opensearch.org",
    ),
    (
        "OpenSearch Artifacts CDN",
        "https://artifacts.opensearch.org",
    ),
    (
        "ClickHouse CDN",
        "https://clickhouse.com",
    ),
    (
        "Elastic Artifacts CDN",
        "https://artifacts.elastic.co",
    ),
    (
        "Zabbix CDN",
        "https://www.zabbix.com",
    ),
    (
        "Grafana Mimir CDN",
        "https://grafana.com/oss/mimir",
    ),
    (
        "Red Hat CDN",
        "https://access.redhat.com",
    ),
    (
        "Rocky Linux CDN",
        "https://rockylinux.org",
    ),
    (
        "AlmaLinux CDN",
        "https://almalinux.org",
    ),
    (
        "Grafana Tempo CDN",
        "https://grafana.com/oss/tempo",
    ),
    (
        "Graylog CDN",
        "https://graylog.org",
    ),
    (
        "HashiCorp Consul CDN",
        "https://www.consul.io",
    ),
    (
        "HashiCorp Vault CDN",
        "https://www.vaultproject.io",
    ),
    (
        "OpenSearch Forum CDN",
        "https://forum.opensearch.org",
    ),
    (
        "OpenNebula CDN",
        "https://opennebula.io",
    ),
    (
        "Cloudflare Workers CDN",
        "https://workers.cloudflare.com",
    ),
    (
        "Traefik Proxy CDN",
        "https://traefik.io",
    ),
    (
        "Envoy Proxy CDN",
        "https://www.envoyproxy.io",
    ),
    (
        "Caddy Web Server CDN",
        "https://caddyserver.com",
    ),
    (
        "Kong API Gateway CDN",
        "https://konghq.com",
    ),
    (
        "RabbitMQ CDN",
        "https://www.rabbitmq.com",
    ),
    (
        "Apache Kafka CDN",
        "https://kafka.apache.org",
    ),
    (
        "HashiCorp Boundary CDN",
        "https://www.boundaryproject.io",
    ),
    (
        "HashiCorp Packer CDN",
        "https://www.packer.io",
    ),
    (
        "Nginx CDN",
        "https://nginx.org",
    ),
    (
        "HAProxy CDN",
        "https://www.haproxy.org",
    ),
    (
        "Varnish Cache CDN",
        "https://varnish-cache.org",
    ),
    (
        "Apache Cassandra CDN",
        "https://cassandra.apache.org",
    ),
    (
        "Apache Flink CDN",
        "https://flink.apache.org",
    ),
    (
        "Apache Spark CDN",
        "https://spark.apache.org",
    ),
    (
        "Apache Pulsar CDN",
        "https://pulsar.apache.org",
    ),
    (
        "Apache Airflow CDN",
        "https://airflow.apache.org",
    ),
    (
        "ScyllaDB CDN",
        "https://www.scylladb.com",
    ),
    (
        "Apache Iceberg CDN",
        "https://iceberg.apache.org",
    ),
    (
        "Apache Arrow CDN",
        "https://arrow.apache.org",
    ),
    (
        "Apache Druid CDN",
        "https://druid.apache.org",
    ),
    (
        "Apache Superset CDN",
        "https://superset.apache.org",
    ),
    (
        "Trino CDN",
        "https://trino.io",
    ),
    (
        "DuckDB CDN",
        "https://duckdb.org",
    ),
    (
        "Apache Doris CDN",
        "https://doris.apache.org",
    ),
    (
        "Apache Hudi CDN",
        "https://hudi.apache.org",
    ),
    (
        "Polars CDN",
        "https://pola.rs",
    ),
    (
        "Ray CDN",
        "https://www.ray.io",
    ),
    (
        "Apache APISIX CDN",
        "https://apisix.apache.org",
    ),
    (
        "Squid Cache CDN",
        "https://www.squid-cache.org",
    ),
    (
        "Cilium CDN",
        "https://cilium.io",
    ),
    (
        "Linkerd CDN",
        "https://linkerd.io",
    ),
    (
        "Istio CDN",
        "https://istio.io",
    ),
    (
        "Milvus CDN",
        "https://milvus.io",
    ),
    (
        "Privoxy CDN",
        "https://www.privoxy.org",
    ),
    (
        "Tinyproxy CDN",
        "https://tinyproxy.github.io",
    ),
    (
        "Mitmproxy CDN",
        "https://mitmproxy.org",
    ),
    (
        "Tor Project CDN",
        "https://www.torproject.org",
    ),
    (
        "LangChain CDN",
        "https://www.langchain.com",
    ),
    (
        "vLLM CDN",
        "https://vllm.ai",
    ),
    (
        "Open WebUI CDN",
        "https://openwebui.com",
    ),
    (
        "Dify CDN",
        "https://dify.ai",
    ),
    (
        "Cloudflare Tunnel CDN",
        "https://pkg.cloudflare.com",
    ),
    (
        "Ngrok Download CDN",
        "https://download.ngrok.com",
    ),
    (
        "LiteLLM CDN",
        "https://litellm.ai",
    ),
    (
        "CrewAI CDN",
        "https://www.crewai.com",
    ),
    (
        "LangGraph CDN",
        "https://langgraph.org",
    ),
    (
        "Continue.dev CDN",
        "https://www.continue.dev",
    ),
    (
        "Frp Release CDN",
        "https://github.com/fatedier/frp/releases",
    ),
    (
        "Tailscale Package CDN",
        "https://pkgs.tailscale.com",
    ),
    (
        "Unsloth AI CDN",
        "https://unsloth.ai",
    ),
    (
        "ComfyUI CDN",
        "https://comfy.org",
    ),
    (
        "AnythingLLM CDN",
        "https://useanything.com",
    ),
    (
        "Windsurf AI CDN",
        "https://windsurf.codeium.com",
    ),
    (
        "ZeroTier Network CDN",
        "https://download.zerotier.com",
    ),
    (
        "Gost Tunnel CDN",
        "https://github.com/go-gost/gost/releases",
    ),
    (
        "Promptfoo CDN",
        "https://www.promptfoo.dev",
    ),
    (
        "CopilotKit CDN",
        "https://copilotkit.ai",
    ),
    (
        "Browserbase CDN",
        "https://www.browserbase.com",
    ),
    (
        "DSPy AI CDN",
        "https://dspy.ai",
    ),
    (
        "Rathole Tunnel CDN",
        "https://github.com/rapiz1/rathole/releases",
    ),
    (
        "Chisel Tunnel CDN",
        "https://github.com/jpillora/chisel/releases",
    ),
    (
        "OpenHands AI CDN",
        "https://docs.all-hands.dev",
    ),
    (
        "Pydantic AI CDN",
        "https://ai.pydantic.dev",
    ),
    (
        "Smolagents CDN",
        "https://huggingface.co/docs/smolagents",
    ),
    (
        "Mem0 AI CDN",
        "https://mem0.ai",
    ),
    (
        "Nebula Network CDN",
        "https://github.com/slackhq/nebula/releases",
    ),
    (
        "Bore Tunnel CDN",
        "https://github.com/ekzhang/bore/releases",
    ),
    (
        "Crawl4AI CDN",
        "https://crawl4ai.com",
    ),
    (
        "Browser Use CDN",
        "https://browser-use.com",
    ),
    (
        "Agno AI CDN",
        "https://docs.agno.com",
    ),
    (
        "DeepEval CDN",
        "https://deepeval.confident-ai.com",
    ),
    (
        "Brook Proxy CDN",
        "https://github.com/txthinking/brook/releases",
    ),
    (
        "NPC Tunnel CDN",
        "https://github.com/ehang-io/nps/releases",
    ),
    (
        "Docling AI CDN",
        "https://docling.ai",
    ),
    (
        "Mirascope AI CDN",
        "https://mirascope.com",
    ),
    (
        "Letta AI CDN",
        "https://www.letta.com",
    ),
    (
        "Pipecat AI CDN",
        "https://www.pipecat.ai",
    ),
    (
        "Shadowsocks Proxy CDN",
        "https://github.com/shadowsocks/shadowsocks-rust/releases",
    ),
    (
        "V2Ray Proxy CDN",
        "https://github.com/v2fly/v2ray-core/releases",
    ),
    (
        "FastRTC AI CDN",
        "https://fastrtc.org",
    ),
    (
        "Stagehand AI CDN",
        "https://stagehand.dev",
    ),
    (
        "SGLang AI CDN",
        "https://sgl-project.ai",
    ),
    (
        "LlamaParse AI CDN",
        "https://llamaparse.ai",
    ),
    (
        "Sing-box Proxy CDN",
        "https://github.com/SagerNet/sing-box/releases",
    ),
    (
        "Hysteria Proxy CDN",
        "https://github.com/apernet/hysteria/releases",
    ),
    (
        "Jan AI CDN",
        "https://jan.ai",
    ),
    (
        "MarkItDown AI CDN",
        "https://github.com/microsoft/markitdown",
    ),
    (
        "LM Studio AI CDN",
        "https://lmstudio.ai",
    ),
    (
        "Fish Audio CDN",
        "https://fish.audio",
    ),
    (
        "Trojan Proxy CDN",
        "https://github.com/trojan-gfw/trojan/releases",
    ),
    (
        "NaïveProxy CDN",
        "https://github.com/klzgrad/naiveproxy/releases",
    ),
    (
        "Kokoro AI CDN",
        "https://github.com/hexgrad/kokoro",
    ),
    (
        "OmniParser AI CDN",
        "https://github.com/microsoft/OmniParser",
    ),
    (
        "Bolt.new AI CDN",
        "https://bolt.new",
    ),
    (
        "Open-NotebookLM CDN",
        "https://github.com/gabrielchua/open-notebooklm",
    ),
    (
        "Tuic Proxy CDN",
        "https://github.com/EAimTY/tuic/releases",
    ),
    (
        "Juicity Proxy CDN",
        "https://github.com/juicity/juicity/releases",
    ),
    (
        "Open-R1 AI CDN",
        "https://github.com/huggingface/open-r1",
    ),
    (
        "MCP Servers CDN",
        "https://github.com/modelcontextprotocol/servers",
    ),
    (
        "Qwen AI CDN",
        "https://github.com/QwenLM/Qwen2.5",
    ),
    (
        "Moonshot Kimi AI CDN",
        "https://kimi.moonshot.cn",
    ),
    (
        "Snell Proxy CDN",
        "https://github.com/surge-networks/snell/releases",
    ),
    (
        "Leaf Proxy CDN",
        "https://github.com/eycorsican/leaf/releases",
    ),
    (
        "DeepSeek-R1 AI CDN",
        "https://github.com/deepseek-ai/DeepSeek-R1",
    ),
    (
        "OpenManus AI CDN",
        "https://github.com/mannaandpoem/OpenManus",
    ),
    (
        "Claude Code CDN",
        "https://github.com/anthropics/claude-code",
    ),
    (
        "Mini-Omni AI CDN",
        "https://github.com/gpt-omni/mini-omni",
    ),
    (
        "Mihomo Proxy CDN",
        "https://github.com/MetaCubeX/mihomo/releases",
    ),
    (
        "Stunnel Proxy CDN",
        "https://github.com/mtrojnar/stunnel/releases",
    ),
    (
        "QwQ AI CDN",
        "https://github.com/QwenLM/QwQ",
    ),
    (
        "RAGFlow AI CDN",
        "https://github.com/infiniflow/ragflow",
    ),
    (
        "LobeChat AI CDN",
        "https://github.com/lobehub/lobe-chat",
    ),
    (
        "Moshi AI CDN",
        "https://github.com/kyutai-labs/moshi",
    ),
    (
        "Shadow-TLS Proxy CDN",
        "https://github.com/ihciah/shadow-tls/releases",
    ),
    (
        "Tun2socks Proxy CDN",
        "https://github.com/xjasonlyu/tun2socks/releases",
    ),
    (
        "Janus-Pro AI CDN",
        "https://github.com/deepseek-ai/Janus",
    ),
    (
        "SmolLM2 AI CDN",
        "https://github.com/huggingface/smollm",
    ),
    (
        "DeepEP AI CDN",
        "https://github.com/deepseek-ai/DeepEP",
    ),
    (
        "Vanna AI CDN",
        "https://github.com/vanna-ai/vanna",
    ),
    (
        "WSTunnel Proxy CDN",
        "https://github.com/erebe/wstunnel/releases",
    ),
    (
        "Cloak Proxy CDN",
        "https://github.com/cbeuw/Cloak/releases",
    ),
    (
        "DeepSeek-V3 AI CDN",
        "https://github.com/deepseek-ai/DeepSeek-V3",
    ),
    (
        "UI-TARS AI CDN",
        "https://github.com/bytedance/UI-TARS",
    ),
    (
        "LightRAG AI CDN",
        "https://github.com/HKUDS/LightRAG",
    ),
    (
        "Kimi-k1.5 AI CDN",
        "https://github.com/MoonshotAI/Kimi-k1.5",
    ),
    (
        "Wireproxy Proxy CDN",
        "https://github.com/pufferffish/wireproxy/releases",
    ),
    (
        "MTG Proxy CDN",
        "https://github.com/9seconds/mtg/releases",
    ),
    (
        "Qwen2.5-Coder AI CDN",
        "https://github.com/QwenLM/Qwen2.5-Coder",
    ),
    (
        "DeepSeek-VL2 AI CDN",
        "https://github.com/deepseek-ai/DeepSeek-VL2",
    ),
    (
        "SWE-agent AI CDN",
        "https://github.com/princeton-nlp/SWE-agent",
    ),
    (
        "Fish-Speech AI CDN",
        "https://github.com/fishaudio/fish-speech",
    ),
    (
        "Kcptun Proxy CDN",
        "https://github.com/xtaci/kcptun/releases",
    ),
    (
        "Glider Proxy CDN",
        "https://github.com/nadoo/glider/releases",
    ),
    (
        "Qwen2.5-VL AI CDN",
        "https://github.com/QwenLM/Qwen2.5-VL",
    ),
    (
        "HunyuanVideo AI CDN",
        "https://github.com/Tencent/HunyuanVideo",
    ),
    (
        "Aider AI CDN",
        "https://github.com/Aider-AI/aider",
    ),
    (
        "MiniCPM AI CDN",
        "https://github.com/OpenBMB/MiniCPM",
    ),
    (
        "Redsocks Proxy CDN",
        "https://github.com/darkk/redsocks/releases",
    ),
    (
        "Dante Proxy CDN",
        "https://github.com/royhills/dante",
    ),
    (
        "Wan2.1 AI CDN",
        "https://github.com/Wan-Video/Wan2.1",
    ),
    (
        "ChatTTS AI CDN",
        "https://github.com/2noise/ChatTTS",
    ),
    (
        "OmniGen AI CDN",
        "https://github.com/VectorSpaceLab/OmniGen",
    ),
    (
        "Llama-3.3 AI CDN",
        "https://github.com/meta-llama/llama-models",
    ),
    (
        "3proxy Proxy CDN",
        "https://github.com/3proxy/3proxy/releases",
    ),
    (
        "Polipo Proxy CDN",
        "https://github.com/jech/polipo",
    ),
    (
        "GLM-4 AI CDN",
        "https://github.com/THUDM/GLM-4",
    ),
    (
        "CosyVoice AI CDN",
        "https://github.com/FunAudioLLM/CosyVoice",
    ),
    (
        "LLaMA-Factory AI CDN",
        "https://github.com/hiyouga/LLaMA-Factory",
    ),
    (
        "FLUX AI CDN",
        "https://github.com/black-forest-labs/flux",
    ),
    (
        "Pproxy Proxy CDN",
        "https://github.com/qwj/python-proxy",
    ),
    (
        "MicroSOCKS Proxy CDN",
        "https://github.com/rofl0r/microsocks",
    ),
    (
        "InternVL AI CDN",
        "https://github.com/OpenGVLab/InternVL",
    ),
    (
        "CogVideoX AI CDN",
        "https://github.com/THUDM/CogVideo",
    ),
    (
        "F5-TTS AI CDN",
        "https://github.com/SWivid/F5-TTS",
    ),
    (
        "SenseVoice AI CDN",
        "https://github.com/FunAudioLLM/SenseVoice",
    ),
    (
        "Cntlm Proxy CDN",
        "https://github.com/versit/cntlm",
    ),
    (
        "COW Proxy CDN",
        "https://github.com/cyfdecyf/cow",
    ),
    (
        "GraphRAG AI CDN",
        "https://github.com/microsoft/graphrag",
    ),
    (
        "Moondream AI CDN",
        "https://github.com/vikhyat/moondream",
    ),
    (
        "FastEmbed AI CDN",
        "https://github.com/qdrant/fastembed",
    ),
    (
        "AutoGPT AI CDN",
        "https://github.com/Significant-Gravitas/AutoGPT",
    ),
    (
        "Sniproxy Proxy CDN",
        "https://github.com/dlundquist/sniproxy",
    ),
    (
        "Dnsproxy Proxy CDN",
        "https://github.com/AdguardTeam/dnsproxy",
    ),
    (
        "LLaVA AI CDN",
        "https://github.com/haotian-liu/LLaVA",
    ),
    (
        "Whisper AI CDN",
        "https://github.com/openai/whisper",
    ),
    (
        "Ovis AI CDN",
        "https://github.com/AIDC-AI/Ovis",
    ),
    (
        "Diffusers AI CDN",
        "https://github.com/huggingface/diffusers",
    ),
    (
        "Mosdns Proxy CDN",
        "https://github.com/IrineSistiana/mosdns",
    ),
    (
        "SmartDNS Proxy CDN",
        "https://github.com/pymumu/smartdns",
    ),
    (
        "Transformers AI CDN",
        "https://github.com/huggingface/transformers",
    ),
    (
        "PyTorch AI CDN",
        "https://github.com/pytorch/pytorch",
    ),
    (
        "Gemma AI CDN",
        "https://github.com/google-deepmind/gemma",
    ),
    (
        "Cline AI CDN",
        "https://github.com/cline/cline",
    ),
    (
        "CoreDNS Proxy CDN",
        "https://github.com/coredns/coredns",
    ),
    (
        "AdGuard Home Proxy CDN",
        "https://github.com/AdguardTeam/AdGuardHome",
    ),
    (
        "Llama.cpp AI CDN",
        "https://github.com/ggerganov/llama.cpp",
    ),
    (
        "Open-WebUI AI CDN",
        "https://github.com/open-webui/open-webui",
    ),
    (
        "AutoGen AI CDN",
        "https://github.com/microsoft/autogen",
    ),
    (
        "Torchtune AI CDN",
        "https://github.com/pytorch/torchtune",
    ),
    (
        "DNSCrypt-proxy Proxy CDN",
        "https://github.com/DNSCrypt/dnscrypt-proxy",
    ),
    (
        "Unbound Proxy CDN",
        "https://github.com/NLnetLabs/unbound",
    ),
    (
        "Roo-Code AI CDN",
        "https://github.com/RooVetGit/Roo-Cline",
    ),
    (
        "Haystack AI CDN",
        "https://github.com/deepset-ai/haystack",
    ),
    (
        "Zed AI CDN",
        "https://github.com/zed-industries/zed",
    ),
    (
        "Plandex AI CDN",
        "https://github.com/plandex-ai/plandex",
    ),
    (
        "Dnsmasq Proxy CDN",
        "https://github.com/imp/dnsmasq",
    ),
    (
        "Pi-hole Proxy CDN",
        "https://github.com/pi-hole/pi-hole",
    ),
    (
        "MetaGPT AI CDN",
        "https://github.com/geekan/MetaGPT",
    ),
    (
        "ChatDev AI CDN",
        "https://github.com/OpenBMB/ChatDev",
    ),
    (
        "FastGPT AI CDN",
        "https://github.com/labring/FastGPT",
    ),
    (
        "BabyAGI AI CDN",
        "https://github.com/yoheinakajima/babyagi",
    ),
    (
        "Blocky Proxy CDN",
        "https://github.com/0xERR0R/blocky",
    ),
    (
        "Stubby Proxy CDN",
        "https://github.com/getdnsapi/stubby",
    ),
    (
        "Langflow AI CDN",
        "https://github.com/langflow-ai/langflow",
    ),
    (
        "Flowise AI CDN",
        "https://github.com/FlowiseAI/Flowise",
    ),
    (
        "AgentGPT AI CDN",
        "https://github.com/reworkd/AgentGPT",
    ),
    (
        "Dagger AI CDN",
        "https://github.com/dagger/dagger",
    ),
    (
        "Knot Resolver Proxy CDN",
        "https://github.com/CZ-NIC/knot-resolver",
    ),
    (
        "PowerDNS Proxy CDN",
        "https://github.com/PowerDNS/pdns",
    ),
    (
        "LibreChat AI CDN",
        "https://github.com/danny-avila/LibreChat",
    ),
    (
        "LocalAI AI CDN",
        "https://github.com/mudler/LocalAI",
    ),
    (
        "Tabby AI CDN",
        "https://github.com/TabbyML/tabby",
    ),
    (
        "Khoj AI CDN",
        "https://github.com/khoj-ai/khoj",
    ),
    (
        "dnsdist Proxy CDN",
        "https://dnsdist.org",
    ),
    (
        "BIND 9 Proxy CDN",
        "https://gitlab.isc.org/isc-projects/bind9",
    ),
    (
        "Open-Interpreter AI CDN",
        "https://github.com/openinterpreter/open-interpreter",
    ),
    (
        "Text-Gen-WebUI AI CDN",
        "https://github.com/oobabooga/text-generation-webui",
    ),
    (
        "Fooocus AI CDN",
        "https://github.com/lllyasviel/Fooocus",
    ),
    (
        "InvokeAI AI CDN",
        "https://github.com/invoke-ai/InvokeAI",
    ),
    (
        "NextDNS Proxy CDN",
        "https://nextdns.io",
    ),
    (
        "pdnsd Proxy CDN",
        "http://members.home.nl/p.a.rombouts/pdnsd",
    ),
    (
        "AnythingLLM AI CDN",
        "https://github.com/Mintplex-Labs/anything-llm",
    ),
    (
        "Continue AI CDN",
        "https://github.com/continuedev/continue",
    ),
    (
        "PrivateGPT AI CDN",
        "https://github.com/zylon-ai/private-gpt",
    ),
    (
        "SearXNG AI CDN",
        "https://github.com/searxng/searxng",
    ),
    (
        "MaraDNS Proxy CDN",
        "https://maradns.samiam.org",
    ),
    (
        "i2pd Proxy CDN",
        "https://i2pd.website",
    ),
    (
        "Chatbox AI CDN",
        "https://github.com/Bin-Huang/chatbox",
    ),
    (
        "SuperAGI AI CDN",
        "https://github.com/TransformerOptimus/SuperAGI",
    ),
    (
        "GPT-Engineer AI CDN",
        "https://github.com/gpt-engineer-org/gpt-engineer",
    ),
    (
        "Phidata AI CDN",
        "https://github.com/phidatahq/phidata",
    ),
    (
        "dae Proxy CDN",
        "https://github.com/daeuniverse/dae",
    ),
    (
        "Yggdrasil Proxy CDN",
        "https://yggdrasil-network.github.io",
    ),
    (
        "Devika AI CDN",
        "https://github.com/stitionai/devika",
    ),
    (
        "STORM AI CDN",
        "https://github.com/stanford-oval/storm",
    ),
    (
        "Open-Sora AI CDN",
        "https://github.com/hpcaitech/Open-Sora",
    ),
    (
        "AgentOps AI CDN",
        "https://github.com/AgentOps-AI/agentops",
    ),
    (
        "cjdns Proxy CDN",
        "https://github.com/cjdelisle/cjdns",
    ),
    (
        "tinc Proxy CDN",
        "https://www.tinc-vpn.org",
    ),
    (
        "Instructor AI CDN",
        "https://github.com/jxnl/instructor",
    ),
    (
        "Phoenix AI CDN",
        "https://github.com/Arize-ai/phoenix",
    ),
    (
        "LitServe AI CDN",
        "https://github.com/Lightning-AI/litserve",
    ),
    (
        "Swarm AI CDN",
        "https://github.com/openai/swarm",
    ),
    (
        "OpenVPN Proxy CDN",
        "https://openvpn.net",
    ),
    (
        "SoftEther Proxy CDN",
        "https://www.softether.org",
    ),
    (
        "SmolVLM AI CDN",
        "https://github.com/huggingface/smolvlm",
    ),
    (
        "Agent-Zero AI CDN",
        "https://github.com/frdel/agent-zero",
    ),
    (
        "TinyZero AI CDN",
        "https://github.com/Jiayi-Pan/TinyZero",
    ),
    (
        "FinRobot AI CDN",
        "https://github.com/AI4Finance-Foundation/FinRobot",
    ),
    (
        "OpenConnect Proxy CDN",
        "https://www.infradead.org/openconnect/",
    ),
    (
        "strongSwan VPN CDN",
        "https://www.strongswan.org",
    ),
    (
        "MinerU AI CDN",
        "https://github.com/opendatalab/MinerU",
    ),
    (
        "Pydantic-AI CDN",
        "https://github.com/pydantic/pydantic-ai",
    ),
    (
        "KTransformers AI CDN",
        "https://github.com/kvcache-ai/ktransformers",
    ),
    (
        "FlashAttention AI CDN",
        "https://github.com/Dao-AILab/flash-attention",
    ),
    (
        "WireGuard VPN CDN",
        "https://www.wireguard.com",
    ),
    (
        "Headscale Mesh CDN",
        "https://headscale.net",
    ),
    (
        "Cherry Studio AI CDN",
        "https://github.com/Kangfenfei/cherry-studio",
    ),
    (
        "Kotaemon RAG AI CDN",
        "https://github.com/Cinnamon/kotaemon",
    ),
    (
        "Marker PDF AI CDN",
        "https://github.com/VikParuchuri/marker",
    ),
    (
        "PDFMathTranslate AI CDN",
        "https://github.com/Byaidu/PDFMathTranslate",
    ),
    (
        "NetBird Mesh CDN",
        "https://netbird.io",
    ),
    (
        "Cloudflare WARP CDN",
        "https://developers.cloudflare.com/cloudflare-one/connections/connect-devices/warp/",
    ),
    (
        "DeepScaler AI CDN",
        "https://github.com/agentica-project/deepscaler",
    ),
    (
        "Podcastfy AI CDN",
        "https://github.com/souzatharsis/podcastfy",
    ),
    (
        "OpenCompass AI CDN",
        "https://github.com/open-compass/opencompass",
    ),
    (
        "NanoFlow AI CDN",
        "https://github.com/infinigence/NanoFlow",
    ),
    (
        "Amnezia VPN CDN",
        "https://amnezia.org",
    ),
    (
        "OpenZiti Mesh CDN",
        "https://openziti.io",
    ),
    (
        "DeepGEMM AI CDN",
        "https://github.com/deepseek-ai/DeepGEMM",
    ),
    (
        "DualPipe AI CDN",
        "https://github.com/deepseek-ai/DualPipe",
    ),
    (
        "EPLB AI CDN",
        "https://github.com/deepseek-ai/eplb",
    ),
    (
        "3FS AI CDN",
        "https://github.com/deepseek-ai/3FS",
    ),
    (
        "Firezone ZTNA CDN",
        "https://www.firezone.dev",
    ),
    (
        "Netmaker Mesh CDN",
        "https://www.netmaker.io",
    ),
    (
        "Deep-Searcher AI CDN",
        "https://github.com/zilliztech/deep-searcher",
    ),
    (
        "Open-Thoughts AI CDN",
        "https://github.com/open-thoughts/open-thoughts",
    ),
    (
        "Deep-Live-Cam AI CDN",
        "https://github.com/hacksider/Deep-Live-Cam",
    ),
    (
        "BitNet AI CDN",
        "https://github.com/microsoft/BitNet",
    ),
    (
        "Pritunl VPN CDN",
        "https://pritunl.com",
    ),
    (
        "Twingate ZTNA CDN",
        "https://www.twingate.com",
    ),
    (
        "YuE AI CDN",
        "https://github.com/multimodal-art-projection/YuE",
    ),
    (
        "Surya AI CDN",
        "https://github.com/VikParuchuri/surya",
    ),
    (
        "Step-Audio AI CDN",
        "https://github.com/stepfun-ai/Step-Audio",
    ),
    (
        "Skywork AI CDN",
        "https://github.com/SkyworkAI/Skywork-Reward",
    ),
    (
        "EasyTier Mesh CDN",
        "https://github.com/EasyTier/EasyTier",
    ),
    (
        "Teleport ZTNA CDN",
        "https://goteleport.com",
    ),
    (
        "Smolagents AI CDN",
        "https://github.com/huggingface/smolagents",
    ),
    (
        "Open-R1 AI CDN",
        "https://github.com/huggingface/open-r1",
    ),
    (
        "Kokoro AI CDN",
        "https://github.com/hexgrad/kokoro",
    ),
    (
        "Qwen-Agent AI CDN",
        "https://github.com/QwenLM/Qwen-Agent",
    ),
    (
        "Pomerium ZTNA CDN",
        "https://www.pomerium.com",
    ),
    (
        "Innernet Mesh CDN",
        "https://github.com/tonari/innernet",
    ),
    (
        "Composio AI CDN",
        "https://github.com/ComposioHQ/composio",
    ),
    (
        "Mini-Omni2 AI CDN",
        "https://github.com/gpt-omni/mini-omni2",
    ),
    (
        "InternLM AI CDN",
        "https://github.com/InternLM/InternLM",
    ),
    (
        "OpenCodeInterpreter AI CDN",
        "https://github.com/OpenCodeInterpreter/OpenCodeInterpreter",
    ),
    (
        "Defguard VPN CDN",
        "https://defguard.net",
    ),
    (
        "Inlets Tunnel CDN",
        "https://inlets.dev",
    ),
    (
        "OpenCoder AI CDN",
        "https://github.com/OpenCoder-llm/OpenCoder-llm",
    ),
    (
        "Hunyuan3D AI CDN",
        "https://github.com/Tencent/Hunyuan3D-2",
    ),
    (
        "SmolVLM2 AI CDN",
        "https://github.com/huggingface/smolvlm",
    ),
    (
        "Codestral AI CDN",
        "https://mistral.ai/news/codestral/",
    ),
    (
        "zrok Tunnel CDN",
        "https://zrok.io",
    ),
    (
        "n2n Mesh VPN CDN",
        "https://github.com/ntop/n2n",
    ),
    (
        "Open-Deep-Research AI CDN",
        "https://github.com/huggingface/open-deep-research",
    ),
    (
        "DiffRhythm AI CDN",
        "https://github.com/ASLP-lab/DiffRhythm",
    ),
    (
        "Pixtral AI CDN",
        "https://mistral.ai/news/pixtral-12b/",
    ),
    (
        "MiniMax-01 AI CDN",
        "https://github.com/MiniMax-AI/MiniMax-01",
    ),
    (
        "Hyprspace Mesh CDN",
        "https://github.com/hyprspace/hyprspace",
    ),
    (
        "Fastd Tunnel CDN",
        "https://fastd.readthedocs.io",
    ),
    (
        "LeRobot AI CDN",
        "https://github.com/huggingface/lerobot",
    ),
    (
        "GOT-OCR AI CDN",
        "https://github.com/Ucas-HaoranWei/GOT-OCR2.0",
    ),
    (
        "Marco-o1 AI CDN",
        "https://github.com/AIDC-AI/Marco-o1",
    ),
    (
        "Megatron-LM AI CDN",
        "https://github.com/NVIDIA/Megatron-LM",
    ),
    (
        "EdgeVPN Mesh CDN",
        "https://github.com/mudler/edgevpn",
    ),
    (
        "boringproxy Tunnel CDN",
        "https://github.com/boringproxy/boringproxy",
    ),
    (
        "Step-Video-T2V AI CDN",
        "https://github.com/stepfun-ai/Step-Video-T2V",
    ),
    (
        "GLM-4-Voice AI CDN",
        "https://github.com/THUDM/GLM-4-Voice",
    ),
    (
        "SkyReels-V1 AI CDN",
        "https://github.com/SkyworkAI/SkyReels-V1",
    ),
    (
        "Emu3 AI CDN",
        "https://github.com/baaivision/Emu3",
    ),
    (
        "Ghostunnel TLS Proxy CDN",
        "https://github.com/ghostunnel/ghostunnel",
    ),
    (
        "FreeLAN Mesh VPN CDN",
        "https://github.com/freelan-developers/freelan",
    ),
    (
        "Kimi-k1.5 AI CDN",
        "https://github.com/MoonshotAI/Kimi-k1.5",
    ),
    (
        "Janus-Flow AI CDN",
        "https://github.com/deepseek-ai/Janus-Flow",
    ),
    (
        "Show-o AI CDN",
        "https://github.com/showlab/Show-o",
    ),
    (
        "FastVideo AI CDN",
        "https://github.com/hao-ai-lab/FastVideo",
    ),
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
        .user_agent(concat!("proxydoc/", env!("CARGO_PKG_VERSION"), " (https://github.com/TrustDec/proxydoc)"))
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
        .user_agent(concat!("proxydoc/", env!("CARGO_PKG_VERSION"), " (https://github.com/TrustDec/proxydoc)"))
        .timeout(Duration::from_secs(timeout_secs))
        .danger_accept_invalid_certs(false)
        .no_proxy()
        .build()?;

    let mut proxy_client_builder = reqwest::Client::builder()
        .user_agent(concat!("proxydoc/", env!("CARGO_PKG_VERSION"), " (https://github.com/TrustDec/proxydoc)"))
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
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "StepFun (Step-2) API"));
        assert!(standard.iter().any(|(name, _)| *name == "01.AI (Yi) API"));
        assert!(standard.iter().any(|(name, _)| *name == "Voyage AI API"));
        assert!(standard.iter().any(|(name, _)| *name == "Novita AI API"));
        assert!(standard.iter().any(|(name, _)| *name == "Hyperbolic API"));
        assert!(standard.iter().any(|(name, _)| *name == "Lambda Labs API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Upstage (Solar) API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Ai21 Labs (Jamba) API"));
        assert!(standard.iter().any(|(name, _)| *name == "Yarn Registry"));
        assert!(standard.iter().any(|(name, _)| *name == "PyPI Registry"));
        assert!(standard.iter().any(|(name, _)| *name == "Go Proxy"));
        assert!(standard.iter().any(|(name, _)| *name == "Docker Hub"));
        assert!(standard.iter().any(|(name, _)| *name == "Ollama"));
        assert!(standard.iter().any(|(name, _)| *name == "OpenRouter API"));
        assert!(standard.iter().any(|(name, _)| *name == "Groq API"));
        assert!(standard.iter().any(|(name, _)| *name == "Baseten API"));
        assert!(standard.iter().any(|(name, _)| *name == "FriendliAI API"));
        assert!(standard.iter().any(|(name, _)| *name == "Lepton AI API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Nebius AI Studio API"));
        assert!(standard.iter().any(|(name, _)| *name == "Chutes AI API"));
        assert!(standard.iter().any(|(name, _)| *name == "Infini-AI API"));
        assert!(standard.iter().any(|(name, _)| *name == "AIML API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Crusoe Cloud API"));
        assert!(standard.iter().any(|(name, _)| *name == "Modal"));
        assert!(standard.iter().any(|(name, _)| *name == "ElevenLabs API"));
        assert!(standard.iter().any(|(name, _)| *name == "Cartesia Voice AI API"));
        assert!(standard.iter().any(|(name, _)| *name == "Portkey AI Gateway"));
        assert!(standard.iter().any(|(name, _)| *name == "Langfuse API"));
        assert!(standard.iter().any(|(name, _)| *name == "Stackage (Haskell)"));
        assert!(standard.iter().any(|(name, _)| *name == "Hex.pm Registry"));
        assert!(standard.iter().any(|(name, _)| *name == "Tavily AI Search API"));
        assert!(standard.iter().any(|(name, _)| *name == "Exa AI Search API"));
        assert!(standard.iter().any(|(name, _)| *name == "Qdrant Cloud"));
        assert!(standard.iter().any(|(name, _)| *name == "AssemblyAI Speech API"));
        assert!(standard.iter().any(|(name, _)| *name == "Clojars (Clojure)"));
        assert!(standard.iter().any(|(name, _)| *name == "MetaCPAN (Perl)"));
        assert!(standard.iter().any(|(name, _)| *name == "Jina AI API"));
        assert!(standard.iter().any(|(name, _)| *name == "Firecrawl API"));
        assert!(standard.iter().any(|(name, _)| *name == "Weaviate Cloud"));
        assert!(standard.iter().any(|(name, _)| *name == "Chroma Vector DB"));
        assert!(standard.iter().any(|(name, _)| *name == "LuaRocks Registry"));
        assert!(standard.iter().any(|(name, _)| *name == "Perplexity AI API"));
        assert!(standard.iter().any(|(name, _)| *name == "Codestral (Mistral) API"));
        assert!(standard.iter().any(|(name, _)| *name == "Zilliz Cloud (Milvus)"));
        assert!(standard.iter().any(|(name, _)| *name == "Nimble Directory"));
        assert!(standard.iter().any(|(name, _)| *name == "Hex PM Registry (Erlang)"));
        assert!(standard.iter().any(|(name, _)| *name == "RubyGems Registry"));
        assert!(standard.iter().any(|(name, _)| *name == "Cursor AI API"));
        assert!(standard.iter().any(|(name, _)| *name == "LanceDB Cloud"));
        assert!(standard.iter().any(|(name, _)| *name == "LlamaIndex Cloud"));
        assert!(standard.iter().any(|(name, _)| *name == "Mem0 AI API"));
        assert!(standard.iter().any(|(name, _)| *name == "Supabase API"));
        assert!(standard.iter().any(|(name, _)| *name == "Dify Cloud API"));
        assert!(standard.iter().any(|(name, _)| *name == "Unstructured.io API"));
        assert!(standard.iter().any(|(name, _)| *name == "Upstash API"));
        assert!(standard.iter().any(|(name, _)| *name == "Vercel Cloud API"));
        assert!(standard.iter().any(|(name, _)| *name == "Resend API"));
        assert!(standard.iter().any(|(name, _)| *name == "Convex Cloud"));
        assert!(standard.iter().any(|(name, _)| *name == "Inngest API"));
        assert!(standard.iter().any(|(name, _)| *name == "Trigger.dev Cloud"));
        assert!(standard.iter().any(|(name, _)| *name == "Civitai API"));
        assert!(standard.iter().any(|(name, _)| *name == "Axiom API"));
        assert!(standard.iter().any(|(name, _)| *name == "Stripe API"));
        assert!(standard.iter().any(|(name, _)| *name == "Cloudflare API"));
        assert!(standard.iter().any(|(name, _)| *name == "Dub.co API"));
        assert!(standard.iter().any(|(name, _)| *name == "Koyeb Cloud API"));
        assert!(standard.iter().any(|(name, _)| *name == "Novu API"));
        assert!(standard.iter().any(|(name, _)| *name == "Linear API"));
        assert!(standard.iter().any(|(name, _)| *name == "Codeium API"));
        assert!(standard.iter().any(|(name, _)| *name == "Datadog API"));
        assert!(standard.iter().any(|(name, _)| *name == "Sanity API"));
        assert!(standard.iter().any(|(name, _)| *name == "Meilisearch Cloud"));
        assert!(standard.iter().any(|(name, _)| *name == "PlanetScale API"));
        assert!(standard.iter().any(|(name, _)| *name == "Better Stack API"));
        assert!(standard.iter().any(|(name, _)| *name == "Mintlify API"));
        assert!(standard.iter().any(|(name, _)| *name == "Val Town API"));
        assert!(standard.iter().any(|(name, _)| *name == "Hetzner Cloud API"));
        assert!(standard.iter().any(|(name, _)| *name == "GitLab API"));
        assert!(standard.iter().any(|(name, _)| *name == "DigitalOcean API"));
        assert!(standard.iter().any(|(name, _)| *name == "Netlify API"));
        assert!(standard.iter().any(|(name, _)| *name == "Scaleway Cloud API"));
        assert!(standard.iter().any(|(name, _)| *name == "Linode (Akamai) API"));
        assert!(standard.iter().any(|(name, _)| *name == "Bitbucket API"));
        assert!(standard.iter().any(|(name, _)| *name == "Vultr Cloud API"));
        assert!(standard.iter().any(|(name, _)| *name == "Fastly API"));
        assert!(standard.iter().any(|(name, _)| *name == "Gitee API"));
        assert!(standard.iter().any(|(name, _)| *name == "Deno Deploy API"));
        assert!(standard.iter().any(|(name, _)| *name == "Ollama Registry"));
        assert!(standard.iter().any(|(name, _)| *name == "GHCR (GitHub Packages)"));
        assert!(standard.iter().any(|(name, _)| *name == "GitHub Releases (Objects)"));
        assert!(standard.iter().any(|(name, _)| *name == "Tailscale API"));
        assert!(standard.iter().any(|(name, _)| *name == "Replit API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Rust Toolchain (rustup)"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "HashiCorp Releases"));
        assert!(standard.iter().any(|(name, _)| *name == "Alpine Linux CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Gradle Distributions"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Conan Center (JFrog)"));
        assert!(standard.iter().any(|(name, _)| *name == "PyPI Files CDN"));
        assert!(standard.iter().any(|(name, _)| *name == "Node.js Dist"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Homebrew Formulae"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Debian Archive CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Ubuntu Archive CDN"));
        assert!(standard.iter().any(|(name, _)| *name == "Flathub CDN"));
        assert!(standard.iter().any(|(name, _)| *name == "Snapcraft API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Fedora Mirror CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Arch Linux Mirrors"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "GitHub Actions Runners"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Amazon ECR Public"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Google Container Registry (GCR)"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Microsoft Container Registry (MCR)"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Rocky Linux CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Jenkins Update Center"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Buildkite API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "AlmaLinux CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Oracle Linux CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Woodpecker CI"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenSUSE CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Drone CI"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Codeberg API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Gitea Releases"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Void Linux CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "FreeBSD Pkg CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "CircleCI API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Azure DevOps API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Gentoo Linux CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenBSD CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "NixOS Binary Cache"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "JetBrains Marketplace & Plugins"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Atlassian API & Bitbucket"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "NetBSD CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "HashiCorp Terraform Cloud API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Concourse CI"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "GitLab Package Registry CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "JFrog Releases"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "DragonFly BSD CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Debian Archive CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Ubuntu Archive CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "New Relic Infrastructure Intake API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "New Relic Package CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Datadog Package CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Linux Mint Package CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Raspberry Pi OS Archive CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Elastic Package Registry CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Elastic Artifacts CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Grafana Labs Package CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenTelemetry CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Manjaro Linux Package CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Vector Package CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Cloudflare cdnjs CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "jsDelivr CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "unpkg CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Fluent Bit Package CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Fluentd Package CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Fastly CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Statically CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "InfluxData Package CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "InfluxData Cloud API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "esm.sh CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Skypack CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Prometheus CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Thanos Monitoring CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Cortex Metrics CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "PagerDuty Events API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpsGenie Alert API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "VictoriaMetrics CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Grafana Loki CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Jaeger Tracing CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Netdata Cloud API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Netdata Monitoring CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Elastic Cloud API"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Grafana Pyroscope CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenSearch CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenSearch Artifacts CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "ClickHouse CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Elastic Artifacts CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Zabbix CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Grafana Mimir CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Red Hat CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Rocky Linux CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "AlmaLinux CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Grafana Tempo CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Graylog CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "HashiCorp Consul CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "HashiCorp Vault CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenSearch Forum CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenNebula CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Cloudflare Workers CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Traefik Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Envoy Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Caddy Web Server CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Kong API Gateway CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "RabbitMQ CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Apache Kafka CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "HashiCorp Boundary CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "HashiCorp Packer CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Nginx CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "HAProxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Varnish Cache CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Apache Cassandra CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Apache Flink CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Apache Spark CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Apache Pulsar CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Apache Airflow CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "ScyllaDB CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Apache Iceberg CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Apache Arrow CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Apache Druid CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Apache Superset CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Trino CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "DuckDB CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Apache Doris CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Apache Hudi CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Polars CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Ray CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Apache APISIX CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Squid Cache CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Cilium CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Linkerd CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Istio CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Milvus CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Privoxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Tinyproxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Mitmproxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Tor Project CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "LangChain CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "vLLM CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Open WebUI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Dify CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Cloudflare Tunnel CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Ngrok Download CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "LiteLLM CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "CrewAI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "LangGraph CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Continue.dev CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Frp Release CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Tailscale Package CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Unsloth AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "ComfyUI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "AnythingLLM CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Windsurf AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "ZeroTier Network CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Gost Tunnel CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Promptfoo CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "CopilotKit CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Browserbase CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "DSPy AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Rathole Tunnel CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Chisel Tunnel CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenHands AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Pydantic AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Smolagents CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Mem0 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Nebula Network CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Bore Tunnel CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Crawl4AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Browser Use CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Agno AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "DeepEval CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Brook Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "NPC Tunnel CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Docling AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Mirascope AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Letta AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Pipecat AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Shadowsocks Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "V2Ray Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "FastRTC AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Stagehand AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "SGLang AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "LlamaParse AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Sing-box Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Hysteria Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Jan AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "MarkItDown AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "LM Studio AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Fish Audio CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Trojan Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "NaïveProxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Kokoro AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OmniParser AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Bolt.new AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Open-NotebookLM CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Tuic Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Juicity Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Open-R1 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "MCP Servers CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Qwen AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Moonshot Kimi AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Snell Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Leaf Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "DeepSeek-R1 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenManus AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Claude Code CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Mini-Omni AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Mihomo Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Stunnel Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "QwQ AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "RAGFlow AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "LobeChat AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Moshi AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Shadow-TLS Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Tun2socks Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Janus-Pro AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "SmolLM2 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "DeepEP AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Vanna AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "WSTunnel Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Cloak Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "DeepSeek-V3 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "UI-TARS AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "LightRAG AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Kimi-k1.5 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Wireproxy Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "MTG Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Qwen2.5-Coder AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "DeepSeek-VL2 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "SWE-agent AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Fish-Speech AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Kcptun Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Glider Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Qwen2.5-VL AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "HunyuanVideo AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Aider AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "MiniCPM AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Redsocks Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Dante Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Wan2.1 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "ChatTTS AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OmniGen AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Llama-3.3 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "3proxy Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Polipo Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "GLM-4 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "CosyVoice AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "LLaMA-Factory AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "FLUX AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Pproxy Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "MicroSOCKS Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "InternVL AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "CogVideoX AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "F5-TTS AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "SenseVoice AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Cntlm Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "COW Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "GraphRAG AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Moondream AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "FastEmbed AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "AutoGPT AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Sniproxy Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Dnsproxy Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "LLaVA AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Whisper AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Ovis AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Diffusers AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Mosdns Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "SmartDNS Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Transformers AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "PyTorch AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Gemma AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Cline AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "CoreDNS Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "AdGuard Home Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Llama.cpp AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Open-WebUI AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "AutoGen AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Torchtune AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "DNSCrypt-proxy Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Unbound Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Roo-Code AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Haystack AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Zed AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Plandex AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Dnsmasq Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Pi-hole Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "MetaGPT AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "ChatDev AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "FastGPT AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "BabyAGI AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Blocky Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Stubby Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Langflow AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Flowise AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "AgentGPT AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Dagger AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Knot Resolver Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "PowerDNS Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "LibreChat AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "LocalAI AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Tabby AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Khoj AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "dnsdist Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "BIND 9 Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Open-Interpreter AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Text-Gen-WebUI AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Fooocus AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "InvokeAI AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "NextDNS Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "pdnsd Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "AnythingLLM AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Continue AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "PrivateGPT AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "SearXNG AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "MaraDNS Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "i2pd Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Chatbox AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "SuperAGI AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "GPT-Engineer AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Phidata AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "dae Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Yggdrasil Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Devika AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "STORM AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Open-Sora AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "AgentOps AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "cjdns Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "tinc Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Instructor AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Phoenix AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "LitServe AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Swarm AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenVPN Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "SoftEther Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "SmolVLM AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Agent-Zero AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "TinyZero AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "FinRobot AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenConnect Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "strongSwan VPN CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "MinerU AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Pydantic-AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "KTransformers AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "FlashAttention AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "WireGuard VPN CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Headscale Mesh CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Cherry Studio AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Kotaemon RAG AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Marker PDF AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "PDFMathTranslate AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "NetBird Mesh CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Cloudflare WARP CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "DeepScaler AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Podcastfy AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenCompass AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "NanoFlow AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Amnezia VPN CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenZiti Mesh CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "DeepGEMM AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "DualPipe AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "EPLB AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "3FS AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Firezone ZTNA CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Netmaker Mesh CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Deep-Searcher AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Open-Thoughts AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Deep-Live-Cam AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "BitNet AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Pritunl VPN CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Twingate ZTNA CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "YuE AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Surya AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Step-Audio AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Skywork AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "EasyTier Mesh CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Teleport ZTNA CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Smolagents AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Open-R1 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Kokoro AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Qwen-Agent AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Pomerium ZTNA CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Innernet Mesh CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Composio AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Mini-Omni2 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "InternLM AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenCodeInterpreter AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Defguard VPN CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Inlets Tunnel CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "OpenCoder AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Hunyuan3D AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "SmolVLM2 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Codestral AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "zrok Tunnel CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "n2n Mesh VPN CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Open-Deep-Research AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "DiffRhythm AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Pixtral AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "MiniMax-01 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Hyprspace Mesh CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Fastd Tunnel CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "LeRobot AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "GOT-OCR AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Marco-o1 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Megatron-LM AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "EdgeVPN Mesh CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "boringproxy Tunnel CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Step-Video-T2V AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "GLM-4-Voice AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "SkyReels-V1 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Emu3 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Ghostunnel TLS Proxy CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "FreeLAN Mesh VPN CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Kimi-k1.5 AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Janus-Flow AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "Show-o AI CDN"));
        assert!(standard
            .iter()
            .any(|(name, _)| *name == "FastVideo AI CDN"));
        assert!(!standard.iter().any(|(name, _)| *name == "Custom Target"));

        let with_custom = build_target_list(Some("https://api.example.com"));
        assert!(with_custom
            .iter()
            .any(|(name, url)| *name == "Custom Target" && *url == "https://api.example.com"));
    }
}
