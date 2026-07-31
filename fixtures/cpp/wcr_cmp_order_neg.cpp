// WCR negative — one case per refusal row of the two-call comparator's ORDER
// relations, each with its measured cost on the 878-TU dc3 workload recorded in
// the rung document. Every function here must census **0/N in class**;
// `c2rs census` is the check that each gate refuses and `Port=NotImplemented`
// the check that nothing slipped past it into codegen.

struct U {
    int m() const;
    int n() const;
    unsigned um() const;
    unsigned un() const;
    int ma(int) const;
    char mc() const;
    short ms() const;
    bool mb() const;
    float mf() const;
    double md() const;
    const U* mp() const;
    const void* mv() const;
    const char* mc2() const;
};

// n_rel_* — the three relations still refused, each **0 functions** on the
// workload in this shape (measured with the per-relation refusal keys
// `mcall-cmp-rel-{ne,le,ge}`, which is why this rung's residue is a number and
// not a rumour).
//
// `>=` and `<=` are not merely unbuilt: they are the two cells
// `docs/CMP_PRODUCES_A_VALUE.md` reading 1 names, where a **`bool` result is two
// words longer than an `int` one** (`adde` into a temp plus `clrlwi r3,t,24`,
// against `adde r3` and nothing). `a->m() >= b->n()` returns `bool`, so a rung
// admitting them on the strength of the `int` form would be two words short with
// `.pdata FuncLen` and both `$M` values wrong to match. `!=` is a different three
// words (`addic` + `subfe`).
bool n_rel_ne(const U* p, const U* q) { return p->m() != q->n(); }
bool n_rel_le(const U* p, const U* q) { return p->m() <= q->n(); }
bool n_rel_ge(const U* p, const U* q) { return p->m() >= q->n(); }
// …and the gate must not be order-dependent.
bool n_rel_ge_rev(const U* p, const U* q) { return q->n() >= p->m(); }
// …nor signedness-dependent: unsigned `>=`/`<=` are a *different* pair of spines
// (`li r10,-1 ; subfc ; subfze`) and equally unbuilt.
bool n_rel_u_ge(const U* p, const U* q) { return p->um() >= q->un(); }
bool n_rel_u_le(const U* p, const U* q) { return p->um() <= q->un(); }

// n_mixed_* — MIXED SIGNEDNESS. c1xx inserts an explicit `2C <unsigned> 00`
// convert on whichever side needs one, and the two positions are different
// grammar cells: with the `int` operand on the left the convert sits between the
// first `4C` and the second `26`, with it on the right it sits between the second
// `4C` and the relation. Both refuse in the grammar, and `operand_signedness`'s
// agreement check is the second lock behind them.
bool n_mixed_lhs(const U* p, const U* q) { return p->m() >  q->un(); }
bool n_mixed_rhs(const U* p, const U* q) { return p->um() >  q->n(); }
bool n_mixed_lt(const U* p, const U* q) { return p->m() <  q->un(); }
// …and the same in the POINTER class, where the two operands' `86 43` triples
// differ only in the pointee id: `void*` against `const char*` carries a convert
// too, so the class-agreement check in `operand_signedness` is behind a grammar
// refusal here exactly as the integer pair is.
bool n_mixed_ptr(const U* p, const U* q) { return p->mv() <  q->mc2(); }

// n_narrow_* / n_bool — the OPERANDS' type, first gate. A `char`/`short`/`bool`
// result carries a widening `2C` before the relation, so these refuse in the
// GRAMMAR and keep their `…-then-cmp-*-whole2` key. (A POINTER operand is NOT
// here: it takes the unsigned spine byte for byte and is in class — see
// `p_ptr_*` in the positive fixture. That row was written as a refusal and the
// workload said otherwise, 66 functions' worth.)
bool n_narrow_c(const U* p, const U* q) { return p->mc() > q->mc(); }
bool n_narrow_s(const U* p, const U* q) { return p->ms() < q->ms(); }
bool n_bool(const U* p, const U* q) { return p->mb() > q->mb(); }

// n_fp_* — **THE ROW THIS RUNG LEFT BEHIND, and it is the whole of it.** Of the
// 760 functions `mcall-cmp-rel` measured, **693 are these**: two calls returning
// `float`, compared with `>` or `<`. They reach the relation intact (no widening
// convert) and refuse at `mcall-cmp-rel-operand-type-8645`.
//
// They are not a spine away. `bool f(const U* p, const U* q)
// { return p->mf() > q->mf(); }` is
//
//     stfd f31,-24(r1) ; … ; fmr f31,f1 ; bl ; fcmpu cr6,f31,f1
//     li r11,1 ; bt 25,.+8 ; li r11,0 ; clrlwi r3,r11,24 ; … ; lfd f31,-24(r1)
//
// — a **conditional branch** inside the body and an **FP callee-saved register**
// saved with `stfd`/`lfd` beside the GPRs, plus `_fltused`. Basic blocks and an
// FPR frame model, neither of which this port has.
bool n_fp_f(const U* p, const U* q) { return p->mf() > q->mf(); }
bool n_fp_d(const U* p, const U* q) { return p->md() < q->md(); }

// n_result_narrow — the RESULT's type. `bool`, `int` and `unsigned` are the same
// bytes; a `char`/`short` result widens differently.
char  n_result_char (const U* p, const U* q) { return p->m() > q->n(); }
short n_result_short(const U* p, const U* q) { return p->m() < q->n(); }

// n_three — three calls: Class C, the `bl __savegprlr_29` helper, declined on
// measurement. The gate is the TOTAL saved count.
bool n_three(const U* p, const U* q, const U* r) { return p->m() > q->m() + r->m(); }

// n_arg_* — an explicit argument on either call: the marshalling interleaves with
// the callee-saved move and which is hoisted is what `plan_saved_gprs` refuses to
// guess.
bool n_arg_lhs(const U* p, const U* q, int k) { return p->ma(k) > q->m(); }
bool n_arg_rhs(const U* p, const U* q, int k) { return p->m() < q->ma(k); }

// n_recv_global — a receiver that is not one of this function's own formals: the
// emission is a register move and a global is a load.
extern U* g_u;
bool n_recv_global(const U* q) { return g_u->m() > q->n(); }

// n_recv_volatile — a `volatile` receiver is homed in the frame and reloaded.
bool n_recv_volatile(U* volatile p, const U* q) { return p->m() > q->n(); }

// n_nine — nine formals; past the eighth a parameter is stack-homed.
bool n_nine(int a, int b, int c, int d, int e, int f, int g,
            const U* p, const U* q) { return p->m() > q->n(); }

// n_branch — the comparison is BRANCHED on rather than returned. This is where
// the bulk of the family lives (`-and-branch-more`: 1,666 `>` and 1,900 `<` in
// the `recv-load` row alone) and every one of them needs basic blocks.
int n_branch(const U* p, const U* q) { if (p->m() > q->n()) return 1; return 2; }

// n_free — two FREE-function calls. No witness in this shape on the workload; the
// first call saves nothing and the register file is one register shifted.
int gf1(); int gf2();
bool n_free() { return gf1() > gf2(); }

// n_chained — the second receiver is another call's result: the
// `expr-call-in-expr-chained` production, and Class A rather than Class B.
struct X { const U* g() const; };
bool n_chained(const X* p, const U* q) { return p->g()->m() > q->n(); }

// n_one_call — ONE call and a formal, which is the shape
// `docs/CMP_PRODUCES_A_VALUE.md` measured at 66 and this rung still does not
// admit: it is Class B (the formal is live across the `bl`) but the difference is
// formed against a *parameter* rather than a second call's result. Its `<` cell
// is `expr-call-in-expr-recv-load-then-cmp-lt-and-type-int1-whole2`, **68
// functions** — the largest thing left in the *integer* comparison family, an
// order of magnitude below the `float` row above.
bool n_one_call(const U* p, int k) { return p->m() > k; }
