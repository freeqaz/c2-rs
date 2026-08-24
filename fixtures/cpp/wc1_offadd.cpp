// **Phase 1 slice C1 — the BYTE-OFFSET ADD (`0x27`) in a general expression.**
// Lane `w-c1`, 2026-08-24. Board #3472-#3476.
//
// `27 <PTR TYPE>` pops a byte offset that the preceding `33` pushed as a
// literal, adds it to the address under it, and re-types the result. A member
// designator is one `27` per step (`&t->s.b` is two), and c2 lowers the whole
// run as a SINGLE `addi rD,rBase,<sum>` — or an `addis`+`addi` pair when the
// sum does not fit the 16-bit immediate.
//
// **Every one of the 22 bodies here was `expr-op-0x27` at base `e85253cda` —
// 0 of 22 in class, all 22 on that one key, the whole TU `vocab-gap`** — and
// all 22 are in class and the TU is `Port=Match` at this tree. That is the
// #3455 control: a fixture that grades the same green with the change reverted
// is not a fixture. It is re-runnable without a checkout too, because C1's
// decision point is named — `C2RS_OFF_ADD=off` restores the pre-C1 parser and
// this file goes straight back to 0 of 22 on `expr-op-0x27`.
//
// **Graded at all 18 registered lanes** (`scripts/lanes.txt`): `match` at 14 —
// every `/O1`, `/O2` and `/Ox` row including `/EHsc`, `/GR`, `/Gy` and the
// workload's own `/O1 /Oi /EHsc /GR` — and `codegen-gap` (a refusal, never a
// mismatch) at the four `/Od` rows. **0 mismatch at every lane.**
//
// The boundary this file does NOT claim is in `wc1_offadd_neg.cpp`, and the
// single most important line of it is the multi-argument call: board **#149**'s
// ordering rule for a computed argument was SEARCHED and REFUTED at 87.0 %
// (`ROADMAP.md` §9.19.5), so the only call shape admitted here is the one with
// no permutation to get wrong.

struct S { int a; int b; int c; };
struct T { int x; S s; int y; };
struct U { int h; T t; };
struct Big { int pad[20000]; int tail; };
struct M { char c; short h; int i; long long q; double d; float f; };
struct C { int m; int viaThis() const; };

// ---------------------------------------------------------------------------
// A — the designator run itself. One `addi`, whatever the step count.
// ---------------------------------------------------------------------------

// One step.                                             addi r3,r3,16
int one_step(T* t) { return (int)&t->y; }
// Two steps, summed by the walk, not emitted twice.     addi r3,r3,8
int two_steps(T* t) { return (int)&t->s.b; }
// Three.                                                addi r3,r3,20
int three_steps(U* u) { return (int)&u->t.s.c; }
// **Offset zero emits NOTHING** — the address is already the formal, so the
// body is a bare `blr`. The `zero` arm of #127's family, and the reason the sum
// is carried as a value rather than assumed non-zero.
int zero_off(T* t) { return (int)&t->x; }
// Past the 16-bit `addi` immediate: `addis r3,r3,1 ; addi r3,r3,14464`.
int wide_off(Big* b) { return (int)&b->tail; }

// ---------------------------------------------------------------------------
// B — the POINTEE TYPE axis. `addi` is the same word for every pointee width,
// which is exactly why the `27`'s type is admitted from a literal whitelist
// (`designator::is_ptr_any`) rather than checked against the member's width.
// The tag's width nibble is NOT a dependable statement of the pointee width
// here — measured in `designator.rs`'s own header.
// ---------------------------------------------------------------------------

int narrow_c(M* m) { return (int)&m->c; } // char
int narrow_h(M* m) { return (int)&m->h; } // short
int wide_q(M* m) { return (int)&m->q; }   // long long
int real_d(M* m) { return (int)&m->d; }   // double — an FP member, an integer address
int real_f(M* m) { return (int)&m->f; }   // float
int const_p(const T* t) { return (int)&t->s.b; } // a const-qualified base

// ---------------------------------------------------------------------------
// C — the VALUE MODEL. These are the bodies the pre-C1 sink refused as
// `expr-ptr-arith:mid`: it cleared the class stack at the `27`, so the coarse
// whole-expression flag could not see that the `2C` had already moved the value
// out of the pointer class before the `+`. Modelling `27` as the binary token
// it is makes the exact guard answer instead.
// ---------------------------------------------------------------------------

int plus_formal(T* t, int i) { return (int)&t->s.b + i; }
unsigned as_uns(T* t) { return (unsigned)&t->s.a; }
int C::viaThis() const { return (int)&this->m; }

// ---------------------------------------------------------------------------
// D — the off-add as a CALL ARGUMENT, at **one argument only**.
//
// This is part of board **#149**'s population (`p->m(&t->s.k)`), and #149 is
// NOT shipped: a computed address in a multi-argument call needs a `SlotArg`
// variant and its position in the permutation walk, and the rule for that
// position was refuted. What is admitted here is the case with no permutation
// at all — one argument, one register, no other move to order against — and it
// is byte-exact at every lane above. `wc1_offadd_neg.cpp` holds the six
// multi-argument neighbours that must keep refusing.
// ---------------------------------------------------------------------------

extern void a1(int*);
extern int r1(int*);

void arg_first(T* t) { a1(&t->s.b); }              // addi r3,r3,8 ; b a1
void arg_other_base(int k, T* t) { a1(&t->s.b); }  // addi r3,r4,8 ; b a1
void arg_zero(T* t) { a1(&t->x); }                 // b a1  — nothing at all
void arg_zero_move(int k, T* t) { a1(&t->x); }     // mr r3,r4 ; b a1
void arg_wide(Big* b) { a1(&b->tail); }            // addis ; addi ; b a1
void arg_wide_move(int k, Big* b) { a1(&b->tail); }// addis r3,r4,1 ; addi ; b
int arg_value(T* t) { return r1(&t->s.b); }        // the result is used
int arg_value_move(int k, T* t) { return r1(&t->s.b); }
