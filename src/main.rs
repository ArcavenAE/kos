#![forbid(unsafe_code)]

use std::path::PathBuf;

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

        /// Path to the aae-orc workspace root (env: KOS_WORKSPACE)
        #[arg(long, env = "KOS_WORKSPACE")]
        workspace: Option<PathBuf>,

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
        Commands::Orient {
            target,
            workspace: workspace_path,
            json,
            log,
        } => {
            let cwd = std::env::current_dir()?;

            // 1. Try auto-discovery from cwd
            // 2. Fall back to --workspace / KOS_WORKSPACE
            let workspace = kos::workspace::Workspace::discover(&cwd).or_else(|discover_err| {
                if let Some(ref ws_path) = workspace_path {
                    kos::workspace::Workspace::from_explicit(ws_path)
                } else {
                    Err(discover_err)
                }
            })?;

            // Infer target: explicit > workspace-relative > cwd directory name > "kos"
            let target = target
                .or_else(|| workspace.infer_target(&cwd))
                .or_else(|| {
                    cwd.file_name()
                        .and_then(|n| n.to_str())
                        .map(ToString::to_string)
                })
                .unwrap_or_else(|| "kos".to_string());

            kos::orient::run(&workspace, &target, json, log)?;
        }
    }

    Ok(())
}
