use c2_il::IlBundle;

/// **The two `.ex` readers must disagree on a bundle the gate refuses**, or
/// `gap.rs` step 3 can be fed by step 1g's variable without anything going
/// red.
///
/// That substitution actually happened, and the run it produced was green in
/// every field a reviewer scans: `mismatch 0`, `match 6`, `0 failed`. What it
/// did was move **865 TUs from `vocab-gap` to `codegen-gap`** — a report
/// claiming the port decodes the whole workload. The failure direction is
/// flattering, which is exactly the shape ROADMAP §9.18.8 records twelve
/// times.
///
/// So this pins the *difference*, not either reader: a bundle whose `.ex`
/// carries function-start markers the gate cannot accept must give
/// `ex_segment_count() = Some(n > 0)` and `functions() = None`. A test that
/// only asserted each reader's own value would pass with the two wired
/// together.
///
/// **What it does NOT do**, said here so it is not mistaken for a guard:
/// re-folding `gap.rs` step 3 onto step 1g's variable still passes this
/// test. The re-fold is only visible in the class counts, and `classify_one`
/// needs a toolchain. This makes the invariant executable; the 3-TU
/// both-ways scan in the rung doc is the evidence for the consequence.
#[test]
fn the_class_predicate_is_not_the_segment_counter() {
    // `.ex` with two `4F 1F` starts and nothing the body parser can read;
    // no `.gl` records, so the binding refuses. The point is only that the
    // gate says NO while segments exist.
    let mut ex = vec![0x11u8; 8];
    ex.extend_from_slice(&[0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00]);
    ex.extend_from_slice(&[0x22; 16]);
    ex.extend_from_slice(&[0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00]);
    ex.extend_from_slice(&[0x33; 16]);

    let mut b = IlBundle::default();
    b.base_name = "_CL_guard".to_string();
    b.files.insert("ex".to_string(), ex);
    b.files.insert("gl".to_string(), vec![0u8; 32]);

    let segs = b.ex_segment_count();
    assert_eq!(
        segs,
        Some(2),
        "the pure reader must count both `4F 1F` starts, whatever the gate thinks"
    );
    assert!(
        b.functions().is_none(),
        "control: the gate must REFUSE this bundle, or the pair below proves nothing"
    );
    assert_ne!(
        segs.is_none(),
        b.functions().is_none(),
        "the segment counter and the acceptance decision must not agree here — \
         if they do, gap.rs step 3 can be fed by step 1g's variable and \
         `vocab-gap` silently becomes `codegen-gap`"
    );
}
