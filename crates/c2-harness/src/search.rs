//! T-A (angle C) — the **IL-space search prototype**.
//!
//! The inversion thesis reduced to practice: given a TARGET `.obj`, search IL
//! space — starting from a seed [`IlModel`], applying K3a edit moves, judging
//! each candidate by compiling it through **real c2** — for an IL whose obj is
//! **byte-exact** to the target. K3a gave the verified edit primitive; this
//! closes the loop into a hill-climber and measures how efficiently it does so.
//!
//! ## Doctrine (CLAUDE.md correctness boundary)
//!
//! The compiler + obj compare is the **sole judge**. A candidate is a SUCCESS
//! only when its c2-compiled obj is **byte-exact** (timestamp-normalized) to the
//! target — [`Judged::ByteExact`], full-obj [`ObjImage::diff`] `Identical`. The
//! `.text` fuzzy score ([`fuzzy_text`]) is the search **gradient ONLY**; it
//! guides the climb and is never a terminal criterion. Every candidate is judged
//! by a REAL replay ([`Toolchain::replay_within`], timeout-bounded per P0.6c) —
//! no simulated scoring on the toolchain path. Edits go through the K3a
//! fail-closed API; an out-of-scope edit refuses cleanly and the search skips it.
//!
//! ## The loop
//!
//! `propose → compile → score → accept`, terminal = byte-exact:
//! 1. From the current [`IlModel`], enumerate a bounded neighborhood of K3a
//!    edits ([`MoveSet::neighbors`]) — each is a fresh candidate model; a refused
//!    edit ([`c2_il::EditError`]) is simply not emitted.
//! 2. Judge each candidate with a [`Scorer`] (the real one replays through c2;
//!    the mock one scores against a target model for the portable tests).
//! 3. If any candidate is byte-exact → **solved**. Else greedily accept the
//!    highest-fuzzy candidate that strictly improves on the current model and
//!    repeat; on no improvement, stop (or take a deterministic restart).
//! Budget-bounded (`max_steps`, `max_compiles`); an exhausted budget is reported
//! as an honest failure, never a fuzzy "success".
//!
//! ## Solvable-instance protocol (the honest solve-rate)
//!
//! A failure must be a real *search* failure, not an unreachable target. So the
//! instances put a solution one move away by construction: capture a fixture,
//! let the **solution IL** be its parsed model and the **target obj** its replay,
//! then perturb the solution by a SMALL known edit *inside the move set* (widen a
//! literal, add/drop a term) to make the **seed**. The inverse edit is in the
//! neighborhood, so a byte-exact IL is provably reachable — the climber either
//! recovers it (measuring search efficiency) or reveals a real gradient failure.
//! [`solve_rate`] runs this over a fixture roster and reports solve-rate@d plus
//! mean compiles-to-solve.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use c2_il::{ExToken, IlBundle, IlModel};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::{CapturedReference, Toolchain};

use crate::corpus;
use crate::retrieval::{self, text_section, Item};

// ===========================================================================
// Gradient — the `.text` fuzzy score (search guide ONLY, never terminal)
// ===========================================================================

/// PPC-word (4-byte) match ratio between a candidate obj and the target, over
/// their COFF `.text` sections. `1.0` iff the emitted code matches word-for-word
/// (which, combined with matching relocs/headers, is the byte-exact case the
/// terminal check confirms separately); `0.0` on disjoint code.
///
/// `.text`-only by design (per P1.3 / il-witness P1.3): the full obj embeds its
/// `/Fo` path in `S_OBJNAME`, so a whole-obj ratio would be path-dominated. The
/// gradient scores the *code*; the terminal success check is full
/// timestamp-normalized byte equality (see [`Judged`]). Objs are compared on
/// their normalized bytes so the COFF `TimeDateStamp` never perturbs the score.
pub fn fuzzy_text(cand: &ObjImage, target: &ObjImage) -> f64 {
    let cn = cand.normalized();
    let tn = target.normalized();
    let (ct, _) = text_section(&cn);
    let (tt, _) = text_section(&tn);
    word_match_ratio(ct, tt)
}

/// Fraction of aligned 4-byte words that are equal, over `max(words_a, words_b)`
/// (so a length mismatch is penalized). Trailing bytes shorter than a word are
/// compared as a final partial word. Two empty slices score `1.0` (vacuously
/// equal); one empty and one not scores `0.0`.
///
/// This is the **superseded** flat gradient (retained for reference / the
/// word-ratio unit test). The climber now scores with [`insn_text_similarity`],
/// which gives the instruction-granular partial credit that guides multi-move
/// descent (the d=2 stall this ratio could not break — it scores a body with a
/// fixed opcode but wrong operand identically to one that fixed nothing).
fn word_match_ratio(a: &[u8], b: &[u8]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let wa = a.len().div_ceil(4);
    let wb = b.len().div_ceil(4);
    let denom = wa.max(wb);
    if denom == 0 {
        return 1.0;
    }
    let mut matched = 0usize;
    for w in 0..wa.min(wb) {
        let lo = w * 4;
        let hi_a = (lo + 4).min(a.len());
        let hi_b = (lo + 4).min(b.len());
        if a[lo..hi_a] == b[lo..hi_b] {
            matched += 1;
        }
    }
    matched as f64 / denom as f64
}

// ===========================================================================
// Instruction-aware `.text` gradient (the multi-move descent guide)
// ===========================================================================
//
// The word-ratio above is blind to instruction structure: it compares whole
// 4-byte words for equality, so a candidate that fixed an opcode but left an
// operand wrong (`addi r11,r3,5` vs the target `addi r3,r3,5`) scores 0 for that
// word — identical to a candidate that got the opcode wrong too. On the tiny MVP
// bodies that flatness collapses the d=2 gradient (deleting one of two redundant
// terms does not raise the ratio above the seed → the climber stalls at a local
// optimum). The instruction-aware similarity decodes `.text` into PPC words and
// grades each instruction with PARTIAL credit — opcode match is worth a base,
// each correct operand field adds more — so fixing one field at a time is a
// strictly-uphill move, and a differing-length body is aligned by a gap-tolerant
// DP so an inserted/deleted instruction is graded rather than shattering the
// positional compare. It is STILL only the gradient: the terminal is unchanged,
// full timestamp-normalized [`ObjImage::diff`] `Identical` (see [`Judged`]).

/// Fraction of the per-instruction score awarded for a matching opcode identity
/// (primary + extended opcode); the remainder is split across the operand fields.
/// `0.5` makes "fixed the opcode, operands still wrong" score halfway — strictly
/// above "wrong opcode" (`≤ 0.15`) and strictly below a full match, so descending
/// one field at a time is monotone.
const OPCODE_WEIGHT: f64 = 0.5;

/// A PPC instruction word decoded down to the fields the gradient compares: an
/// **opcode identity** (`opkey`, the primary opcode plus the extended opcode for
/// the XO/XL/branch forms) and an ordered list of operand fields. Deliberately
/// coarse — enough to grade "same opcode, which operands agree", not a full
/// disassembler.
#[derive(Clone, Debug)]
struct PpcInsn {
    raw: u32,
    /// Primary opcode, bits 0-5 (IBM convention, bit 0 = MSB).
    primary: u8,
    /// Opcode identity: primary opcode, folded with the extended opcode for the
    /// forms where the primary alone is ambiguous (op 31 XO, op 19 XL, op 18's
    /// AA/LK). Two instructions with equal `opkey` are the "same instruction,
    /// maybe different operands" case.
    opkey: u32,
    /// The operand fields, in a fixed positional order per form (dest, source A,
    /// source B / immediate). Same `opkey` ⇒ same form ⇒ same length, so they
    /// compare positionally. Each field is tagged [`Operand::Reg`] or
    /// [`Operand::Imm`] so the register-renaming-tolerant credit
    /// ([`register_bijection`]) only remaps register fields, never immediates.
    operands: Vec<Operand>,
}

/// A decoded operand field, tagged so register-renaming tolerance
/// ([`register_bijection`]) applies to **register** fields only — an immediate
/// (a folded literal, a branch displacement) is never remapped, it must match by
/// value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operand {
    /// A GPR field (dest / source A / source B). Its number is subject to a
    /// consistent renaming under the best-effort bijection.
    Reg(u32),
    /// An immediate / displacement field. Compared by raw value only.
    Imm(u32),
}

/// Decode one big-endian PPC word into its [`PpcInsn`] fields. The field split
/// follows `docs/CODEGEN_PPC_MVP.md`: op 31 (XO-form: add/mullw/subf/mflr/…),
/// op 19 (XL-form: bclr/`blr`), op 18 (I-form branch: `b`/`bl`, REL24), and a
/// D-form default (addi/addis/ori/lwz/stw/… — dest, rA, 16-bit immediate) that
/// grades any other primary opcode positionally.
fn decode_ppc(word: u32) -> PpcInsn {
    let primary = (word >> 26) as u8;
    let dest = (word >> 21) & 0x1F; // bits 6-10  (RT/RD/RS/BO)
    let ra = (word >> 16) & 0x1F; // bits 11-15 (RA/BI)
    let rb = (word >> 11) & 0x1F; // bits 16-20 (RB/BB/SH)
    let imm16 = word & 0xFFFF; // bits 16-31 (D-form immediate)
    let (opkey, operands) = match primary {
        31 | 19 => {
            // XO / XL form: the extended opcode (bits 21-30) disambiguates.
            // All three fields are registers (dest, source A, source B).
            let xo = (word >> 1) & 0x3FF;
            (
                (u32::from(primary) << 16) | xo,
                vec![Operand::Reg(dest), Operand::Reg(ra), Operand::Reg(rb)],
            )
        }
        18 => {
            // I-form branch: AA/LK (bits 30-31) fold into the identity; the
            // signed 24-bit displacement (bits 6-29) is the sole (immediate)
            // operand.
            let li = (word >> 2) & 0x00FF_FFFF;
            ((18u32 << 16) | (word & 0x3), vec![Operand::Imm(li)])
        }
        // D-form default (addi/addis/ori/lwz/stw/…): dest reg, source-A reg,
        // 16-bit immediate.
        _ => (
            u32::from(primary) << 16,
            vec![Operand::Reg(dest), Operand::Reg(ra), Operand::Imm(imm16)],
        ),
    };
    PpcInsn {
        raw: word,
        primary,
        opkey,
        operands,
    }
}

/// Does candidate operand `c` match target operand `t`, optionally under a
/// candidate→target **register** renaming `phi`? Immediates always compare by raw
/// value. A register `c` matches `t` iff `phi[c] == t` (a consistent rename) or,
/// where `c` is unmapped, iff `c == t` (raw). A register that IS mapped elsewhere
/// does NOT also raw-match — so the bijection is a renaming, not "any reg matches
/// any" (which would over-credit).
fn operand_matches(c: &Operand, t: &Operand, phi: Option<&BTreeMap<u32, u32>>) -> bool {
    match (c, t) {
        (Operand::Imm(x), Operand::Imm(y)) => x == y,
        (Operand::Reg(x), Operand::Reg(y)) => match phi.and_then(|m| m.get(x)) {
            Some(mapped) => mapped == y,
            None => x == y,
        },
        _ => false,
    }
}

/// Similarity of two decoded instructions in `0.0..=1.0`, with the partial credit
/// that makes the gradient smooth:
/// - byte-identical → `1.0`;
/// - same opcode identity → [`OPCODE_WEIGHT`] plus the operand-agreement fraction
///   scaled into the rest (so fixing operands one at a time climbs toward 1.0);
/// - different opcode but same primary (e.g. two op-31 XOs) → a small `0.15`
///   floor (the family is right, the operation wrong);
/// - otherwise → `0.0`.
///
/// `phi` is an optional candidate→target register renaming ([`register_bijection`]):
/// operand agreement is scored **under** it, so an instruction that is correct up
/// to a consistent temp-register reshuffle (c2 re-colors r11/r10/… when a term is
/// added/removed — see `docs/CODEGEN_PPC_MVP.md`) earns full operand credit
/// instead of being penalized for the differing register numbers. `None` = the
/// raw (renaming-blind) compare.
fn insn_similarity(a: &PpcInsn, b: &PpcInsn, phi: Option<&BTreeMap<u32, u32>>) -> f64 {
    if a.raw == b.raw {
        return 1.0;
    }
    if a.opkey != b.opkey {
        return if a.primary == b.primary { 0.15 } else { 0.0 };
    }
    // Same opcode identity ⇒ same form ⇒ equal-length operand vectors.
    let n = a.operands.len();
    if n == 0 {
        return 1.0; // an operand-free opcode (e.g. `blr`) that matched its key
    }
    let matched = a
        .operands
        .iter()
        .zip(&b.operands)
        .filter(|(x, y)| operand_matches(x, y, phi))
        .count();
    OPCODE_WEIGHT + (1.0 - OPCODE_WEIGHT) * (matched as f64 / n as f64)
}

/// Decode a `.text` byte slice into its big-endian instruction words. A trailing
/// run shorter than a full word (malformed/padding) is zero-extended into a final
/// word so no bytes are silently dropped from the compare.
fn decode_text(text: &[u8]) -> Vec<u32> {
    text.chunks(4)
        .map(|c| {
            let mut w = [0u8; 4];
            w[..c.len()].copy_from_slice(c);
            u32::from_be_bytes(w)
        })
        .collect()
}

/// The best-alignment DP over two decoded instruction sequences, scored by a
/// per-pair similarity closure. Returns the total aligned similarity **and** the
/// list of aligned (candidate-index, target-index) diagonal pairs (the
/// traceback), so a caller can both read the score and learn which instructions
/// matched (to mine a register renaming from them). Gaps (insert/delete) score 0.
fn align_dp(
    da: &[PpcInsn],
    db: &[PpcInsn],
    sim: &dyn Fn(&PpcInsn, &PpcInsn) -> f64,
) -> (f64, Vec<(usize, usize)>) {
    let (m, n) = (da.len(), db.len());
    let mut dp = vec![vec![0f64; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            let diag = dp[i - 1][j - 1] + sim(&da[i - 1], &db[j - 1]);
            let up = dp[i - 1][j];
            let left = dp[i][j - 1];
            dp[i][j] = diag.max(up).max(left);
        }
    }
    // Traceback, preferring the diagonal on ties (so an equal-credit alignment
    // records the pairing) — deterministic.
    let mut pairs = Vec::new();
    let (mut i, mut j) = (m, n);
    while i > 0 && j > 0 {
        let diag = dp[i - 1][j - 1] + sim(&da[i - 1], &db[j - 1]);
        if (dp[i][j] - diag).abs() < 1e-12 {
            pairs.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if (dp[i][j] - dp[i - 1][j]).abs() < 1e-12 {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    pairs.reverse();
    (dp[m][n], pairs)
}

/// Build a best-effort candidate→target **register** renaming from a set of
/// aligned instruction pairs. Only same-opcode pairs contribute, and only their
/// register fields (position-matched): each such pair casts a vote
/// `cand_reg → tgt_reg`. A greedy **injective** assignment then takes the
/// highest-voted `(c, t)` pairs first, skipping any whose `c` or `t` is already
/// claimed — so the result is a *consistent renaming* (one c ↦ one t, one t ⟵ one
/// c), NOT "any register matches any" (which would over-credit). Deterministic:
/// votes are tallied and drained in a fixed (count desc, c asc, t asc) order.
fn register_bijection(
    da: &[PpcInsn],
    db: &[PpcInsn],
    pairs: &[(usize, usize)],
) -> BTreeMap<u32, u32> {
    let mut votes: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for &(i, j) in pairs {
        let (ca, cb) = (&da[i], &db[j]);
        if ca.opkey != cb.opkey {
            continue;
        }
        for (x, y) in ca.operands.iter().zip(&cb.operands) {
            if let (Operand::Reg(rc), Operand::Reg(rt)) = (x, y) {
                *votes.entry((*rc, *rt)).or_insert(0) += 1;
            }
        }
    }
    // Sort by vote count desc, then (c, t) asc for determinism.
    let mut ranked: Vec<((u32, u32), usize)> = votes.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut phi: BTreeMap<u32, u32> = BTreeMap::new();
    let mut used_t: BTreeSet<u32> = BTreeSet::new();
    for ((c, t), _) in ranked {
        if phi.contains_key(&c) || used_t.contains(&t) {
            continue;
        }
        phi.insert(c, t);
        used_t.insert(t);
    }
    phi
}

/// Instruction-sequence similarity in `0.0..=1.0`, **register-renaming-tolerant**.
/// Equal-length sequences are compared positionally; different-length sequences
/// are aligned by a gap-tolerant DP (Needleman-Wunsch, match = [`insn_similarity`],
/// gap = 0) normalized by `max(len)`, so an inserted/deleted instruction is graded
/// and a length mismatch is penalized.
///
/// Two passes: (1) align + score with the raw (renaming-blind) per-pair
/// similarity, mine a consistent candidate→target register renaming from the
/// aligned same-opcode pairs ([`register_bijection`]); (2) re-score under that
/// renaming and return the **better** of the two normalized scores. So a candidate
/// that is correct up to c2's temp-register reshuffle (r11/r10/… recolored when a
/// term is added/removed) is credited for the structural match instead of being
/// penalized for the register numbers — while a wrong opcode or a genuinely
/// different structure gains nothing (the renaming is injective and same-opcode
/// only). Two empty sequences score `1.0`; one empty and one not scores `0.0`.
fn insn_seq_similarity(a: &[u32], b: &[u32]) -> f64 {
    let (m, n) = (a.len(), b.len());
    if m == 0 && n == 0 {
        return 1.0;
    }
    if m == 0 || n == 0 {
        return 0.0;
    }
    let da: Vec<PpcInsn> = a.iter().map(|&w| decode_ppc(w)).collect();
    let db: Vec<PpcInsn> = b.iter().map(|&w| decode_ppc(w)).collect();
    let denom = m.max(n) as f64;

    let raw = |x: &PpcInsn, y: &PpcInsn| insn_similarity(x, y, None);
    let (raw_total, pairs) = align_dp(&da, &db, &raw);
    let raw_score = raw_total / denom;

    // Pass 2: mine a renaming from the raw alignment, re-score under it.
    let phi = register_bijection(&da, &db, &pairs);
    if phi.is_empty() {
        return raw_score;
    }
    let toln = |x: &PpcInsn, y: &PpcInsn| insn_similarity(x, y, Some(&phi));
    let (tol_total, _) = align_dp(&da, &db, &toln);
    (tol_total / denom).max(raw_score)
}

/// Instruction-aware `.text` similarity between a candidate obj and the target —
/// the search **gradient** (never a terminal; see [`fuzzy_text`]'s note and
/// [`Judged`]). Decodes each obj's COFF `.text` into PPC instruction words and
/// scores them with [`insn_seq_similarity`], so a move that fixes an opcode or an
/// operand field scores strictly higher than one that does not. `.text`-only for
/// the same path-freeness reason as [`fuzzy_text`] (the full obj embeds its `/Fo`
/// path); objs are compared on their timestamp-normalized bytes.
///
/// **Reconciliation with the terminal:** this can reach `1.0` on an obj that is
/// NOT byte-exact — the `.text` decode is blind to relocations, the symbol table,
/// and `.debug$S`, so two objs with identical code but differing tail bytes score
/// `1.0` here yet compare `Differs` under [`ObjImage::diff`]. A `1.0` gradient is
/// therefore NEVER a success; only [`Judged::ByteExact`] terminates.
pub fn insn_text_similarity(cand: &ObjImage, target: &ObjImage) -> f64 {
    let cn = cand.normalized();
    let tn = target.normalized();
    let (ct, _) = text_section(&cn);
    let (tt, _) = text_section(&tn);
    insn_seq_similarity(&decode_text(ct), &decode_text(tt))
}

// ===========================================================================
// Per-function-decomposed gradient (the whole-`.text` plateau fix)
// ===========================================================================
//
// The whole-`.text` gradient above scores the WHOLE code section in one aligned
// DP, so on a multi-function TU the intact sibling functions dominate the score:
// a correct edit to the one function under edit moves the aggregate by only its
// small share of the total instruction count, and greedy descent stalls at a
// plateau (the 0-step stalls the T-A readout diagnosed as "the multi-function
// whole-`.text` plateau"). The per-function gradient splits `.text` into
// per-function segments and averages their similarity with EQUAL weight per
// function, so an improvement to the edited function is credited at `1/nfns`
// regardless of how large the intact siblings are — the masked progress becomes
// visible. It is STILL only the gradient: the terminal is unchanged, full
// timestamp-normalized `ObjImage::diff` `Identical`.

/// PPC `blr` (`4E80 0020`) — the return terminator each MVP straight-line
/// function ends with; used to split a `.text` word stream into per-function
/// segments (each such function returns via exactly one `blr`).
const BLR_WORD: u32 = 0x4E80_0020;

/// Split a decoded `.text` word stream into per-function segments at each `blr`
/// terminator. Each `blr` ends its segment (and is included in it); a trailing
/// run with no final `blr` becomes a final segment so no words are dropped.
fn split_by_blr(words: &[u32]) -> Vec<Vec<u32>> {
    let mut segs: Vec<Vec<u32>> = Vec::new();
    let mut cur: Vec<u32> = Vec::new();
    for &w in words {
        cur.push(w);
        if w == BLR_WORD {
            segs.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        segs.push(cur);
    }
    segs
}

/// Per-function instruction-sequence similarity: split both word streams into
/// per-`blr` segments and average [`insn_seq_similarity`] over the segment pairs
/// with EQUAL weight per function, so a corrected edit to one function is not
/// diluted by intact sibling functions (the whole-`.text` plateau).
///
/// **Guarded fallback.** If the two streams do not split into the same number of
/// segments — an off-target structure (an edit that changed the return/function
/// count, or a body whose code did not decode into the expected `blr` pattern) —
/// this falls back to the whole-stream [`insn_seq_similarity`] rather than align
/// mismatched segments (which would mis-credit). `nfns` is the caller's expected
/// function count (the K3a invariant); the segment split must agree with it AND
/// between the two streams, or the honest whole-stream score is used.
fn insn_seq_similarity_perfn(a: &[u32], b: &[u32], nfns: usize) -> f64 {
    let sa = split_by_blr(a);
    let sb = split_by_blr(b);
    if sa.is_empty() || sa.len() != sb.len() || sb.len() != nfns {
        return insn_seq_similarity(a, b);
    }
    let n = sa.len() as f64;
    let sum: f64 = sa
        .iter()
        .zip(&sb)
        .map(|(x, y)| insn_seq_similarity(x, y))
        .sum();
    sum / n
}

/// Per-function-decomposed instruction-aware `.text` similarity — the multi-
/// function search **gradient** (never a terminal; see [`insn_text_similarity`]).
/// Decodes each obj's COFF `.text` into PPC words, splits both into per-`blr`
/// function segments, and averages the per-segment similarity with equal weight,
/// so an edit to the function under edit is scored at `1/nfns` instead of being
/// masked by intact siblings. Falls back to the whole-`.text` score when the two
/// segment splits disagree (see [`insn_seq_similarity_perfn`]).
pub fn insn_text_similarity_perfn(cand: &ObjImage, target: &ObjImage, nfns: usize) -> f64 {
    let cn = cand.normalized();
    let tn = target.normalized();
    let (ct, _) = text_section(&cn);
    let (tt, _) = text_section(&tn);
    insn_seq_similarity_perfn(&decode_text(ct), &decode_text(tt), nfns)
}

// ===========================================================================
// Scorer — the judge abstraction (real replay vs. mock, same climber)
// ===========================================================================

/// The verdict on one candidate model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Judged {
    /// The candidate's obj is byte-exact (timestamp-normalized, full obj) to the
    /// target — the ONLY success. Terminal.
    ByteExact,
    /// The candidate compiled; `.text` fuzzy gradient in `0.0..=1.0`. A guide,
    /// never a success — a `1.0` here that is not also `ByteExact` means the code
    /// matched but the obj did not (relocs/headers differ), and the search
    /// continues.
    Fuzzy(f64),
    /// The candidate did not compile — a replay crash / timeout, or (upstream) a
    /// refused edit. A clean per-candidate reject; the search skips it.
    Reject,
}

/// Judges a candidate [`IlModel`] against an (implicit) target, counting every
/// real compile. The climber ([`hill_climb`]) is written entirely against this
/// trait, so the same accept/terminal/budget logic runs under a toolchain-free
/// mock ([`MockScorer`]) and the real c2 replay ([`ReplayScorer`]).
pub trait Scorer {
    /// Judge `model`. Implementations MUST count a real compile here (see
    /// [`Scorer::compiles`]); the mock counts a comparison.
    fn judge(&mut self, model: &IlModel) -> Judged;
    /// Total judgements performed — the compiles-to-solve metric.
    fn compiles(&self) -> usize;
}

// ===========================================================================
// Move set — the K3a-licensed neighborhood
// ===========================================================================

/// Which K3a edit families the neighborhood enumerates. The default is the full
/// licensed family; [`MoveSet::length_only`] restricts to the pure length moves
/// (widen/narrow + term add/delete) that P0.6a proved re-optimize byte-exact.
#[derive(Clone, Debug)]
pub struct MoveSet {
    /// Widen/narrow the varint form of each int literal (same value; P0.6a A/B).
    pub widen_narrow: bool,
    /// Nudge each int literal by each delta in `value_nudges` (relative, so a
    /// value perturbation of magnitude ≤ the nudge range is recoverable). The
    /// emitted immediate is a flat field, so this is search-by-trial, not a
    /// smooth gradient — kept to a small local window.
    pub literal_value: bool,
    /// Relative deltas tried by `literal_value` (`current + delta`).
    pub value_nudges: Vec<i32>,
    /// Delete a trailing `<operand> <op>` term (`a+b+c` → `a+b`; P0.6a F).
    pub term_delete: bool,
    /// Insert a `<operand> <op>` term after an existing value token (`a+5` →
    /// `(a+5)+5`; P0.6a E). Ops from `insert_ops`; the operand vocabulary is the
    /// body's own operands **plus**, when `insert_from_scope`, generated operands
    /// (params in scope + `insert_literals`) so a *vanished* operand can be
    /// regenerated (the drop-term lossy-seed case).
    pub term_insert: bool,
    /// Binary ops used when inserting a term.
    pub insert_ops: Vec<ExToken>,
    /// Generative insert vocabulary: also enumerate insert operands NOT present in
    /// the body — every formal parameter in scope (as a `Load`) and each literal
    /// in `insert_literals` — so a term whose operand vanished from the seed (a
    /// dropped `+param` or `+k`) can be reconstructed. Bounded (params ≤ arity,
    /// literals a small fixed set) to keep the branching factor sane.
    pub insert_from_scope: bool,
    /// The small literal vocabulary generated for insert when `insert_from_scope`
    /// (the "vanished literal" set — a dropped `+k` is only recoverable if `k` is
    /// here or already elsewhere in the body).
    pub insert_literals: Vec<i32>,
    /// **Opt-in, OFF by default** ([`MoveSet::default`]/[`MoveSet::length_only`]
    /// both leave it `false`; enable via [`MoveSet::with_mul_reorder`]). When set,
    /// [`MoveSet::neighbors`] emits, for each `MUL` (`04`) node whose two immediate
    /// operands are single-token leaves, the operand-swapped ordering as one d=1
    /// neighbor — the primitive for the commutative-order class (the real
    /// `Box::Volume` regalloc floor is a commutative reorder).
    ///
    /// **Why it is gated (CLAUDE.md rule 1).** The move is licensed ONLY because
    /// `04` is **commutative**, so the swap stays inside the licensed action space.
    /// It is guarded strictly to `04`: an operand swap of `03` (SUB), `-`, `/`,
    /// `%`, `<<`, `>>`, or a function/argument/comparison swap is a
    /// *non-commutative silent corruption* (the DC3 `SetupCharacter` flip of
    /// `z0 - aspect*size` matched flat at 83.8% and made all game text invisible),
    /// so this generator NEVER emits one. It is off by default because the binding
    /// seam is the **adopt-as-truth / floor-certification** path: a generator that
    /// can emit swaps must be opt-in there, since `fuzzy%` cannot see a
    /// match-neutral behavioral corruption in machine-minted source trusted without
    /// a byte-exact witness. On the *search* path the byte-exact terminal (the sole
    /// judge) makes any accepted candidate behavior-safe as a crack regardless, so
    /// enabling the move only widens which candidates are compiled — it can never
    /// promote a non-byte-exact result.
    pub mul_reorder: bool,
}

impl Default for MoveSet {
    fn default() -> Self {
        MoveSet {
            widen_narrow: true,
            literal_value: true,
            value_nudges: vec![-10, -5, -3, -2, -1, 1, 2, 3, 5, 10],
            term_delete: true,
            term_insert: true,
            insert_ops: vec![ExToken::Add, ExToken::Sub, ExToken::Mul],
            insert_from_scope: true,
            insert_literals: vec![1, 2, 5],
            mul_reorder: false, // opt-in only (see the field docstring)
        }
    }
}

impl MoveSet {
    /// The pure length-move family: widen/narrow + term add/delete, no literal
    /// value enumeration. This is the family K3a/P0.6a licensed as
    /// re-optimize-to-byte-exact, and the one whose recovery the climber guides
    /// structurally rather than by trial.
    pub fn length_only() -> Self {
        MoveSet {
            widen_narrow: true,
            literal_value: false,
            value_nudges: Vec::new(),
            term_delete: true,
            term_insert: true,
            insert_ops: vec![ExToken::Add, ExToken::Sub, ExToken::Mul],
            insert_from_scope: true,
            insert_literals: vec![1, 2, 5],
            mul_reorder: false, // opt-in only (see the field docstring)
        }
    }

    /// Enable the opt-in commutative **MUL-factor reorder** move (OFF by default).
    /// See the [`MoveSet::mul_reorder`] field docstring for the guard rationale
    /// (MUL/`04`-only, licensed by commutativity; the adopt-as-truth /
    /// floor-certification seam is why it is opt-in). Chains on any constructor,
    /// e.g. `MoveSet::default().with_mul_reorder()`.
    pub fn with_mul_reorder(mut self) -> Self {
        self.mul_reorder = true;
        self
    }

    /// Enumerate the bounded neighborhood of `model`: every in-scope K3a edit,
    /// applied to a fresh clone. A refused edit ([`c2_il::EditError`]) is skipped
    /// (fail-closed — the model is left untouched by a failed splice). Candidates
    /// are deduplicated by their `.ex` bytes and returned in a deterministic
    /// order, each labelled for the readout/log.
    pub fn neighbors(&self, model: &IlModel) -> Vec<(String, IlModel)> {
        let mut out: Vec<(String, IlModel)> = Vec::new();
        let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
        // The seed's own `.ex` is not a neighbor of itself.
        if let Some(ex) = model.encode().get("ex") {
            seen.insert(ex.to_vec());
        }

        let nfns = model.ex_function_count();
        for fi in 0..nfns {
            let Ok(tokens) = model.function_tokens(fi) else {
                continue; // opaque body — not token-addressable
            };

            // ---- widen / narrow each literal -------------------------------
            if self.widen_narrow {
                for (ti, tok) in tokens.iter().enumerate() {
                    if let ExToken::Lit { wide, .. } = tok {
                        let mut cand = model.clone();
                        if cand.set_literal_wide(fi, ti, !wide).is_ok() {
                            let label = format!(
                                "fn{fi} lit@{ti} {}",
                                if *wide { "narrow" } else { "widen" }
                            );
                            push(&mut out, &mut seen, label, cand);
                        }
                    }
                }
            }

            // ---- nudge each literal by a local delta ------------------------
            if self.literal_value {
                for (ti, tok) in tokens.iter().enumerate() {
                    if let ExToken::Lit { value, wide } = *tok {
                        for &d in &self.value_nudges {
                            let Some(v) = value.checked_add(d) else {
                                continue;
                            };
                            if v == value {
                                continue;
                            }
                            let mut cand = model.clone();
                            // Keep the same varint width where the value permits;
                            // a narrow slot with a wide value must widen.
                            let want_wide = wide || !(0..=0x7F).contains(&v);
                            let repl = vec![ExToken::Lit { value: v, wide: want_wide }];
                            if cand
                                .splice_function_tokens(fi, ti..ti + 1, repl)
                                .is_ok()
                            {
                                push(&mut out, &mut seen, format!("fn{fi} lit@{ti} {v:+}"), cand);
                            }
                        }
                    }
                }
            }

            // ---- delete a trailing `<operand> <op>` term -------------------
            if self.term_delete {
                for i in 0..tokens.len().saturating_sub(1) {
                    if is_operand(&tokens[i]) && is_binop(&tokens[i + 1]) {
                        let mut cand = model.clone();
                        if cand.splice_function_tokens(fi, i..i + 2, vec![]).is_ok() {
                            push(&mut out, &mut seen, format!("fn{fi} del term@{i}"), cand);
                        }
                    }
                }
            }

            // ---- insert a `<operand> <op>` term after a value token ---------
            // A value-producing token (operand OR binop) leaves exactly one net
            // value on the stack, so appending `<operand> <op>` after it is
            // always valid postfix (`… V` → `… (V op W)`). Anchoring after
            // operands too — not only ops — lets insert reconstruct a term even
            // when the seed body has no remaining binop (e.g. a fully-dropped
            // single term), the direction P0.6a E exercised.
            if self.term_insert {
                let operands: Vec<ExToken> = if self.insert_from_scope {
                    generative_operands(&tokens, &self.insert_literals)
                } else {
                    distinct_operands(&tokens)
                };
                for (i, tok) in tokens.iter().enumerate() {
                    if !is_operand(tok) && !is_binop(tok) {
                        continue;
                    }
                    for operand in &operands {
                        for op in &self.insert_ops {
                            let mut cand = model.clone();
                            let repl = vec![operand.clone(), op.clone()];
                            if cand
                                .splice_function_tokens(fi, i + 1..i + 1, repl)
                                .is_ok()
                            {
                                let label = format!("fn{fi} ins@{i} {}", op_name(op));
                                push(&mut out, &mut seen, label, cand);
                            }
                        }
                    }
                }
            }

            // ---- commutative MUL-factor reorder (opt-in; MUL `04` ONLY) ------
            // For a `… A B MUL` node whose two immediate operands `A`,`B` are
            // single-token leaves, emit the operand-swapped ordering `… B A MUL`
            // as one d=1 neighbor. Guarded strictly to `ExToken::Mul` (opcode
            // `04`) — the ONLY commutative binop here; SUB/`03` (and `-` `/` `%`
            // `<<` `>>`, argument/comparison swaps) are non-commutative silent
            // corruptions and are never swapped (CLAUDE.md rule 1; the guard's
            // full rationale + the adopt-as-truth/floor-cert seam are on the
            // `mul_reorder` field docstring). Identical operands (`a*a`) are
            // skipped — the swap is a no-op the `.ex` dedup would drop anyway.
            if self.mul_reorder {
                for i in 2..tokens.len() {
                    if !matches!(tokens[i], ExToken::Mul) {
                        continue;
                    }
                    let (a, b) = (&tokens[i - 2], &tokens[i - 1]);
                    if is_operand(a) && is_operand(b) && a != b {
                        let mut cand = model.clone();
                        let repl = vec![b.clone(), a.clone()];
                        if cand.splice_function_tokens(fi, i - 2..i, repl).is_ok() {
                            push(&mut out, &mut seen, format!("fn{fi} mul-swap@{i}"), cand);
                        }
                    }
                }
            }
        }
        out
    }
}

fn push(
    out: &mut Vec<(String, IlModel)>,
    seen: &mut BTreeSet<Vec<u8>>,
    label: String,
    cand: IlModel,
) {
    let ex = cand
        .encode()
        .get("ex")
        .map(|b| b.to_vec())
        .unwrap_or_default();
    if seen.insert(ex) {
        out.push((label, cand));
    }
}

fn is_operand(t: &ExToken) -> bool {
    // `FloatLoad` is a leaf operand exactly as `Load` is — the float-leaf codec
    // widening (Box::Volume class) types the float-arith stream, and its member
    // diffs/products push a single float value per `FloatLoad`, so a `FloatLoad`
    // is a delete/insert anchor for the length moves (and a swap anchor for the
    // MUL reorder below), just like the int `Load`. Without this a float leaf has
    // zero operand anchors, i.e. an empty action space on every real dc3 target.
    matches!(
        t,
        ExToken::Load(_) | ExToken::FloatLoad(_) | ExToken::Lit { .. }
    )
}

fn is_binop(t: &ExToken) -> bool {
    matches!(t, ExToken::Add | ExToken::Sub | ExToken::Mul)
}

fn op_name(t: &ExToken) -> &'static str {
    match t {
        ExToken::Add => "add",
        ExToken::Sub => "sub",
        ExToken::Mul => "mul",
        _ => "op",
    }
}

/// The distinct operand tokens in a body (each `Load`/`Lit`), in first-seen
/// order — the operands a term-insert reuses.
fn distinct_operands(tokens: &[ExToken]) -> Vec<ExToken> {
    let mut out: Vec<ExToken> = Vec::new();
    for t in tokens {
        if is_operand(t) && !out.contains(t) {
            out.push(t.clone());
        }
    }
    out
}

/// The **generative** insert-operand vocabulary for a function body: its distinct
/// body operands (as [`distinct_operands`]) **plus** operands that may have
/// *vanished* from the body —
/// 1. every **formal parameter** in scope, as a `Load(t)` (a function's
///    `Formal(t)` header token and its `Load(t)` share the token id `t`, so a
///    param used nowhere in the current body is still loadable); and
/// 2. each literal in `literals`, as a narrow `Lit`.
///
/// This lets a term-insert reconstruct a dropped `+param` or `+k` even though the
/// operand no longer appears in the seed (the drop-term lossy-seed case a
/// reuse-only insert cannot solve). Deduplicated, first-seen order; bounded by
/// arity + `literals.len()` so the branching factor stays sane.
fn generative_operands(tokens: &[ExToken], literals: &[i32]) -> Vec<ExToken> {
    let mut out = distinct_operands(tokens);
    let mut add = |t: ExToken| {
        if !out.contains(&t) {
            out.push(t);
        }
    };
    // Params in scope: each Formal(id) is loadable as Load(id).
    for t in tokens {
        if let ExToken::Formal(id) = t {
            add(ExToken::Load(*id));
        }
    }
    // The small "vanished literal" set.
    for &v in literals {
        add(ExToken::Lit {
            value: v,
            wide: !(0..=0x7F).contains(&v),
        });
    }
    out
}

// ===========================================================================
// The climber
// ===========================================================================

/// Search budget. A hill-climb stops at the first of: byte-exact (success),
/// `max_steps` accepted moves, `max_compiles` judgements, or no improving
/// neighbor (a local optimum, unless restarts remain).
#[derive(Clone, Copy, Debug)]
pub struct Budget {
    pub max_steps: usize,
    pub max_compiles: usize,
    /// Deterministic restarts from the seed on hitting a local optimum. `0` = a
    /// single greedy descent. Restarts re-run the same deterministic
    /// neighborhood, so they only help a scorer with nondeterminism or a future
    /// randomized tie-break; kept for the interface, defaulted off.
    pub restarts: usize,
    /// Beam width — how many best candidates the search keeps at each step. `1`
    /// is pure greedy hill-climb (accept only a strictly-improving move, stop at
    /// a local optimum). `≥ 2` is a beam that keeps the top-`k` candidates by
    /// fuzzy gradient **even when none improves on the parent**, so the search can
    /// take a non-improving (lateral/downhill) step to cross a plateau and reach
    /// the byte-exact basin the greedy climb stalls before (the d≥2 add-term
    /// stall). The terminal is unchanged — only a byte-exact obj wins; the beam
    /// only widens which candidates are compiled.
    pub beam_width: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Budget {
            max_steps: 8,
            max_compiles: 400,
            restarts: 0,
            beam_width: 4,
        }
    }
}

/// Why the climb stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    /// A byte-exact candidate was found (success).
    Solved,
    /// A local optimum: no neighbor strictly improved the fuzzy score.
    LocalOptimum,
    /// `max_steps` reached without a byte-exact candidate.
    StepsExhausted,
    /// `max_compiles` reached without a byte-exact candidate.
    CompilesExhausted,
}

/// Outcome of one hill-climb.
#[derive(Clone, Debug)]
pub struct SearchOutcome {
    pub solved: bool,
    pub steps: usize,
    pub compiles: usize,
    pub best_fuzzy: f64,
    pub reason: StopReason,
    /// The move labels accepted along the winning/best path (for the log).
    pub path: Vec<String>,
}

/// A live beam node: the model, its fuzzy gradient, and the move path that
/// reached it.
struct BeamNode {
    fuzzy: f64,
    model: IlModel,
    path: Vec<String>,
}

/// The `.ex` bytes of a model (its dedup / identity key). Empty if it has no
/// `.ex` file (never, for a captured/hand-built function model).
fn ex_bytes(model: &IlModel) -> Vec<u8> {
    model
        .encode()
        .get("ex")
        .map(|b| b.to_vec())
        .unwrap_or_default()
}

/// Greedy IL-space hill-climb from `seed` — the width-1 special case of
/// [`beam_search`] (accept only a strictly-improving move; stop at a local
/// optimum). Kept as the name the portable greedy tests and the terminal-pin
/// drive; forces `beam_width = 1` regardless of `budget`.
///
/// Deterministic: the neighborhood order is fixed and ties are broken by
/// first-seen (the enumeration order), so with a deterministic scorer the whole
/// climb is reproducible — no wall-clock, no RNG.
///
/// TERMINAL is byte-exact ([`Judged::ByteExact`]) and nothing else — a fuzzy
/// `1.0` that is not byte-exact keeps the search going. On a compile/replay
/// reject the candidate is skipped, never fatal.
pub fn hill_climb(
    seed: &IlModel,
    moves: &MoveSet,
    scorer: &mut dyn Scorer,
    budget: &Budget,
) -> SearchOutcome {
    let mut b = *budget;
    b.beam_width = 1;
    beam_search(seed, moves, scorer, &b)
}

/// IL-space **beam search** from `seed`, judged by `scorer`, bounded by `budget`,
/// exploring the [`MoveSet`] neighborhood. Keeps the top-`budget.beam_width`
/// candidates by fuzzy gradient at each step.
///
/// - **width 1** is pure greedy: accept the single strictly-improving best move,
///   stop [`StopReason::LocalOptimum`] when none improves (identical to the
///   original hill-climb; that is what [`hill_climb`] calls).
/// - **width ≥ 2** keeps the top-`k` candidates **even when none beats the
///   parent**, so the search can take a non-improving (lateral / slightly
///   downhill) step to cross a fuzzy plateau and reach the byte-exact basin the
///   greedy climb stalls before (the d≥2 add-term stall: no single term-delete
///   raises the whole-`.text` gradient, but two deletes reach the exact IL, and
///   the byte-exact terminal — not the gradient — fires when they do).
///
/// Deterministic: neighborhoods are enumerated in a fixed order and the beam is
/// truncated by `(fuzzy desc, .ex bytes asc)`; every judged model is globally
/// de-duplicated by its `.ex` bytes, so no model is compiled twice and the
/// compile budget is spent on new candidates. No wall-clock, no RNG.
///
/// TERMINAL is byte-exact ([`Judged::ByteExact`]) and nothing else — a fuzzy
/// `1.0` that is not byte-exact keeps the search going. On a compile/replay
/// reject the candidate is skipped, never fatal. Budget-bounded (`max_steps`
/// beam rounds, `max_compiles` judgements); an exhausted budget is an honest
/// failure, never a fuzzy "success".
pub fn beam_search(
    seed: &IlModel,
    moves: &MoveSet,
    scorer: &mut dyn Scorer,
    budget: &Budget,
) -> SearchOutcome {
    let width = budget.beam_width.max(1);

    // Judge the seed. (A perturbed seed is not byte-exact, but a caller may hand
    // us an already-solved model — honor it.)
    let seed_judged = scorer.judge(seed);
    if seed_judged == Judged::ByteExact {
        return SearchOutcome {
            solved: true,
            steps: 0,
            compiles: scorer.compiles(),
            best_fuzzy: 1.0,
            reason: StopReason::Solved,
            path: Vec::new(),
        };
    }
    let seed_fuzzy = match seed_judged {
        Judged::Fuzzy(f) => f,
        _ => 0.0, // seed itself did not compile — climb from a zero floor
    };
    let mut best_fuzzy = seed_fuzzy;
    // The highest-fuzzy path seen (the honest "best effort" path on a non-solve).
    let mut best_path: Vec<String> = Vec::new();

    // Global dedup: never compile the same `.ex` twice.
    let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
    seen.insert(ex_bytes(seed));

    let mut frontier: Vec<BeamNode> = vec![BeamNode {
        fuzzy: seed_fuzzy,
        model: seed.clone(),
        path: Vec::new(),
    }];

    let done = |solved, path: Vec<String>, best_fuzzy, reason, compiles| SearchOutcome {
        solved,
        steps: path.len(),
        compiles,
        best_fuzzy,
        reason,
        path,
    };

    for _round in 0..budget.max_steps {
        // Expand every frontier node into fresh, de-duplicated, judged candidates.
        let mut cands: Vec<BeamNode> = Vec::new();
        for node in &frontier {
            for (label, cand) in moves.neighbors(&node.model) {
                if !seen.insert(ex_bytes(&cand)) {
                    continue; // already judged this exact model
                }
                if scorer.compiles() >= budget.max_compiles {
                    return done(false, best_path, best_fuzzy, StopReason::CompilesExhausted, scorer.compiles());
                }
                match scorer.judge(&cand) {
                    Judged::ByteExact => {
                        let mut p = node.path.clone();
                        p.push(label);
                        return done(true, p, 1.0, StopReason::Solved, scorer.compiles());
                    }
                    Judged::Fuzzy(f) => {
                        let mut p = node.path.clone();
                        p.push(label);
                        if f > best_fuzzy {
                            best_fuzzy = f;
                            best_path = p.clone();
                        }
                        cands.push(BeamNode {
                            fuzzy: f,
                            model: cand,
                            path: p,
                        });
                    }
                    Judged::Reject => {} // skip cleanly
                }
            }
        }

        if cands.is_empty() {
            // No new distinct candidates anywhere in the beam — converged.
            return done(false, best_path, best_fuzzy, StopReason::LocalOptimum, scorer.compiles());
        }

        if width == 1 {
            // Greedy: the single best candidate, and only if it strictly improves
            // on the parent (else a local optimum). First-seen wins ties.
            let cur = frontier[0].fuzzy;
            let best_idx = cands
                .iter()
                .enumerate()
                .fold(0usize, |bi, (i, n)| if n.fuzzy > cands[bi].fuzzy { i } else { bi });
            if cands[best_idx].fuzzy > cur {
                let chosen = cands.remove(best_idx);
                frontier = vec![chosen];
            } else {
                return done(false, best_path, best_fuzzy, StopReason::LocalOptimum, scorer.compiles());
            }
        } else {
            // Beam: keep the top-k by (fuzzy desc, .ex asc) — a NON-improving step
            // is allowed, which is what crosses the plateau. Deterministic order.
            cands.sort_by(|a, b| {
                b.fuzzy
                    .partial_cmp(&a.fuzzy)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| ex_bytes(&a.model).cmp(&ex_bytes(&b.model)))
            });
            cands.truncate(width);
            frontier = cands;
        }
    }

    done(false, best_path, best_fuzzy, StopReason::StepsExhausted, scorer.compiles())
}

// ===========================================================================
// ReplayScorer — the real c2 judge
// ===========================================================================

/// Judges candidates by a REAL standalone-c2 replay through the toolchain, to a
/// FIXED `-Fo` path (so the embedded `S_OBJNAME` matches the target and a
/// byte-exact terminal is achievable), bounded by a timeout (a replay
/// crash/timeout is a clean [`Judged::Reject`], per P0.6c). This is the sole
/// judge on the real path — no simulated scoring.
pub struct ReplayScorer<'a> {
    tc: &'a Toolchain,
    base: &'a CapturedReference,
    target: ObjImage,
    scratch: PathBuf,
    /// The FIXED `-Fo` path every replay (target render + all candidates) uses,
    /// so the embedded `S_OBJNAME` matches and a byte-exact terminal is possible.
    pub fo: PathBuf,
    timeout: Duration,
    counter: usize,
    compiles: usize,
    /// When `Some(nfns)` (and `nfns > 1`), score the fuzzy gradient with the
    /// per-function-decomposed similarity ([`insn_text_similarity_perfn`]) so a
    /// correct edit to one function of a multi-function target is not masked by
    /// its intact siblings (the whole-`.text` plateau). `None` = the whole-`.text`
    /// gradient. The TERMINAL (byte-exact) is identical either way.
    per_fn: Option<usize>,
}

impl<'a> ReplayScorer<'a> {
    /// `base` supplies the captured c2 argv (its `-il`/`-Fo` are swapped per
    /// replay); `target` is the obj to reach byte-exact; `scratch` is a private
    /// work dir (candidate bundles + the fixed `-Fo` obj land under it). The
    /// fixed `-Fo` is [`ReplayScorer::fo`] — render the target to it (see
    /// [`solve_instance`]) so target and candidates share the embedded path.
    pub fn new(
        tc: &'a Toolchain,
        base: &'a CapturedReference,
        target: ObjImage,
        scratch: PathBuf,
        timeout: Duration,
    ) -> Self {
        let fo = scratch.join("cand.obj");
        ReplayScorer {
            tc,
            base,
            target,
            scratch,
            fo,
            timeout,
            counter: 0,
            compiles: 0,
            per_fn: None,
        }
    }

    /// The fixed `-Fo` path candidates and the target both replay to.
    pub fn fo_path(&self) -> &Path {
        &self.fo
    }

    /// Switch the fuzzy gradient to the per-function-decomposed similarity for a
    /// multi-function target (`nfns > 1`); `nfns <= 1` leaves the whole-`.text`
    /// gradient (a single function has nothing to decompose). The terminal check
    /// is unchanged. Returns `self` for chaining.
    pub fn per_function(&mut self, nfns: usize) -> &mut Self {
        self.per_fn = if nfns > 1 { Some(nfns) } else { None };
        self
    }
}

impl<'a> Scorer for ReplayScorer<'a> {
    fn judge(&mut self, model: &IlModel) -> Judged {
        self.compiles += 1;
        self.counter += 1;
        let cap = CapturedReference {
            bundle: model.encode(),
            ..self.base.clone()
        };
        let dir = self.scratch.join(format!("cand{}", self.counter));
        let verdict = match self
            .tc
            .replay_within(&cap, &dir, &self.fo, self.timeout)
        {
            Ok(obj) => {
                if matches!(ObjImage::diff(&obj, &self.target), ObjDiff::Identical) {
                    Judged::ByteExact
                } else {
                    // Instruction-aware gradient (never a terminal — see
                    // `insn_text_similarity`'s reconciliation note). The byte-exact
                    // terminal above is the sole success; this only ranks moves.
                    // For a multi-function target the per-function decomposition
                    // keeps the edited function's progress from being masked by
                    // intact siblings (the whole-`.text` plateau).
                    let fuzzy = match self.per_fn {
                        Some(nfns) => insn_text_similarity_perfn(&obj, &self.target, nfns),
                        None => insn_text_similarity(&obj, &self.target),
                    };
                    Judged::Fuzzy(fuzzy)
                }
            }
            Err(_) => Judged::Reject, // crash / timeout / no obj — skip cleanly
        };
        let _ = std::fs::remove_dir_all(&dir);
        verdict
    }

    fn compiles(&self) -> usize {
        self.compiles
    }
}

// ===========================================================================
// Solvable-instance harness — the honest solve-rate
// ===========================================================================

/// One perturbation family used to build a solvable instance from a solution IL.
///
/// Each family's inverse is in the [`MoveSet`], so a byte-exact IL is reachable
/// by construction — a failure is a real *search* failure. Note that `WidenLit`
/// is **obj-invisible** on the real toolchain (P0.6a A: c2 re-optimizes a
/// widened literal to byte-identical code), so it is a valid perturbation only in
/// `.ex`-space (the mock scorer / unit tests); the real solve-rate roster uses
/// the obj-changing families (`AddTerm`, `LitNudge`, `DropTerm`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Perturb {
    /// Widen a narrow literal — `.ex`-visible but obj-invisible (see above).
    WidenLit,
    /// Insert `d` redundant `+<operand>` terms (seed longer; recover by `d`
    /// deletes — a genuine obj change, gradient-guided for `d ≥ 2`).
    AddTerm,
    /// Nudge a literal by `+3d` (seed's immediate differs; recover by a value
    /// move — flat gradient, so recovery is enumeration within the nudge window).
    LitNudge,
    /// Delete a trailing term (seed shorter; recover by insert — only where the
    /// dropped operand survives elsewhere in the body).
    DropTerm,
}

impl Perturb {
    /// A short name for logs/readouts.
    pub fn label(&self) -> &'static str {
        match self {
            Perturb::WidenLit => "widen-lit",
            Perturb::AddTerm => "add-term",
            Perturb::LitNudge => "lit-nudge",
            Perturb::DropTerm => "drop-term",
        }
    }
}

/// Apply a `d`-step perturbation to `solution`, returning the seed model, or
/// `None` if there is no site (the instance is skipped, never faked). `d` is the
/// perturbation distance: `d` stacked edits, whose `d`-move inverse the climber
/// must find.
pub fn perturb(solution: &IlModel, kind: Perturb, d: usize) -> Option<IlModel> {
    let mut m = solution.clone();
    for _ in 0..d.max(1) {
        m = perturb_step(&m, kind)?;
    }
    // A perturbation that produced no net `.ex` change is not a real instance.
    if m.encode().get("ex") == solution.encode().get("ex") {
        return None;
    }
    Some(m)
}

/// A d=1 perturbation (one edit). Retained for the unit tests.
pub fn perturb_once(solution: &IlModel, kind: Perturb) -> Option<IlModel> {
    perturb(solution, kind, 1)
}

fn perturb_step(model: &IlModel, kind: Perturb) -> Option<IlModel> {
    let nfns = model.ex_function_count();
    for fi in 0..nfns {
        let Ok(tokens) = model.function_tokens(fi) else {
            continue;
        };
        match kind {
            Perturb::WidenLit => {
                for (ti, tok) in tokens.iter().enumerate() {
                    if matches!(tok, ExToken::Lit { wide: false, .. }) {
                        let mut m = model.clone();
                        if m.set_literal_wide(fi, ti, true).is_ok() {
                            return Some(m);
                        }
                    }
                }
            }
            Perturb::AddTerm => {
                // Duplicate the first operand as a `+<operand>` term after the
                // first binop — a redundant term the climber removes by delete.
                let operands = distinct_operands(&tokens);
                let first_op = tokens.iter().position(is_binop);
                if let (Some(operand), Some(i)) = (operands.first(), first_op) {
                    let mut m = model.clone();
                    let repl = vec![operand.clone(), ExToken::Add];
                    if m.splice_function_tokens(fi, i + 1..i + 1, repl).is_ok() {
                        return Some(m);
                    }
                }
            }
            Perturb::LitNudge => {
                for (ti, tok) in tokens.iter().enumerate() {
                    if let ExToken::Lit { value, wide } = *tok {
                        let v = value.wrapping_add(3);
                        let mut m = model.clone();
                        let want_wide = wide || !(0..=0x7F).contains(&v);
                        let repl = vec![ExToken::Lit { value: v, wide: want_wide }];
                        if m.splice_function_tokens(fi, ti..ti + 1, repl).is_ok() {
                            return Some(m);
                        }
                    }
                }
            }
            Perturb::DropTerm => {
                for i in 0..tokens.len().saturating_sub(1) {
                    if is_operand(&tokens[i]) && is_binop(&tokens[i + 1]) {
                        let mut m = model.clone();
                        if m.splice_function_tokens(fi, i..i + 2, vec![]).is_ok() {
                            return Some(m);
                        }
                    }
                }
            }
        }
    }
    None
}

/// One instance's result within a [`SolveReport`].
#[derive(Clone, Debug)]
pub struct InstanceResult {
    pub fixture: String,
    pub perturb: Perturb,
    pub d: usize,
    /// `None` = no site for this perturbation on this fixture (skipped).
    pub outcome: Option<SearchOutcome>,
    /// A toolchain/capture error (also skipped, reported honestly).
    pub error: Option<String>,
}

/// Aggregate solve-rate over a roster of solvable instances.
#[derive(Clone, Debug, Default)]
pub struct SolveReport {
    pub instances: Vec<InstanceResult>,
}

impl SolveReport {
    /// (attempted, solved, mean-compiles-to-solve) — attempted excludes skipped
    /// (no-site) and errored instances so the rate is over real search attempts.
    pub fn tally(&self) -> (usize, usize, f64) {
        Self::tally_of(self.instances.iter())
    }

    /// Per-`(family, d)` breakdown, in first-seen order — so a lumped headline
    /// never hides that different families have different reachability.
    pub fn by_family(&self) -> Vec<((Perturb, usize), (usize, usize, f64))> {
        let mut keys: Vec<(Perturb, usize)> = Vec::new();
        for r in &self.instances {
            let k = (r.perturb, r.d);
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
        keys.into_iter()
            .map(|k| {
                let t = Self::tally_of(
                    self.instances.iter().filter(|r| (r.perturb, r.d) == k),
                );
                (k, t)
            })
            .collect()
    }

    fn tally_of<'a, I: Iterator<Item = &'a InstanceResult>>(it: I) -> (usize, usize, f64) {
        let mut attempted = 0usize;
        let mut solved = 0usize;
        let mut compiles_sum = 0usize;
        for r in it {
            if let Some(o) = &r.outcome {
                attempted += 1;
                if o.solved {
                    solved += 1;
                    compiles_sum += o.compiles;
                }
            }
        }
        let mean = if solved > 0 {
            compiles_sum as f64 / solved as f64
        } else {
            0.0
        };
        (attempted, solved, mean)
    }
}

/// Build a solvable instance from a fixture `.cpp` and one perturbation, then
/// climb it, judged by real c2. Captures the fixture, takes the parsed model as
/// the solution and its replay as the target, perturbs to a seed, and hill-climbs
/// back. Requires a ready toolchain (see [`Toolchain::has_strace`]/`has_mingw`).
#[allow(clippy::too_many_arguments)]
pub fn solve_instance(
    tc: &Toolchain,
    cpp: &Path,
    kind: Perturb,
    d: usize,
    moves: &MoveSet,
    budget: &Budget,
    scratch: &Path,
    timeout: Duration,
) -> InstanceResult {
    let fixture = cpp
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mk = |outcome, error| InstanceResult {
        fixture: fixture.clone(),
        perturb: kind,
        d,
        outcome,
        error,
    };

    let base = match tc.capture_reference(cpp, &scratch.join("cap")) {
        Ok(c) => c,
        Err(e) => return mk(None, Some(format!("capture: {e}"))),
    };
    let solution = match IlModel::parse(&base.bundle) {
        Ok(m) => m,
        Err(e) => return mk(None, Some(format!("codec: {e}"))),
    };
    // The scorer's fixed `-Fo` is a pure function of its scratch dir, so compute
    // it up front, render the TARGET (the solution IL replayed) to it, then hand
    // the target's bytes to the scorer. Target and every candidate thus share the
    // embedded `S_OBJNAME`, making a byte-exact terminal reachable.
    let search_dir = scratch.join("search");
    let fo = search_dir.join("cand.obj");
    let target = match tc.replay_within(&base, &scratch.join("tgt_il"), &fo, timeout) {
        Ok(o) => o,
        Err(e) => return mk(None, Some(format!("target replay: {e}"))),
    };
    let mut scorer = ReplayScorer::new(tc, &base, target, search_dir, timeout);
    debug_assert_eq!(scorer.fo_path(), fo.as_path());

    let Some(seed) = perturb(&solution, kind, d) else {
        return mk(None, None); // no site — skipped, not a failure
    };

    // The real solve-rate path runs the beam (width from `budget.beam_width`;
    // width 1 degrades to greedy) so multi-move descents can cross the plateaus
    // greedy stalls on. TERMINAL is unchanged — byte-exact obj only.
    let outcome = beam_search(&seed, moves, &mut scorer, budget);
    mk(Some(outcome), None)
}

/// Run the solvable-instance protocol over a roster of fixtures × perturbations,
/// returning the aggregate [`SolveReport`]. Deterministic given the roster.
#[allow(clippy::too_many_arguments)]
pub fn solve_rate(
    tc: &Toolchain,
    fixtures: &[PathBuf],
    perturbs: &[(Perturb, usize)],
    moves: &MoveSet,
    budget: &Budget,
    scratch: &Path,
    timeout: Duration,
) -> SolveReport {
    let mut report = SolveReport::default();
    let mut n = 0usize;
    for cpp in fixtures {
        for &(kind, d) in perturbs {
            let dir = scratch.join(format!("inst{n}"));
            n += 1;
            let r = solve_instance(tc, cpp, kind, d, moves, budget, &dir, timeout);
            let _ = std::fs::remove_dir_all(&dir);
            report.instances.push(r);
        }
    }
    report
}

// ===========================================================================
// From-unrelated-seed — the P1.3-retrieval-seeded search (the REAL pipeline)
// ===========================================================================
//
// The solvable-instance protocol above seeds from a SMALL perturbation of the
// known solution, so a byte-exact IL is one move away by construction — it prices
// the search but is not the real task. This rung attempts the real pipeline:
// given a TARGET obj whose IL is unknown, use **P1.3 retrieval** to pick the
// nearest corpus IL as the seed, then beam-search from that unrelated seed toward
// the target — terminal byte-exact. Most targets have no corpus twin (retrieval
// recall@1 is low), so the seed is only APPROXIMATE: the search must bridge the
// gap through K3a edits, and the honest solve-rate + failure taxonomy is the
// finding.

/// The seed the retrieval step picks for a target, or why there is none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeedChoice {
    /// The nearest non-self corpus neighbor is an exact-`.text` **twin** of the
    /// target — it compiles byte-exact with no search (a trivial / degenerate
    /// "retrieval-solved" case). Reported separately, never fed to the search.
    RetrievalTrivial { twin_id: String },
    /// The nearest non-self, non-twin neighbor's index in the item slice — the
    /// (approximate) seed the search starts from.
    Seed { index: usize },
    /// No candidate at all (the item slice held only the target).
    NoCandidate,
}

/// Pick the retrieval seed for `target` from `items` (the corpus): rank by the
/// P1.3 obj-`.text` cosine feature, take the nearest neighbor that is **not the
/// target's own row** and **not an exact-`.text` twin** (a twin is a trivial
/// solve, reported separately). Pure over the item features — deterministic, no
/// toolchain.
pub fn select_seed(target: &Item, items: &[Item]) -> SeedChoice {
    for i in retrieval::rank(target, items) {
        let cand = &items[i];
        if cand.id == target.id {
            continue; // never seed from self
        }
        if cand.text_key == target.text_key {
            // The nearest non-self neighbor is a behavioral twin → trivial solve.
            return SeedChoice::RetrievalTrivial {
                twin_id: cand.id.clone(),
            };
        }
        return SeedChoice::Seed { index: i };
    }
    SeedChoice::NoCandidate
}

/// The taxonomy bucket for one from-seed target outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FromSeedClass {
    /// A `.text` twin was retrieved — solved without any search (trivial).
    RetrievalTrivial,
    /// Seed and target have different function counts — out of K3a scope
    /// (whole-function add/remove is K3b, unimplemented), so the search is not
    /// attempted.
    K3bBlocked,
    /// The search reached a byte-exact obj from the unrelated seed.
    Solved,
    /// The search stalled at a local optimum below byte-exact — the move set
    /// (K3a edits + generative vocab) could not bridge the seed→target gap
    /// (a plateau / vocabulary limit).
    Plateau,
    /// The search hit its step or compile budget still short of byte-exact.
    BudgetExhausted,
    /// A per-target error (seed bundle would not load/parse, a capture/replay
    /// failure) — reported honestly, never faked as a solve.
    Error,
}

impl FromSeedClass {
    pub fn label(&self) -> &'static str {
        match self {
            FromSeedClass::RetrievalTrivial => "retrieval-trivial",
            FromSeedClass::K3bBlocked => "k3b-blocked",
            FromSeedClass::Solved => "SOLVED",
            FromSeedClass::Plateau => "plateau/vocab",
            FromSeedClass::BudgetExhausted => "budget-exhausted",
            FromSeedClass::Error => "error",
        }
    }
}

/// One target's from-seed result.
#[derive(Clone, Debug)]
pub struct FromSeedRecord {
    pub target_id: String,
    pub target_fns: usize,
    pub seed_id: Option<String>,
    pub seed_fns: Option<usize>,
    pub class: FromSeedClass,
    /// A short human note (twin id, fn-count mismatch, error text, …).
    pub detail: String,
    /// The primary search outcome (per-function gradient for a multi-function
    /// target, whole-`.text` for single-function). `None` for a non-searched
    /// class (trivial / K3b / error before search).
    pub outcome: Option<SearchOutcome>,
    /// For a multi-function target only: the SAME search run WITHOUT the
    /// per-function gradient (whole-`.text`), the with/without comparison the
    /// plateau-fix is measured against.
    pub outcome_wholetext: Option<SearchOutcome>,
}

/// Config for a from-retrieval eval run — kept small and bounded (CPU is shared).
#[derive(Clone, Debug)]
pub struct FromSeedConfig {
    /// Total held-out targets to attempt.
    pub sample: usize,
    /// Of `sample`, how many should be multi-function (to exercise the
    /// per-function gradient); the rest are single-function.
    pub multi: usize,
    /// Deterministic sample-selection seed (a stride offset over the sorted ids).
    pub select_seed: u64,
    /// Per-target search budget.
    pub budget: Budget,
    /// Per-replay wall-clock timeout.
    pub timeout: Duration,
}

impl Default for FromSeedConfig {
    fn default() -> Self {
        FromSeedConfig {
            sample: 24,
            multi: 4,
            select_seed: 0,
            budget: Budget {
                max_steps: 10,
                max_compiles: 300,
                restarts: 0,
                beam_width: 5,
            },
            timeout: Duration::from_secs(30),
        }
    }
}

/// The aggregate from-retrieval report.
#[derive(Clone, Debug, Default)]
pub struct FromSeedReport {
    pub records: Vec<FromSeedRecord>,
    /// Corpus size the sample was drawn from.
    pub n_items: usize,
}

impl FromSeedReport {
    /// Count of records in each class, in a fixed order.
    pub fn class_counts(&self) -> Vec<(FromSeedClass, usize)> {
        let order = [
            FromSeedClass::Solved,
            FromSeedClass::Plateau,
            FromSeedClass::BudgetExhausted,
            FromSeedClass::K3bBlocked,
            FromSeedClass::RetrievalTrivial,
            FromSeedClass::Error,
        ];
        order
            .iter()
            .map(|&c| (c, self.records.iter().filter(|r| r.class == c).count()))
            .collect()
    }

    /// (searched, solved): searched excludes trivial / K3b / error (the classes
    /// where no real search ran), so the solve-rate is over genuine attempts.
    pub fn search_tally(&self) -> (usize, usize) {
        let searched = self
            .records
            .iter()
            .filter(|r| r.outcome.is_some())
            .count();
        let solved = self
            .records
            .iter()
            .filter(|r| r.class == FromSeedClass::Solved)
            .count();
        (searched, solved)
    }
}

/// Per-corpus-row metadata the from-seed runner needs (from the manifest).
struct RowMeta {
    source_rel: String,
    il_dir_rel: String,
    il_base: String,
    fns: usize,
}

/// Deterministically pick `n` ids from the sorted `ids`, strided from a
/// `seed`-derived start so the sample spreads across the corpus and is
/// reproducible. Tops up on collisions so it always returns `min(n, len)` ids.
fn pick_ids(ids: &[String], n: usize, seed: u64) -> Vec<String> {
    let len = ids.len();
    if len == 0 || n == 0 {
        return Vec::new();
    }
    let n = n.min(len);
    let stride = (len / n).max(1);
    let start = (seed as usize) % len;
    let mut used: BTreeSet<usize> = BTreeSet::new();
    let mut out: Vec<String> = Vec::new();
    let mut step = 0usize;
    while out.len() < n && step < len {
        let k = (start + step * stride) % len;
        if used.insert(k) {
            out.push(ids[k].clone());
        }
        step += 1;
    }
    let mut k = 0usize;
    while out.len() < n && k < len {
        if used.insert(k) {
            out.push(ids[k].clone());
        }
        k += 1;
    }
    out
}

/// Run the from-unrelated-seed protocol over a deterministically-selected sample
/// of held-out corpus targets, returning the per-target records + aggregate.
///
/// For each target: pick the retrieval seed ([`select_seed`], excluding self and
/// `.text` twins); scope to a compatible function structure (same function count
/// — a mismatch is [`FromSeedClass::K3bBlocked`], NOT forced); render the target
/// obj by replaying its own IL to a fixed `-Fo` (so a byte-exact terminal is
/// reachable); then beam-search from the seed toward the target obj, judged by a
/// REAL c2 replay, terminal byte-exact. Multi-function targets are searched with
/// the per-function gradient AND, for the with/without comparison, again with the
/// whole-`.text` gradient.
pub fn from_retrieval_eval(
    tc: &Toolchain,
    root: &Path,
    moves: &MoveSet,
    cfg: &FromSeedConfig,
    scratch: &Path,
) -> std::io::Result<FromSeedReport> {
    let items = retrieval::load_items(root)?;
    let manifest = corpus::load_manifest(root)?;

    // id -> metadata (only `ok` rows with a full IL side).
    let mut meta: BTreeMap<String, RowMeta> = BTreeMap::new();
    for r in manifest {
        if r.status != "ok" {
            continue;
        }
        if let (Some(source_rel), Some(il_dir_rel), Some(il_base)) =
            (r.source_rel, r.il_dir_rel, r.il_base)
        {
            meta.insert(
                r.id.clone(),
                RowMeta {
                    source_rel,
                    il_dir_rel,
                    il_base,
                    fns: r.gl_offsets.len(),
                },
            );
        }
    }
    let idx_of: BTreeMap<String, usize> = items
        .iter()
        .enumerate()
        .map(|(i, it)| (it.id.clone(), i))
        .collect();

    // Partition ids present in BOTH the items and the metadata by function count.
    let mut single: Vec<String> = Vec::new();
    let mut multi: Vec<String> = Vec::new();
    for (id, m) in &meta {
        if !idx_of.contains_key(id) {
            continue;
        }
        if m.fns <= 1 {
            single.push(id.clone());
        } else if m.fns >= 2 {
            multi.push(id.clone());
        }
    }
    single.sort();
    multi.sort();

    let n_multi = cfg.multi.min(cfg.sample);
    let n_single = cfg.sample.saturating_sub(n_multi);
    let mut targets = pick_ids(&single, n_single, cfg.select_seed);
    targets.extend(pick_ids(&multi, n_multi, cfg.select_seed));

    let mut report = FromSeedReport {
        records: Vec::new(),
        n_items: items.len(),
    };

    for (n, target_id) in targets.iter().enumerate() {
        let t_idx = idx_of[target_id];
        let target_item = &items[t_idx];
        let t_meta = &meta[target_id];
        let target_fns = t_meta.fns;

        let mut rec = FromSeedRecord {
            target_id: target_id.clone(),
            target_fns,
            seed_id: None,
            seed_fns: None,
            class: FromSeedClass::Error,
            detail: String::new(),
            outcome: None,
            outcome_wholetext: None,
        };

        // --- seed selection (retrieval; excludes self + twins) --------------
        match select_seed(target_item, &items) {
            SeedChoice::NoCandidate => {
                rec.detail = "no retrieval candidate".into();
                report.records.push(rec);
                continue;
            }
            SeedChoice::RetrievalTrivial { twin_id } => {
                rec.seed_id = Some(twin_id.clone());
                rec.class = FromSeedClass::RetrievalTrivial;
                rec.detail = format!("exact-.text twin {twin_id} retrieved (no search)");
                report.records.push(rec);
                continue;
            }
            SeedChoice::Seed { index } => {
                let seed_item = &items[index];
                rec.seed_id = Some(seed_item.id.clone());
                let Some(s_meta) = meta.get(&seed_item.id) else {
                    rec.detail = "seed row missing from manifest".into();
                    report.records.push(rec);
                    continue;
                };

                // --- load + parse the seed IL (no toolchain) ----------------
                let seed_dir = root.join(&s_meta.il_dir_rel);
                let seed_bundle = match IlBundle::load_from_dir(&seed_dir, &s_meta.il_base) {
                    Ok(b) => b,
                    Err(e) => {
                        rec.detail = format!("seed bundle load: {e}");
                        report.records.push(rec);
                        continue;
                    }
                };
                let seed_model = match IlModel::parse(&seed_bundle) {
                    Ok(m) => m,
                    Err(e) => {
                        rec.detail = format!("seed codec: {e}");
                        report.records.push(rec);
                        continue;
                    }
                };
                let seed_fns = seed_model.ex_function_count();
                rec.seed_fns = Some(seed_fns);

                // --- in-scope filter (compatible function structure) --------
                if seed_fns != target_fns {
                    rec.class = FromSeedClass::K3bBlocked;
                    rec.detail =
                        format!("seed {seed_fns} fns vs target {target_fns} — K3b (whole-fn) out of scope");
                    report.records.push(rec);
                    continue;
                }

                // --- render the target obj (its own IL → fixed -Fo) ---------
                let inst_dir = scratch.join(format!("inst{n}"));
                let src = root.join(&t_meta.source_rel);
                let base = match tc.capture_reference(&src, &inst_dir.join("cap")) {
                    Ok(c) => c,
                    Err(e) => {
                        rec.detail = format!("target capture: {e}");
                        report.records.push(rec);
                        let _ = std::fs::remove_dir_all(&inst_dir);
                        continue;
                    }
                };
                let search_dir = inst_dir.join("search");
                let fo = search_dir.join("cand.obj");
                let target_obj =
                    match tc.replay_within(&base, &inst_dir.join("tgt_il"), &fo, cfg.timeout) {
                        Ok(o) => o,
                        Err(e) => {
                            rec.detail = format!("target replay: {e}");
                            report.records.push(rec);
                            let _ = std::fs::remove_dir_all(&inst_dir);
                            continue;
                        }
                    };

                // --- primary search (per-function gradient for multi) -------
                let mut scorer = ReplayScorer::new(
                    tc,
                    &base,
                    target_obj.clone(),
                    search_dir.clone(),
                    cfg.timeout,
                );
                scorer.per_function(target_fns);
                let outcome = beam_search(&seed_model, moves, &mut scorer, &cfg.budget);
                rec.class = classify_outcome(&outcome);
                rec.detail = format!("{:?} best_fuzzy={:.4}", outcome.reason, outcome.best_fuzzy);
                rec.outcome = Some(outcome);

                // --- with/without comparison run (multi-function only) ------
                // Same search_dir → same fixed `-Fo` → the rendered target obj
                // still matches candidates. Whole-`.text` gradient this time.
                if target_fns > 1 {
                    let mut scorer2 = ReplayScorer::new(
                        tc,
                        &base,
                        target_obj.clone(),
                        search_dir.clone(),
                        cfg.timeout,
                    );
                    // per_fn left None → whole-`.text`.
                    let out2 = beam_search(&seed_model, moves, &mut scorer2, &cfg.budget);
                    rec.outcome_wholetext = Some(out2);
                }

                let _ = std::fs::remove_dir_all(&inst_dir);
                report.records.push(rec);
            }
        }
    }

    Ok(report)
}

/// Map a finished search outcome to its taxonomy bucket.
fn classify_outcome(o: &SearchOutcome) -> FromSeedClass {
    if o.solved {
        return FromSeedClass::Solved;
    }
    match o.reason {
        StopReason::Solved => FromSeedClass::Solved,
        StopReason::LocalOptimum => FromSeedClass::Plateau,
        StopReason::StepsExhausted | StopReason::CompilesExhausted => {
            FromSeedClass::BudgetExhausted
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A toolchain-free scorer: judges a candidate by comparing its `.ex` bytes to
    // a fixed target model. ByteExact on equality; else a fuzzy score over the
    // fraction of matching bytes (a stand-in gradient). Exercises the climber's
    // accept / terminal / budget / reject logic with zero toolchain.
    struct MockScorer {
        target_ex: Vec<u8>,
        compiles: usize,
        /// `.ex` byte prefixes that should be treated as a compile reject.
        reject_if_contains: Option<Vec<u8>>,
    }

    impl MockScorer {
        fn new(target: &IlModel) -> Self {
            MockScorer {
                target_ex: target.encode().get("ex").unwrap().to_vec(),
                compiles: 0,
                reject_if_contains: None,
            }
        }
    }

    impl Scorer for MockScorer {
        fn judge(&mut self, model: &IlModel) -> Judged {
            self.compiles += 1;
            let ex = model.encode().get("ex").unwrap().to_vec();
            if let Some(marker) = &self.reject_if_contains {
                if ex.windows(marker.len()).any(|w| w == marker.as_slice()) {
                    return Judged::Reject;
                }
            }
            if ex == self.target_ex {
                return Judged::ByteExact;
            }
            let matched = ex
                .iter()
                .zip(&self.target_ex)
                .filter(|(a, b)| a == b)
                .count();
            let denom = ex.len().max(self.target_ex.len()).max(1);
            Judged::Fuzzy(matched as f64 / denom as f64)
        }
        fn compiles(&self) -> usize {
            self.compiles
        }
    }

    // A hand-built model: one function, body `LOAD a + 5`, with a `.gl` offset —
    // reuses the corpus synthetic-bundle shape but adds a literal so the move set
    // has widen/narrow + value + insert/delete sites.
    fn model_add_lit(lit: i32, wide: bool) -> IlModel {
        use c2_il::IlBundle;
        let mut b = IlBundle::new("_search_test");
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&c2_il::EX_MAGIC);
        ex.extend_from_slice(&[0x00; 8]);
        let fn_start = ex.len() as u32;
        ex.extend_from_slice(&[0x4F, 0x1F]); // fn start
        ex.extend_from_slice(&[0x11, 0x22]); // opaque meta
        ex.push(0x46); // Formals
        ex.extend_from_slice(&[0x2D, 0xE3, 0x01]); // Formal a
        ex.extend_from_slice(&[0x4C, 0x4F, 0x11]); // LO
        ex.push(0x53); // Ss
        ex.extend_from_slice(&[0xB9, 0xE3, 0x01, 0x86, 0x41, 0x74]); // Load a
        // literal
        ex.push(0x33);
        ex.extend_from_slice(&[0x86, 0x41, 0x74]);
        if wide {
            ex.push(0x80);
            ex.extend_from_slice(&lit.to_le_bytes());
        } else {
            ex.push(lit as u8);
        }
        ex.push(0x02); // Add
        ex.extend_from_slice(&[0x54, 0x02, 0x29, 0xE3, 0x00]); // Return
        ex.extend_from_slice(&[0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00]); // FnTail
        ex.extend_from_slice(&[0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x00, 0x4D]); // ModuleEnd
        b.set("ex", ex);

        let mut gl: Vec<u8> = Vec::new();
        gl.extend_from_slice(b"?addk@@YAHH@Z\x00");
        gl.extend_from_slice(&[0x80, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00]);
        gl.push(0x80);
        gl.extend_from_slice(&fn_start.to_le_bytes());
        b.set("gl", gl);
        b.set("sy", b"a\x00\x00".to_vec());
        b.set("in", vec![0x86, 0x41, 0x74, 0x00]);
        b.set("db", Vec::new());
        IlModel::parse(&b).expect("hand-built model parses")
    }

    // A hand-built model with body `a + a` (LOAD a, LOAD a, ADD) — a repeated
    // operand, so a dropped `+a` term is reconstructable by insert (the operand
    // survives in the seed). Same framing/`.gl` shape as `model_add_lit`.
    fn model_add_aa() -> IlModel {
        use c2_il::IlBundle;
        let mut b = IlBundle::new("_search_test_aa");
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&c2_il::EX_MAGIC);
        ex.extend_from_slice(&[0x00; 8]);
        let fn_start = ex.len() as u32;
        ex.extend_from_slice(&[0x4F, 0x1F]);
        ex.extend_from_slice(&[0x11, 0x22]);
        ex.push(0x46);
        ex.extend_from_slice(&[0x2D, 0xE3, 0x01]); // Formal a
        ex.extend_from_slice(&[0x4C, 0x4F, 0x11]); // LO
        ex.push(0x53); // Ss
        ex.extend_from_slice(&[0xB9, 0xE3, 0x01, 0x86, 0x41, 0x74]); // Load a
        ex.extend_from_slice(&[0xB9, 0xE3, 0x01, 0x86, 0x41, 0x74]); // Load a
        ex.push(0x02); // Add
        ex.extend_from_slice(&[0x54, 0x02, 0x29, 0xE3, 0x00]); // Return
        ex.extend_from_slice(&[0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00]); // FnTail
        ex.extend_from_slice(&[0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x00, 0x4D]); // ModuleEnd
        b.set("ex", ex);
        let mut gl: Vec<u8> = Vec::new();
        gl.extend_from_slice(b"?adda@@YAHH@Z\x00");
        gl.extend_from_slice(&[0x80, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00]);
        gl.push(0x80);
        gl.extend_from_slice(&fn_start.to_le_bytes());
        b.set("gl", gl);
        b.set("sy", b"a\x00\x00".to_vec());
        b.set("in", vec![0x86, 0x41, 0x74, 0x00]);
        b.set("db", Vec::new());
        IlModel::parse(&b).expect("hand-built aa model parses")
    }

    // A hand-built model with body `a <op> b` (LOAD a, LOAD b, <op_byte>) over two
    // DISTINCT formals — the minimal two-leaf binop the MUL-reorder guard is proved
    // on: `op_byte` = `0x04` (MUL, swappable) vs `0x03` (SUB) / `0x02` (ADD, not a
    // reorder target). Same framing/`.gl` shape as `model_add_aa`.
    fn model_binop_ab(op_byte: u8) -> IlModel {
        use c2_il::IlBundle;
        let mut b = IlBundle::new("_search_test_op_ab");
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&c2_il::EX_MAGIC);
        ex.extend_from_slice(&[0x00; 8]);
        let fn_start = ex.len() as u32;
        ex.extend_from_slice(&[0x4F, 0x1F]);
        ex.extend_from_slice(&[0x11, 0x22]);
        ex.push(0x46);
        ex.extend_from_slice(&[0x2D, 0xE3, 0x01]); // Formal a
        ex.extend_from_slice(&[0x2D, 0xE4, 0x01]); // Formal b
        ex.extend_from_slice(&[0x4C, 0x4F, 0x11]); // LO
        ex.push(0x53); // Ss
        ex.extend_from_slice(&[0xB9, 0xE3, 0x01, 0x86, 0x41, 0x74]); // Load a
        ex.extend_from_slice(&[0xB9, 0xE4, 0x01, 0x86, 0x41, 0x74]); // Load b
        ex.push(op_byte); // the binop under test
        ex.extend_from_slice(&[0x54, 0x02, 0x29, 0xE3, 0x00]); // Return
        ex.extend_from_slice(&[0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00]); // FnTail
        ex.extend_from_slice(&[0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x00, 0x4D]); // ModuleEnd
        b.set("ex", ex);
        let mut gl: Vec<u8> = Vec::new();
        gl.extend_from_slice(b"?opab@@YAHHH@Z\x00");
        gl.extend_from_slice(&[0x80, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00]);
        gl.push(0x80);
        gl.extend_from_slice(&fn_start.to_le_bytes());
        b.set("gl", gl);
        b.set("sy", b"a\x00b\x00\x00".to_vec());
        b.set("in", vec![0x86, 0x41, 0x74, 0x00]);
        b.set("db", Vec::new());
        IlModel::parse(&b).expect("hand-built op-ab model parses")
    }

    // ---- MUL-factor commutative-reorder move (Piece B) — the guard proof ----

    #[test]
    fn mul_reorder_is_opt_in_off_by_default() {
        // Neither default constructor turns the move on.
        assert!(!MoveSet::default().mul_reorder);
        assert!(!MoveSet::length_only().mul_reorder);
        assert!(MoveSet::default().with_mul_reorder().mul_reorder);
        // And the default neighborhood of a two-leaf MUL emits no mul-swap.
        let m = model_binop_ab(0x04);
        assert!(
            !MoveSet::default()
                .neighbors(&m)
                .iter()
                .any(|(l, _)| l.contains("mul-swap")),
            "mul-swap must not appear without opting in"
        );
    }

    #[test]
    fn mul_reorder_generated_for_mul_swaps_the_two_leaves() {
        // On `a * b`, with_mul_reorder emits exactly one mul-swap whose body is
        // `b * a` (the two leaves reordered, MUL opcode preserved).
        let m = model_binop_ab(0x04);
        let orig = m.function_tokens(0).unwrap();
        let omi = orig.iter().position(|t| matches!(t, ExToken::Mul)).unwrap();
        let (a_tok, b_tok) = (orig[omi - 2].clone(), orig[omi - 1].clone());
        assert!(matches!(a_tok, ExToken::Load(_)) && matches!(b_tok, ExToken::Load(_)));
        assert_ne!(a_tok, b_tok, "the fixture's two leaves are distinct");

        let ns = MoveSet::default().with_mul_reorder().neighbors(&m);
        let swaps: Vec<_> = ns.iter().filter(|(l, _)| l.contains("mul-swap")).collect();
        assert_eq!(swaps.len(), 1, "one two-leaf MUL ⇒ exactly one swap");

        let toks = swaps[0].1.function_tokens(0).unwrap();
        let mi = toks.iter().position(|t| matches!(t, ExToken::Mul)).unwrap();
        assert!(mi >= 2);
        assert_eq!(toks[mi - 2], b_tok, "leaf order is swapped (b now first)");
        assert_eq!(toks[mi - 1], a_tok, "leaf order is swapped (a now second)");
        assert!(matches!(toks[mi], ExToken::Mul), "the MUL opcode is preserved");
    }

    #[test]
    fn mul_reorder_never_generated_for_sub_or_add() {
        // THE GUARD: opcode `03` (SUB) and `02` (ADD) are NOT reorder targets even
        // with the move opted in — SUB is a non-commutative silent corruption, and
        // the move is strictly MUL-only (CLAUDE.md rule 1). Same two-leaf shape as
        // the MUL case, so ONLY the opcode differs.
        for op in [0x03u8, 0x02u8] {
            let m = model_binop_ab(op);
            let ns = MoveSet::default().with_mul_reorder().neighbors(&m);
            assert!(
                !ns.iter().any(|(l, _)| l.contains("mul-swap")),
                "opcode {op:#04x} must NEVER produce a mul-swap (MUL-only guard)"
            );
        }
    }

    // ---- instruction-aware gradient fixtures -------------------------------
    //
    // Real MVP PPC words (big-endian, per docs/CODEGEN_PPC_MVP.md). The ladder is
    // the exact d=2 add-term stall this rung fixes: target `a+5`, and the seed
    // bodies after 1 and 2 redundant `+a` terms.
    const ADDI_R3_R3_5: u32 = 0x3863_0005; // addi r3,r3,5   (target `a+5` op)
    const ADDI_R11_R3_5: u32 = 0x3963_0005; // addi r11,r3,5  (a+5 as a non-final temp)
    const ADD_R3_R11_R3: u32 = 0x7C6B_1A14; // add r3,r11,r3  (final `+a`)
    const ADD_R11_R11_R3: u32 = 0x7D6B_1A14; // add r11,r11,r3 (intermediate `+a`)
    const BLR: u32 = 0x4E80_0020;

    fn text_bytes(words: &[u32]) -> Vec<u8> {
        words.iter().flat_map(|w| w.to_be_bytes()).collect()
    }

    // Target `a+5`, and the d=1 / d=2 add-term seeds' `.text`.
    fn text_target() -> Vec<u8> {
        text_bytes(&[ADDI_R3_R3_5, BLR])
    }
    fn text_d1() -> Vec<u8> {
        text_bytes(&[ADDI_R11_R3_5, ADD_R3_R11_R3, BLR])
    }
    fn text_d2() -> Vec<u8> {
        text_bytes(&[ADDI_R11_R3_5, ADD_R11_R11_R3, ADD_R3_R11_R3, BLR])
    }

    #[test]
    fn insn_similarity_opcode_fix_beats_nothing_fixed() {
        let target = decode_ppc(ADDI_R3_R3_5);
        // Fixed the opcode (addi) + rA + imm, only the dest reg wrong.
        let opcode_fixed = decode_ppc(ADDI_R11_R3_5);
        // Wrong opcode entirely (an `add` where the target is `addi`).
        let nothing_fixed = decode_ppc(ADD_R3_R11_R3);
        let s_fixed = insn_similarity(&opcode_fixed, &target, None);
        let s_nothing = insn_similarity(&nothing_fixed, &target, None);
        assert!(
            s_fixed > s_nothing,
            "fixing the opcode must score higher: {s_fixed} vs {s_nothing}"
        );
        // Same opcode + 2/3 operands (dest wrong) = 0.5 + 0.5*2/3.
        assert!((s_fixed - (0.5 + 0.5 * 2.0 / 3.0)).abs() < 1e-9);
        // Different primary opcode (op14 addi vs op31 add) = 0.0.
        assert_eq!(s_nothing, 0.0);
        // Byte-identical = 1.0; same op, all operands right but only reg differs
        // is strictly below 1.0 (partial credit, never a false full match).
        assert_eq!(insn_similarity(&target, &target, None), 1.0);
        assert!(s_fixed < 1.0);
    }

    #[test]
    fn insn_seq_gradient_is_monotone_toward_target() {
        // The d=2 stall, in gradient form: deleting a redundant term must RAISE
        // the instruction-aware score (d2 < d1 < 1.0), where the old word-ratio
        // left both flat at 0 (position 0: addi r11 vs addi r3; position 1: add
        // vs blr — every word differs).
        let t = decode_text(&text_target());
        let d1 = decode_text(&text_d1());
        let d2 = decode_text(&text_d2());
        let s_d1 = insn_seq_similarity(&d1, &t);
        let s_d2 = insn_seq_similarity(&d2, &t);
        assert!(s_d2 < s_d1, "d2 ({s_d2}) must score below d1 ({s_d1})");
        assert!(s_d1 < 1.0, "d1 ({s_d1}) is not yet the target");
        assert!(s_d2 > 0.0, "d2 ({s_d2}) must earn partial credit, not flat 0");
        // The old flat gradient scored both seeds 0 — the concrete stall.
        assert_eq!(word_match_ratio(&text_d1(), &text_target()), 0.0);
        assert_eq!(word_match_ratio(&text_d2(), &text_target()), 0.0);
        // Target vs itself is a full 1.0.
        assert_eq!(insn_seq_similarity(&t, &t), 1.0);
    }

    #[test]
    fn insn_seq_edit_distance_handles_different_lengths() {
        // Different-length bodies (an inserted instruction) are aligned by the DP:
        // a body one `add` longer than the target still earns credit for the
        // aligned `addi`/`blr`, strictly between the wrong-length flat cases.
        let short = decode_text(&text_bytes(&[ADDI_R3_R3_5, BLR]));
        let long = decode_text(&text_bytes(&[ADDI_R3_R3_5, ADD_R3_R11_R3, BLR]));
        let s = insn_seq_similarity(&long, &short);
        // 2 of 2 target insns align exactly (addi, blr), 1 inserted `add` is a
        // gap → (1.0 + 1.0) / max(3,2) = 2/3.
        assert!((s - 2.0 / 3.0).abs() < 1e-9, "edit-distance align = 2/3, got {s}");
        // Empty vs non-empty is 0; both empty is 1.
        assert_eq!(insn_seq_similarity(&[], &short), 0.0);
        assert_eq!(insn_seq_similarity(&[], &[]), 1.0);
    }

    // Build a minimal 1-section COFF whose `.text` is `text` (so
    // `retrieval::text_section` finds it), with `tail` appended after the code
    // (the reloc/symbol region). Two such objs with the same `text` but different
    // `tail` have identical `.text` yet are not byte-exact.
    fn coff_with_text(text: &[u8], tail: &[u8]) -> ObjImage {
        let mut v = vec![0u8; 20]; // COFF header
        v[2] = 1; // NumberOfSections = 1 (LE u16)
        // nsym (offset 12), opt-hdr size (offset 16) both left 0.
        let rawptr = 60u32; // 20 header + 40 section header
        let mut sh = vec![0u8; 40];
        sh[..5].copy_from_slice(b".text");
        sh[16..20].copy_from_slice(&(text.len() as u32).to_le_bytes()); // SizeOfRawData
        sh[20..24].copy_from_slice(&rawptr.to_le_bytes()); // PointerToRawData
        v.extend_from_slice(&sh);
        v.extend_from_slice(text);
        v.extend_from_slice(tail);
        ObjImage::new(v)
    }

    // A scorer that mirrors `ReplayScorer`'s verdict split — REAL `ObjImage::diff`
    // for the terminal, `insn_text_similarity` for the gradient — but maps every
    // model to a FIXED obj, so it needs no toolchain. Used to pin the seam.
    struct FixedObjScorer {
        obj: ObjImage,
        target: ObjImage,
        compiles: usize,
    }
    impl Scorer for FixedObjScorer {
        fn judge(&mut self, _model: &IlModel) -> Judged {
            self.compiles += 1;
            if matches!(ObjImage::diff(&self.obj, &self.target), ObjDiff::Identical) {
                Judged::ByteExact
            } else {
                Judged::Fuzzy(insn_text_similarity(&self.obj, &self.target))
            }
        }
        fn compiles(&self) -> usize {
            self.compiles
        }
    }

    #[test]
    fn max_gradient_on_non_byte_exact_obj_does_not_terminate() {
        // The reviewer's filed residue: a candidate whose `.text` is
        // instruction-identical to the target (gradient == 1.0) but whose obj is
        // NOT byte-exact (a differing reloc/symbol byte) must NOT be a success —
        // only real byte-exactness terminates.
        let code = text_target();
        let target = coff_with_text(&code, &[0xAA, 0xBB, 0xCC, 0xDD]); // reloc tail A
        let cand = coff_with_text(&code, &[0xAA, 0xBB, 0xCC, 0xEE]); // reloc tail B

        // The seam: gradient is maximal, yet the objs are not byte-exact.
        assert_eq!(
            insn_text_similarity(&cand, &target),
            1.0,
            "identical `.text` must max the gradient"
        );
        assert_ne!(
            ObjImage::diff(&cand, &target),
            ObjDiff::Identical,
            "the objs differ in the reloc/symbol tail — not byte-exact"
        );

        // Drive the climber through that verdict split: it must never declare
        // success on the fuzzy 1.0. (Every judgement returns Fuzzy(1.0); none is
        // ByteExact, so no neighbor strictly improves → an honest LocalOptimum.)
        let mut scorer = FixedObjScorer {
            obj: cand,
            target,
            compiles: 0,
        };
        let seed = model_add_lit(5, false);
        let out = hill_climb(&seed, &MoveSet::default(), &mut scorer, &Budget::default());
        assert!(
            !out.solved,
            "a fuzzy 1.0 that is not byte-exact must NOT terminate: {out:?}"
        );
        assert_eq!(out.reason, StopReason::LocalOptimum);
        assert_eq!(out.best_fuzzy, 1.0, "the gradient did reach its max");
    }

    #[test]
    fn word_match_ratio_basics() {
        assert_eq!(word_match_ratio(&[], &[]), 1.0);
        assert_eq!(word_match_ratio(&[1, 2, 3, 4], &[1, 2, 3, 4]), 1.0);
        assert_eq!(word_match_ratio(&[1, 2, 3, 4], &[9, 9, 9, 9]), 0.0);
        // one of two words matches
        let r = word_match_ratio(&[1, 2, 3, 4, 5, 6, 7, 8], &[1, 2, 3, 4, 0, 0, 0, 0]);
        assert!((r - 0.5).abs() < 1e-9);
        // length mismatch penalized (1 word vs 2)
        let r = word_match_ratio(&[1, 2, 3, 4], &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert!((r - 0.5).abs() < 1e-9);
    }

    #[test]
    fn neighbors_are_in_scope_and_deduped() {
        let m = model_add_lit(5, false);
        let moves = MoveSet::default();
        let ns = moves.neighbors(&m);
        assert!(!ns.is_empty(), "expected a non-empty neighborhood");
        // Every neighbor round-trips (a refused edit is never emitted) and is
        // distinct from the seed and from each other by `.ex`.
        let seed_ex = m.encode().get("ex").unwrap().to_vec();
        let mut seen = BTreeSet::new();
        for (_label, cand) in &ns {
            let ex = cand.encode().get("ex").unwrap().to_vec();
            assert_ne!(ex, seed_ex, "a neighbor equals the seed");
            assert!(seen.insert(ex), "duplicate neighbor emitted");
        }
        // There is a widen move (the narrow literal → wide).
        assert!(ns.iter().any(|(l, _)| l.contains("widen")));
    }

    #[test]
    fn climber_recovers_a_widen_perturbation() {
        // Target = solution `a + 5` (narrow). Seed = widened literal (d=1). The
        // narrow move must recover the target byte-exact in one step.
        let solution = model_add_lit(5, false);
        let seed = perturb_once(&solution, Perturb::WidenLit).expect("has a lit");
        assert_ne!(
            seed.encode().get("ex"),
            solution.encode().get("ex"),
            "perturbation must change the seed"
        );
        let mut scorer = MockScorer::new(&solution);
        let out = hill_climb(&seed, &MoveSet::default(), &mut scorer, &Budget::default());
        assert!(out.solved, "d=1 widen must be recoverable: {out:?}");
        assert_eq!(out.reason, StopReason::Solved);
        assert!(out.steps <= 1, "widen recovery is one move");
    }

    #[test]
    fn climber_recovers_an_added_term_by_delete() {
        // Seed = solution + a redundant term; delete recovers it.
        let solution = model_add_lit(5, false);
        let seed = perturb_once(&solution, Perturb::AddTerm).expect("has an operand+op");
        let mut scorer = MockScorer::new(&solution);
        let out = hill_climb(&seed, &MoveSet::default(), &mut scorer, &Budget::default());
        assert!(out.solved, "added term must be removable: {out:?}");
    }

    #[test]
    fn budget_bounds_compiles_and_reports_failure() {
        // An unreachable target (different literal, value moves off) with a tiny
        // compile budget must stop honestly, not loop.
        let solution = model_add_lit(5, false);
        let seed = model_add_lit(9, true); // 2 edits away, value moves disabled
        let mut scorer = MockScorer::new(&solution);
        let budget = Budget {
            max_steps: 8,
            max_compiles: 6,
            restarts: 0,
            beam_width: 1,
        };
        let out = hill_climb(&seed, &MoveSet::length_only(), &mut scorer, &budget);
        assert!(!out.solved);
        assert!(scorer.compiles() <= 6, "compile budget must bound the run");
        assert!(matches!(
            out.reason,
            StopReason::CompilesExhausted | StopReason::LocalOptimum
        ));
    }

    #[test]
    fn climber_skips_rejects_cleanly() {
        // Mark every wide-literal candidate a reject; the climber must skip them
        // and still find another path — the value nudge 8 + (−3) = 5 recovers the
        // target (−3 is in the default nudge window).
        let solution = model_add_lit(5, false);
        let seed = model_add_lit(8, false);
        let mut scorer = MockScorer::new(&solution);
        // Reject any candidate carrying the wide-literal marker `80` after the
        // int-type — forces the value path rather than widen.
        scorer.reject_if_contains = Some(vec![0x86, 0x41, 0x74, 0x80]);
        let out = hill_climb(&seed, &MoveSet::default(), &mut scorer, &Budget::default());
        assert!(out.solved, "value nudge 8->5 recovers despite rejects: {out:?}");
    }

    #[test]
    fn already_solved_seed_is_zero_step_success() {
        let solution = model_add_lit(5, false);
        let mut scorer = MockScorer::new(&solution);
        let out = hill_climb(&solution, &MoveSet::default(), &mut scorer, &Budget::default());
        assert!(out.solved);
        assert_eq!(out.steps, 0);
    }

    #[test]
    fn perturb_drop_then_recover_by_insert() {
        // Solution `a + a` (a repeated operand); drop the trailing `+a` → seed
        // `a`; the insert move must put `+a` back. The dropped operand (`a`) is
        // still available in the seed body, so insert-recovery reconstructs it
        // byte-exact — the direction P0.6a E exercised (a genuinely-grown stream).
        let solution = model_add_aa();
        let seed = perturb_once(&solution, Perturb::DropTerm).expect("has a term");
        assert_ne!(
            seed.encode().get("ex"),
            solution.encode().get("ex"),
            "drop must shorten the seed"
        );
        let mut scorer = MockScorer::new(&solution);
        let out = hill_climb(&seed, &MoveSet::default(), &mut scorer, &Budget::default());
        assert!(out.solved, "dropped term must be reinsertable: {out:?}");
    }

    // ---- Part 1: register-renaming-tolerant operand credit -----------------

    fn add_word(d: u32, a: u32, b: u32) -> u32 {
        (31 << 26) | (d << 21) | (a << 16) | (b << 11) | (266 << 1)
    }
    fn mullw_word(d: u32, a: u32, b: u32) -> u32 {
        (31 << 26) | (d << 21) | (a << 16) | (b << 11) | (235 << 1)
    }

    #[test]
    fn register_tolerant_credit_beats_wrong_and_raw() {
        // A candidate that is correct up to a consistent temp-register rename
        // (`add r11,r4,r5` where the target has `add r3,r4,r5` — c2 recolored the
        // result temp when a term count changed) must earn FULL credit under the
        // bijection, strictly above (a) a wrong-opcode candidate and (b) the raw
        // renaming-blind score.
        let target = [add_word(3, 4, 5), BLR];
        let renamed = [add_word(11, 4, 5), BLR]; // r11↦r3 is a clean renaming
        let wrong = [mullw_word(3, 4, 5), BLR]; // right regs, wrong op

        let s_renamed = insn_seq_similarity(&renamed, &target);
        let s_wrong = insn_seq_similarity(&wrong, &target);

        // Consistent renaming ⇒ full structural credit (1.0 gradient — still not a
        // terminal; only a byte-exact obj terminates).
        assert!(
            (s_renamed - 1.0).abs() < 1e-9,
            "a consistent register renaming must earn full credit, got {s_renamed}"
        );
        // The raw per-instruction credit (renaming-blind) is what the bijection
        // beats: `add r11` vs `add r3` = 0.5 + 0.5*2/3 on the add, 1.0 on blr →
        // (0.8333 + 1.0)/2 = 0.9166 raw; the tolerant score (1.0) is strictly above.
        let raw = insn_similarity(&decode_ppc(renamed[0]), &decode_ppc(target[0]), None);
        assert!(raw < 1.0, "raw credit is partial (< 1.0): {raw}");
        assert!(
            s_renamed > s_wrong,
            "renamed-but-correct ({s_renamed}) must beat wrong-opcode ({s_wrong})"
        );
    }

    #[test]
    fn register_bijection_is_injective_not_any_matches_any() {
        // Guard against over-credit: `add r11,r11,r5` cannot be a renaming of
        // `add r3,r4,r5` (r11 would have to map to BOTH r3 and r4). The injective
        // bijection maps r11 to only one, so the score stays partial (< 1.0), not
        // a false full match.
        let target = [add_word(3, 4, 5), BLR];
        let ambiguous = [add_word(11, 11, 5), BLR];
        let s = insn_seq_similarity(&ambiguous, &target);
        assert!(
            s < 1.0,
            "a non-injective 'renaming' must NOT reach full credit, got {s}"
        );
    }

    // ---- Part 2: beam / restarts (escape a plateau) ------------------------

    // A deceptive-plateau scorer: byte-exact only on the exact target `.ex`;
    // EVERY other model scores a flat `0.5`. Greedy (width 1, needs a strict
    // improvement) therefore stalls at the seed — no single move improves — while
    // a beam that keeps non-improving candidates can still reach the byte-exact
    // target two moves away. Counts every judgement (a real compile stand-in).
    struct PlateauScorer {
        target_ex: Vec<u8>,
        compiles: usize,
    }
    impl Scorer for PlateauScorer {
        fn judge(&mut self, model: &IlModel) -> Judged {
            self.compiles += 1;
            if ex_bytes(model) == self.target_ex {
                Judged::ByteExact
            } else {
                Judged::Fuzzy(0.5)
            }
        }
        fn compiles(&self) -> usize {
            self.compiles
        }
    }

    fn plateau_setup() -> (IlModel, IlModel) {
        // Target = `a+5`; seed = `((a+5)+a)+a` (two redundant terms). The 2-delete
        // inverse reaches the target `.ex`, but no single delete improves the flat
        // gradient — the beam must take a non-improving step.
        let solution = model_add_lit(5, false);
        let seed = perturb(&solution, Perturb::AddTerm, 2).expect("d2 add-term site");
        (solution, seed)
    }

    #[test]
    fn greedy_stalls_but_beam_crosses_the_plateau() {
        let (solution, seed) = plateau_setup();
        let target_ex = ex_bytes(&solution);

        // Greedy (width 1): stalls — nothing strictly improves the flat 0.5.
        let mut g = PlateauScorer { target_ex: target_ex.clone(), compiles: 0 };
        let greedy = hill_climb(&seed, &MoveSet::length_only(), &mut g, &Budget::default());
        assert!(!greedy.solved, "greedy must stall on the plateau: {greedy:?}");
        assert_eq!(greedy.reason, StopReason::LocalOptimum);
        assert_eq!(greedy.steps, 0, "greedy takes no step (no improvement)");

        // Beam (wide): keeps non-improving candidates → reaches the byte-exact
        // target two moves away. best_fuzzy never exceeds 0.5, proving the solving
        // path went THROUGH a non-improving intermediate.
        let mut b = PlateauScorer { target_ex, compiles: 0 };
        let budget = Budget { max_steps: 4, max_compiles: 5000, restarts: 0, beam_width: 64 };
        let beam = beam_search(&seed, &MoveSet::length_only(), &mut b, &budget);
        assert!(beam.solved, "the beam must cross the plateau: {beam:?}");
        // Greedy stalled at 0 steps because no move improved the flat 0.5; the beam
        // reaches the byte-exact target in ≥ 2 steps on that SAME flat landscape —
        // so every step it took was necessarily non-improving. (best_fuzzy reports
        // 1.0 on a solve, the byte-exact terminal; it cannot witness the plateau —
        // the step-count contrast against greedy does.)
        assert!(beam.steps >= 2, "recovery is a two-move (non-improving) descent: {beam:?}");
    }

    #[test]
    fn beam_is_deterministic_and_budget_bounded() {
        let (solution, seed) = plateau_setup();
        let target_ex = ex_bytes(&solution);
        let budget = Budget { max_steps: 4, max_compiles: 5000, restarts: 0, beam_width: 64 };

        // Deterministic: two identical runs give identical outcomes (same solve,
        // steps, compiles, path) — no wall-clock, no RNG.
        let mut s1 = PlateauScorer { target_ex: target_ex.clone(), compiles: 0 };
        let r1 = beam_search(&seed, &MoveSet::length_only(), &mut s1, &budget);
        let mut s2 = PlateauScorer { target_ex: target_ex.clone(), compiles: 0 };
        let r2 = beam_search(&seed, &MoveSet::length_only(), &mut s2, &budget);
        assert_eq!(r1.solved, r2.solved);
        assert_eq!(r1.steps, r2.steps);
        assert_eq!(r1.compiles, r2.compiles);
        assert_eq!(r1.path, r2.path, "the beam path must be reproducible");

        // Budget-bounded: a tiny compile budget stops honestly, never overspends.
        let mut sb = PlateauScorer { target_ex, compiles: 0 };
        let tight = Budget { max_steps: 4, max_compiles: 3, restarts: 0, beam_width: 64 };
        let rb = beam_search(&seed, &MoveSet::length_only(), &mut sb, &tight);
        assert!(sb.compiles() <= 3, "compile budget must bound the beam");
        assert!(
            !rb.solved || rb.compiles <= 3,
            "an honest stop within budget: {rb:?}"
        );
    }

    // ---- Part 3: generative insert vocabulary ------------------------------

    #[test]
    fn generative_operands_regenerates_vanished_scope() {
        use c2_il::ExToken::*;
        // A hand-built token run: two formals (a, b) declared, but the body uses
        // only `a` and the literal 5 — `b` has vanished from the body. The
        // generative vocabulary must still offer `Load(b)` (a param in scope) plus
        // the small literal set, so a dropped `+b` or `+k` is reconstructable.
        let a = 0xE301u16;
        let b = 0xE401u16;
        let tokens = vec![
            Formals,
            Formal(a),
            Formal(b),
            Load(a),
            Lit { value: 5, wide: false },
            Add,
        ];
        let vocab = generative_operands(&tokens, &[1, 2, 5]);

        // Body operands are present (reuse case).
        assert!(vocab.contains(&Load(a)), "body operand a must be offered");
        assert!(vocab.contains(&Lit { value: 5, wide: false }));
        // The vanished param `b` is regenerated as a Load (the generative gain).
        assert!(
            vocab.contains(&Load(b)),
            "an in-scope param absent from the body must be loadable: {vocab:?}"
        );
        // The small literal vocabulary is present (a vanished `+k` is recoverable).
        assert!(vocab.contains(&Lit { value: 1, wide: false }));
        assert!(vocab.contains(&Lit { value: 2, wide: false }));
        // Deduplicated: `Load(a)` and `Lit 5` appear once despite being in both
        // the body and (5) the literal set.
        assert_eq!(vocab.iter().filter(|t| **t == Load(a)).count(), 1);
        assert_eq!(
            vocab.iter().filter(|t| **t == Lit { value: 5, wide: false }).count(),
            1
        );

        // Reuse-only enumeration does NOT offer the vanished param — the contrast
        // that motivates the generative set.
        let reuse = distinct_operands(&tokens);
        assert!(!reuse.contains(&Load(b)), "reuse-only cannot regenerate b");
    }

    // ---- Part 4: per-function-decomposed gradient (the plateau fix) --------

    #[test]
    fn split_by_blr_partitions_at_returns() {
        // Two functions, each ending in a `blr`; the split yields exactly two
        // segments, each including its terminating `blr`.
        let words = [ADDI_R3_R3_5, BLR, ADDI_R11_R3_5, ADD_R3_R11_R3, BLR];
        let segs = split_by_blr(&words);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0], vec![ADDI_R3_R3_5, BLR]);
        assert_eq!(segs[1], vec![ADDI_R11_R3_5, ADD_R3_R11_R3, BLR]);
        // A trailing run with no final `blr` becomes its own segment (no drop).
        let tail = split_by_blr(&[ADDI_R3_R3_5, BLR, ADDI_R11_R3_5]);
        assert_eq!(tail.len(), 2);
        assert_eq!(tail[1], vec![ADDI_R11_R3_5]);
    }

    #[test]
    fn per_fn_gradient_lifts_a_masked_function_edit() {
        // Two functions. fn0 is large and INTACT in both candidate and target
        // (10 identical insns + blr); fn1 is small and under edit. The
        // whole-`.text` gradient lets the big intact fn0 mask fn1's progress; the
        // per-function gradient scores each function with equal weight, so fixing
        // fn1 moves the score far more — the plateau fix.
        let mut fn0 = vec![ADDI_R3_R3_5; 10];
        fn0.push(BLR);
        let fn1_target = [ADDI_R3_R3_5, BLR]; // `a+5`
        let fn1_wrong = [ADD_R3_R11_R3, BLR]; // wrong opcode (add, not addi)

        let target: Vec<u32> = fn0.iter().copied().chain(fn1_target).collect();
        let seed: Vec<u32> = fn0.iter().copied().chain(fn1_wrong).collect();
        let fixed: Vec<u32> = fn0.iter().copied().chain(fn1_target).collect();

        let perfn = |c: &[u32]| insn_seq_similarity_perfn(c, &target, 2);
        let whole = |c: &[u32]| insn_seq_similarity(c, &target);

        // A correct edit to fn1 raises the per-function score even with fn0
        // intact, and reaches a full 1.0 on the exact match.
        assert!(perfn(&seed) < perfn(&fixed), "the fn1 fix must raise per-fn");
        assert!((perfn(&fixed) - 1.0).abs() < 1e-9, "exact match is 1.0");

        // The plateau fix: the same edit gets a STRICTLY larger gradient step
        // under the per-function decomposition than under the whole-`.text`
        // score (where the 10-insn intact fn0 dilutes it).
        let d_perfn = perfn(&fixed) - perfn(&seed);
        let d_whole = whole(&fixed) - whole(&seed);
        assert!(
            d_perfn > d_whole,
            "per-fn must give the masked edit a stronger gradient: \
             Δperfn={d_perfn} vs Δwhole={d_whole}"
        );
    }

    #[test]
    fn per_fn_gradient_falls_back_when_splits_disagree() {
        // A 2-segment candidate vs a 1-segment target (or a wrong nfns hint) must
        // NOT align mismatched segments — it falls back to the honest whole-stream
        // score rather than over-/under-crediting a bad split.
        let a = [ADDI_R3_R3_5, BLR, ADDI_R11_R3_5, BLR]; // 2 segments
        let b = [ADDI_R3_R3_5, BLR]; // 1 segment
        assert_eq!(
            insn_seq_similarity_perfn(&a, &b, 2),
            insn_seq_similarity(&a, &b),
            "unequal segment counts fall back to whole-stream"
        );
        // A correct nfns but mismatched split (candidate has 1 seg, hint says 2)
        // also falls back.
        assert_eq!(
            insn_seq_similarity_perfn(&b, &b, 2),
            insn_seq_similarity(&b, &b),
        );
    }

    #[test]
    fn per_fn_gradient_max_does_not_terminate() {
        // The terminal seam for the per-function gradient: two objs with identical
        // `.text` but a differing reloc/symbol tail score a full 1.0 gradient yet
        // are NOT byte-exact — a maxed per-fn gradient is never a success (only
        // `ObjImage::diff == Identical` terminates).
        let code = text_target(); // `addi r3,r3,5 ; blr` — one `blr` segment
        let target = coff_with_text(&code, &[0xAA, 0xBB, 0xCC, 0xDD]);
        let cand = coff_with_text(&code, &[0xAA, 0xBB, 0xCC, 0xEE]);
        assert_eq!(
            insn_text_similarity_perfn(&cand, &target, 1),
            1.0,
            "identical `.text` maxes the per-fn gradient"
        );
        assert_ne!(
            ObjImage::diff(&cand, &target),
            ObjDiff::Identical,
            "the objs differ in the reloc tail — not byte-exact"
        );
    }

    // ---- Part 5: retrieval seed selection (self / twin exclusion) ----------

    fn mk_item(id: &str, text: &[u8]) -> Item {
        let (hist, norm) = retrieval::byte_histogram(text);
        Item {
            id: id.into(),
            src_key: corpus::sha256_hex(id.as_bytes()), // unique per row
            text_key: corpus::sha256_hex(text),
            full_key: format!("full-{id}"),
            hist,
            norm,
            text_len: text.len(),
            nsym: 0,
            obj_len: text.len(),
        }
    }

    #[test]
    fn select_seed_flags_a_twin_as_retrieval_trivial() {
        // The corpus holds the target, an exact-`.text` twin (different source),
        // and two distinct-code rows. The nearest non-self neighbor is the twin
        // (cosine 1.0) → a trivial retrieval solve, never fed to the search.
        let items = vec![
            mk_item("q", &[1, 2, 3, 4]),
            mk_item("tw", &[1, 2, 3, 4]), // twin (identical .text)
            mk_item("nr", &[1, 2, 3, 5]), // near
            mk_item("fr", &[9, 9, 9, 9]), // far
        ];
        match select_seed(&items[0], &items) {
            SeedChoice::RetrievalTrivial { twin_id } => assert_eq!(twin_id, "tw"),
            other => panic!("expected a twin trivial, got {other:?}"),
        }
    }

    #[test]
    fn select_seed_picks_nearest_non_self_non_twin() {
        // No twin present: the seed is the nearest non-self neighbor, and it is
        // never the target's own row.
        let items = vec![
            mk_item("q", &[1, 2, 3, 4]),
            mk_item("nr", &[1, 2, 3, 5]), // closest distinct code
            mk_item("fr", &[9, 9, 9, 9]), // far
        ];
        match select_seed(&items[0], &items) {
            SeedChoice::Seed { index } => {
                assert_ne!(items[index].id, "q", "must never seed from self");
                assert_eq!(items[index].id, "nr", "nearest distinct row is the seed");
            }
            other => panic!("expected a Seed, got {other:?}"),
        }
    }

    // =====================================================================
    // Stuck-dc3 near-miss lane — decode stress test + codec/move blocker
    // probe (il-witness STUCK_DC3_ATTEMPT). Toolchain-gated: SKIPs cleanly
    // when wibo/cl.exe/c2.dll/strace are absent. Run with:
    //   cargo test -p c2-harness stuck_dc3 -- --nocapture --test-threads=1
    // =====================================================================

    /// Write `src` to a fresh single-function `.cpp` under a scratch dir and
    /// return its path. The scratch dir is created under the system tempdir,
    /// keyed by test name so parallel tests do not collide.
    fn scratch_cpp(dir: &Path, name: &str, src: &str) -> PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        let p = dir.join(format!("{name}.cpp"));
        std::fs::write(&p, src).unwrap();
        p
    }

    /// Primary-opcode histogram of a decoded `.text`, plus whether each primary
    /// is *specially* decoded (op 18 branch, 19 XL, 31 XO) or grades through the
    /// coarse **D-form default** (everything else).
    fn opcode_report(words: &[u32]) -> Vec<(u8, usize, bool)> {
        let mut counts: BTreeMap<u8, usize> = BTreeMap::new();
        for &w in words {
            *counts.entry(decode_ppc(w).primary).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .map(|(p, n)| (p, n, matches!(p, 18 | 19 | 31)))
            .collect()
    }

    /// STEP 1 — decode stress test on REAL non-straight-line bodies.
    ///
    /// Compiles single-function C++ that exercises the opcode classes real dc3
    /// bodies use (mullw, shift/mask→rlwinm, compare+branch, memory load/store,
    /// float), decodes each obj's `.text`, and reports (a) the primary-opcode
    /// coverage — which primaries are specially decoded vs graded by the D-form
    /// default — and (b) whether the instruction-aware gradient DISCRIMINATES a
    /// 1-instruction difference (a graded score strictly between the wrong-opcode
    /// floor and 1.0) on each real body. Terminal correctness is unaffected —
    /// this only probes the gradient.
    #[test]
    fn stuck_dc3_step1_decode_stress() {
        let Some(tc) = Toolchain::locate() else {
            eprintln!("SKIP stuck_dc3_step1: toolchain absent");
            return;
        };
        let dir = std::env::temp_dir().join("c2rs_stuck_dc3_step1");
        let _ = std::fs::remove_dir_all(&dir);

        // (name, source) — each a single leaf function; one opcode class each.
        let bodies: &[(&str, &str)] = &[
            ("mul_int", "int f(int a,int b){return a*b;}"), // mullw (op31)
            ("shift_mask", "int f(int x,int n){return (x<<n)&0xff;}"), // slw+rlwinm
            ("select_max", "int f(int a,int b){return a>b?a:b;}"), // cmpw + branch/isel
            ("ptr_load", "int f(const int*p){return p[0]+p[2];}"), // lwz (op32)
            ("ptr_store", "void f(int*p,int v){p[0]=v;p[2]=v;}"), // stw (op36)
            // Box::Volume shape — float subtract + float multiply chain.
            (
                "float_vol",
                "float f(float ax,float ay,float az,float bx,float by,float bz){\
                 return (bx-ax)*(by-ay)*(bz-az);}",
            ),
        ];

        println!("\n=== STEP 1: decode stress on real non-straight-line bodies ===");
        for (name, src) in bodies {
            let cpp = scratch_cpp(&dir, name, src);
            let obj = match tc.compile_obj(&cpp, &dir.join(format!("{name}.obj"))) {
                Ok(o) => o,
                Err(e) => {
                    println!("  {name:<12} COMPILE-FAIL: {e}");
                    continue;
                }
            };
            let norm = obj.normalized();
            let (text, _) = text_section(&norm);
            let words = decode_text(text);
            let hist = opcode_report(&words);
            let special: Vec<String> = hist
                .iter()
                .filter(|(_, _, s)| *s)
                .map(|(p, n, _)| format!("op{p}x{n}"))
                .collect();
            let dform: Vec<String> = hist
                .iter()
                .filter(|(_, _, s)| !*s)
                .map(|(p, n, _)| format!("op{p}x{n}"))
                .collect();
            println!(
                "  {name:<12} {} insns | special-decode: [{}] | D-form-default: [{}]",
                words.len(),
                special.join(" "),
                dform.join(" "),
            );

            // Gradient discrimination: mutate ONE middle instruction's rA field
            // and confirm the instruction-aware similarity grades it strictly
            // between a wholly-different body (0-ish floor) and identity (1.0).
            if words.len() >= 3 {
                let mid = words.len() / 2;
                let mut cand = words.clone();
                cand[mid] ^= 1 << 16; // flip rA low bit (bits 11-15)
                let s_self = insn_seq_similarity(&words, &words);
                let s_mut = insn_seq_similarity(&cand, &words);
                // A fully-disjoint body (all zeroed words) as the floor.
                let floor_body = vec![0u32; words.len()];
                let s_floor = insn_seq_similarity(&floor_body, &words);
                let graded = s_mut > s_floor && s_mut < s_self;
                println!(
                    "               gradient: self={s_self:.4} 1insn-diff={s_mut:.4} floor={s_floor:.4}  discriminates={graded}",
                );
                assert!(
                    (s_self - 1.0).abs() < 1e-9,
                    "{name}: identity must score 1.0"
                );
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// STEP 2/3 — the codec + move-set blocker on the real near-miss classes.
    ///
    /// The stuck-dc3 near-miss cohort (frontier: register-swap / control-flow /
    /// offset-swap / float / commutative-order) is dominated by float math,
    /// struct-member memory access, and branches. This probe captures a
    /// `Box::Volume`-shaped float body and an offset-swap-shaped memory body
    /// through the REAL toolchain, parses the IL, and shows the K3a editor has
    /// NO editable neighborhood on them (`function_tokens` → OpaqueFunctionBody
    /// and/or `MoveSet::neighbors` empty) — so the IL-space search has an empty
    /// action space and cannot make a single move. Contrasted with an in-class
    /// int-arithmetic body, which DOES yield moves.
    #[test]
    fn stuck_dc3_step2_codec_move_blocker() {
        let Some(tc) = Toolchain::locate() else {
            eprintln!("SKIP stuck_dc3_step2: toolchain absent");
            return;
        };
        if !tc.has_strace() {
            eprintln!("SKIP stuck_dc3_step2: strace absent (IL capture needs it)");
            return;
        }
        let dir = std::env::temp_dir().join("c2rs_stuck_dc3_step2");
        let _ = std::fs::remove_dir_all(&dir);

        // (name, source, in_class) — in_class = the codec's straight-line int
        // arithmetic family (expected to yield moves); the others mirror real
        // near-miss classes (expected: no editable neighborhood).
        let cases: &[(&str, &str, bool)] = &[
            // In-class baseline: int add chain (the MVP family).
            ("int_add3", "int f(int a,int b,int c){return a+b+c;}", true),
            // Box::Volume shape: float subtract + multiply (commutative-order floor).
            (
                "float_vol",
                "float f(float ax,float ay,float az,float bx,float by,float bz){\
                 return (bx-ax)*(by-ay)*(bz-az);}",
                false,
            ),
            // Offset-swap shape: struct-member/memory arithmetic.
            ("offset_swap", "int f(const int*p){return p[0]*p[2]-p[1];}", false),
        ];

        println!("\n=== STEP 2/3: codec + move-set action space on near-miss classes ===");
        let mut in_class_had_moves = false;
        let mut out_class_had_moves = false;
        for (name, src, in_class) in cases {
            let cpp = scratch_cpp(&dir, name, src);
            let cap = match tc.capture_reference(&cpp, &dir.join(format!("cap_{name}"))) {
                Ok(c) => c,
                Err(e) => {
                    println!("  {name:<12} CAPTURE-FAIL: {e}");
                    continue;
                }
            };
            let model = match IlModel::parse(&cap.bundle) {
                Ok(m) => m,
                Err(e) => {
                    println!("  {name:<12} IL-PARSE-FAIL: {e}");
                    continue;
                }
            };
            let nfns = model.ex_function_count();
            let mut editable = 0usize;
            let mut opaque = 0usize;
            for fi in 0..nfns {
                match model.function_tokens(fi) {
                    Ok(toks) => {
                        // Editable iff it holds a run of arithmetic operands/ops
                        // the move set can act on (Load/Lit + Add/Sub/Mul).
                        let has_arith = toks.iter().any(is_binop)
                            && toks.iter().any(is_operand);
                        if has_arith {
                            editable += 1;
                        }
                    }
                    Err(_) => opaque += 1,
                }
            }
            let neighbors = MoveSet::default().neighbors(&model);
            println!(
                "  {name:<12} in_class={in_class} fns={nfns} arith-editable={editable} opaque-body={opaque} | K3a neighbors={}",
                neighbors.len(),
            );
            if *in_class {
                in_class_had_moves = !neighbors.is_empty();
            } else if !neighbors.is_empty() {
                out_class_had_moves = true;
            }
        }
        println!(
            "  VERDICT: in-class body yields moves={in_class_had_moves}; any out-of-class body yields moves={out_class_had_moves}"
        );
        // The finding: the in-class family is searchable; the real near-miss
        // classes (float/memory) present an EMPTY K3a action space. This is not
        // asserted hard (a future codec K2/K3b widening could change it — that is
        // exactly the scoped remaining work), but is printed as the headline.
        let _ = std::fs::remove_dir_all(&dir);
    }
}
