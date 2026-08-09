// w-wordwrap `_neg` — the stored value is the SECOND formal.
//
//     void G_second(unsigned int a, unsigned int x) { g_u = x; }
//       lis 11,0
//       stw 4,0(11)   <== r4
//       blr
//
// Twelve bytes again, one word different, and the class has no field for which
// argument register the value is in — it is fenced at exactly one formal
// (`gstore-not-exactly-one-formal-in-r3`). Admitting a second formal and
// emitting r3 would be a complete, plausible, wrong body.
//
// `fnbyte-exact` reads **0**.

unsigned int g_u;

void SetSecond(unsigned int a, unsigned int x) { g_u = x; }
