//! Float-leaf codec-widening gate — the **non-zero K3a neighbors** assertion.
//!
//! The stuck-dc3 attempt proved the near-miss lane was blocked UPSTREAM of
//! search: a `Box::Volume`-class float leaf (float arithmetic over struct-member
//! loads) parsed to typed tokens with **interleaved opaque runs**, so
//! `IlModel::function_tokens` failed `OpaqueFunctionBody` and the move set found
//! **zero** K3a neighbors — an empty action space.
//!
//! The float-leaf codec widening (`c2-il::codec`) decoded those opaque runs
//! (float/pointer loads, the `MEMBER_PTR`/`DEREF` member-access idiom,
//! `CAST`/`STORE`, the float result-type) into typed `ExToken`s, so the body is
//! now a **contiguous typed run**. This test is the search-side half of the gate
//! (the codec-side half — round-trip + token-addressability — lives in
//! `c2-il`'s `codec.rs`): it builds the captured float-leaf model and asserts
//! `MoveSet::neighbors` now yields **≥ 1** editable neighbor. Portable (no
//! toolchain): the neighborhood enumeration is a pure function of the model.

use c2_harness::search::MoveSet;
use c2_il::{ExToken, IlBundle, IlModel};

/// The real single-function `.ex` segment of the `Box::Volume` reduction
/// `float volf(const V* a,const V* b){ float x=a->x-b->x; float y=a->y-b->y;
/// float z=a->z-b->z; return x*y*z; }` — captured live (16.00.11886.00,
/// `/Bd /d2nop /Ox /GS- /c`) from the `4F 1F` marker. Byte-identical to the
/// `VOLF_SEGMENT` fixture in `codec.rs` (the codec's round-trip test owns the
/// provenance); duplicated here because that fixture is a private test const.
/// PROV[O] a verbatim `.ex` SEGMENT captured from a live 16.00.11886.00 compile of a `Box::Volume`-class float leaf — the same capture `c2_il::codec::FLOAT_TYPE` cites. A transcription of real toolchain output.
const VOLF_SEGMENT: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D, 0x66,
    0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18, 0x01, 0x00,
    0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D, 0x08, 0x00, 0x0F,
    0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xEF, 0x09, 0x46, 0x2D, 0xEE, 0x09,
    0x2D, 0xED, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x03, 0x26, 0xF1, 0x09, 0xB9, 0xED, 0x09,
    0x86, 0x43, 0x82, 0x20, 0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0x86, 0x43, 0x86, 0x20, 0x30, 0xA6,
    0x45, 0x85, 0x20, 0xB9, 0xEE, 0x09, 0x86, 0x43, 0x82, 0x20, 0x33, 0x86, 0x41, 0x74, 0x00, 0x27,
    0x86, 0x43, 0x86, 0x20, 0x30, 0xA6, 0x45, 0x85, 0x20, 0x03, 0x2C, 0x86, 0x45, 0x40, 0x00, 0x32,
    0x86, 0x45, 0x40, 0x4B, 0x4F, 0x01, 0x04, 0x26, 0xF2, 0x09, 0xB9, 0xED, 0x09, 0x86, 0x43, 0x82,
    0x20, 0x33, 0x86, 0x41, 0x74, 0x04, 0x27, 0x86, 0x43, 0x86, 0x20, 0x30, 0xA6, 0x45, 0x85, 0x20,
    0xB9, 0xEE, 0x09, 0x86, 0x43, 0x82, 0x20, 0x33, 0x86, 0x41, 0x74, 0x04, 0x27, 0x86, 0x43, 0x86,
    0x20, 0x30, 0xA6, 0x45, 0x85, 0x20, 0x03, 0x2C, 0x86, 0x45, 0x40, 0x00, 0x32, 0x86, 0x45, 0x40,
    0x4B, 0x4F, 0x01, 0x05, 0x26, 0xF3, 0x09, 0xB9, 0xED, 0x09, 0x86, 0x43, 0x82, 0x20, 0x33, 0x86,
    0x41, 0x74, 0x08, 0x27, 0x86, 0x43, 0x86, 0x20, 0x30, 0xA6, 0x45, 0x85, 0x20, 0xB9, 0xEE, 0x09,
    0x86, 0x43, 0x82, 0x20, 0x33, 0x86, 0x41, 0x74, 0x08, 0x27, 0x86, 0x43, 0x86, 0x20, 0x30, 0xA6,
    0x45, 0x85, 0x20, 0x03, 0x2C, 0x86, 0x45, 0x40, 0x00, 0x32, 0x86, 0x45, 0x40, 0x4B, 0x4F, 0x01,
    0x06, 0xB9, 0xF1, 0x09, 0x86, 0x45, 0x40, 0xB9, 0xF2, 0x09, 0x86, 0x45, 0x40, 0x04, 0xB9, 0xF3,
    0x09, 0x86, 0x45, 0x40, 0x04, 0x41, 0x86, 0x45, 0x40, 0x3A, 0xF0, 0x09, 0x4F, 0x01, 0x07, 0x54,
    0x02, 0x29, 0xF0, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F,
    0x01, 0x08, 0x4D,
];

fn volf_model() -> IlModel {
    let mut ex = c2_il::EX_MAGIC.to_vec();
    ex.extend_from_slice(&[0x00; 12]); // opaque header pad
    ex.extend_from_slice(VOLF_SEGMENT);
    let mut bundle = IlBundle::new("_CL_volf");
    bundle.set("ex", ex);
    IlModel::parse(&bundle).expect("volf float-leaf bundle round-trips")
}

#[test]
fn float_leaf_body_yields_nonzero_k3a_neighbors() {
    let model = volf_model();
    // Precondition: the body is token-addressable (the codec widening made it a
    // contiguous typed run — the stuck-dc3 blocker was that this failed).
    assert_eq!(model.ex_function_count(), 1);
    assert!(
        model.function_tokens(0).is_ok(),
        "float-leaf body must be a contiguous typed run (no OpaqueFunctionBody)"
    );

    // THE GATE: the move set now enumerates ≥ 1 K3a neighbor for the float leaf
    // (was exactly zero before the widening — the empty action space).
    let neighbors = MoveSet::default().neighbors(&model);
    assert!(
        !neighbors.is_empty(),
        "float-leaf body must yield NON-ZERO K3a neighbors (had {} )",
        neighbors.len()
    );

    // Every neighbor is a real, distinct, re-parseable candidate model (the
    // move set only emits models whose fail-closed splice succeeded).
    for (_label, cand) in &neighbors {
        IlModel::parse(&cand.encode()).expect("each neighbor re-parses (fail-closed splice)");
    }
}

/// **Piece A.** A `FloatLoad` term is now a **delete anchor** on the `volf` body.
/// `is_operand` covers `FloatLoad` (mirroring int `Load`), so the length-move
/// `term_delete` — which anchors on `is_operand(tokens[i]) && is_binop(tokens[i+1])`
/// — fires at a `FloatLoad , MUL` site. Before the one-liner, `FloatLoad` was not
/// an operand and the float leaf had *zero* delete anchors.
#[test]
fn floatload_is_a_delete_anchor_on_volf() {
    let model = volf_model();
    let tokens = model.function_tokens(0).expect("token-addressable");
    // There IS a `FloatLoad` immediately followed by a binop (the return's
    // `FloatLoad(x) FloatLoad(y) MUL …` — position of the second FloatLoad).
    let has_floatload_before_binop = (0..tokens.len().saturating_sub(1)).any(|i| {
        matches!(tokens[i], ExToken::FloatLoad(_))
            && matches!(tokens[i + 1], ExToken::Sub | ExToken::Mul)
    });
    assert!(
        has_floatload_before_binop,
        "fixture must have a `FloatLoad , binop` site for the anchor test"
    );

    // Every `del term@i` neighbor's anchor index `i` must be an operand; assert at
    // least one anchors a `FloatLoad` — i.e. the codec's float leaves are now real
    // delete anchors (the Piece-A win).
    let mut floatload_anchored = 0usize;
    for (label, _cand) in MoveSet::default().neighbors(&model) {
        if let Some(rest) = label.strip_prefix("fn0 del term@") {
            let i: usize = rest.parse().expect("del-term label carries an index");
            if matches!(tokens[i], ExToken::FloatLoad(_)) {
                floatload_anchored += 1;
            }
        }
    }
    assert!(
        floatload_anchored >= 1,
        "a FloatLoad must be a delete anchor on the volf body (had {floatload_anchored})"
    );
}

/// **Piece B on the float class.** The opt-in MUL-reorder move emits the
/// operand-swapped ordering of the return's inner `FloatLoad x , FloatLoad y , MUL`
/// as a d=1 neighbor — and ONLY when opted in. The default move set (reorder off)
/// emits no `mul-swap`, and the two share every OTHER neighbor (the reorder is
/// purely additive).
#[test]
fn float_mul_reorder_is_opt_in_and_generated_on_volf() {
    let model = volf_model();

    let default_ns = MoveSet::default().neighbors(&model);
    assert!(
        !default_ns.iter().any(|(l, _)| l.contains("mul-swap")),
        "the DEFAULT move set must NOT emit a mul-swap (opt-in only)"
    );

    let reorder_ns = MoveSet::default().with_mul_reorder().neighbors(&model);
    let swaps: Vec<&String> = reorder_ns
        .iter()
        .map(|(l, _)| l)
        .filter(|l| l.contains("mul-swap"))
        .collect();
    assert!(
        !swaps.is_empty(),
        "with_mul_reorder must emit ≥1 float mul-swap on the volf return product"
    );

    // Purely additive: enabling the move only ADDS mul-swap neighbors.
    assert_eq!(
        reorder_ns.len(),
        default_ns.len() + swaps.len(),
        "the reorder move is additive to the default neighborhood"
    );

    // The swap actually reorders a MUL's two FloatLoad leaves: the produced
    // candidate re-parses and differs from the seed only by the swap.
    for (_l, cand) in reorder_ns.iter().filter(|(l, _)| l.contains("mul-swap")) {
        let toks = cand.function_tokens(0).expect("swapped body token-addressable");
        assert_eq!(toks.iter().filter(|t| matches!(t, ExToken::Mul)).count(), 2);
        assert_ne!(
            cand.encode().get("ex"),
            model.encode().get("ex"),
            "a mul-swap must change the `.ex`"
        );
    }
}
