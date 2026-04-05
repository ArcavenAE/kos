use std::path::{Path, PathBuf};

use crate::error::{KosError, Result};
use crate::model::{GraphManifest, GraphScope, GraphSource};

/// The kos directory name convention: private-not-hidden.
pub const KOS_DIR: &str = "_kos";
/// The manifest filename inside a _kos/ directory.
pub const MANIFEST_FILE: &str = "kos.yaml";

/// The aae-orc workspace layout, discovered from the filesystem.
#[derive(Debug)]
pub struct Workspace {
    /// Root of the aae-orc orchestrator (contains charter.md, sprint/, kos/, etc.)
    pub root: PathBuf,
    /// Root of the kos subrepo (contains nodes/, findings/, probes/, schema/)
    pub kos_root: PathBuf,
    /// Discovered graph sources (_kos/ directories with kos.yaml manifests).
    pub graphs: Vec<GraphSource>,
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
                let graphs = discover_graphs(&root);
                return Ok(Workspace {
                    root: root.clone(),
                    kos_root: root,
                    graphs,
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

        let graphs = discover_graphs(&root);

        Ok(Workspace {
            root,
            kos_root,
            graphs,
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
            let graphs = discover_graphs(&orc_root);
            return Ok(Workspace {
                root: orc_root,
                kos_root,
                graphs,
            });
        }

        // Is this the aae-orc root with a kos/ subdir?
        if path.join("kos").is_dir() {
            let graphs = discover_graphs(&path);
            return Ok(Workspace {
                root: path.clone(),
                kos_root: path.join("kos"),
                graphs,
            });
        }

        // Is this a standalone repo with _kos/kos.yaml?
        if path.join(KOS_DIR).join(MANIFEST_FILE).exists() {
            let graphs = discover_graphs(&path);
            return Ok(Workspace {
                root: path.clone(),
                kos_root: path,
                graphs,
            });
        }

        Err(KosError::InvalidWorkspace {
            path: path.display().to_string(),
        })
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
            "aclaude",
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
fn discover_graphs(root: &Path) -> Vec<GraphSource> {
    let mut sources = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    // 1. Check orchestrator root for _kos/
    let orc_kos = root.join(KOS_DIR);
    if let Some(source) = load_graph_source(&orc_kos) {
        seen_paths.insert(orc_kos.clone());

        // 2. Follow includes from orchestrator manifest
        for include in &source.manifest.includes {
            let include_path = root.join(&include.path);
            if !seen_paths.contains(&include_path) {
                if let Some(included) = load_graph_source(&include_path) {
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
                    if let Some(source) = load_graph_source(&sub_kos) {
                        seen_paths.insert(sub_kos);
                        sources.push(source);
                    }
                }
            }
        }
    }

    sources
}

/// Load a graph source from a _kos/ directory if it has a valid kos.yaml.
fn load_graph_source(kos_dir: &Path) -> Option<GraphSource> {
    let manifest_path = kos_dir.join(MANIFEST_FILE);
    let content = std::fs::read_to_string(&manifest_path).ok()?;
    let manifest: GraphManifest = serde_yaml::from_str(&content).ok()?;

    Some(GraphSource {
        graph_id: manifest.graph_id.clone(),
        path: kos_dir.to_path_buf(),
        scope: manifest.scope.clone(),
        manifest,
    })
}
