use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output};

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
    exec_git_internal(command, args, root)
}

pub fn exec_git(args: &[impl AsRef<OsStr>], root: impl AsRef<Path>) -> Output {
    
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("failed to execute process")
}

fn exec_git_internal(
    command: impl AsRef<OsStr>,
    args: &[impl AsRef<OsStr>],
    root: impl AsRef<Path>,
) -> Output {
    
    Command::new(command)
        .args(args)
        .current_dir(root)
        .output()
        .expect("failed to execute process")
}
