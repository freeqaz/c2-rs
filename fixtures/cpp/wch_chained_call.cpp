// WCH positive — `return p->a()->b();`, the chained member call as a whole body.
// The largest `-whole` row on the board (12,479 functions) and it is **Class A**:
// each call's result lands in r3, which is where the next call's `this` belongs,
// so nothing is ever live across a `bl` and nothing is saved.
//
// Every function here must census **N/N in class** and `c2rs diff` must report
// `Port=Match`. The rows are chosen so that each one is a fact the emitter would
// get wrong if it modelled a *different* rule that fits the others.

struct I {
    int         gi();
    void        vv();
    const I*    self();
    int         gia(int);
    int         gib(int, int);
};

struct O {
    I*  Next();
    I*  NextA(int k);
    I*  NextB(int j, int k);
    O*  Self();
    int oi();
};

// p_ret — the base shape, and the whole rung in nine words:
//   mflr r12 ; stw r12,-8(r1) ; stwu r1,-96(r1)
//   bl ?Next ; bl ?gi
//   addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; blr
int p_ret(O* p) { return p->Next()->gi(); }

// p_void — the statement form. The same 36 bytes: a discarded result is still
// left in r3 and the epilogue does not care.
void p_void(O* p) { p->Next()->vv(); }

// p_braced — the same body inside a brace scope, which closes BETWEEN the
// statement end and the return branch. Every other whole-body shape carries this
// row because the scope run sits on the far side of the `3A`.
void p_braced(O* p) { { p->Next()->vv(); } }

// p_three / p_four — CHAIN DEPTH IS FREE. `call_seq_text` takes one setup per
// call, so a third and a fourth link are one more `bl` each and four more bytes.
// A recognizer hardcoded at two links passes `p_ret` and emits nothing here.
int p_three(O* p) { return p->Self()->Next()->gi(); }
int p_four (O* p) { return p->Self()->Self()->Next()->gi(); }

// p_same_method — the same method twice in one chain. ONE callee external, not
// two: both `bl`s take the same REL24 target and the symbol table is one entry
// shorter than a model that emitted a symbol per link.
int p_same_method(O* p) { return p->Self()->Self()->oi(); }

// p_order — THE CELL THAT DECIDES THE RUNG. The method symbols stack LIFO, so
// `26 <gi>` is FIRST in the IL and `?gi` is called SECOND. A recognizer that
// walked the pushes in stream order emits both `bl`s to the wrong callees, and
// the two objs differ only in two REL24 symbol indices — invisible in any body
// whose links happen to call the same method, which is why `p_same_method` above
// cannot grade this and this row can: the two callees are distinct and their
// symbol-table positions differ.
const I* p_order(O* p) { return p->Next()->self(); }

// p_recv_slot — the receiver is NOT in r3, so the innermost call needs a real
// `mr r3,r4` before the first `bl`. `GAPS.md` §6 records four separate defects
// where a formal's index and its register were the same number in every fixture.
int p_recv_slot(int z, O* p) { return p->Next()->gi(); }

// p_arg_inner — an explicit argument on the INNERMOST link, already in its
// register: `this` is slot 0 and `k` is slot 1, the identity permutation, so the
// setup is empty and the body is `p_ret`'s exactly.
int p_arg_inner(O* p, int k) { return p->NextA(k)->gi(); }

// p_arg_inner_move — the same argument list with the formals declared the other
// way round, which is a 2-cycle: `mr r11,r4 ; mr r4,r3 ; mr r3,r11`. The
// innermost call marshals out of the argument registers with nothing clobbered
// yet, so it goes through the SAME permutation locator every tail call uses.
int p_arg_inner_move(int k, O* p) { return p->NextA(k)->gi(); }

// p_arg_inner_perm — two explicit arguments passed in the other order, over
// three formals: `mr r11,r5 ; mr r5,r4 ; mr r4,r11`, with r3 already in place.
int p_arg_inner_perm(O* p, int j, int k) { return p->NextB(k, j)->gi(); }

// p_this — the receiver is the implicit `this`, which is `params[0]` and already
// in r3. The setup is empty for the same reason `p_ret`'s is, but by a different
// route: the token comes from the `this` binding rather than from a `2D` formal.
// (`M` declares its own `Next()` rather than inheriting `O`'s: a call to an
// INHERITED method through `this` goes via the base-adjust intrinsic 2113, which
// is a different receiver production and is refused — see the negative fixture.)
struct M { I* Next(); int mm(); void mv(); };
int  M::mm() { return Next()->gi(); }
void M::mv() { Next()->vv(); }

// p_recv_cast — a pointer conversion on the receiver (`2C <ptr4> 00`). It emits
// nothing: the address is the same and the register does not move. Admitted
// through the shared receiver locator, not a copy of it.
int p_recv_cast(void* v) { return ((O*)v)->Next()->gi(); }

// p_neighbour_* — the `/Gy` label counter and the symbol-table order are only
// graded against a FOLLOWING function. A wrong per-function label stride is
// invisible in a one-function TU and six wrong bytes in this one.
int g_free(int);
int p_neighbour_framed(int a) { return g_free(a) + 1; }
int p_neighbour_leaf(int a) { return a + 1; }
int p_neighbour_cmp(int a) { return a == 3; }
