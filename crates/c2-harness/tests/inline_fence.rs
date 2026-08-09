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
const FLAGS: [&str; 8] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/O1", "/Oi", "/EHsc",
];

/// The census key the fence mints. Spelled once: a test that re-types the string
/// its subject produces is a test of two spellings.
const FENCE: &str = "callee-defined-in-tu";

/// A scratch directory keyed on the tag **and** the pid — board #1045: four
/// parallel tests sharing one PID-keyed directory raced their captures and
/// fabricated a finding.
fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-inlfence-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
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
    // **Every LOCAL callee below is NON-EMPTY**, deliberately: an empty one is
    // exempted by mechanism E (see the third test), and a cell whose callee is
    // `{}` would grade the exemption while claiming to grade the fence.
    let pairs: [(&str, &str, &str); 4] = [
        (
            "tail",
            "void wif_t_ext();\nvoid wif_t_g();\n\
             void wif_t_use() { wif_t_g(); }\nvoid wif_t_g() { wif_t_ext(); }\n",
            "void wif_t_g();\nvoid wif_t_use() { wif_t_g(); }\n",
        ),
        (
            "seq",
            "void wif_s_ext();\nstruct S { void m(); void n(); };\n\
             void wif_s_use(S *s) { s->m(); s->n(); }\n\
             void S::m() { wif_s_ext(); }\nvoid S::n() { wif_s_ext(); }\n",
            "struct S { void m(); void n(); };\n\
             void wif_s_use(S *s) { s->m(); s->n(); }\n",
        ),
        (
            "valuetail",
            "void wif_v_ext();\n\
             struct T { void Split(); float Ms(); float SplitMs(); };\n\
             float T::SplitMs() { Split(); return Ms(); }\n\
             void T::Split() { wif_v_ext(); }\nfloat T::Ms() { return 0.0f; }\n",
            "struct T { void Split(); float Ms(); float SplitMs(); };\n\
             float T::SplitMs() { Split(); return Ms(); }\n",
        ),
        (
            "inttail",
            "int wif_i_g(int a);\nint wif_i_use(int a) { return wif_i_g(a); }\n\
             int wif_i_g(int a) { return a + 1; }\n",
            "int wif_i_g(int a);\nint wif_i_use(int a) { return wif_i_g(a); }\n",
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

    // …and the control: the SAME shape with a non-empty callee is fenced, so
    // this test cannot pass by the fence having stopped working.
    let (rows, _) = cells(
        &tc,
        "empty-control",
        "void wif_e_ext();\nvoid wif_e_g();\n\
         void wif_e_use() { wif_e_g(); }\nvoid wif_e_g() { wif_e_ext(); }\n",
    );
    assert_eq!(
        fenced(&rows),
        1,
        "the control: one `{{}}` removed from the callee and the fence must \
         fire. Rows: {rows:?}"
    );
}
