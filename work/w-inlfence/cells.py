#!/usr/bin/env python3
"""w-inlfence — rewrite `inline_fence.rs`'s cells now that the fence yields to
mechanism E.

Every LOCAL cell's callee has to be non-empty, or the exemption fires and the
cell grades the exemption instead of the fence — a confounded cell of exactly
the kind board #2085 and w-bdnz record. The exemption gets its own cell.
"""
p = "crates/c2-harness/tests/inline_fence.rs"
s = open(p).read()

old = s[s.index("    // (tag, local source, opaque source)"):s.index("    for (tag, local, opaque) in pairs {")]
new = '''    // (tag, local source, opaque source)
    //
    // **Every LOCAL callee below is NON-EMPTY**, deliberately: an empty one is
    // exempted by mechanism E (see the third test), and a cell whose callee is
    // `{}` would grade the exemption while claiming to grade the fence.
    let pairs: [(&str, &str, &str); 4] = [
        (
            "tail",
            "void wif_t_ext();\\nvoid wif_t_g();\\n\\
             void wif_t_use() { wif_t_g(); }\\nvoid wif_t_g() { wif_t_ext(); }\\n",
            "void wif_t_g();\\nvoid wif_t_use() { wif_t_g(); }\\n",
        ),
        (
            "seq",
            "void wif_s_ext();\\nstruct S { void m(); void n(); };\\n\\
             void wif_s_use(S *s) { s->m(); s->n(); }\\n\\
             void S::m() { wif_s_ext(); }\\nvoid S::n() { wif_s_ext(); }\\n",
            "struct S { void m(); void n(); };\\n\\
             void wif_s_use(S *s) { s->m(); s->n(); }\\n",
        ),
        (
            "valuetail",
            "void wif_v_ext();\\n\\
             struct T { void Split(); float Ms(); float SplitMs(); };\\n\\
             float T::SplitMs() { Split(); return Ms(); }\\n\\
             void T::Split() { wif_v_ext(); }\\nfloat T::Ms() { return 0.0f; }\\n",
            "struct T { void Split(); float Ms(); float SplitMs(); };\\n\\
             float T::SplitMs() { Split(); return Ms(); }\\n",
        ),
        (
            "inttail",
            "int wif_i_g(int a);\\nint wif_i_use(int a) { return wif_i_g(a); }\\n\\
             int wif_i_g(int a) { return a + 1; }\\n",
            "int wif_i_g(int a);\\nint wif_i_use(int a) { return wif_i_g(a); }\\n",
        ),
    ];

'''
s = s.replace(old, new, 1)

s = s.rstrip() + '''

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
        "void wif_e_g();\\nvoid wif_e_use() { wif_e_g(); }\\nvoid wif_e_g() {}\\n",
    );
    assert_eq!(
        fenced(&rows),
        0,
        "the callee is defined here AND empty, which is mechanism E's own \\
         population — refusing it is the fence being over-broad. Rows: {rows:?}"
    );

    // …and the control: the SAME shape with a non-empty callee is fenced, so
    // this test cannot pass by the fence having stopped working.
    let (rows, _) = cells(
        &tc,
        "empty-control",
        "void wif_e_ext();\\nvoid wif_e_g();\\n\\
         void wif_e_use() { wif_e_g(); }\\nvoid wif_e_g() { wif_e_ext(); }\\n",
    );
    assert_eq!(
        fenced(&rows),
        1,
        "the control: one `{{}}` removed from the callee and the fence must \\
         fire. Rows: {rows:?}"
    );
}
'''
open(p, "w").write(s)
print("ok")
