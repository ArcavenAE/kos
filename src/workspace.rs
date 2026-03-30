use std::path::{Path, PathBuf};

use crate::error::{KosError, Result};

/// The aae-orc workspace layout, discovered from the filesystem.
#[derive(Debug)]
pub struct Workspace {
    /// Root of the aae-orc orchestrator (contains charter.md, sprint/, kos/, etc.)
    pub root: PathBuf,
    /// Root of the kos subrepo (contains nodes/, findings/, probes/, schema/)
    pub kos_root: PathBuf,
}

impl Workspace {
    /// Discover the workspace by walking up from `start` looking for markers.
    ///
    /// The kos repo can be found in two ways:
    /// 1. We're already inside the kos directory (has KOS-charter.md)
    /// 2. We're in aae-orc or a sibling subrepo (kos/ is a subdirectory of aae-orc)
    pub fn discover(start: &Path) -> Result<Self> {
        let start = std::fs::canonicalize(start).map_err(KosError::Io)?;

        // Walk up looking for either aae-orc root or kos root
        let mut current = start.as_path();
        loop {
            // Are we in the kos repo itself?
            if current.join("KOS-charter.md").exists() {
                // kos root found — aae-orc is the parent
                let kos_root = current.to_path_buf();
                let orc_root = current
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| kos_root.clone());
                return Ok(Workspace {
                    root: orc_root,
                    kos_root,
                });
            }

            // Are we in the aae-orc root? (has charter.md and kos/ subdir)
            if current.join("charter.md").exists() && current.join("kos").is_dir() {
                let orc_root = current.to_path_buf();
                let kos_root = orc_root.join("kos");
                return Ok(Workspace {
                    root: orc_root,
                    kos_root,
                });
            }

            // Are we in a sibling subrepo? Check if parent is aae-orc
            if let Some(parent) = current.parent() {
                if parent.join("charter.md").exists() && parent.join("kos").is_dir() {
                    return Ok(Workspace {
                        root: parent.to_path_buf(),
                        kos_root: parent.join("kos"),
                    });
                }
                current = parent;
            } else {
                break;
            }
        }

        Err(KosError::WorkspaceNotFound {
            start: start.display().to_string(),
        })
    }

    /// Construct a workspace from an explicit path (--workspace or KOS_WORKSPACE).
    /// The path should be the aae-orc root (containing kos/ subdir) or the kos
    /// root itself (containing KOS-charter.md).
    pub fn from_explicit(path: &Path) -> Result<Self> {
        let path = std::fs::canonicalize(path).map_err(KosError::Io)?;

        // Is this the kos root directly?
        if path.join("KOS-charter.md").exists() {
            let kos_root = path.clone();
            let orc_root = path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| kos_root.clone());
            return Ok(Workspace {
                root: orc_root,
                kos_root,
            });
        }

        // Is this the aae-orc root with a kos/ subdir?
        if path.join("kos").is_dir() {
            return Ok(Workspace {
                root: path.clone(),
                kos_root: path.join("kos"),
            });
        }

        Err(KosError::InvalidWorkspace {
            path: path.display().to_string(),
        })
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
}
