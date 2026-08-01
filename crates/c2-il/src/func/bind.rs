//! **The correspondence seam** — how a `.ex` function segment gets its name,
//! its automatic locals, and its callees' names.
//!
//! This is `docs/ROADMAP.md` #14's defect class and the one place in the crate
//! where **the oracle cannot grade the answer**. A byte-exact obj compare
//! grades what the port *emitted*; it cannot grade a *correspondence*, because
//! binding segment 3 to the wrong name produces an obj that is wrong in a way
//! the differential only notices if the two names happen to differ in the
//! bytes. `docs/GAPS.md` §6 states the general form. Everything here is
//! therefore fail-closed: a binding that cannot be established is `None`, never
//! a plausible guess.
//!
//! # Two bindings, and they are not the same binding
//!
//! [`IlBundle::functions`] (the gate) and [`IlBundle::census_functions`] (the
//! diagnostic that sizes the census/gate disagreement) each answer "what is
//! this segment called?", and **they answer it differently today**:
//!
//! | | gate | census |
//! |---|---|---|
//! | segments | `split_functions_at` — every `4F 1F` | `split_function_bodies_at` — anchored on the `LO` body marker |
//! | names | [`Bindings::per_record`] — each `.gl` record's framed body-start offset must **be** a split point, in order and 1:1 | [`Bindings::positional`] — `mangled_names` zipped onto segments, used only when the counts match |
//! | on failure | refuses the whole TU | reports the body's real blocker, with no name |
//!
//! Both differences are deliberate and documented at their sites: `4F 1F` is
//! two bytes and also occurs inside token payloads, so a raw scan over a real
//! TU over-counts by ~2 % (measured on `system/world/Dir.cpp`), which is why
//! the *denominator* anchors on `LO`; and `mangled_names` accepts only `?…@@…`
//! forms while `.gl` also lists externals, so positional pairing is
//! meaningless on a real TU and is gated on the count agreeing.
//!
//! **They are still two answers to one question**, and unifying them —
//! `census_functions` binding per record through [`Bindings::per_record`] like
//! the gate — is roadmap #14's scheduled follow-up. It is *not* this module's
//! introduction, because it **moves the census numerator**, and the numerator
//! is the measurement everything else is differenced against. What this module
//! does is put both bindings, and the three different ways the census reads its
//! name list, in one file where the disagreement is visible instead of spread
//! across two hundred lines of two call sites.
//! `the_two_bindings_are_the_open_seam_and_are_pinned_apart` below pins the
//! disagreement so the day it closes is visible.
//!
//! # What lives here
//!
//! The *policy*: which binding, the fail-closed checks, and the name-derived
//! gate ([`mangled_is_varargs`]) both callers apply. The `.gl` and `.sy`
//! *readers* stay in [`super::gl`] and [`super::sy`] — those are format
//! decoders with their own witness tests (`gl_names_bind_to_their_own_record_
//! not_their_position`, `a_token_two_symbols_claim_is_dropped_rather_than_
//! guessed`), and splitting a reader across two files is the defect this whole
//! restructure exists to avoid.

use super::gl::{gl_defined_names, gl_symbol_runs, mangled_names, source_path, GlIndex};
use super::sy::{SyLocals, SyView};

/// How far a `.gl` function record's name may end before its body-offset field.
///
/// The same 32 bytes `gl::MAX_NAME_TO_OFFSET` uses, and for the same reason: it
/// is the bound that makes "nearest preceding run" mean *this record's* name
/// rather than whatever happened to be last in the file. It is also the single
/// most load-bearing constant in [`EmitBinding`] — see that type's docs for what
/// dropping it costs, measured.
const EMIT_MAX_NAME_TO_OFFSET: usize = 32;

/// **The emitted-function binding** (`docs/GAPS.md` §8): census row ↔ the
/// mangled name of the `.gl` record whose body-start offset lands in that row's
/// `.ex` segment.
///
/// # Why this is not [`Bindings::per_record`]
///
/// It reads the same `.gl` record and takes the same name, but it is a
/// *diagnostic* binding and the two differ in exactly two places, both forced:
///
/// | | [`Bindings::per_record`] (the gate) | [`EmitBinding`] (the instrument) |
/// |---|---|---|
/// | framing | `80 ?? 10 00 00 00 00` before the offset field | `80 <LE32 < 0x10000> 00 00` before it |
/// | totality | all-or-nothing: any divergence binds **nothing** | per record, with every failure in a named residue |
///
/// **The framing byte was over-fitted, and this is the measurement.** The gate's
/// predicate pins `gl[o-5] == 0x10`, which is not a tag — it is the third byte
/// of the *preceding* `80 <LE32>` field, so pinning it demands that field's value
/// lie in `0x1000..=0x10FF`. Every fixture happens to satisfy that. `src/App.cpp`
/// does not: its records carry `0x19A1`, `0x19AB`, `0xA4F6`, … and the gate's
/// framing therefore finds **34 records in a translation unit with 9,033 function
/// bodies and 158 emitted functions**. Requiring only the two high bytes to be
/// zero finds **6,069**, of which 5,908 land on a segment start and **0** bind two
/// names to one offset. That is why this type exists rather than a call to the
/// gate's reader; the gate's own predicate is deliberately left alone, because
/// loosening it would move the accepted class and this instrument must not.
///
/// # The invariants it is graded on, since the oracle cannot grade it
///
/// A byte compare grades emitted bytes. It cannot say whether row *R* is symbol
/// *S* (`docs/GAPS.md` §6, and the `.sy` positional relaxation that was census
/// +2,981, mismatch 0, and wrong on 62 % of its bindings). So:
///
/// * **Injective, by construction and then by count.** Each row keeps at most one
///   name; a name two rows claim is dropped from **both**
///   ([`EmitBinding::dropped_name_conflict`]), and a row two records claim is
///   dropped ([`EmitBinding::dropped_row_conflict`]). Both are reported, so the
///   claim is checkable and not merely asserted.
/// * **Total, with a residue that prints.** Every record either binds or lands in
///   [`EmitBinding::records_nameless`] / [`EmitBinding::records_outside`] /
///   a conflict bucket, and `records` equals their sum plus the bound count.
///   The residue that vanishes silently is the failure mode this project keeps
///   hitting; `c2rs gap` prints these on every scan.
/// * **Checkable where the answer is known exactly.** On a translation unit the
///   port compiles byte-exact, c2's emitted symbol set is the port's, which came
///   from [`Bindings::per_record`] — so on those TUs this binding must agree with
///   the gate's, name for name. `c2rs gap` asserts that on every `match` TU.
///
/// **Fail-closed in one direction only, and it is the safe one:** an emitted
/// symbol this binding cannot claim is counted as residue, never as in class. So
/// the read-out it feeds is a floor.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EmitBinding {
    /// Segment index → bound mangled name. Injective over the `Some` entries.
    row: Vec<Option<String>>,
    /// Framed body-offset records found in `.gl`.
    pub records: usize,
    /// Records whose offset falls before the first segment (nothing contains it).
    pub records_outside: usize,
    /// Records with no symbol run ending within [`EMIT_MAX_NAME_TO_OFFSET`] —
    /// a record shape whose name this reader cannot locate. Never guessed.
    pub records_nameless: usize,
    /// **Records** lost to a row two or more of them landed in; that row binds
    /// nothing. Counted per record, not per row, because the totality identity
    /// ([`EmitBinding::accounting`]) is stated over records.
    pub dropped_row_conflict: usize,
    /// Names two or more rows claimed; every one of those rows binds nothing.
    pub dropped_name_conflict: usize,
}

impl EmitBinding {
    /// Build the binding from `.gl` and the census's own segment start offsets
    /// (`seg_starts`, ascending — [`super::bundle::split_function_bodies_at`]).
    ///
    /// A record binds to the segment **containing** its body-start offset, not to
    /// the segment whose start *equals* it. The two differ: on `src/App.cpp` 6,068
    /// of 6,069 record offsets are `4F 1F` function-start markers but only 5,908
    /// are *census* segment starts, because the census anchors segments on the `LO`
    /// body marker and a `4F 1F` that no `LO` follows is inside its predecessor's
    /// segment, not a segment of its own. Containment binds those 160 to the row
    /// that actually holds their body; equality would have dropped them into a
    /// residue bucket that meant nothing.
    pub fn new(gl: &[u8], seg_starts: &[usize]) -> EmitBinding {
        let mut out = EmitBinding {
            row: vec![None; seg_starts.len()],
            ..EmitBinding::default()
        };
        let runs = gl_symbol_runs(gl);
        let ends: Vec<usize> = runs.iter().map(|&(_, end, _)| end).collect();
        // Record offset → candidate name, one pass over `.gl`.
        let mut claims: Vec<(usize, String)> = Vec::new();
        let mut p = 0usize;
        while p + 5 <= gl.len() {
            if !emit_offset_framed(gl, p) {
                p += 1;
                continue;
            }
            out.records += 1;
            let off = u32::from_le_bytes([gl[p + 1], gl[p + 2], gl[p + 3], gl[p + 4]]) as usize;
            // The record's own name: the last run to END at or before the field,
            // and near enough to be part of the same record. A record whose name
            // is further away is one whose shape this reader does not know — it
            // must NOT borrow its predecessor's name, which is precisely the
            // defect the bound exists to stop. Measured across 371 workload TUs:
            // without the bound, 3,799 emitted symbols end up claimed by two rows
            // each; with it, **zero**.
            let name = match ends.partition_point(|&e| e <= p) {
                k @ 1.. if p - ends[k - 1] <= EMIT_MAX_NAME_TO_OFFSET => runs[k - 1].2.clone(),
                _ => {
                    out.records_nameless += 1;
                    p += 5;
                    continue;
                }
            };
            // The row containing this offset.
            match seg_starts.partition_point(|&s| s <= off) {
                0 => out.records_outside += 1,
                k => claims.push((k - 1, name)),
            }
            p += 5;
        }
        // Two records in one row: neither binds.
        let mut per_row: Vec<Option<Option<String>>> = vec![None; seg_starts.len()];
        for (row, name) in claims {
            match &mut per_row[row] {
                slot @ None => *slot = Some(Some(name)),
                slot @ Some(Some(_)) => {
                    // TWO records are lost here, not one: the incumbent as well
                    // as this one. Counting rows instead of records is what broke
                    // the totality identity on 607 workload TUs — the identity is
                    // stated over *records*, and it is the check that says the
                    // residue is complete, so it has to be counted in the unit it
                    // is stated in.
                    *slot = Some(None);
                    out.dropped_row_conflict += 2;
                }
                Some(None) => out.dropped_row_conflict += 1,
            }
        }
        // One name in two rows: none of them binds. `.gl` gives a defined
        // function one record, so a repeat means this reader read something else
        // as one — dropping is the third value, exactly as `gl_symbol_conflicts`
        // does for a token two names claim.
        let mut seen: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
        for slot in per_row.iter().flatten().flatten() {
            *seen.entry(slot.as_str()).or_insert(0) += 1;
        }
        let dup: std::collections::BTreeSet<String> = seen
            .into_iter()
            .filter(|&(_, n)| n > 1)
            .map(|(k, _)| k.to_string())
            .collect();
        for (i, slot) in per_row.into_iter().enumerate() {
            match slot.flatten() {
                Some(n) if dup.contains(&n) => out.dropped_name_conflict += 1,
                Some(n) => out.row[i] = Some(n),
                None => {}
            }
        }
        out
    }

    /// The name bound to census row `i`, if any.
    pub fn name(&self, i: usize) -> Option<&str> {
        self.row.get(i).and_then(|n| n.as_deref())
    }

    /// How many rows bound a name.
    pub fn bound(&self) -> usize {
        self.row.iter().filter(|n| n.is_some()).count()
    }

    /// **The totality identity**, over RECORDS: every framed record found became
    /// exactly one of a binding or one named residue, so
    /// `records == bound + outside + nameless + row-conflicts + name-conflicts`.
    ///
    /// Stated over records rather than rows because a row conflict consumes two
    /// or more records while costing one row — counting it per row makes the
    /// identity fail, which is exactly how it was caught (607 workload TUs).
    /// Returns both sides so a caller asserts it rather than trusting it; `c2rs
    /// gap` reports the breaks as `emit-accounting-broken`, known answer 0.
    pub fn accounting(&self) -> (usize, usize) {
        (
            self.records,
            self.bound()
                + self.records_outside
                + self.records_nameless
                + self.dropped_row_conflict
                + self.dropped_name_conflict,
        )
    }
}

/// True iff a `.gl` body-start offset field's `80 <LE32>` at `o` sits in the
/// record framing `80 <LE32 v> 00 00` with `v < 0x10000`.
///
/// This is [`crate::codec::gl_offset_framed`] with its `gl[o-5] == 0x10` clause
/// dropped — see [`EmitBinding`] for the measurement that says that clause pins
/// a *value* byte, not a tag. The gate's version is deliberately unchanged.
fn emit_offset_framed(gl: &[u8], o: usize) -> bool {
    o >= 7
        && gl[o] == 0x80
        && gl[o - 7] == 0x80
        && gl[o - 4] == 0x00
        && gl[o - 3] == 0x00
        && gl[o - 2] == 0x00
        && gl[o - 1] == 0x00
}

/// Everything a `.ex` segment list needs bound to it, built once per bundle.
///
/// There is no "which binding" discriminant field: the two constructors —
/// [`Bindings::per_record`] and [`Bindings::positional`] — *are* the
/// distinction, and a field nothing reads would be dead weight pretending to be
/// a check.
pub(crate) struct Bindings<'a> {
    names: Vec<String>,
    paired: bool,
    /// `.gl` symbols no defined-function record claimed. Only meaningful for
    /// [`Bindings::per_record`]; empty otherwise.
    pub(crate) unclaimed: Vec<String>,
    /// The source path from `.gl` — provenance the emitter does not embed.
    pub(crate) src: Option<String>,
    /// Token → symbol name. Built lazily on first use, so a TU of straight-line
    /// leaves never constructs the index at all.
    symbols: GlIndex<'a>,
    /// Per-segment automatic locals, keyed on the segment's exit label. Yields
    /// nothing at all unless `.sy` parses whole, so a TU without one is exactly
    /// as restricted as it was before locals were modeled.
    locals: SyLocals,
}

impl<'a> Bindings<'a> {
    /// The **gate's** binding: per `.gl` record, gated fail-closed on the
    /// records' framed body-start offsets being exactly the `.ex` split points,
    /// in order and 1:1.
    ///
    /// `None` when that check fails. A disagreement means either `.gl` has a
    /// record shape we cannot frame or the splitter miscounted bodies, and in
    /// both cases every name after the divergence would be wrong — so bind none
    /// of them.
    pub(crate) fn per_record(
        gl: &'a [u8],
        sy: Option<&[u8]>,
        segs: &[&[u8]],
        starts: &[usize],
    ) -> Option<Bindings<'a>> {
        let (bound, unclaimed) = gl_defined_names(gl);
        if bound.len() != segs.len()
            || bound
                .iter()
                .zip(starts)
                .any(|(&(off, _), &s)| off as usize != s)
        {
            return None;
        }
        let names: Vec<String> = bound.into_iter().map(|(_, n)| n).collect();
        Some(Bindings {
            paired: true,
            names,
            unclaimed,
            src: source_path(gl),
            symbols: GlIndex::new(gl),
            locals: SyLocals::new(sy, segs),
        })
    }

    /// The **census's** binding: `mangled_names` paired onto the segment list by
    /// position, which is only meaningful when it yields exactly one name per
    /// body. On a real TU it finds far fewer, so pairing there would attach
    /// wrong names to functions — [`Self::reported_name`] then reports none
    /// rather than a plausible-looking lie.
    ///
    /// Infallible on purpose: the census's job is to report *why* a body is out
    /// of class, and refusing the TU for want of names would replace every real
    /// blocking feature with this one and destroy the histogram that ranks the
    /// roadmap.
    pub(crate) fn positional(gl: &'a [u8], sy: Option<&[u8]>, segs: &[&[u8]]) -> Bindings<'a> {
        let names = mangled_names(gl);
        let paired = names.len() == segs.len();
        Bindings {
            paired,
            names,
            unclaimed: Vec::new(),
            src: source_path(gl),
            symbols: GlIndex::new(gl),
            locals: SyLocals::new(sy, segs),
        }
    }

    /// All bound names, in segment order. The gate uses this whole list for its
    /// TU-level accounting (unclaimed symbols, locally-defined callees).
    pub(crate) fn names(&self) -> &[String] {
        &self.names
    }

    /// The name handed to `shape_to_function` for segment `i`.
    ///
    /// Empty when there is none. That is deliberate and is why it is spelled
    /// out here rather than left as an `unwrap_or_default()` at a call site:
    /// under [`Bindings::positional`] with an unpaired list, `names[i]` may
    /// still exist and be the **wrong** name, and the census passes it anyway —
    /// the value only reaches `IlFunction::mangled_name`, which the census
    /// never emits from, and refusing here would cost the histogram a row. A
    /// caller that emits must use [`Bindings::per_record`].
    pub(crate) fn name_for_shape(&self, i: usize) -> String {
        self.names.get(i).cloned().unwrap_or_default()
    }

    /// The name **reported** for segment `i` — `None` unless the pairing is
    /// meaningful, so the census never prints a name it does not believe.
    pub(crate) fn reported_name(&self, i: usize) -> Option<String> {
        if self.paired {
            self.names.get(i).cloned()
        } else {
            None
        }
    }

    /// The one name-derived gate both callers apply, at segment `i`.
    ///
    /// A variadic function's body IL is byte-identical to its non-variadic
    /// twin's, so this cannot live in the body parser. Asking it here, through
    /// one predicate on one name list, is what keeps the census and the gate
    /// from disagreeing about what is in class.
    pub(crate) fn is_varargs(&self, i: usize) -> bool {
        self.paired && self.names.get(i).is_some_and(|n| mangled_is_varargs(n))
    }

    /// Token → callee symbol name. `None` refuses: a CALL token carries a
    /// function-*type* id, not the callee, so guessing a name is a relocation
    /// against the wrong symbol — a mis-emit, not a gap.
    pub(crate) fn resolve(&self, tok: u32) -> Option<String> {
        self.symbols.map().get(&tok).cloned()
    }

    /// The automatic locals of segment `i`.
    pub(crate) fn locals(&self, i: usize) -> SyView<'_> {
        self.locals.view(i)
    }
}

/// Whether a mangled name declares a **variadic** function — `int f(int, ...)`.
///
/// This is the one fact about a function that has to be read off its *name*,
/// because the IL body does not carry it. Measured: the `.ex` and `.sy` streams of
/// `int va(int a, ...) { return a; }` and `int va(int a) { return a; }` are
/// **byte-identical** (2745 B and 30 B respectively, compared whole), and the only
/// files that differ are `.gl` — by the one byte of this name — and `.db`. So no
/// body-level or local-symbol gate can see it, and there is nothing to decode.
///
/// It is not cosmetic. c2 gives a variadic function a frame that homes r4–r10 and
/// a `.pdata` entry, so the obj has six sections where the port emits four
/// instructions and five:
///
/// ```text
/// int va(int a, ...) { return a; }
///   c2:   std r4,0x18(r1) … std r10,0x48(r1) ; blr     + .pdata
///   port: blr
///   Port=Mismatch @ offset 2   (NumberOfSections, 06 vs 05)
/// ```
///
/// It fires in every optimization mode including the workload's own, and it was
/// live on `straight-line`, `indirect-load-leaf`, `__stdcall`, and member-function
/// bodies — the oldest accepted shapes in the port.
///
/// **The rule.** MSVC terminates a mangled argument list with `@` (a list ended),
/// `X` (no arguments), or `Z` (an ellipsis), and then closes the name with a final
/// `Z`. So a variadic function's name ends `ZZ`. Measured, with the neighbour that
/// breaks the naive reading:
///
/// ```text
/// ?a1@@YAHH@Z          int a1(int)                     not variadic
/// ?a2@@YAHXZ           int a2()                        not variadic — ends XZ, not @Z
/// ?a3@@YAHPAUZ@@@Z     int a3(Z*)   a type NAMED Z     not variadic
/// ?a4@@YAHW4E@@@Z      int a4(E)    an enum            not variadic
/// ?a5@@YAHHZZ          int a5(int, ...)                VARIADIC
/// ?a6@@YAHZZ           int a6(...)                     VARIADIC
/// ?f@C@@QBAHHZZ        int C::f(int, ...) const        VARIADIC
/// ```
///
/// `a3` is the discriminating case: a parameter whose *type* is `struct Z` puts a
/// `Z` in the name and must not be caught. `a2` is why the test cannot be "does
/// not end `@Z`".
///
/// An `extern "C"` variadic function has an undecorated name and is invisible here.
/// That is covered, for a different reason that must not be quietly relied upon:
/// `gl_defined_names` accepts only `?…@@…` forms, so a TU containing one binds no
/// names at all and `functions` refuses it whole. If that ever loosens, this gate
/// stops covering C variadics — measured today (`extern "C" int cva(int, ...)` is
/// `Port=NotImplemented`), and stated here so the coupling is visible.
pub(crate) fn mangled_is_varargs(name: &str) -> bool {
    name.ends_with("ZZ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One `.gl` function record, in the shape `codec::gl_offset_framed`
    /// recognizes: a NUL-delimited name run, the framing `80 XX 10 00 00 00 00`,
    /// then the `80 <LE32>` body-start offset. Same builder `gl.rs`'s own
    /// binding tests use — the record FORMAT is that module's fact, and this one
    /// only grades the policy built on top of it.
    fn gl_record(name: &str, body_off: u32) -> Vec<u8> {
        let mut v = vec![0u8];
        v.extend_from_slice(name.as_bytes());
        v.push(0);
        v.extend_from_slice(&[0x80, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x80]);
        v.extend_from_slice(&body_off.to_le_bytes());
        v
    }

    fn two_records() -> Vec<u8> {
        let mut gl = Vec::new();
        gl.extend_from_slice(&gl_record("?a@@YAHXZ", 0));
        gl.extend_from_slice(&gl_record("?b@@YAHXZ", 40));
        gl
    }

    /// The gate's binding, established — and it is the per-RECORD one, so the
    /// name of segment `k` is the name of the record carrying segment `k`'s
    /// offset, not the `k`-th name in the file.
    #[test]
    fn per_record_binds_each_segment_to_the_record_carrying_its_offset() {
        let gl = two_records();
        let segs: Vec<&[u8]> = vec![&[0u8; 4], &[0u8; 4]];
        let b = Bindings::per_record(&gl, None, &segs, &[0, 40])
            .expect("offsets 0 and 40 ARE the split points");
        assert_eq!(b.names(), ["?a@@YAHXZ".to_string(), "?b@@YAHXZ".to_string()]);
        // A per-record binding is 1:1 by construction, so it is always paired.
        assert!(b.paired);
        assert_eq!(b.reported_name(1).as_deref(), Some("?b@@YAHXZ"));
        assert_eq!(b.name_for_shape(1), "?b@@YAHXZ");
    }

    /// The fail-closed check is the whole point. When the records' body offsets
    /// are not exactly the `.ex` split points, bind **none** of them: every name
    /// after a divergence would be wrong, and a wrong name is a relocation
    /// against the wrong symbol — a mis-emit, not a gap.
    #[test]
    fn per_record_binds_nothing_when_the_offsets_are_not_the_split_points() {
        let gl = two_records();
        let segs: Vec<&[u8]> = vec![&[0u8; 4], &[0u8; 4]];
        assert!(Bindings::per_record(&gl, None, &segs, &[0, 41]).is_none());
        assert!(Bindings::per_record(&gl, None, &segs, &[4, 40]).is_none());
        // Order matters too: the same two offsets, swapped, is a divergence.
        assert!(Bindings::per_record(&gl, None, &segs, &[40, 0]).is_none());
        // …and a count mismatch is the same refusal.
        assert!(Bindings::per_record(&gl, None, &segs[..1], &[0]).is_none());
    }

    /// Positional pairing is meaningful only when the counts agree, and when it
    /// is not, the census reports **no** name rather than a plausible lie — but
    /// still hands the shape conversion whatever is at that index, and holds the
    /// varargs gate silent. Those are three different reads of one list, made at
    /// three different places in one closure before this module existed.
    #[test]
    fn positional_reports_no_name_when_unpaired_but_still_feeds_the_conversion() {
        let gl = two_records();
        let three: Vec<&[u8]> = vec![&[0u8; 4], &[0u8; 4], &[0u8; 4]];
        let b = Bindings::positional(&gl, None, &three);
        assert!(!b.paired, "2 names against 3 segments is not a pairing");
        assert_eq!(b.reported_name(0), None);
        assert_eq!(b.reported_name(2), None);
        // …and the conversion still gets index 0's name, wrong or not, because
        // the census never emits from it and refusing would cost a histogram row.
        assert_eq!(b.name_for_shape(0), "?a@@YAHXZ");
        assert_eq!(b.name_for_shape(2), String::new());
        // The name-derived gate is silent when unpaired, by the same rule.
        assert!(!b.is_varargs(0));

        // Paired, the same list reports and gates normally.
        let two: Vec<&[u8]> = vec![&[0u8; 4], &[0u8; 4]];
        let b = Bindings::positional(&gl, None, &two);
        assert!(b.paired);
        assert_eq!(b.reported_name(0).as_deref(), Some("?a@@YAHXZ"));
    }

    /// **The open seam.** The gate binds per record and the census binds by
    /// position, and on the very layout `gl.rs`'s own test was written for they
    /// give different answers: `mangled_names` cannot see a `??`-prefixed name,
    /// so it pairs `?w_add` with the *thunk's* body and the data symbol with
    /// `?w_add`'s.
    ///
    /// Closing this — `census_functions` binding per record like the gate — is
    /// roadmap #14's scheduled follow-up, and it **moves the census numerator**,
    /// which is why it is not done silently. This test exists so the day it
    /// lands is visible: when the two agree, this assertion fails, and the
    /// numerator move gets recorded instead of absorbed.
    #[test]
    fn the_two_bindings_are_the_open_seam_and_are_pinned_apart() {
        // The `il_gl_record_order.cpp` layout.
        let mut gl = Vec::new();
        gl.extend_from_slice(&gl_record("??__Egs@@YAXXZ", 0));
        gl.extend_from_slice(&gl_record("?w_add@@YAHH@Z", 40));
        gl.push(0);
        gl.extend_from_slice(b"?gs@@3US@@A");
        gl.push(0);
        let segs: Vec<&[u8]> = vec![&[0u8; 4], &[0u8; 4]];

        let per_record = Bindings::per_record(&gl, None, &segs, &[0, 40])
            .expect("the records' offsets are the split points");
        let positional = Bindings::positional(&gl, None, &segs);

        assert_eq!(
            per_record.names(),
            ["??__Egs@@YAXXZ".to_string(), "?w_add@@YAHH@Z".to_string()],
            "the gate binds each name to the record carrying its own body offset"
        );
        assert_eq!(
            positional.names(),
            ["?w_add@@YAHH@Z".to_string(), "?gs@@3US@@A".to_string()],
            "the census's narrow scan cannot see `??` names, so it slides by one"
        );
        assert_ne!(
            per_record.names(),
            positional.names(),
            "if these ever agree, roadmap #14's follow-up landed — record the \
             census numerator move rather than deleting this test quietly"
        );
        // And the slide is silent: positional is *paired* here, so the census
        // would report `?w_add` as the name of the thunk's body with no
        // indication anything is wrong. That is the wrong-but-green shape the
        // oracle cannot grade.
        assert!(positional.paired);
        assert_eq!(positional.reported_name(0).as_deref(), Some("?w_add@@YAHH@Z"));
    }

    // ---- EmitBinding (`docs/GAPS.md` §8) ---------------------------------
    //
    // The oracle cannot grade a correspondence, so these grade the binding on
    // its own invariants: injectivity, totality with a named residue, and
    // agreement where the answer is known. Each assertion carries a DISTINCT
    // message, and every negative control holds the guard's quantity fixed
    // while it mutates one thing — an early guard that fires first would make
    // the assertion under test unreachable and the control vacuous.

    /// One `.gl` record in the shape a REAL translation unit uses: the framing's
    /// preceding `80 <LE32 v>` field carries an arbitrary `v < 0x10000`, not the
    /// `0x10..` the fixtures happen to have. `pad` bytes sit between the name and
    /// the record, which is how the name→offset distance is exercised.
    /// `pad` is measured in *gap* — the distance from the name's terminating NUL
    /// to the offset field, which is what [`EMIT_MAX_NAME_TO_OFFSET`] bounds. The
    /// record's own fixed 8 bytes (the name NUL, then `80 <LE32 tid> 00 00`) are
    /// subtracted here so
    /// the tests can name the quantity the bound is stated in.
    fn emit_record(name: &str, tid: u16, body_off: u32, gap: usize) -> Vec<u8> {
        let pad = gap.saturating_sub(8);
        let mut v = vec![0u8];
        v.extend_from_slice(name.as_bytes());
        v.push(0);
        v.extend(std::iter::repeat(0x11u8).take(pad));
        v.push(0x80);
        v.extend_from_slice(&(tid as u32).to_le_bytes());
        v.extend_from_slice(&[0x00, 0x00, 0x80]);
        v.extend_from_slice(&body_off.to_le_bytes());
        v
    }

    fn three_emit_records() -> Vec<u8> {
        let mut gl = Vec::new();
        gl.extend_from_slice(&emit_record("?a@@YAHXZ", 0x19AB, 10, 0));
        gl.extend_from_slice(&emit_record("?b@@YAHXZ", 0xA4F6, 50, 0));
        gl.extend_from_slice(&emit_record("?c@@YAHXZ", 0x1001, 90, 0));
        gl
    }

    /// The binding, established: each record binds to the segment CONTAINING its
    /// body-start offset, and the residue is empty when every record is good.
    #[test]
    fn emit_binding_binds_each_record_to_the_row_containing_its_offset() {
        let b = EmitBinding::new(&three_emit_records(), &[0, 40, 80]);
        assert_eq!(b.records, 3, "three framed records must be found");
        assert_eq!(
            (b.name(0), b.name(1), b.name(2)),
            (Some("?a@@YAHXZ"), Some("?b@@YAHXZ"), Some("?c@@YAHXZ")),
            "offsets 10/50/90 land inside rows 0/1/2"
        );
        assert_eq!(b.bound(), 3, "all three rows must bind");
        let (records, accounted) = b.accounting();
        assert_eq!(
            (records, accounted),
            (3, 3),
            "the totality identity must hold with an empty residue"
        );
    }

    /// The framing the GATE uses would find one of these three, because it pins
    /// `gl[o-5] == 0x10` — a byte of the preceding field's *value*, not a tag.
    /// This is the over-fit that made `Bindings::per_record` report 34 records on
    /// a translation unit with 9,033 bodies, and it is pinned here so relaxing
    /// the gate's own predicate (which would move the accepted class) is a
    /// visible decision rather than an accident.
    #[test]
    fn the_gates_framing_sees_one_record_where_the_instrument_sees_three() {
        let gl = three_emit_records();
        let gate = (0..gl.len())
            .filter(|&p| p + 5 <= gl.len() && crate::codec::gl_offset_framed(&gl, p))
            .count();
        assert_eq!(
            gate, 1,
            "only the 0x1001-typed record satisfies the gate's `gl[o-5] == 0x10`"
        );
        assert_eq!(
            EmitBinding::new(&gl, &[0, 40, 80]).records,
            3,
            "the instrument's framing must not depend on the type id's value"
        );
    }

    /// INJECTIVITY. Two rows claiming one name is a reading error — `.gl` gives a
    /// defined function one record — so neither binds. Reported, not silent.
    #[test]
    fn a_name_two_rows_claim_binds_to_neither() {
        let mut gl = Vec::new();
        gl.extend_from_slice(&emit_record("?dup@@YAHXZ", 0x19AB, 10, 0));
        gl.extend_from_slice(&emit_record("?dup@@YAHXZ", 0x19AB, 50, 0));
        gl.extend_from_slice(&emit_record("?ok@@YAHXZ", 0x19AB, 90, 0));
        let b = EmitBinding::new(&gl, &[0, 40, 80]);
        assert_eq!(b.name(0), None, "the first claimant of a duplicated name must not bind");
        assert_eq!(b.name(1), None, "nor the second — a duplicate drops both");
        assert_eq!(b.name(2), Some("?ok@@YAHXZ"), "an unaffected row still binds");
        assert_eq!(
            b.dropped_name_conflict, 2,
            "both dropped rows must be counted in the residue, not vanish"
        );
        let (records, accounted) = b.accounting();
        assert_eq!((records, accounted), (3, 3), "the identity must still hold");
    }

    /// INJECTIVITY, the other direction: two records landing in one row means the
    /// segmentation and `.gl` disagree about where a body starts, and binding
    /// either name would be a coin flip.
    #[test]
    fn a_row_two_records_claim_binds_to_neither() {
        let mut gl = Vec::new();
        gl.extend_from_slice(&emit_record("?a@@YAHXZ", 0x19AB, 10, 0));
        gl.extend_from_slice(&emit_record("?b@@YAHXZ", 0x19AB, 20, 0));
        let b = EmitBinding::new(&gl, &[0, 40]);
        assert_eq!(b.name(0), None, "a row two records claim must bind nothing");
        assert_eq!(
            b.dropped_row_conflict, 2,
            "BOTH records are lost, and the identity is stated over records"
        );
        assert_eq!(b.accounting(), (2, 2), "the totality identity must hold");
    }

    /// The identity is stated over records, so a THREE-record collision has to
    /// account for three. Counting the row once instead read 1 of 3 and broke the
    /// identity on 607 workload TUs — the residue check catching its own
    /// arithmetic is the whole reason it is an assertion and not a comment.
    #[test]
    fn a_three_record_collision_accounts_for_all_three_records() {
        let mut gl = Vec::new();
        for off in [10u32, 20, 30] {
            gl.extend_from_slice(&emit_record("?a@@YAHXZ", 0x19AB, off, 0));
        }
        let b = EmitBinding::new(&gl, &[0, 40]);
        assert_eq!(b.records, 3, "three records must be found");
        assert_eq!(b.dropped_row_conflict, 3, "all three are lost to the collision");
        assert_eq!(b.bound(), 0, "and nothing binds");
        assert_eq!(
            b.accounting(),
            (3, 3),
            "records must equal what they became — this is the residue's own check"
        );
    }

    /// NEGATIVE CONTROL — the name-distance bound. The guard's quantity (two
    /// records, two rows) is held FIXED; only the padding between the second
    /// record's name and its offset field moves. Without the bound the second
    /// record borrows the first's name, which across 371 workload TUs produced
    /// 3,799 emitted symbols claimed by two rows each.
    #[test]
    fn a_record_whose_name_is_too_far_away_binds_nothing_rather_than_borrowing() {
        let near = {
            let mut gl = emit_record("?a@@YAHXZ", 0x19AB, 10, 0);
            gl.extend_from_slice(&emit_record("?b@@YAHXZ", 0x19AB, 50, EMIT_MAX_NAME_TO_OFFSET));
            gl
        };
        let control = EmitBinding::new(&near, &[0, 40]);
        assert_eq!(
            (control.name(0), control.name(1)),
            (Some("?a@@YAHXZ"), Some("?b@@YAHXZ")),
            "control: at exactly the bound, both records must still bind"
        );
        assert_eq!(control.records_nameless, 0, "control: nothing is nameless here");

        let far = {
            let mut gl = emit_record("?a@@YAHXZ", 0x19AB, 10, 0);
            gl.extend_from_slice(&emit_record(
                "?b@@YAHXZ",
                0x19AB,
                50,
                EMIT_MAX_NAME_TO_OFFSET + 1,
            ));
            gl
        };
        let b = EmitBinding::new(&far, &[0, 40]);
        assert_eq!(
            b.records, 2,
            "the mutation must not change how many records are FOUND — otherwise \
             this control tests the framing, not the name bound"
        );
        assert_eq!(
            b.name(1),
            None,
            "one byte past the bound, the record must bind nothing"
        );
        assert_ne!(
            b.name(1),
            Some("?a@@YAHXZ"),
            "and above all it must not borrow the PREVIOUS record's name"
        );
        assert_eq!(
            b.records_nameless, 1,
            "the unbindable record must appear in the nameless residue"
        );
    }

    /// NEGATIVE CONTROL — a record pointing before the first segment. Same two
    /// records, same two rows; only the second offset moves.
    #[test]
    fn a_record_pointing_before_the_first_row_is_residue_not_a_binding() {
        let mut gl = emit_record("?a@@YAHXZ", 0x19AB, 110, 0);
        gl.extend_from_slice(&emit_record("?b@@YAHXZ", 0x19AB, 150, 0));
        let control = EmitBinding::new(&gl, &[100, 140]);
        assert_eq!(
            (control.name(0), control.name(1)),
            (Some("?a@@YAHXZ"), Some("?b@@YAHXZ")),
            "control: with the rows at 100/140 both records bind"
        );
        assert_eq!(control.records_outside, 0, "control: nothing is outside");

        let mut gl = emit_record("?a@@YAHXZ", 0x19AB, 110, 0);
        gl.extend_from_slice(&emit_record("?b@@YAHXZ", 0x19AB, 20, 0));
        let b = EmitBinding::new(&gl, &[100, 140]);
        assert_eq!(b.records, 2, "the mutation must not change the record count");
        assert_eq!(
            b.records_outside, 1,
            "an offset before the first row must be counted as outside, not \
             clamped onto row 0"
        );
        assert_eq!(b.bound(), 1, "only the good record may bind");
    }

    /// NEGATIVE CONTROL — the framing itself. Same name, same offset value; only
    /// the two separator bytes move, and the record must disappear entirely
    /// rather than bind at a plausible-looking offset.
    #[test]
    fn a_broken_framing_yields_no_record_at_all() {
        let good = emit_record("?a@@YAHXZ", 0x19AB, 10, 0);
        let control = EmitBinding::new(&good, &[0]);
        assert_eq!(control.records, 1, "control: the intact record is found");
        assert_eq!(control.name(0), Some("?a@@YAHXZ"), "control: and it binds");

        let mut bad = good.clone();
        let sep = bad.len() - 6; // the `00 00` before the offset field's `80`
        bad[sep] = 0x01;
        let b = EmitBinding::new(&bad, &[0]);
        assert_eq!(
            b.records, 0,
            "a broken separator must yield NO record — not a record bound to \
             whatever the following four bytes read as"
        );
        assert_eq!(b.name(0), None, "and therefore no binding");
    }

    /// The varargs gate is name-derived, and both callers ask the SAME predicate
    /// through the same `Bindings` — that is the only reason the census and the
    /// gate cannot disagree about a variadic function.
    #[test]
    fn the_varargs_gate_is_one_predicate_on_both_paths() {
        assert!(mangled_is_varargs("?a5@@YAHHZZ"));
        assert!(mangled_is_varargs("?f@C@@QBAHHZZ"));
        // The discriminating neighbours from `mangled_is_varargs`'s own table.
        assert!(!mangled_is_varargs("?a1@@YAHH@Z"));
        assert!(!mangled_is_varargs("?a2@@YAHXZ"));
        assert!(!mangled_is_varargs("?a3@@YAHPAUZ@@@Z"));

        let mut gl = Vec::new();
        gl.extend_from_slice(&gl_record("?va@@YAHHZZ", 0));
        gl.extend_from_slice(&gl_record("?ok@@YAHH@Z", 40));
        let segs: Vec<&[u8]> = vec![&[0u8; 4], &[0u8; 4]];
        let gate = Bindings::per_record(&gl, None, &segs, &[0, 40]).expect("bound");
        let census = Bindings::positional(&gl, None, &segs);
        assert!(gate.is_varargs(0) && !gate.is_varargs(1));
        assert_eq!(
            (census.is_varargs(0), census.is_varargs(1)),
            (gate.is_varargs(0), gate.is_varargs(1)),
            "the two callers must never disagree about a variadic function"
        );
    }
}
