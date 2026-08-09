// w-wordwrap `_neg` — a SUBSCRIPTED destination.
//
//     void G_arr2(unsigned int x) { g_arr[2] = x; }
//       lis  11,0
//       addi 11,11,0    <== the low half is an `addi`, and the REFLO moves
//       stw  3,8(11)
//       blr
//
// Sixteen bytes, and the relocation SITES are different: the accepted cell's
// REFLO sits on the store, this one's on an `addi` with the displacement in the
// store instead. The destination is required to be a bare `26 <tok>` with no
// offset run.
//
// `fnbyte-exact` reads **0**.

unsigned int g_arr[4];

void SetArr2(unsigned int x) { g_arr[2] = x; }
