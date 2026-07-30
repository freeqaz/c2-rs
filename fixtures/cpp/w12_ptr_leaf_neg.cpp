// **Negative** — everything one byte away from the pointer leaf accepted in
// `w12_ptr_leaf.cpp`. Every function here must keep refusing (`NotImplemented`),
// and the file must never produce a `mismatch`.
//
// The rung that admits pointer-valued loads is decode-only precisely *because*
// the accepted shapes emit instructions the port already emits. Every function
// below is a shape that looks the same in the IL to within one token and emits
// something the port does not have — so each is a wrong-bytes emit waiting for a
// gate that was written one notch too wide.
//
// ## The offset-add without a `30` is the dangerous one
//
// `n_addr_of` is `return &s->b;`. Its IL is the getter's, minus the load:
//
//   getter    B9 <s> <ptr> 33 <int> 4 27 <ptr> 30 <ptr> [2C <ptr> 00] 41 <ptr>
//   &s->b     B9 <s> <ptr> 33 <int> 4 27 <ptr>          [2C <ptr> 00] 41 <ptr>
//                                                        ^ no 30
//
// and it emits `addi r3,r3,4`, not a load and not nothing. A *literal* capture of
// this shape exists in the workload: `system/utl/MemMgr.cpp` at `.ex` 0x545e is
//
//   4c 4f 11 53 b9 a8 1c a6 43 96 23 33 86 41 74 0c 27 a2 43 aa 23
//                2c 86 43 91 20 00 41 86 43 91 20 3a …
//
// — 7 of the 40 pointer-shaped bodies in three scanned TUs are this. An identity
// recognizer that skipped an optional offset add before checking for the `30`
// would emit a bare `blr` for all of them. So the identity leaf is anchored on
// the `B9` *immediately* followed by the `2C`/`41`, and the getter requires the
// `30`; nothing accepts the shape in between.
//
// ## The rest, and what each emits instead
//
//   n_deref2   **ppp        two loads: lwz r3,0(r3) ; lwz r3,0(r3). One `30` is
//                           the whole accepted class; a second is a second
//                           instruction.
//   n_padd     p + 1        addi r3,r3,4 — c2 **scales by the element size**, so
//                           lowering a pointer add as an integer add would emit
//                           `addi r3,r3,1`. This is why the general expression
//                           operand gate is NOT widened by this rung: a pointer
//                           in an add chain needs the scale, and the arithmetic
//                           selector has no idea it is holding a pointer.
//   n_padd_v   p + i        slwi r11,r4,2 ; add r3,r11,r3 — the variable form,
//                           an extra instruction and a scratch register.
//   n_upcast   D* -> B*     addi r3,r3,4 (the second base's offset). This is the
//                           neighbour that would look identical under the wrong
//                           rule for `2C`: it is a pointer produced from a
//                           pointer, and it is NOT free. It does not come
//                           through `2C` at all — it is intrinsic 2113
//                           (`33 <int> 80 41 08 00 00 40 …`), which the parser
//                           has never accepted — and this function is here to
//                           keep that discriminating, not to be assumed.
//   n_glob     g_h->mpi     the base is a **global**, not a formal or `this`:
//                           lis r11,g_h@ha ; lwz r11,g_h@l(r11) ; lwz r3,0(r11)
//                           plus two relocations. `bind_params` only ever finds
//                           formals and `this`, so the token misses and the
//                           parse refuses — positively, by absence from a list
//                           it built, never by a failed search.
//   n_mr       (a, s) -> s  `mr r3,r4`: the identity is free only when the value
//                           is already in r3. Refused by the same
//                           `straight_line_is_out_of_class` the integer path
//                           uses — one predicate, not a second copy.
//   n_offbig   w->tail      offset 40000 does not fit the `lwz` 16-bit
//                           displacement; c2 materializes an index register.
//   n_null     (T*)0        a pointer **literal** — `33` not `B9` — needing an
//                           `li r3,0`. Censused as `expr-lit-type-8643xx` and
//                           deliberately left there: literals are unprobed.
//   n_store    *pp = p      a memory write, `stw`.
//   n_deref_c  *pc          the pointee is 1 byte: `lbz`. The loaded value's
//                           width, not the pointer's, picks the instruction —
//                           the exact confusion `gp_c` in the positives pins
//                           from the other side.
//   n_pdiff    p - q        subf then a signed divide-by-4 idiom.

struct S {
    int a;
    int b;
};

struct H {
    int* mpi;
};

struct A {
    int a0;
};
struct B {
    int b0;
};
struct D : A, B {
    int d0;
};

struct Wide {
    int pad[10000];
    int* tail;
};

H g_h;

int* n_addr_of(S* s) { return &s->b; }

int* n_deref2(int*** ppp) { return **ppp; }

int* n_padd(int* p) { return p + 1; }
int* n_padd_v(int* p, int i) { return p + i; }

B* n_upcast(D* d) { return d; }

int* n_glob() { return g_h.mpi; }

S* n_mr(int a, S* s) { return s; }

int* n_offbig(Wide* w) { return w->tail; }

int* n_null() { return 0; }

void n_store(int** pp, int* p) { *pp = p; }

char n_deref_c(char* pc) { return *pc; }

int n_pdiff(int* p, int* q) { return (int)(p - q); }
