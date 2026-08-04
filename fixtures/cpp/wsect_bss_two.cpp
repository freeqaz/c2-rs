// w-sect / board #174 — TWO `.bss` objects, NO alignment padding anywhere.
// Two is the measured class bound (47 of 48 real sections); no padding is the
// case where every candidate allocator coincides, so this fixture grades the
// WALK ORDER alone — and 10 of the 64 real no-padding sections are still wrong,
// which is why the walk is graded rather than assumed.
// `.gl` record order is `b2 b1`, the reverse of declaration.
int b1;
int b2;
