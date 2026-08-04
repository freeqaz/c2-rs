// w-sect / board #174 — the two SIZE thresholds in one TU.
// `align = max(natural, 1 if n<2 else 4 if n<64 else 8)` steps at n=2 and
// n=64, and the `.gl` size field escapes its varint at 128 (`80 c8 00 00 00`
// for 200). A grid that stopped at `int` sees none of the three.
char one;
char a200[200];
