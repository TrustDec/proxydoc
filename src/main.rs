use anyhow::Result;
use clap::{Parser, Subcommand};

mod checker;
mod doctor;
mod manager;
mod scanner;

#[derive(Parser)]
#[command(
    name = "proxydoc",
    author = "TrustDec <https://github.com/TrustDec>",
    version,
    about = "🩺 All-in-one terminal & dev tools proxy diagnostics and manager for developers.",
    long_about = "proxydoc is a modern, lightweight CLI tool to diagnose, test, and manage proxy settings across all developer toolchains (Shell, Git, NPM, Homebrew, Docker, AI APIs)."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 🩺 Run full system proxy diagnostic (Shell, Git, NPM, Docker, Homebrew, listening ports)
    Doctor {
        /// Show verbose diagnostic details
        #[arg(short, long)]
        verbose: bool,
    },
    /// ⚡ Test connection and latency to GitHub, Google, Anthropic, OpenAI, HuggingFace
    Test {
        /// Custom proxy URL to test (e.g. http://127.0.0.1:7890 or socks5://127.0.0.1:1080)
        #[arg(short, long)]
        proxy: Option<String>,
    },
    /// 🔍 Scan active local proxy client ports (Clash, Shadowrocket, Sing-box, V2Ray, CPA)
    Scan,
    /// 🟢 Output shell export commands to turn ON proxy for current session
    On {
        /// Proxy port or URL (defaults to detected active proxy port or 7890)
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
        Some(Commands::Test { proxy }) => {
            checker::run(proxy).await?;
        }
        Some(Commands::Scan) => {
            scanner::run().await?;
        }
        Some(Commands::On { target, git, npm }) => {
            manager::turn_on(&target, git, npm)?;
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
