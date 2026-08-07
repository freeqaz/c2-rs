// GRID K3 cell `h_livearg_ctrl` — w-carrier, board #1199.
// DECLARED POST-HOC (see grid3gen.py's header) — not frozen ahead.
// TARGETS:   the same bind where the call does NOT pass `s`
// PREDICTED: IN-CLASS / match
// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).
struct BE { BE* mNext; BE* mPrev; };
struct S { unsigned pad; BE list; unsigned n; };   // list at 4, NOT 0
struct H {
    unsigned mSize;
    BE* mSpare;
    H(unsigned a, S* s);
    BE* Alloc(unsigned a);
};

H::H(unsigned a, S* s) {
    mSize = a;
    BE& l = s->list;
    l.mNext = 0;
    Alloc(a);
}
