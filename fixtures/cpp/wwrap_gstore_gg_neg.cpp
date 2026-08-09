// w-wordwrap `_neg` — the stored value is ANOTHER GLOBAL, not the formal.
//
//     void SetFromGlobal(unsigned int x) { g_u = g_v; }
//       lis 11,0        REFHI g_v      <== BOTH high halves are hoisted first,
//       lis 10,0        REFHI g_u          which no single-object derivation
//       lwz 11,0(11)    REFLO g_v          can even read
//       stw 11,0(10)    REFLO g_u
//       blr
//
// **This is the cell that makes `gstore-value-is-not-the-formal` load-bearing,
// and it is the sharpest `_neg` in the set.** A global read is spelled
// `B9 <tok> <TYPE>` exactly as a formal read is, so without the token
// comparison the IL stream `26 <g_u> · B9 <g_v> <T> · 32 <T> · 4B` is
// STRUCTURALLY IDENTICAL to the accepted cell and every other clause passes —
// the width table, the no-conversion clause, the type restatement, the one
// statement, the one formal. The class would emit `lis 11 ; stw 3,0(11)` for a
// body that reads memory and never mentions r3: twenty bytes of c2 against
// twelve of port, two relocation pairs against one, and a `.text` that still
// links.
//
// `fnbyte-exact` reads **0**. The formal `x` is unused on purpose: it is what
// keeps the body's argument list identical to the accepted cell's, so the
// refusal cannot be attributed to the arity fence instead.

unsigned int g_u;
unsigned int g_v;

void SetFromGlobal(unsigned int x) { g_u = g_v; }
