//! **The tag-0x10 ALIAS decode, run over a real corpus and dumped for
//! comparison against an independent implementation.**
//!
//! This lane's whole verification is *two implementations of one disassembly
//! transcript agreeing*: `work/w-emitp/alias.py` (Python, frozen) and
//! `c2_il::gl_alias_table` (Rust, this crate). Aggregate counts agreeing is the
//! weak form; the strong form is the **850 per-TU tables agreeing name for
//! name**, and this test is what produces the Rust side of that comparison.
//!
//! It reads the `.gl` of every entry in a capture-cache index and writes one
//! JSON object per TU. It needs **no toolchain** — `.gl` is already on disk —
//! but it does need a corpus, so:
//!
//! ```sh
//! C2RS_ALIAS_CACHEIDX=work/w-alias/cacheidx.tsv \
//! C2RS_ALIAS_OUT=work/w-alias/rust_alias.jsonl \
//!     cargo test -p c2-il --release --test gl_alias_corpus -- --nocapture
//! ```
//!
//! **Without `C2RS_ALIAS_CACHEIDX` it prints `SKIP` and passes**, the same way
//! every other corpus-dependent instrument in this workspace degrades
//! (`CLAUDE.md`). And, per `docs/STATUS.md` trap 5, it prints a **count** when
//! it runs — a silent pass and a real pass must not look alike.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// One TU's decode, in the order the JSON prints it.
struct Row {
    src: String,
    stats: c2_il::GlAliasStats,
    /// `p−1` and `p+1`: (bound, shape). The null.
    null_m1: (usize, usize),
    null_p1: (usize, usize),
    pairs: Vec<(String, String)>,
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// The `_CL_*gl` inside a capture-cache entry.
fn gl_path(entry: &Path) -> Option<PathBuf> {
    let rd = std::fs::read_dir(entry).ok()?;
    for e in rd.flatten() {
        let n = e.file_name();
        let n = n.to_string_lossy();
        if n.starts_with("_CL_") && n.ends_with("gl") {
            return Some(e.path());
        }
    }
    None
}

fn shape(t: &c2_il::GlAliasTable) -> usize {
    t.stats().shape_e_to_g
}

#[test]
fn gl_alias_corpus_dump() {
    let Ok(idx) = std::env::var("C2RS_ALIAS_CACHEIDX") else {
        println!("SKIP: no corpus — set C2RS_ALIAS_CACHEIDX to a cacheidx.tsv");
        return;
    };
    let idx = PathBuf::from(idx);
    let root = std::env::var("C2RS_ALIAS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| idx.parent().unwrap_or(Path::new(".")).to_path_buf());
    let text = std::fs::read_to_string(&idx).expect("cacheidx unreadable");

    let mut work: Vec<(String, PathBuf)> = Vec::new();
    for line in text.lines() {
        let mut f = line.split('\t');
        let (Some(src), Some(entry)) = (f.next(), f.next()) else {
            continue;
        };
        let p = Path::new(entry);
        let p = if p.is_absolute() {
            p.to_path_buf()
        } else {
            root.join(p)
        };
        work.push((src.to_string(), p));
    }
    assert!(!work.is_empty(), "cacheidx listed no TUs — a run that graded 0 is a failure");

    let next = AtomicUsize::new(0);
    let rows: Mutex<Vec<Row>> = Mutex::new(Vec::new());
    let missing = AtomicUsize::new(0);
    let jobs: usize = std::env::var("C2RS_ALIAS_JOBS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(6);

    std::thread::scope(|s| {
        for _ in 0..jobs.max(1) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= work.len() {
                    break;
                }
                let (src, entry) = &work[i];
                let Some(glp) = gl_path(entry) else {
                    missing.fetch_add(1, Ordering::Relaxed);
                    continue;
                };
                let Ok(gl) = std::fs::read(&glp) else {
                    missing.fetch_add(1, Ordering::Relaxed);
                    continue;
                };
                let t = c2_il::gl_alias_table(&gl);
                let m1 = c2_il::gl_alias_table_shifted(&gl, -1);
                let p1 = c2_il::gl_alias_table_shifted(&gl, 1);
                let row = Row {
                    src: src.clone(),
                    stats: t.stats().clone(),
                    null_m1: (m1.stats().bound, shape(&m1)),
                    null_p1: (p1.stats().bound, shape(&p1)),
                    pairs: t
                        .iter_names()
                        .map(|(a, b)| (a.to_string(), b.to_string()))
                        .collect(),
                };
                rows.lock().unwrap().push(row);
            });
        }
    });

    let mut rows = rows.into_inner().unwrap();
    rows.sort_by(|a, b| a.src.cmp(&b.src));

    // The totals this lane registered against w-emitp's Python.
    let mut tot: BTreeMap<&str, usize> = BTreeMap::new();
    let mut out = String::new();
    for r in &rows {
        let st = &r.stats;
        *tot.entry("tus").or_default() += 1;
        *tot.entry("tag10").or_default() += st.tag10;
        *tot.entry("bound").or_default() += st.bound;
        *tot.entry("shape").or_default() += st.shape_e_to_g;
        *tot.entry("head_fail").or_default() += st.head_fail;
        *tot.entry("rt_fail").or_default() += st.rt_fail;
        *tot.entry("unbound_target").or_default() += st.unbound_target;
        *tot.entry("self_alias").or_default() += st.self_alias;
        *tot.entry("dup").or_default() += st.dup;
        *tot.entry("dom_with_body").or_default() += st.dom_with_body;
        *tot.entry("bound_m1").or_default() += r.null_m1.0;
        *tot.entry("shape_m1").or_default() += r.null_m1.1;
        *tot.entry("bound_p1").or_default() += r.null_p1.0;
        *tot.entry("shape_p1").or_default() += r.null_p1.1;

        out.push_str(&format!(
            "{{\"src\":\"{}\",\"tag10\":{},\"bound\":{},\"shape\":{},\"head_fail\":{},\
             \"rt_fail\":{},\"unbound_target\":{},\"self\":{},\"dup\":{},\
             \"dom_with_body\":{},\"bound_m1\":{},\"shape_m1\":{},\"bound_p1\":{},\
             \"shape_p1\":{},\"pairs\":{{",
            json_escape(&r.src),
            st.tag10,
            st.bound,
            st.shape_e_to_g,
            st.head_fail,
            st.rt_fail,
            st.unbound_target,
            st.self_alias,
            st.dup,
            st.dom_with_body,
            r.null_m1.0,
            r.null_m1.1,
            r.null_p1.0,
            r.null_p1.1,
        ));
        for (i, (a, b)) in r.pairs.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!("\"{}\":\"{}\"", json_escape(a), json_escape(b)));
        }
        out.push_str("}}\n");
    }

    if let Ok(p) = std::env::var("C2RS_ALIAS_OUT") {
        std::fs::write(&p, &out).expect("could not write C2RS_ALIAS_OUT");
        println!("wrote {p}");
    }

    println!(
        "alias-corpus tus {} missing {} tag10 {} bound {} shape {} head_fail {} rt_fail {} \
         unbound_target {} self {} dup {} dom_with_body {} | null-m1 bound {} shape {} | \
         null-p1 bound {} shape {}",
        tot["tus"],
        missing.load(Ordering::Relaxed),
        tot["tag10"],
        tot["bound"],
        tot["shape"],
        tot["head_fail"],
        tot["rt_fail"],
        tot["unbound_target"],
        tot["self_alias"],
        tot["dup"],
        tot["dom_with_body"],
        tot["bound_m1"],
        tot["shape_m1"],
        tot["bound_p1"],
        tot["shape_p1"],
    );

    // The one invariant this test asserts rather than prints: an alias that also
    // has a body would make w-emitp §6 rule 4 suppress a symbol that must be
    // emitted. Measured 0; if a corpus ever breaks it, the rule is unsafe and
    // this is where that is found.
    assert_eq!(
        tot["dom_with_body"], 0,
        "dom(alias) has a bodied member — rule 4 would be a WRONG EMIT here"
    );
}
