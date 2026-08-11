use std::ffi::OsStr;
use std::path::Path;
use std::process::Output;

use rari_utils::git::try_exec_git;

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
