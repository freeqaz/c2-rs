// W-ADJUST negative — every cell the NAMED-OBJECT receiver refuses, one per
// reason, with the measurement behind each.
//
// `c2rs census` must report **0/N in class** and `c2rs diff` must report
// `Port=NotImplemented`. A cell that quietly enters the class here is a
// wrong-bytes emit in `wadjust_obj_recv.cpp`'s neighbourhood, which is the whole
// point of carrying the refutations as a graded file rather than as prose.
//
//  1. **A formal that has to MOVE** (`m1`..`m3`). A *free* function forwarding
//     its own arguments is a permutation past a hoisted `lis` — `gDbg.puti(a)`
//     wants `a` in r4 while it is in r3 and the address is going to r3 — and WR1
//     refuses the whole class as `call-arg-sym-permuted`: at two shifting formals
//     c2 pre-saves into r11 and moves the `lis` to r10, a schedule one probe does
//     not separate from the one-move one. This is the boundary that makes the dc3
//     row a *member*-function row: `this` is formal 0 and is dropped, so the
//     surviving formals are already where they need to be. It is the SLOT that
//     decides and not the caller's kind, which `wadjust_obj_recv.cpp`'s `p1`
//     grades from the other side: a free function whose argument's formal INDEX
//     already equals its slot (`void p1(const char*, int b){ gDbg.puti(b); }`)
//     is in class, and the same body one parameter shorter is not.
//
//  2. **A DEFINED object** (`d1`) and a **defined static data member** (`d2`).
//     Indistinguishable from an extern by mangling; the `.gl` linkage byte is
//     what separates them (`02` undefined-extern, `01` defined here), and a
//     defined one puts a whole `.data`/`.bss` section in the middle of the
//     section table. WR1's `data-sym-not-extern` gate, reached through the
//     receiver position for the first time.
//
//  3. **AN OFFSET off the object** (`o1`, `o2`) — `gPair.b.m()`. The addend is
//     never folded into the relocation: MEASURED by WR1, `&gT.b` is
//     `lis r11 ; addi r11,r11,0 ; addi r3,r11,4`, a THIRD instruction whose base
//     is the scratch and not the destination. `eat_sym_addr_value` refuses the
//     `33 <k> 27` run for that reason and this file is where the refusal is
//     graded through the receiver.
//
//  4. **A SECOND symbol in the same call** (`t1`) — `gDbg.put2(&gI)`. c2
//     materializes exactly one address per function through a relocation pair and
//     derives the rest by `.rdata` pool-offset difference, which is a whole-TU
//     layout the port's per-function selector cannot see.
//
//  5. **The object's method called through a CHAIN** (`c1`) — `gDbg.self()->m()`.
//     Two `bl`s and a value live across the first: Class B, and not this
//     production. It is here because its head is byte-identical for two tokens,
//     which is what `eat_receiver_object` has to decline WITHOUT consuming.
//
//  6. **The result consumed by a literal post-op** (`f1`) — `return gDbg.get()+1;`
//     is a *framed* call, and `framed_member_call` rebuilds the receiver as
//     `IlOp::Load` from a token that here names a data symbol. Refused by name
//     (`tail-object-receiver-is-not-a-tail-call`) rather than routed into a shape
//     with no capture for an address receiver.
//
//  7. **A second statement after the call** (`s1`). The Class A sequence with an
//     object-receiver member call in it; the body does not end at the call.

struct Dbg {
    void nul();
    void puti(int);
    void two(int, int);
    void put2(int*);
    int get();
    Dbg* self();
};

struct Pair {
    int a;
    Dbg b;
};

struct Holder {
    static Dbg sDefined;
};

extern Dbg gDbg;
extern Pair gPair;
extern int gI;

Dbg gDefined;
Dbg Holder::sDefined;

// 1 — a free caller whose formal has to move past the hoisted `lis`
void m1(int a) { gDbg.puti(a); }
void m2(int a, int b) { gDbg.two(a, b); }
void m3(int b, int) { gDbg.puti(b); }

// 2 — a defined object, and a defined static data member
void d1() { gDefined.nul(); }
void d2() { Holder::sDefined.nul(); }

// 3 — an offset off the object
void o1() { gPair.b.nul(); }
void o2(int) { gPair.b.nul(); }

// 4 — a second symbol in the same call
void t1() { gDbg.put2(&gI); }

// 5 — a chain through the object
void c1() { gDbg.self()->nul(); }

// 6 — the result consumed by a literal post-op: a framed call
int f1() { return gDbg.get() + 1; }

// 7 — a second statement after the call
void s1() { gDbg.nul(); gDbg.puti(1); }
