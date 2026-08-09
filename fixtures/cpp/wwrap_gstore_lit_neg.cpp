// w-wordwrap `_neg` — the stored value is a LITERAL rather than the formal.
//
//     void G_lit() { g_u = 7u; }
//       lis 10,0     <== r10
//       li  11,7
//       stw 11,0(10)
//       blr
//
// Sixteen bytes, four words, both scratch registers used, and the value word is
// one this class has no field for. Refused at
// `gstore-value-is-not-a-bare-load`.
//
// `fnbyte-exact` reads **0**.

unsigned int g_u;

void SetLit() { g_u = 7u; }
