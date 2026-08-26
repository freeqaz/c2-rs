//! Helpers shared by more than one `cli` submodule: the scratch-directory
//! allocator, the `<cpp>` positional accessor, the shared `--cwd`/`--flags-file`
//! dependency edge, and the one-line error formatter.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::Args;

// PROV[N] not load-bearing — a process-local counter for unique scratch names. Scratch state.
static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch_path(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("c2rs-cli-{tag}-{}-{}-{}", std::process::id(), nanos, n))
}

/// A working directory with an owner, so that "who deletes this" is a property
/// of the value and not of whether every `return` in a 900-line handler
/// remembered to call `remove_dir_all`.
///
/// Every one of these handlers already *intended* to delete its scratch dir --
/// all but two spelled out a `remove_dir_all` on the paths their author thought
/// of. The tail call cannot run on the paths nobody thinks of: a panic unwinding
/// out of a decoder, or an early `return` added later. That gap left 2,376
/// directories in `$TMPDIR` here (256 MB of them under `census`, which *has* a
/// tail `remove_dir_all` on all three of its exits). `Drop` runs on the unwind
/// path too, which is exactly the set the manual calls were missing.
///
/// The owner flag is the other half. `gap`, `listing-scan` and `prefilter` take
/// `--work DIR`, and there the directory is the user's, named on their command
/// line: we use it and leave it alone. Only a dir this process minted in
/// `$TMPDIR` is ours to remove.
///
/// Nothing is lost by removing it. These are *containers*: the scans create and
/// delete a per-TU subdir inside (`gap/scan.rs`, `listing.rs`), so the container
/// is empty by the time the run ends -- 1,923 of the 1,924 leaked `gap` dirs
/// here were empty, the exception being a scan still running. Command output
/// goes where the user asked (`--jsonl`, `--factors-tsv`, `--keep-il`, stdout),
/// never here.
pub(crate) struct Scratch {
    path: PathBuf,
    owned: bool,
}

impl Scratch {
    /// A private scratch dir under `$TMPDIR`, removed when this value drops.
    pub(crate) fn new(tag: &str) -> Self {
        let path = scratch_path(tag);
        let _ = std::fs::create_dir_all(&path);
        Self { path, owned: true }
    }

    /// The `--work DIR` seam: the user's directory if they named one -- left on
    /// disk, and not created here, exactly as before -- else a private scratch
    /// dir we own and remove.
    ///
    /// Passing `--work` is therefore the documented way to keep the working
    /// tree of a run for inspection.
    pub(crate) fn or_work(work: Option<PathBuf>, tag: &str) -> Self {
        match work {
            Some(path) => Self { path, owned: false },
            None => Self::new(tag),
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl std::ops::Deref for Scratch {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<Path> for Scratch {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if self.owned {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
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
/// PROV[N] not load-bearing — an argument-dependency table for this crate's own CLI (`--cwd` requires `--flags-file`).
pub(crate) const CPP_PROFILE_REQUIRES: &[(&str, &str)] = &[("--cwd", "--flags-file")];

pub(crate) fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ordinary path: a scratch dir, with contents, is gone after its guard
    /// drops. The old `scratch()` returned a bare `PathBuf` and left this to a
    /// tail call in each handler.
    #[test]
    fn owned_scratch_is_removed_on_drop() {
        let path = {
            let s = Scratch::new("selftest-drop");
            std::fs::write(s.join("bundle.ex"), b"payload").unwrap();
            assert!(s.path().is_dir());
            s.path().to_path_buf()
        };
        assert!(!path.exists(), "scratch dir survived its guard: {}", path.display());
    }

    /// The path that actually leaked. `census` has a `remove_dir_all` on all
    /// three of its exits and still left 385 directories (256 MB) in `$TMPDIR`,
    /// because a panic unwinding out of the decode loop reaches none of them.
    /// `Drop` runs during unwind, so this is the case the manual calls could not
    /// cover.
    #[test]
    fn owned_scratch_is_removed_when_the_handler_panics() {
        let (tx, rx) = std::sync::mpsc::channel();
        let r = std::panic::catch_unwind(move || {
            let s = Scratch::new("selftest-panic");
            std::fs::write(s.join("bundle.ex"), b"payload").unwrap();
            tx.send(s.path().to_path_buf()).unwrap();
            panic!("decoder blew up mid-census");
        });
        assert!(r.is_err(), "the test's own panic did not propagate");
        let path = rx.recv().expect("guard never reported its path");
        assert!(
            !path.exists(),
            "scratch dir survived a panic: {}",
            path.display()
        );
    }

    /// The `--work DIR` contract: the directory is the user's, so it is neither
    /// created nor deleted here. `prefilter` used to delete it -- an
    /// unconditional `remove_dir_all` on a path the harness did not mint.
    #[test]
    fn user_supplied_work_dir_is_never_removed() {
        let dir = std::env::temp_dir().join(format!(
            "c2rs-selftest-userwork-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("keepme"), b"the user's file").unwrap();
        {
            let s = Scratch::or_work(Some(dir.clone()), "unused-tag");
            assert_eq!(s.path(), dir);
        }
        assert!(dir.join("keepme").is_file(), "--work DIR was deleted");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// `--work` absent falls back to a private dir we own and remove, so the
    /// fallback cannot leak the way `gap` did.
    #[test]
    fn absent_work_falls_back_to_an_owned_scratch() {
        let path = {
            let s = Scratch::or_work(None, "selftest-fallback");
            assert!(s.path().is_dir());
            s.path().to_path_buf()
        };
        assert!(!path.exists(), "fallback scratch leaked: {}", path.display());
    }

    /// `or_work` must not create the user's directory: the scans call
    /// `create_dir_all` on it themselves, and that was the behaviour before.
    #[test]
    fn user_supplied_work_dir_is_not_created_here() {
        let dir = std::env::temp_dir().join(format!(
            "c2rs-selftest-nocreate-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(!dir.exists());
        {
            let _s = Scratch::or_work(Some(dir.clone()), "unused-tag");
        }
        assert!(!dir.exists(), "or_work created the user's --work dir");
    }
}
