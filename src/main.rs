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
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Orient { target, json: _ } => {
            let target = target.as_deref().unwrap_or("(current directory)");
            println!("kos orient: {target}");
            println!("(not yet implemented — session-006 probe)");
        }
    }

    Ok(())
}
