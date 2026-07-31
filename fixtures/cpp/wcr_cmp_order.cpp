// WCR positive — `return a->m() > b->n();`, the two-call comparator's ORDER
// relations. WCB (`wcb_cmp_two_calls.cpp`) built the `==` cell of this
// production; this is `>` and `<`, which are the whole of what it left behind
// (`mcall-cmp-rel` = 760: 692 `>`, 68 `<`, 0 for the other three).
//
// Every function here must census **N/N in class** and `c2rs diff` must report
// `Port=Match`. The rows are chosen so that each one is a fact the emitter would
// get wrong if it modelled a *different* rule that fits the others.
//
// Three things separate this from the `==` cell, and every one of them is a
// register number rather than an opcode:
//
//   * SIGNEDNESS is not in the operator byte (`0x22` is both `<`s). Signed is a
//     five-word `subfc`/`eqv`/`srwi`/`addze`/`clrlwi` spine; unsigned is a
//     three-word `subfc`/`subfe`/`clrlwi` one.
//   * `<` is `>` with the spine's two operands exchanged, and `lhs_first` is a
//     SECOND exchange decided by c2's call order. They compose.
//   * a SIGNED order comparator takes **two extra label-counter slots ahead of
//     its own `$M` triple** (stride 7 under `/Gy`, 6 packed) and an unsigned one
//     takes none — the first per-function leading count in the port.

struct S { double d; };

struct U {
    int m() const;
    int n() const;
    unsigned um() const;
    unsigned un() const;
    long ml() const;
    unsigned long mul() const;
    const U* mp() const;
    const void* mv() const;
    const char* mc() const;
    double* md() const;
    S* ms() const;
};

// p_gt / p_lt — the base signed spines. `mr r31,r4 ; bl ?m ; mr r30,r3 ;
// mr r3,r31 ; bl ?n`, then `subfc r11,r30,r3 ; eqv r10,r30,r3 ; srwi r9,r10,31 ;
// addze r8,r9 ; clrlwi r3,r8,31` — and `<` is those same five words with the
// `subfc`'s and the `eqv`'s two register operands exchanged.
bool p_gt(const U* p, const U* q) { return p->m() >  q->n(); }
bool p_lt(const U* p, const U* q) { return p->m() <  q->n(); }

// p_gt_rev / p_lt_rev — THE CELLS THAT DECIDE THE RUNG. The source's left operand
// is `q->n()` and c2 emits `p`'s call FIRST anyway (the calls are ordered by the
// receivers' allocation rank). Every word up to the spine is byte-identical to
// `p_gt`; only the two operand-carrying words swap. Crossed with the relation,
// these four functions are the grid where the two exchanges compose — a model
// that applies one of them and not the other emits `p_lt` for `p_gt_rev`.
bool p_gt_rev(const U* p, const U* q) { return q->n() >  p->m(); }
bool p_lt_rev(const U* p, const U* q) { return q->n() <  p->m(); }

// p_u_gt / p_u_lt — UNSIGNED operands, the three-word spine, and the same two
// exchanges. Nothing in the operator byte distinguishes these from `p_gt`/`p_lt`:
// only the callees' result TYPE does (`86 42 75` against `86 41 74`).
bool p_u_gt(const U* p, const U* q) { return p->um() >  q->un(); }
bool p_u_lt(const U* p, const U* q) { return p->um() <  q->un(); }
bool p_u_gt_rev(const U* p, const U* q) { return q->un() > p->um(); }

// p_long_* — `long` and `unsigned long` are the same two 4-byte classes with
// different type ids, so they must take the same two spines. A predicate keyed on
// the literal triple `86 41 74` rather than on the class would refuse these.
bool p_long_gt(const U* p, const U* q) { return p->ml() > q->ml(); }
bool p_ulong_lt(const U* p, const U* q) { return p->mul() < q->mul(); }

// p_int_* / p_unsigned_result — the RESULT type against the OPERAND type. They
// are two facts and only the operand one picks the spine: `p_int_u_gt` has an
// `int` result (`2C 86 41 74 00`) over *unsigned* operands and is the three-word
// spine. Reading the convert instead of the call tokens emits five words here.
int p_int_gt(const U* p, const U* q) { return p->m() > q->n(); }
int p_int_u_gt(const U* p, const U* q) { return p->um() > q->un(); }
unsigned p_unsigned_result_lt(const U* p, const U* q) { return p->m() < q->n(); }

// p_ptr_* — **66 OF THIS RUNG'S 67 REALIZED FUNCTIONS.** Two pointers under an
// order relation take the UNSIGNED spine, byte for byte: `subc r11,r30,r3 ;
// subfe r11,r11,r11 ; clrlwi r3,r11,31`, the same three words `p->um() < q->un()`
// emits. Read off a reference obj, not deduced from "a pointer is an unsigned
// number" — and the pointee width does not enter it, because a result-position
// pointer TYPE is `86 43 <pointee id>` for every one of these.
bool p_ptr_lt(const U* p, const U* q) { return p->mp() < q->mp(); }
bool p_ptr_gt(const U* p, const U* q) { return p->mp() > q->mp(); }
bool p_ptr_rev(const U* p, const U* q) { return q->mp() < p->mp(); }
bool p_ptr_void(const U* p, const U* q) { return p->mv() < q->mv(); }
bool p_ptr_char(const U* p, const U* q) { return p->mc() > q->mc(); }
bool p_ptr_double(const U* p, const U* q) { return p->md() < q->md(); }
bool p_ptr_struct(const U* p, const U* q) { return p->ms() > q->ms(); }

// p_recv_slot — neither receiver is in r3, so the first call needs a marshalling
// `mr r3,r4` as well as the save. The register file moves; the spine does not.
bool p_recv_slot(int z, const U* p, const U* q) { return p->m() > q->n(); }
bool p_recv_slot_rev(int z, const U* p, const U* q) { return q->n() > p->m(); }

// p_same_recv — one receiver, two methods, equal receiver tokens: the tie where
// the ordering rule falls back to IL order. Two saved GPRs either way.
bool p_same_recv(const U* p) { return p->m() > p->n(); }
bool p_same_recv_u(const U* p) { return p->um() < p->un(); }

// p_this_* — `this` is parameter index 0 and the HIGHEST token, so `a`'s call
// goes first although `this` is the source's left operand and sits in r3. That
// makes the save HOISTED (`mr r31,r3 ; mr r3,r4`) — and under an order relation a
// wrong call order also swaps the spine's operands, so it is four wrong words
// rather than two.
struct H {
    int m() const;
    unsigned um() const;
    bool q1(const U* a) const;
    bool q2(const U* a) const;
    bool q3(const U* a) const;
};
bool H::q1(const U* a) const { return m() >  a->m(); }
bool H::q2(const U* a) const { return a->m() > m(); }
bool H::q3(const U* a) const { return um() < a->um(); }

// p_recv_cast — a pointer conversion on a receiver (`2C <ptr4> 00`), which emits
// nothing.
bool p_recv_cast(void* v, const U* q) { return ((const U*)v)->m() > q->n(); }

// p_neighbour_* — THE LABEL SURCHARGE, which only a following function grades.
// `p_gt` above takes 2 leading slots and then the framed 5 (`/Gy`) or 4 (packed),
// so every function after it in this file is at `+2` against where WCB's `==`
// shape would put it. An unsigned comparator pays nothing, which is why one of
// each is in this TU.
int g_free(int);
int p_neighbour_framed(int a) { return g_free(a) + 1; }
int p_neighbour_leaf(int a) { return a + 1; }
// (`== 3`, not `<= 3`: a signed `<=` comparison LEAF consumes three label slots
// and is refused beside any framed function — a different rule, in the leaf's own
// `label_slots`.)
int p_neighbour_cmp(int a) { return a == 3; }
