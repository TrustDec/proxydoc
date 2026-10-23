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
    /// 🩺 Deep diagnostics (System vs Terminal sync, DNS poisoning, Git/NPM/Cargo/Pip/cURL dead ports)
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
    /// ⚡ Concurrently test latency to GitHub, OpenAI, Anthropic, Google, HuggingFace & inspect Egress IP
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
        /// Also configure npm proxy
        #[arg(long)]
        npm: bool,
        /// Also configure pip global proxy
        #[arg(long)]
        pip: bool,
        /// Also configure git, npm, and pip proxy
        #[arg(short, long)]
        all: bool,
    },
    /// 🔴 Output shell unset commands to turn OFF proxy for current session
    Off {
        /// Also unset git global proxy
        #[arg(long)]
        git: bool,
        /// Also unset npm proxy
        #[arg(long)]
        npm: bool,
        /// Also unset pip global proxy
        #[arg(long)]
        pip: bool,
        /// Also unset git, npm, and pip proxy
        #[arg(short, long)]
        all: bool,
    },
    /// 🐚 Print shell integration helper script for eval: `eval "$(proxydoc init)"`
    Init,
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
        Some(Commands::Test { proxy, direct, compare, json }) => {
            checker::run(proxy, direct, compare, json).await?;
        }
        Some(Commands::Scan { json }) => {
            scanner::run(json).await?;
        }
        Some(Commands::On { target, git, npm, pip, all }) => {
            manager::turn_on(&target, git, npm, pip, all).await?;
        }
        Some(Commands::Off { git, npm, pip, all }) => {
            manager::turn_off(git, npm, pip, all)?;
        }
        Some(Commands::Init) => {
            manager::print_init_script();
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
            Some(Commands::Test { proxy, direct, compare, json }) => {
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
    fn test_cli_parse_off_all() {
        let cli = Cli::try_parse_from(["proxydoc", "off", "-a"]).unwrap();
        match cli.command {
            Some(Commands::Off { all, .. }) => {
                assert!(all);
            }
            _ => panic!("Expected Off command"),
        }
    }
}
