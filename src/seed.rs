#![forbid(unsafe_code)]

//! Seed scan: inventory a repo's knowledge artifacts for graph seeding.
//!
//! Two layers of detection:
//! 1. Built-in universal conventions (ADRs, README, CLAUDE.md, CI/CD, etc.)
//! 2. SDD system fingerprints (bmad, speckit, chainlink, beads, etc.)
//!
//! Outputs an annotated manifest for agent consumption (human-readable
//! or JSONL). Bridge profiles from sideshow extend detection for
//! system-specific artifact semantics.

use std::path::{Path, PathBuf};

use crate::error::Result;

/// A detected artifact in the repo.
#[derive(Debug)]
pub struct DetectedArtifact {
    /// Relative path from repo root.
    pub path: String,
    /// What kind of artifact this is.
    pub kind: ArtifactKind,
    /// Which system it belongs to (if detected).
    pub system: Option<String>,
    /// What kos node type it would map to.
    pub maps_to: Option<String>,
    /// Additional context for the agent.
    pub note: Option<String>,
}

/// Categories of detected artifacts.
#[derive(Debug, Clone)]
pub enum ArtifactKind {
    /// Architecture Decision Record
    Adr,
    /// README, CONTRIBUTING, etc.
    ProjectDoc,
    /// CHANGELOG, release notes
    Changelog,
    /// CLAUDE.md, .claude/, SOUL.md, charter.md
    AgentConfig,
    /// CI/CD pipeline definitions
    CiCd,
    /// Build system (justfile, Makefile, etc.)
    BuildSystem,
    /// Documentation tree
    DocsTree,
    /// Test directory
    TestDir,
    /// SDD system installation
    SddSystem,
    /// Knowledge graph (_kos/)
    KosGraph,
    /// Source code root
    SourceDir,
    /// Configuration file
    Config,
    /// License
    License,
}

impl std::fmt::Display for ArtifactKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactKind::Adr => write!(f, "adr"),
            ArtifactKind::ProjectDoc => write!(f, "project-doc"),
            ArtifactKind::Changelog => write!(f, "changelog"),
            ArtifactKind::AgentConfig => write!(f, "agent-config"),
            ArtifactKind::CiCd => write!(f, "ci-cd"),
            ArtifactKind::BuildSystem => write!(f, "build-system"),
            ArtifactKind::DocsTree => write!(f, "docs-tree"),
            ArtifactKind::TestDir => write!(f, "test-dir"),
            ArtifactKind::SddSystem => write!(f, "sdd-system"),
            ArtifactKind::KosGraph => write!(f, "kos-graph"),
            ArtifactKind::SourceDir => write!(f, "source-dir"),
            ArtifactKind::Config => write!(f, "config"),
            ArtifactKind::License => write!(f, "license"),
        }
    }
}

/// A detected SDD system with version info.
#[derive(Debug)]
pub struct DetectedSystem {
    pub name: String,
    pub version: Option<String>,
    pub fingerprint: String,
    pub note: String,
}

/// Git repository statistics.
#[derive(Debug)]
pub struct GitStats {
    pub commit_count: u64,
    pub contributor_count: u64,
    pub age_days: u64,
    pub branch_count: u64,
    pub default_branch: String,
}

/// Full scan result.
#[derive(Debug)]
pub struct ScanResult {
    pub root: PathBuf,
    pub systems: Vec<DetectedSystem>,
    pub artifacts: Vec<DetectedArtifact>,
    pub git: Option<GitStats>,
}

/// Run the seed scan against a directory.
pub fn scan(root: &Path) -> Result<ScanResult> {
    let mut artifacts = Vec::new();
    let mut systems = Vec::new();

    // Detect SDD systems
    detect_systems(root, &mut systems);

    // Detect universal conventions
    detect_universal(root, &mut artifacts);

    // Detect source and test directories
    detect_source_dirs(root, &mut artifacts);

    // Git stats
    let git = collect_git_stats(root);

    Ok(ScanResult {
        root: root.to_path_buf(),
        systems,
        artifacts,
        git,
    })
}

/// Print scan results as human-readable text.
pub fn print_human(result: &ScanResult) {
    println!(
        "=== kos seed scan: {} ===\n",
        result
            .root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(".")
    );

    if !result.systems.is_empty() {
        println!("## SDD Systems Detected\n");
        for sys in &result.systems {
            print!("  {} ", sys.name);
            if let Some(ref v) = sys.version {
                print!("(v{v}) ");
            }
            println!("— {}", sys.note);
            println!("    fingerprint: {}", sys.fingerprint);
        }
        println!();
    }

    if !result.artifacts.is_empty() {
        println!("## Artifacts\n");
        for art in &result.artifacts {
            print!("  [{}] {}", art.kind, art.path);
            if let Some(ref sys) = art.system {
                print!("  ({sys})");
            }
            if let Some(ref maps) = art.maps_to {
                print!("  → {maps}");
            }
            println!();
            if let Some(ref note) = art.note {
                println!("    {note}");
            }
        }
        println!();
    }

    if let Some(ref git) = result.git {
        println!("## Git Stats\n");
        println!("  commits:      {}", git.commit_count);
        println!("  contributors: {}", git.contributor_count);
        println!("  age:          {} days", git.age_days);
        println!("  branches:     {}", git.branch_count);
        println!("  default:      {}", git.default_branch);
        println!();
    }

    let total = result.artifacts.len() + result.systems.len();
    if total == 0 {
        println!("  (no artifacts detected)");
        println!("  Hint: this may be a new project, or artifacts use unfamiliar conventions.");
    }
}

/// Print scan results as JSONL.
pub fn print_jsonl(result: &ScanResult) {
    let root_name = result
        .root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(".");

    // Meta line
    let meta = serde_json::json!({
        "type": "scan_meta",
        "root": root_name,
        "system_count": result.systems.len(),
        "artifact_count": result.artifacts.len(),
        "has_git": result.git.is_some(),
    });
    println!("{meta}");

    // Systems
    for sys in &result.systems {
        let json = serde_json::json!({
            "type": "system",
            "name": sys.name,
            "version": sys.version,
            "fingerprint": sys.fingerprint,
            "note": sys.note,
        });
        println!("{json}");
    }

    // Artifacts
    for art in &result.artifacts {
        let json = serde_json::json!({
            "type": "artifact",
            "path": art.path,
            "kind": art.kind.to_string(),
            "system": art.system,
            "maps_to": art.maps_to,
            "note": art.note,
        });
        println!("{json}");
    }

    // Git stats
    if let Some(ref git) = result.git {
        let json = serde_json::json!({
            "type": "git_stats",
            "commit_count": git.commit_count,
            "contributor_count": git.contributor_count,
            "age_days": git.age_days,
            "branch_count": git.branch_count,
            "default_branch": git.default_branch,
        });
        println!("{json}");
    }
}

// ── SDD System Detection ──────────────────────────────────────

fn detect_systems(root: &Path, systems: &mut Vec<DetectedSystem>) {
    // Each system has a unique directory fingerprint.
    // Order: most specific first.

    // bmad
    let bmad_manifest = root.join("_bmad/_config/manifest.yaml");
    if bmad_manifest.exists() {
        let version = parse_bmad_version(&bmad_manifest);
        systems.push(DetectedSystem {
            name: "bmad".to_string(),
            version,
            fingerprint: "_bmad/_config/manifest.yaml".to_string(),
            note: "BMAD Method — agents, workflows, tasks".to_string(),
        });
    }

    // speckit
    let speckit_opts = root.join(".specify/init-options.json");
    if speckit_opts.exists() {
        let version = parse_speckit_version(&speckit_opts);
        systems.push(DetectedSystem {
            name: "speckit".to_string(),
            version,
            fingerprint: ".specify/init-options.json".to_string(),
            note: "Spec-Driven Development pipeline — specify, plan, tasks, implement".to_string(),
        });
    } else if root.join(".specify").is_dir() {
        systems.push(DetectedSystem {
            name: "speckit".to_string(),
            version: None,
            fingerprint: ".specify/".to_string(),
            note: "Spec-Driven Development pipeline (no init-options found)".to_string(),
        });
    }

    // chainlink
    if root.join(".chainlink/issues.db").exists() || root.join(".chainlink").is_dir() {
        systems.push(DetectedSystem {
            name: "chainlink".to_string(),
            version: None,
            fingerprint: ".chainlink/".to_string(),
            note: "Hierarchical issue tracker for AI agents (VSDD)".to_string(),
        });
    }

    // beads
    if root.join(".beads").is_dir() {
        systems.push(DetectedSystem {
            name: "beads".to_string(),
            version: None,
            fingerprint: ".beads/".to_string(),
            note: "Version-controlled issue tracking (Dolt)".to_string(),
        });
    }

    // multiclaude
    if root.join(".multiclaude").is_dir() {
        systems.push(DetectedSystem {
            name: "multiclaude".to_string(),
            version: None,
            fingerprint: ".multiclaude/".to_string(),
            note: "Multi-agent Claude orchestration".to_string(),
        });
    }

    // OpenClaudia
    if root.join(".openclaudia/config.yaml").exists() || root.join(".openclaudia").is_dir() {
        systems.push(DetectedSystem {
            name: "openclaudia".to_string(),
            version: None,
            fingerprint: ".openclaudia/".to_string(),
            note: "Claude wrapper with memory, plugins, VDD integration".to_string(),
        });
    }

    // gastown
    if root.join(".gastown/config.toml").exists() || root.join(".gastown").is_dir() {
        systems.push(DetectedSystem {
            name: "gastown".to_string(),
            version: None,
            fingerprint: ".gastown/".to_string(),
            note: "Town workspace — rigs, crews, formulas, convoys".to_string(),
        });
    }

    // VSDD Factory
    if root.join(".factory/STATE.md").exists() {
        systems.push(DetectedSystem {
            name: "vsdd-factory".to_string(),
            version: None,
            fingerprint: ".factory/STATE.md".to_string(),
            note: "Verified Spec-Driven Development — L1-L4 specs, behavioral contracts, verification properties".to_string(),
        });
    } else if root.join(".factory").is_dir() {
        systems.push(DetectedSystem {
            name: "vsdd-factory".to_string(),
            version: None,
            fingerprint: ".factory/".to_string(),
            note: "VSDD pipeline directory (no STATE.md found)".to_string(),
        });
    }

    // pennyfarthing
    if root.join(".pennyfarthing").is_dir() {
        systems.push(DetectedSystem {
            name: "pennyfarthing".to_string(),
            version: None,
            fingerprint: ".pennyfarthing/".to_string(),
            note: "Legacy AI console framework (predecessor to aclaude)".to_string(),
        });
    }

    // spectacle (provider repo has spectacle.yaml at root)
    if root.join("spectacle.yaml").exists() {
        let version = parse_spectacle_version(&root.join("spectacle.yaml"));
        systems.push(DetectedSystem {
            name: "spectacle".to_string(),
            version,
            fingerprint: "spectacle.yaml".to_string(),
            note: "IEEE/ISO standards-based spec templates".to_string(),
        });
    }
    // spectacle consumer (has commands installed)
    else if root.join(".claude/commands/spectacle").is_dir() {
        systems.push(DetectedSystem {
            name: "spectacle".to_string(),
            version: None,
            fingerprint: ".claude/commands/spectacle/".to_string(),
            note: "IEEE/ISO spec templates (installed as commands)".to_string(),
        });
    }

    // kos
    let kos_manifest = root.join("_kos/kos.yaml");
    if kos_manifest.exists() {
        systems.push(DetectedSystem {
            name: "kos".to_string(),
            version: None,
            fingerprint: "_kos/kos.yaml".to_string(),
            note: "Knowledge graph — nodes, findings, probes, ideas".to_string(),
        });
    }
}

// ── Universal Convention Detection ────────────────────────────

fn detect_universal(root: &Path, artifacts: &mut Vec<DetectedArtifact>) {
    // ADR directories
    for adr_dir in &[
        "decisions",
        "docs/adr",
        "docs/decisions",
        "docs/adrs",
        "adr",
    ] {
        let path = root.join(adr_dir);
        if path.is_dir() {
            let count = count_files(&path, &["md"]);
            artifacts.push(DetectedArtifact {
                path: adr_dir.to_string(),
                kind: ArtifactKind::Adr,
                system: None,
                maps_to: Some("element/graveyard".to_string()),
                note: Some(format!("{count} markdown files")),
            });
        }
    }

    // ThreeDoors BOARD pattern
    if root.join("docs/decisions/BOARD.md").exists() {
        artifacts.push(DetectedArtifact {
            path: "docs/decisions/BOARD.md".to_string(),
            kind: ArtifactKind::Adr,
            system: Some("threedoors".to_string()),
            maps_to: Some("element + graveyard (decisions with rejected alternatives)".to_string()),
            note: Some("Architectural decision log with adopted AND rejected options".to_string()),
        });
    }

    // Project docs
    for (file, maps_to) in &[
        ("README.md", "element (project overview)"),
        ("CONTRIBUTING.md", "element (process)"),
        ("CODE_OF_CONDUCT.md", "value"),
        ("SECURITY.md", "element (security policy)"),
        ("SOUL.md", "value (design principles)"),
        ("ROADMAP.md", "question + element (backlog + completed)"),
    ] {
        if root.join(file).exists() {
            artifacts.push(DetectedArtifact {
                path: file.to_string(),
                kind: ArtifactKind::ProjectDoc,
                system: None,
                maps_to: Some(maps_to.to_string()),
                note: None,
            });
        }
    }

    // Charter
    for charter in &["charter.md", "KOS-charter.md"] {
        if root.join(charter).exists() {
            artifacts.push(DetectedArtifact {
                path: charter.to_string(),
                kind: ArtifactKind::ProjectDoc,
                system: None,
                maps_to: Some(
                    "bedrock + frontier + graveyard (re-introduction document)".to_string(),
                ),
                note: None,
            });
        }
    }

    // Changelog
    for cl in &["CHANGELOG.md", "HISTORY.md", "CHANGES.md"] {
        if root.join(cl).exists() {
            artifacts.push(DetectedArtifact {
                path: cl.to_string(),
                kind: ArtifactKind::Changelog,
                system: None,
                maps_to: Some("element (version history)".to_string()),
                note: None,
            });
        }
    }

    // License
    for lic in &["LICENSE", "LICENSE.md", "LICENSE.txt"] {
        if root.join(lic).exists() {
            artifacts.push(DetectedArtifact {
                path: lic.to_string(),
                kind: ArtifactKind::License,
                system: None,
                maps_to: None,
                note: None,
            });
        }
    }

    // Agent config
    if root.join("CLAUDE.md").exists() {
        artifacts.push(DetectedArtifact {
            path: "CLAUDE.md".to_string(),
            kind: ArtifactKind::AgentConfig,
            system: None,
            maps_to: Some("element (agent instructions)".to_string()),
            note: None,
        });
    }
    if root.join(".claude").is_dir() {
        let mut parts = Vec::new();
        if root.join(".claude/settings.json").exists() {
            parts.push("settings");
        }
        if root.join(".claude/commands").is_dir() {
            let count = count_files(&root.join(".claude/commands"), &["md"]);
            parts.push("commands");
            if count > 0 {
                artifacts.push(DetectedArtifact {
                    path: ".claude/commands/".to_string(),
                    kind: ArtifactKind::AgentConfig,
                    system: None,
                    maps_to: None,
                    note: Some(format!("{count} slash commands")),
                });
            }
        }
        if root.join(".claude/rules").is_dir() {
            parts.push("rules");
        }
        if !parts.is_empty() {
            artifacts.push(DetectedArtifact {
                path: ".claude/".to_string(),
                kind: ArtifactKind::AgentConfig,
                system: None,
                maps_to: None,
                note: Some(format!("contains: {}", parts.join(", "))),
            });
        }
    }
    // Other agent tool configs
    for (dir, name) in &[
        (".cursor", "Cursor"),
        (".gemini", "Gemini"),
        (".windsurf", "Windsurf"),
    ] {
        if root.join(dir).is_dir() {
            artifacts.push(DetectedArtifact {
                path: format!("{dir}/"),
                kind: ArtifactKind::AgentConfig,
                system: Some(name.to_string()),
                maps_to: None,
                note: None,
            });
        }
    }
    if root.join("AGENTS.md").exists() {
        artifacts.push(DetectedArtifact {
            path: "AGENTS.md".to_string(),
            kind: ArtifactKind::AgentConfig,
            system: None,
            maps_to: Some("element (agent definitions)".to_string()),
            note: None,
        });
    }

    // CI/CD
    if root.join(".github/workflows").is_dir() {
        let count = count_files(&root.join(".github/workflows"), &["yml", "yaml"]);
        artifacts.push(DetectedArtifact {
            path: ".github/workflows/".to_string(),
            kind: ArtifactKind::CiCd,
            system: None,
            maps_to: None,
            note: Some(format!("{count} workflow files")),
        });
    }
    if root.join(".gitlab-ci.yml").exists() {
        artifacts.push(DetectedArtifact {
            path: ".gitlab-ci.yml".to_string(),
            kind: ArtifactKind::CiCd,
            system: None,
            maps_to: None,
            note: None,
        });
    }

    // Build system
    for (file, name) in &[
        ("justfile", "just"),
        ("Justfile", "just"),
        ("Makefile", "make"),
        ("Taskfile.yml", "task"),
        ("Cargo.toml", "cargo (Rust)"),
        ("go.mod", "go module"),
        ("package.json", "npm/node"),
        ("pyproject.toml", "python"),
    ] {
        if root.join(file).exists() {
            artifacts.push(DetectedArtifact {
                path: file.to_string(),
                kind: ArtifactKind::BuildSystem,
                system: Some(name.to_string()),
                maps_to: None,
                note: None,
            });
        }
    }

    // Docs tree
    if root.join("docs").is_dir() {
        let count = count_files(&root.join("docs"), &["md"]);
        artifacts.push(DetectedArtifact {
            path: "docs/".to_string(),
            kind: ArtifactKind::DocsTree,
            system: None,
            maps_to: Some("element (documentation)".to_string()),
            note: Some(format!("{count} markdown files")),
        });
    }

    // Specs (speckit or similar)
    if root.join("specs").is_dir() {
        let count = count_subdirs(&root.join("specs"));
        artifacts.push(DetectedArtifact {
            path: "specs/".to_string(),
            kind: ArtifactKind::DocsTree,
            system: Some("speckit".to_string()),
            maps_to: Some("element (feature specifications)".to_string()),
            note: Some(format!("{count} feature directories")),
        });
    }

    // Sprint / RD
    if root.join("sprint").is_dir() {
        artifacts.push(DetectedArtifact {
            path: "sprint/".to_string(),
            kind: ArtifactKind::DocsTree,
            system: None,
            maps_to: Some("probe + finding (rapid development)".to_string()),
            note: None,
        });
    }

    // Config files at root
    for cfg in &["repos.yaml", "repos.yml"] {
        if root.join(cfg).exists() {
            artifacts.push(DetectedArtifact {
                path: cfg.to_string(),
                kind: ArtifactKind::Config,
                system: Some("orchestrator".to_string()),
                maps_to: None,
                note: Some("Multi-repo orchestrator registry".to_string()),
            });
        }
    }
}

// ── Source and Test Detection ──────────────────────────────────

fn detect_source_dirs(root: &Path, artifacts: &mut Vec<DetectedArtifact>) {
    // Source directories
    for src in &["src", "cmd", "internal", "pkg", "lib", "app"] {
        if root.join(src).is_dir() {
            artifacts.push(DetectedArtifact {
                path: format!("{src}/"),
                kind: ArtifactKind::SourceDir,
                system: None,
                maps_to: None,
                note: None,
            });
        }
    }

    // Test directories
    for test in &[
        "tests",
        "test",
        "__tests__",
        "testdata",
        "test_data",
        "benchmarks",
    ] {
        if root.join(test).is_dir() {
            artifacts.push(DetectedArtifact {
                path: format!("{test}/"),
                kind: ArtifactKind::TestDir,
                system: None,
                maps_to: None,
                note: None,
            });
        }
    }
}

// ── Git Stats ─────────────────────────────────────────────────

fn collect_git_stats(root: &Path) -> Option<GitStats> {
    if !root.join(".git").exists() {
        return None;
    }

    let commit_count = run_git(root, &["rev-list", "--count", "HEAD"])
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);

    let contributor_count = run_git(root, &["shortlog", "-sn", "--no-merges", "HEAD"])
        .map(|s| s.lines().count() as u64)
        .unwrap_or(0);

    let age_days = run_git(root, &["log", "--reverse", "--format=%at", "-1"])
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|first_commit| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            (now.saturating_sub(first_commit)) / 86400
        })
        .unwrap_or(0);

    let branch_count = run_git(root, &["branch", "--list"])
        .map(|s| s.lines().count() as u64)
        .unwrap_or(0);

    let default_branch = run_git(root, &["rev-parse", "--abbrev-ref", "HEAD"])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Some(GitStats {
        commit_count,
        contributor_count,
        age_days,
        branch_count,
        default_branch,
    })
}

fn run_git(root: &Path, args: &[&str]) -> Option<String> {
    std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
}

// ── Version Parsing ───────────────────────────────────────────

fn parse_bmad_version(manifest: &Path) -> Option<String> {
    let content = std::fs::read_to_string(manifest).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("version:") {
            let v = rest.trim().trim_matches('"').trim_matches('\'');
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn parse_speckit_version(opts: &Path) -> Option<String> {
    let content = std::fs::read_to_string(opts).ok()?;
    // JSON: look for "speckit_version": "..."
    let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
    parsed
        .get("speckit_version")
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
}

fn parse_spectacle_version(manifest: &Path) -> Option<String> {
    let content = std::fs::read_to_string(manifest).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("version:") {
            let v = rest.trim().trim_matches('"').trim_matches('\'');
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

// ── Helpers ───────────────────────────────────────────────────

fn count_files(dir: &Path, extensions: &[&str]) -> usize {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.path().is_file()
                && e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| extensions.contains(&ext))
        })
        .count()
}

fn count_subdirs(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().is_dir())
                .count()
        })
        .unwrap_or(0)
}
