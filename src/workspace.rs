use std::path::{Path, PathBuf};

use crate::error::{KosError, Result};
use crate::model::{GraphScope, GraphSource};

/// The kos directory name convention: private-not-hidden.
pub const KOS_DIR: &str = "_kos";
/// The manifest filename inside a _kos/ directory.
pub const MANIFEST_FILE: &str = "kos.yaml";

/// A `kos.yaml` that exists but could not be read or parsed. Discovery keeps
/// it rather than dropping the graph without a word (kos#164).
#[derive(Debug, Clone)]
pub struct ManifestProblem {
    /// Absolute path to the `kos.yaml` file.
    pub path: PathBuf,
    /// What went wrong, as the reader reported it.
    pub message: String,
}

impl std::fmt::Display for ManifestProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.message)
    }
}

/// The aae-orc workspace layout, discovered from the filesystem.
#[derive(Debug)]
pub struct Workspace {
    /// Root of the aae-orc orchestrator (contains charter.md, sprint/, kos/, etc.)
    pub root: PathBuf,
    /// Root of the kos subrepo (contains nodes/, findings/, probes/, schema/)
    pub kos_root: PathBuf,
    /// Discovered graph sources (_kos/ directories with kos.yaml manifests).
    pub graphs: Vec<GraphSource>,
    /// Manifests that exist but do not read or parse, so no graph was found
    /// for them. Absent manifests are not listed.
    pub manifest_errors: Vec<ManifestProblem>,
}

impl Workspace {
    /// Discover the workspace by walking up from `start` looking for markers.
    ///
    /// Discovery order (first match wins):
    /// 1. KOS-charter.md — we're inside the kos repo itself
    /// 2. charter.md + kos/ subdir — we're at the aae-orc orchestrator root
    /// 3. Parent has charter.md + kos/ — we're in a sibling subrepo of aae-orc
    /// 4. _kos/kos.yaml — standalone repo with its own knowledge graph
    pub fn discover(start: &Path) -> Result<Self> {
        let start = std::fs::canonicalize(start).map_err(KosError::Io)?;

        // Walk up looking for workspace markers
        let mut current = start.as_path();
        let (root, kos_root) = loop {
            // Are we in the kos repo itself?
            if current.join("KOS-charter.md").exists() {
                let kos_root = current.to_path_buf();
                let orc_root = current
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| kos_root.clone());
                break (orc_root, kos_root);
            }

            // Are we in the aae-orc root? (has charter.md and kos/ subdir)
            if current.join("charter.md").exists() && current.join("kos").is_dir() {
                let orc_root = current.to_path_buf();
                let kos_root = orc_root.join("kos");
                break (orc_root, kos_root);
            }

            // Are we in a sibling subrepo? Check if parent is aae-orc
            if let Some(parent) = current.parent() {
                if parent.join("charter.md").exists() && parent.join("kos").is_dir() {
                    break (parent.to_path_buf(), parent.join("kos"));
                }
            }

            // Standalone repo with _kos/kos.yaml?
            let standalone_kos = current.join(KOS_DIR);
            if standalone_kos.join(MANIFEST_FILE).exists() {
                let root = current.to_path_buf();
                let (graphs, manifest_errors) = discover_graphs(&root);
                return Ok(Workspace {
                    root: root.clone(),
                    kos_root: root,
                    graphs,
                    manifest_errors,
                });
            }

            if let Some(parent) = current.parent() {
                current = parent;
            } else {
                return Err(KosError::WorkspaceNotFound {
                    start: start.display().to_string(),
                });
            }
        };

        let (graphs, manifest_errors) = discover_graphs(&root);

        Ok(Workspace {
            root,
            kos_root,
            graphs,
            manifest_errors,
        })
    }

    /// Construct a workspace from an explicit path (--workspace or KOS_WORKSPACE).
    /// The path can be:
    /// 1. The kos repo root (containing KOS-charter.md)
    /// 2. The aae-orc root (containing kos/ subdir)
    /// 3. A standalone repo with _kos/kos.yaml
    pub fn from_explicit(path: &Path) -> Result<Self> {
        let path = std::fs::canonicalize(path).map_err(KosError::Io)?;

        // Is this the kos root directly?
        if path.join("KOS-charter.md").exists() {
            let kos_root = path.clone();
            let orc_root = path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| kos_root.clone());
            let (graphs, manifest_errors) = discover_graphs(&orc_root);
            return Ok(Workspace {
                root: orc_root,
                kos_root,
                graphs,
                manifest_errors,
            });
        }

        // Is this the aae-orc root with a kos/ subdir?
        if path.join("kos").is_dir() {
            let (graphs, manifest_errors) = discover_graphs(&path);
            return Ok(Workspace {
                root: path.clone(),
                kos_root: path.join("kos"),
                graphs,
                manifest_errors,
            });
        }

        // Is this a standalone repo with _kos/kos.yaml?
        if path.join(KOS_DIR).join(MANIFEST_FILE).exists() {
            let (graphs, manifest_errors) = discover_graphs(&path);
            return Ok(Workspace {
                root: path.clone(),
                kos_root: path,
                graphs,
                manifest_errors,
            });
        }

        Err(KosError::InvalidWorkspace {
            path: path.display().to_string(),
        })
    }

    /// Graph ids that more than one discovered root claims, one report per
    /// id. A report only: validation does not fail on it (aae-orc-gtv04).
    pub fn shared_graph_ids(&self) -> Vec<ManifestProblem> {
        resolve_graph_ids(&self.root, &self.graphs).1
    }

    /// Returns `true` when the workspace is standalone — a repo with
    /// `_kos/kos.yaml` that is NOT under an aae-orc orchestrator.
    ///
    /// In standalone mode, `root == kos_root` because there is no parent
    /// orchestrator to separate them.
    pub fn is_standalone(&self) -> bool {
        self.root == self.kos_root
    }

    /// Infer the target repo name from a path.
    /// If the path is inside a known subrepo, return its name.
    /// Otherwise return None.
    pub fn infer_target(&self, path: &Path) -> Option<String> {
        let canonical = std::fs::canonicalize(path).ok()?;
        let canonical_root = std::fs::canonicalize(&self.root).ok()?;

        // Strip the workspace root to get relative path
        let relative = canonical.strip_prefix(&canonical_root).ok()?;

        // The first component is the subrepo name
        relative
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
            .map(ToString::to_string)
    }

    /// List known subrepo directories that exist.
    pub fn subrepos(&self) -> Vec<String> {
        let known = [
            "forestage",
            "switchboard",
            "marvel",
            "specticle",
            "director",
            "kos",
            "ai",
            "ourbot",
            "ThreeDoors",
        ];
        known
            .iter()
            .filter(|name| self.root.join(name).is_dir())
            .map(ToString::to_string)
            .collect()
    }

    /// The unreadable manifest that governs `from`, if one does: the
    /// deepest `_kos/kos.yaml` above `from` that failed to parse, unless a
    /// healthy graph sits nearer. Without this, such a directory looks like
    /// one with no graph and validation falls back silently (kos#164).
    pub fn nearest_manifest_problem(&self, from: &Path) -> Option<&ManifestProblem> {
        let canonical = std::fs::canonicalize(from).ok()?;
        let depth = |p: &Path| p.components().count();
        let repo_of = |p: &ManifestProblem| {
            p.path
                .parent()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
        };
        let (problem, repo) = self
            .manifest_errors
            .iter()
            .filter_map(|p| repo_of(p).map(|r| (p, r)))
            .filter(|(_, r)| canonical.starts_with(r))
            .max_by_key(|(_, r)| depth(r))?;
        let healthy_nearer = self.graphs.iter().any(|g| {
            g.path
                .parent()
                .is_some_and(|gp| canonical.starts_with(gp) && depth(gp) > depth(&repo))
        });
        (!healthy_nearer).then_some(problem)
    }

    /// Find the graph source for the nearest _kos/ directory relative to a path.
    /// Returns the local graph if cwd has _kos/, otherwise the orchestrator graph.
    pub fn nearest_graph(&self, from: &Path) -> Option<&GraphSource> {
        let canonical = std::fs::canonicalize(from).ok()?;

        // Check if we're inside a subrepo that has its own graph
        for graph in &self.graphs {
            if graph.scope == GraphScope::Repo {
                if let Some(graph_parent) = graph.path.parent() {
                    if canonical.starts_with(graph_parent) {
                        return Some(graph);
                    }
                }
            }
        }

        // Fall back to orchestrator graph
        self.graphs
            .iter()
            .find(|g| g.scope == GraphScope::Orchestrator)
    }

    /// Return the graph root for node operations.
    /// Prefers _kos/ layout; falls back to legacy nodes/ layout at kos_root.
    pub fn node_root(&self) -> PathBuf {
        // Check for _kos/ in kos repo
        let kos_graph = self.kos_root.join(KOS_DIR);
        if kos_graph.join(MANIFEST_FILE).exists() {
            return kos_graph;
        }

        // Legacy: nodes/ directly in kos_root
        self.kos_root.clone()
    }
}

/// Discover all _kos/ graph sources under a workspace root.
///
/// Algorithm:
/// 1. Check workspace root for _kos/kos.yaml → orchestrator graph
/// 2. Follow includes from orchestrator manifest
/// 3. Check kos/ subrepo for _kos/kos.yaml
/// 4. Check other subrepo dirs for _kos/kos.yaml
///
/// Falls back to legacy layout if no _kos/ directories found.
fn discover_graphs(root: &Path) -> (Vec<GraphSource>, Vec<ManifestProblem>) {
    let mut sources = Vec::new();
    let mut problems = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    // 1. Check orchestrator root for _kos/
    let orc_kos = root.join(KOS_DIR);
    if let Some(source) = load_graph_source(&orc_kos, &mut problems) {
        seen_paths.insert(orc_kos.clone());

        // 2. Follow includes from orchestrator manifest
        for include in &source.manifest.includes {
            let include_path = root.join(&include.path);
            if !seen_paths.contains(&include_path) {
                if let Some(included) = load_graph_source(&include_path, &mut problems) {
                    seen_paths.insert(include_path);
                    sources.push(included);
                }
            }
        }

        sources.insert(0, source);
    }

    // 3. Scan subrepo directories for _kos/ (discover graphs not in includes)
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                let sub_kos = path.join(KOS_DIR);
                if !seen_paths.contains(&sub_kos) {
                    if let Some(source) = load_graph_source(&sub_kos, &mut problems) {
                        seen_paths.insert(sub_kos);
                        sources.push(source);
                    }
                }
            }
        }
    }

    (sources, problems)
}

/// Load a graph source from a _kos/ directory if it has a valid kos.yaml.
/// A missing manifest is `None` and nothing more: discovery looks in every
/// subdirectory. A manifest that exists but does not read or parse is also
/// `None`, but is recorded in `problems` so the caller can say so (kos#164).
fn load_graph_source(kos_dir: &Path, problems: &mut Vec<ManifestProblem>) -> Option<GraphSource> {
    let manifest_path = kos_dir.join(MANIFEST_FILE);
    let manifest = match crate::findings::read_manifest(kos_dir) {
        Ok(Some(manifest)) => manifest,
        Ok(None) => return None,
        Err(e) => {
            let message = match e {
                KosError::Manifest { message, .. } => message,
                other => other.to_string(),
            };
            problems.push(ManifestProblem {
                path: manifest_path,
                message,
            });
            return None;
        }
    };

    Some(GraphSource {
        graph_id: manifest.graph_id.clone(),
        path: kos_dir.to_path_buf(),
        scope: manifest.scope.clone(),
        manifest,
    })
}

/// True when the nearest `.git` at or above `path` is a directory, which is a
/// primary checkout; in a linked worktree it is a file.
fn is_primary_checkout(path: &Path) -> bool {
    path.ancestors()
        .map(|dir| dir.join(".git"))
        .find(|git| git.exists())
        .is_some_and(|git| git.is_dir())
}

/// Which graph answers for each graph id, and which ids more than one graph
/// claims. Discovery lists graphs in directory order, which is not stable, so
/// the answer comes from this rule and never from the listing (aae-orc-gtv04):
///
/// 1. the root the orchestrator manifest's `includes` names;
/// 2. a root in a primary checkout: the nearest `.git` found walking up from
///    the root is a directory, where a linked worktree has a `.git` file. This
///    covers the orchestrator's own `<root>/_kos` as well as `<root>/<dir>/_kos`;
/// 3. the smallest canonical path.
///
/// A shared id is reported once, as a `ManifestProblem` on the chosen root. It
/// is a report, not a failure.
pub(crate) fn resolve_graph_ids<'a>(
    root: &Path,
    graphs: &'a [GraphSource],
) -> (Vec<&'a GraphSource>, Vec<ManifestProblem>) {
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let included: Vec<PathBuf> = graphs
        .iter()
        .filter(|g| g.scope == GraphScope::Orchestrator)
        .flat_map(|g| {
            g.manifest
                .includes
                .iter()
                .map(|i| canon(&root.join(&i.path)))
        })
        .collect();
    let rank = |g: &GraphSource| {
        let path = canon(&g.path);
        let tier = if included.contains(&path) {
            0
        } else if is_primary_checkout(&path) {
            1
        } else {
            2
        };
        (tier, path)
    };

    let mut by_id: std::collections::BTreeMap<String, Vec<&GraphSource>> =
        std::collections::BTreeMap::new();
    for g in graphs {
        by_id.entry(g.graph_id.to_lowercase()).or_default().push(g);
    }
    let mut chosen = Vec::new();
    let mut problems = Vec::new();
    for (id, mut group) in by_id {
        group.sort_by_cached_key(|g| rank(g));
        let first = group[0];
        if group.len() > 1 {
            let others: Vec<String> = group[1..]
                .iter()
                .map(|g| g.path.display().to_string())
                .collect();
            problems.push(ManifestProblem {
                path: first.path.join(MANIFEST_FILE),
                message: format!(
                    "graph_id '{id}' is also claimed by {}; using this one",
                    others.join(", ")
                ),
            });
        }
        chosen.push(first);
    }
    (chosen, problems)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::model::{GraphInclude, GraphManifest};

    fn manifest(id: &str, scope: GraphScope, includes: &[&str]) -> GraphManifest {
        GraphManifest {
            graph_id: id.to_string(),
            scope,
            description: None,
            schema_version: "0.3".to_string(),
            includes: includes
                .iter()
                .map(|p| GraphInclude {
                    path: (*p).to_string(),
                })
                .collect(),
            findings: None,
        }
    }

    fn source(root: &Path, dir: &str, id: &str, git: Option<bool>) -> GraphSource {
        let repo = root.join(dir);
        let kos = repo.join(KOS_DIR);
        fs::create_dir_all(&kos).unwrap();
        match git {
            Some(true) => fs::create_dir_all(repo.join(".git")).unwrap(),
            Some(false) => fs::write(repo.join(".git"), "gitdir: elsewhere\n").unwrap(),
            None => {}
        }
        GraphSource {
            graph_id: id.to_string(),
            path: kos,
            scope: GraphScope::Repo,
            manifest: manifest(id, GraphScope::Repo, &[]),
        }
    }

    fn orchestrator(root: &Path, includes: &[&str]) -> GraphSource {
        let kos = root.join(KOS_DIR);
        fs::create_dir_all(&kos).unwrap();
        GraphSource {
            graph_id: "orc".to_string(),
            path: kos,
            scope: GraphScope::Orchestrator,
            manifest: manifest("orc", GraphScope::Orchestrator, includes),
        }
    }

    /// The chosen root's directory name for `id`, with the input in `order`.
    fn chosen(root: &Path, graphs: &[GraphSource], id: &str) -> String {
        let (picked, _) = resolve_graph_ids(root, graphs);
        let g = picked.iter().find(|g| g.graph_id == id).unwrap();
        g.path
            .parent()
            .and_then(Path::file_name)
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    /// Both input orders must give the same answer; return it.
    fn both_orders(root: &Path, graphs: Vec<GraphSource>, id: &str) -> String {
        let forward = chosen(root, &graphs, id);
        let mut reversed = graphs;
        reversed.reverse();
        assert_eq!(
            forward,
            chosen(root, &reversed, id),
            "order changed the answer"
        );
        forward
    }

    #[test]
    fn the_primary_checkout_beats_a_worktree_in_either_order() {
        let tmp = tempfile::tempdir().unwrap();
        let r = tmp.path();
        // The worktree sorts first by path, so only rule 2 can pick the other.
        let graphs = vec![
            source(r, "a-wt", "x", Some(false)),
            source(r, "alpha", "x", Some(true)),
        ];
        assert_eq!(both_orders(r, graphs, "x"), "alpha");
    }

    #[test]
    fn the_orchestrators_own_checkout_beats_a_same_id_worktree_in_either_order() {
        let tmp = tempfile::tempdir().unwrap();
        let r = tmp.path();
        fs::create_dir_all(r.join(".git")).unwrap();
        // "0-wt" sorts ahead of "_kos", so only the .git rule can pick the orc root.
        let mut wt = source(r, "0-wt", "orc", Some(false));
        wt.scope = GraphScope::Repo;
        let graphs = vec![wt, orchestrator(r, &[])];
        let (picked, _) = resolve_graph_ids(r, &graphs);
        assert_eq!(picked[0].path, r.join(KOS_DIR));
        let mut reversed = graphs;
        reversed.reverse();
        let (picked, _) = resolve_graph_ids(r, &reversed);
        assert_eq!(picked[0].path, r.join(KOS_DIR));
    }

    #[test]
    fn the_root_the_manifest_includes_beats_the_primary_checkout() {
        let tmp = tempfile::tempdir().unwrap();
        let r = tmp.path();
        let graphs = vec![
            orchestrator(r, &["z-wt/_kos"]),
            source(r, "alpha", "x", Some(true)),
            source(r, "z-wt", "x", Some(false)),
        ];
        assert_eq!(both_orders(r, graphs, "x"), "z-wt");
    }

    #[test]
    fn with_no_other_rule_the_smallest_path_wins_in_either_order() {
        let tmp = tempfile::tempdir().unwrap();
        let r = tmp.path();
        let graphs = vec![source(r, "m2", "x", None), source(r, "m1", "x", None)];
        assert_eq!(both_orders(r, graphs, "x"), "m1");
    }

    #[test]
    fn a_shared_id_is_reported_once_and_a_unique_id_not_at_all() {
        let tmp = tempfile::tempdir().unwrap();
        let r = tmp.path();
        let graphs = vec![
            source(r, "alpha", "x", Some(true)),
            source(r, "a-wt", "x", Some(false)),
            source(r, "a-wt2", "x", Some(false)),
            source(r, "solo", "y", Some(true)),
        ];
        let (_, problems) = resolve_graph_ids(r, &graphs);
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].message.contains("'x'"), "{}", problems[0]);
        assert!(
            problems[0].path.starts_with(r.join("alpha")),
            "{}",
            problems[0]
        );

        // Two claimants are enough to report; one is not.
        let pair = vec![source(r, "p1", "z", None), source(r, "p2", "z", None)];
        assert_eq!(resolve_graph_ids(r, &pair).1.len(), 1);
        assert!(resolve_graph_ids(r, &pair[..1]).1.is_empty());
    }
}
