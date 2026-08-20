//! **The inline fence** (lane `w-inlfence`, board **#2220**–**#2226**) — a
//! callee this TU also DEFINES is a callee c2 may **inline**, and the port may
//! not emit the call.
//!
//! # What this pins that a fixture cannot
//!
//! `fixtures/cpp/winlfence_local_callee_neg.cpp` grades a **whole-TU refusal**,
//! and a whole-TU refusal survives any one of its seven cells silently becoming
//! a positive — board **#2085** is that exact defect, found in `w-mcall`'s
//! `_neg` file by censusing it per function rather than by any gate row. So the
//! cells below are asserted **per function, by census key**, and each is paired
//! with its own OPAQUE twin: the identical source with the callee declared and
//! not defined, which must stay in class. A fence that refused both would pass a
//! test that only checked the refusals.
//!
//! # The two facts behind the fence, both measured against real `c2`
//!
//! * `int f(int); int use(int a){return f(a);} int f(int a){return a+1;}` — c2's
//!   `.text` is two copies of `addi r3,r3,1 ; blr` with **no relocations**; the
//!   port's `b ?f` mismatched at file offset 8.
//! * `?SplitMs@Timer@@QAAMXZ`, **434 emitted functions in 434 workload TUs** —
//!   reference body 31 words against the port's 13, because both callees are
//!   `inline` members in the same header
//!   (`docs/rungs/2026-08-09-w-fltret.md` §6, board #2082).
//!
//! Nothing here uses a size, a ceiling or a flag bit.
//! `docs/whitebox/WB_INLINE_FINDINGS.md` §7 measures c2's decision on 320 obj
//! cells and declines to offer the accept side of any of them; this fence uses
//! only the categorical direction — **c2 cannot inline a body it does not
//! have** — which needs no constant and carries no `DISCLOSURE.md` row.
//!
//! `SKIP: toolchain absent` when there is no toolchain, like every other
//! integration test here.

use std::path::PathBuf;

use c2_reference::Toolchain;

/// The workload's own profile, minus the `/I` paths a standalone cell cannot
/// use. `/O1` implies `/Gy`, which is the regime the whole 878-TU scan lives in.
const FLAGS: [&str; 8] = c2_harness::testsupport::WORKLOAD_FLAGS;

/// The census key the fence mints. Spelled once: a test that re-types the string
/// its subject produces is a test of two spellings.
const FENCE: &str = "callee-defined-in-tu";

/// A scratch directory keyed on the tag **and** the pid — board #1045: four
/// parallel tests sharing one PID-keyed directory raced their captures and
/// fabricated a finding.
fn work(tag: &str) -> PathBuf {
    c2_harness::testsupport::scratch_dir("inlfence", tag)
}

/// Capture one source and return `(mangled name, census key)` per `.ex` body,
/// plus whether `IlBundle::functions` — the port's own acceptance path —
/// accepts the whole TU.
///
/// The name is the census's own `reported_name` and is empty when the
/// positional pairing is not meaningful. It is carried for the assertion
/// messages only; every claim below is counted over the KEYS, for the reason
/// [`fenced`] states.
fn cells(tc: &Toolchain, tag: &str, body: &str) -> (Vec<(String, String)>, bool) {
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
        .unwrap_or_else(|| panic!("cell `{tag}`: no census"));
    let rows = census
        .iter()
        .map(|c| {
            (
                c.name.clone().unwrap_or_default(),
                c.verdict.key(),
            )
        })
        .collect();
    (rows, cap.bundle.functions().is_some())
}

/// How many of this TU's census rows the fence claims.
///
/// Counted rather than looked up by name: the census's positional binding
/// reports a name only when the `.gl` name count equals the segment count, and
/// a TU with one defined function and one external callee does not pair. A
/// name-keyed assertion would have silently skipped exactly the opaque twins
/// this test exists to grade.
fn fenced(rows: &[(String, String)]) -> usize {
    rows.iter().filter(|r| r.1.starts_with(FENCE)).count()
}

/// **Every fenced shape, and its opaque twin.**
///
/// The pairs are the four call carriers `IlFunction::callees` has a workload
/// population for: the void tail call, the member-call sequence's statement
/// form (`BodyShape::CallSeq`, w-mcall), its float VALUE tail
/// (`SeqTail::CallValueFp`, w-fltret — the 444's own class), and the int tail
/// call. A fence written against `tail_call` alone passes two of these four.
#[test]
fn a_callee_this_tu_defines_is_fenced_and_its_opaque_twin_is_not() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    // (tag, local source, opaque source)
    //
    // **Every LOCAL callee below is non-empty AND out of class**, deliberately.
    // A callee that emits nothing is mechanism E's population and a callee the
    // port can lower is mechanism I's; the fence yields to both, so a cell with
    // either would grade an exemption while claiming to grade the fence. A
    // counted loop is the cheapest body that is neither — it is also the shape
    // of the five surviving `bl`s in `src/keygen_xbox.cpp`.
    let pairs: [(&str, &str, &str); 4] = [
        (
            "tail",
            "void wif_t_g(char *p);\n\
             void wif_t_use(char *p) { wif_t_g(p); }\n\
             void wif_t_g(char *p) { for (int i = 0; i < 64; ++i) { p[i] = (char)(p[i] * 3 + i); } }\n",
            "void wif_t_g(char *p);\nvoid wif_t_use(char *p) { wif_t_g(p); }\n",
        ),
        (
            "seq",
            "struct S { char *b; void m(); void n(); };\n\
             void wif_s_use(S *s) { s->m(); s->n(); }\n\
             void S::m() { char *p = b; for (int i = 0; i < 64; ++i) { p[i] = (char)(p[i] * 3 + i); } }\n\
             void S::n() { char *p = b; for (int i = 0; i < 64; ++i) { p[i] = (char)(p[i] * 3 + i); } }\n",
            "struct S { char *b; void m(); void n(); };\n\
             void wif_s_use(S *s) { s->m(); s->n(); }\n",
        ),
        (
            "valuetail",
            "struct T { char *b; void Split(); float Ms(); float SplitMs(); };\n\
             float T::SplitMs() { Split(); return Ms(); }\n\
             void T::Split() { char *p = b; for (int i = 0; i < 64; ++i) { p[i] = (char)(p[i] * 3 + i); } }\n\
             float T::Ms() { return (float)b[0] * 3.5f; }\n",
            "struct T { void Split(); float Ms(); float SplitMs(); };\n\
             float T::SplitMs() { Split(); return Ms(); }\n",
        ),
        (
            "inttail",
            "int wif_i_g(char *p);\nint wif_i_use(char *p) { return wif_i_g(p); }\n\
             int wif_i_g(char *p) { for (int i = 0; i < 64; ++i) { p[i] = (char)(p[i] * 3 + i); } return p[0]; }\n",
            "int wif_i_g(char *p);\nint wif_i_use(char *p) { return wif_i_g(p); }\n",
        ),
    ];

    for (tag, local, opaque) in pairs {
        let (rows, gate) = cells(&tc, &format!("{tag}-local"), local);
        assert_eq!(
            fenced(&rows),
            1,
            "cell `{tag}`: the callee is DEFINED in this TU, so c2 may inline it \
             and exactly one row — the caller — must be fenced. Rows: {rows:?}"
        );
        assert!(
            !gate,
            "cell `{tag}`: IlBundle::functions accepted a TU whose callee it \
             also defines — that is a wrong obj, not a gap"
        );

        let (rows, gate) = cells(&tc, &format!("{tag}-opaque"), opaque);
        assert_eq!(
            fenced(&rows),
            0,
            "cell `{tag}`: the OPAQUE twin declares its callee and does not \
             define it — c2 cannot inline a body it does not have, and fencing \
             this is the fence being over-broad. Rows: {rows:?}"
        );
        assert!(
            gate,
            "cell `{tag}`: the opaque twin's whole TU must still be accepted — \
             every body is in class and no callee is defined here. Rows: {rows:?}"
        );
    }
}

/// **The fence is a whole-name set membership, and the near-miss proves it.**
///
/// This TU defines `wif_p_leaf` and calls `wif_p_leaf_x`, of which the defined
/// name is a strict PREFIX. `fixtures/cpp/winlfence_opaque_callee.cpp` cell F5
/// grades the same claim byte-exact against real `c2`; this one grades the key,
/// so a `starts_with` regression names itself here instead of showing up as a
/// silent census drop nobody attributes.
#[test]
fn a_defined_name_that_is_a_prefix_of_the_callee_fences_nothing() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    let (rows, gate) = cells(
        &tc,
        "prefix",
        "int wif_p_leaf_x(int a);\n\
         int wif_p_leaf(int a) { return a + 1; }\n\
         int wif_p_use(int a) { return wif_p_leaf_x(a); }\n",
    );
    assert_eq!(
        fenced(&rows),
        0,
        "`?wif_p_leaf_x@@YAHH@Z` is not `?wif_p_leaf@@YAHH@Z`; c2 has no body \
         for it. Rows: {rows:?}"
    );
    assert!(
        gate,
        "the whole TU should still be accepted — every body is in class and no \
         callee is defined here. Rows: {rows:?}"
    );
}

/// **The fence YIELDS to a graded model — mechanism E.**
///
/// `void g() {} void f() { g(); }` is a callee this TU defines, so the clause
/// above sees it; but `c2_core::elide` already models what c2 does with it —
/// the empty body inlines to nothing and the branch disappears — and the judge
/// grades that **1,877 of 1,877 byte-exact** over the 878-TU workload. Fencing
/// it would refuse a body the port provably gets right.
///
/// This is not the accept side of `WB_INLINE_FINDINGS.md`. Nothing here
/// predicts c2's decision from a size, a flag or a ceiling: the exemption is
/// the port's own pre-existing, obj-graded elision rule, and its population is
/// the one `crates/c2-harness/tests/call_targets.rs` has pinned since `w-inl0`.
#[test]
fn the_fence_yields_to_the_empty_callee_mechanism_e_already_models() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    let (rows, _) = cells(
        &tc,
        "empty",
        "void wif_e_g();\nvoid wif_e_use() { wif_e_g(); }\nvoid wif_e_g() {}\n",
    );
    assert_eq!(
        fenced(&rows),
        0,
        "the callee is defined here AND empty, which is mechanism E's own \
         population — refusing it is the fence being over-broad. Rows: {rows:?}"
    );

    // …and mechanism I's: an in-class callee is one `c2_core::splice` has a
    // body to substitute, graded 723 of 723 byte-exact.
    let (rows, _) = cells(
        &tc,
        "lowerable",
        "int wif_l_g(int a);\nint wif_l_use(int a) { return wif_l_g(a); }\n\
         int wif_l_g(int a) { return a + 1; }\n",
    );
    assert_eq!(
        fenced(&rows),
        0,
        "the callee is defined here and the port LOWERS it — mechanism I's own \
         discriminating cell (`empty_elision.rs` c19). Rows: {rows:?}"
    );

    // …and the control: the same shape with a callee the port has NO model of
    // must be fenced, so this test cannot pass by the fence having stopped
    // working altogether.
    let (rows, _) = cells(
        &tc,
        "unmodelled-control",
        "void wif_e_g(char *p);\nvoid wif_e_use(char *p) { wif_e_g(p); }\n\
         void wif_e_g(char *p) { for (int i = 0; i < 64; ++i) { p[i] = (char)(p[i] * 3 + i); } }\n",
    );
    assert_eq!(
        fenced(&rows),
        1,
        "the control: a callee that is neither empty nor lowerable, and the \
         fence must fire. Rows: {rows:?}"
    );
}

/// **W-FENCE2 — the parser hands a TU on ONLY when the facts the composition
/// seam needs are available, and refuses otherwise.**
///
/// The wholesale refusal above stopped being wholesale on 2026-08-09: a callee
/// this TU defines no longer refuses the TU when its `.gl` defined record has
/// **plain external** linkage and every segment is at `/O1`
/// (`c2_il::func::gl::plain_external_defined_names`,
/// `docs/rungs/2026-08-09-w-fence2.md`). This is the cell that says the
/// narrowing is a NARROWING and not a removal.
///
/// **Every negative here is a REALIZED wrong emit, not a hypothetical.** The
/// reference objs were dumped (`work/w-fence2/probe/`), and in both the `static`
/// and the `__forceinline` cell c2 **inlined** the 152-byte callee — the
/// wrapper's own `.text` is 152 bytes with **no REL24 to the callee at all** —
/// while in the positive cell the wrapper is 12 bytes and carries the branch.
/// Delete either clause and the port emits a call c2 does not.
///
/// The four sources are the four shipped fixtures, `include_str!`d rather than
/// retyped: a cell that drifts from the fixture it claims to be would grade a
/// different file with the same confidence.
#[test]
fn the_parser_hands_on_only_the_linkage_class_the_decline_bound_was_measured_on() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    let kept = include_str!("../../../fixtures/cpp/wfence2_kept_local_callee.cpp");
    let stat = include_str!("../../../fixtures/cpp/wfence2_static_callee_neg.cpp");
    let forced = include_str!("../../../fixtures/cpp/wfence2_forceinline_callee_neg.cpp");
    let small = include_str!("../../../fixtures/cpp/wfence2_small_callee_neg.cpp");

    let (rows, gate) = cells(&tc, "f2-kept", kept);
    assert!(
        gate,
        "the POSITIVE: a plain-external, non-`inline`, `/O1` callee over the \
         decline bound. c2 keeps this call (the reference wrapper is 12 bytes \
         and carries the REL24) and the whole TU is byte-exact. Rows: {rows:?}"
    );

    for (tag, src, why) in [
        (
            "f2-static",
            stat,
            "`static` — F1 puts the STATIC ceiling at (300,308], three times the \
             shipped bound, and the reference obj shows c2 INLINING this callee",
        ),
        (
            "f2-forceinline",
            forced,
            "`__forceinline` — F4: it bypasses every size test, and the linkage \
             byte cannot see it. The reference obj shows c2 INLINING this callee",
        ),
    ] {
        let (rows, gate) = cells(&tc, tag, src);
        assert!(
            !gate,
            "cell `{tag}`: IlBundle::functions accepted a TU it has no decline \
             proof for — {why}. Rows: {rows:?}"
        );
    }

    // The SMALL cell is the one that says both halves of the fence are live: the
    // parser exempts it (plain external, `/O1`) and the obj is still not
    // emitted, because `c2_core::comdat::fenced_inlined_callee` refuses a callee
    // whose lowered body is at or under `INLINE_DECLINE_BYTES`. A cell that only
    // checked the parser would pass with the seam deleted.
    let (rows, gate) = cells(&tc, "f2-small", small);
    assert!(
        gate,
        "cell `f2-small`: the PARSER must hand this TU on — its callee is plain \
         external at `/O1`, and the size question is not the parser's. \
         Rows: {rows:?}"
    );
}
