// w-wordwrap `_neg` — a CONVERSION on the stored value, and the sharpest cell
// in GRID G.
//
//     void G_narrow(unsigned int x) { g_us = (unsigned short)x; }
//       lis 10,0        <== r10, not r11
//       sth 3,0(10)
//       blr
//
// **Twelve bytes — the SAME LENGTH as the accepted cell — with two of its three
// words different.** The address scratch moves to r10 the moment the body needs
// a second register, and a length check cannot tell this apart from
// `wwrap_gstore.cpp`. The recognizer's `gstore-value-carries-a-conversion`
// clause is what does, and this file is the reason it is not tidiness.
//
// `fnbyte-exact` reads **0**.

unsigned short g_us;

void SetNarrow(unsigned int x) { g_us = (unsigned short)x; }
