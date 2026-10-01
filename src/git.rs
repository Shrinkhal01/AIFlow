use crate::domain::GitInfo;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("Not a git repository or git command failed: {0}")]
    ExecutionFailed(String),
    #[allow(dead_code)]
    #[error("Failed to parse git output: {0}")]
    ParseError(String),
}

/// Inspect the git repository status in a safe, read-only manner
pub fn inspect_git(repo_dir: &Path) -> Result<GitInfo, GitError> {
    // 1. Get current branch name
    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| GitError::ExecutionFailed(e.to_string()))?;

    let mut branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
    if branch.is_empty() {
        // Might be detached HEAD or empty repo
        let head_check = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(repo_dir)
            .output();
        if let Ok(out) = head_check {
            let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !sha.is_empty() {
                branch = format!("detached:{}", sha);
            } else {
                branch = "main (no commits)".to_string();
            }
        } else {
            branch = "unknown".to_string();
        }
    }

    // 2. Get status --porcelain to count modified and untracked files
    let status_output = Command::new("git")
        .args(["status", "--porcelain=v1"])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| GitError::ExecutionFailed(e.to_string()))?;

    let status_str = String::from_utf8_lossy(&status_output.stdout);
    let mut modified_count = 0;
    let mut untracked_count = 0;

    for line in status_str.lines() {
        if line.starts_with("??") {
            untracked_count += 1;
        } else if !line.trim().is_empty() {
            modified_count += 1;
        }
    }

    // 3. Get last commit message
    let log_output = Command::new("git")
        .args(["log", "-n", "1", "--oneline"])
        .current_dir(repo_dir)
        .output();

    let last_commit_message = match log_output {
        Ok(out) if out.status.success() => {
            let msg = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if msg.is_empty() {
                None
            } else {
                Some(msg)
            }
        }
        _ => None,
    };

    Ok(GitInfo {
        branch,
        modified_count,
        untracked_count,
        last_commit_message,
    })
}
