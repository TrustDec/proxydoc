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
    version = "0.2.0",
    about = "🩺 Deep developer proxy diagnostics, DNS poisoning checker & one-click auto-repair CLI.",
    long_about = "proxydoc is a modern, lightweight CLI tool to diagnose root causes of broken developer network environments, detect DNS poisoning on GitHub/AI APIs, sync system proxies to terminal, and repair broken configurations automatically."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 🩺 Deep diagnostics (System vs Terminal sync, DNS poisoning, Git/NPM dead ports)
    Doctor {
        /// Show verbose diagnostic details
        #[arg(short, long)]
        verbose: bool,
    },
    /// 🩹 One-click automatic repair for detected network and proxy issues
    Fix,
    /// ⚡ Concurrently test latency to GitHub, OpenAI, Anthropic, Google, HuggingFace
    Test {
        /// Custom proxy URL to test (e.g. http://127.0.0.1:7890 or socks5://127.0.0.1:1082)
        #[arg(short, long)]
        proxy: Option<String>,
    },
    /// 🔍 Scan active local proxy daemon ports (Clash, Shadowrocket, Sing-box, V2Ray, CPA)
    Scan,
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
    },
    /// 🔴 Output shell unset commands to turn OFF proxy for current session
    Off {
        /// Also unset git global proxy
        #[arg(long)]
        git: bool,
        /// Also unset npm proxy
        #[arg(long)]
        npm: bool,
    },
    /// 🐚 Print shell integration helper script for eval: `eval "$(proxydoc init)"`
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            doctor::run(false).await?;
        }
        Some(Commands::Doctor { verbose }) => {
            doctor::run(verbose).await?;
        }
        Some(Commands::Fix) => {
            fixer::run().await?;
        }
        Some(Commands::Test { proxy }) => {
            checker::run(proxy).await?;
        }
        Some(Commands::Scan) => {
            scanner::run().await?;
        }
        Some(Commands::On { target, git, npm }) => {
            manager::turn_on(&target, git, npm).await?;
        }
        Some(Commands::Off { git, npm }) => {
            manager::turn_off(git, npm)?;
        }
        Some(Commands::Init) => {
            manager::print_init_script();
        }
    }

    Ok(())
}
