// WCB negative — one case per refusal row of the two-call comparator, each with
// its measured cost on the 878-TU dc3 workload recorded in the rung document.
// Every function here must census **0/N in class**: `c2rs census` on this file is
// the check that each gate below actually refuses, and `Port=NotImplemented` is
// the check that nothing slipped past it into codegen.

struct E20 { int a, b, c, d, e; };

struct U {
    int m() const;
    int n() const;
    int ma(int) const;
    char mc() const;
    short ms() const;
    float mf() const;
    double md() const;
};

// n_rel_* — any relation but `==`. MEASURED, and it is the largest thing this
// rung leaves behind: **760 functions** on the 878-TU workload move into
// `mcall-cmp-rel` (692 `>` and 68 `<`), and `!=` in this shape is **0**.
//
// They are refused rather than lowered because the four order relations are the
// five-word sign-sum spines, two of whose 24 cells change bytes with a `bool`
// result (`docs/CMP_PRODUCES_A_VALUE.md` reading 1) — a spine borrowed from
// `compare_leaf_text` as it stands is two words short there, with `.pdata
// FuncLen` and both `$M` values wrong to match. `!=` is a different three words
// (`addic` + `subfe`) and has no witness on the workload at all.
bool n_rel_ne(const U* p, const U* q) { return p->m() != q->n(); }
bool n_rel_lt(const U* p, const U* q) { return p->m() <  q->n(); }
bool n_rel_le(const U* p, const U* q) { return p->m() <= q->n(); }
bool n_rel_gt(const U* p, const U* q) { return p->m() >  q->n(); }
bool n_rel_ge(const U* p, const U* q) { return p->m() >= q->n(); }
// …and the gate must not be order-dependent: the same relation with the calls in
// the other source order refuses too.
bool n_rel_gt_rev(const U* p, const U* q) { return q->n() >  p->m(); }

// n_three — three calls, so three values are live at once and c2 switches to
// `bl __savegprlr_29`: a second REL24 site, a **tail-branch epilogue with no
// `blr` at all**, and two extra `/Gy` label slots taken ahead of the function's
// own `$M` pair. That is Class C, declined on measurement
// (`docs/rungs/2026-07-31-frame-class-c-declined.md`), and the gate for it is the
// TOTAL saved count, not the saved-formal count — which is the whole reason
// `plan_saved_gprs` takes an `extra_saved` argument.
bool n_three(const U* p, const U* q, const U* r) { return p->m() == q->m() + r->m(); }
bool n_three2(const U* p, const U* q, const U* r) { return p->m() + q->m() == r->m(); }

// n_arg_* — an explicit argument on either call. The marshalling writes an
// argument register beside the callee-saved move, and which of the two is hoisted
// is exactly the rule `plan_saved_gprs` refuses to guess: a save whose source the
// marshalling overwrites is hoisted and one it leaves alone trails, and the model
// that assumed otherwise was wrong on 11 of 17 probes.
bool n_arg_lhs(const U* p, const U* q, int k) { return p->ma(k) == q->m(); }
bool n_arg_rhs(const U* p, const U* q, int k) { return p->m() == q->ma(k); }

// n_narrow_* / n_fp_* / n_ptr — the OPERANDS' type. `char` and `short` results
// are sign/zero-extended before the difference, and a `float`/`double` one makes the
// TU carry `_fltused` and compares in the FP file. Refused at the operand type,
// not guessed at. (A POINTER operand is NOT here: it is the same three words and
// it is in class — see `p_ptr_result` in the positive fixture. That row was
// written as a refusal and the census said otherwise.)
bool n_narrow_c(const U* p, const U* q) { return p->mc() == q->mc(); }
bool n_narrow_s(const U* p, const U* q) { return p->ms() == q->ms(); }
bool n_fp_f(const U* p, const U* q) { return p->mf() == q->mf(); }
bool n_fp_d(const U* p, const U* q) { return p->md() == q->md(); }

// n_result_narrow — the RESULT's type. `bool`, `int` and `unsigned` are the same
// bytes; a `char`/`short` result widens differently and the annotation must
// restate the value's class or the body is the `rlwinm` mask of a widening.
char  n_result_char (const U* p, const U* q) { return p->m() == q->n(); }
short n_result_short(const U* p, const U* q) { return p->m() == q->n(); }

// n_recv_global — a receiver that is not one of this function's own formals. The
// emission is a register *move*; a global is a load.
extern U* g_u;
bool n_recv_global(const U* q) { return g_u->m() == q->n(); }

// n_recv_volatile — a `volatile` receiver. c2 homes the parameter in the frame
// and reloads it, so the register model is wrong; refused through the shared
// operand-type locator, which is how this position inherits `GAPS.md` §6
// instance #13 rather than restating it.
bool n_recv_volatile(U* volatile p, const U* q) { return p->m() == q->n(); }

// n_nine — nine formals. Past the eighth a parameter is stack-homed and reading
// it is `lwz r3,<slot>(r1)`, not a register move. The refusal is on the whole
// formals LIST, the same predicate `select_text` raises for every framed shape.
bool n_nine(int a, int b, int c, int d, int e, int f, int g,
            const U* p, const U* q) { return p->m() == q->n(); }

// n_branch — the comparison's value is BRANCHED on rather than returned. 9,490
// functions of this family (`-and-branch-more`) and every one of them needs basic
// blocks; refused here so the row keeps the key that says so.
int n_branch(const U* p, const U* q) { if (p->m() == q->n()) return 1; return 2; }

// n_free — two FREE-function calls rather than member ones. The receiver-less
// form has no witness in this shape on the workload and its first call saves
// nothing, so the register file is one register shifted; not admitted.
int gf1(); int gf2();
bool n_free() { return gf1() == gf2(); }

// n_chained — the second "receiver" is another call's result, which is the
// `expr-call-in-expr-chained` production (12,479 functions on the workload, and
// **Class A** — no saved GPR at all). A different rung, refused by name here.
struct X { const U* g() const; };
bool n_chained(const X* p, const U* q) { return p->g()->m() == q->n(); }

// n_this_adjust — an INHERITED method called through `this`. The receiver is not
// a plain `B9 <tok> <ptr4>` but the base-adjust intrinsic 2113, a different
// receiver production with its own lowering (`expr-intrinsic-this-adjust`, the
// second-largest key on the board at 141,800). Refused before the comparison is
// even read, which is why the positive fixture's `H` declares its own `m()`.
struct B { int m() const; };
struct D : B { bool q(const U* a) const; };
bool D::q(const U* a) const { return m() == a->m(); }
