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
//! | segments | `split_functions_at` — every `4F 1F` | `split_function_bodies` — anchored on the `LO` body marker |
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

use super::gl::{gl_defined_names, mangled_names, source_path, GlIndex};
use super::sy::{SyLocals, SyView};

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
