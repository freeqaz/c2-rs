//! **The GATE's own first refusal, by name** — lane `w-vec`, board **#2500**.
//!
//! # What this pins that no fixture verdict can
//!
//! A `vocab-gap` verdict is one bit. `IlBundle::decode_causes` decomposes it
//! into eleven named causes and had **no caller in `c2-harness`** from the day
//! lane `w-vocab` built it until this one — so every conversion price ever taken
//! off a `vocab-gap` row was taken without knowing which gate fired.
//! `docs/CEILING.md` §11.4 item 8 is the rule that says to quote the gate's
//! number; this is the instrument that makes it a field rather than a scratch
//! patch.
//!
//! The concrete cost of not having it: `src/system/math/vec.cpp` was
//! commissioned as *"`_fltused` plus seven non-instruction sections"*, and the
//! gate's first stop on it is **`gl-stop-26-introduced`** — a `.gl` walk stop,
//! four mechanisms upstream of either half of that price.
//!
//! # Why the cells are the shipped fixtures, `include_str!`-ed
//!
//! `w-fence2` §5.1's rule: a cell that re-types its subject grades a copy. The
//! three sources below are read out of `fixtures/cpp/` so a fixture cannot drift
//! from the assertion that claims to grade it.
//!
//! `SKIP: toolchain absent` when there is no toolchain, like every other
//! integration test here.

use std::path::PathBuf;

use c2_reference::Toolchain;

/// The workload's own profile minus the `/I` paths a standalone cell cannot
/// use. **`/O1`, deliberately**: it implies `/Gy`, which is the regime the
/// 878-TU scan lives in and the one `vec.obj`'s COMDAT `.text` shape belongs to.
/// At `/Ox` these same three sources produce a different section order and a
/// different `.ex`, so a cell graded there would not be grading `vec.cpp`'s
/// question.
const FLAGS: [&str; 8] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/O1", "/Oi", "/EHsc",
];

fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-gatecause-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Capture one source at [`FLAGS`] and return its `DecodeCauses`.
fn causes(tc: &Toolchain, tag: &str, body: &str) -> c2_il::func::DecodeCauses {
    let dir = work(tag);
    let cpp = dir.join(format!("{tag}.cpp"));
    std::fs::write(&cpp, body).unwrap();
    let flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
    let src = c2_reference::to_wibo_path(&cpp);
    let cap = tc
        .capture_reference_with(&src, &dir, &flags, None)
        .unwrap_or_else(|e| panic!("cell `{tag}`: capture failed: {e}"));
    cap.bundle.decode_causes()
}

/// **The three `w-vec` fixtures stop at three DIFFERENT places, and the
/// difference is the whole price of `vec.cpp`.**
///
/// A single refusal cell would have made the TU look one repair from a match.
/// It is at least five, and this is the executable form of that claim: the
/// positive decodes, and the two negatives are refused by causes neither of
/// which implies the other.
#[test]
fn the_three_vec_cells_stop_at_three_different_gates() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };

    // The control. It decodes, so `causes` must be EMPTY — `c2_il`'s own
    // invariant is `causes.is_empty() == decodes`, and asserting it here is what
    // says the two negatives below are refusals of their added field and not of
    // the shared body.
    let pos = causes(
        &tc,
        "pos",
        include_str!("../../../fixtures/cpp/wvec_float_store_leaf.cpp"),
    );
    assert!(
        pos.decodes && pos.causes.is_empty() && pos.first.is_none(),
        "the positive must decode with no cause: {pos:?}"
    );
    assert_eq!(pos.segments, 1, "one `.ex` body: {pos:?}");
    assert_eq!(pos.bodies_out_of_class, 0, "and it is in class: {pos:?}");

    // Add two file-scope objects and the gate stops at the DATA symbols — after
    // binding, after the body parse, with the downstream gates evaluated.
    let data = causes(
        &tc,
        "data",
        include_str!("../../../fixtures/cpp/wvec_float_store_leaf_data_bss_neg.cpp"),
    );
    assert_eq!(
        data.first,
        Some(c2_il::func::cause::UNCLAIMED),
        "the `.data`/`.bss` cell must stop at the unclaimed symbols: {data:?}"
    );
    assert!(
        data.downstream_evaluated,
        "…which means the binding SUCCEEDED and the post-binding gates ran: {data:?}"
    );
    assert_eq!(data.segments, 1, "still one `.ex` body: {data:?}");
    assert_eq!(data.bodies_out_of_class, 0, "still in class: {data:?}");

    // The `vec.cpp` shape stops EARLIER — at the binding — so none of the
    // above is even asked. This is factor A showing up as a decode cause.
    let ctor = causes(
        &tc,
        "ctor",
        include_str!("../../../fixtures/cpp/wvec_inclass_ctor_folded_statics_neg.cpp"),
    );
    assert_eq!(
        ctor.first,
        Some(c2_il::func::cause::BIND_COUNT),
        "the in-class-ctor cell must stop at the record/segment count: {ctor:?}"
    );
    assert!(
        !ctor.downstream_evaluated,
        "…so `unclaimed-gl-symbol` has NO answer on this TU and must not be \
         reported as passing: {ctor:?}"
    );
    assert!(
        ctor.records_gate < ctor.segments,
        "the surplus direction: more `.ex` bodies than `.gl` records ({} < {}): {ctor:?}",
        ctor.records_gate,
        ctor.segments
    );

    // The three causes are pairwise distinct. Stated as a set so a future edit
    // that collapses two cells into one is a failure and not a silent loss.
    let keys: std::collections::BTreeSet<Option<&str>> =
        [pos.first, data.first, ctor.first].into_iter().collect();
    assert_eq!(keys.len(), 3, "three cells, three distinct verdicts: {keys:?}");
}

/// **The anti-drift invariant, on real captures.** `DecodeCauses::decodes` is
/// read from the real [`c2_il::IlBundle::decodes`] and `causes` from the
/// re-asked predicates; `causes.is_empty() == decodes` is what stops the
/// diagnostic from becoming a second, disagreeing gate. `c2-il` asserts it on
/// synthetic bundles; this asserts it on the three shipped sources, which is
/// where a real divergence would live.
#[test]
fn the_diagnostic_agrees_with_the_gate_on_every_vec_cell() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    for (tag, body) in [
        ("inv-pos", include_str!("../../../fixtures/cpp/wvec_float_store_leaf.cpp")),
        (
            "inv-data",
            include_str!("../../../fixtures/cpp/wvec_float_store_leaf_data_bss_neg.cpp"),
        ),
        (
            "inv-ctor",
            include_str!("../../../fixtures/cpp/wvec_inclass_ctor_folded_statics_neg.cpp"),
        ),
    ] {
        let c = causes(&tc, tag, body);
        assert_eq!(
            c.causes.is_empty(),
            c.decodes,
            "cell `{tag}`: the diagnostic and the gate disagree: {c:?}"
        );
        assert_eq!(
            c.first.is_none(),
            c.decodes,
            "cell `{tag}`: a first cause exists iff the bundle is refused: {c:?}"
        );
    }
}
