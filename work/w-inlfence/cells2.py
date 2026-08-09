#!/usr/bin/env python3
"""w-inlfence — third and final rewrite of `inline_fence.rs`'s cells.

The fence refuses a callee the port has NO model of, so every LOCAL cell's
callee must be out of class as well as non-empty: an in-class callee is
mechanism I's population (the splice lowers it) and an empty one is mechanism
E's. Both earlier drafts of these cells were confounded — the first graded the
E exemption, the second graded the I exemption — and both were caught by the
test, not by reading.
"""
p = "crates/c2-harness/tests/inline_fence.rs"
s = open(p).read()

LOOP = "for (int i = 0; i < 64; ++i) { p[i] = (char)(p[i] * 3 + i); }"

old = s[s.index("    // (tag, local source, opaque source)"):s.index("    for (tag, local, opaque) in pairs {")]
new = '''    // (tag, local source, opaque source)
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
            "void wif_t_g(char *p);\\n\\
             void wif_t_use(char *p) { wif_t_g(p); }\\n\\
             void wif_t_g(char *p) { LOOPBODY }\\n",
            "void wif_t_g(char *p);\\nvoid wif_t_use(char *p) { wif_t_g(p); }\\n",
        ),
        (
            "seq",
            "struct S { char *b; void m(); void n(); };\\n\\
             void wif_s_use(S *s) { s->m(); s->n(); }\\n\\
             void S::m() { char *p = b; LOOPBODY }\\n\\
             void S::n() { char *p = b; LOOPBODY }\\n",
            "struct S { char *b; void m(); void n(); };\\n\\
             void wif_s_use(S *s) { s->m(); s->n(); }\\n",
        ),
        (
            "valuetail",
            "struct T { char *b; void Split(); float Ms(); float SplitMs(); };\\n\\
             float T::SplitMs() { Split(); return Ms(); }\\n\\
             void T::Split() { char *p = b; LOOPBODY }\\n\\
             float T::Ms() { return (float)b[0] * 3.5f; }\\n",
            "struct T { void Split(); float Ms(); float SplitMs(); };\\n\\
             float T::SplitMs() { Split(); return Ms(); }\\n",
        ),
        (
            "inttail",
            "int wif_i_g(char *p);\\nint wif_i_use(char *p) { return wif_i_g(p); }\\n\\
             int wif_i_g(char *p) { LOOPBODY return p[0]; }\\n",
            "int wif_i_g(char *p);\\nint wif_i_use(char *p) { return wif_i_g(p); }\\n",
        ),
    ];

'''
s = s.replace(old, new, 1)

old_yield = s[s.index("    let (rows, _) = cells(\n        &tc,\n        \"empty\","):]
new_yield = '''    let (rows, _) = cells(
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

    // …and mechanism I's: an in-class callee is one `c2_core::splice` has a
    // body to substitute, graded 723 of 723 byte-exact.
    let (rows, _) = cells(
        &tc,
        "lowerable",
        "int wif_l_g(int a);\\nint wif_l_use(int a) { return wif_l_g(a); }\\n\\
         int wif_l_g(int a) { return a + 1; }\\n",
    );
    assert_eq!(
        fenced(&rows),
        0,
        "the callee is defined here and the port LOWERS it — mechanism I's own \\
         discriminating cell (`empty_elision.rs` c19). Rows: {rows:?}"
    );

    // …and the control: the same shape with a callee the port has NO model of
    // must be fenced, so this test cannot pass by the fence having stopped
    // working altogether.
    let (rows, _) = cells(
        &tc,
        "unmodelled-control",
        "void wif_e_g(char *p);\\nvoid wif_e_use(char *p) { wif_e_g(p); }\\n\\
         void wif_e_g(char *p) { LOOPBODY }\\n",
    );
    assert_eq!(
        fenced(&rows),
        1,
        "the control: a callee that is neither empty nor lowerable, and the \\
         fence must fire. Rows: {rows:?}"
    );
}
'''
s = s.replace(old_yield, new_yield, 1)
s = s.replace("LOOPBODY", LOOP)
open(p, "w").write(s)
print("ok")
