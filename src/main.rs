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
        /// Also configure git, cargo, npm, yarn, pnpm, bun, pip, conda, docker, gradle, and maven proxy
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
        /// Also unset git, cargo, npm, yarn, pnpm, bun, pip, conda, docker, gradle, and maven proxy
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
        Some(Commands::Test { proxy, direct, compare, json, target, timeout }) => {
            checker::run(proxy, direct, compare, json, target, timeout).await?;
        }
        Some(Commands::Scan { json }) => {
            scanner::run(json).await?;
        }
        Some(Commands::On { target, git, cargo, npm, yarn, pnpm, bun, pip, conda, docker, gradle, maven, all, shell }) => {
            manager::turn_on(&target, git, cargo, npm, yarn, pnpm, bun, pip, conda, docker, gradle, maven, all, shell.as_deref()).await?;
        }
        Some(Commands::Off { git, cargo, npm, yarn, pnpm, bun, pip, conda, docker, gradle, maven, all, shell }) => {
            manager::turn_off(git, cargo, npm, yarn, pnpm, bun, pip, conda, docker, gradle, maven, all, shell.as_deref())?;
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
        let cli = Cli::try_parse_from(["proxydoc", "test", "--proxy", "http://127.0.0.1:7890", "--json"]).unwrap();
        match cli.command {
            Some(Commands::Test { proxy, direct, compare, json, .. }) => {
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
            Some(Commands::Test { compare, direct, .. }) => {
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
            Some(Commands::Test { direct, compare, .. }) => {
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
            Some(Commands::On { target, all, pip, .. }) => {
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
            "proxydoc", "test", "-t", "https://api.deepseek.com/models", "--timeout", "10",
        ])
        .unwrap();
        match cli.command {
            Some(Commands::Test { target, timeout, .. }) => {
                assert_eq!(target, Some("https://api.deepseek.com/models".to_string()));
                assert_eq!(timeout, 10);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_on_docker_and_shell() {
        let cli = Cli::try_parse_from(["proxydoc", "on", "7897", "--docker", "--shell", "fish"]).unwrap();
        match cli.command {
            Some(Commands::On { target, docker, shell, .. }) => {
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
}
