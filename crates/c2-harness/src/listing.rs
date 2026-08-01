//! `c2-harness::listing` — the population scan behind the listing seam
//! (roadmap §9, board #132), answering board **#134** and **#136**.
//!
//! Two measurements ride on one capture per TU, because both need c2's own
//! `.cod` beside the obj the differential grades:
//!
//! # #134 — `/QXSTALLS`, the first scheduling-demand instrument
//!
//! Board #119 records the general allocator/scheduler as *"the largest unbounded
//! unknown, with NO instrument"*. `/QXSTALLS` annotates the listing with each
//! instruction's issue cycle, a per-function stall summary, and an estimated
//! IPC. The question this scan answers is: **what fraction of blocked EMITTED
//! functions carry any stall annotation** — a bound on how much of the blocked
//! set needs scheduling the port has never attempted.
//!
//! **The number is reported in emitted-function units** (denominator: the
//! `.text` COMDATs c2 actually wrote), never in IL bodies — §8.1 retired the
//! body count as a steering metric.
//!
//! **And it is reported beside its control.** The same fraction is computed over
//! the **in-class** emitted functions — the ones the port already reproduces
//! byte-exact with no scheduler at all. If the two fractions are close, the
//! annotation is present everywhere and discriminates nothing, and the headline
//! number means nothing; a scan that printed only the blocked fraction would go
//! green on an ambient signal. That is the failure this project keeps making,
//! and it is why both rows are printed whether or not they are flattering.
//!
//! Registered up front, and it bounds the claim: an annotation says the
//! **emitted schedule stalls**, not that c2 *reordered* anything. The fraction
//! is an upper bound on scheduling demand.
//!
//! # #136 — a second, name-carrying source for the emitted census
//!
//! The emitted census (§8.1, 19.09 %) binds a census row to an emitted function
//! through the `.gl` body-offset record and the obj's `.text` COMDAT leader
//! symbol. The `.cod` `PROC` set is an **independent** source for the same fact,
//! spelled by c2 in mangled names. Reconciling the two per TU is the only
//! available error term on that 19.09 %, which board #118 records as unwatched.
//!
//! **The oracle cannot grade a correspondence.** A byte compare cannot tell you
//! a binding is right, so the grading is on the binding's own invariants:
//!
//! 1. **Injectivity** — no mangled name appears twice in one TU's `PROC` set.
//! 2. **Totality with a named, printed residue** — every `PROC` has an obj
//!    COMDAT and vice versa; whatever does not is printed by name and class, not
//!    absorbed into a denominator.
//! 3. **Agreement on the byte-exact TUs**, where the answer is independently
//!    known: on a TU the port compiles byte-exact, the two sources must agree
//!    exactly.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::jstr;

use c2_core::{Backend, PortC2};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::cod::{is_build_stamp, CodListing};
use c2_reference::Toolchain;

/// Scan configuration (mirrors the `gap` scan's shape).
pub struct ListingScanConfig {
    pub sources: Vec<String>,
    pub flags: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub limit: Option<usize>,
    pub jobs: usize,
    pub work: PathBuf,
    /// Append `/QXSTALLS` (board #134). Without it the `stall_*` columns are
    /// all zero **by construction**, which is the negative control for the
    /// annotation reader.
    pub qxstalls: bool,
    pub jsonl: Option<PathBuf>,
}

/// Per-TU record.
#[derive(Clone, Debug, Default)]
pub struct ListingTu {
    pub src: String,
    /// Non-empty when the capture failed; every count below is then 0 and the
    /// TU is excluded from the ratios rather than counted as agreement.
    pub error: String,
    /// The port compiled this whole TU byte-exact — invariant 3's population.
    pub byte_exact: bool,

    // ---- #136: the two sources -------------------------------------------
    /// `PROC` blocks in the listing.
    pub cod_procs: usize,
    /// `.text` COMDAT leaders in the obj.
    pub obj_comdats: usize,
    /// Invariant 1: names appearing more than once in the `PROC` set.
    pub cod_duplicates: Vec<String>,
    /// Invariant 2, residue A: in the listing, not in the obj.
    pub cod_only: Vec<String>,
    /// Invariant 2, residue B: in the obj, not in the listing.
    pub obj_only: Vec<String>,

    // ---- the census join --------------------------------------------------
    /// Emitted functions the `.gl`+COMDAT binding claimed (§8.1's numerator
    /// denominator pair).
    pub emitted_bound: usize,
    pub emitted_in_class: usize,
    pub emitted_blocked: usize,
    /// Emitted symbols no census row claimed — §8.1's printed residue.
    pub emitted_unbound: usize,

    // ---- #134: the stall annotation, split by population -------------------
    pub blocked_stalled: usize,
    pub blocked_total: usize,
    pub in_class_stalled: usize,
    pub in_class_total: usize,
    /// Functions carrying the load-hit-store note specifically — the one
    /// annotation that names a *scheduling* remedy rather than a latency.
    pub blocked_lhs: usize,
    pub in_class_lhs: usize,
}

/// Aggregated report.
#[derive(Debug, Default)]
pub struct ListingReport {
    pub tus: Vec<ListingTu>,
    pub captured: usize,
    pub failed: usize,
}

impl ListingReport {
    /// (#134) `(blocked stalled, blocked total, in-class stalled, in-class total)`.
    pub fn stall_totals(&self) -> (usize, usize, usize, usize) {
        self.tus.iter().fold((0, 0, 0, 0), |a, t| {
            (
                a.0 + t.blocked_stalled,
                a.1 + t.blocked_total,
                a.2 + t.in_class_stalled,
                a.3 + t.in_class_total,
            )
        })
    }

    /// (#136) `(cod PROCs, obj COMDATs, duplicate names, cod-only, obj-only)`.
    pub fn reconcile_totals(&self) -> (usize, usize, usize, usize, usize) {
        self.tus.iter().fold((0, 0, 0, 0, 0), |a, t| {
            (
                a.0 + t.cod_procs,
                a.1 + t.obj_comdats,
                a.2 + t.cod_duplicates.len(),
                a.3 + t.cod_only.len(),
                a.4 + t.obj_only.len(),
            )
        })
    }

    /// (#136 invariant 3) The byte-exact TUs, and how many of them have a
    /// **perfectly** reconciled pair of sources.
    pub fn byte_exact_agreement(&self) -> (usize, usize) {
        let ok = self
            .tus
            .iter()
            .filter(|t| t.byte_exact)
            .filter(|t| t.cod_duplicates.is_empty() && t.cod_only.is_empty() && t.obj_only.is_empty())
            .count();
        (ok, self.tus.iter().filter(|t| t.byte_exact).count())
    }

    /// The emitted census as this scan measured it, for comparison against the
    /// `gap` scan's own reading of the same quantity.
    pub fn emitted_census(&self) -> (usize, usize, usize) {
        self.tus.iter().fold((0, 0, 0), |a, t| {
            (
                a.0 + t.emitted_in_class,
                a.1 + t.emitted_bound + t.emitted_unbound,
                a.2 + t.emitted_unbound,
            )
        })
    }

    /// The residue's shape, by mangling class — a residue reported only as a
    /// number is a rumour.
    pub fn residue_classes(&self) -> Vec<(String, usize)> {
        let mut m: BTreeMap<String, usize> = BTreeMap::new();
        for t in &self.tus {
            for n in t.cod_only.iter() {
                *m.entry(format!("cod-only|{}", mangling_class(n))).or_insert(0) += 1;
            }
            for n in t.obj_only.iter() {
                *m.entry(format!("obj-only|{}", mangling_class(n))).or_insert(0) += 1;
            }
        }
        let mut v: Vec<_> = m.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }
}

/// Coarse class of a decorated name, so a residue can be characterized rather
/// than merely counted.
pub fn mangling_class(name: &str) -> &'static str {
    if is_build_stamp(name) {
        return "build-stamp";
    }
    if name.starts_with("__unwind$") || name.starts_with("$T") || name.starts_with("$M") {
        return "eh-or-label";
    }
    if name.starts_with("??_G") || name.starts_with("??_E") {
        return "deleting-dtor";
    }
    if name.starts_with("??1") {
        return "dtor";
    }
    if name.starts_with("??0") {
        return "ctor";
    }
    if name.starts_with("??4") {
        return "assign";
    }
    if name.starts_with("??_7") || name.starts_with("??_8") {
        return "vftable";
    }
    if name.starts_with('?') {
        return "ordinary";
    }
    if name.starts_with('_') || name.starts_with('@') {
        return "c-or-runtime";
    }
    "other"
}

/// **The annotation reader.** True iff this function's listing block carries any
/// `/QXSTALLS` stall indicator.
///
/// Three shapes, all of which c2 emits, and matching only the summary would miss
/// the small bodies that stall without earning one:
///
/// * `;  ***  Stall summary for function  ***` and its indicator rows;
/// * a per-instruction cycle marker with a stall code appended —
///   `; [I    31B] D2(from I 29A)`, `P9`, `S2`, `DA4`, `L…`;
/// * the load-hit-store note, which is the one that names a *scheduling* fix.
///
/// The per-instruction form is matched by "the cycle marker is followed by
/// something", never by a bare code letter, because `P`/`D`/`S` occur constantly
/// in mangled operand text.
pub fn stall_flags(lines: &[String]) -> (bool, bool) {
    let mut stalled = false;
    let mut lhs = false;
    for l in lines {
        let t = l.trim_start();
        if !t.starts_with(';') {
            continue;
        }
        if t.contains("Possible load-hit-store penalty") {
            lhs = true;
            stalled = true;
            continue;
        }
        if t.contains("Stall summary for function")
            || t.contains("Dependency stall")
            || t.contains("Structural hazard")
            || t.contains("Stalled for non-pipelined instruction")
        {
            stalled = true;
            continue;
        }
        // `; [I    31B] D2(from I 29A)` — anything after the bracket is a code.
        if let Some(rest) = t.strip_prefix("; [I") {
            if let Some((_, tail)) = rest.split_once(']') {
                if !tail.trim().is_empty() {
                    stalled = true;
                }
            }
        }
    }
    (stalled, lhs)
}

/// Run the scan.
pub fn listing_scan(
    tc: &Toolchain,
    cfg: &ListingScanConfig,
    progress: &(dyn Fn(usize, usize, &ListingTu) + Sync),
) -> std::io::Result<ListingReport> {
    let sources: Vec<&str> = cfg
        .sources
        .iter()
        .map(|s| s.as_str())
        .take(cfg.limit.unwrap_or(usize::MAX))
        .collect();
    let total = sources.len();
    std::fs::create_dir_all(&cfg.work)?;

    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let results: Mutex<Vec<ListingTu>> = Mutex::new(Vec::with_capacity(total));
    let jobs = cfg.jobs.max(1).min(total.max(1));

    std::thread::scope(|scope| {
        for _ in 0..jobs {
            let sources = &sources;
            let next = &next;
            let done = &done;
            let results = &results;
            scope.spawn(move || loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= sources.len() {
                    break;
                }
                let src = sources[i];
                let work = cfg.work.join(format!("tu{i:05}"));
                let _ = std::fs::create_dir_all(&work);
                let r = scan_one(tc, cfg, src, &work);
                let _ = std::fs::remove_dir_all(&work);
                let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                progress(n, total, &r);
                results.lock().unwrap().push(r);
            });
        }
    });

    let mut tus = results.into_inner().unwrap();
    tus.sort_by(|a, b| a.src.cmp(&b.src));
    let failed = tus.iter().filter(|t| !t.error.is_empty()).count();
    let captured = tus.len() - failed;

    if let Some(path) = &cfg.jsonl {
        let mut f = std::fs::File::create(path)?;
        for t in &tus {
            writeln!(
                f,
                "{{\"src\":{},\"error\":{},\"byte_exact\":{},\"cod_procs\":{},\
                 \"obj_comdats\":{},\"cod_duplicates\":{},\"cod_only\":{},\"obj_only\":{},\
                 \"emitted_bound\":{},\"emitted_in_class\":{},\"emitted_blocked\":{},\
                 \"emitted_unbound\":{},\"blocked_stalled\":{},\"blocked_total\":{},\
                 \"in_class_stalled\":{},\"in_class_total\":{},\"blocked_lhs\":{},\
                 \"in_class_lhs\":{}}}",
                jstr(&t.src),
                jstr(&t.error),
                t.byte_exact,
                t.cod_procs,
                t.obj_comdats,
                json_names(&t.cod_duplicates),
                json_names(&t.cod_only),
                json_names(&t.obj_only),
                t.emitted_bound,
                t.emitted_in_class,
                t.emitted_blocked,
                t.emitted_unbound,
                t.blocked_stalled,
                t.blocked_total,
                t.in_class_stalled,
                t.in_class_total,
                t.blocked_lhs,
                t.in_class_lhs,
            )?;
        }
    }

    Ok(ListingReport {
        tus,
        captured,
        failed,
    })
}

fn json_names(v: &[String]) -> String {
    let inner: Vec<String> = v.iter().map(|s| jstr(s)).collect();
    format!("[{}]", inner.join(","))
}

fn scan_one(tc: &Toolchain, cfg: &ListingScanConfig, src: &str, work: &std::path::Path) -> ListingTu {
    let mut r = ListingTu {
        src: src.to_string(),
        ..Default::default()
    };
    let (captured, cod) = match tc.capture_listing_with(
        src,
        work,
        &cfg.flags,
        cfg.cwd.as_deref(),
        cfg.qxstalls,
    ) {
        Ok(v) => v,
        Err(e) => {
            r.error = clip(&e.to_string(), 160);
            return r;
        }
    };

    let listing = CodListing::parse(&cod);
    let Some(emitted) = captured.ref_obj.text_comdat_functions() else {
        r.error = "obj .text COMDATs did not decode".to_string();
        return r;
    };

    // ---- #136 invariants 1 and 2 -----------------------------------------
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for f in &listing.functions {
        if !seen.insert(f.name.as_str()) {
            r.cod_duplicates.push(f.name.clone());
        }
    }
    let obj_set: BTreeSet<&str> = emitted.iter().map(String::as_str).collect();
    r.cod_procs = listing.functions.len();
    r.obj_comdats = emitted.len();
    for n in &seen {
        if !obj_set.contains(n) {
            r.cod_only.push((*n).to_string());
        }
    }
    for n in &obj_set {
        if !seen.contains(n) {
            r.obj_only.push((*n).to_string());
        }
    }

    // ---- the census join, and #134 ----------------------------------------
    // Same binding rule as the `gap` scan's emitted census: a symbol claimed by
    // exactly one census row binds to that row; anything else is residue.
    if let Some(census) = captured.bundle.census_functions() {
        let mut claim: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
        for (i, (f, _)) in census.iter().enumerate() {
            if let Some(n) = f.emit_name.as_deref() {
                claim.entry(n).or_default().push(i);
            }
        }
        // Listing block per emitted name, for the annotation read.
        let mut blocks: BTreeMap<&str, &Vec<String>> = BTreeMap::new();
        for f in &listing.functions {
            blocks.insert(f.name.as_str(), &f.lines);
        }
        for name in &emitted {
            match claim.get(name.as_str()).map(Vec::as_slice) {
                Some([row]) => {
                    r.emitted_bound += 1;
                    let in_class = census[*row].0.verdict.in_class();
                    if in_class {
                        r.emitted_in_class += 1;
                    } else {
                        r.emitted_blocked += 1;
                    }
                    // #134 — only over functions that (a) c2 emitted and (b) the
                    // census bound, so blocked and in-class are the same kind of
                    // population and the two fractions are comparable.
                    if let Some(lines) = blocks.get(name.as_str()) {
                        let (stalled, lhs) = stall_flags(lines);
                        if in_class {
                            r.in_class_total += 1;
                            r.in_class_stalled += usize::from(stalled);
                            r.in_class_lhs += usize::from(lhs);
                        } else {
                            r.blocked_total += 1;
                            r.blocked_stalled += usize::from(stalled);
                            r.blocked_lhs += usize::from(lhs);
                        }
                    }
                }
                _ => r.emitted_unbound += 1,
            }
        }
    }

    // ---- invariant 3's population: the byte-exact TUs ----------------------
    if captured.bundle.functions().is_some() {
        let obj_name = c2_reference::to_wibo_path(&captured.ref_obj_path);
        let gy = PortC2::flags_imply_function_level_linking(&cfg.flags);
        let port = PortC2::new(obj_name.clone()).with_function_level_linking(gy);
        if let Ok(obj) = port.compile_to(&captured.bundle, &obj_name) {
            r.byte_exact = matches!(ObjImage::diff(&captured.ref_obj, &obj), ObjDiff::Identical);
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reader must fire on all three annotation shapes, and — the half that
    /// matters — must **not** fire on an un-annotated listing block. A reader
    /// that matched something ambient would report ~100 % on both populations
    /// and the #134 number would be meaningless while looking decisive.
    #[test]
    fn stall_flags_reads_the_three_shapes_and_nothing_else() {
        let plain: Vec<String> = [
            "",
            "; 25   :     void_func(x);",
            "  00014\t48000001\t bl           ?void_func@@YAXH@Z",
            "; Function compile flags: /Ogsu",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(
            stall_flags(&plain),
            (false, false),
            "an un-annotated block was read as stalled — the #134 fractions \
             would then be ~100 % on every population"
        );

        let per_insn: Vec<String> = ["  00024\t48000001\t bl x".into(), "; [I    31B] D2(from I 29A)".into()].into();
        assert_eq!(
            stall_flags(&per_insn),
            (true, false),
            "the per-instruction stall code `D2(from I 29A)` was not read"
        );

        let bare_cycle: Vec<String> = ["; [I    11A]".into()].into();
        assert_eq!(
            stall_flags(&bare_cycle),
            (false, false),
            "a bare issue-cycle marker with NO stall code was read as a stall — \
             every annotated function carries these, so this alone would make \
             the fraction 100 % by construction"
        );

        let summary: Vec<String> = [
            ";  ***  Stall summary for function  ***".into(),
            ";         D      2        7  Dependency stall".into(),
        ]
        .into();
        assert_eq!(stall_flags(&summary), (true, false), "the summary was not read");

        let lhs: Vec<String> = ["; Possible load-hit-store penalty".into()].into();
        assert_eq!(
            stall_flags(&lhs),
            (true, true),
            "the load-hit-store note was not read as both a stall and an LHS"
        );
    }

    /// A `;`-less line can never be an annotation, and mangled operand text is
    /// full of `P`, `D` and `S`.
    #[test]
    fn mangled_operands_are_never_read_as_stall_codes() {
        let rows: Vec<String> = [
            "  00000\t906b0000\t stw          r3,?gDPS@@3HA(r11)".into(),
            "?PDS@@YAXXZ PROC NEAR".into(),
        ]
        .into();
        assert_eq!(stall_flags(&rows), (false, false));
    }

    #[test]
    fn mangling_class_separates_generated_from_ordinary() {
        assert_eq!(mangling_class("?f@@YAHH@Z"), "ordinary");
        assert_eq!(mangling_class("??_GFoo@@UAEPAXI@Z"), "deleting-dtor");
        assert_eq!(mangling_class("??1Foo@@QAE@XZ"), "dtor");
        assert_eq!(mangling_class("__C2_11886"), "build-stamp");
        assert_eq!(mangling_class("__unwind$2568"), "eh-or-label");
    }
}

/// Truncate on a char boundary (local copy: `gap::clip` is private and this
/// module deliberately does not depend on the gap scan).
fn clip(s: &str, n: usize) -> String {
    if s.len() <= n {
        return s.to_string();
    }
    let mut end = n;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}
