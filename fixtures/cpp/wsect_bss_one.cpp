// w-sect / board #174 — ONE `.bss` object, the trivially-right case that is
// 23,253 of the workload's 24,055 `.data`/`.bss` sections.
// `.bss` sits BETWEEN the two `.XBLD$W` watermarks (Rule S1), which is the
// clause prereg P3 got backwards.
int b1;
