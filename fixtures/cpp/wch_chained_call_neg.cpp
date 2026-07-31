// WCH negative — one case per refusal row of the chained member call, each with
// its measured cost on the 878-TU dc3 workload recorded in the rung document.
// Every function here must census **0/N in class**: `c2rs census` on this file is
// the check that each gate below actually refuses, and `Port=NotImplemented` is
// the check that nothing slipped past it into codegen.

struct I {
    int      gi();
    int      gia(int);
    float    gf();
    const I* self();
};

struct O {
    I*  Next();
    I*  NextA(int k);
    I** pNext();
    O*  Self();
};

// n_link_arg_formal — a FORMAL argument on a later link. It has to survive the
// first `bl`, so the body is **Class B** — `std r31 ; mr r31,r4 ; bl ?Next ;
// mr r4,r31 ; bl ?gia` — and the save/marshalling interleave is exactly the rule
// `plan_saved_gprs` refuses to guess.
int n_link_arg_formal(O* p, int k) { return p->Next()->gia(k); }

// n_link_arg_lit — a LITERAL argument on a later link. Class A, 40 bytes, and
// still refused: the setup is `li r4,7`, into r4, and `select_text` — the one
// argument-setup locator `call_seq_parts` calls for every setup — computes into
// r3 and only r3. The two cells share one key because they share one cause: a
// later call's argument register is not r3.
int n_link_arg_lit(O* p) { return p->Next()->gia(7); }

// n_arg_computed — a COMPUTED argument on the innermost link. With `this` in the
// list that is a two-argument call, and the multi-argument path models only a
// pure permutation of bare formals; a computed argument needs its own register
// and interacts with the permutation temp in ways no capture covers.
int n_arg_computed(O* p, int k) { return p->NextA(k + 1)->gi(); }

// n_recv_global — the chain's innermost receiver is a named object. The last
// symbol push IS the receiver, so no `B9` designator follows and the production
// declines: a global is re-materialized from its address, which is a different
// lowering. It keeps its own `expr-call-in-expr-chained-…` census key.
extern O g_o;
int n_recv_global() { return g_o.Next()->gi(); }

// n_recv_deref — the receiver is read from memory (`… 30 <T>`), one load before
// the first `bl`. A different receiver designator with its own production.
int n_recv_deref(O** pp) { return (*pp)->Next()->gi(); }

// n_recv_field — the receiver is a pointer member at a nonzero byte offset,
// which costs an `addi`/`lwz` before the branch. Again a different designator.
struct W { int pad; O* o; };
int n_recv_field(W* w) { return w->o->Next()->gi(); }

// n_postop — the chain's result consumed by a literal `+ k` and then returned.
// That is the `-then-…` half of the family (`chained-then-type-ptr-and-op-more`
// is 15,049 functions on the workload, larger than this rung's own row) and it
// is a different tail; refused by name so the row keeps the key that says so.
int n_postop(O* p) { return p->Next()->gi() + 1; }

// n_deref_result — the chain's result dereferenced. The same `-then-…` family
// (`chained-then-deref-load-more`, 717) and the same reason.
struct V { int f; };
struct OV { V* Get(); };
struct OO { OV* Next(); };
int n_deref_result(OO* p) { return p->Next()->Get()->f; }

// n_branch — the chain's result BRANCHED on rather than returned. Needs basic
// blocks; refused here so the row keeps the key that says so.
int n_branch(O* p) { if (p->Next()->gi()) return 1; return 2; }

// n_second_stmt — a chain followed by a second statement. The body does not end
// at the chain, so it is the Class A statement sequence with a chain in it —
// a further rung, refused by name rather than routed into a production that has
// never been graded with a chained receiver.
void gv();
void n_second_stmt(O* p) { p->Next()->gi(); gv(); }

// n_fp_discarded — a discarded `float` result. The TU still has to carry the
// undefined external `_fltused`, which the port has no model of; asked through
// the shared `CallRet` so this position cannot drift from the others.
void n_fp_discarded(O* p) { p->Next()->gf(); }

// n_recv_volatile — a `volatile` receiver. c2 homes the parameter in the frame
// and reloads it, so the register model is wrong; refused through the shared
// operand-type locator, which is how this position inherits `GAPS.md` §6
// instance #13 rather than restating it.
int n_recv_volatile(O* volatile p) { return p->Next()->gi(); }

// n_nine — nine formals. Past the eighth a parameter is stack-homed and reading
// it is `lwz r3,<slot>(r1)`, not a register move. The refusal is on the whole
// formals LIST, the same predicate `select_text` raises for every framed shape.
int n_nine(int a, int b, int c, int d, int e, int f, int g, int h, O* p) {
    return p->Next()->gi();
}

// n_cmp — two chains compared. The first chain's result is live across the
// second chain's calls, which is Class B with a comparison spine on top: the
// two-call comparator's row (WCB) crossed with this one, and neither production
// admits it.
struct OI { I* Next(); };
bool n_cmp(OI* p, OI* q) { return p->Next()->gi() == q->Next()->gi(); }
