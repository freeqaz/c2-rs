// **Negative** — W36's boundary. NOT ONE function here may census in class, and
// the whole TU must read `Port=NotImplemented`.
//
// Every case is a neighbour of `p->m(a…);` that lowers to something other than a
// register permutation plus `b <method>`, or that this port has not separated.
// Each carries its measured cost on the 878-TU dc3 workload where one is known;
// `UNMEASURED` means the row exists and its size has not been taken, and
// `structural` means the walk cannot reach the case at all.

struct Obj {
    int i;
    void v0();
    void v1(int);
    void v2(int, int);
    void v3(int, int, int);
    void v4(int, int, int, int);
    void v8(int, int, int, int, int, int, int, int);
    int  g0() const;
    int  gk(int) const;
    float gf();
    long long gll();
    Obj *nxt();
};
struct Wrap { int w; Obj *o; Obj em; };
struct Val  { int a, b; };
struct Ret  { Val gv(); };
struct Base { void bm(); };
struct Der : Base { int d; };
struct Virt { virtual void vf(); };
extern Obj gObj;
extern int gInt;

// (1) A COMPUTED argument beside the receiver. `o->gk(k+1)` is `gk(o, k+1)` —
//     `addi r4,r4,1 ; b gk` — and the multi-argument tail call models only a pure
//     register permutation, because which register a computed argument is
//     evaluated into interacts with the permutation temp and no capture covers
//     it. The single-argument form has always lowered arg setup; adding `this`
//     makes every such call multi-argument. **396 functions**, measured by
//     diffing the scan with and without this rung.
int  n_argk(Obj *o, int k) { return o->gk(k + 1); }

// (2) A permutation with a cycle longer than three. `permute_args_text` is right
//     at 2 and 3 and WRONG at 4 and 5 (`docs/GAPS.md` §6 instance #10, measured
//     over the complete grids), so the receiver being slot 0 does not change the
//     gate — it only makes the cycle one longer. Shared with the free-function
//     tail call, one locator. **structural** (the gate predates this rung).
void n_cycle4(int a, int b, int c, Obj *o) { o->v3(a, b, c); }

// (3) A permutation with more than one cycle: c2's own schedule past one cycle is
//     not characterized. Shared gate, same reason.
void n_multicycle(Obj *o, int a, int b, int c, int d) { o->v4(d, c, b, a); }

// (4) An argument that is not one of this function's formals — a global read is
//     a relocation and a load, not a register move. **4 functions.**
void n_global_arg(Obj *o) { o->v1(gInt); }

// (5) A receiver that is not a formal: a **named object**, whose address has to
//     be materialized with a `lis`/`addi` relocation pair. The census names it
//     `expr-call-in-expr-recv-object-…` — a different receiver production with a
//     different lowering, not a narrower one.
void n_recv_object() { gObj.v0(); }

// (6) A receiver that is a **member** of another object: `w->o` is an indirect
//     load first (`recv-deref`), and `&w->em` a byte-offset add (`recv-field`).
//     Both are addresses this body has to compute before the call.
void n_recv_deref(Wrap *w) { w->o->v0(); }
void n_recv_field(Wrap *w) { w->em.v0(); }

// (7) A receiver that is another CALL's result — `recv-call`, and necessarily a
//     `calls-2plus` body, which needs a frame before it needs anything else.
void n_recv_call(Obj *o) { o->nxt()->v0(); }

// (8) A method inherited from a non-virtual BASE. `this` is adjusted by intrinsic
//     2113 rather than passed straight through, which is a different production
//     (`expr-intrinsic-this-adjust`, 141,800 functions on the workload).
void n_base_method(Der *d) { d->bm(); }

// (9) VIRTUAL dispatch. Opcode `67` with a `9A` bind, not `99` — the whole reason
//     reading a `99` as a branch to a named callee is licensed at all.
void n_virtual(Virt *v) { v->vf(); }

// (10) A return type outside the modeled width-4 integer / pointer class. The
//      value is in the right register either way, but the `41` result annotation
//      is the port's own gate and it is shared with every other shape.
float     n_ret_float(Obj *o) { return o->gf(); }
long long n_ret_ll(Obj *o)    { return o->gll(); }

// (10b) …and a `float`/`double` result that is **DISCARDED**, which is the one
//      the `41` annotation cannot catch because there is no annotation. The call
//      is a bare `b <method>` and touches no FP register, and the obj still
//      carries `_fltused` — so the port emitted one symbol too few,
//      `Port=Mismatch @ offset 12`. `docs/GAPS.md` §6 instance **#14**, and it
//      was **live on mainline for the free-function form** (`float gf(); void
//      f(){ gf(); }`) long before this rung; W36's generated sweep found it on
//      the axis "the callee's return type, crossed with discarded and returned",
//      which no fixture had ever varied. Refused under `call-ret-fp` at the one
//      locator every call shape goes through.
void n_ret_float_discarded(Obj *o) { o->gf(); }

// (11) A struct returned BY VALUE. c1xx spells it with a `9B` temporary bind and
//      a hidden buffer pointer, so it never reaches this production — which is
//      what keeps the port from tail-calling a call whose r3 is not `this`.
void n_ret_struct(Ret *r) { r->gv(); }

// (12) A FLOAT argument. It travels in the other register file and the integer
//      operand vocabulary cannot spell it (`expr-load-type-8645`).
void n_float_arg(Obj *o, float f) { o->v1((int)f); }

// (13) A second STATEMENT after the call. The body is no longer terminal, so the
//      call is not a tail call at all — it is the Class A statement sequence with
//      a member call in it, a further rung. Falls through with its measured
//      second-blocker key rather than being claimed here.
void n_two_stmts(Obj *o, Obj *q) { o->v0(); q->v0(); }

// (14) Nine argument registers: `this` plus eight explicit. Past the eighth a
//      parameter is stack-homed and the setup is a store, not a move.
void n_nine(Obj *o, int a, int b, int c, int d, int e, int f, int g, int h)
{
    o->v8(a, b, c, d, e, f, g, h);
}
