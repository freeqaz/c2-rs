//! **TEMPORARY — lane `w-deadsites`.** Applied and reverted; nothing in this
//! file lands.
//!
//! A first-hit reachability marker. `hit(ix, id)` records, once per index per
//! process, that control reached a site the `w-mutcensus` census mutated. The
//! first hit appends `id` to the file named by `C2RS_DEADPROBE_LOG`; every
//! later hit is one relaxed atomic and nothing else, so instrumenting a site
//! that fires on millions of bodies costs nothing measurable.
//!
//! It is behaviour-preserving by construction: it returns `()` and touches no
//! program state, so an instrumented corpus run must reproduce the baseline's
//! pass/fail/target counts exactly. That identity is the instrument's own
//! self-check.

use std::sync::atomic::{AtomicU64, Ordering};

static SEEN: AtomicU64 = AtomicU64::new(0);

pub(crate) fn hit(ix: u32, id: &str) {
    let bit = 1u64 << ix;
    if SEEN.fetch_or(bit, Ordering::Relaxed) & bit != 0 {
        return;
    }
    let Ok(path) = std::env::var("C2RS_DEADPROBE_LOG") else {
        return;
    };
    // ONE `write_all` of one buffer, never `writeln!`: `writeln!` issues a
    // syscall per format piece, and two processes appending concurrently
    // produced the interleaved line `X2X2` in this lane's first run. An
    // `O_APPEND` write of a whole line is atomic; two of half a line are not.
    use std::io::Write;
    let mut line = String::with_capacity(id.len() + 1);
    line.push_str(id);
    line.push('\n');
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}
