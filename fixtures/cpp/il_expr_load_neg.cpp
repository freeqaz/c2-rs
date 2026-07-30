// **Negative** — everything one byte away from the indirect-load leaf. Every
// function here must keep refusing.
//
// The class accepted in `il_expr_deref.cpp` is *one* load, of *one* 4-byte
// integer, through *one* byte-offset add that fits a 16-bit displacement, with
// nothing after it but the return. Each function below breaks exactly one of those
// and each is a captured case where the emitted code differs — not a range
// limitation.
//
// ## The loaded width picks a different instruction
//
//   char*     30 82 11 70   ->  88630000  lbz  r3,0(r3)
//   short*    30 84 21 11   ->  a0630000  lhz  r3,0(r3)     (lhz, NOT lha)
//   float*    30 86 45 40   ->  c0230000  lfs  f1,0(r3)
//   double*   30 88 85 41   ->  c8230000  lfd  f1,0(r3)
//   int**     30 86 43 f4 08 -> 80630000  lwz  r3,0(r3)     (same word, still refused)
//
// `n_ldpp` is the interesting one: `int**` *does* emit the same `lwz`, so refusing
// it costs a real case. It is refused anyway because the gate is "the loaded type
// is a 4-byte integer", and admitting a pointer result would mean admitting the
// `A6`/`96` pointer-to-pointer families whose deref chains are not characterized.
// Refusing something provable is cheap; the reverse is not.
//
// ## Arithmetic over a load is not "the load, then the arithmetic"
//
// This is the load-bearing group, because a selector that lowered `LoadInd` as an
// ordinary operand would emit plausible, wrong bytes:
//
//   int f(int* p)        { return *p + 1; }   ->  lwz r11,0(r3) ; addi r3,r11,1
//   int f(int* p,int b)  { return *p + b; }   ->  lwz r11,0(r3) ; add  r3,r11,r4
//   int f(int* p,int* q) { return *p + *q; }  ->  lwz r10,0(r3) ; lwz r11,0(r4)
//                                                 add r3,r10,r11
//   int f(int* p)        { return *p * 3; }   ->  lwz r11,0(r3) ; slwi r10,r11,1
//                                                 add r3,r11,r10
//
// The load lands in the **scratch** register r11, not in the destination — so
// `lwz r3,0(r3) ; addi r3,r3,1` is wrong even though it is the obvious lowering.
// `n_mul3` is worse: c2 strength-reduces `x * 3` to `x + (x << 1)`, so there is no
// `mullw` at all. And `n_two` shows the scratch order is r10-then-r11 with two
// distinct bases, while `s.a + s.b` (one base, read twice — already refused by the
// repeated-leaf gate) allocates r11-then-r10 and reverses the add operands. The
// allocator is not a rule this port has.
//
// ## Structure
//
//   n_idxvar     p[i]        slwi r11,r4,2 ; lwzx r3,r11,r3   — a variable index is
//                            an indexed load, one instruction more, plus a scratch
//   n_idx2d      p[i][j]     two offset adds: slwi ; add ; slwi ; lwzx
//   n_idxhuge    p[100000]   offset 400000 does not fit the displacement:
//                            lis r11,6 ; ori r11,r11,0x1a80 ; lwzx r3,r3,r11
//   n_store      *p = v      stw r4,0(r3) ; or r3,r4,r4 — a memory write. The IL
//                            has NO deref opcode on this side: the lvalue is the
//                            loaded pointer and `32 <TYPE>` stores through it, so
//                            the same `32 <TYPE>` that is a register copy after a
//                            `26 <sym>` is a store here. Distinguished only by what
//                            pushed the destination.
//   n_storek     *p = 7      mr r11,r3 ; li r10,7 ; li r3,0 ; stw r10,0(r11) — c2
//                            reorders the store past the return value and uses two
//                            scratch registers for a one-line body.
//   n_via_assign int x = *p;  byte-identical to `return *p;` (c2 allocates the local
//                return x;    away), and still refused: the assignment statement's
//                            right-hand side goes through `parse_expr`, which has no
//                            pointer operands. An honest near-miss, recorded rather
//                            than admitted by widening a gate that would then also
//                            admit `int x = *p; return x + 1;`.
//   n_bitfield   b->g        `27 <TYPE>` then two literals then `43 37` — the `0x43`
//                            escape, not a plain offset add (`il_expr_ternary.cpp`).

struct S { int a; int b; };
struct B { unsigned f : 3; unsigned g : 5; };

char n_ldc(char* p) { return *p; }
short n_lds(short* p) { return *p; }
float n_ldf(float* p) { return *p; }
double n_ldd(double* p) { return *p; }
int* n_ldpp(int** p) { return *p; }

int n_add1(int* p) { return *p + 1; }
int n_addb(int* p, int b) { return *p + b; }
int n_two(int* p, int* q) { return *p + *q; }
int n_mul3(int* p) { return *p * 3; }
int n_memsum(S* s) { return s->a + s->b; }

int n_idxvar(int* p, int i) { return p[i]; }
int n_idx2d(int (*p)[4], int i, int j) { return p[i][j]; }
int n_idxhuge(int* p) { return p[100000]; }

int n_store(int* p, int v) { *p = v; return v; }
int n_storek(int* p) { *p = 7; return 0; }

int n_via_assign(int* p) {
    int x = *p;
    return x;
}

int n_bitfield(B* b) { return b->g; }
