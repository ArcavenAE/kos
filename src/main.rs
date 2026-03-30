#![forbid(unsafe_code)]

use clap::Parser;

/// KOS — Knowledge Operating System
///
/// Graph-based knowledge accumulation for designed systems.
/// Reads typed YAML nodes, validates schema, renders graphs,
/// and detects drift across the knowledge substrate.
#[derive(Parser)]
#[command(name = "kos", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Show relevant knowledge for the current directory
    Orient {
        /// Target repo name (inferred from cwd if omitted)
        target: Option<String>,

        /// Output as JSONL instead of human-readable text
        #[arg(long)]
        json: bool,

        /// Append usage metrics to ~/.local/share/kos/orient.jsonl
        #[arg(long)]
        log: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Orient { target, json, log } => {
            let cwd = std::env::current_dir()?;
            let workspace = kos::workspace::Workspace::discover(&cwd)?;

            let target = target
                .or_else(|| workspace.infer_target(&cwd))
                .unwrap_or_else(|| "kos".to_string());

            kos::orient::run(&workspace, &target, json, log)?;
        }
    }

    Ok(())
}
