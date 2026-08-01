// W-ADJUST — a **named data object standing as a member call's receiver**:
// `gObj.m(a);`, where the receiver designator is a data symbol's *address*
// rather than a pointer already sitting in a register.
//
// The IL is the W36 member call with one token changed — the receiver is
// `26 <sym> [2C …] 99 …` instead of `B9 <tok> <TYPE ptr4> [2C …] 99 …` — so the
// production is the existing tail call and the address is WR1's: `lis r11,0` +
// `addi rN,r11,0` carrying a REFHI/REFLO PAIR quad, the `addi` emitted LAST.
// Nothing new is emitted; what is new is that the two are composed, and the
// composition is what this file grades.
//
// Board #128, measured before it was written: the census row
// `expr-call-in-expr-recv-object-then-type-ptr-whole` is **1,380 emitted
// functions, clean 1,380 = 100 %**, and its completion counterfactual over the
// 878-TU dc3 workload is **+1,385 emitted** — the row entire, plus 16 from three
// neighbouring `recv-object` rows. See `docs/rungs/2026-08-01-w-adjust.md`.
//
// ---- the shape of what is in class, stated positively ----------------------
//
// The receiver takes **slot 0**, and `sym_addr_tail_call` requires every other
// slot to hold the formal that is already in that argument register. So the
// in-class population is exactly:
//
//   * a caller with no arguments to pass on, of any kind;
//   * a **member** caller passing its own formals in order — `this` is formal 0
//     and is dropped, so formal `i` wants slot `i` and is already there. This is
//     the dc3 row verbatim:
//     `void DebugFailer::operator<<(const char* s) { TheDebug << s; }`;
//   * a literal beside the receiver, which `li`s into place.
//
// A **free** caller passing a formal on (`void f(int a){ gDbg.puti(a); }`) is a
// 2-cycle permutation past a hoisted `lis`, and WR1 refuses it as
// `call-arg-sym-permuted` — that cell and every other refusal is in
// `wadjust_obj_recv_neg.cpp`, one per named reason.
//
// ---- the value axes this varies, and why -----------------------------------
//
//  * **how many slots ride beside the receiver** (0, 1, 2, 3): the receiver is
//    slot 0 and every other slot shifts along it, which is the arrangement the
//    address `addi`'s RD field is read out of;
//  * **the caller's kind** — free, member, and a member that RETURNS the result,
//    since `this` being dropped is what makes the surviving formals in-place;
//  * **the argument's type**, `int` and `const char*`: the `55 <TYPE>` formal
//    annotation and the `B9` operand type widen in step, and a pointer argument
//    is what the census key names (`-then-type-ptr`);
//  * **a literal argument beside the symbol**, whose `li` lands between the
//    `lis` and the address `addi` (WR1's `c1`/`c4` rule, here with the symbol
//    pinned at slot 0 — see the note below);
//  * **the result**: discarded (`4B`) and returned (`41 <T>`), the two plumbing
//    arms;
//  * **the symbol NAME across the 8-byte COFF inline-name boundary**, the second
//    path in `emit_external_symbol`, reached here with a CLASS object;
//  * **one object referenced from several functions** (`gDbg`), which must appear
//    as exactly ONE undefined external;
//  * **a framed function last** (`fr`), so the compiler-label stride is graded
//    over a body of this shape.
//
// **What this fixture structurally CANNOT reach, and why that is written down.**
// WR1's own ALARM was that its hand-written fixture put the symbol at slot 0 in
// all three literal cases, so a descending-destination walk and the true
// address-last rule agreed everywhere it looked. Here the symbol is **always** at
// slot 0 — it is the receiver, and the receiver is argument zero on this ABI —
// so this file cannot discriminate the two rules at all. It does not have to:
// `wr1_sym_addr.cpp`'s `c4` does, and this rung changes no emitter. The
// generated axis that varies what rides beside the receiver is
// `scripts/sweep.d/61-object-receiver.py`.

struct Dbg {
    void put(const char* s);
    void puti(int k);
    void two(int a, int b);
    void three(int a, int b, int c);
    void nul();
    int get();
    int getk(int k);
};

struct LongNamedType {
    void put(const char* s);
    void nul();
};

extern Dbg gDbg;
extern Dbg gOther;
extern LongNamedType gObjectWithALongName;

int gf(int);

// ---- no arguments beside the receiver: the whole address and nothing else ----

void n0() { gDbg.nul(); }
int r0() { return gDbg.get(); }
void n0b() { gObjectWithALongName.nul(); }

// ---- a literal beside the receiver: the `li` between `lis` and the `addi` ----

void l1() { gDbg.puti(7); }
void l2() { gDbg.puti(-1); }
void l3() { gDbg.two(3, 4); }
void l4() { gDbg.three(1, 2, 3); }
int l5() { return gDbg.getk(32767); }

// ---- the dc3 shape: a MEMBER caller forwarding its own formals in order ------

struct Fwd {
    void operator<<(const char* s);
    void fwd_i(int k);
    void fwd_2(int a, int b);
    void fwd_3(int a, int b, int c);
    int fwd_r();
    int fwd_rk(int k);
    void fwd_long(const char* s);
    void fwd_lit(int k);
};

void Fwd::operator<<(const char* s) { gDbg.put(s); }
void Fwd::fwd_i(int k) { gDbg.puti(k); }
void Fwd::fwd_2(int a, int b) { gDbg.two(a, b); }
void Fwd::fwd_3(int a, int b, int c) { gDbg.three(a, b, c); }
int Fwd::fwd_r() { return gDbg.get(); }
int Fwd::fwd_rk(int k) { return gDbg.getk(k); }
void Fwd::fwd_long(const char* s) { gObjectWithALongName.put(s); }
void Fwd::fwd_lit(int) { gDbg.puti(9); }

// ---- the same object from several functions: ONE undefined external ---------

void s1() { gOther.nul(); }
void s2() { gDbg.nul(); }

// ---- and a FRAMED function after all of them, so the label stride is graded --

int fr(int a) { return gf(a) + 1; }
