//! The displacement range check and **§3.3.1's long-branch expansion** —
//! `docs/CFG_SHAPE.md` §6.2 item **D**.
//!
//! > **D. A displacement range check with a defined expansion.** `bc` reaches
//! > ±32764. Past that, invert the condition and branch over an unconditional
//! > `b` (§3.3.1). This must be in the fixup pass, because the overflow is only
//! > visible after layout.
//!
//! # Read the title and the sentence separately — item G's rule, applied
//!
//! Lane `w-ir-g` found that item **G**'s *sentence* ("band 3 or refuse") would
//! have refused a shipped byte-exact class, and that the item's *title* ("per
//! accepted shape") was the half that bound. Item D splits the same way and the
//! split is the whole reason this module has the shape it has.
//!
//! **The title is what ships here.** *"A displacement range check with a
//! **defined expansion**"*. The check has existed since W11 — three times over,
//! in fact (see below). What did not exist anywhere in this crate is the
//! expansion: not its bytes, not its arithmetic, not its preconditions.
//! **Defined** is the word, and it is the word this module is graded on, against
//! §3.3.1's four measured rows.
//!
//! **The sentence's *because* is right and its *must* is false.** *"the overflow
//! is only visible after layout"* — yes: nothing can know a displacement until
//! every block has a position. *"This must be in the fixup pass"* — no, and the
//! fixup pass's own signature is the witness:
//!
//! ```text
//!     pub fn resolve(self, text: &mut [u8]) -> Result<(), BackendError>
//!                                 ^^^^^^^^^
//! ```
//!
//! [`LabelMap::resolve`] takes a **slice**. A slice cannot grow, and the
//! expansion **inserts a word**. That is not an accident of API design that a
//! `&mut Vec<u8>` would fix: inserting a word at offset `X` moves every byte
//! after it by four, which invalidates **every already-bound label offset** and
//! **every other pending site's own offset** in the same map — including sites
//! already patched on this pass — and can push a second branch that was in range
//! out of it. A patch is a write of four bytes at a known offset; the expansion
//! is a **re-layout**, and re-layout has to run to a fixpoint (it is monotone —
//! expansions only ever grow the text — so the fixpoint exists, which is the one
//! easy part). Detection belongs after layout, exactly as the sentence says. The
//! rewrite belongs *before* the fixup pass, not in it.
//!
//! [`LabelMap::resolve`]: super::labels::LabelMap::resolve
//!
//! # The check was spelled twenty-four times and the expansion zero
//!
//! Before this module, `codegen/` refused an out-of-range `bc`/`b` displacement
//! in **twenty-four** places: [`super::labels::LabelMap::resolve`]'s invariant
//! 5, and **twenty-three** private
//! `ok_or_else(|| out_of_class("… out of range"))` sites spread over **thirteen**
//! lowerings, each with its own wording — *"outside its displacement field"*,
//! *"out of range"*, *"past the `bc` field"*, *"does not fit a `b`
//! displacement"*, *"if/else-with-a-join: join displacement"*. Exactly **one**
//! of the twenty-four, the map's, mentions §3.3.1 at all, and it mentions it to
//! say the expansion is *not built*.
//!
//! Twenty-four encodings of one fact is the shape `docs/GAPS.md` §6 keeps
//! recording, and it is why item D reads as half-built: every site knew the
//! branch was too far and **no site knew what c2 does about it**. This module is
//! the one answer, and [`direct`] is the gate the twenty-three call.
//!
//! **Two sites are deliberately left out and neither is an oversight.**
//! `labels.rs`'s is peer-held and its invariant 5 is item B's, not item D's — it
//! should delegate here and that is a one-line change belonging to its owner.
//! And the two `bdnz` back edges (`xtea_round_loop`, `pool_ctor_chain`) go
//! through `encode_bdnz`, a **different** encoder at `BO_DNZ`: [`Form`] has no
//! variant for them, and this model already says why — a `BO` that tests CTR
//! rather than a condition register is [`Unmeasured::NoSenseToInvert`], so
//! §3.3.1's expansion has nothing to invert on one. Adding a `Form::Bdnz` would
//! be widening a peer-held type to carry a case the expansion cannot serve.
//!
//! # What it derives rather than restates
//!
//! * **The threshold.** `Reach::of` asks [`encode_bc`]/[`encode_b_intra`]
//!   whether the displacement fits and reads the answer. `BC_MAX_DISP` and
//!   `B_MAX_DISP` are **not** spelled here — they are `encode`'s, and
//!   `BC_MAX_DISP`'s own doc already carries §3.3.1's bracket (+32628 direct,
//!   +34148 expanded, *"the full field before expanding"*).
//! * **The form.** [`Form`] is [`super::labels`]'s, item **C**, board **#191** —
//!   *the discriminator is the target, not the opcode*. A second enum for "which
//!   of the two intra-section encodings" is the corruption #191 names, so there
//!   is not one.
//! * **The inversion.** [`invert_sense`] is [`super::cond`]'s, item **E**, and it
//!   refuses precisely what `BO_IGNORES_CR` covers.
//!
//! # What it does NOT do
//!
//! **It performs no expansion.** [`Reach::Expanded`] hands back the two words
//! and says so; nothing in this crate rewrites a body around them, because
//! nothing in the corpus is 32 KB and a relaxation pass with no body to relax is
//! an ungraded code path by construction — w-frame row **F-c**, the mistake
//! `encode_b_intra`'s own header records being made and reverted once already.
//! The bytes are *defined and graded*; applying them is the day a 32 KB body
//! arrives, and that day the sentence above says where the pass goes.
//!
//! It emits nothing on any path a shipped lowering takes: every word it returns
//! comes out of an encoder in [`super::encode`] that a byte-graded class already
//! produces, and [`direct`] returns exactly what its call site's `encode_bc` /
//! `encode_b_intra` returned before. That equality is this lane's success
//! criterion, not a side effect of it.

use super::cond::invert_sense;
use super::encode::{encode_b_intra, encode_bc};
use super::labels::Form;
use super::select::out_of_class;
use crate::BackendError;

/// **§3.3.1's expansion, as the two words it is.**
///
/// ```text
///   N = 4400, target +34148 bytes away
///     419a0008   beq cr6,+8      <- the INVERTED condition, over the `b`
///     48008564   b   +34148      <- the far target, on the wide LI field
/// ```
///
/// Two instructions, and **never** a register-indirect `bcctr` form — §3.3.1
/// swept N to 6000 and c2 emits this pair every time. [`Self::bytes`] is the
/// pair in emission order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LongBranch {
    over: [u8; 4],
    far: [u8; 4],
}

/// How far the inverted `bc` jumps: **past the `b` this pair inserts**.
///
/// Two words. The `bc` sits at `X`, the `b` at `X + 4`, and the instruction that
/// followed the original branch is at `X + 8` — which is the `0008` in
/// §3.3.1's measured `419a0008`, at both N = 4400 and N = 6000.
const OVER_THE_B: i32 = 8;

impl LongBranch {
    /// Build the pair for a `bc BO,BI` whose target is `disp` bytes away, where
    /// `disp` is the displacement the branch would have needed **before** the
    /// expansion inserted its word.
    ///
    /// `None` if the sense cannot be inverted ([`invert_sense`]) or if the far
    /// target is past even the `b`'s 24-bit `LI` field.
    ///
    /// # The `b` carries `disp`, not `disp − 4`, and that is a measured claim
    ///
    /// The pair replaces one word with two, so everything at or after the
    /// original branch's successor moves by four — **including the target**, as
    /// long as the target is *forward*. Writing `T` for the pre-expansion target
    /// offset and `X` for the branch's:
    ///
    /// ```text
    ///   forward:   T' = T + 4,   LI = T' − (X + 4) = T − X = disp
    ///   backward:  T' = T,       LI = T  − (X + 4) = disp − 4
    /// ```
    ///
    /// The forward line is what §3.3.1's bytes say: the table's *"displacement
    /// needed"* column reads +34148 and +46708, and the emitted `b` words are
    /// `48008564` and `4800b674` — `LI` of exactly 34148 and 46708. The backward
    /// line is **arithmetic, not measurement**: §3.3.1's probe is
    /// `if(a==0){ … } return b+1;`, forward on every row of the sweep. So
    /// [`Reach::of`] refuses a backward overflow as [`Reach::Unmeasured`] rather
    /// than applying the second line, which differs from the first by exactly
    /// the one word this pair inserts and would be silently wrong if the reading
    /// above were the wrong way round.
    pub fn new(bo: u8, bi: u8, disp: i32) -> Option<Self> {
        Some(LongBranch {
            over: encode_bc(invert_sense(bo)?, bi, OVER_THE_B)?,
            far: encode_b_intra(disp)?,
        })
    }

    /// The inverted conditional branch, over the `b`.
    pub fn over(self) -> [u8; 4] {
        self.over
    }

    /// The unconditional branch to the far target.
    pub fn far(self) -> [u8; 4] {
        self.far
    }

    /// Both words, in emission order.
    pub fn bytes(self) -> [u8; 8] {
        let mut out = [0u8; 8];
        out[..4].copy_from_slice(&self.over);
        out[4..].copy_from_slice(&self.far);
        out
    }
}

/// Why a site past its field has no defined answer. Diagnostic, and each variant
/// is a case §3.3.1 did **not** sweep rather than a case it swept and this
/// module declined.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unmeasured {
    /// A **backward** `bc` past its field. The inserted word does not move a
    /// target that is already behind it, so the `b` would carry `disp − 4` and
    /// not `disp` — see [`LongBranch::new`]. §3.3.1's probe is forward on every
    /// row.
    Backward,
    /// An unconditional `b` past its own 24-bit `LI` field, i.e. past ±32 MB.
    ///
    /// There is nothing to expand *into*: §3.3.1's expansion **is** a `b`, so a
    /// `b` that does not reach would need a form — a `bcctr` through a loaded
    /// address — that §3.3.1 explicitly did not observe (*"never a
    /// register-indirect `bcctr` form"*) at any N it swept.
    BOutOfReach,
    /// A `bc` past its field whose `BO` has no sense to invert — `BO_ALWAYS`,
    /// `BO_DNZ`. See [`invert_sense`].
    NoSenseToInvert,
}

impl Unmeasured {
    /// What it is, for a refusal that has to name it.
    pub fn what(self) -> &'static str {
        match self {
            Unmeasured::Backward => {
                "a BACKWARD branch past its displacement field: CFG_SHAPE.md \
                 §3.3.1 swept forward only, and the arithmetic differs — the word \
                 the expansion inserts does not move a target that is already \
                 behind it, so the `b` would carry `disp - 4` and not `disp`"
            }
            Unmeasured::BOutOfReach => {
                "an unconditional `b` past its own 24-bit LI field: §3.3.1's \
                 expansion IS a `b`, so there is nothing left to expand into, and \
                 the register-indirect `bcctr` form is the one thing §3.3.1 says \
                 c2 never emits"
            }
            Unmeasured::NoSenseToInvert => {
                "a branch past its displacement field whose BO does not test a \
                 condition register (BO_ALWAYS, BO_DNZ), so §3.3.1's \"invert the \
                 condition\" has nothing to inv"
            }
        }
    }
}

/// **What one reference site's displacement is** — the verdict item D's range
/// check reduces to once the expansion exists to be an alternative.
///
/// Four-valued, and the four are not interchangeable. §3.3.1 measured exactly
/// one of them ([`Expanded`](Self::Expanded)) and the discipline this crate
/// applies to [`super::cond::CrEffect`] and [`super::fold::BandVerdict`] applies
/// here for the same reason: *"I have no measurement for this"* must never be
/// read as *"this is the measured case"*, because the measured case emits bytes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reach {
    /// The displacement **fits the form's own field**. One word, and it is
    /// exactly what the encoder gives — this module computes no encoding of its
    /// own.
    ///
    /// This is the verdict every site in this crate gets today, on every body it
    /// emits, and keeping it that way is what makes the lane that built this
    /// module a zero-byte-delta lane.
    Direct([u8; 4]),
    /// Past the `bc` field, forward, on an invertible sense: **§3.3.1's measured
    /// two-word expansion**.
    ///
    /// Handed back, never applied — see the module header on why a re-layout is
    /// not a patch.
    Expanded(LongBranch),
    /// The displacement is not a whole number of words.
    ///
    /// **A distinct answer from "too far", and the clause order is why.** Both
    /// make `encode_bc` return `None`, and reading that `None` as "too far"
    /// would hand a misaligned site an expansion whose `b` is misaligned in
    /// exactly the same way — a second wrong branch instead of one. A misaligned
    /// displacement is a lowering defect upstream of anything §3.3.1 measures.
    Misaligned,
    /// Past the field, in a case §3.3.1 did not sweep. Carries which.
    Unmeasured(Unmeasured),
}

/// Encode one reference in its form. The four-line dispatch [`Form`] keeps
/// private, and the only reason it is written twice.
///
/// The *facts* are not duplicated — `BC_MAX_DISP` and `B_MAX_DISP` live in
/// [`super::encode`] and both spellings read them through the same two encoders.
/// Making `Form::encode` visible to this module and deleting this function is a
/// one-line change in `labels.rs`; it is not taken here because that file is
/// held by another lane.
fn encode_in_form(form: Form, disp: i32) -> Option<[u8; 4]> {
    match form {
        Form::Bc { bo, bi } => encode_bc(bo, bi, disp),
        Form::B => encode_b_intra(disp),
    }
}

impl Reach {
    /// **The decision**, in §3.3.1's own clause order.
    ///
    /// `disp` is the **pre-expansion** self-relative displacement,
    /// `target_offset − branch_offset` (§3.3) — the number the branch would
    /// carry if it fitted. That contract matters for exactly one of the four
    /// verdicts and is spelled out in [`LongBranch::new`].
    ///
    /// The clause order is load-bearing:
    ///
    /// 1. **Misaligned first**, before anything asks about range — see
    ///    [`Reach::Misaligned`].
    /// 2. **In range → [`Direct`](Self::Direct)**, derived by asking the encoder
    ///    rather than by comparing against a threshold spelled here. c2 uses the
    ///    full field before expanding (§3.3.1: the transition sits at the
    ///    architectural limit *with no slack*), so "fits" and "direct" are the
    ///    same question and there is no margin to get wrong.
    /// 3. **Out of range** splits three ways before it reaches the expansion,
    ///    and every one of the three is a case §3.3.1 did not sweep.
    pub fn of(form: Form, disp: i32) -> Reach {
        if disp % 4 != 0 {
            return Reach::Misaligned;
        }
        if let Some(w) = encode_in_form(form, disp) {
            return Reach::Direct(w);
        }
        let (bo, bi) = match form {
            Form::B => return Reach::Unmeasured(Unmeasured::BOutOfReach),
            Form::Bc { bo, bi } => (bo, bi),
        };
        if disp < 0 {
            return Reach::Unmeasured(Unmeasured::Backward);
        }
        match LongBranch::new(bo, bi, disp) {
            Some(lb) => Reach::Expanded(lb),
            // `invert_sense` is the only thing left that can refuse: `disp` is
            // aligned, positive, and past the 14-bit BD field but nowhere near
            // the 24-bit LI one — a body 32 MB long is not a shape this crate
            // can produce, and if it ever were, `encode_b_intra` would have said
            // so through `Form::B` above.
            None => Reach::Unmeasured(Unmeasured::NoSenseToInvert),
        }
    }

    /// What this verdict is, for a refusal that has to name it.
    pub fn what(self) -> &'static str {
        match self {
            Reach::Direct(_) => "in range",
            Reach::Expanded(_) => {
                "past its displacement field, where CFG_SHAPE.md §3.3.1's \
                 long-branch expansion applies — invert the condition, branch \
                 over an unconditional `b`, put the far target on the `b`. The \
                 two words are DEFINED (codegen::reach::LongBranch) and are NOT \
                 APPLIED here: inserting a word is a re-layout and not a patch, \
                 so it cannot happen at a fixed site"
            }
            Reach::Misaligned => {
                "a displacement that is not a whole number of PowerPC words, \
                 which is a lowering defect upstream of any range question — \
                 expanding it would produce a second misaligned branch"
            }
            Reach::Unmeasured(u) => u.what(),
        }
    }
}

/// **The gate every shipped emitter calls**: the branch word if and only if the
/// site is in range, and otherwise a refusal that names the site, the verdict
/// and §3.3.1.
///
/// This is item D's check, with — for the first time — an *answer* behind the
/// refusal rather than twenty different spellings of "no". It returns byte-for-
/// byte what the call site's own `encode_bc`/`encode_b_intra` returned before,
/// which is the property that makes routing an already-byte-exact class through
/// it a re-expression and not a change.
///
/// `site` names the branch for the refusal text: a refusal that cannot say
/// *which* branch it is about is one somebody has to re-derive. It is the same
/// role `FoldShape::admit`'s `site` plays (item **G**).
///
/// [`FoldShape::admit`]: super::fold::FoldShape::admit
pub fn direct(form: Form, disp: i32, site: &str) -> Result<[u8; 4], BackendError> {
    match Reach::of(form, disp) {
        Reach::Direct(w) => Ok(w),
        other => Err(out_of_class(&format!(
            "{site}: a displacement of {disp} is {}",
            other.what()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::cond_tail::branch_sense;
    use crate::codegen::encode::{
        cr_bi, BC_MAX_DISP, BO_ALWAYS, BO_DNZ, BO_FALSE, BO_TRUE, CR_BIT_EQ, CR_COMPARE,
    };
    use c2_il::Rel;

    /// `bne cr6` — the form every row of §3.3.1's sweep branches with.
    fn bne_cr6() -> Form {
        Form::Bc { bo: BO_FALSE, bi: cr_bi(CR_COMPARE, CR_BIT_EQ) }
    }

    // ===================================================================
    //  §3.3.1's four measured rows — the whole sweep, reproduced
    // ===================================================================

    /// **The known-answer control, and it is real `c2.dll` output.**
    /// `docs/CFG_SHAPE.md` §3.3.1 swept held-out probe `pe.cpp` over N and read
    /// four branch encodings off the objs. All four come back out of this model:
    /// the two short rows as one word each, the two long rows as the pair.
    ///
    /// This is the assertion the module exists to satisfy. Nothing else in the
    /// crate has ever produced the last two.
    #[test]
    fn the_whole_of_section_3_3_1s_sweep_comes_back_out_of_the_model() {
        // N = 4000, +31176 -> direct `bne cr6`
        assert_eq!(
            Reach::of(bne_cr6(), 31176),
            Reach::Direct([0x40, 0x9a, 0x79, 0xc8])
        );
        // N = 4200, +32628 -> STILL direct. c2 uses the full field.
        assert_eq!(
            Reach::of(bne_cr6(), 32628),
            Reach::Direct([0x40, 0x9a, 0x7f, 0x74])
        );
        // N = 4400, +34148 -> the expansion.
        let Reach::Expanded(lb) = Reach::of(bne_cr6(), 34148) else {
            panic!("+34148 is past the BD field and forward: §3.3.1 expands it")
        };
        assert_eq!(lb.over(), [0x41, 0x9a, 0x00, 0x08]);
        assert_eq!(lb.far(), [0x48, 0x00, 0x85, 0x64]);
        assert_eq!(
            lb.bytes(),
            [0x41, 0x9a, 0x00, 0x08, 0x48, 0x00, 0x85, 0x64]
        );
        // N = 6000, +46708 -> the same pair, same inverted word, further `b`.
        let Reach::Expanded(lb) = Reach::of(bne_cr6(), 46708) else {
            panic!("+46708 expands too")
        };
        assert_eq!(lb.over(), [0x41, 0x9a, 0x00, 0x08]);
        assert_eq!(lb.far(), [0x48, 0x00, 0xb6, 0x74]);
    }

    /// **The `b` carries `disp`, not `disp − 4`.** Stated as its own test
    /// because it is the one number in the expansion that an implementer would
    /// naturally get wrong, and §3.3.1's own bytes decide it: the table's
    /// *"displacement needed"* column and the emitted `LI` are the **same
    /// integer** on both long rows.
    #[test]
    fn the_far_branch_carries_the_pre_expansion_displacement() {
        for disp in [34148i32, 46708] {
            let Reach::Expanded(lb) = Reach::of(bne_cr6(), disp) else {
                panic!("{disp} expands")
            };
            let li = u32::from_be_bytes(lb.far()) & 0x03FF_FFFC;
            assert_eq!(li as i32, disp, "the `b`'s LI is the displacement needed");
        }
    }

    /// **Two instructions, never a `bcctr`.** §3.3.1's closing sentence, as a
    /// check on the bytes: the pair is eight bytes, the first word is primary
    /// opcode 16 (`bc`) and the second is primary opcode 18 (`b`) — not 19,
    /// which is where `bcctr` lives.
    #[test]
    fn the_expansion_is_a_bc_and_a_b_and_never_a_register_indirect_form() {
        let Reach::Expanded(lb) = Reach::of(bne_cr6(), 34148) else {
            panic!("expands")
        };
        assert_eq!(lb.bytes().len(), 8);
        assert_eq!(u32::from_be_bytes(lb.over()) >> 26, 16, "bc");
        assert_eq!(u32::from_be_bytes(lb.far()) >> 26, 18, "b, not 19 (bcctr)");
    }

    /// **The transition is at the architectural limit with no slack** — §3.3.1
    /// brackets it between +32628 and +34148 and reads it as the full field.
    /// The boundary is therefore `BC_MAX_DISP` exactly, and it is *derived*:
    /// this test names the constant rather than the number, so a model that
    /// invented a safety margin goes red here rather than quietly emitting a
    /// pair c2 does not.
    #[test]
    fn the_last_direct_displacement_is_the_field_and_the_next_word_expands() {
        assert!(matches!(
            Reach::of(bne_cr6(), BC_MAX_DISP),
            Reach::Direct(_)
        ));
        assert!(matches!(
            Reach::of(bne_cr6(), BC_MAX_DISP + 4),
            Reach::Expanded(_)
        ));
    }

    // ===================================================================
    //  The three verdicts §3.3.1 did NOT measure
    // ===================================================================

    /// A misaligned displacement is [`Reach::Misaligned`] and **not** an
    /// overflow, even when it is also far out of range — the clause order.
    /// Expanding it would emit a second misaligned branch.
    #[test]
    fn a_misaligned_displacement_is_its_own_verdict_at_any_distance() {
        assert_eq!(Reach::of(bne_cr6(), 6), Reach::Misaligned);
        assert_eq!(Reach::of(bne_cr6(), 34146), Reach::Misaligned);
        assert_eq!(Reach::of(Form::B, -2), Reach::Misaligned);
    }

    /// A **backward** overflow is refused rather than expanded, and the refusal
    /// says why the arithmetic differs. A backward branch that *fits* is
    /// [`Reach::Direct`] — `ptr_walk_loop`'s back edge is one, and narrowing
    /// this to "forward only" would refuse a shipped byte-exact class.
    #[test]
    fn a_backward_overflow_is_unmeasured_but_a_backward_fit_is_direct() {
        assert_eq!(
            Reach::of(bne_cr6(), -40_000),
            Reach::Unmeasured(Unmeasured::Backward)
        );
        assert!(Reach::of(bne_cr6(), -40_000).what().contains("disp - 4"));
        // …and the in-range backward case, which every loop in this crate emits.
        assert!(matches!(Reach::of(bne_cr6(), -12), Reach::Direct(_)));
        assert!(matches!(Reach::of(bne_cr6(), -BC_MAX_DISP - 4), Reach::Direct(_)));
    }

    /// The same distance is fine for the wider `LI` field, which is what makes
    /// the `bc` rows a statement about the *field* and not about the length —
    /// `labels.rs` pins the same pair of facts one level up.
    #[test]
    fn the_wider_b_field_reaches_where_the_bc_field_does_not() {
        assert!(matches!(Reach::of(Form::B, 40_000), Reach::Direct(_)));
        assert!(matches!(Reach::of(bne_cr6(), 40_000), Reach::Expanded(_)));
    }

    /// Past even the `b`'s field there is no measured answer — the expansion is
    /// itself a `b`, so there is nothing left to expand into.
    #[test]
    fn a_b_past_its_own_field_has_nothing_to_expand_into() {
        assert_eq!(
            Reach::of(Form::B, 0x0200_0000),
            Reach::Unmeasured(Unmeasured::BOutOfReach)
        );
        assert!(Reach::of(Form::B, 0x0200_0000).what().contains("bcctr"));
    }

    /// A `BO` that does not test the condition register has no condition to
    /// invert. `BO_ALWAYS` and `BO_DNZ` both refuse, through `invert_sense`.
    #[test]
    fn a_bo_with_no_sense_cannot_be_expanded() {
        for bo in [BO_ALWAYS, BO_DNZ] {
            assert_eq!(
                Reach::of(Form::Bc { bo, bi: 0 }, 40_000),
                Reach::Unmeasured(Unmeasured::NoSenseToInvert)
            );
        }
        assert_eq!(LongBranch::new(BO_ALWAYS, 26, 40_000), None);
    }

    // ===================================================================
    //  The inversion, cross-checked against the existing measured table
    // ===================================================================

    /// **The inversion is graded against `cond_tail::branch_sense`'s six rows,
    /// not against itself.** `branch_sense` is this crate's existing reader of
    /// "which `(BO, bit)` pair an IL relation becomes", and its six rows are
    /// three negation pairs: `Eq`/`Ne`, `Lt`/`Ge`, `Gt`/`Le`. Inverting the
    /// sense of one row must land on its partner, on the same bit — which is
    /// what §3.3.1's expansion does when it turns `409a…` into `419a…` and
    /// leaves `BI` alone.
    #[test]
    fn inverting_the_sense_walks_branch_senses_own_negation_pairs() {
        for (a, b) in [(Rel::Eq, Rel::Ne), (Rel::Lt, Rel::Ge), (Rel::Gt, Rel::Le)] {
            let (bo_a, bit_a) = branch_sense(a);
            let (bo_b, bit_b) = branch_sense(b);
            assert_eq!(bit_a, bit_b, "a negation pair tests the same CR bit");
            assert_eq!(invert_sense(bo_a), Some(bo_b));
            assert_eq!(invert_sense(bo_b), Some(bo_a));
        }
        // …and the flip is an involution on both of §3.1's two senses.
        assert_eq!(invert_sense(BO_TRUE), Some(BO_FALSE));
        assert_eq!(invert_sense(BO_FALSE), Some(BO_TRUE));
    }

    /// The expansion leaves `BI` **untouched** — the field and the bit belong to
    /// the branch, and only the sense moves. §3.3.1's `409a…` → `419a…` is the
    /// witness: `0x9a` is unchanged across the expansion.
    #[test]
    fn the_expansion_moves_the_sense_and_nothing_else() {
        let bi = cr_bi(CR_COMPARE, CR_BIT_EQ);
        let Reach::Expanded(lb) = Reach::of(Form::Bc { bo: BO_FALSE, bi }, 34148) else {
            panic!("expands")
        };
        let over = u32::from_be_bytes(lb.over());
        assert_eq!(((over >> 21) & 0x1F) as u8, BO_TRUE, "the sense inverted");
        assert_eq!(((over >> 16) & 0x1F) as u8, bi, "BI is the branch's own");
    }

    // ===================================================================
    //  The gate the twenty shipped sites call
    // ===================================================================

    /// **`?MemFree`'s branch, from `CFG_SHAPE.md` §4.1's published bytes.**
    /// The real obj carries `409a0010` at 0x08 — `bne cr6` to the else block
    /// sixteen bytes on. [`direct`] must return that word and nothing else: it
    /// is the word `cond_tail` ships and the word every in-range site in this
    /// crate is an instance of.
    #[test]
    fn the_gate_returns_memfrees_own_branch_word() {
        let w = direct(bne_cr6(), 16, "?MemFree's guard").unwrap();
        assert_eq!(w, [0x40, 0x9a, 0x00, 0x10]);
    }

    /// The gate refuses everything that is not [`Reach::Direct`], and the
    /// refusal names the site, the displacement and §3.3.1's expansion — which
    /// is the information twenty private `"out of range"` strings did not carry.
    #[test]
    fn the_gate_refuses_an_overflow_and_names_the_site_and_the_expansion() {
        let err = direct(bne_cr6(), 34148, "the join branch").unwrap_err();
        let s = format!("{err:?}");
        assert!(s.contains("the join branch"), "{s}");
        assert!(s.contains("34148"), "{s}");
        assert!(s.contains("3.3.1"), "{s}");
        assert!(s.contains("re-layout"), "{s}");
        // …and a misaligned site is refused with its own reason, not this one.
        let err = direct(bne_cr6(), 6, "the join branch").unwrap_err();
        assert!(format!("{err:?}").contains("whole number of PowerPC words"));
    }

    /// **The gate is byte-for-byte the encoder, on every in-range site.** This
    /// is the property that makes routing an already-byte-exact class through
    /// it a re-expression rather than a change, and it is asserted over the
    /// whole `bc` field rather than at a handful of points.
    #[test]
    fn the_gate_agrees_with_the_encoder_across_the_entire_field() {
        let (bo, bi) = (BO_FALSE, cr_bi(CR_COMPARE, CR_BIT_EQ));
        let mut checked = 0usize;
        let mut d = -BC_MAX_DISP - 4;
        while d <= BC_MAX_DISP {
            assert_eq!(
                direct(Form::Bc { bo, bi }, d, "sweep").ok(),
                encode_bc(bo, bi, d),
                "disp {d}"
            );
            checked += 1;
            d += 4;
        }
        assert_eq!(checked, 16_384, "every representable BD, both signs");
        // The same for the `b` form, at the edges of its own field.
        for d in [-B_EDGE, -4, 0, 4, B_EDGE] {
            assert_eq!(
                direct(Form::B, d, "sweep").ok(),
                encode_b_intra(d),
                "disp {d}"
            );
        }
    }

    const B_EDGE: i32 = 0x01FF_FFFC;
}
