use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git command failed: {0}")]
    CommandFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Default)]
pub struct RepoStatus {
    pub branch: String,
    pub modified_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub is_clean: bool,
}

pub struct HybridGitEngine;

impl HybridGitEngine {
    /// Evaluates repository status locally
    pub fn status(repo_path: &Path) -> Result<RepoStatus, GitError> {
        let output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .arg("-b")
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(GitError::CommandFailed(err.to_string()));
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let mut status = RepoStatus::default();

        for line in text.lines() {
            if line.starts_with("##") {
                status.branch = line.trim_start_matches("##").trim().to_string();
            } else if line.starts_with("??") {
                status.untracked_files.push(line[3..].trim().to_string());
            } else if !line.trim().is_empty() {
                status.modified_files.push(line[3..].trim().to_string());
            }
        }

        status.is_clean = status.modified_files.is_empty() && status.untracked_files.is_empty();
        Ok(status)
    }

    /// Stages and commits local changes
    pub fn commit(repo_path: &Path, message: &str) -> Result<String, GitError> {
        let add_out = Command::new("git")
            .arg("add")
            .arg("-A")
            .current_dir(repo_path)
            .output()?;

        if !add_out.status.success() {
            return Err(GitError::CommandFailed(String::from_utf8_lossy(&add_out.stderr).to_string()));
        }

        let commit_out = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .current_dir(repo_path)
            .output()?;

        if !commit_out.status.success() {
            return Err(GitError::CommandFailed(String::from_utf8_lossy(&commit_out.stderr).to_string()));
        }

        Ok(String::from_utf8_lossy(&commit_out.stdout).to_string())
    }

    /// Syncs local repository by pulling with rebase and pushing
    pub fn sync(repo_path: &Path) -> Result<String, GitError> {
        let pull_out = Command::new("git")
            .arg("pull")
            .arg("--rebase")
            .current_dir(repo_path)
            .output()?;

        let mut output_str = String::from_utf8_lossy(&pull_out.stdout).to_string();

        let push_out = Command::new("git")
            .arg("push")
            .current_dir(repo_path)
            .output()?;

        output_str.push_str("\n");
        output_str.push_str(&String::from_utf8_lossy(&push_out.stdout));

        Ok(output_str)
    }

    /// Lists local git branches
    pub fn branches(repo_path: &Path) -> Result<Vec<String>, GitError> {
        let out = Command::new("git")
            .arg("branch")
            .arg("--list")
            .current_dir(repo_path)
            .output()?;

        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(GitError::CommandFailed(err.to_string()));
        }

        let text = String::from_utf8_lossy(&out.stdout);
        let branches = text
            .lines()
            .map(|l| l.trim_start_matches('*').trim().to_string())
            .filter(|b| !b.is_empty())
            .collect();

        Ok(branches)
    }
}
