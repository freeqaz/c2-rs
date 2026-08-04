use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use c2_il::{ExToken, IlModel};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::Toolchain;

use crate::corpus;
use crate::retrieval::{self, text_section, Item};

use super::engine::{beam_search, ex_bytes, hill_climb, Budget, Judged, Scorer, StopReason};
use super::from_seed::{select_seed, SeedChoice};
use super::moves::{distinct_operands, generative_operands, is_binop, is_operand, MoveSet};
use super::perturb::{perturb, perturb_once, Perturb};
use super::similarity::{
    decode_ppc, decode_text, insn_seq_similarity, insn_seq_similarity_perfn, insn_similarity,
    insn_text_similarity, insn_text_similarity_perfn, split_by_blr, word_match_ratio,
};

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
// probe (the stuck-dc3 near-miss investigation). Toolchain-gated: SKIPs cleanly
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
    // Per-process scratch: a FIXED name here is a cross-process race —
    // one run's `remove_dir_all` deletes another's working tree
    // mid-compile, which fails as a mysterious capture error
    // (roadmap #55, and the same shape as the c2host stub race).
    let dir = std::env::temp_dir().join(format!(
        "c2rs_stuck_dc3_step1-{}",
        std::process::id()
    ));
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
    // Per-process scratch: a FIXED name here is a cross-process race —
    // one run's `remove_dir_all` deletes another's working tree
    // mid-compile, which fails as a mysterious capture error
    // (roadmap #55, and the same shape as the c2host stub race).
    let dir = std::env::temp_dir().join(format!(
        "c2rs_stuck_dc3_step2-{}",
        std::process::id()
    ));
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
