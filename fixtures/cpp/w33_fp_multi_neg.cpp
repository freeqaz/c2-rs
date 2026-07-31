// **Negative** — the multi-argument FP tail call's boundary. Every function
// here must be refused (`0/N in class`), and every refusal is priced: the ones
// c2 emits *differently* are wrong-bytes risks, the ones it emits in a shape
// this rung simply does not spell are coverage, and the comment says which.
//
// `docs/rungs/2026-07-31-fp-multiarg.md`. Read off reference objs (`/O1 /GS- /c`):
//
//   both(int a,int b,float c,float d)  { gif2(b,a,d,c); }
//        fmr f0,f2 ; mr r11,r4 ; mr r4,r3 ; fmr f2,f1 ; mr r3,r11 ; fmr f1,f0
//   gpr (int a,int b,float c,float d)  { gifm(a,c,b,d); }   mr r5,r4
//   two (a,b,c,d)                      { g4f(b,a,d,c); }
//        fmr f0,f2 ; fmr f13,f4 ; fmr f4,f3 ; fmr f2,f1 ; fmr f1,f0 ; fmr f3,f13
//   vall(a,b,c,d)                      { g4f(c,d,b,a); }
//        fmr f0,f3 ; fmr f13,f4 ; fmr f4,f1 ; fmr f3,f2 ; fmr f1,f0 ; fmr f2,f13
//   nar (double a,b,c) -> g3f(float,float,float)(b,c,a)
//        fmr f0,f2 ; fmr f13,f3 ; frsp f3,f1 ; frsp f1,f0 ; frsp f2,f13
//   dup (a,b)                          { g2f(a,a); }        fmr f2,f1
//   out (a,b,c)                        { g2f(b,c); }        fmr f1,f2 ; fmr f2,f3
//
// **`both` is the refusal the whole rung is built around.** The two register
// files' move sequences interleave — save-FP, save-GPR, move-GPR, move-FP,
// restore-GPR, restore-FP — and no per-file solver reproduces that schedule
// (`docs/CODEGEN_FP_ARGS.md` §1.1). Splitting the family at "every argument is
// floating-point" is what makes the positive fixture a claim that does not
// depend on it.
//
// **`gpr` is the boundary's other side, and it is pure coverage.** Only the GPR
// file moves, so there is nothing to interleave; it is refused because this
// recognizer indexes the FP file and has no destination numbering for the other
// one. Its cost is measured in the rung doc.
//
// **`two` and `vall` are the two-scratch cases, and they are the reason the gate
// counts LOCAL MINIMA rather than cycle length.** `two` is two disjoint
// 2-cycles and `vall` is a single 4-cycle whose sequence descends and then
// ascends; both park a **second** scratch — `f13`, which no capture had before
// this rung — and the order the two independent chains interleave in is the same
// residue `docs/CODEGEN_ARG_PERM.md` §2.1 leaves open in the GPR file (26 of the
// 120 cells at n = 5, in both files). Their unimodal 4- and 5-cycle neighbours
// are in the positive fixture and are byte-exact, which is what makes this a
// measured boundary and not a length limit.
//
// **`nar` is the load-bearing conversion refusal.** `double`→`float` at the
// argument boundary fuses into the move that writes the destination — and with
// *every* argument of a 3-cycle narrowing, c2 changes the schedule outright:
// five moves and two scratches where the same permutation without the
// conversion is four and one. One type change, a different lowering. W31
// measured the single-argument `frsp`'s census value at **0**, so the whole
// conversion is refused here rather than modeled from the cases that happen to
// fuse.
//
// `dup` and `out` are neither permutations nor mis-emit risks: a value passed
// twice is a copy graph and `g2f(b,c)` is a shift out of a register the call
// does not otherwise write (the GPR file refuses the same shape under
// `call-arg-outer-formal`, where it was also a panic). Both are cheap coverage
// the rung doc sizes rather than claims.
//
// `cmp`, `lit` and `res` are the single-argument rung's own refusals in the
// multi-argument position: a computed argument, an FP literal through an
// `.rdata` COMDAT, and a conversion applied to the call's RESULT.

float  g2f(float, float);
float  g3f(float, float, float);
float  g4f(float, float, float, float);
int    gif2(int, int, float, float);
int    gifm(int, float, int, float);

int    both(int a, int b, float c, float d)         { return gif2(b, a, d, c); }
int    gpr (int a, int b, float c, float d)         { return gifm(a, c, b, d); }
float  two (float a, float b, float c, float d)     { return g4f(b, a, d, c); }
float  vall(float a, float b, float c, float d)     { return g4f(c, d, b, a); }
float  nar (double a, double b, double c)           { return g3f(b, c, a); }
float  dup (float a, float b)                       { return g2f(a, a); }
float  out (float a, float b, float c)              { return g2f(b, c); }
float  cmp (float a, float b)                       { return g2f(b, a + b); }
float  lit (float a)                                { return g2f(a, 1.5f); }
double res (float a, float b)                       { return g2f(b, a); }
