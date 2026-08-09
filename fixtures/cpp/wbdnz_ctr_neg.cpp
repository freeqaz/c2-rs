// **w-bdnz — the counted-loop class's FENCE, one cell per clause.**
//
// The property this file's row in the gate grades is **`Port=NotImplemented`
// over the whole TU** — every cell here is outside the class and none may
// census in it. `wbdnz_ctr.cpp` is the separating control.
//
// # Read the third column, because it is what makes this a measurement
//
// Every cell below was compiled by real `c2.dll` under wibo at the workload's
// own `/O1 /Oi /EHsc /GR` before this file was written, and the conversion
// column is COUNTED off the reference obj by a script, not read by eye.
//
// **The objs are NOT committed** — nothing under `fixtures/` or `work/` may
// carry one — and neither are the `.cpp` copies a reader would otherwise have
// to trust this comment about. Both regenerate from files that ARE committed,
// in one command each (`work/w-bdnz/probe/{L3,L4}.cpp` and this file):
//
//     c2rs compile fixtures/cpp/wbdnz_ctr_neg.cpp --keep-obj /tmp/n.obj \
//         --flags-file work/w-bdnz/o1.txt
//     python3 scripts/gt_dump.py /tmp/n.obj --text-only
//
// Board #1127's lesson is that a rung's handover claims about its own artifacts
// get checked, so this comment names a command rather than a path.
//
// **SIXTEEN of the twenty-three are loops c2 DOES convert** — they carry a
// `mtctr`/`bdnz` in the reference obj and this port refuses them anyway, because
// reproducing them needs a word this class has no field for. (An earlier draft
// of this header said *six*, from counting by eye; the script said sixteen. That
// is the paraphrase failure this project keeps paying for, caught here before it
// was committed.) Those sixteen are the honest half of the fence: a boundary
// drawn only around what c2 itself refuses would be a boundary drawn around
// nothing.
//
//   cell        cnv  clause                     what real c2 emits
//   ----------- ---  -------------------------- ------------------------------
//   n_break      no  clause 2, single exit      addic./bf 2 -- no bdnz
//   n_cont      YES  clause 1, one back edge    bdnz + an inner cmpwi/bt
//   n_i64        no  clause 3, 32-bit counter   extsw/cmpd/bt 24 -- no bdnz
//   n_step2     YES  clause 4, step in {+1,-1}  addi -1 / srwi 1 / addi +1, bdnz
//   n_stepv      no  clause 4, constant step    cmpw/bt 24 -- no bdnz
//   n_bexpr      no  clause 5, SYMBOL bound     srawi/addze/addic. -- no bdnz
//   n_ctru      YES  clause 6, counter unused   bdnz + a second addi r11,r11,1
//   n_call       no  clause 7, no call          __savegprlr_29, bl, addic./bf 2
//   n_nest      YES  clause 7, no inner CTR     INNER takes bdnz, outer addic.
//   n_swap      YES  the bound is formal SLOT 0 li r11,1 / forward bf 25 /
//                                               mr r3,r11 -- a different PLAN
//   n_after     YES  the loop is the TAIL       addi r3,r11,7; the accumulator
//                                               stays in r11
//   n_litop     YES  the operand is a FORMAL    mulli r3,r3,3
//   n_addop      no  OP is not `+=`             mullw r3,r11,4 -- THE LOOP IS
//                                               GONE, guard and all
//   n_divop     YES  OP is not `/=`             rotlwi/divw/addi/twi/andc/twi
//   n_initover  YES  INIT fits simm16           lis / <the GUARD COMPARE> / ori
//   n_three     YES  exactly two formals        byte-identical to p_mul
//   n_long      YES  the accumulator is `int`   byte-identical to p_mul
//   n_uacc      YES  a SIGNED accumulator       srw r3,r3,r4 -- not sraw
//   n_start3    YES  start == 0                 cmpwi cr6,r11,3 + addi r11,-3
//   n_le        YES  the relation is `<`        bclr 12,24 + addi r11,r11,1
//   n_ne        YES  the relation is `<`        bclr 12,26 -- EQ, the SAME WORD
//                                               the unsigned `<` guard takes
//   n_down      YES  the relation is `<`        identical text to p_mul, from a
//                                               different IL production
//   n_dowhile    no  the class is the `for`     no mtctr and no bdnz at all --
//                                               wb-loop's own P3.4 miss
//                                               (`cal_dowhile`), reproduced
//
// # The three cells that are here because c2 emits bytes this port ALREADY HAS
//
// `n_three`, `n_long` and `n_down` have reference text **byte-identical** to
// `p_mul`'s. They are refused because nothing graded them when the class was
// drawn, and the rule this project runs on is that the accepted set and the
// graded set are the same set. A later lane may take any of them; each needs its
// own cell, not an argument that the bytes "must" be the same. `n_down` is the
// instructive one — a descending `for` and an ascending one produce the same
// eight words, so the class is a statement about IL productions and not about
// source loops.
//
// # `n_ne` is the sharpest row and is worth one more sentence
//
// A signed `i != n` guard and an *unsigned* `i < n` guard are **the same word**
// (`bclr 12,26`). So the guard word does not determine the source relation, and
// a class keyed on the emitted byte rather than on the IL relational opcode
// would merge them. This port keys on the IL (`22` required literally) and ships
// `<` alone.

int gf(int);

int n_break(int n, int k)  { int s = 1; for (int i = 0; i < n; ++i) { s *= k; if (s > 100) break; } return s; }
int n_cont(int n, int k)   { int s = 1; for (int i = 0; i < n; ++i) { if (k > 0) continue; s *= k; } return s; }
int n_i64(int n, int k)    { int s = 1; for (long long i = 0; i < n; ++i) s *= k; return s; }
int n_step2(int n, int k)  { int s = 1; for (int i = 0; i < n; i += 2) s *= k; return s; }
int n_stepv(int n, int k)  { int s = 1; for (int i = 0; i < n; i += k) s *= k; return s; }
int n_bexpr(int n, int k)  { int s = 1; for (int i = 0; i < n / 2 + 3; ++i) s *= k; return s; }
int n_ctru(int n, int k)   { int s = 1; for (int i = 0; i < n; ++i) s *= i; return s; }
int n_call(int n, int k)   { int s = 1; for (int i = 0; i < n; ++i) s *= gf(k); return s; }
int n_nest(int n, int k)   { int s = 1; for (int i = 0; i < n; ++i) for (int j = 0; j < k; ++j) s *= k; return s; }
int n_swap(int k, int n)   { int s = 1; for (int i = 0; i < n; ++i) s *= k; return s; }
int n_after(int n, int k)  { int s = 1; for (int i = 0; i < n; ++i) s *= k; return s + 7; }
int n_litop(int n, int k)  { int s = 1; for (int i = 0; i < n; ++i) s *= 3; return s; }
int n_addop(int n, int k)  { int s = 0; for (int i = 0; i < n; ++i) s += k; return s; }
int n_divop(int n, int k)  { int s = 1; for (int i = 0; i < n; ++i) s /= k; return s; }
int n_initover(int n, int k) { int s = 32768; for (int i = 0; i < n; ++i) s *= k; return s; }
int n_three(int n, int k, int j) { int s = 1; for (int i = 0; i < n; ++i) s *= k; return s; }
long n_long(int n, long k) { long s = 1; for (int i = 0; i < n; ++i) s *= k; return s; }
unsigned n_uacc(int n, unsigned k) { unsigned s = 1; for (int i = 0; i < n; ++i) s >>= k; return s; }
int n_start3(int n, int k) { int s = 1; for (int i = 3; i < n; ++i) s *= k; return s; }
int n_le(int n, int k)     { int s = 1; for (int i = 0; i <= n; ++i) s *= k; return s; }
int n_ne(int n, int k)     { int s = 1; for (int i = 0; i != n; ++i) s *= k; return s; }
int n_down(int n, int k)   { int s = 1; for (int i = n; i > 0; --i) s *= k; return s; }
int n_dowhile(int n, int k){ int s = 1; int i = 0; do { s *= k; ++i; } while (i < n); return s; }
