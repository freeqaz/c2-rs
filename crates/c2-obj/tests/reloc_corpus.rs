//! **The corpus instrument for lane w-reloc**: decode every relocation in a
//! directory of real reference objs and publish the histogram.
//!
//! Point it at a tree of `.obj` files with `C2RS_RELOC_OBJS`; without that it
//! skips, like every other toolchain-dependent test here. Build the tree with
//! `work/w-reloc/build_corpus.sh`, which wraps `work/w-frame/refobj.sh` so the
//! objs are compiled at the **workload's own flags** — an obj captured at `/Ox`
//! is a different compilation from the one `c2rs gap` grades (board #195).
//!
//! ```sh
//! C2RS_RELOC_OBJS=work/w-reloc/objs \
//!   cargo test -p c2-obj --release --test reloc_corpus -- --nocapture
//! ```
//!
//! **This test cannot pass by producing nothing.** Absence read as success is
//! this project's most-recorded failure mode, and a relocation histogram is an
//! easy place for it: a reader that refuses every obj prints an empty table that
//! looks exactly like "this workload uses a narrow vocabulary". So when the
//! corpus directory exists, the assertions demand a *positive* obj count, a
//! positive record count, and zero decode refusals — and the run prints all
//! three next to the histogram rather than a status word.

use c2_obj::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Every `.obj` under `root`, at most `depth` levels down. Bounded on purpose:
/// the standing rule on this box forbids unbounded recursive walks from
/// anywhere near the repo root (`work/capture-cache` has caused two kernel OOM
/// kills), and a corpus directory is two levels deep by construction.
fn objs_under(root: &Path, depth: usize) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else {
        return out;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if depth > 0 {
                out.extend(objs_under(&p, depth - 1));
            }
        } else if p.extension().map(|x| x == "obj").unwrap_or(false) {
            out.push(p);
        }
    }
    out.sort();
    out
}

/// The section name a relocation's `section` index refers to, for the histogram.
/// A `$`-suffixed COMDAT name is folded to its stem (`.text$yd` → `.text`,
/// `?f@@YAHH@Z` sections excepted since c2 names those `.text` too), because the
/// interesting axis is *which kind of section carries which relocation*, and an
/// obj has one `.text` per emitted function.
fn section_bucket(name: &str) -> &str {
    match name.split_once('$') {
        Some((stem, _)) => stem,
        None => name,
    }
}

#[test]
fn the_workload_relocation_histogram() {
    let Ok(root) = std::env::var("C2RS_RELOC_OBJS") else {
        eprintln!("SKIP: C2RS_RELOC_OBJS unset — no reference-obj corpus to decode");
        return;
    };
    let root = PathBuf::from(root);
    let files = objs_under(&root, 3);
    // **Set-but-empty is a failure, not a skip.** The first run of this test
    // printed `SKIP: no .obj under work/w-reloc/objs` with 871 objs sitting in
    // that directory — `cargo test` runs with the *crate* as its working
    // directory, so the relative path resolved under `crates/c2-obj/`. A skip
    // there is absence read as success with the corpus present and unread.
    // Unset means "no corpus"; set means "there is a corpus, decode it".
    assert!(
        !files.is_empty(),
        "C2RS_RELOC_OBJS is set to {} but no .obj is under it — pass an ABSOLUTE \
         path; `cargo test` runs from the crate directory, not the repo root",
        root.display()
    );

    let mut n_obj = 0usize;
    let mut n_refused = 0usize;
    let mut refused_names: Vec<String> = Vec::new();
    let mut n_reloc = 0usize;
    // base type -> count, and base type -> (section bucket -> count)
    let mut by_type: BTreeMap<u16, usize> = BTreeMap::new();
    let mut by_type_sec: BTreeMap<(u16, String), usize> = BTreeMap::new();
    let mut by_flags: BTreeMap<u16, usize> = BTreeMap::new();
    let mut unknown_type: BTreeMap<u16, usize> = BTreeMap::new();
    // PAIR: how many, and how many with a nonzero SymbolTableIndex.
    let mut n_pair = 0usize;
    let mut n_pair_nonzero = 0usize;
    let mut pair_examples: Vec<String> = Vec::new();
    // REFHI/REFLO pairing: how many took a following PAIR at the same VA.
    let mut n_hi_lo = 0usize;
    let mut n_hi_lo_paired = 0usize;
    let mut n_other_paired = 0usize;
    let mut max_sec_relocs = 0usize;
    // Is `SymbolTableIndex` actually an index? A misaligned decode produces wild
    // values here, so this doubles as a check that the 10-byte stride is right.
    let mut n_indexable = 0usize;
    let mut n_sym_out_of_range = 0usize;
    // REFHI/REFLO balance per section — c2 is free to emit a REFLO off an
    // already-materialized high half, and the corpus says whether it does.
    let mut n_lone_lo = 0usize;
    let mut n_lone_hi = 0usize;

    for f in &files {
        let Ok(bytes) = std::fs::read(f) else { continue };
        let img = ObjImage::new(bytes);
        n_obj += 1;
        let Some(recs) = img.relocations() else {
            n_refused += 1;
            refused_names.push(f.display().to_string());
            continue;
        };
        let names = img.section_names().unwrap_or_default();
        let nsym = u32::from_le_bytes([
            img.as_bytes()[12],
            img.as_bytes()[13],
            img.as_bytes()[14],
            img.as_bytes()[15],
        ]);
        n_reloc += recs.len();
        let mut per_sec: BTreeMap<usize, usize> = BTreeMap::new();
        let mut hi_lo: BTreeMap<usize, (usize, usize)> = BTreeMap::new();
        for (i, r) in recs.iter().enumerate() {
            *per_sec.entry(r.section).or_default() += 1;
            let bucket = names
                .get(r.section)
                .map(|n| section_bucket(n).to_string())
                .unwrap_or_else(|| format!("<sec {}>", r.section));
            *by_type.entry(r.base()).or_default() += 1;
            *by_type_sec.entry((r.base(), bucket)).or_default() += 1;
            if r.flags() != 0 || r.unknown_bits() != 0 {
                *by_flags.entry(r.ty & !IMAGE_REL_PPC_TYPEMASK).or_default() += 1;
            }
            if reloc_type_name(r.base()).is_none() {
                *unknown_type.entry(r.base()).or_default() += 1;
            }
            if r.base() == IMAGE_REL_PPC_PAIR {
                n_pair += 1;
                if r.sym != 0 {
                    n_pair_nonzero += 1;
                    if pair_examples.len() < 8 {
                        pair_examples.push(format!(
                            "{}: sec {} va 0x{:x} sym 0x{:x}",
                            f.display(),
                            r.section,
                            r.va,
                            r.sym
                        ));
                    }
                }
                // What did this PAIR follow?
                if let Some(prev) = i.checked_sub(1).and_then(|j| recs.get(j)) {
                    let b = prev.base();
                    if b != IMAGE_REL_PPC_REFHI
                        && b != IMAGE_REL_PPC_REFLO
                        && b != IMAGE_REL_PPC_SECRELHI
                        && b != IMAGE_REL_PPC_SECRELLO
                    {
                        n_other_paired += 1;
                    }
                }
            }
            if r.sym_is_an_index() {
                n_indexable += 1;
                if r.sym >= nsym {
                    n_sym_out_of_range += 1;
                }
            }
            if r.base() == IMAGE_REL_PPC_REFHI {
                hi_lo.entry(r.section).or_default().0 += 1;
            }
            if r.base() == IMAGE_REL_PPC_REFLO {
                hi_lo.entry(r.section).or_default().1 += 1;
            }
            if r.base() == IMAGE_REL_PPC_REFHI || r.base() == IMAGE_REL_PPC_REFLO {
                n_hi_lo += 1;
                if let Some(next) = recs.get(i + 1) {
                    if next.base() == IMAGE_REL_PPC_PAIR
                        && next.va == r.va
                        && next.section == r.section
                    {
                        n_hi_lo_paired += 1;
                    }
                }
            }
        }
        max_sec_relocs = max_sec_relocs.max(per_sec.values().copied().max().unwrap_or(0));
        for (hi, lo) in hi_lo.values() {
            n_lone_lo += lo.saturating_sub(*hi);
            n_lone_hi += hi.saturating_sub(*lo);
        }
    }

    println!("\n=== w-reloc corpus histogram ===");
    println!("objs decoded          : {n_obj}");
    println!("objs REFUSED by reader: {n_refused}");
    for n in refused_names.iter().take(8) {
        println!("    refused: {n}");
    }
    println!("relocation records    : {n_reloc}");
    println!("max records in one section: {max_sec_relocs}");
    println!("\n-- base type --");
    for (t, c) in &by_type {
        let name = reloc_type_name(*t).unwrap_or("<NOT IN TABLE>");
        println!("  0x{t:04X} {name:<10} {c:>9}");
    }
    println!("\n-- base type x section --");
    for ((t, sec), c) in &by_type_sec {
        let name = reloc_type_name(*t).unwrap_or("<NOT IN TABLE>");
        println!("  {name:<10} {sec:<12} {c:>9}");
    }
    println!("\n-- packed high byte (modifier bits), nonzero only --");
    if by_flags.is_empty() {
        println!("  (none — every Type word in the corpus has a zero high byte)");
    }
    for (f, c) in &by_flags {
        println!("  0x{f:04X} {c:>9}");
    }
    println!("\n-- PAIR --");
    println!("  PAIR records            : {n_pair}");
    println!("  PAIR with sym != 0      : {n_pair_nonzero}");
    for e in &pair_examples {
        println!("      {e}");
    }
    println!("  PAIR not following a HI/LO: {n_other_paired}");
    println!("  REFHI+REFLO records     : {n_hi_lo}");
    println!("  ...of which PAIR-followed at the same VA: {n_hi_lo_paired}");
    println!("  sections with surplus REFLO (lone lo): {n_lone_lo}");
    println!("  sections with surplus REFHI (lone hi): {n_lone_hi}");
    println!("\n-- SymbolTableIndex as an index (PAIR excluded) --");
    println!("  records whose sym field IS an index : {n_indexable}");
    println!("  ...with sym >= NumberOfSymbols      : {n_sym_out_of_range}");
    println!("\n-- base types with no row in the ported table --");
    if unknown_type.is_empty() {
        println!("  (none — every base type in the corpus is in the table)");
    }
    for (t, c) in &unknown_type {
        println!("  0x{t:04X} {c:>9}");
    }
    println!("=== end ===\n");

    // POSITIVE claims. None of these can be satisfied by decoding nothing.
    assert!(n_obj >= 1, "no obj was read at all");
    assert_eq!(n_refused, 0, "the reader refused {n_refused} of {n_obj} objs");
    assert!(
        n_reloc > 1000,
        "only {n_reloc} relocations across {n_obj} objs — a real workload obj \
         carries hundreds, so this is the reader failing quietly, not a narrow corpus"
    );
    assert!(
        by_type.len() >= 4,
        "only {} distinct base types across {n_reloc} records",
        by_type.len()
    );
    // A misaligned 10-byte stride would scatter `sym` across the whole u32
    // range, so this is simultaneously a claim about the format and a control on
    // the reader itself.
    assert!(n_indexable > 1000, "only {n_indexable} index-bearing records");
    assert_eq!(
        n_sym_out_of_range, 0,
        "{n_sym_out_of_range} of {n_indexable} non-PAIR records carry a \
         SymbolTableIndex past the end of the symbol table"
    );
    assert!(
        unknown_type.is_empty(),
        "base types with no row in the ported table: {unknown_type:?} — a type in \
         the corpus we do not know about is a finding, not a rounding error"
    );
}
