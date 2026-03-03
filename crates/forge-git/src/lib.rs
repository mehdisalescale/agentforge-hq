//! Git worktree management for agent session isolation.
//!
//! Each agent session gets its own worktree so concurrent agents
//! can work on the same repo without conflicts.

use std::path::{Path, PathBuf};
use std::process::Command;

use forge_core::ForgeError;

/// Information about an active forge worktree.
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub session_id: String,
}

/// Create a git worktree for a session.
///
/// Creates a new worktree at `<repo_dir>/.worktrees/<session_id>`
/// on a new branch `forge/<session_id>`.
pub fn create_worktree(repo_dir: &Path, session_id: &str) -> Result<PathBuf, ForgeError> {
    let worktree_dir = repo_dir.join(".worktrees").join(session_id);
    let branch = format!("forge/{}", session_id);

    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            &worktree_dir.to_string_lossy(),
            "-b",
            &branch,
        ])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| ForgeError::Internal(format!("failed to run git worktree add: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ForgeError::Internal(format!(
            "git worktree add failed: {}",
            stderr.trim()
        )));
    }

    Ok(worktree_dir)
}

/// Remove a worktree and delete its branch.
pub fn remove_worktree(repo_dir: &Path, session_id: &str) -> Result<(), ForgeError> {
    let worktree_dir = repo_dir.join(".worktrees").join(session_id);
    let branch = format!("forge/{}", session_id);

    // Remove worktree (--force handles dirty worktrees)
    let _ = Command::new("git")
        .args([
            "worktree",
            "remove",
            &worktree_dir.to_string_lossy(),
            "--force",
        ])
        .current_dir(repo_dir)
        .output();

    // Delete the branch
    let _ = Command::new("git")
        .args(["branch", "-D", &branch])
        .current_dir(repo_dir)
        .output();

    Ok(())
}

/// List all forge-managed worktrees (branches matching `forge/*`).
pub fn list_worktrees(repo_dir: &Path) -> Result<Vec<WorktreeInfo>, ForgeError> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| ForgeError::Internal(format!("failed to run git worktree list: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch: Option<String> = None;

    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(path));
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            current_branch = Some(branch.to_string());
        } else if line.is_empty() {
            if let (Some(path), Some(ref branch)) = (&current_path, &current_branch) {
                if let Some(sid) = branch.strip_prefix("forge/") {
                    worktrees.push(WorktreeInfo {
                        path: path.clone(),
                        branch: branch.clone(),
                        session_id: sid.to_string(),
                    });
                }
            }
            current_path = None;
            current_branch = None;
        }
    }

    // Handle last entry if stdout doesn't end with empty line
    if let (Some(ref path), Some(ref branch)) = (&current_path, &current_branch) {
        if let Some(sid) = branch.strip_prefix("forge/") {
            worktrees.push(WorktreeInfo {
                path: path.clone(),
                branch: branch.clone(),
                session_id: sid.to_string(),
            });
        }
    }

    Ok(worktrees)
}

/// Check if a directory is inside a git repository.
pub fn is_git_repo(dir: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        // git requires at least one commit before worktrees can be created
        Command::new("git")
            .args(["commit", "--allow-empty", "-m", "init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        dir
    }

    #[test]
    fn create_and_remove_worktree() {
        let repo = setup_test_repo();
        let path = create_worktree(repo.path(), "test-session-1").unwrap();
        assert!(path.exists());
        assert!(path.join(".git").exists()); // worktree has a .git file

        remove_worktree(repo.path(), "test-session-1").unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn list_worktrees_finds_forge_branches() {
        let repo = setup_test_repo();
        create_worktree(repo.path(), "sess-a").unwrap();
        create_worktree(repo.path(), "sess-b").unwrap();

        let wts = list_worktrees(repo.path()).unwrap();
        assert_eq!(wts.len(), 2);
        assert!(wts.iter().any(|w| w.session_id == "sess-a"));
        assert!(wts.iter().any(|w| w.session_id == "sess-b"));

        // Cleanup
        remove_worktree(repo.path(), "sess-a").unwrap();
        remove_worktree(repo.path(), "sess-b").unwrap();
    }

    #[test]
    fn list_worktrees_empty_on_fresh_repo() {
        let repo = setup_test_repo();
        let wts = list_worktrees(repo.path()).unwrap();
        assert!(wts.is_empty());
    }

    #[test]
    fn create_duplicate_worktree_fails() {
        let repo = setup_test_repo();
        create_worktree(repo.path(), "dup-sess").unwrap();
        let result = create_worktree(repo.path(), "dup-sess");
        assert!(result.is_err());

        // Cleanup
        remove_worktree(repo.path(), "dup-sess").unwrap();
    }

    #[test]
    fn remove_nonexistent_worktree_is_ok() {
        let repo = setup_test_repo();
        // Should not error — idempotent removal
        let result = remove_worktree(repo.path(), "does-not-exist");
        assert!(result.is_ok());
    }

    #[test]
    fn is_git_repo_true_for_repo() {
        let repo = setup_test_repo();
        assert!(is_git_repo(repo.path()));
    }

    #[test]
    fn is_git_repo_false_for_tmp() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!is_git_repo(dir.path()));
    }
}
