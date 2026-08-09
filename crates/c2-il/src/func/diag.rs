//! **Why a bundle does not decode** — the decomposition of `gap.rs`'s
//! `vocab-gap` bucket into named causes with counts (lane `w-vocab`).
//!
//! # What problem this solves
//!
//! `c2rs gap` files 863 of 878 real workload TUs as `vocab-gap`, with one
//! reason string for all of them: `il function decode failed`. That is the
//! largest single number on the board and it is not actionable, because
//! [`IlBundle::functions`] **short-circuits**: it returns `None` at the first
//! of eleven independent gates, so a TU blocked by six of them is
//! indistinguishable from a TU blocked by one.
//!
//! [`IlBundle::decode_causes`] answers the actionable version of the question:
//! *which* gates would fire, evaluated as independently as the data allows, so
//! the ceiling of any one repair can be counted rather than estimated.
//!
//! # This is a DIAGNOSTIC and it must never become a gate
//!
//! Nothing here decides acceptance. [`IlBundle::functions`] is untouched and
//! remains the sole acceptance path; this module re-asks its predicates and
//! reports. The one place that could rot is drift between the two, so the
//! struct carries [`DecodeCauses::decodes`] — read from the real
//! [`IlBundle::decodes`] — and the invariant `causes.is_empty() == decodes` is
//! checkable per TU by any caller and is asserted by this crate's tests. A
//! diagnostic that silently disagreed with the gate would be worse than none:
//! it would produce a ranked repair list for a compiler that is not this one.
//!
//! # Independence, and where it stops
//!
//! Some gates cannot be asked without the answer to an earlier one. The binding
//! is the boundary: a TU whose `.gl` records do not bind has no names, so
//! "would the callee resolve" and "is any `.gl` symbol unclaimed" are not
//! questions with answers. Those are reported as **not evaluated**
//! ([`DecodeCauses::downstream_evaluated`]) rather than as absent, because
//! absence read as success is this project's most-recorded failure mode
//! (`docs/STATUS.md` trap 5). What *is* evaluated for every TU is the per-body
//! decode, because the automatic-locals view a body parse needs
//! (`SyLocals::new`) is a function of `.sy` and the segment list only, and
//! neither depends on the naming.

use super::bind::{emit_offset_framed, Bindings};
use super::body::parse_segment;
use super::bundle::{split_functions_at, LO_MARKER};
use super::gl::{drectve_is_boilerplate, gl_defined_names_framed, label_counter, GlBindStop};
use super::readers::find_subslice;
use super::bundle::shape_to_function;
use crate::IlBundle;

/// One named reason a bundle does not decode. The strings are an **interface**
/// — a histogram keyed on them is meant to be comparable across sessions, so
/// they are appended to, never renamed.
pub mod cause {
    /// The bundle has no `.gl` file at all.
    pub const NO_GL: &str = "no-gl";
    /// The bundle has no `.ex` file at all.
    pub const NO_EX: &str = "no-ex";
    /// `.drectve` is not the constant the writer emits — a `#pragma
    /// comment(lib, …)` or a `/EXPORT:`.
    pub const DRECTVE: &str = "drectve-not-boilerplate";
    /// No `4F 1F` function start, but an `LO` body marker is present: a `.ex`
    /// this reader failed to split, not an empty module.
    pub const SPLIT_EMPTY_WITH_LO: &str = "split-empty-with-lo";

    /// `gl_defined_names` stopped: a framed record whose name is absent or
    /// further than 32 bytes away.
    pub const GL_NAME_TOO_FAR: &str = "gl-stop-name-too-far";
    /// …a record name with no `@@` (an undecorated `extern "C"`).
    pub const GL_NAME_NOT_MANGLED: &str = "gl-stop-name-not-mangled";
    /// …a name run that ends at `26`.
    pub const GL_RUN_ENDS_26: &str = "gl-stop-run-ends-26";
    /// …`__declspec(dllexport)` linkage.
    pub const GL_DLLEXPORT: &str = "gl-stop-dllexport";
    /// …a `26`-**introduced** defined name (COMDAT linkage, board #232).
    pub const GL_26_INTRODUCED: &str = "gl-stop-26-introduced";

    /// The records bound, but their count is not the `.ex` segment count.
    pub const BIND_COUNT: &str = "bind-record-count-ne-segments";
    /// The counts agree but a record's offset is not its segment's start.
    pub const BIND_OFFSET: &str = "bind-offset-ne-segment-start";

    /// A defined name that mangles as variadic.
    pub const VARARGS: &str = "varargs";
    /// At least one `.ex` segment is outside the modeled body class.
    pub const BODY_DECODE: &str = "body-out-of-class";
    /// A body decoded, but a CALL or data token did not resolve to a `.gl`
    /// symbol.
    pub const SHAPE_RESOLVE: &str = "shape-token-unresolved";
    /// A framed function shares the TU with a class whose label stride the
    /// counter does not charge.
    pub const LABEL_STRIDE: &str = "label-stride-mismatch";
    /// A framed function and no readable `$M` seed in `.gl`.
    pub const LABEL_COUNTER: &str = "label-counter-unreadable";
    /// A mangled `.gl` symbol no record claimed and no function accounts for.
    pub const UNCLAIMED: &str = "unclaimed-gl-symbol";
    /// A callee this TU also defines — c2 may inline it.
    pub const LOCAL_CALLEE: &str = "locally-defined-callee";
}

/// The decomposition of one bundle's refusal. See the module docs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DecodeCauses {
    /// Every cause that fires, ascending and deduplicated. Empty **iff** the
    /// bundle decodes (see [`DecodeCauses::decodes`]).
    pub causes: Vec<&'static str>,
    /// The cause [`IlBundle::functions`] actually stops on — the first-cause
    /// histogram's key. `None` when `functions()` accepts.
    pub first: Option<&'static str>,
    /// What [`IlBundle::decodes`] says, read from the real predicate. The
    /// anti-drift control: `causes.is_empty() == decodes` for every bundle.
    pub decodes: bool,
    /// `dyninit_tu()` accepted — the second acceptance path, which can make
    /// `decodes` true while `causes` is non-empty on the *function* path.
    pub whole_tu: bool,

    /// `4F 1F` segments in `.ex` — the count the port consumes.
    pub segments: usize,
    /// `.gl` body-start records the **gate's** framing can see
    /// (`codec::gl_offset_framed`, whose `gl[o-5] == 0x10` clause pins the
    /// preceding field into `0x1000..=0x10FF`).
    pub records_gate: usize,
    /// `.gl` body-start records the **window-free** framing can see
    /// (`bind::emit_offset_framed`, the same record shape with the value
    /// restricted only to `< 0x10000`).
    pub records_wide: usize,
    /// `.ex` segments whose body is outside the modeled class. Evaluated for
    /// every bundle with a splittable `.ex`, whatever the binding did.
    pub bodies_out_of_class: usize,

    /// Whether the post-binding gates (resolve, label, unclaimed, local
    /// callee) were asked at all. `false` means the binding failed and those
    /// questions have no answers on this TU — **not** that they passed.
    pub downstream_evaluated: bool,
    /// **The AB-g measurement.** The gate's framing refuses this TU and the
    /// window-free framing binds it: every `.gl` record's offset is its
    /// segment's start, 1:1 and in order, and no stop clause fires. This is
    /// "the type-index window is *a* cause here", not "it is the only one" —
    /// for that, intersect with `causes` (see the rung).
    pub window_blocks_binding: bool,
}

impl DecodeCauses {
    fn push(&mut self, c: &'static str) {
        if self.first.is_none() {
            self.first = Some(c);
        }
        if !self.causes.contains(&c) {
            self.causes.push(c);
        }
    }
}

fn stop_cause(s: GlBindStop) -> &'static str {
    match s {
        GlBindStop::NameTooFar => cause::GL_NAME_TOO_FAR,
        GlBindStop::NameNotMangled => cause::GL_NAME_NOT_MANGLED,
        GlBindStop::RunEndsAt26 => cause::GL_RUN_ENDS_26,
        GlBindStop::DllexportLinkage => cause::GL_DLLEXPORT,
        GlBindStop::Name26Introduced => cause::GL_26_INTRODUCED,
    }
}

/// Count the framed records a given framing predicate can see, with no naming
/// and no gating — the arity axis of the two framings, so "the gate cannot see
/// N records" is a count rather than an inference.
fn framed_record_count(gl: &[u8], framed: fn(&[u8], usize) -> bool) -> usize {
    let mut n = 0usize;
    let mut p = 0usize;
    while p + 5 <= gl.len() {
        if framed(gl, p) {
            n += 1;
            p += 5;
        } else {
            p += 1;
        }
    }
    n
}

/// Does `gl_defined_names` under `framed` produce a binding the gate would
/// accept — 1:1 with the segments, offsets equal, in order?
fn binds_under(
    gl: &[u8],
    segs_len: usize,
    starts: &[usize],
    framed: fn(&[u8], usize) -> bool,
) -> bool {
    match gl_defined_names_framed(gl, true, framed) {
        Ok((bound, _)) => {
            bound.len() == segs_len
                && bound
                    .iter()
                    .zip(starts)
                    .all(|(&(off, _), &s)| off as usize == s)
        }
        Err(_) => false,
    }
}

impl IlBundle {
    /// **Decompose this bundle's decode refusal into named, counted causes.**
    ///
    /// See the module docs for what is and is not evaluated independently, and
    /// for the anti-drift invariant this must satisfy.
    pub fn decode_causes(&self) -> DecodeCauses {
        let mut out = DecodeCauses {
            decodes: self.decodes(),
            whole_tu: self.dyninit_tu().is_some(),
            ..DecodeCauses::default()
        };

        let (gl, ex) = match (self.get("gl"), self.ex()) {
            (Some(gl), Some(ex)) => (gl, ex),
            (gl, ex) => {
                if gl.is_none() {
                    out.push(cause::NO_GL);
                }
                if ex.is_none() {
                    out.push(cause::NO_EX);
                }
                return out;
            }
        };

        out.records_gate = framed_record_count(gl, crate::codec::gl_offset_framed);
        out.records_wide = framed_record_count(gl, emit_offset_framed);

        if !drectve_is_boilerplate(gl) {
            out.push(cause::DRECTVE);
        }

        let (starts, segs) = split_functions_at(ex);
        out.segments = segs.len();
        if segs.is_empty() {
            if find_subslice(ex, &LO_MARKER).is_some() {
                out.push(cause::SPLIT_EMPTY_WITH_LO);
            }
            return out;
        }

        // ── the binding, under the gate's framing ─────────────────────────
        let gate_bind = gl_defined_names_framed(gl, true, crate::codec::gl_offset_framed);
        match &gate_bind {
            Err(s) => out.push(stop_cause(*s)),
            Ok((bound, _)) => {
                if bound.len() != segs.len() {
                    out.push(cause::BIND_COUNT);
                } else if bound
                    .iter()
                    .zip(&starts)
                    .any(|(&(off, _), &s)| off as usize != s)
                {
                    out.push(cause::BIND_OFFSET);
                }
            }
        }

        // ── AB-g: does dropping the type-index window alone bind this TU? ──
        //
        // Asked as a *counterfactual on the same TU*, not as a property of the
        // record count: a TU can have records the gate cannot see and still
        // fail to bind for four other reasons, and only this comparison
        // separates the two.
        out.window_blocks_binding = !binds_under(
            gl,
            segs.len(),
            &starts,
            crate::codec::gl_offset_framed,
        ) && binds_under(gl, segs.len(), &starts, emit_offset_framed);

        // ── the real binding, and the varargs gate that sits behind it ────
        //
        // Taken here, ahead of the body loop, so `first` is the cause
        // `IlBundle::functions` would stop on: there the varargs test runs
        // *before* each segment's `parse_segment`. (It fires on 0 of the 878
        // workload TUs, so the ordering is a correctness property of this
        // instrument rather than a number in any histogram.)
        let real = Bindings::per_record(gl, self.get("in").unwrap_or(&[]), self.get("sy"), &segs, &starts);
        if let Some(b) = &real {
            if (0..segs.len()).any(|i| b.is_varargs(i)) {
                out.push(cause::VARARGS);
            }
        }

        // ── per-body decode, evaluated whatever the binding did ───────────
        //
        // `Bindings::positional` is used ONLY for its locals view, which is
        // `SyLocals::new(sy, segs)` — a function of `.sy` and the segment list
        // and of nothing the naming decides. Its `names` are never read here.
        let probe = Bindings::positional(gl, self.get("in").unwrap_or(&[]), self.get("sy"), &segs);
        let mut shapes = Vec::with_capacity(segs.len());
        for (i, seg) in segs.iter().enumerate() {
            match parse_segment(seg, probe.locals(i)) {
                Some(s) => shapes.push(Some(s)),
                None => {
                    out.bodies_out_of_class += 1;
                    shapes.push(None);
                }
            }
        }
        if out.bodies_out_of_class > 0 {
            out.push(cause::BODY_DECODE);
        }

        // ── everything downstream needs real names ────────────────────────
        let bind = match real {
            Some(b) => b,
            None => return out,
        };
        out.downstream_evaluated = true;

        let names = bind.names();
        let src = bind.src.clone();
        let resolve = |tok: u32| -> Option<String> { bind.resolve(tok) };
        let resolve_data = |tok: u32| -> Option<String> { bind.resolve_data(tok) };
        let resolve_data_def =
            |tok: u32| -> Option<crate::func::IlDataDef> { bind.resolve_data_def(tok) };
        // **W-WORDWRAP** — the `.bss` sibling, built beside the `.data` one and
        // from the same `Bindings`, so the two answer about one `.gl`.
        let resolve_bss_def =
            |tok: u32| -> Option<crate::func::IlDataDef> { bind.resolve_bss_def(tok) };
        let mut funcs = Vec::with_capacity(segs.len());
        for (i, shape) in shapes.into_iter().enumerate() {
            let Some(shape) = shape else { continue };
            match shape_to_function(
                shape,
                &bind.name_for_shape(i),
                &src,
                &resolve,
                &resolve_data,
                &resolve_data_def,
                &resolve_bss_def,
            ) {
                Some(f) => funcs.push(f),
                None => out.push(cause::SHAPE_RESOLVE),
            }
        }

        // The label gates and the two accounting gates are stated over the
        // functions that DID build. On a TU where some body is out of class
        // that is a partial view, and it is reported as a cause only when it
        // fires — a gate that cannot fire on a partial function list is not
        // evidence that it would not fire on the whole one.
        if funcs.iter().any(|f| f.is_framed()) {
            for f in &funcs {
                if f.is_framed() {
                    continue;
                }
                if f.label_slots(false) != Some(f.label_lead() + 1) {
                    out.push(cause::LABEL_STRIDE);
                }
            }
            if label_counter(gl).is_none() {
                out.push(cause::LABEL_COUNTER);
            }
        }

        let mut accounted: Vec<&str> = names.iter().map(String::as_str).collect();
        for f in &funcs {
            for c in f.callees() {
                accounted.push(c);
            }
            for c in &f.eh_unwind_callees {
                accounted.push(c.as_str());
            }
            for d in &f.data_syms {
                accounted.push(d.as_str());
            }
        }
        if bind
            .unclaimed
            .iter()
            .any(|n| !accounted.contains(&n.as_str()))
        {
            out.push(cause::UNCLAIMED);
        }
        // **W-INLFENCE** — the same predicate `IlBundle::functions` and the
        // census ask, so this diagnostic cannot drift from the gate it
        // re-states. `names` is a `per_record` binding, total by construction.
        let defined: std::collections::BTreeSet<String> = names.iter().cloned().collect();
        if funcs
            .iter()
            .any(|f| super::bind::callee_defined_here(f, &defined).is_some())
        {
            out.push(cause::LOCAL_CALLEE);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The anti-drift invariant**, stated on the shapes this crate can build
    /// without a toolchain. A bundle with neither file decodes nothing and must
    /// say so under both readers.
    #[test]
    fn an_empty_bundle_names_both_missing_files_and_agrees_with_the_gate() {
        let b = IlBundle::default();
        let d = b.decode_causes();
        assert_eq!(d.causes, vec![cause::NO_GL, cause::NO_EX]);
        assert_eq!(d.first, Some(cause::NO_GL));
        assert!(!d.decodes);
        assert_eq!(d.causes.is_empty(), d.decodes);
    }

    /// **The cause strings are an INTERFACE, so they must be pairwise
    /// distinct** — lane `w-vec`, board **#2504**.
    ///
    /// The module docs say the set is *"appended to, never renamed"*, and lane
    /// `w-vec` made it a **scan field** (`TuResult::gate_cause`) and a printed
    /// histogram over the whole `vocab-gap` bucket. Two constants sharing a
    /// value would silently merge two rows of that histogram into one, and the
    /// merged row would still look like a well-formed measurement — this
    /// project's absence-reads-as-success shape, applied to a ranking a lane
    /// picks its next repair off.
    ///
    /// Transcribed by hand on purpose: a test that enumerated the constants
    /// reflectively would pass by construction. Adding a cause means adding it
    /// here, which is the same discipline `PORT_WRITER_SECTIONS` carries.
    #[test]
    fn every_decode_cause_string_is_distinct() {
        let all = [
            cause::NO_GL,
            cause::NO_EX,
            cause::DRECTVE,
            cause::SPLIT_EMPTY_WITH_LO,
            cause::GL_NAME_TOO_FAR,
            cause::GL_NAME_NOT_MANGLED,
            cause::GL_RUN_ENDS_26,
            cause::GL_DLLEXPORT,
            cause::GL_26_INTRODUCED,
            cause::BIND_COUNT,
            cause::BIND_OFFSET,
            cause::VARARGS,
            cause::BODY_DECODE,
            cause::SHAPE_RESOLVE,
            cause::LABEL_STRIDE,
            cause::LABEL_COUNTER,
            cause::UNCLAIMED,
            cause::LOCAL_CALLEE,
        ];
        let set: std::collections::BTreeSet<&str> = all.iter().copied().collect();
        assert_eq!(
            set.len(),
            all.len(),
            "two decode causes share a string; a histogram keyed on them would \
             merge their rows: {all:?}"
        );
        assert!(
            all.iter().all(|c| !c.is_empty() && !c.contains(',')),
            "a cause string is empty or carries the `gate_causes` list separator: {all:?}"
        );
    }

    /// **The first cause is always one of the causes.** `DecodeCauses::push` is
    /// the only writer of both fields and sets `first` before deduplicating into
    /// `causes`; a future edit that reorders those two lines would produce a
    /// `first` that names a row the list does not contain, and
    /// `TuResult::gate_cause` / `gate_causes` are published side by side.
    #[test]
    fn the_first_cause_is_always_a_member_of_the_cause_list() {
        let mut d = DecodeCauses::default();
        assert!(d.first.is_none() && d.causes.is_empty());
        for c in [cause::BIND_COUNT, cause::BODY_DECODE, cause::BIND_COUNT] {
            d.push(c);
            let first = d.first.expect("a pushed cause must set `first`");
            assert!(
                d.causes.contains(&first),
                "`first` ({first}) is not in `causes` ({:?})",
                d.causes
            );
        }
        // …and it stays the FIRST one, not the last.
        assert_eq!(d.first, Some(cause::BIND_COUNT));
        // …and a repeat does not duplicate the row.
        assert_eq!(d.causes, vec![cause::BIND_COUNT, cause::BODY_DECODE]);
    }

    /// A `.gl` with no framed record at all is seen as empty by BOTH framings —
    /// the control that says `records_wide` is not simply always larger.
    #[test]
    fn both_framings_see_zero_records_in_a_gl_with_none() {
        let gl = b"\x00?f@@YAHH@Z\x00\x86\x01\x05\x04".to_vec();
        assert_eq!(framed_record_count(&gl, crate::codec::gl_offset_framed), 0);
        assert_eq!(framed_record_count(&gl, emit_offset_framed), 0);
    }

    /// **The window, as a unit fact.** One synthetic record, framed
    /// `80 <LE32 v> 00 00 80 <LE32 off>`: the wide framing sees it for every
    /// `v < 0x10000`, the gate's only for `v` in `0x1000..=0x10FF`. This is the
    /// byte-level statement of board AB-g and it fails under any relaxation of
    /// `codec::gl_offset_framed`'s `gl[o-5]` clause.
    #[test]
    fn the_gate_framing_sees_only_the_low_256_type_indices() {
        let rec = |v: u32| -> Vec<u8> {
            let mut g = vec![0x80];
            g.extend_from_slice(&v.to_le_bytes());
            g.extend_from_slice(&[0x00, 0x00]);
            g.push(0x80);
            g.extend_from_slice(&7u32.to_le_bytes());
            g
        };
        for (v, gate) in [
            (0x0FFFu32, false),
            (0x1000, true),
            (0x10FF, true),
            (0x1100, false),
            (0x19A1, false),
            (0xA4F6, false),
        ] {
            let g = rec(v);
            assert_eq!(
                framed_record_count(&g, crate::codec::gl_offset_framed),
                gate as usize,
                "gate framing at v={v:#06x}"
            );
            assert_eq!(
                framed_record_count(&g, emit_offset_framed),
                1,
                "wide framing at v={v:#06x}"
            );
        }
    }
}
