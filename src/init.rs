use std::path::Path;

use crate::error::{KosError, Result};
use crate::model::{GraphInclude, GraphManifest, GraphScope};
use crate::workspace::{KOS_DIR, MANIFEST_FILE};

/// Current schema version that kos init targets.
const SCHEMA_VERSION: &str = "0.3";

/// Run the init subcommand.
pub fn run(target: &Path, scope: &GraphScope, id: Option<String>, update: bool) -> Result<()> {
    let kos_dir = target.join(KOS_DIR);

    if update {
        return run_update(target, &kos_dir);
    }

    if kos_dir.join(MANIFEST_FILE).exists() {
        return Err(KosError::GraphExists {
            path: kos_dir.display().to_string(),
        });
    }

    let graph_id = id.unwrap_or_else(|| {
        target
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string()
    });

    // 1. Create _kos/ directory structure
    create_directories(&kos_dir)?;

    // 2. Write kos.yaml manifest
    let includes = if *scope == GraphScope::Orchestrator {
        scan_subrepo_graphs(target)
    } else {
        vec![]
    };
    write_manifest(&kos_dir, &graph_id, scope, &includes)?;

    // 3. Create charter.md if none exists
    let charter_created = create_charter_if_missing(target, scope)?;

    // 4. Add session protocol to CLAUDE.md
    let claude_updated = update_claude_md(target, scope)?;

    // 5. Add commit convention rules
    let rules_created = create_commit_rules(target)?;

    // Print summary
    println!("Initialized kos graph in {}/", kos_dir.display());
    println!("  graph_id: {graph_id}");
    println!("  scope: {scope}");
    println!("  schema: v{SCHEMA_VERSION}");
    println!();
    println!("Created:");
    println!("  {KOS_DIR}/kos.yaml          (graph manifest)");
    println!("  {KOS_DIR}/nodes/             (bedrock, frontier, graveyard, placeholder)");
    println!("  {KOS_DIR}/findings/          (probe results)");
    println!("  {KOS_DIR}/probes/            (exploration briefs)");
    println!("  {KOS_DIR}/ideas/             (pre-hypothesis brainstorming)");
    if charter_created {
        println!("  charter.md              (re-introduction document)");
    } else {
        println!(
            "  charter.md              (exists — consider upgrading to re-introduction format)"
        );
    }
    if claude_updated {
        println!("  CLAUDE.md               (session protocol added)");
    }
    if rules_created {
        println!("  .claude/rules/kos-commits.md (commit conventions)");
    }
    if !includes.is_empty() {
        println!();
        println!("Discovered subrepo graphs:");
        for inc in &includes {
            println!("  - {}", inc.path);
        }
    }
    println!();
    println!("Next steps:");
    if charter_created {
        println!("  1. Edit charter.md — describe what this project is solving");
    } else {
        println!(
            "  1. Review charter.md — add problem statement, design values, non-goals, session log"
        );
    }
    println!("  2. Add your first frontier question: {KOS_DIR}/nodes/frontier/question-*.yaml");
    println!("  3. Run `kos doctor` to verify everything is healthy");

    Ok(())
}

/// Run init --update on an existing _kos/ installation.
fn run_update(target: &Path, kos_dir: &Path) -> Result<()> {
    if !kos_dir.join(MANIFEST_FILE).exists() {
        return Err(KosError::Init {
            message: format!(
                "no existing _kos/ found at {}. Run `kos init` first.",
                target.display()
            ),
        });
    }

    let mut changes = Vec::new();

    // Ensure all subdirectories exist
    let subdirs = [
        "nodes/bedrock",
        "nodes/frontier",
        "nodes/graveyard",
        "nodes/placeholder",
        "findings",
        "probes",
        "ideas",
    ];
    for sub in &subdirs {
        let dir = kos_dir.join(sub);
        if !dir.exists() {
            std::fs::create_dir_all(&dir).map_err(KosError::Io)?;
            changes.push(format!("created {KOS_DIR}/{sub}/"));
        }
    }

    // Update schema_version if behind
    let manifest_path = kos_dir.join(MANIFEST_FILE);
    let content = std::fs::read_to_string(&manifest_path).map_err(KosError::Io)?;
    let mut manifest: GraphManifest =
        serde_yaml::from_str(&content).map_err(|e| KosError::Manifest {
            path: manifest_path.display().to_string(),
            message: e.to_string(),
        })?;

    if manifest.schema_version != SCHEMA_VERSION {
        let old = manifest.schema_version.clone();
        manifest.schema_version = SCHEMA_VERSION.to_string();
        write_manifest_raw(&manifest_path, &manifest)?;
        changes.push(format!("schema_version: {old} → {SCHEMA_VERSION}"));
    }

    // For orchestrator scope: scan for new subrepo graphs
    if manifest.scope == GraphScope::Orchestrator {
        let known: std::collections::HashSet<String> =
            manifest.includes.iter().map(|i| i.path.clone()).collect();
        let discovered = scan_subrepo_graphs(target);
        for inc in discovered {
            if !known.contains(&inc.path) {
                changes.push(format!("added include: {}", inc.path));
                manifest.includes.push(inc);
            }
        }
        if changes.iter().any(|c| c.starts_with("added include")) {
            write_manifest_raw(&manifest_path, &manifest)?;
        }
    }

    // Add session protocol to CLAUDE.md if missing
    if update_claude_md(target, &manifest.scope)? {
        changes.push("added session protocol to CLAUDE.md".to_string());
    }

    // Add commit rules if missing
    if create_commit_rules(target)? {
        changes.push("added .claude/rules/kos-commits.md".to_string());
    }

    if changes.is_empty() {
        println!("kos init --update: everything up to date.");
    } else {
        println!("kos init --update: {} change(s):", changes.len());
        for change in &changes {
            println!("  - {change}");
        }
    }

    Ok(())
}

fn create_directories(kos_dir: &Path) -> Result<()> {
    let dirs = [
        "",
        "nodes",
        "nodes/bedrock",
        "nodes/frontier",
        "nodes/graveyard",
        "nodes/placeholder",
        "findings",
        "probes",
        "ideas",
    ];
    for d in &dirs {
        std::fs::create_dir_all(kos_dir.join(d)).map_err(KosError::Io)?;
    }
    Ok(())
}

fn write_manifest(
    kos_dir: &Path,
    graph_id: &str,
    scope: &GraphScope,
    includes: &[GraphInclude],
) -> Result<()> {
    let manifest = GraphManifest {
        graph_id: graph_id.to_string(),
        scope: scope.clone(),
        description: None,
        schema_version: SCHEMA_VERSION.to_string(),
        includes: includes.to_vec(),
    };
    write_manifest_raw(&kos_dir.join(MANIFEST_FILE), &manifest)
}

fn write_manifest_raw(path: &Path, manifest: &GraphManifest) -> Result<()> {
    let yaml = serde_yaml::to_string(manifest).map_err(|e| KosError::Manifest {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    std::fs::write(path, yaml).map_err(KosError::Io)
}

/// Scan subrepo directories for existing _kos/ graphs.
fn scan_subrepo_graphs(root: &Path) -> Vec<GraphInclude> {
    let mut includes = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                let sub_kos = path.join(KOS_DIR).join(MANIFEST_FILE);
                if sub_kos.exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        includes.push(GraphInclude {
                            path: format!("{name}/{KOS_DIR}"),
                        });
                    }
                }
            }
        }
    }
    includes.sort_by(|a, b| a.path.cmp(&b.path));
    includes
}

/// Create charter.md if none exists. Returns true if created.
fn create_charter_if_missing(target: &Path, scope: &GraphScope) -> Result<bool> {
    let charter_path = target.join("charter.md");
    if charter_path.exists() {
        return Ok(false);
    }

    let template = match scope {
        GraphScope::Orchestrator => CHARTER_TEMPLATE_ORCHESTRATOR,
        GraphScope::Repo => CHARTER_TEMPLATE_REPO,
    };

    std::fs::write(&charter_path, template).map_err(KosError::Io)?;
    Ok(true)
}

/// Add session protocol to CLAUDE.md. Returns true if modified or created.
fn update_claude_md(target: &Path, scope: &GraphScope) -> Result<bool> {
    let claude_path = target.join("CLAUDE.md");
    let section_marker = "## How to Work Here (kos Process)";

    if claude_path.exists() {
        let content = std::fs::read_to_string(&claude_path).map_err(KosError::Io)?;
        if content.contains(section_marker) {
            return Ok(false); // Already has session protocol
        }
        // Append session protocol
        let protocol = match scope {
            GraphScope::Orchestrator => SESSION_PROTOCOL_ORCHESTRATOR,
            GraphScope::Repo => SESSION_PROTOCOL_REPO,
        };
        let updated = format!("{content}\n{protocol}");
        std::fs::write(&claude_path, updated).map_err(KosError::Io)?;
        Ok(true)
    } else {
        // Create new CLAUDE.md with placeholder + session protocol
        let project_name = target
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");
        let protocol = match scope {
            GraphScope::Orchestrator => SESSION_PROTOCOL_ORCHESTRATOR,
            GraphScope::Repo => SESSION_PROTOCOL_REPO,
        };
        let content =
            format!("# {project_name}\n\n<!-- Describe your project here -->\n\n{protocol}");
        std::fs::write(&claude_path, content).map_err(KosError::Io)?;
        Ok(true)
    }
}

/// Create .claude/rules/kos-commits.md if .claude/ exists and rule is missing.
/// Creates .claude/rules/ if .claude/ exists but rules/ doesn't.
/// Returns true if created.
fn create_commit_rules(target: &Path) -> Result<bool> {
    let claude_dir = target.join(".claude");
    if !claude_dir.exists() {
        return Ok(false);
    }
    let rules_dir = claude_dir.join("rules");
    if !rules_dir.exists() {
        std::fs::create_dir_all(&rules_dir).map_err(KosError::Io)?;
    }
    let rules_file = rules_dir.join("kos-commits.md");
    if rules_file.exists() {
        return Ok(false);
    }
    std::fs::write(&rules_file, COMMIT_RULES_TEMPLATE).map_err(KosError::Io)?;
    Ok(true)
}

// ── Templates ───────────────────────────────────────────────

const CHARTER_TEMPLATE_ORCHESTRATOR: &str = r#"# Project Charter

Re-introduction document. Captures what we know, what's open, what's been ruled out.

Follows the kos process: Orient → Question → Probe → Harvest → Promote.
Authoritative graph: _kos/nodes/. This document is a prose projection of the graph.

Last updated: <!-- date -->

---

## The Problem Statement

<!-- What is this project solving? Why does it exist? What fails without it? -->

---

## Design Values

<!-- Three principles that resolved every design tradeoff. -->

1. **<!-- Value 1 -->**
2. **<!-- Value 2 -->**
3. **<!-- Value 3 -->**

---

## Non-Goals

- <!-- What this project explicitly is NOT -->

---

## Bedrock

*Established. Evidence-based or decided with rationale. Changing requires new evidence.*

<!-- Nothing established yet — start probing. -->

---

## Frontier

*Actively open — under exploration, not yet resolved.*

<!-- No open questions yet — orient first. -->

---

## Graveyard

*Tried, ruled out, permanently recorded.*

<!-- Nothing ruled out yet. -->

---

## Session Log

<!-- Compressed record of each session's work. -->
"#;

const CHARTER_TEMPLATE_REPO: &str = r#"# Project Charter

What we know, what's open, what's been ruled out.

Follows the kos process: Orient → Question → Probe → Harvest → Promote.
Authoritative graph: _kos/nodes/.
Cross-repo questions belong in the orchestrator's charter.

Last updated: <!-- date -->

---

## Bedrock

*Established. Evidence-based or decided with rationale.*

<!-- Nothing established yet — start probing. -->

---

## Frontier

*Actively open — under exploration, not yet resolved.*

<!-- No open questions yet — orient first. -->

---

## Graveyard

*Tried, ruled out, permanently recorded.*

<!-- Nothing ruled out yet. -->
"#;

const SESSION_PROTOCOL_ORCHESTRATOR: &str = r#"## How to Work Here (kos Process)

### Re-introduction
Read charter.md before any substantive work. It contains:
- Current bedrock (what's committed)
- Current frontier (what's under exploration)
- Current graveyard (what's been ruled out)
- Open questions (the current work queue)

### Session Protocol
1. Read charter.md (orient)
2. Identify the highest-value open question — or capture new ideas in _kos/ideas/
3. Write an Exploration Brief in _kos/probes/ (or sprint/rd/ for code probes)
4. Do the probe work (may span multiple subrepos)
5. Write a finding in _kos/findings/
6. Harvest: update affected nodes, move files if confidence changed
7. Update charter.md if bedrock changed

### Ideas (pre-hypothesis brainstorming)
Ideas live in _kos/ideas/ as markdown files. They are generative, possibly
contradictory, and carry no commitment. Ideas are part of the graph's memory
(the DAG) — they spring into existence here. When an idea crystallizes into
something testable, extract it into a frontier question node and a probe brief.
The idea file gets a note pointing to what it became.

### Multi-Repo Probes
Questions that span repos live at the orchestrator level (_kos/nodes/).
Questions scoped to a single repo live in that repo's _kos/ (if it has one)
or as issues/docs in the repo itself.

When a probe touches multiple repos, the finding lives in the orchestrator's
_kos/findings/ — cross-cutting knowledge belongs at the composition layer.
Commits in subrepos use kos action vocabulary: probe(scope):, finding(scope):,
harvest(scope):.

### Node Files
Nodes live in _kos/nodes/[confidence]/[id].yaml
Schema follows kos schema v0.3 (see kos/schema/node.schema.yaml)
One node per file. Filename = node id.

### Confidence Changes
Moving a file between confidence directories IS the promotion.
Always accompany with a commit message explaining the evidence.

### Harvest Verification
Before starting the next cycle, verify:
- [ ] Finding written and committed
- [ ] Charter updated if bedrock changed
- [ ] Frontier questions updated (closed, opened, or revised)
- [ ] Exploration briefs marked complete or carried forward
- [ ] Subrepo artifacts updated if the finding affects them
"#;

const SESSION_PROTOCOL_REPO: &str = r#"## How to Work Here (kos Process)

### Re-introduction
Read charter.md before any substantive work. It contains:
- Current bedrock (what's committed)
- Current frontier (what's under exploration)
- Current graveyard (what's been ruled out)

### Session Protocol
1. Read charter.md (orient)
2. Identify the highest-value open question — or capture new ideas in _kos/ideas/
3. Write an Exploration Brief in _kos/probes/
4. Do the probe work
5. Write a finding in _kos/findings/
6. Harvest: update affected nodes, move files if confidence changed
7. Update charter.md if bedrock changed

Cross-repo questions belong in the orchestrator's _kos/, not here.

### Ideas (pre-hypothesis brainstorming)
Ideas live in _kos/ideas/ as markdown files. Generative, possibly contradictory,
no commitment. When an idea crystallizes, extract into a frontier question + brief.

### Node Files
Nodes live in _kos/nodes/[confidence]/[id].yaml
Schema follows kos schema v0.3.
One node per file. Filename = node id.

### Confidence Changes
Moving a file between confidence directories IS the promotion.
Always accompany with a commit message explaining the evidence.

### Harvest Verification
Before starting the next cycle, verify:
- [ ] Finding written and committed
- [ ] Charter updated if bedrock changed
- [ ] Frontier questions updated (closed, opened, or revised)
- [ ] Exploration briefs marked complete or carried forward
"#;

const COMMIT_RULES_TEMPLATE: &str = r#"# kos Commit Conventions

## kos-specific actions (from KOS process cycle)

In addition to conventional commit types (feat, fix, docs, etc.),
use these kos action types when working with the knowledge graph:

- `harvest`: update nodes after a probe cycle completes
- `promote`: move a node to a higher confidence tier
- `graveyard`: move a node to graveyard (ruled out)
- `probe`: begin or continue an exploration
- `finding`: write a finding from a probe
- `schema`: update the node schema
- `charter`: update the charter document

### Format
`[action]: [node-ids affected] — [one line description]`

### Examples
```
harvest: question-director-design — probe complete, see finding-042
promote: elem-byoa-platform — validated across 3 repos
graveyard: grv-monolith-approach — ruled out, see graveyard entry
probe(marvel): workspace model — exploring organizational hierarchy
finding(kos): distributed graph — three-tier model confirmed
```

### No AI Attribution
Do not add "Generated with Claude Code", "Co-Authored-By: Claude", or any
AI attribution to commits. The human is the author.
"#;
