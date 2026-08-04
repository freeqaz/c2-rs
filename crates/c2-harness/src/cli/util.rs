//! Helpers shared by more than one `cli` submodule: the scratch-directory
//! allocator, the `<cpp>` positional accessor, the shared `--cwd`/`--flags-file`
//! dependency edge, and the one-line error formatter.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::Args;

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) fn scratch(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!("c2rs-cli-{tag}-{}-{}-{}", std::process::id(), nanos, n));
    let _ = std::fs::create_dir_all(&d);
    d
}

/// The `<cpp>` positional, from a **parsed** argument set.
///
/// It used to take `rest` and return `rest.first()` verbatim, which meant a
/// flag-shaped first token became the source path: `c2rs diff --help` looked for
/// a file called `--help`. `Args` has already separated options from
/// positionals, so that spelling is not expressible here any more.
pub(crate) fn require_cpp(args: &Args) -> Option<PathBuf> {
    match args.first() {
        Some(p) => Some(PathBuf::from(p)),
        None => {
            eprintln!("{}: expected a <cpp> path", args.cmd());
            None
        }
    }
}

/// The profile plumbing `capture`, `compile` and `census` share, plus the
/// `--cwd` dependency that all three used to drop in silence.
pub(crate) const CPP_PROFILE_REQUIRES: &[(&str, &str)] = &[("--cwd", "--flags-file")];

pub(crate) fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s)
}
