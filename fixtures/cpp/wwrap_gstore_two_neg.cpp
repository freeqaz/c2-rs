// w-wordwrap `_neg` — TWO statements, which is not two copies of the accepted
// body.
//
//     void G_two(unsigned int x, unsigned int y) { g_u = x; g_i = (int)y; }
//       lis 11,0      <== BOTH high halves are hoisted above BOTH stores
//       lis 10,0
//       stw 3,0(11)
//       stw 4,0(10)
//       blr
//
// The schedule interleaves; a per-statement lowering would emit
// `lis · stw · lis · stw` and every relocation would still resolve — board
// #232's shape. The statement count is fenced by `eat_return_plumbing`: a
// second statement leaves a `26` where the `3A` has to be.
//
// `fnbyte-exact` reads **0**.

unsigned int g_u;
int g_i;

void SetTwo(unsigned int x, unsigned int y) { g_u = x; g_i = (int)y; }
