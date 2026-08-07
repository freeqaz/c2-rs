// **W-ALIGN16 / board #1120 — the ALLOCATOR at 16, which is the thing the
// prereg said it expected to lose on.**
//
// Two objects in ONE `.bss`: a one-byte `char` and a 16-aligned struct. c2 gives
// the section nibble **5** (Rule B1's max, with a 16 in it) and puts `?g@@3UA@@A`
// at offset **16**, not 4 — the cursor is rounded up past the `char` by the
// second object's own alignment.
//
// That rounding is `coff::data::bump_layout`, and it changed behaviour with **no
// textual edit**: it calls `placement_align`, so widening the promotion table
// silently extended Rule A3′ past every cell it was ever fitted on
// (`OBJ_DATA_BSS_SHAPE.md` §5.7 scored it on 1/2/4/8, and every real workload
// section it scored against had `align ≤ 8`). Prereg P14 put this cell at 0.55
// and named it as the likelier of the two losses.
//
// It did not lose, and this fixture is why the next lane does not have to take
// that on trust: a change to the promotion table that gets the *nibble* right
// and the *offset* wrong passes `wa16_data_align16.cpp` and fails here.

__declspec(align(16)) struct A{int a;};
A g;
char c;
