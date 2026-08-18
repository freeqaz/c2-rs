//! **The narrow string-literal admission and its EH-state inline fence** (lane
//! `w-fence163`, rung `docs/rungs/2026-08-17-fence163.md`) — the three sites
//! that let a `??_C@_0…` address be relocated against, and the two fences that
//! bound the licence.
//!
//! # Why this file exists at all
//!
//! The widening it guards moved `fnbyte-exact` **35,734 → 35,897** on the 878-TU
//! workload (+163, every one relocation-graded) at `d28326b4`, and the whole of
//! that yield sits behind a predicate — *"a strlit-carrying body whose
//! defined-here callee is neither modelled nor `eh-state1` refuses"* — that
//! **no test could see when it shipped**. `cargo test --workspace` passed
//! 1,648/0 with the fence inverted-able, deletable and widenable in silence.
//! That is this repo's most-recorded defect family (`docs/STATUS.md` trap 5,
//! board #3199/#3214-#3217), and mutants MF1-MF3 of the lane's prereg are
//! registered RED against exactly these cells.
//!
//! # Why the cells are CAPTURES and not synthetic bytes
//!
//! Census clause **(c)** — `callee-defined-in-tu`, the older inline fence —
//! **shadows** clause (c2) on any TU whose `.gl` defined-name walk succeeds, and
//! it succeeds on every small TU. Measured at `d28326b4`: `g3_mkstring.cpp` (the
//! `MakeString` mirror, four lines) reports `callee-defined-in-tu:eof`, not the
//! strlit fence's key. Clause (c2) is reachable **only where
//! `defined_name_set`'s whole-TU walk binds nothing**, which is the condition
//! that made `?ContentPath@XboxContentMgr@@UAAPBDH@Z` grade `fnbyte-differs`
//! under an unfenced admission in the first place.
//!
//! So every cell below carries the `gl.rs` #232 shape — `struct M : Bd` whose
//! **implicitly generated** `??1M@@QAA@XZ` is `26`-introduced, which refuses
//! `gl_defined_names` for the whole file — as the *carrier* of the blindness.
//! It is not decoration: delete those four lines and clause (c) answers first,
//! clause (c2) is never asked, and this file goes green for the wrong reason.
//! That is asserted, not trusted ([`the_older_inline_fence_shadows_this_one_on_a_walkable_tu`]).
//!
//! # The cells
//!
//! | cell | the one fact | key at `d28326b4` |
//! |---|---|---|
//! | `X` narrow, callee EXTERNAL | — | `multiarg-tail-call` (in class) |
//! | `W` wide (`??_C@_1`), callee external | literal width | `data-sym-strlit-fenced:eof` |
//! | `N` narrow, callee DEFINED HERE, `eh-none` | callee EH state | `data-sym-strlit-fenced:eof` |
//! | `S` narrow, callee DEFINED HERE, `eh-state1` | callee EH state | `multiarg-tail-call` (in class) |
//!
//! `X`/`W` differ in one character of source (`char` vs `wchar_t`) and pin the
//! narrow-only prefix. `N`/`S` differ in whether the local callee holds an
//! unwindable local and pin the fence's discriminator — the four-cell obj grid
//! (rung §2) measured c2 KEEPING a call to an EH-stateful callee and INLINING an
//! EH-stateless one, so `N` un-inlined would be 3 words against c2's 14 and `S`
//! is the shape the +163 is made of.
//!
//! `SKIP: toolchain absent` when there is no toolchain, like every other
//! integration test here.

use std::path::PathBuf;

use c2_reference::Toolchain;

/// The workload's own profile, minus the `/I` paths a standalone cell cannot
/// use — the flags the 878-TU scan and the whole +163 were measured at.
///
/// **Load-bearing, not boilerplate.** At the `census` CLI's default `/Ox /GS- /c`
/// the literal's `.gl` record does not bind at all and cell `X` reports
/// `data-sym-unresolved:eof` — measured at `d28326b4`. A cell captured at the
/// wrong profile grades a different question and would pass with the fence gone.
const FLAGS: [&str; 8] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/O1", "/Oi", "/EHsc",
];

/// The key the fence mints, and the key a body in class reports. Spelled once
/// each: a test that re-types the string its subject produces is a test of two
/// spellings.
const FENCED: &str = "data-sym-strlit-fenced:eof";
const IN_CLASS: &str = "multiarg-tail-call";

/// The `.gl` shape that makes census clause (c) BLIND, prepended to every cell.
///
/// `??1M@@QAA@XZ` — `M`'s implicitly generated destructor — is `26`-introduced,
/// and a `26`-introduced record refuses `gl_defined_names` for the whole file
/// (`c2-il/src/func/gl.rs`, board **#232**, whose own test builds this record by
/// hand). With the walk refusing, clause (c)'s `defined` set is empty and the
/// question *"is this callee defined in this TU?"* falls through to clause (c2),
/// which asks it against the **emit binding** instead.
const BLIND: &str = "struct Bd { Bd(); ~Bd(); int b0; };\n\
                     struct M : Bd { M(); };\n\
                     struct D : M { D(); };\n\
                     D::D() {}\n";

/// A scratch directory keyed on the tag **and** the pid — board #1045: four
/// parallel tests sharing one PID-keyed directory raced their captures and
/// fabricated a finding.
fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-strlit-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Capture one source and return `(census keys in `.ex` order, whether
/// `IlBundle::functions` accepts the whole TU)`.
///
/// Keyed by position rather than by name because the census's positional
/// pairing reports a name only when the `.gl` name count equals the segment
/// count, and none of these TUs pair — a name-keyed lookup would silently skip
/// every cell here.
fn cells(tc: &Toolchain, tag: &str, body: &str) -> (Vec<String>, bool) {
    let dir = work(tag);
    let cpp = dir.join(format!("{tag}.cpp"));
    std::fs::write(&cpp, body).unwrap();
    let flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
    let src = c2_reference::to_wibo_path(&cpp);
    let cap = tc
        .capture_reference_with(&src, &dir, &flags, None)
        .unwrap_or_else(|e| panic!("cell `{tag}`: capture failed: {e}"));
    let census = cap
        .bundle
        .function_census()
        .unwrap_or_else(|| panic!("cell `{tag}`: no census at all — a cell that \
             stopped producing rows grades nothing and must fail loudly rather \
             than pass"));
    assert!(
        !census.is_empty(),
        "cell `{tag}`: the census is EMPTY. Every assertion below would be \
         vacuous (`docs/STATUS.md` trap 5)"
    );
    (
        census.iter().map(|c| c.verdict.key()).collect(),
        cap.bundle.functions().is_some(),
    )
}

/// How many of this TU's rows carry `key`. Counted, so a cell that grew a body
/// cannot silently satisfy a `contains`.
fn n(keys: &[String], key: &str) -> usize {
    keys.iter().filter(|k| k.as_str() == key).count()
}

/// **MF1 / MF2 — the fence's discriminator is the callee's EH STATE, and both
/// directions are graded.**
///
/// Cells `N` and `S` are the same body, the same literal, the same
/// locally-defined callee shape; the ONE difference is whether that callee holds
/// an unwindable local (`~Obj`) and so decodes `eh-state1`. The obj grid (rung
/// §2, cells g2/g3) measured real `c2.dll` KEEPING the call in the `eh-state1`
/// case and INLINING it in the `eh-none` case, so:
///
/// * `N` (`eh-none`) must REFUSE — un-inlined it is 3 words against c2's 14,
///   which is `?ContentPath@…`'s exact wrong lowering;
/// * `S` (`eh-state1`) must stay IN CLASS — it is the shape the +163 is made of
///   (1,047 calls to `?__stl_throw_length_error@…` plus 8 to
///   `?__stl_throw_out_of_range@…`, both `eh-state1`).
///
/// **Inverting** the predicate swaps both answers; **deleting** it turns `N` in
/// class. Either mutation fails this test, which is what MF1 and MF2 registered
/// RED and what nothing in the suite could see at `d28326b4`.
#[test]
fn the_strlit_fence_turns_on_the_local_callees_eh_state_and_nothing_else() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    // `mk` has NO unwindable local -> `eh-none` -> c2 inlines it -> refuse.
    let none_src = format!(
        "{BLIND}\
         class Buf {{ public: Buf(const char*); const char* Str(); }};\n\
         inline const char* mk(const char* s) {{ Buf b(s); return b.Str(); }}\n\
         const char* caller() {{ return mk(\"UPDATE:\"); }}\n"
    );
    // `mk` holds a dtor local -> `eh-state1` -> c2 keeps the call -> admit.
    let state1_src = format!(
        "{BLIND}\
         class Obj {{ public: Obj(const char*); ~Obj(); const char* Get(); }};\n\
         inline const char* mk(const char* s) {{ Obj o(s); return o.Get(); }}\n\
         const char* caller() {{ return mk(\"UPDATE:\"); }}\n"
    );

    let (none_keys, none_emits) = cells(&tc, "ehnone", &none_src);
    let (state1_keys, state1_emits) = cells(&tc, "ehstate1", &state1_src);

    assert_eq!(
        n(&none_keys, FENCED), 1,
        "MF1/MF2: a narrow string literal handed to a locally-defined \
         EH-STATELESS callee must be refused under `{FENCED}` — c2 INLINES that \
         callee (grid cell g3) and the port's un-inlined tail call is 3 words \
         against 14, which is `?ContentPath@XboxContentMgr@@UAAPBDH@Z`'s \
         measured wrong lowering. Keys were {none_keys:?}"
    );
    assert_eq!(
        n(&none_keys, IN_CLASS), 0,
        "MF2: …and it must not be IN CLASS by any route. Keys were {none_keys:?}"
    );
    assert_eq!(
        n(&state1_keys, IN_CLASS), 1,
        "MF1: the SAME body whose local callee holds an unwindable local decodes \
         `eh-state1`, and c2 KEEPS the call to it (grid cell g2 — the \
         discriminating cell, a dtor local with no throw anywhere). It must stay \
         IN CLASS: this is the population the whole +163 is made of, and a fence \
         that refuses it too has a price equal to its yield. Keys were \
         {state1_keys:?}"
    );
    assert_eq!(
        n(&state1_keys, FENCED), 0,
        "MF1: …and the `eh-state1` cell must not take the fence's key at all. \
         Keys were {state1_keys:?}"
    );

    // The two cells differ in ONE fact and their verdicts differ. Asserted as a
    // pair, because either assertion alone passes on a constant answer.
    assert_ne!(
        n(&none_keys, FENCED), n(&state1_keys, FENCED),
        "the pair must DISCRIMINATE: `eh-none` refuses and `eh-state1` does not. \
         Equal counts mean the fence is answering the same way regardless of the \
         callee's EH state and the cells are not locating the clause"
    );

    // **The whole-TU gate refuses BOTH, and it must** — `IlBundle::functions`
    // returns `None` for any admitted body carrying a `??_C@_0` sym, because the
    // real obj DEFINES the literal's `.rdata` COMDAT and this writer has no
    // emitter for it (`IL_CALL_IN_EXPR.md` §17.2 item 7's 5-against-6-section
    // mis-emit). The admission's whole yield is per-function byte credit.
    assert!(
        !state1_emits,
        "the whole-TU gate must REFUSE a TU whose admitted body references a \
         narrow string literal, even though the body itself is in class — the \
         writer cannot emit the literal's `.rdata` COMDAT and accounting it as an \
         undefined external is §17.2 item 7's five-section mis-emit"
    );
    assert!(
        !none_emits,
        "the fenced TU must not emit either (it is refused twice over)"
    );
}

/// **MF3 — the admission is the NARROW literal only, and the pair differs by one
/// character of source.**
///
/// `??_C@_0` is `char`; `??_C@_1` is `wchar_t`. `w-section` §3.3 split the class
/// by name over 1,458 head functions and found **wide 0, other 0** — so the wide
/// form is a population no capture has ever graded, and admitting it is the
/// exact generalization `docs/GAPS.md` §6 forbids. Widening the prefix to
/// `??_C@` turns cell `W` in class, which is MF3's registered RED.
///
/// Cell `X` is the positive control and it is not optional: without it a fence
/// that refused every literal would pass the negative alone. `X` is also the
/// **+163's own shape** — one narrow literal, one external callee.
#[test]
fn only_the_narrow_string_literal_is_admitted_and_the_wide_twin_still_refuses() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    let (narrow, narrow_emits) = cells(
        &tc,
        "narrow",
        "void d1(const char*);\nvoid f1() { d1(\"aa\"); }\n",
    );
    let (wide, wide_emits) = cells(
        &tc,
        "wide",
        "void d2(const wchar_t*);\nvoid f2() { d2(L\"aa\"); }\n",
    );

    assert_eq!(
        n(&narrow, IN_CLASS), 1,
        "MF3 positive control: a NARROW literal passed to an EXTERNAL callee is \
         the +163's own shape and must be IN CLASS. Got {narrow:?}. If this is \
         not in class the admission is refusing its whole input and the negative \
         below proves nothing"
    );
    assert_eq!(
        n(&wide, FENCED), 1,
        "MF3: the SAME body with a WIDE literal (`??_C@_1…`) must keep refusing, \
         under the fence's own key. Got {wide:?}. `w-section` §3.3 measured wide \
         **0** of 1,458 — nothing has ever graded a wide literal's emit, and \
         widening `??_C@_0` to `??_C@` admits it silently"
    );
    assert_eq!(
        n(&wide, IN_CLASS), 0,
        "MF3: …and the wide cell must not be in class by any route. Got {wide:?}"
    );
    assert_ne!(
        narrow, wide,
        "the pair must DISCRIMINATE: these two TUs differ only in the literal's \
         width, so identical key lists mean the prefix gate is not being asked"
    );

    // Both TUs carry a literal the writer cannot define, so neither emits — the
    // narrow one *despite* being in class. Stated here as well as in the pair
    // above because it is the claim that keeps the +163 honest: byte credit, not
    // a TU conversion.
    assert!(
        !narrow_emits,
        "the in-class narrow cell must still be refused WHOLE by \
         `IlBundle::functions` — per-function byte credit is the entire yield"
    );
    assert!(
        !wide_emits,
        "and the wide cell, which is not in class at all, must not emit"
    );
}

/// **The shadowing fact, asserted so this file cannot go green for the wrong
/// reason.**
///
/// Census clause (c) (`callee-defined-in-tu`) is asked BEFORE clause (c2). On a
/// TU whose `.gl` defined-name walk succeeds it answers first and the strlit
/// fence is never reached — measured at `d28326b4` on the four-line
/// `g3_mkstring.cpp`. Every cell in the tests above therefore carries the
/// [`BLIND`] carrier, and this test is the control that says the carrier is what
/// makes the difference: the identical body WITHOUT it reports clause (c)'s key
/// instead.
///
/// Two things break if this is deleted. A future lane that fixes
/// `defined_name_set`'s walk (board #1721's neighbourhood) makes clause (c) see
/// everything and silently retires clause (c2) — this test is where that shows
/// up as a red rather than as a fence nobody exercises. And a reader who trims
/// the four `struct` lines as noise turns the pair above into two cells that
/// agree, both green, both meaningless.
#[test]
fn the_older_inline_fence_shadows_this_one_on_a_walkable_tu() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    let body = "class Buf { public: Buf(const char*); const char* Str(); };\n\
                inline const char* mk(const char* s) { Buf b(s); return b.Str(); }\n\
                const char* caller() { return mk(\"UPDATE:\"); }\n";
    let (walkable, _) = cells(&tc, "walkable", body);
    let (blind, _) = cells(&tc, "blinded", &format!("{BLIND}{body}"));

    assert_eq!(
        n(&walkable, "callee-defined-in-tu:eof"), 1,
        "control: on a TU whose `.gl` defined-name walk SUCCEEDS, the older \
         inline fence (clause (c)) claims this body and clause (c2) is never \
         asked. Got {walkable:?}. If this is no longer true the cells in this \
         file are grading clause (c2) by accident and their `BLIND` carrier is \
         doing nothing"
    );
    assert_eq!(
        n(&walkable, FENCED), 0,
        "control: …so the strlit fence's key must NOT appear on the walkable TU. \
         Got {walkable:?}"
    );
    assert_eq!(
        n(&blind, FENCED), 1,
        "…and the SAME body behind the `26`-introduced implicit destructor — \
         which refuses `gl_defined_names` whole-file — falls through to clause \
         (c2) and takes the strlit fence's key. Got {blind:?}. This is the one \
         reachability condition the fence has, and it is the condition \
         `?ContentPath@…` met on `ContentMgr_Xbox.cpp`"
    );
    assert_eq!(
        n(&blind, "callee-defined-in-tu:eof"), 0,
        "…and clause (c) must be silent there, or the two clauses are both \
         firing and the cells above cannot attribute their verdict"
    );
}

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
    // `ty` and `lit` move together — a `wchar_t` literal needs a `wchar_t`
    // callee or `cl.exe` refuses the TU at C2664, which is the same coupling
    // `only_the_narrow_string_literal_is_admitted_and_the_wide_twin_still_refuses`
    // carries in its own pair. The ONE fact under test is still the literal's
    // width; the parameter type is that fact spelled in the signature.
    let defined_state1 = |ty: &str, lit: &str| {
        format!(
            "{BLIND}\
             class Obj {{ public: Obj(const {ty}*); ~Obj(); const {ty}* Get(); }};\n\
             inline const {ty}* mk(const {ty}* s) {{ Obj o(s); return o.Get(); }}\n\
             const {ty}* caller() {{ return mk({lit}); }}\n"
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
    let (ws_keys, _) = cells(&tc, "site1-wide", &defined_state1("wchar_t", "L\"UPDATE:\""));
    let (s_keys, _) = cells(&tc, "site1-narrow-ctl", &defined_state1("char", "\"UPDATE:\""));
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
