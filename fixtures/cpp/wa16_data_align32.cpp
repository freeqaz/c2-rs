// **W-ALIGN16 / board #1149 — the NEGATIVE cell at the new boundary. 16 is not
// the ceiling, and 32 is MEASURED and REFUSED.**
//
// This is the role `walign_dyninit_align16.cpp` used to play, moved up one power
// of two because #1120 took the value it guarded. That fixture's own header
// asked for exactly this: *"a later widening that adds `8A` without teaching
// `placement_align` about 16 turns this fixture from a refusal into a
// mismatch"*. The widening taught it, so the guard has to move with it — a
// boundary with no cell on its far side is a boundary nothing checks.
//
// `__declspec(align(32))` spells the `.gl` tag **`CC`** (wide, width field
// `8C` = 32) and real c2 gives the object `Characteristics` nibble **6**.
// `align(64)` is `CE` and nibble **7**, so the `log2 + 1` law itself is
// confirmed all the way to 64 — this refusal is NOT "we do not know what c2
// does".
//
// It is refused because **the grid stops here**. At 16 this lane varied
// structure nine ways: a scalar, a plain aggregate, an empty class, an array, a
// polymorphic class, a type made 16-aligned by a member rather than by the
// attribute, internal linkage, an initialized `.data`, and two objects sharing
// one `.bss`. At 32 and 64 it varied nothing — one shape each. Extending the
// table by `log2` to a value three cells confirm and no cell *constrains* is the
// "mostly right" table a refusal beats, and the incumbent refusal is right 100 %
// of the time on what it refuses.
//
// Price to take it: one more grid of this shape at 32/64. Nothing else.

__declspec(align(32)) struct A{int a;};
A g;
