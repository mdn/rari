use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("failed to run git: {0}")]
    Io(#[from] std::io::Error),
    #[error("git command exited with non-zero code {exit_code}: {stderr}")]
    CommandFailed { exit_code: String, stderr: String },
}

pub fn exec_git(args: &[impl AsRef<OsStr>], root: impl AsRef<Path>) -> Output {
    try_exec_git("git", args, root).expect("git command failed")
}

pub fn try_exec_git(
    command: impl AsRef<OsStr>,
    args: &[impl AsRef<OsStr>],
    root: impl AsRef<Path>,
) -> Result<Output, GitError> {
    let output = Command::new(command)
        .args(args)
        .current_dir(root)
        .output()?;

    if !output.status.success() {
        return Err(GitError::CommandFailed {
            exit_code: output
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "None".to_string()),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    Ok(output)
}
