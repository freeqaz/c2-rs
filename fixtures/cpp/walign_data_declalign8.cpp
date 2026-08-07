// **W-ALIGN — `__declspec(align(N))` MOVES THE TYPE TAG, and the class need not
// be polymorphic to spell the wide form.**
//
// The prereg's least-confident prediction (P5, 0.50) and the cell that would
// have caught a plausible one-line widening emitting a wrong `Characteristics`
// word. `struct A { int a; }` is naturally 4-aligned and spells `86`. Add
// `__declspec(align(8))` and the same struct spells **`C8`** — wide, mark `81`,
// width field `88` — and c2 gives the object `.bss` **ALIGN_8**.
//
//   ?g@@3UA@@A   00   c8   81   06   00 02   01   08   00
//
// So the width field under the wide bit tracks the type's REQUIRED alignment,
// not its natural layout, and not its size (this object is 8 bytes and so is
// `walign`'s naturally-4-aligned sibling).
//
// It is a `data_tu` cell, not a `dyninit_tu` one — no constructor, no
// `.CRT$XCU`. Before board #1110's arm this TU was `codegen-gap`: *"a TU that
// defines no functions but whose `.gl` names storage"*, i.e. `shell_only_tu`
// refused it (correctly — c2 emits a `.bss`) and `data_tu` could not read the
// record either. It is the second consumer the one arm unblocks, and the reason
// #1110's price of "one match arm" was right about the arm and wrong about it
// being reachable through only one path.

__declspec(align(8)) struct A{int a;};
A g;
