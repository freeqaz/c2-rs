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

use super::gl::{
    gl_defined_names, gl_extern_data_names, gl_symbol_runs_all_separators, mangled_names,
    source_path, GlIndex,
};
use super::sy::{SyLocals, SyView};

/// How far a `.gl` function record's name may end before its body-offset field.
///
/// The same 32 bytes `gl::MAX_NAME_TO_OFFSET` uses, and for the same reason: it
/// is the bound that makes "nearest preceding run" mean *this record's* name
/// rather than whatever happened to be last in the file. It is also the single
/// most load-bearing constant in [`EmitBinding`] — see that type's docs for what
/// dropping it costs, measured.
///
/// **W-VGL: 32 is correct and must NOT be widened.** `ROADMAP.md` §9.18.3 read
/// the unbindable records as virtual members whose record "carries extra material
/// that breaks the framing *and* the 32-byte name-distance bound", which invites
/// raising this number. It was measured instead, once
/// [`gl_symbol_runs_all_separators`] let the reader see `26`-introduced names at
/// all: the name→offset distance then takes the values **15, 17, 19, 21, 23, 25,
/// 27** and nothing else, over 676 records on `TextFile.cpp` and 127 across the
/// held-out structural grid — **maximum 27**. The 85–194 byte distances that
/// motivated widening it were never a record's own name; they were the distance
/// to some *other* symbol, because this record's name was invisible. Widening
/// this constant would not have recovered one of them and would have started
/// borrowing names, which is a mis-emit.
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
/// framing therefore finds **38 framed records in a translation unit with 9,033
/// function bodies and 158 emitted functions** — of which the 32-byte name bound
/// then drops 4, leaving the **34** an earlier revision of this comment reported
/// as the framing's own count. The two numbers are different measurements of
/// different things and board #121 is the difference: **38 is the framing, 34 is
/// the reader.** (The 4 it drops are `?_Copy_str@exception@std@@AAAXPBD@Z`,
/// `?what@bad_exception@std@@UBAPBDXZ`, `??1bad_alloc@std@@UAA@XZ` and
/// `?_Ret@?$_BothPtrType@…@@SA?AU__true_type@2@XZ`, at distances 85/96/97/81 —
/// every one of them a `26`-separated-name case, see
/// [`gl_symbol_runs_all_separators`].)
///
/// Requiring only the two high bytes to be zero finds **6,069**, of which 5,908
/// land on a segment start and **0** bind two names to one offset. That is why
/// this type exists rather than a call to the gate's reader; the gate's own
/// predicate is deliberately left alone, because loosening it would move the
/// accepted class and this instrument must not.
///
/// **What the gate's 34 costs, measured rather than inferred (#121).**
/// `Bindings::per_record` refuses `src/App.cpp` outright: `gl_defined_names`
/// returns empty the moment one framed record's nearest preceding run is further
/// than 32 bytes away, and 4 of the 38 are. So the gate binds **0 of 9,033
/// bodies** there — not 34. A comment that reported 34 as "what the gate finds"
/// made an all-or-nothing refusal look like a partial read.
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
/// * **ARITY, because a residue of 0 is not a control (board #144).** Totality is
///   stated over records *as entities*; it cannot see a change that keeps every
///   record and loses something **inside** one. Moving a record from `bound` to
///   `records_nameless` satisfies the identity exactly. So
///   [`EmitBinding::record_offsets`] publishes the framed records' body-start
///   offsets — the record *contents*, which are a property of the framing alone
///   and must be **invariant under every change to the naming step**. #144 was
///   earned the hard way: dropping a `DUP` expansion left totality silent at
///   residue 0 while an arity check went 22 red.
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
    /// **The arity axis (#144)** — every framed record's body-start offset, in
    /// `.gl` file order, whether or not the record went on to bind a name.
    ///
    /// This is the record's *contents*, and it is a property of the **framing**
    /// only. Nothing downstream of the framing — the name scan, the distance
    /// bound, the conflict rules — may change it, so a diff here and a diff in
    /// [`EmitBinding::bound`] mean two different things and a report that prints
    /// only the second cannot tell them apart.
    offsets: Vec<u32>,
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
        let runs = gl_symbol_runs_all_separators(gl);
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
            // ARITY (#144): recorded HERE, before anything about naming is asked,
            // because that is what makes it a control on the naming step rather
            // than a restatement of it.
            out.offsets.push(off as u32);
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

    /// **The arity read-out (#144)** — every framed record's body-start offset,
    /// in `.gl` file order.
    ///
    /// Stated separately from [`EmitBinding::accounting`] because they can only
    /// go red for different reasons, and a report carrying one of them cannot
    /// substitute for the other:
    ///
    /// | change | totality residue | arity |
    /// |---|---|---|
    /// | a record stops being **framed** | moves | **moves** |
    /// | a record stops being **named** | 0 (it just changes bucket) | unchanged |
    ///
    /// So *arity moved and residue did not* means the framing changed, and
    /// *residue moved and arity did not* means only the naming changed — which is
    /// exactly the distinction W-VGL's `26`-separator repair had to be held to,
    /// and the one #144 records this project losing once already.
    pub fn record_offsets(&self) -> &[u32] {
        &self.offsets
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

    /// The arity identity, as a pair a caller asserts: one recorded offset per
    /// framed record. Breaks only if the framing loop and the arity axis have
    /// drifted apart, which is the one way [`EmitBinding::record_offsets`] could
    /// silently stop being a control.
    pub fn arity(&self) -> (usize, usize) {
        (self.records, self.offsets.len())
    }
}

/// True iff a `.gl` body-start offset field's `80 <LE32>` at `o` sits in the
/// record framing `80 <LE32 v> 00 00` with `v < 0x10000`.
///
/// This is [`crate::codec::gl_offset_framed`] with its `gl[o-5] == 0x10` clause
/// dropped — see [`EmitBinding`] for the measurement that says that clause pins
/// a *value* byte, not a tag. The gate's version is deliberately unchanged.
pub(crate) fn emit_offset_framed(gl: &[u8], o: usize) -> bool {
    o >= 7
        && gl[o] == 0x80
        && gl[o - 7] == 0x80
        && gl[o - 4] == 0x00
        && gl[o - 3] == 0x00
        && gl[o - 2] == 0x00
        && gl[o - 1] == 0x00
}

/// **Every name that owns a framed `.gl` body-start record**, whether or not
/// [`EmitBinding`] managed to bind it to a census row (W-EMITSET).
///
/// A record with a body-start offset is c1xx saying *this translation unit has
/// a body for this symbol*. [`EmitBinding`] then answers the harder question —
/// *which `.ex` segment is that body* — and loses 9.89 % of c2's emitted
/// symbols to `records_nameless`, `dropped_row_conflict` and
/// `dropped_name_conflict`. The two failures look identical downstream and they
/// are not the same thing at all:
///
/// * an emitted symbol with **no** record here has no body in this bundle, so a
///   port that emits one COMDAT per `.ex` segment can never produce it — a
///   **wall**, closable only by synthesizing the COMDAT;
/// * an emitted symbol **with** a record here does have a body and the row
///   binding merely failed to find it — an **instrument defect**, closable by
///   fixing this file.
///
/// The emit-set ceiling of `docs/ROADMAP.md` §9.16.3 is stated over the first
/// and measured over the second, so it cannot be read until they are separated.
/// That separation is this function's whole job.
///
/// **Diagnostic only.** Nothing in the gate, the census verdict or the emitter
/// consults it; `c2rs gap` reports the counts. Same framing and same
/// name-distance bound as [`EmitBinding::new`] — deliberately, so a difference
/// in the two answers is a difference in the *binding*, never in the reader.
pub fn gl_body_record_names(gl: &[u8]) -> std::collections::BTreeSet<String> {
    let runs = gl_symbol_runs_all_separators(gl);
    let ends: Vec<usize> = runs.iter().map(|&(_, end, _)| end).collect();
    let mut out = std::collections::BTreeSet::new();
    let mut p = 0usize;
    while p + 5 <= gl.len() {
        if !emit_offset_framed(gl, p) {
            p += 1;
            continue;
        }
        if let k @ 1.. = ends.partition_point(|&e| e <= p) {
            if p - ends[k - 1] <= EMIT_MAX_NAME_TO_OFFSET {
                out.insert(runs[k - 1].2.clone());
            }
        }
        p += 5;
    }
    out
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
    /// The `.gl` bytes, for the record-level reads the token index does not
    /// carry — [`Bindings::resolve_data`]'s linkage gate and
    /// [`Bindings::resolve_data_def`]'s record frame.
    gl: &'a [u8],
    /// **W-DATA** — the `.in` bytes, for [`Bindings::resolve_data_def`]'s
    /// initializer half.
    ///
    /// It is a **constructor argument** rather than something a caller attaches
    /// later, and that is the point: the census and the gate each build their
    /// own `Bindings`, and a defaulted-empty `.in` on one of them is precisely
    /// how the census comes to count a function in class that the gate refuses.
    /// Empty is a legitimate value (a TU with no initializers); it is not a
    /// value a caller can reach by forgetting.
    inb: &'a [u8],
    /// **WR1** — the undefined-external DATA names, built on first use, so a TU
    /// that references no global never walks `.gl` for them.
    extern_data: std::cell::OnceCell<std::collections::BTreeSet<String>>,
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
        inb: &'a [u8],
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
            gl,
            inb,
            extern_data: std::cell::OnceCell::new(),
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
    pub(crate) fn positional(
        gl: &'a [u8],
        inb: &'a [u8],
        sy: Option<&[u8]>,
        segs: &[&[u8]],
    ) -> Bindings<'a> {
        let names = mangled_names(gl);
        let paired = names.len() == segs.len();
        Bindings {
            paired,
            names,
            unclaimed: Vec::new(),
            src: source_path(gl),
            symbols: GlIndex::new(gl),
            gl,
            inb,
            extern_data: std::cell::OnceCell::new(),
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

    /// **WR1 — token → the name of an UNDEFINED-EXTERNAL DATA symbol.**
    ///
    /// [`Bindings::resolve`] plus the `.gl` linkage gate, as one predicate,
    /// because the two facts are asked at exactly one place and separating them
    /// is how a caller ends up applying one and not the other. Three populations
    /// it refuses, each for its own measured reason:
    ///
    /// * **a string literal** — its `.gl` record carries the `25` separator
    ///   `gl_symbol_index` excludes, so it resolves to nothing at all;
    /// * **a defined or static global** — [`gl_extern_data_names`], and the cost
    ///   of admitting one is a whole extra section (`docs/IL_CALL_IN_EXPR.md`
    ///   §17.2 item 7);
    /// * **a function** — its record fails the same frame check, so a callee's
    ///   token can never be read as an object's address.
    pub(crate) fn resolve_data(&self, tok: u32) -> Option<String> {
        let name = self.resolve(tok)?;
        self.extern_data
            .get_or_init(|| gl_extern_data_names(self.gl))
            .contains(&name)
            .then_some(name)
    }

    /// **W-DATA — token → a data object this TU DEFINES**, with its bytes.
    ///
    /// The mirror of [`Bindings::resolve_data`] and deliberately a *different*
    /// function rather than a widening of it: that one answers *"may the port
    /// reference this address without emitting a section?"* and its whole
    /// population is linkage `02`. This one answers *"does this TU define an
    /// object here, and what is in it?"*, and its population is exactly the two
    /// linkages that one refuses.
    ///
    /// Both readers are consulted, and the object is admitted only if they
    /// agree about it — `.gl` for the name, size, alignment and section kind,
    /// `.in` for the bytes. The clauses, each a refusal:
    ///
    /// * **COMDAT and INITIALIZED.** A non-COMDAT object is placed *before*
    ///   `.text` (GRID A cell `a4`, board #1682) and an uninitialized one is a
    ///   `.bss` COMDAT (cell `a3`); this lane graded a writer for neither.
    /// * **not thread-local.** `__declspec(thread)` lands in `.tls$` and says so
    ///   nowhere else in the record ([`gl::DATA_FLAG_THREAD_LOCAL`]).
    /// * **the `.in` value decodes to exactly `size` bytes.** Short, long, or
    ///   absent is a refusal and never a zero-fill — `IlBundle::data_tu`'s
    ///   clause 7, in this class.
    /// * **no relocations inside the initializer.** A tag-`02` element is a
    ///   pointer slot needing an ADDR32 into this object's own section, which
    ///   the COMDAT `.data` writer has no cell for. Board **#232**'s direction
    ///   is a `.data` whose bytes are right and whose addresses are not, so the
    ///   references gate the object rather than being dropped from it.
    ///
    /// Living on `Bindings` is what keeps the **census and the gate** asking one
    /// question: both build one of these and both call this, so the census
    /// cannot count a function in class that `IlBundle::functions` refuses for
    /// want of an object (`docs/GAPS.md` §6, and w-cfgclass's #1638).
    pub(crate) fn resolve_data_def(&self, tok: u32) -> Option<crate::func::IlDataDef> {
        let (_, o) = super::gl::gl_data_objects_ordered(self.gl)
            .into_iter()
            .find(|(t, _)| *t == tok)?;
        if !o.comdat || !o.initialized {
            return None;
        }
        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {
            return None;
        }
        let init = super::ininit::in_scalar_initializers(self.inb);
        // The `.in` reader's own totality identity, as a GATE. A silently lost
        // record is an object with no bytes, which is the failure this whole
        // path exists to prevent (`data_tu`'s clause 7/8 comment).
        if init.accepted + init.residue.len() != init.records {
            return None;
        }
        if !init.refs.get(&tok).map(|r| r.is_empty()).unwrap_or(true) {
            return None;
        }
        let bytes = init.values.get(&tok)?.clone();
        if bytes.len() != o.size as usize {
            return None;
        }
        Some(crate::func::IlDataDef {
            coff_name: o.coff_name,
            size: o.size,
            natural_align: o.natural_align,
            bytes,
            uninitialized: false,
        })
    }

    /// **W-WORDWRAP — the `.bss` sibling of [`Self::resolve_data_def`].**
    ///
    /// One token → the UNINITIALIZED object it names, when this TU defines one.
    /// The three gates are the exact complement of the `.data` resolver's, and
    /// each is a positive requirement rather than a relaxation:
    ///
    /// * **not COMDAT.** A COMDAT `.bss` is a function-local `static` with no
    ///   initializer (`gl.rs`'s cell `a3`, attribute `20`), and it is placed
    ///   *after* the code groups where a non-COMDAT one is placed before them —
    ///   two different section orders, and no cell has graded the first.
    /// * **not initialized.** An initialized object is `.data` and belongs to
    ///   the resolver above; admitting one here would emit a `.bss` for an
    ///   object whose bytes exist.
    /// * **not thread-local**, the same clause `resolve_data_def` draws.
    ///
    /// **There is no `.in` read at all**, and that is the point: an
    /// uninitialized object has no `.in` record, so the totality gate the
    /// `.data` path runs has nothing to check here. `size` and `natural_align`
    /// come from the `.gl` record exactly as they do there.
    ///
    /// The result is marked [`crate::func::IlDataDef::uninitialized`], which the
    /// COFF writer refuses by name — see that field for what is and is not
    /// answerable about a `.bss` object today.
    pub(crate) fn resolve_bss_def(&self, tok: u32) -> Option<crate::func::IlDataDef> {
        let (_, o) = super::gl::gl_data_objects_ordered(self.gl)
            .into_iter()
            .find(|(t, _)| *t == tok)?;
        if o.comdat || o.initialized {
            return None;
        }
        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {
            return None;
        }
        if o.size == 0 {
            return None;
        }
        Some(crate::func::IlDataDef {
            coff_name: o.coff_name,
            size: o.size,
            natural_align: o.natural_align,
            bytes: Vec::new(),
            uninitialized: true,
        })
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

/// **W-INLFENCE — the callee this TU also DEFINES, or `None`.**
///
/// The one predicate behind the port's inline fence, asked by
/// [`crate::IlBundle::functions`] (as a whole-TU refusal), by the census (as a
/// per-function one) and by [`super::diag`]'s re-ask, so the three cannot
/// disagree about what the port may emit.
///
/// # What it is a fence against
///
/// **c2 inlines**, and the port does not. `int f(int); int use(int a){return
/// f(a);} int f(int a){return a+1;}` gets a `.text` of *two* copies of `addi
/// r3,r3,1 ; blr` and **no relocations** — c2 cloned `f` into `use` rather than
/// branching to it, and the port's `b ?f` against an undefined external
/// mismatched at file offset 8. Lane `w-fltret` measured the same mechanism on
/// the workload at scale (`docs/rungs/2026-08-09-w-fltret.md` §6, board #2082):
/// `?SplitMs@Timer@@QAAMXZ`'s reference body is **31 words** where the port
/// emits 13, because `Timer::Split()` and `Timer::Ms()` are `inline` members
/// defined in the same header, and **not one of that class's 444 functions is
/// byte-exact**.
///
/// # Why the test is DEFINED-HERE and not a size or a ceiling
///
/// `docs/whitebox/WB_INLINE_FINDINGS.md` measures c2's decision on 320 obj
/// cells and its §7 says of the accept side: *"The accept side is not
/// offered"* — a mis-predicted accept is a wrong obj, and none of the measured
/// boundaries is a number (they are brackets: `(300,308]`, `(212,252]`,
/// `(56,80]` for a loop-bodied callee). **Nothing from that document is copied
/// here.** What this predicate uses is the one categorical fact that needs no
/// constant at all: *c2 cannot inline a body it does not have.* Where the
/// callee is a true external the port keeps its own call and is byte-exact —
/// 5,172 `tail` and 1,238 `seq` emitted functions are graded byte-exact against
/// real c2 at base, every one of them a call this fence must not take.
///
/// # The direction it fails in
///
/// `defined` is the set of names this TU is *known* to define. A name **in** it
/// is certainly defined here and the function is refused. A name **absent** may
/// still be defined here on a TU whose `.gl` records the walk could not frame —
/// [`super::gl::gl_defined_names`] yields an empty pair when it stops. That
/// residue is fail-OPEN in the census and fail-CLOSED in the gate, which
/// refuses such a TU for want of names before this is ever asked; it is sized
/// rather than hidden (`docs/rungs/2026-08-09-w-inlfence.md` §5).
///
/// Returns the offending name so a caller can report *which* callee, rather
/// than a bare bool that makes every row look alike.
pub(crate) fn callee_defined_here<'a>(
    f: &'a crate::func::IlFunction,
    defined: &std::collections::BTreeSet<String>,
) -> Option<&'a str> {
    f.callees().find(|c| defined.contains(*c))
}

/// [`callee_defined_here`], minus a caller-supplied set of callees for which
/// **that caller already has an answer**.
///
/// The two callers supply two different sets, and neither is the other's:
///
/// | caller | `exempt` | what the exemption asserts |
/// |---|---|---|
/// | the **census** | `census::tu_modelled_callees` | mechanism **E** (`c2_core::elide`) — the callee emits **nothing**, so the call costs no branch. Graded **1,877 of 1,877 byte-exact**. DEPTH 1 where `elide`'s reduction is a fixpoint, so it under-exempts and never over-exempts |
/// | the **gate** ([`crate::IlBundle::functions`]) | `gl::plain_external_defined_names`, at `/O1` | **W-FENCE2** — the callee's linkage class is the one `WB_INLINE_FINDINGS` F2 measured the decline ceiling on, and it is not `inline`/`__forceinline` (F4). It asserts nothing about *size*: the size question is asked one stage later, at `c2_core::comdat::fenced_inlined_callee`, where a lowered body exists to measure |
///
/// **The gate's exemption is NOT a claim that the call survives.** It is a claim
/// that the *facts the composition seam needs are available* — which is why it
/// is sound for `IlBundle::functions` to hand the TU on rather than refuse it,
/// and why `w-inlfence`'s D8 (*"the gate does not take the exemption"*) is
/// superseded rather than contradicted: that lane had no seam to hand to, and
/// `w-inlfence2` built one on the same day.
pub(crate) fn callee_defined_here_unmodelled<'a>(
    f: &'a crate::func::IlFunction,
    defined: &std::collections::BTreeSet<String>,
    exempt: &std::collections::BTreeSet<String>,
) -> Option<&'a str> {
    f.callees()
        .find(|c| defined.contains(*c) && !exempt.contains(*c))
}

/// The set [`callee_defined_here`] tests against, for a caller that has only
/// `.gl` — the census, whose [`Bindings::positional`] names are all mangled
/// names and not the defined ones.
///
/// Every member is a name a `.gl` record with a **body-start offset** claimed,
/// i.e. a function this TU defines. The set is a subset of the truth and never
/// a superset, which is what makes a membership test sound in the refusing
/// direction on a TU that does not fully bind.
pub(crate) fn defined_name_set(gl: &[u8]) -> std::collections::BTreeSet<String> {
    gl_defined_names(gl).0.into_iter().map(|(_, n)| n).collect()
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
        let b = Bindings::per_record(&gl, &[], None, &segs, &[0, 40])
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
        assert!(Bindings::per_record(&gl, &[], None, &segs, &[0, 41]).is_none());
        assert!(Bindings::per_record(&gl, &[], None, &segs, &[4, 40]).is_none());
        // Order matters too: the same two offsets, swapped, is a divergence.
        assert!(Bindings::per_record(&gl, &[], None, &segs, &[40, 0]).is_none());
        // …and a count mismatch is the same refusal.
        assert!(Bindings::per_record(&gl, &[], None, &segs[..1], &[0]).is_none());
    }

    /// **FEWER records than segments is a refusal, and relaxing the `!=` above to
    /// `>` is a wrong emit** — the direction that check exists for. It had no
    /// test, and it was measured as a live mis-emit rather than argued about.
    ///
    /// The relaxation reads plausibly, which is why it was proposed. `.gl` omits
    /// a function c2 dropped (an unreferenced internal-linkage one — its symbol
    /// is absent from `.gl` **entirely**, verified on
    /// `65-linkage-comdat-0012.cpp`, whose only mangled run is `?f@@YAHH@Z`);
    /// the emit loop in [`IlBundle::functions`] is
    /// `names.iter().take(n).zip(&segs)` and so is bounded by the *name* count;
    /// so the port emits the bound prefix and drops the tail, which is what c2
    /// did. It gains **8** cases in `scripts/sweep.d/65-linkage-comdat.py`
    /// (37 → 45 `Port=Match`, measured in both directions) at **0** mismatches
    /// over the whole generated corpus.
    ///
    /// It is still wrong, because "no record for this segment" has a second
    /// cause the generated corpus never writes: a record
    /// [`crate::codec::gl_offset_framed`] cannot **see**. That framing pins the
    /// record's preceding `80 <LE32>` field — the signature's CodeView type
    /// index — into `0x1000..=0x10FF`, and a TU with enough distinct types walks
    /// a *later* record's index out of the window while the earlier ones stay
    /// inside it. The tail record is then invisible rather than absent, and the
    /// port drops a function c2 emitted.
    ///
    /// Two guards catch most of that and neither catches this. `gl_defined_names`
    /// has five total-refusal paths that make `bound` **empty**, and
    /// `PortC2::build` then refuses because [`IlBundle::shell_only_tu`] asks
    /// `is_empty_module(.ex)`, which a TU with code fails. And a mangled name no
    /// record claimed lands in `unclaimed`, which `functions()` refuses. An
    /// `extern "C"` name contains no `@@`, so [`super::gl::looks_mangled`] is
    /// false and it is **not** accounted — the one shape that slips both.
    ///
    /// MEASURED, `work/w-small/probe/l1_counterexample.cpp`, `/Ox /GS- /c`:
    /// `int f(int)` and `int h(int,int)`, then 63 dropped-static burners, then
    /// `extern "C" int g(char*)`. The three records' type indices are `0x10FD`,
    /// `0x10FF` and `0x1101` — the first two framed, the third not.
    /// `Port=NotImplemented` today; **`Port=Mismatch @ offset 8`** under `>`,
    /// with `g` present as symbol 15 of the reference obj's `.text` and absent
    /// from the port's. The burner count is a **boundary**: 62 and 64 both still
    /// refuse under the relaxation and only 63 mis-emits, so the counterfactual
    /// is specific to this hole rather than a blanket breaker.
    #[test]
    fn per_record_refuses_a_short_prefix_because_an_unseen_record_looks_like_a_dropped_one() {
        // Three segments, two records, and the records' offsets ARE the first
        // two split points — the prefix is aligned and it is still a refusal.
        let gl = two_records();
        let segs: Vec<&[u8]> = vec![&[0u8; 4], &[0u8; 4], &[0u8; 4]];
        assert!(
            Bindings::per_record(&gl, &[], None, &segs, &[0, 40, 80]).is_none(),
            "an aligned PREFIX of records must not bind: the unbound tail may be \
             a record the framing cannot see, not a function c2 dropped"
        );
        // …and one record against two segments is the same refusal, which is the
        // exact shape `65-linkage-comdat`'s eight flipping cases present.
        let one = gl_record("?a@@YAHXZ", 0);
        assert!(Bindings::per_record(&one, &[], None, &segs[..2], &[0, 40]).is_none());
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
        let b = Bindings::positional(&gl, &[], None, &three);
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
        let b = Bindings::positional(&gl, &[], None, &two);
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

        let per_record = Bindings::per_record(&gl, &[], None, &segs, &[0, 40])
            .expect("the records' offsets are the split points");
        let positional = Bindings::positional(&gl, &[], None, &segs);

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
        emit_record_sep(0x00, name, tid, body_off, gap)
    }

    /// [`emit_record`] with the byte that **introduces** the name spelled out.
    /// `.gl` uses `00` or `26` (`gl::NAME_SEPARATORS`), and which one it is is the
    /// single mutation W-VGL's controls turn.
    fn emit_record_sep(sep: u8, name: &str, tid: u16, body_off: u32, gap: usize) -> Vec<u8> {
        let pad = gap.saturating_sub(8);
        let mut v = vec![sep];
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

    /// **W-EMITSET — the wall/defect split, on the case that discriminates it.**
    ///
    /// The two records collide on one row, so [`EmitBinding`] binds neither and
    /// both names are lost. If `gl_body_record_names` were derived from the
    /// binding it would lose them too, and the emit-set ceiling would read the
    /// collision as "c2 emitted a symbol with no body" — a wall — when the body
    /// is right there and it is this file that cannot find it. So the assertion
    /// is that the two readers **disagree here**, in this direction: the binding
    /// says nothing, the record scan says both.
    ///
    /// `?c@@YAHXZ` is the control that could fail the other way: it binds
    /// normally, and a record scan that reported only *unbound* names would drop
    /// it and make the two sets disjoint instead of nested.
    #[test]
    fn a_body_record_is_reported_even_when_its_row_binding_collides() {
        let mut gl = Vec::new();
        gl.extend_from_slice(&emit_record("?a@@YAHXZ", 0x19AB, 10, 0));
        gl.extend_from_slice(&emit_record("?b@@YAHXZ", 0x19AB, 20, 0));
        gl.extend_from_slice(&emit_record("?c@@YAHXZ", 0x1001, 50, 0));
        let b = EmitBinding::new(&gl, &[0, 40]);
        assert_eq!(b.name(0), None, "the collided row binds nothing");
        assert_eq!(b.name(1).as_deref(), Some("?c@@YAHXZ"));
        let recs = gl_body_record_names(&gl);
        assert!(
            recs.contains("?a@@YAHXZ") && recs.contains("?b@@YAHXZ"),
            "a body the BINDING lost is still a body this bundle has: {recs:?}"
        );
        assert!(recs.contains("?c@@YAHXZ"), "and the bound one is not dropped");
        assert_eq!(recs.len(), 3);
    }

    /// …and the wall direction, which is the one the ceiling is stated over: a
    /// symbol with no record at all must NOT be reported. Without this the split
    /// is vacuous — every name would look like an instrument defect and
    /// `emit-unbound-no-record` would be 0 by construction, which is exactly the
    /// absence-read-as-success shape (#144).
    #[test]
    fn a_symbol_with_no_body_record_is_not_reported() {
        let gl = emit_record("?a@@YAHXZ", 0x19AB, 10, 0);
        let recs = gl_body_record_names(&gl);
        assert!(recs.contains("?a@@YAHXZ"));
        assert!(
            !recs.contains("??_EFoo@@UAAPAXI@Z"),
            "a vector deleting destructor c2 synthesizes has no record and must \
             not be invented: {recs:?}"
        );
        // …and the name-distance bound applies here exactly as it does in the
        // binding: a record whose name is 40 bytes away is a record shape this
        // reader does not know, and guessing is what it exists not to do.
        let far = emit_record("?far@@YAHXZ", 0x19AB, 10, 40);
        assert!(
            !gl_body_record_names(&far).contains("?far@@YAHXZ"),
            "the name-distance bound must refuse, not borrow"
        );
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

    /// **W-VGL / board #151 — the `26` name separator, and the ARITY control that
    /// says which half of the reader moved.**
    ///
    /// The guard's quantity is held fixed in the strongest available sense: the
    /// two inputs differ in **exactly one byte**, the one that introduces the
    /// second record's name. Everything the framing reads is identical, so:
    ///
    /// * `records` and [`EmitBinding::record_offsets`] must be **unchanged** —
    ///   this is the #144 arity axis, and it is what says the repair moved the
    ///   *naming* step and not the framing;
    /// * the binding must be unchanged too, which is the point: a `26`-introduced
    ///   name is a name, and before this repair it was not merely mis-framed, it
    ///   was invisible, so the record went to `records_nameless` and its symbol
    ///   was counted as a body this bundle does not have.
    ///
    /// `??_G…` is the real population: on `src/system/obj/TextFile.cpp` this is 70
    /// of 674 framed records, and `??_GDataArray@@AAAPAXI@Z` — an emitted symbol
    /// the wall dump reported as `no-record` — appears **zero** times as a
    /// NUL-delimited run and once as a `26`-introduced one.
    #[test]
    fn a_26_separated_name_binds_and_the_framing_arity_does_not_move() {
        let build = |sep: u8| {
            let mut gl = emit_record("?a@@YAHXZ", 0x19AB, 10, 0);
            gl.extend_from_slice(&emit_record_sep(sep, "??_Gb@@UAAPAXI@Z", 0x19AB, 50, 0));
            gl
        };
        let nul = EmitBinding::new(&build(0x00), &[0, 40]);
        let com = EmitBinding::new(&build(0x26), &[0, 40]);

        // ARITY (#144) — the framing's own contents, invariant under the mutation.
        assert_eq!(
            nul.records, com.records,
            "the mutation must not change how many records are FRAMED — otherwise \
             this control tests the framing, not the name scan"
        );
        assert_eq!(
            nul.record_offsets(),
            com.record_offsets(),
            "ARITY: a change to the NAMING step must leave every framed record's \
             body offset exactly where it was"
        );
        assert_eq!(com.arity(), (2, 2), "one recorded offset per framed record");

        // …and the naming step, which is the half that was broken.
        assert_eq!(
            com.name(1),
            Some("??_Gb@@UAAPAXI@Z"),
            "a name introduced by `26` is a name; it must bind, not go nameless"
        );
        assert_eq!(
            (com.bound(), com.records_nameless),
            (nul.bound(), nul.records_nameless),
            "the two separators must produce the SAME binding — that is the whole \
             claim, and the residue must be empty under both"
        );
        assert_eq!(com.accounting(), (2, 2), "the totality identity still holds");
    }

    /// **W-VGL — the corrupted names this repair also fixes, and why a run has to
    /// TERMINATE at `26` and not merely open there.**
    ///
    /// Record bytes that happen to be printable ASCII sit immediately before the
    /// separator, and a scan that only splits on NUL glues them onto the front of
    /// the next name. Measured on `TextFile.cpp`, which was emitting fourteen of
    /// these, `"H=&??_7FixedSizeAlloc@@6B@"` among them — `H=` is `0x48 0x3D`.
    ///
    /// A name that is wrong in its first two bytes is worse than a missing one:
    /// it is a plausible-looking symbol that no obj carries, and `docs/GAPS.md` §6
    /// is the rule it breaks.
    #[test]
    fn record_bytes_before_a_26_are_not_glued_onto_the_next_name() {
        let mut gl = vec![0x00, b'H', b'='];
        gl.extend_from_slice(&emit_record_sep(
            0x26,
            "??_7FixedSizeAlloc@@6B@",
            0x19AB,
            10,
            0,
        ));
        let b = EmitBinding::new(&gl, &[0]);
        assert_eq!(b.records, 1, "one record, whatever the name scan decides");
        assert_eq!(
            b.name(0),
            Some("??_7FixedSizeAlloc@@6B@"),
            "the name must be the name, not `H=&` plus the name"
        );
        assert!(
            !b.name(0).is_some_and(|n| n.contains('&')),
            "a `26` inside a bound name means the run swallowed its own separator"
        );
    }

    /// …and the fail-closed direction, so the repair cannot be the
    /// absence-read-as-success shape in reverse: `26` introduces a name only when
    /// what follows it *is* one. The record count is held fixed by the mutation.
    #[test]
    fn a_26_that_introduces_no_plausible_name_yields_no_binding() {
        let mut gl = emit_record_sep(0x26, "?ok@@YAHXZ", 0x19AB, 10, 0);
        let good = EmitBinding::new(&gl, &[0]);
        assert_eq!(good.name(0), Some("?ok@@YAHXZ"), "control: a real name binds");

        // Same bytes, same record, but the name now starts with a digit — not an
        // identifier, so there is no run and nothing may be borrowed.
        gl[1] = b'9';
        let b = EmitBinding::new(&gl, &[0]);
        assert_eq!(
            b.records, good.records,
            "the mutation must not change the record count"
        );
        assert_eq!(b.name(0), None, "no plausible name means no binding");
        assert_eq!(
            b.records_nameless, 1,
            "and the record must appear in the named residue, not vanish"
        );
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
        let gate = Bindings::per_record(&gl, &[], None, &segs, &[0, 40]).expect("bound");
        let census = Bindings::positional(&gl, &[], None, &segs);
        assert!(gate.is_varargs(0) && !gate.is_varargs(1));
        assert_eq!(
            (census.is_varargs(0), census.is_varargs(1)),
            (gate.is_varargs(0), gate.is_varargs(1)),
            "the two callers must never disagree about a variadic function"
        );
    }

    /// **W-INLFENCE — the fence matches a whole mangled NAME, never a prefix.**
    ///
    /// The near-miss is in `fixtures/cpp/winlfence_opaque_callee.cpp` as cell
    /// F5 and graded byte-exact there; this is the same claim at unit cost, so
    /// the day someone rewrites the set membership as a `starts_with` the
    /// portable lane fails without a toolchain.
    #[test]
    fn the_inline_fence_matches_a_whole_name_and_not_a_prefix() {
        let defined: std::collections::BTreeSet<String> =
            ["?wif_local_leaf@@YAHH@Z".to_string()].into_iter().collect();
        let mut f = crate::func::IlFunction::base("?caller@@YAHH@Z", &None);
        f.tail_call = Some("?wif_local_leaf_x@@YAHH@Z".to_string());
        assert_eq!(
            callee_defined_here(&f, &defined),
            None,
            "a callee of which a defined name is a strict PREFIX is a different \
             symbol and c2 has no body for it"
        );
        f.tail_call = Some("?wif_local_leaf@@YAHH@Z".to_string());
        assert_eq!(
            callee_defined_here(&f, &defined),
            Some("?wif_local_leaf@@YAHH@Z"),
            "the exact name is the fence"
        );
    }

    /// **The fence reads EVERY call edge, because it reads
    /// [`crate::func::IlFunction::callees`].**
    ///
    /// A fence written against `tail_call` alone would have been green on every
    /// cell of `winlfence_local_callee_neg.cpp` that is a tail call and silent
    /// on its `CallSeq` ones — which is five of the seven, including the shape
    /// of the one function the whole 878-TU workload takes back.
    #[test]
    fn the_inline_fence_reads_every_call_edge_and_not_only_the_tail() {
        let defined: std::collections::BTreeSet<String> =
            ["?g@@YAHH@Z".to_string()].into_iter().collect();
        let mut f = crate::func::IlFunction::base("?caller@@YAHH@Z", &None);
        f.framed_call = Some(crate::func::FramedCall {
            callee: "?g@@YAHH@Z".to_string(),
            add_k: 0,
        });
        assert_eq!(callee_defined_here(&f, &defined), Some("?g@@YAHH@Z"));
    }

    /// **An empty defined set fences nothing**, which is the direction this
    /// predicate fails in and the reason the rung sizes the residue instead of
    /// claiming coverage: `gl_defined_names` yields an empty pair when its walk
    /// stops, and that is **845 of the 871 captured workload TUs**.
    #[test]
    fn an_empty_defined_set_fences_nothing_and_that_is_the_open_direction() {
        let none = std::collections::BTreeSet::new();
        let mut f = crate::func::IlFunction::base("?caller@@YAHH@Z", &None);
        f.tail_call = Some("?g@@YAHH@Z".to_string());
        assert_eq!(callee_defined_here(&f, &none), None);
    }
}
