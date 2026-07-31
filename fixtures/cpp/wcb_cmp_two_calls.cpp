// WCB positive — `return a->m() == b->n();`, two member calls in one expression
// with the first result live across the second `bl`. The port's first **Class B**
// production: two callee-saved GPRs, `std`/`ld` inline, a 112-byte frame.
//
// Every function here must census **N/N in class** and `c2rs diff` must report
// `Port=Match`. The rows are chosen so that each one is a fact the emitter would
// get wrong if it modelled a *different* rule that fits the others.

struct U {
    int m() const;
    int n() const;
    int o() const;
    unsigned mu() const;
    const U* mp() const;
};

// p_plain — the base shape. `mr r31,r4 ; bl ?m ; mr r30,r3 ; mr r3,r31 ; bl ?n`,
// then `subf r11,r30,r3 ; cntlzw r10,r11 ; rlwinm r3,r10,27,31,31`.
bool p_plain(const U* p, const U* q) { return p->m() == q->n(); }

// p_same_callee — both calls to the SAME method. One callee external, not two;
// the reverse-first-reference LIFO has nothing to order and a model that emitted
// a second symbol would be one symbol long.
bool p_same_callee(const U* p, const U* q) { return p->m() == q->m(); }

// p_reordered — THE CELL THAT DECIDES THE RUNG. The source's left operand is
// `q->n()` and c2 emits `p`'s call FIRST anyway, because the two calls are
// ordered by the receiver's IL token. The prologue, both `mr`s and both `bl`s are
// byte-identical to `p_plain`; only the spine's `subf` operands swap. A model
// that emitted the calls in source order gets four words wrong here and none in
// `p_plain`.
bool p_reordered(const U* p, const U* q) { return q->n() == p->m(); }

// p_recv_slot — neither receiver is in r3, so the first call needs a marshalling
// `mr r3,r4` AND the save `mr r31,r5`, in that order (the save's source is not
// overwritten, so it trails). `GAPS.md` §6 records four defects where a formal's
// index and its register were the same number in every fixture.
bool p_recv_slot(int z, const U* p, const U* q) { return p->m() == q->n(); }
bool p_recv_slot_rev(int z, const U* p, const U* q) { return q->n() == p->m(); }

// p_same_recv — one receiver, two methods. Still TWO saved GPRs: the receiver has
// to survive the first `bl` and so does the first result. The two receiver tokens
// are equal, which is the one cell where the ordering rule falls back to IL order.
bool p_same_recv(const U* p) { return p->m() == p->n(); }
bool p_same_recv_rev(const U* p) { return p->n() == p->m(); }
bool p_same_recv_slot(int z, const U* p) { return p->m() == p->n(); }

// p_int / p_unsigned — the SAME BYTES with a different IL spelling: an integer
// result carries `2C <int4> 00` and annotates `41 <int4>`, a `bool` result carries
// neither and annotates `41 <int1>`. The class has to be *restated* by the
// annotation or the value is the `rlwinm` mask of a widening.
int p_int(const U* p, const U* q) { return p->m() == q->n(); }
unsigned p_unsigned(const U* p, const U* q) { return p->m() == q->n(); }

// p_unsigned_operands — unsigned operands. `==` is sign-agnostic and the opcode
// does not carry the signedness; the operand type does, and it changes nothing.
bool p_unsigned_operands(const U* p, const U* q) { return p->mu() == q->mu(); }

// p_this — `this` has parameter index 0 and the HIGHEST token, because c1xx
// numbers the implicit receiver after the declared formals. So `a`'s call goes
// first even though `this` is the source's left operand and sits in r3: the
// prologue saves `this` (`mr r31,r3`) and marshals `a` into r3 (`mr r3,r4`), a
// HOISTED save — the only shape in this fixture that takes that arm. A model
// ordering by parameter index emits both moves backwards.
// (`H` declares its own `m()` rather than inheriting one: a call to an inherited
// method through `this` goes via the base-adjust intrinsic 2113, which is a
// different receiver production and is refused — see the negative fixture.)
struct H {
    int m() const;
    bool q1(const U* a) const;
    bool q2(const U* a) const;
    bool q3(const U* a, const U* b) const;
};
bool H::q1(const U* a) const { return m() == a->m(); }
bool H::q2(const U* a) const { return a->m() == m(); }
// …and `this` unused as a receiver: nothing saves it, and the two declared
// formals order normally.
bool H::q3(const U* a, const U* b) const { return b->m() == a->m(); }

// p_ptr_result — the compared values are POINTERS. The difference spine does not
// care: `subf` over two addresses is the same three words, and the result is the
// same 0/1. Measured, not assumed.
bool p_ptr_result(const U* p, const U* q) { return p->mp() == q->mp(); }

// p_recv_cast — a pointer conversion on a receiver (`2C <ptr4> 00`), which emits
// nothing: the address is the same and the register does not move.
bool p_recv_cast(void* v, const U* q) { return ((const U*)v)->m() == q->n(); }

// p_neighbour_* — the `/Gy` label counter and the symbol-table order are only
// graded against a FOLLOWING function. This shape introduces two callee externals
// and takes the plain framed stride of 5 with no leading surcharge; a wrong
// surcharge is invisible in a one-function TU and six wrong bytes in this one.
int g_free(int);
int p_neighbour_framed(int a) { return g_free(a) + 1; }
int p_neighbour_leaf(int a) { return a + 1; }
// (`== 3` and not `<= 3`: a signed `<=` comparison leaf consumes THREE label
// slots and is already refused beside any framed function — `wfr_cmp_stride_neg`.
// `==` consumes one, so it is the neighbour that grades this shape's stride
// rather than the neighbour's own.)
int p_neighbour_cmp(int a) { return a == 3; }
