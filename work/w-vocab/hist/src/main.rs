//! `vocab-hist` — decompose `c2rs gap`'s `vocab-gap` bucket into named causes.
//!
//! Reads the same workload list and flags `c2rs gap` reads, pulls each TU's
//! bundle from the same content-addressed capture cache, and prints:
//!
//!   * the FIRST-cause histogram — what `IlBundle::functions()` actually stops
//!     on, which is the bucket `gap.rs` collapses to one label;
//!   * the INDEPENDENT-cause histogram — every gate that would fire, so a
//!     repair's ceiling can be counted;
//!   * the ceiling of the AB-g type-index-window repair, with no discount;
//!   * the anti-drift control `causes.is_empty() == decodes`, known answer 0
//!     disagreements.
//!
//! Usage:
//!   vocab-hist <files.txt> <flags.txt> <cwd> [jobs]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use c2_harness::capture_cache::CaptureCache;
use c2_il::func::DecodeCauses;
use c2_reference::Toolchain;

struct Row {
    src: String,
    captured: bool,
    d: DecodeCauses,
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() < 3 {
        eprintln!("usage: vocab-hist <files.txt> <flags.txt> <cwd> [jobs]");
        std::process::exit(2);
    }
    let jobs: usize = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(6);
    let sources: Vec<String> = std::fs::read_to_string(&a[0])
        .expect("files.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect();
    let flags: Vec<String> = std::fs::read_to_string(&a[1])
        .expect("flags.txt")
        .split_whitespace()
        .map(str::to_string)
        .collect();
    let cwd = PathBuf::from(&a[2]);

    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let repo = c2_harness::provenance::main_repo_root();
    let cache = CaptureCache::new(repo.join("work/capture-cache"), &tc, Some(&cwd), 0)
        .expect("capture cache");
    let work = repo.join("work/w-vocab/scratch");
    let _ = std::fs::create_dir_all(&work);

    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<Option<Row>>> = Mutex::new((0..sources.len()).map(|_| None).collect());
    std::thread::scope(|s| {
        for t in 0..jobs {
            let (tc, cache, sources, flags, cwd, next, out, work) =
                (&tc, &cache, &sources, &flags, &cwd, &next, &out, &work);
            s.spawn(move || {
                let mine: PathBuf = work.join(format!("w{t}"));
                let _ = std::fs::create_dir_all(&mine);
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= sources.len() {
                        return;
                    }
                    let src = &sources[i];
                    let (r, _) = cache.capture(tc, src, flags, Some(cwd.as_path()), &mine);
                    let row = match r {
                        Ok(c) => Row {
                            src: src.clone(),
                            captured: true,
                            d: c.bundle.decode_causes(),
                        },
                        Err(_) => Row {
                            src: src.clone(),
                            captured: false,
                            d: DecodeCauses::default(),
                        },
                    };
                    out.lock().unwrap()[i] = Some(row);
                }
            });
        }
    });
    let rows: Vec<Row> = out.into_inner().unwrap().into_iter().flatten().collect();
    report(&rows);
}

fn bump(m: &mut BTreeMap<String, usize>, k: &str) {
    *m.entry(k.to_string()).or_insert(0) += 1;
}

fn ranked(m: &BTreeMap<String, usize>) -> Vec<(&String, &usize)> {
    let mut v: Vec<_> = m.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    v
}

fn report(rows: &[Row]) {
    let captured: Vec<&Row> = rows.iter().filter(|r| r.captured).collect();
    let fails: Vec<&&Row> = captured.iter().filter(|r| !r.d.decodes).collect();

    println!("VOCAB-GAP DECOMPOSITION — {} TUs listed", rows.len());
    println!(
        "  captured {}  |  capture-fail {}  |  decodes {}  |  DOES NOT decode {}",
        captured.len(),
        rows.len() - captured.len(),
        captured.len() - fails.len(),
        fails.len()
    );

    // ── the anti-drift control ────────────────────────────────────────────
    let drift = captured
        .iter()
        .filter(|r| r.d.causes.is_empty() != r.d.decodes)
        .count();
    let drift_fn_only = captured
        .iter()
        .filter(|r| r.d.causes.is_empty() != r.d.decodes && r.d.whole_tu)
        .count();
    println!(
        "  ANTI-DRIFT CONTROL  causes.is_empty() != decodes on {drift} TU(s) \
         ({drift_fn_only} of them accepted by the WHOLE-TU path, which is the only \
         licensed disagreement)"
    );

    // ── first cause: what functions() actually stops on ───────────────────
    let mut first = BTreeMap::new();
    for r in &fails {
        bump(&mut first, r.d.first.unwrap_or("<none — see drift>"));
    }
    println!("\nFIRST CAUSE (what `functions()` short-circuits on) over {} TUs", fails.len());
    for (k, n) in ranked(&first) {
        println!("  {n:6}  {k}");
    }

    // ── independent causes: every gate that fires ─────────────────────────
    let mut ind = BTreeMap::new();
    for r in &fails {
        for c in &r.d.causes {
            bump(&mut ind, c);
        }
    }
    println!("\nINDEPENDENT CAUSES (every gate that fires; a TU may appear in several)");
    for (k, n) in ranked(&ind) {
        println!("  {n:6}  {k}");
    }

    // ── how many causes does one TU carry ─────────────────────────────────
    let mut ncauses = BTreeMap::new();
    for r in &fails {
        bump(&mut ncauses, &format!("{:02} cause(s)", r.d.causes.len()));
    }
    println!("\nCAUSES PER TU (the reason a single repair converts little)");
    for (k, n) in ranked(&ncauses) {
        println!("  {n:6}  {k}");
    }

    // ── the AB-g window ───────────────────────────────────────────────────
    let win_narrows = captured
        .iter()
        .filter(|r| r.d.records_wide > r.d.records_gate)
        .count();
    let win_blocks = captured.iter().filter(|r| r.d.window_blocks_binding).count();
    println!("\nAB-g — THE TYPE-INDEX WINDOW");
    println!(
        "  TUs where the wide framing sees MORE records than the gate's: {win_narrows} of {}",
        captured.len()
    );
    println!(
        "  TUs where dropping the window ALONE makes the binding succeed:  {win_blocks}"
    );

    // The ceiling, with NO discount: a TU converts only if the window is the
    // sole thing between it and `functions()` accepting — i.e. every other
    // cause it carries is a binding cause the wide framing removes.
    let bind_causes = [
        c2_il::func::cause::GL_NAME_TOO_FAR,
        c2_il::func::cause::GL_NAME_NOT_MANGLED,
        c2_il::func::cause::GL_RUN_ENDS_26,
        c2_il::func::cause::GL_DLLEXPORT,
        c2_il::func::cause::GL_26_INTRODUCED,
        c2_il::func::cause::BIND_COUNT,
        c2_il::func::cause::BIND_OFFSET,
    ];
    let sole: Vec<&&Row> = fails
        .iter()
        .filter(|r| r.d.window_blocks_binding && r.d.causes.iter().all(|c| bind_causes.contains(c)))
        .copied()
        .collect();
    println!(
        "  CEILING, no discount — TUs whose ONLY causes are binding causes the\n  \
         wide framing removes (so they would decode): {}",
        sole.len()
    );
    for r in sole.iter().take(40) {
        println!("      {}", r.src);
    }
    // …and what blocks the rest of the window population.
    let mut blockers = BTreeMap::new();
    for r in fails
        .iter()
        .filter(|r| r.d.window_blocks_binding && !r.d.causes.iter().all(|c| bind_causes.contains(c)))
    {
        for c in r.d.causes.iter().filter(|c| !bind_causes.contains(c)) {
            bump(&mut blockers, c);
        }
    }
    if !blockers.is_empty() {
        println!("  what still blocks the other window TUs:");
        for (k, n) in ranked(&blockers) {
            println!("    {n:6}  {k}");
        }
    }

    // ── the TUs a BODY-CLASS widening could reach ─────────────────────────
    let mut near: Vec<&&Row> = fails
        .iter()
        .filter(|r| r.d.first == Some(c2_il::func::cause::BODY_DECODE))
        .copied()
        .collect();
    near.sort_by_key(|r| r.d.bodies_out_of_class);
    println!(
        "\nTHE TUs WHOSE ONLY FIRST BLOCKER IS A BODY ({}) — binding, drectve and\n\
         framing are all already satisfied here (blocked / segments / other causes | src)",
        near.len()
    );
    for r in &near {
        let others: Vec<&str> = r
            .d
            .causes
            .iter()
            .copied()
            .filter(|&c| c != c2_il::func::cause::BODY_DECODE)
            .collect();
        println!(
            "  {:5} / {:5}  {:30}  {}",
            r.d.bodies_out_of_class,
            r.d.segments,
            if others.is_empty() { "-".to_string() } else { others.join(",") },
            r.src
        );
    }

    // How far is the nearest TU from an all-bodies-decode verdict?
    let mut dist = BTreeMap::new();
    for r in &fails {
        let b = r.d.bodies_out_of_class;
        let k = if b == 0 {
            "0".to_string()
        } else if b <= 1 {
            "1".to_string()
        } else if b <= 10 {
            "2..10".to_string()
        } else if b <= 100 {
            "11..100".to_string()
        } else if b <= 1000 {
            "101..1000".to_string()
        } else {
            ">1000".to_string()
        };
        bump(&mut dist, &k);
    }
    println!("\nOUT-OF-CLASS BODIES PER NON-DECODING TU (the real distance)");
    for (k, n) in ranked(&dist) {
        println!("  {n:6}  {k}");
    }

    // ── the absolute decode ceiling, whatever the binding does ────────────
    let all_bodies_ok = captured
        .iter()
        .filter(|r| r.d.segments > 0 && r.d.bodies_out_of_class == 0)
        .count();
    let empty_ex = captured.iter().filter(|r| r.d.segments == 0).count();
    println!("\nTHE BODY-DECODE BOUND (independent of every binding question)");
    println!(
        "  TUs with at least one `4F 1F` segment and ZERO out-of-class bodies: {all_bodies_ok}"
    );
    println!("  TUs with no `4F 1F` segment at all (the empty-module shape):        {empty_ex}");
    println!(
        "  => no reader repair can take `functions()` past {} TUs, ever.",
        all_bodies_ok + empty_ex
    );

    // ── record-count spread, the arity axis of the two framings ───────────
    let (mut g, mut w) = (0usize, 0usize);
    for r in &captured {
        g += r.d.records_gate;
        w += r.d.records_wide;
    }
    println!("\nFRAMING ARITY over the captured TUs");
    println!("  `.gl` records the gate's framing sees: {g}");
    println!("  `.gl` records the wide framing sees:   {w}");
    println!("  `.ex` segments:                        {}", captured.iter().map(|r| r.d.segments).sum::<usize>());

    // machine-readable tail
    println!("\nVOCAB-METRICS");
    println!("vocab-metric listed {}", rows.len());
    println!("vocab-metric captured {}", captured.len());
    println!("vocab-metric not-decoding {}", fails.len());
    println!("vocab-metric drift {drift}");
    println!("vocab-metric window-narrows {win_narrows}");
    println!("vocab-metric window-blocks-binding {win_blocks}");
    println!("vocab-metric window-ceiling {}", sole.len());
    println!("vocab-metric bodies-all-in-class {all_bodies_ok}");
    println!("vocab-metric empty-ex {empty_ex}");
    println!("vocab-metric decode-ceiling {}", all_bodies_ok + empty_ex);
    for (k, n) in ranked(&first) {
        println!("vocab-metric first|{k} {n}");
    }
    for (k, n) in ranked(&ind) {
        println!("vocab-metric independent|{k} {n}");
    }
}

#[allow(dead_code)]
fn unused(_: &Path) {}
