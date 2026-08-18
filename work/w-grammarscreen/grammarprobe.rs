//! **TEMPORARY — lane `w-grammarscreen`.** Applied and reverted; nothing in
//! this file lands. `git diff master..HEAD -- crates fixtures scripts` is
//! EMPTY at the tip.
//!
//! A first-hit reachability marker for the **grammar fail-closed class** —
//! every textual `blk(` / `blk_type(` / `Block::refuse(` call site in
//! `crates/c2-il/src`. `w-mutcensus` §2.1 dropped that class with a count
//! (1,227 raw `blk(` grep lines) because a mutation census over it is ≈ 5 days
//! serial; `w-deadsites` F1 sized a per-site bitmask screen at ~20 runs.
//!
//! This is cheaper than both and needs **no per-site edit at all**: the three
//! constructors are marked `#[track_caller]` and ask
//! `std::panic::Location::caller()` who called them. One `&'static Location`
//! exists per call site, so its ADDRESS is the site key, and the whole
//! 1,336-site population is screened by three one-line insertions.
//!
//! Behaviour-preserving by construction: `hit` returns `()`, touches no
//! program state, and reads only an environment variable. An instrumented
//! corpus run must therefore reproduce the clean baseline's pass/fail/target
//! counts exactly — that identity is the instrument's own validity check
//! (`w-deadsites` §3.1).
//!
//! Two modes, selected by environment:
//!   `C2RS_GRAMMARPROBE_LOG=<path>`    first-hit marker; appends `file:line:col`
//!                                     once per site per thread.
//!   `C2RS_GRAMMARPROBE_PANIC=<path>`  confirmation: `<path>` lists the sites
//!                                     the screen found REACHED, one
//!                                     `file:line:col` per line; reaching any
//!                                     site NOT in that list `panic!()`s and
//!                                     names itself. A run that completes clean
//!                                     confirms every quiet site at once
//!                                     (`#3246`'s named probe, batched).

use std::cell::RefCell;
use std::collections::HashSet;
use std::io::Write;
use std::panic::Location;
use std::sync::OnceLock;

thread_local! {
    /// Addresses of the `&'static Location`s this thread has already recorded.
    /// Thread-local rather than a global `Mutex<HashSet>` on purpose: `blk` is
    /// called tens of millions of times over the workload scan and a contended
    /// global lock would change the run's duration by more than its counts.
    static SEEN: RefCell<HashSet<usize>> = RefCell::new(HashSet::new());
}

/// The REACHED set, for panic mode. Read once, from the file named by
/// `C2RS_GRAMMARPROBE_PANIC`.
static REACHED: OnceLock<Option<HashSet<(String, u32, u32)>>> = OnceLock::new();

fn reached() -> &'static Option<HashSet<(String, u32, u32)>> {
    REACHED.get_or_init(|| {
        let path = std::env::var("C2RS_GRAMMARPROBE_PANIC").ok()?;
        let text = std::fs::read_to_string(path).ok()?;
        let mut set = HashSet::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut it = line.rsplitn(3, ':');
            let col: u32 = it.next()?.parse().ok()?;
            let ln: u32 = it.next()?.parse().ok()?;
            let file = it.next()?.to_string();
            set.insert((file, ln, col));
        }
        Some(set)
    })
}

#[inline]
pub(crate) fn hit(loc: &'static Location<'static>) {
    let key = loc as *const Location<'static> as usize;
    // `try_with`, never `with`: a `thread_local!` access during thread
    // destruction PANICS, and a probe that can panic is a probe that can
    // invalidate the run it is measuring. On failure the dedup is skipped and
    // the hit is recorded anyway — duplicate lines cost a `sort -u`, whereas a
    // dropped hit is an UNDER-count, which is the flattering direction and the
    // one #3288 says survives review.
    let first = SEEN.try_with(|s| s.borrow_mut().insert(key)).unwrap_or(true);
    if first {
        record(loc);
    }
}

#[cold]
#[inline(never)]
fn record(loc: &'static Location<'static>) {
    if let Some(set) = reached() {
        if !set.contains(&(loc.file().to_string(), loc.line(), loc.column())) {
            panic!(
                "w-grammarscreen QUIET SITE REACHED {}:{}:{}",
                loc.file(),
                loc.line(),
                loc.column()
            );
        }
        return;
    }
    let Ok(path) = std::env::var("C2RS_GRAMMARPROBE_LOG") else {
        return;
    };
    // ONE `write_all` of one buffer, never `writeln!`: `writeln!` issues a
    // syscall per format piece and two processes appending concurrently
    // interleave half-lines. An `O_APPEND` write of a whole line is atomic.
    let mut line = String::with_capacity(loc.file().len() + 16);
    line.push_str(loc.file());
    line.push(':');
    line.push_str(&loc.line().to_string());
    line.push(':');
    line.push_str(&loc.column().to_string());
    line.push('\n');
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}
