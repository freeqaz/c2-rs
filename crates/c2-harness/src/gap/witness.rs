//! The env-gated scratch dumps and the emitted-symbol **witness list** (board
//! #159). Split out of `gap.rs` unchanged; see [`super`] for the module docs.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use super::{GapReport, TuResult};

/// **W-EMITSET scratch read-out** — one TSV line per emitted symbol the census
/// did not bind, appended to `C2RS_WALL_DUMP`. Off, and free, when unset.
///
/// `src · has-record|no-record · mangled name`
///
/// It exists because `mangling_class` is a *prefix* rule and prefix rules have
/// lied four times this week: `special-generated` is every `??_…`, which is
/// `??_G`/`??_E`/`??_D` (real synthesized functions) **and** `??_7` (vftable),
/// `??_R0`…`??_R4` (RTTI) and `??_C` (string literals), which are data. A
/// decomposition that reports 47.7 % `special-generated` and never prints a name
/// cannot tell those apart, and the whole reading of the wall rests on which it
/// is. Read-only: it changes no count.
pub(super) fn wall_dump(src: &str, name: &str, kind: &str) {
    // PROV[N] not load-bearing — a `OnceLock` holding the witness log file handle. Scratch state.
    static OUT: std::sync::OnceLock<Option<Mutex<std::fs::File>>> = std::sync::OnceLock::new();
    let out = OUT.get_or_init(|| {
        let p = std::env::var("C2RS_WALL_DUMP").ok()?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .ok()
            .map(Mutex::new)
    });
    let Some(out) = out else { return };
    if let Ok(mut g) = out.lock() {
        let _ = g.write_all(format!("{src}\t{kind}\t{name}\n").as_bytes());
    }
}

/// **The witness list** (board #159) — the mangled names behind the
/// emitted-symbol residue, emitted by the code that *classifies* them.
///
/// `C2RS_WITNESS=<path>` turns it on and writes two artifacts at the end of the
/// scan:
///
/// * `<path>` — the ranked summary: per bucket, the symbol total, the distinct
///   name count, the TU count, and the top [`WITNESS_CAP`] names by frequency
///   with an example TU for each.
/// * `<path>.rows.tsv` — every row, `src · bucket · in-gl · name`, for slicing
///   per TU.
///
/// **Why this exists rather than a private reader.** `ROADMAP.md` §10.14 is the
/// record of the alternative: a standalone COFF reader was written to answer
/// "what is an `emit-unbound-no-record|ordinary` symbol", it keyed on *no `.gl`
/// run* where the instrument keys on *no framed `.gl` body record*, and it
/// missed the harness's known answer on the first witness TU. A diagnostic that
/// needs a classification the harness already computes must be **emitted by the
/// harness**; a second implementation is a second rule that agrees until the
/// moment it matters.
///
/// **Why an environment variable and not a CLI flag.** The classification lives
/// here, and so does the precedent: [`wall_dump`] and [`row_dump`] are already
/// env-gated scratch instruments in this file. Off by default, and when off the
/// rows are never built — [`witness_path`] is consulted once per process.
///
/// The two `in-gl` columns are **third and fourth** predicates and are labelled
/// as such wherever they are read. Neither is "binds to a census row" and
/// neither is "has a framed body record": they ask whether the symbol's name is
/// in `.gl` **at all**, which separates "c2 invented this symbol" from "the name
/// is right there and only the framed body record is missing".
///
/// **There are two because one of them cannot see half the residue.**
/// [`c2_il::mangled_names`] requires the run's second byte to be alphabetic and
/// therefore **silently drops every `??`-prefixed name** — its own doc comment
/// says so — which is every `dtor` and every `special-generated` row in this
/// list. Read alone it reports `0 of 947` for `??_G…` and that zero is an
/// artifact of the predicate, not a fact about `.gl`. So the second column is
/// [`c2_il::gl_symbol_index`], the binding's own token→name index, which does
/// carry `??`-names. Reporting both, and naming which is which, is the whole
/// discipline `ROADMAP.md` §10.11/§10.14 was written about.
pub(super) fn witness_path() -> Option<&'static std::path::Path> {
    // PROV[N] not load-bearing — a `OnceLock` holding the witness log path.
    static P: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    P.get_or_init(|| std::env::var_os("C2RS_WITNESS").map(PathBuf::from))
        .as_deref()
}

/// How many names each bucket prints in the ranked summary. The remainder is
/// printed as a count of names *and* a count of symbols, never elided — a tail
/// that renders as nothing is the failure mode `docs/GAPS.md` §7 is about.
/// PROV[N] not load-bearing — how many witness names are printed; the counts are printed too and the tail is never elided (`docs/GAPS.md` §7).
const WITNESS_CAP: usize = 40;

/// One witness row: which residue bucket, the mangled name, and whether that
/// name appears as a mangled run in `.gl` at all.
#[derive(Clone, Debug)]
pub struct WitnessRow {
    /// The residue bucket, spelled exactly as the counter key it accompanies:
    /// `emit-unbound-no-record|<mangling class>` or `emit-unbound-has-record`.
    pub bucket: String,
    pub name: String,
    /// `c2_il::mangled_names` contains this name — a **different predicate**
    /// from the one that put the row in its bucket, and one that cannot see a
    /// `??`-prefixed name at all.
    pub in_gl_runs: bool,
    /// `c2_il::gl_symbol_index` binds this name to some operand token — the
    /// predicate that *can* see `??`-names. Also not the bucketing predicate.
    pub in_gl_index: bool,
}

/// Aggregated witness numbers for one bucket. Every field is a count, so a
/// bucket that collected nothing prints zeros beside a nonzero grand total
/// rather than vanishing.
pub struct WitnessBucket {
    pub bucket: String,
    pub symbols: usize,
    pub tus: usize,
    /// Rows whose name `c2_il::mangled_names` finds (blind to `??`-names).
    pub in_gl_runs: usize,
    /// Rows whose name `c2_il::gl_symbol_index` binds to a token.
    pub in_gl_index: usize,
    /// `(name, occurrences, TUs it appears in, an example TU)`, ranked by
    /// occurrences descending then name ascending.
    pub names: Vec<(String, usize, usize, String)>,
}

/// **The per-row read-out (W-ADJUST, boards #127/#128/#131).** One TSV line per
/// census row whose key is named in `C2RS_ROW_DUMP` (or `*` for all), appended to
/// `C2RS_ROW_DUMP_OUT`; `C2RS_ROW_DUMP_EMITTED` restricts it to rows that bind to
/// a symbol c2 actually emitted. Off — and free — when the variable is unset.
///
/// ```text
/// src · index · key · EMITTED|not-emitted · mangled name · frame · cflow · eh
///     · dispatch · production · completeness · hex_mark · the blocking-byte window
/// ```
///
/// **Every axis this scan prints is a histogram, and a histogram cannot answer a
/// question about a JOINT.** `docs/ROADMAP.md` §8.6's standing rule — never
/// multiply marginals for an intersection, measure the joint per TU — has no tool
/// behind it without this: the EH, frame and control-flow crosses are each a
/// separate `BTreeMap`, so "how many emitted rows of THIS key are straight *and*
/// EH-free *and* single-call" is unanswerable from the report. It is answerable
/// from one pass over this file, and that is where the 3,062-clean figure for
/// `expr-intrinsic-this-adjust` and the 9,111-clean figure for the whole
/// receiver-designator site came from.
///
/// Two further questions it exists for, both of which changed a ranking:
///
/// * **which production site actually refused** — `expr-intrinsic-this-adjust`
///   names the byte the *assignment* parser stopped on, while 99.99 % of the row
///   declines one reader earlier at the receiver designator;
/// * **is a row N distinct source functions or one replicated across TUs** —
///   `…recv-object-then-type-ptr-whole` is 1,380 emitted functions and **four**
///   mangled names, which is a fact about the differential coverage a rung can
///   claim, and no aggregate can see it.
///
/// **Read-only over the census: it changes no count and no verdict.** Asserted by
/// running the whole 878-TU scan with the dump armed and comparing all five
/// published numbers against the un-armed scan — 703,875 / 2,462,571 bodies,
/// 34,674 / 178,968 emitted, 6 match, 0 mismatch, disagreement 0, identical. An
/// instrument whose inertness is argued rather than run is this project's
/// dominant failure mode (`docs/GAPS.md` §6).
pub(super) fn row_dump(
    src: &str,
    census: &[(c2_il::FnCensus, Result<c2_il::IlFunction, &'static str>)],
    emitted: Option<&[String]>,
) {
    // PROV[N] not load-bearing — a second witness-log `OnceLock`. Scratch state.
    static OUT: std::sync::OnceLock<Option<Mutex<std::fs::File>>> = std::sync::OnceLock::new();
    let Ok(want) = std::env::var("C2RS_ROW_DUMP") else {
        return;
    };
    let wanted: Vec<&str> = want.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    let out = OUT.get_or_init(|| {
        let p = std::env::var("C2RS_ROW_DUMP_OUT").ok()?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .ok()
            .map(Mutex::new)
    });
    let Some(out) = out else { return };
    let emitted_set: std::collections::BTreeSet<&str> =
        emitted.unwrap_or(&[]).iter().map(String::as_str).collect();
    let emitted_only = std::env::var_os("C2RS_ROW_DUMP_EMITTED").is_some();
    let mut buf = String::new();
    for (f, _) in census {
        let key = f.verdict.key();
        if !wanted.iter().any(|w| key == *w || *w == "*") {
            continue;
        }
        let name = f.emit_name.as_deref().unwrap_or("-");
        let is_emitted = f.emit_name.as_deref().is_some_and(|n| emitted_set.contains(n));
        if emitted_only && !is_emitted {
            continue;
        }
        let hex: String = f.hex.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ");
        buf.push_str(&format!(
            "{src}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            f.index,
            key,
            if is_emitted { "EMITTED" } else { "not-emitted" },
            name,
            f.frame_class(),
            f.cflow,
            f.eh,
            f.dispatch,
            f.prod,
            f.verdict.completeness().name(),
            f.hex_mark,
            hex,
        ));
    }
    if buf.is_empty() {
        return;
    }
    if let Ok(mut g) = out.lock() {
        let _ = g.write_all(buf.as_bytes());
    }
}

/// Rank one scan's [`WitnessRow`]s per bucket. Pure over `results`, so the unit
/// test below grades it without a toolchain.
pub fn witness_buckets(results: &[TuResult]) -> Vec<WitnessBucket> {
    // bucket -> (symbols, TUs, in-gl, name -> (occurrences, TUs, example TU))
    type PerName = BTreeMap<String, (usize, std::collections::BTreeSet<String>, String)>;
    #[allow(clippy::type_complexity)]
    let mut agg: BTreeMap<
        String,
        (usize, std::collections::BTreeSet<String>, usize, usize, PerName),
    > = BTreeMap::new();
    for r in results {
        for w in &r.emit_witness {
            let e = agg.entry(w.bucket.clone()).or_insert_with(|| {
                (0, std::collections::BTreeSet::new(), 0, 0, BTreeMap::new())
            });
            e.0 += 1;
            e.1.insert(r.src.clone());
            e.2 += usize::from(w.in_gl_runs);
            e.3 += usize::from(w.in_gl_index);
            let n = e
                .4
                .entry(w.name.clone())
                .or_insert_with(|| (0, std::collections::BTreeSet::new(), r.src.clone()));
            n.0 += 1;
            n.1.insert(r.src.clone());
        }
    }
    let mut out: Vec<WitnessBucket> = agg
        .into_iter()
        .map(|(bucket, (symbols, tus, in_gl_runs, in_gl_index, names))| {
            let mut ranked: Vec<(String, usize, usize, String)> = names
                .into_iter()
                .map(|(name, (count, tus, example))| (name, count, tus.len(), example))
                .collect();
            // Frequency descending, then name ascending — a total order, so two
            // runs of the same scan print the same table.
            ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            WitnessBucket {
                bucket,
                symbols,
                tus: tus.len(),
                in_gl_runs,
                in_gl_index,
                names: ranked,
            }
        })
        .collect();
    out.sort_by(|a, b| b.symbols.cmp(&a.symbols).then_with(|| a.bucket.cmp(&b.bucket)));
    out
}

/// Write the ranked summary to `path` and every row to `<path>.rows.tsv`.
///
/// Every line is a **count**. There is no "no witnesses" status: a scan that
/// collected nothing prints `0 rows` against the scan's own residue totals, and
/// those totals disagreeing with the row count is the check that the list is
/// complete (`docs/GAPS.md` §7 — absence must not read as success).
pub(super) fn write_witness(report: &GapReport, path: &std::path::Path) -> std::io::Result<()> {
    let buckets = witness_buckets(&report.results);
    let rows: usize = report.results.iter().map(|r| r.emit_witness.len()).sum();

    let mut raw = std::io::BufWriter::new(std::fs::File::create(path.with_extension("rows.tsv"))?);
    writeln!(raw, "src\tbucket\tin_gl_runs\tin_gl_index\tname")?;
    for r in &report.results {
        for w in &r.emit_witness {
            writeln!(
                raw,
                "{}\t{}\t{}\t{}\t{}",
                r.src,
                w.bucket,
                u8::from(w.in_gl_runs),
                u8::from(w.in_gl_index),
                w.name
            )?;
        }
    }
    raw.flush()?;

    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    writeln!(
        f,
        "WITNESS LIST — the emitted-symbol residue, named (board #159)\n\
         scan: {} TUs, {rows} witness rows over {} buckets\n",
        report.results.len(),
        buckets.len()
    )?;
    // The cross-check that makes the list evidence rather than a plausible
    // sample: the rows must sum, per bucket, to the counter the same loop
    // incremented. Printed per bucket, as counts, both sides.
    for b in &buckets {
        let counted = report.emit_total(&b.bucket);
        writeln!(
            f,
            "== {} — {} symbols / {} TUs / {} distinct names\n\
             \x20  name present in `.gl`: {} by `mangled_names` (BLIND to `??`-names), \
             {} by `gl_symbol_index` — two predicates, neither is this bucket's\n\
             \x20  cross-check vs the scan's own counter: rows {} vs counter {} — agree: {}",
            b.bucket,
            b.symbols,
            b.tus,
            b.names.len(),
            b.in_gl_runs,
            b.in_gl_index,
            b.symbols,
            counted,
            b.symbols == counted
        )?;
        for (i, (name, count, tus, example)) in b.names.iter().take(WITNESS_CAP).enumerate() {
            writeln!(f, "  {:>4}. {count:>6} sym {tus:>4} TU  {name}  [{example}]", i + 1)?;
        }
        if b.names.len() > WITNESS_CAP {
            let shown: usize = b.names.iter().take(WITNESS_CAP).map(|(_, c, _, _)| *c).sum();
            writeln!(
                f,
                "  … and {} more distinct names covering {} symbols (top {WITNESS_CAP} cover {shown})",
                b.names.len() - WITNESS_CAP,
                b.symbols - shown
            )?;
        }
        writeln!(f)?;
    }
    f.flush()?;
    Ok(())
}
