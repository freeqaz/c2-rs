// **Board #1274 — AN INTERIOR ADDRESS IN A STORE'S VALUE POSITION IS A
// PRODUCER**, positive fixture. `w-midrun`, the emitter rung at the bottom of
// `src/xdk/nuispeech/xboxheap.cpp`'s ladder.
//
// `&h->mBlk` materialises one `addi rD, rBase, off`. That is a producer in
// exactly the sense a literal is — it occupies a slot in `order::schedule`'s
// layout and takes a register from `alloc::allocate` — and it differs from a
// literal only in `alloc::ProducerKind`. Board **#1218** measured that the port
// could not say so at all: *"the producer list is built only from `s.lit`, so an
// interior-address producer is invisible to BOTH `order::schedule` and
// `alloc::allocate`"*.
//
// **TWO SPELLINGS, ONE PRODUCER, TWO BODIES.** `codegen::leaf::store::Prod`'s
// own equality collapses the spellings — a bound reference (`IlOp::BoundAddr`,
// board #1199) and the four-op `[Load(vb), AddrOf{off}]` group are the same
// address and c2 emits one `addi` for both. The store's base SYMBOL does *not*
// collapse: that is board **#1128**, and it is why `fn_bind1` and `fn_direct1`
// below — the same two statements, one IL bind apart — emit the stores in
// DIFFERENT orders and both are predicted from the base symbol alone:
//
//     fn_bind1     addi 11,3,20 ; stw 11,20(3) ; stw 5,16(3)     source order
//     fn_direct1   addi 11,3,20 ; stw 5,16(3) ; stw 11,20(3)     [1, 0]
//
// `w-carrier` §4.2 measured that pair **byte-IDENTICAL** — at ZERO formal
// stores, the one arrangement where one base symbol and two agree — and declined
// the family on the strength of it. Crossing the spelling with the formal-store
// count is what separates them.
//
// **The clause `store-run-bind-address-producer` used to refuse this and does
// not any more; it is NARROWED rather than deleted.** What still refuses, and
// what `w1199_bind_run_neg.cpp` still carries a witness for: an address BESIDE a
// literal (`xboxheap.cpp`'s own shape — peer rung, boards #836/#868/#1134), TWO
// distinct addresses, and displacement 0 (c2 materialises nothing at all).
//
// GRID M — `work/w-midrun/grid`, 94 cells sha256'd before the first `cl.exe` —
// grades every function here and 70 more against real `c2.dll` under wibo at the
// workload's own `/GR /O1 /Oi /EHsc`: **76 of 76 in-domain cells byte-exact, 0
// mismatch.** Every word below is read off those objs, never off the models.
//
// Consumed by `c2rs perf` (the fixture port gate) AND by
// `crates/c2-harness/tests/differential.rs`'s
// `differential_w1274_interior_address_producer_byte_exact`, which names it
// explicitly — a fixture no test names is not in the test lane, whatever the
// directory suggests.

struct BE { BE* n0; BE* n1; BE* n2; BE* n3; };
struct H {
    unsigned mA;       // 0
    BE mBlk;           // 4   n0@4 n1@8 n2@12 n3@16
    unsigned mB;       // 20
    BE* mP0;           // 24
    BE* mP1;           // 28
    H(unsigned p, unsigned q);
    void lf1(unsigned p, unsigned q);
    void ld1(unsigned p, unsigned q);
    void lf3(unsigned p, unsigned q);
    void lother(unsigned p, unsigned q);
    BE* Grab(unsigned n);
};

// GRID M `m_bl_u1_f1_af` — the BIND spelling. One address, one use, one formal
// store, the address store leading. Two base symbols, so the run keeps source
// order and the producer sits at slot 0.
void H::lf1(unsigned p, unsigned q) {
    BE& r = mBlk;
    r.n0 = &r;
    mA = q;
}

// GRID M `m_dl_u1_f1_af` — the DIRECT spelling of the SAME two statements, a
// four-op group. ONE base symbol, so `order::store_order`'s floor moves the
// produced store off position 0 and the two objs part company.
void H::ld1(unsigned p, unsigned q) {
    mBlk.n0 = &mBlk;
    mA = q;
}

// GRID M `m_bl_u3_f0` — THREE uses of one address and nothing beside it. The
// arity axis: one `addi`, three stores, no second producer. Every fixture and
// generated block before this rung had at most one word beside the address.
void H::lf3(unsigned p, unsigned q) {
    BE& r = mBlk;
    r.n0 = &r;
    r.n1 = &r;
    r.n2 = &r;
}

// GRID M `o_dl_u2_f1_af` — the address of one object stored into ANOTHER, which
// is the only arrangement that has an address producer and a single base symbol
// at the same time. It separates the producer from the symbol.
void H::lother(unsigned p, unsigned q) {
    mP0 = &mBlk;
    mP1 = &mBlk;
    mA = q;
}

// GRID M `m_bc_u1_f1_af` — board **#844**'s composition: the same run as the
// MIDDLE of a framed body, with the `mr r31,r3` spliced into it. The copy's slot
// is `save_slot(nprod, u_lead)` and the bind's second base symbol moves it — the
// direct twin of this body puts the copy one slot later, from the same rule.
H::H(unsigned p, unsigned q) {
    BE& r = mBlk;
    r.n0 = &r;
    mA = q;
    Grab(p);
}
