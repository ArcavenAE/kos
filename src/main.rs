#![forbid(unsafe_code)]

use std::path::PathBuf;

use clap::Parser;

/// KOS — Knowledge Operating System
///
/// Graph-based knowledge accumulation for designed systems.
/// Reads typed YAML nodes, validates schema, renders graphs,
/// and detects drift across the knowledge substrate.
#[derive(Parser)]
#[command(name = "kos", version = env!("KOS_LONG_VERSION"), about)]
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

    /// Validate all nodes against the schema
    Validate {
        /// Validate all discovered graphs (orchestrator + includes)
        #[arg(long)]
        merged: bool,
    },

    /// Render the node graph as mermaid or dot
    Graph {
        /// Output format: mermaid (default) or dot
        #[arg(long, default_value = "mermaid")]
        format: String,

        /// Render all discovered graphs merged with subgraph clusters
        #[arg(long)]
        merged: bool,
    },

    /// List all discovered knowledge graphs
    Graphs,

    /// Initialize a kos knowledge graph in the current directory
    Init {
        /// Graph scope: repo (default) or orchestrator
        #[arg(long, default_value = "repo")]
        scope: String,

        /// Graph ID (defaults to directory name)
        #[arg(long)]
        id: Option<String>,

        /// Update an existing _kos/ installation
        #[arg(long)]
        update: bool,
    },

    /// Detect drift — content changes and stale dependents
    Drift,

    /// Scan a repo for knowledge artifacts and SDD systems
    Seed {
        /// Subcommand: scan (default)
        #[arg(default_value = "scan")]
        action: String,

        /// Output as JSONL instead of human-readable text
        #[arg(long)]
        json: bool,

        /// Target directory (defaults to cwd)
        #[arg(long)]
        dir: Option<PathBuf>,
    },

    /// Extract RD findings from sprint/rd/ briefs into structured format
    Bridge {
        /// Output as JSONL instead of human-readable text
        #[arg(long)]
        json: bool,
    },

    /// Check the health of a kos knowledge graph
    Doctor {
        /// Check all discovered graphs (orchestrator + includes)
        #[arg(long)]
        merged: bool,

        /// Auto-fix safe issues (missing dirs, schema version)
        #[arg(long)]
        fix: bool,
    },

    /// Create an idea file — pre-hypothesis brainstorming
    Idea {
        /// Slug for the idea filename (e.g., "my-observation")
        slug: String,

        /// Optional title (defaults to slug)
        #[arg(long)]
        title: Option<String>,
    },

    /// Create a frontier question node — the unit of work
    Question {
        /// Slug for the question (e.g., "charter-scaling" → question-charter-scaling)
        slug: String,

        /// Title for the question node
        title: String,
    },

    /// Create a finding — probe result with auto-numbered ID
    Finding {
        /// Slug for the finding (e.g., "charter-inflation")
        slug: String,

        /// Title for the finding
        title: String,
    },

    /// Create an exploration brief — plan for a probe
    Probe {
        /// Slug for the brief (e.g., "substrate" → brief-substrate)
        slug: String,

        /// Title for the probe brief
        title: String,
    },

    /// Update kos to the latest release, or to a specific version
    Update {
        /// Target version tag (e.g., alpha-20260405-075244-abc1234). Omit for latest.
        version: Option<String>,
    },

    /// Show version, commit, build time, and channel
    Version,
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

            let workspace = kos::workspace::Workspace::discover(&cwd).or_else(|discover_err| {
                if let Some(ref ws_path) = workspace_path {
                    kos::workspace::Workspace::from_explicit(ws_path)
                } else {
                    Err(discover_err)
                }
            })?;

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

        Commands::Validate { merged } => {
            let cwd = std::env::current_dir()?;
            let workspace = kos::workspace::Workspace::discover(&cwd)?;

            if merged {
                // Validate all discovered graphs
                let mut total_errors = 0;
                for graph in &workspace.graphs {
                    eprintln!("Validating graph: {} ({})", graph.graph_id, graph.scope);
                    if let Err(e) = kos::validate::run(&graph.path) {
                        eprintln!("  error: {e}");
                        total_errors += 1;
                    }
                }
                // Also validate legacy layout if no _kos/ graphs found
                if workspace.graphs.is_empty() {
                    kos::validate::run(&workspace.kos_root)?;
                }
                if total_errors > 0 {
                    std::process::exit(1);
                }
            } else {
                // Validate nearest graph or legacy layout
                let node_root = workspace.node_root();
                kos::validate::run(&node_root)?;
            }
        }

        Commands::Graph { format, merged } => {
            let cwd = std::env::current_dir()?;
            let workspace = kos::workspace::Workspace::discover(&cwd)?;
            let fmt = match format.as_str() {
                "dot" => kos::graph::GraphFormat::Dot,
                _ => kos::graph::GraphFormat::Mermaid,
            };

            if merged {
                for graph in &workspace.graphs {
                    eprintln!("--- graph: {} ({}) ---", graph.graph_id, graph.scope);
                    if let Err(e) = kos::graph::run(&graph.path, fmt) {
                        eprintln!("  error: {e}");
                    }
                }
                if workspace.graphs.is_empty() {
                    kos::graph::run(&workspace.kos_root, fmt)?;
                }
            } else {
                let node_root = workspace.node_root();
                kos::graph::run(&node_root, fmt)?;
            }
        }

        Commands::Drift => {
            let cwd = std::env::current_dir()?;
            let workspace = kos::workspace::Workspace::discover(&cwd)?;
            let node_root = workspace.node_root();
            kos::drift::run(&node_root)?;
        }

        Commands::Seed { action, json, dir } => match action.as_str() {
            "scan" => {
                let target = dir.unwrap_or_else(|| {
                    std::env::current_dir().expect("cannot determine current directory")
                });
                let result = kos::seed::scan(&target)?;
                if json {
                    kos::seed::print_jsonl(&result);
                } else {
                    kos::seed::print_human(&result);
                }
            }
            other => {
                eprintln!("unknown seed action: {other}");
                eprintln!("available: scan");
                std::process::exit(1);
            }
        },

        Commands::Bridge { json } => {
            let cwd = std::env::current_dir()?;
            let workspace = kos::workspace::Workspace::discover(&cwd)?;
            kos::bridge::run(&workspace, json)?;
        }

        Commands::Graphs => {
            let cwd = std::env::current_dir()?;
            let workspace = kos::workspace::Workspace::discover(&cwd)?;

            if workspace.graphs.is_empty() {
                println!(
                    "No _kos/ graphs discovered. Using legacy layout at {}",
                    workspace.kos_root.display()
                );
                println!("Run `kos init` to create a _kos/ graph.");
            } else {
                println!("{:<16} {:<14} {:<30} NODES", "GRAPH_ID", "SCOPE", "PATH");
                for graph in &workspace.graphs {
                    let node_count = count_nodes(&graph.path);
                    let rel_path = graph
                        .path
                        .strip_prefix(&workspace.root)
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| graph.path.display().to_string());
                    println!(
                        "{:<16} {:<14} {:<30} {}",
                        graph.graph_id, graph.scope, rel_path, node_count
                    );
                }
            }
        }

        Commands::Init { scope, id, update } => {
            let cwd = std::env::current_dir()?;
            let scope = match scope.as_str() {
                "orchestrator" => kos::model::GraphScope::Orchestrator,
                _ => kos::model::GraphScope::Repo,
            };
            kos::init::run(&cwd, &scope, id, update)?;
        }

        Commands::Doctor { merged, fix } => {
            let cwd = std::env::current_dir()?;
            let workspace = kos::workspace::Workspace::discover(&cwd)?;
            kos::doctor::run(&workspace, &cwd, merged, fix)?;
        }

        Commands::Idea { slug, title } => {
            let cwd = std::env::current_dir()?;
            let workspace = kos::workspace::Workspace::discover(&cwd)?;
            kos::process::idea(&workspace, &cwd, &slug, title.as_deref())?;
        }

        Commands::Question { slug, title } => {
            let cwd = std::env::current_dir()?;
            let workspace = kos::workspace::Workspace::discover(&cwd)?;
            kos::process::question(&workspace, &cwd, &slug, &title)?;
        }

        Commands::Finding { slug, title } => {
            let cwd = std::env::current_dir()?;
            let workspace = kos::workspace::Workspace::discover(&cwd)?;
            kos::process::finding(&workspace, &cwd, &slug, &title)?;
        }

        Commands::Probe { slug, title } => {
            let cwd = std::env::current_dir()?;
            let workspace = kos::workspace::Workspace::discover(&cwd)?;
            kos::process::probe(&workspace, &cwd, &slug, &title)?;
        }

        Commands::Update { version } => {
            let method = kos::updater::detect_install_method()?;
            match method {
                kos::updater::InstallMethod::Homebrew => {
                    let formula = kos::updater::brew_formula_name();
                    println!("kos was installed via Homebrew. Run: brew upgrade {formula}");
                    return Ok(());
                }
                kos::updater::InstallMethod::LinuxPackageManager { manager } => {
                    println!(
                        "kos was installed via {manager}. Use your package manager to update."
                    );
                    return Ok(());
                }
                kos::updater::InstallMethod::DirectBinary => {}
            }

            let current_tag = env!("KOS_TAG");
            let target = version.as_deref();

            match kos::updater::check_for_update(target)? {
                Some(tag) if tag == current_tag => {
                    println!("Already up to date: {current_tag}");
                }
                Some(tag) => {
                    println!("Current: {current_tag}");
                    if target.is_some() {
                        println!("Target:  {tag}");
                    } else {
                        println!("Latest:  {tag}");
                    }
                    let new_tag = kos::updater::download_and_install(target)?;
                    println!("Updated to {new_tag}. Restart kos to use the new version.");
                }
                None => {
                    if let Some(v) = target {
                        println!("No release matching '{v}' found.");
                    } else {
                        println!("No releases found.");
                    }
                }
            }
        }

        Commands::Version => {
            println!("kos {}", env!("KOS_LONG_VERSION"));
            println!("  version:    {}", env!("KOS_VERSION"));
            println!("  commit:     {}", env!("KOS_COMMIT"));
            println!("  build time: {}", env!("KOS_BUILD_TIME"));
            println!("  channel:    {}", env!("KOS_CHANNEL"));
        }
    }

    Ok(())
}

/// Count node YAML files in a _kos/ graph directory.
fn count_nodes(kos_dir: &std::path::Path) -> usize {
    let nodes_dir = kos_dir.join("nodes");
    if !nodes_dir.exists() {
        return 0;
    }
    walkdir::WalkDir::new(nodes_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
        })
        .count()
}
