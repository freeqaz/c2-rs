/// **ONE WITNESS PER RAISE SITE for `data-sym-strlit-fenced`** — board **#3246**,
/// `w-calleeguard` §8 **F3**, and the demonstration its **P13** registered and
/// missed.
///
/// # What was missing, precisely
///
/// This key has **two** raise sites, and `w-mutcensus`' frame froze at
/// `3835469c` — before `w-fence163` landed either of them — so **neither has
/// ever been mutated**. The tests above are per-CELL and per-KEY: each asserts
/// that some cell reports `data-sym-strlit-fenced:eof`, and *no assertion
/// anywhere says which of the two sites produced it*. That is exactly the
/// structure `w-mutcensus` **F2** describes and `w-calleeguard` §4.5 prescribes
/// the fix for:
///
/// > a witness TABLE with one row per RAISE SITE, keyed on the input that
/// > reaches that site, asserting the published key string.
///
/// # The table, and why each row's route is PROVEN rather than asserted in prose
///
/// | # | site | witness | the control that rules the OTHER site out |
/// |---|---|---|---|
/// | 1 | `census.rs:1259` — the **pre-parse** `sym_fail` probe, inside `shape_to_function`'s `None` arm | `WS`: **wide** literal, callee defined here, `eh-state1` | `S` — the identical TU with a **narrow** literal — is **IN CLASS**. So site 2's post-parse gate is *off* for this callee, and `WS`'s refusal cannot be coming from it |
/// | 2 | `census.rs:1511` — the **post-parse** `Some(f)` gate on the callee's EH state | `N`: **narrow** literal, callee defined here, `eh-none` | `X` — the identical literal with an **external** callee — is **IN CLASS**. So site 1's probe admits this literal, and `N`'s refusal cannot be coming from it |
///
/// Each row varies **one** fact against its control and everything else is held
/// byte-identical in the source. Neither row can be satisfied by the other
/// site's behaviour, which is the property `w-calleeguard` §2.2 named as *"the
/// clean demonstration that site-witnessing is strictly stronger than
/// key-witnessing"* — and unlike that lane's default-arm demonstration, this one
/// is on a family where **two sites share one key**, which is P13's registered
/// and un-demonstrated form.
///
/// Every assertion is on the **published key string** `FnVerdict::key()` emits,
/// never on `DATA_SYM_STRLIT_FENCED` — `w-guards`' rule: a guard on the constant
/// passes a mutation that renames the constant and its uses while the published
/// key moves.
///
/// # The site COUNT is guarded elsewhere, on purpose
///
/// A witness table covers the sites that existed when it was written. Nothing
/// here makes a **third** raise site add a row — that is
/// `tests/fence_site_census.rs`, which asserts this key has exactly **2**, and
/// the two tests are one guard rather than two only because both are keyed on
/// the same published string.
#[test]
fn each_raise_site_of_the_strlit_fence_has_its_own_witness_and_neither_covers_the_other() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    // The four cells. `BLIND` is not decoration — see this file's header: it is
    // the `26`-introduced record that makes census clause (c) blind, without
    // which clause (c2) is never asked at all and every row below would be
    // green for the wrong reason.
    //
    // `defined_here` differs from `external` in exactly one thing: whether `mk`
    // has a body in this TU.
    let defined_state1 = |lit: &str| {
        format!(
            "{BLIND}\
             class Obj {{ public: Obj(const char*); ~Obj(); const char* Get(); }};\n\
             inline const char* mk(const char* s) {{ Obj o(s); return o.Get(); }}\n\
             const char* caller() {{ return mk({lit}); }}\n"
        )
    };
    let defined_ehnone = |lit: &str| {
        format!(
            "{BLIND}\
             class Buf {{ public: Buf(const char*); const char* Str(); }};\n\
             inline const char* mk(const char* s) {{ Buf b(s); return b.Str(); }}\n\
             const char* caller() {{ return mk({lit}); }}\n"
        )
    };
    let external = |lit: &str| {
        format!(
            "{BLIND}\
             const char* mk(const char* s);\n\
             const char* caller() {{ return mk({lit}); }}\n"
        )
    };

    // ROW 1 and its control: the ONE fact that moves is the literal's WIDTH.
    let (ws_keys, _) = cells(&tc, "site1-wide", &defined_state1("L\"UPDATE:\""));
    let (s_keys, _) = cells(&tc, "site1-narrow-ctl", &defined_state1("\"UPDATE:\""));
    // ROW 2 and its control: the ONE fact that moves is whether the callee is
    // DEFINED HERE.
    let (n_keys, _) = cells(&tc, "site2-definedhere", &defined_ehnone("\"UPDATE:\""));
    let (x_keys, _) = cells(&tc, "site2-external-ctl", &external("\"UPDATE:\""));

    // ---- row 1: `census.rs:1259`, the pre-parse `sym_fail` probe ------------
    assert_eq!(
        n(&ws_keys, FENCED), 1,
        "ROW 1 — the WIDE literal must take `{FENCED}` through the PRE-PARSE \
         `sym_fail` probe (`census.rs:1259`): `resolve_data`'s narrow-prefix \
         clause does not admit `??_C@_1…`, so the symbol never resolves and \
         `shape_to_function` returns `None` with the probe pending. Keys were \
         {ws_keys:?}"
    );
    assert_eq!(
        n(&s_keys, IN_CLASS), 1,
        "ROW 1's CONTROL — the SAME TU with a NARROW literal must be IN CLASS. \
         This is what rules out the other raise site: if the post-parse gate at \
         `census.rs:1511` were what refused the wide cell, it would refuse this \
         one too (same callee, same EH state, same everything but the mangling's \
         width field). Keys were {s_keys:?}"
    );
    assert_eq!(
        n(&s_keys, FENCED), 0,
        "ROW 1's CONTROL must not carry the fence's key at all, or the pair is \
         not isolating the width. Keys were {s_keys:?}"
    );

    // ---- row 2: `census.rs:1511`, the post-parse `Some(f)` gate -------------
    assert_eq!(
        n(&n_keys, FENCED), 1,
        "ROW 2 — the NARROW literal whose callee is DEFINED HERE and decodes \
         `eh-none` must take `{FENCED}` through the POST-PARSE gate \
         (`census.rs:1511`). Keys were {n_keys:?}"
    );
    assert_eq!(
        n(&x_keys, IN_CLASS), 1,
        "ROW 2's CONTROL — the SAME narrow literal with an EXTERNAL callee must \
         be IN CLASS. This is what rules out the other raise site: the pre-parse \
         probe at `census.rs:1259` fires on the LITERAL, not on the callee, so if \
         it were what refused row 2 it would refuse this cell identically. Keys \
         were {x_keys:?}"
    );
    assert_eq!(
        n(&x_keys, FENCED), 0,
        "ROW 2's CONTROL must not carry the fence's key. Keys were {x_keys:?}"
    );

    // ---- the table is a DISCRIMINATION, asserted as a count ----------------
    //
    // Four cells, two fenced and two in class, and the two fenced ones are
    // reached by moving DIFFERENT facts. A collapse — every cell refusing, or
    // every cell in class — satisfies no pair above, and this count says so in
    // one line rather than leaving it to be inferred from four.
    let fenced = [&ws_keys, &s_keys, &n_keys, &x_keys]
        .iter()
        .filter(|k| n(k, FENCED) == 1)
        .count();
    let in_class = [&ws_keys, &s_keys, &n_keys, &x_keys]
        .iter()
        .filter(|k| n(k, IN_CLASS) == 1)
        .count();
    assert_eq!(
        (fenced, in_class),
        (2, 2),
        "the four-cell table must split 2 fenced / 2 in class — got \
         {fenced} fenced, {in_class} in class. Both fenced cells are witnesses \
         (one per raise site); both in-class cells are the controls that keep \
         each witness from being satisfiable by the other site"
    );

    // …and the two witnesses must not be the same cell twice: they differ in
    // the literal's width AND in whether the callee is defined here, so a
    // reader can see the two rows are two inputs and not one restated.
    assert_ne!(
        ws_keys, n_keys,
        "the two witnesses must be distinguishable TUs; identical key vectors \
         would mean the table has one row written twice"
    );
}
