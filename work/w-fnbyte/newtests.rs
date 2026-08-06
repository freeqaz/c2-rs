
// ---------------------------------------------------------------------------
// Board #322 — the per-shape census, the witnesses, and the partition after the
// four `/Gy` shapes became gradeable (lane `w-fnbyte`)
// ---------------------------------------------------------------------------

/// **The per-shape census keeps a `differs` VISIBLE PER SHAPE.**
///
/// The blind spot board #322 closed was invisible for one reason: the alarm was
/// a corpus total and the shapes behind it were printed only for the bucket
/// that was *not* graded. A shape that goes wrong now has to show up as its own
/// row, and a shape that stops being graded loses its row rather than quietly
/// contributing zeros to a total that still reads 0.
#[test]
fn the_shape_census_reports_each_shape_and_verdict_separately() {
    let rep = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[
            ("fnbyte-exact", 3),
            ("fnbyte-differs", 2),
            ("fnbyte-shape|tail|fnbyte-exact", 3),
            ("fnbyte-shape|framed|fnbyte-differs", 2),
        ],
    )]);
    let census = rep.fn_byte_shape_census();
    assert_eq!(
        census,
        vec![
            ("tail".to_string(), "exact".to_string(), 3),
            ("framed".to_string(), "differs".to_string(), 2),
        ],
        "the census is (shape, verdict, count), most frequent first — a corpus \
         total cannot say WHICH shape went wrong"
    );
    // …and the row survives being the only one of its shape. A `differs` of 2 in
    // a denominator of 5 is what a per-shape regression looks like before it is
    // large enough to notice in the ratio.
    assert!(census.iter().any(|(s, v, _)| s == "framed" && v == "differs"));
}

/// **A witness is a reproducer, not a count.** Each differing function is named
/// with its shape, its word counts and the first disagreeing word — and the
/// signature collapse groups identical failures so 1,950 mangled STL names
/// reporting one defect read as one row and not as 1,950.
#[test]
fn the_differ_witnesses_collapse_to_signatures_with_an_example() {
    let rep = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[
            ("fnbyte-differs", 3),
            (
                "fnbyte-differs-fn|tail|w1/1/eq0|first@0:port=48000000,ref=4e800020|?a@@YAXXZ",
                1,
            ),
            (
                "fnbyte-differs-fn|tail|w1/1/eq0|first@0:port=48000000,ref=4e800020|?b@@YAXXZ",
                1,
            ),
            (
                "fnbyte-differs-fn|framed|w9/3/eq0|first@0:port=7d8802a6,ref=81630004|?c@@YAXXZ",
                1,
            ),
        ],
    )]);
    assert_eq!(rep.fn_byte_differ_witnesses().len(), 3, "one row per function");
    let sigs = rep.fn_byte_differ_signatures();
    assert_eq!(sigs.len(), 2, "two distinct failures, not three");
    assert_eq!(sigs[0].0, "tail|w1/1/eq0|first@0:port=48000000,ref=4e800020");
    assert_eq!(sigs[0].1, 2);
    assert!(
        sigs[0].2.starts_with("?a@@") || sigs[0].2.starts_with("?b@@"),
        "a signature carries an EXAMPLE symbol; a signature with no name is not \
         reproducible"
    );
    assert_eq!(sigs[1].1, 1);
}

/// **`partial 0` must be printed as a statement, not as an absent line.**
///
/// Before board #322 `partial by shape` was the size of the under-report and it
/// was never empty. Now it can be, and an omitted line is exactly the shape this
/// project has recorded sixteen times: absence read as success. The histogram
/// returning an empty vector is the *input* to that; the render prints the
/// positive sentence. This pins the input so the render cannot start guessing.
#[test]
fn an_empty_partial_histogram_is_empty_and_not_absent() {
    let rep = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[("fnbyte-exact", 10)],
    )]);
    assert!(rep.fn_byte_partial_histogram().is_empty());
    let f = rep.fn_byte_match().expect("10 graded functions");
    assert_eq!(f.partial, 0);
    assert_eq!(f.denominator, 10, "the denominator is still c2's, and printed");
}

/// **The partition identity holds with the four shapes graded**, and it is the
/// same identity as before: the buckets sum to the denominator. A shape moving
/// from `partial` into `exact` or `differs` must move the *total* by zero.
#[test]
fn the_partition_identity_survives_the_shapes_becoming_gradeable() {
    // Before: 4 partial. After: 3 exact + 1 differs. Same denominator, same sum.
    let before = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[("fnbyte-exact", 6), ("fnbyte-partial", 4)],
    )])
    .fn_byte_match()
    .unwrap();
    let after = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[("fnbyte-exact", 9), ("fnbyte-differs", 1)],
    )])
    .fn_byte_match()
    .unwrap();
    assert_eq!(before.denominator, after.denominator, "10 either way");
    assert_eq!(
        before.exact + before.partial,
        after.exact + after.differs,
        "the population is conserved; only its grading changed"
    );
    assert_eq!(before.partition_broken, 0);
    assert_eq!(after.partition_broken, 0);
    // …and the widened instrument scores the wrong body at zero, exactly as the
    // blind one scored the ungraded one. Grading more can only ever RAISE the
    // count of known-wrong bodies, never the credit.
    assert!(after.value > before.value, "3 of the 4 were right");
    assert_eq!(after.differs, 1, "and the fourth is now an ALARM, not a blank");
}
