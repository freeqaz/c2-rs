// w-sect / board #174 — TWO `.data` objects, and the fixture that discriminates
// the two WALKS. `.data` walks DECLARATION order (Rule A2) while `.bss` walks
// `.gl` file order (Rule A1), and here they disagree: the `.gl` order is
// `d2 d1` and the addresses are `d1@0 d2@4`. A writer that used one order for
// both sections places these backwards — §5.7 scores that at 19 of 68 real
// `.data` sections against Rule A2's 46.
int d1 = 1;
int d2 = 2;
