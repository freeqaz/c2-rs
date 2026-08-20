//! **IR0 — the lossless framing of an IL bundle.**
//!
//! `docs/ARCHITECTURE_PROPOSAL_2026-08-20.md` §3.1 / §5 step 1, built by lane
//! `ir0` (`docs/rungs/2026-08-20-ir0.md`).
//!
//! # What this layer is
//!
//! **IR0 FRAMES. It never decodes and it never admits.** A [`Record`] is a
//! half-open byte range plus the structural marker that OPENED it. A record
//! kind is *never* a claim about what the bytes mean, and no method here
//! returns a verdict about a function, a name, a symbol or a class.
//!
//! **Refusal at this layer means MALFORMED INPUT ONLY — and today there is no
//! such class.** [`Ir0::frame`] and [`Ir0::frame_ex`] are infallible by
//! signature: they return the framing, never a `Result`. Bytes that no marker
//! opens are [`RecordKind::Opaque`], which is a *description of the input*, not
//! a refusal by the reader. This is the property the proposal asks for in
//! deliverable 4, and the way to hold it is to have no refusal predicate here
//! at all rather than a lenient one.
//!
//! [`Ir0Framing::verify`] exists for tests and instruments. It is **never on
//! the emit path**.
//!
//! # The invariant, stated honestly
//!
//! The brief states it as two things — *totality* and *byte-identical
//! re-serialization*. For a framing whose records are **extents** and whose
//! re-serialization is *concatenate `bytes[extent]`*, the second is **implied
//! by** the first: if the extents tile `[0, len)` exactly, the concatenation
//! **is** the input. So this is **one invariant computed two ways**, and
//! [`Ir0Framing::verify`] computes both:
//!
//! * **I1, an INDEX claim** — `records[0].extent.start == 0`;
//!   `records[i].extent.end == records[i + 1].extent.start`;
//!   `records.last().extent.end == bytes.len()`; every extent non-empty.
//! * **I2, a BYTE claim** — the concatenation of `bytes[r.extent]` equals
//!   `bytes`, compared byte for byte.
//!
//! Computing both and diffing them is the **#3288 second derivation**, which is
//! the discipline that has caught a wrong figure in every lane that ran it. It
//! is deliberately *not* written up as two independent checks.
//!
//! The genuinely independent lossless invariant is a different one and it lives
//! in [`crate::codec`]: `IlModel::parse` frames `.ex` into **typed tokens**,
//! re-encodes, and fails closed with `CodecError::CannotRoundTrip`. That is a
//! strictly stronger claim than IR0's, because a wrong token boundary breaks it
//! and cannot break a framing built from extents alone.
//!
//! # Scope — what IR0 v1 deliberately does NOT frame
//!
//! `.gl`, `.sy`, `.in` and `.db` are **one [`RecordKind::Opaque`] record each**.
//! `.gl` record framing is IR1 (proposal §5 step 4) and is genuinely hard:
//! three walkers disagree today (`func/gl.rs`), and `codec::parse_gl`'s 1:1
//! gate is FALSE on 811 of 878 workload TUs. Framing it here would move
//! counts, which is what a construct rung must not do.
//!
//! # IR0 does NOT unify the two splitters
//!
//! The gate framing (every `4F 1F`) and the census framing (`LO`-anchored, plus
//! the strictly-additive bare-`4C` pass) stay **two views**. They are pinned
//! apart on purpose (`func/bind.rs`) and their disagreement is measured — 185
//! agree and 685 disagree over the 870 graded workload TUs, every one of the
//! 685 in the direction of the gate seeing more.
//!
//! **IR0 gives them one INPUT, not one ANSWER.** Making them agree would move
//! the census numerator that everything else is differenced against, and the
//! recorded instance of exactly that substitution moved 865 TUs from
//! `vocab-gap` to `codegen-gap` while `mismatch 0 / match 6 / 0 failed` all read
//! green (`c2-harness`'s `splitter_predicate_guard`). If the two views ever
//! agree on ≥ 850 of 878 TUs, that is the defect and not a result.
//!
//! # NOTHING IN `crates/` PRODUCTION CODE CALLS THIS YET, AND THAT IS MEASURED
//!
//! Lane `ir0` was commissioned to *"re-express `IlBundle::functions()` and the
//! census splitters as views over IR0"*. **It did, measured the price, and
//! reverted.** The switch is a real branch in the history, not a thing nobody
//! tried; read this before re-attempting it.
//!
//! All eleven production call sites were switched (seven `split_functions_at`
//! sites in `func/bundle.rs`, plus `func/diag.rs`, `func/ehscope.rs`, and both
//! `split_function_bodies_at` sites in `func/census.rs`). It compiles, and by
//! construction it cannot change a byte — the three-way differential in
//! `c2-harness/tests/ir0_framing.rs` proves the views identical to the
//! incumbents on 386 fixtures, and `ir0-splitter-crosscheck-broken` proves it
//! on all 870 workload TUs on every scan.
//!
//! **It costs about 2 % of the port's throughput.**
//!
//! | pair | port time per obj | 95 % CI | slower on |
//! |---|---|---|---|
//! | **byte-identical NULL arm** | **+0.10 %** | **[−0.22, +0.42]** | **76 of 153** |
//! | **via IR0 vs incumbent** | **+2.03 %** | [+1.72, +2.34] | 132 of 153 |
//! | via IR0 with `records` suppressed | +4.27 % | [+3.85, +4.69] | 147 of 153 |
//!
//! 20 rounds / 68 `c2rs perf` runs over the 153 fixtures the port matches,
//! order rotated so no arm keeps a slot, estimator = per-fixture minimum over
//! rounds (external load can only add time), paired per fixture. Box load
//! ranged 19–120 on 32 cores throughout, which is why the **null arm** — a
//! `cp` of the incumbent binary, verified with `cmp`, so an effect of exactly
//! zero — is the load-bearing row: it reads +0.10 % and splits the sign 76/153,
//! which is what fixes the floor at ±0.4 % and makes +2.03 % a number.
//!
//! **THE FIRST READING OF THIS WAS "8–14 %", AND IT WAS THE NOISE.** Five
//! paired trials of the round-level geomean *speedup* gave −4.4/−13.5/−5.7/
//! −14.2/−8.8 %. Re-run 12 times, the same protocol reads **−13.68 %
//! [−32.55, +10.47] on the REFERENCE side — the same `c2.dll` in both arms, an
//! effect of zero by construction.** Never divide by a `wibo` subprocess to
//! measure an in-process change, and never publish a difference without an arm
//! that cannot differ.
//!
//! **The cause is NOT isolated, and the earlier claim that it was is
//! withdrawn.** *"Suppress the `Vec<Record>` and the cost disappears"* rested
//! on two trials; measured properly, the records-suppressed build is **+2.19 %
//! slower than the switched one**, the opposite sign. The residual ~2 % is not
//! attributed to any component. What is known is the scale: ~12.6 records per
//! fixture bundle × 24 B ≈ 300 B built and dropped per call, at seven
//! `bundle.rs` sites, several on `PortC2::build`'s path — which is the right
//! order of magnitude for a low single-digit percentage and not for 14 %.
//!
//! **The switch itself is in NO COMMIT on this branch** — it survives only as
//! three uncommitted binaries under gitignored `work/ir0/`. Re-doing it and
//! committing it *before* measuring is a prerequisite for the next attempt;
//! the eleven call sites are enumerated in the `ir0 S4` commit message.
//!
//! The conclusion is about the migration order, not about IR0: **the readers
//! should be re-expressed as views when the records become load-bearing —
//! i.e. folded into IR1 (`ARCHITECTURE_PROPOSAL_2026-08-20.md` §5 step 4),
//! where a record carries a binding — and not landed alone as step 1.** Until
//! then IR0's value is the layer it already is: a totality control, an opaque
//! denominator about the input, and a fence that makes a silent unification of
//! the two splitters impossible.

mod ex;

pub use ex::{FileFraming, Ir0, Ir0Broken, Ir0Framing};

/// A half-open byte range into **one** bundle file. Never owns bytes.
///
/// Owning bytes here would make IR0 a copy of the bundle per call site;
/// `codec::Span::Opaque(Vec<u8>)` owns them because it is the K3a *edit*
/// substrate, and that is the one thing IR0 must not become.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Extent {
    /// Inclusive start, relative to the start of the file.
    pub start: usize,
    /// Exclusive end, relative to the start of the file.
    pub end: usize,
}

impl Extent {
    /// Byte length. Never zero in a well-formed framing (I1 forbids it).
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Whether the extent covers no bytes. A well-formed framing has none.
    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }
}

/// What structural marker OPENED this record.
///
/// **A record kind carries no semantics.** `ExFnSegment` says *"a `4F 1F`
/// opened here"*, which is a fact about two bytes; it is not a claim that the
/// span is a function, that it is well formed, or that any reader can decode
/// it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordKind {
    /// Bytes belonging to no marker this layer knows: the `.ex` header/index
    /// region before the first `4F 1F`, a whole `.ex` with no `4F 1F` in it at
    /// all, or a whole `.gl`/`.sy`/`.in`/`.db`.
    ///
    /// **This is NOT a refusal.** It is the honest name for input this layer
    /// does not frame further, and the reason `ir0-bytes-opaque` is a number
    /// about the INPUT rather than about the reader. Both incumbent splitters
    /// cover only `[first marker, len)` and simply never see the head; owning
    /// it as an explicit record is what turns a *scan* into a *framing*.
    Opaque,
    /// A `4F 1F`-introduced `.ex` function segment, running to the next `4F 1F`
    /// or to end of file — byte-for-byte the segmentation
    /// `func::bundle::split_functions_at` produces and `PortC2::build` consumes.
    ExFnSegment,
}

/// One framed span of one file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    /// Where it is. Relative to the start of its file.
    pub extent: Extent,
    /// What opened it.
    pub kind: RecordKind,
}
