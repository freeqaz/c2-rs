// **Positive** — the indirect-load leaf. Every function here must emit, and the
// whole obj must be byte-exact.
//
// The whole body is one load through a pointer, and c2 lowers all of it to a
// single `lwz rD, off(rBase)` plus the `blr`, folding the byte offset into the
// displacement. Captured (one instruction each):
//
//   int f(int* p)                { return *p; }      ->  80630000  lwz r3,0(r3)
//   int f(int a, int* p)         { return *p; }      ->  80640000  lwz r3,0(r4)
//   int f(int a, int b, int* p)  { return *p; }      ->  80650000  lwz r3,0(r5)
//   int f(S* s)                  { return s->d; }    ->  80630010  lwz r3,16(r3)
//   int f(int* p)                { return p[3]; }    ->  8063000c  lwz r3,12(r3)
//   int f(int* p)                { return p[-1]; }   ->  8063fffc  lwz r3,-4(r3)
//   int f(int* p)                { return p[8000]; } ->  80637d00  lwz r3,32000(r3)
//
// The IL is a *composition*, not one opcode — `s->d` is
//
//   b9 <s> 86 43 81 20      LOAD s                    (S *)
//   33 86 41 74 10          LITERAL int 16            offsetof(S,d), in BYTES
//   27 86 43 f4 08          byte-offset add           -> int *
//   30 86 41 74             indirect load             -> int
//   41 86 41 74             result type
//
// so member access needs no member opcode at all. `0x9B`, which the census names
// `body-0x9B` and which an earlier reading guessed was a member bind, is a
// *temporary* designator and appears nowhere in any of these bodies
// (`il_expr_temp.cpp`).
//
// Two separate offset-add opcodes reach the same place and both are here:
// `27 <TYPE>` for a member (offset typed `int`) and `28 00 00` for a subscript
// (offset typed `long`, pre-scaled by the element size). `il_expr_index.cpp`
// holds the pair that separates them from the *third* spelling, `*(p + k)`,
// which is the ordinary `02` ADD.
//
// `ld_u`/`ld_l`/`ld_cv`/`ld_vo` are the operand-type ladder. All four emit the
// **same** bare `lwz`: signedness is not in the instruction, `long` is `int` on
// this target, and the cv-qualified pointees carry their qualification into the
// load type and then strip it with a `2C` that costs nothing —
//
//   const int *     30 a6 41 84 20  2c 86 41 74 00   ->  80630000
//   volatile int *  30 96 41 86 20  2c 86 41 74 00   ->  80630000
//
// That `2C` is why the class is worth having: a `const` accessor is the common
// real-world shape, and refusing the cv-strip would refuse nearly all of them.
// It is admitted only over a source the parser has pinned as a 4-byte integer,
// where `docs/IL_CAST_CONVERT.md` §4.2 proves the conversion free; the identical
// token over a `char`/`short`/`float` source is a real instruction and still
// refuses (`il_expr_load_neg.cpp`).
//
// The near-miss `int x = *p; return x;` is byte-identical to `return *p;` (c2
// register-allocates the local away) but is NOT accepted, because the assignment
// statement's right-hand side goes through `parse_expr`, which has no pointer
// operands. It is recorded as a refusal in `il_expr_load_neg.cpp` rather than
// quietly admitted here.

struct S { int a; int b; double c; int d; };

int ld_p(int* p) { return *p; }
int ld_p2(int a, int* p) { return *p; }
int ld_p3(int a, int b, int* p) { return *p; }
int ld_p_last(int* p, int a) { return *p; }

int ld_m0(S* s) { return s->a; }
int ld_m4(S* s) { return s->b; }
int ld_m16(S* s) { return s->d; }
int ld_dot(S& s) { return s.d; }

int ld_ix0(int* p) { return p[0]; }
int ld_ix3(int* p) { return p[3]; }
int ld_ixneg(int* p) { return p[-1]; }
int ld_ixbig(int* p) { return p[8000]; }

unsigned ld_u(unsigned* p) { return *p; }
long ld_l(long* p) { return *p; }
int ld_cv(const int* p) { return *p; }
int ld_vo(volatile int* p) { return *p; }
