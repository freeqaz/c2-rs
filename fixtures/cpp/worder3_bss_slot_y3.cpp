// **W-ORDER3 / board #174, #1152 — Rule Y3, and the cell that refutes Rule Y1's
// STATIC clause. Byte-exact.**
//
// `OBJ_DATA_BSS_SHAPE.md` §6.2:
//
// > **Rule Y1 (eager `.bss`).** Emit every EXTERNAL symbol first, in reverse
// > `.gl` record order; then every STATIC symbol, in **declaration order**.
//
// Y1's static clause is fitted **only on TUs that have functions** — that is
// what keeps *their* statics alive, since an unreferenced uninitialized static
// is dropped (`wsect_drop_static.cpp`). `coff::data::emit_data_obj` serves
// **functionless** TUs and was applying it there anyway (board #1148). This is
// the first functionless cell with two statics in one `.bss`, and it separates
// the two candidate orders because they DISAGREE on it:
//
//     .gl record order    h g      <- what c2 emits, addresses h@0 g@4
//     declaration order   g h      <- what Rule Y1's static clause predicts
//
// > **Rule Y3 (slot-`A` `.bss`, functionless).** The group's defined symbols are
// > emitted in **`.gl` record order** — the same permutation as Rule A1's walk,
// > so also **ascending** address.
//
// Confirmed out of sample at n = 3 by `work/w-order3/cells/O16`, where `.gl`,
// the addresses and the symbol table are all `i h g` while declaration order is
// `g h i`. O16 itself is refused here — `MAX_OBJECTS_PER_SECTION` is 2 (board
// #184) — so this file is the shipped witness and O16 is the confirmation.
//
// Y1 is **not** refuted where it was fitted, and its EXTERNAL clause is untouched
// and still carries its 89 real sections. It was being read past its cells.
//
// `.data`'s own group is a control in the same obj: `p` and `q` come out in
// DECLARATION order (`p`@0 `q`@4) while their `.gl` order is `q p`. So one obj
// contains both permutations, which is why no single walk can serve both
// sections (Rule A1 vs Rule A2).

struct A{int a;};
static A g;
static A h;
A* p = &g;
A* q = &h;
