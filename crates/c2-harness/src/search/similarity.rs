use std::collections::{BTreeMap, BTreeSet};

use c2_obj::ObjImage;

use crate::retrieval::text_section;

// ===========================================================================
// Gradient — the `.text` fuzzy score (search guide ONLY, never terminal)
// ===========================================================================

/// PPC-word (4-byte) match ratio between a candidate obj and the target, over
/// their COFF `.text` sections. `1.0` iff the emitted code matches word-for-word
/// (which, combined with matching relocs/headers, is the byte-exact case the
/// terminal check confirms separately); `0.0` on disjoint code.
///
/// `.text`-only by design (per P1.3): the full obj embeds its
/// `/Fo` path in `S_OBJNAME`, so a whole-obj ratio would be path-dominated. The
/// gradient scores the *code*; the terminal success check is full
/// timestamp-normalized byte equality (see [`Judged`](super::Judged)). Objs are compared on
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
pub(super) fn word_match_ratio(a: &[u8], b: &[u8]) -> f64 {
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
/// PROV[F] a fitted SCORING WEIGHT, chosen so the ordering is monotone: strictly above 'wrong opcode' (<= 0.15) and strictly below a full match. Nothing measured 0.5; it is a parameter of this instrument's rule and it has an off-sample failure mode (a pair whose true similarity the ordering inverts).
const OPCODE_WEIGHT: f64 = 0.5;

/// A PPC instruction word decoded down to the fields the gradient compares: an
/// **opcode identity** (`opkey`, the primary opcode plus the extended opcode for
/// the XO/XL/branch forms) and an ordered list of operand fields. Deliberately
/// coarse — enough to grade "same opcode, which operands agree", not a full
/// disassembler.
#[derive(Clone, Debug)]
pub(super) struct PpcInsn {
    raw: u32,
    /// Primary opcode, bits 0-5 (IBM convention, bit 0 = MSB).
    pub(super) primary: u8,
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
pub(super) fn decode_ppc(word: u32) -> PpcInsn {
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
pub(super) fn insn_similarity(a: &PpcInsn, b: &PpcInsn, phi: Option<&BTreeMap<u32, u32>>) -> f64 {
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
pub(super) fn decode_text(text: &[u8]) -> Vec<u32> {
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
pub(super) fn insn_seq_similarity(a: &[u32], b: &[u32]) -> f64 {
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
/// [`Judged`](super::Judged)). Decodes each obj's COFF `.text` into PPC instruction words and
/// scores them with [`insn_seq_similarity`], so a move that fixes an opcode or an
/// operand field scores strictly higher than one that does not. `.text`-only for
/// the same path-freeness reason as [`fuzzy_text`] (the full obj embeds its `/Fo`
/// path); objs are compared on their timestamp-normalized bytes.
///
/// **Reconciliation with the terminal:** this can reach `1.0` on an obj that is
/// NOT byte-exact — the `.text` decode is blind to relocations, the symbol table,
/// and `.debug$S`, so two objs with identical code but differing tail bytes score
/// `1.0` here yet compare `Differs` under [`ObjImage::diff`]. A `1.0` gradient is
/// therefore NEVER a success; only [`Judged::ByteExact`](super::Judged::ByteExact) terminates.
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
/// PROV[S] the PowerPC `blr` instruction word `0x4E800020` (`bclr 20,0,0`), fixed by the ISA. That every function in this class returns via exactly one `blr` is an observation; the WORD is the architecture's.
const BLR_WORD: u32 = 0x4E80_0020;

/// Split a decoded `.text` word stream into per-function segments at each `blr`
/// terminator. Each `blr` ends its segment (and is included in it); a trailing
/// run with no final `blr` becomes a final segment so no words are dropped.
pub(super) fn split_by_blr(words: &[u32]) -> Vec<Vec<u32>> {
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
pub(super) fn insn_seq_similarity_perfn(a: &[u32], b: &[u32], nfns: usize) -> f64 {
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
