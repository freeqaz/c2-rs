// **Positive** — W36, the member call as a whole body. Every function here must
// emit, and the whole obj must be byte-exact.
//
// `p->m(x);` is `m(p, x)` on this ABI: `this` is argument **zero**, in r3, and
// nothing else about the call is different from a free-function one. So the whole
// production is the tail call the port has emitted since the MVP with one extra
// argument slot, and it needs no codegen at all — the receiver is appended to the
// argument list as slot 0 and `tail_call_shape` does the rest.
//
//   4c 4f 11 53                    LO, SS
//   26 <method>                    push the METHOD symbol   — the callee
//   b9 <recv> 86 43 88 20          LOAD the receiver        — `this`
//   99 86 43 89 20 00              bind it as argument zero
//   bd 82 12 30 00 80 05 10 00 00  CALL, void result
//   4c 4b                          apply, statement end
//                                                          => b ?set1@Obj@@…
//
// **This row was the largest single key on the board and it was never a missing
// token.** `mod.rs`'s body dispatch tells a call from an assignment by asking
// whether a `BD` follows the statement-head `26 <tok>`; for a member call it does
// not — the receiver sits in between — so the statement went to the assignment
// parser, which read the receiver as an ordinary LOAD and stopped on the `99`
// bind under `parse_expr`'s generic `expr` fall-through. 280,283 functions,
// 11.4 % of everything blocked, filed as an **opcode** (`expr-op-0x99`) while the
// identical production one byte different — `x = p->m();` — reached
// `mcall::classify` and was filed as a member call all along. `docs/GAPS.md` §6's
// unstable-*attribution* hazard, in the form that costs coverage rather than
// correctness.
//
// The receiver goes on the **end** of the argument list, not the front, because
// the list is in stream order (rightmost source argument first, slot `i` is
// `args[len-1-i]`). `p_swap` and `p_rot3` below are the cases where the two
// readings differ: get it backwards and they emit a different permutation.

struct Obj {
    int i;
    void v0();
    void v1(int);
    void v2(int, int);
    void v3(int, int, int);
    void vp(Obj *);
    void vpp(Obj *, int);
    int  g0() const;
    int  gk(int) const;
    Obj *gp();
};

// --- the minimal case: the receiver is the only formal, already in r3 ---------
void m_nullary(Obj *o)          { o->v0(); }           // b ?v0

// --- the receiver at every argument position: an identity, a 2-cycle, a
//     3-cycle. `this` occupies the slot the first formal would have. ----------
void a_r0(Obj *o, int a, int b) { o->v2(a, b); }       // identity, no moves
void a_r1(int a, Obj *o)        { o->v1(a); }          // 2-cycle over r3/r4
void a_r2(int a, int b, Obj *o) { o->v2(a, b); }       // 3-cycle over r3/r4/r5

// --- explicit-argument permutations, receiver in place ------------------------
void p_swap(Obj *o, int a, int b)        { o->v2(b, a); }
void p_rot3(Obj *o, int a, int b, int c) { o->v3(c, a, b); }

// --- the result is RETURNED rather than discarded. The callee leaves it in the
//     register the caller's own return uses, so it is the same bare branch. ----
int  r_int(Obj *o)              { return o->g0(); }
int  r_arg(Obj *o, int k)       { return o->gk(k); }
Obj *r_ptr(Obj *o)              { return o->gp(); }

// --- …and the result DISCARDED, which is the same production with `4B` where
//     the `41` result annotation would be. ------------------------------------
void d_int(Obj *o)              { o->g0(); }

// --- cv-qualified receivers. The qualifier changes no operator and no shape and
//     it does change the TYPE tag the operand gate reads, which is the axis
//     `docs/GAPS.md` §6's thirteenth live mis-emit hid behind. -----------------
int  cv_const_ptee(const Obj *o) { return o->g0(); }
int  cv_const_ptr(Obj *const o)  { return o->g0(); }

// --- a POINTER argument beside the pointer receiver. Two pointers in two
//     registers; the operand vocabulary already spells both. ------------------
void q_ptr(Obj *o, Obj *q)          { o->vp(q); }
void q_ptr_swap(Obj *q, Obj *o)     { o->vp(q); }
void q_ptr_int(Obj *o, Obj *q, int k) { o->vpp(q, k); }

// --- the receiver is `this`: a member function calling another method on
//     itself. `this` is params[0] here too, so the setup is empty. ------------
struct Self {
    int s;
    void t0();
    void t1(int);
    void go();
    void go_arg(int k);
    int  go_ret();
};
void Self::go()          { t0(); }
void Self::go_arg(int k) { t1(k); }

// --- a braced body: the statement is at lexical depth 3 and the plumbing has to
//     close the scope it opened, exactly as every other shape requires. -------
void b_scope(Obj *o) { { o->v0(); } }

// --- the MANGLED-NAME shapes the callee token has to resolve through `.gl`.
//     A wrong callee name is a relocation against the wrong symbol — a mis-emit,
//     not a gap — and `docs/GAPS.md` §6 records `gl_symbol_index` missing 12,505
//     of 33,059 `?`-mangled names because its anchor was a byte value rather than
//     a field. The sweep generates none of these, so they are pinned here: a
//     namespaced class, a nested one, an overload set, an operator, and a member
//     of a class template.
namespace ns { namespace inner {
struct NObj {
    int i;
    void v0();
    void ov(int);
    void ov(int, int);
    int  operator[](int) const;
};
} }
template <class T> struct Tpl { T t; void set(T); T get() const; };
struct Nest { struct In { int n; void nv(); }; };

void n_ns(ns::inner::NObj *o)               { o->v0(); }
void n_ovl1(ns::inner::NObj *o, int k)      { o->ov(k); }
void n_ovl2(ns::inner::NObj *o, int a, int b) { o->ov(a, b); }
int  n_op(ns::inner::NObj *o, int k)        { return (*o)[k]; }
void n_tpl_set(Tpl<int> *t, int k)          { t->set(k); }
int  n_tpl_get(Tpl<int> *t)                 { return t->get(); }
void n_nested(Nest::In *n)                  { n->nv(); }
