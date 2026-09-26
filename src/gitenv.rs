#![forbid(unsafe_code)]

//! Run git against an explicit repository, whatever the environment says.
//!
//! Git reads a set of variables that name which repository to operate on
//! (`GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and others), and they win
//! over the working directory. Git exports `GIT_DIR` to hooks run from a
//! linked worktree, so a `cargo test` in a pre-push hook inherits it: a test
//! fixture that runs `git init` and `git commit` in a temp dir then commits
//! into the repository being pushed, and a helper given a repo path reads a
//! different one. Every git call that names its repository by path goes
//! through here and drops those variables, so the path is authoritative.

use std::path::Path;
use std::process::Command;

/// The variables `git rev-parse --local-env-vars` lists (git 2.55): the ones
/// that are local to one repository, and which git itself clears when it
/// runs a command in a different repository (a submodule, for example).
pub(crate) const REPO_LOCAL_ENV: &[&str] = &[
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "GIT_OBJECT_DIRECTORY",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_GRAFT_FILE",
    "GIT_INDEX_FILE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REPLACE_REF_BASE",
    "GIT_PREFIX",
    "GIT_SHALLOW_FILE",
    "GIT_COMMON_DIR",
];

/// Remove the repository-local variables from a command's environment.
pub(crate) fn hermetic(cmd: &mut Command) -> &mut Command {
    for var in REPO_LOCAL_ENV {
        cmd.env_remove(var);
    }
    cmd
}

/// A `git` command that runs in `root` and operates on the repository there.
pub(crate) fn git_in(root: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.current_dir(root);
    hermetic(&mut cmd);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git_dir_of(cmd: &mut Command) -> String {
        let out = cmd
            .args(["rev-parse", "--absolute-git-dir"])
            .output()
            .expect("git runnable");
        assert!(out.status.success(), "rev-parse failed");
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    /// The case a pre-push hook creates: GIT_DIR (and friends) point at
    /// another repository. The command still operates on the one at `root`.
    #[test]
    fn inherited_repo_variables_do_not_redirect_git() {
        let repo = tempfile::tempdir().unwrap();
        let decoy = tempfile::tempdir().unwrap();
        assert!(
            git_in(repo.path())
                .args(["init", "-q"])
                .status()
                .unwrap()
                .success()
        );
        assert!(
            git_in(decoy.path())
                .args(["init", "-q"])
                .status()
                .unwrap()
                .success()
        );
        let decoy_git = decoy.path().join(".git");

        // Without the scrub, the inherited GIT_DIR wins over the directory.
        let mut leaky = Command::new("git");
        leaky
            .current_dir(repo.path())
            .env("GIT_DIR", &decoy_git)
            .env("GIT_WORK_TREE", decoy.path())
            .env("GIT_INDEX_FILE", decoy_git.join("index"));
        let leaked = git_dir_of(&mut leaky);
        assert!(
            leaked.ends_with(&*decoy_git.file_name().unwrap().to_string_lossy())
                && leaked.contains(&*decoy.path().file_name().unwrap().to_string_lossy()),
            "control: the variables should redirect an unscrubbed git, got {leaked}"
        );

        // With it, the directory is authoritative.
        let mut scrubbed = Command::new("git");
        scrubbed
            .current_dir(repo.path())
            .env("GIT_DIR", &decoy_git)
            .env("GIT_WORK_TREE", decoy.path())
            .env("GIT_INDEX_FILE", decoy_git.join("index"));
        hermetic(&mut scrubbed);
        let resolved = git_dir_of(&mut scrubbed);
        assert!(
            resolved.contains(&*repo.path().file_name().unwrap().to_string_lossy()),
            "git operated on {resolved}, not the repository at {}",
            repo.path().display()
        );
    }
}
