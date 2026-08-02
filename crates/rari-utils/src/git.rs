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

/// Run `git` with `args` with `root` as current directory.
/// For tests this will run the first arg as command, eg.:
/// Instead of `git mv foo bar` -> `mv foo bar`.
pub fn exec_git_with_test_fallback(args: &[impl AsRef<OsStr>], root: impl AsRef<Path>) -> Output {
    let git = OsStr::new("git");
    let echo = OsStr::new("echo");

    let (command, args) = if cfg!(test) {
        (
            args.first().map(AsRef::as_ref).unwrap_or(echo),
            &args[if args.is_empty() { 0 } else { 1 }..],
        )
    } else {
        (git, args)
    };
    try_exec_git(command, args, root).expect("failed to execute process")
}

pub fn exec_git(args: &[impl AsRef<OsStr>], root: impl AsRef<Path>) -> Output {
    try_exec_git("git", args, root).expect("failed to execute process")
}

fn try_exec_git(
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
