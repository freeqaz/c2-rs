// **W-ALIGN positive cell — the WIDE `.gl` type tag, board #1110.**
//
// `fixtures/cpp/wr1c_dyninit_extern.cpp` with exactly one axis changed: the
// class is polymorphic. That one keyword moves the `.gl` DATA record's TYPE tag
// from `86` to **`C6`**, and `align_of_type_tag` modelled four tags — `82`,
// `84`, `86`, `88` — and refused every wide form. So this shape was
// `NotImplemented` with no RTTI anywhere near it:
//
//   ?gL@@3UL@@A   00   c6   81   06   00 02   01   10   00
//                 ^NUL ^tag ^mark ^kind ^frame ^link ^size ^attr
//
// `TAG_WIDE` (`0x40`) marks the mark byte's presence and nothing else, so the
// alignment is `tag & !TAG_WIDE` = `86` = **4** — and c2's own obj gives this
// object `.bss` ALIGN_4. `sizeof` is 16 (vfptr + three `int`) against
// `wr1c_dyninit_extern`'s 12, which is what makes this cell's `.bss` a
// *different* size class from its non-polymorphic sibling rather than a copy.
//
// The virtual function is DECLARED and not defined, so no vftable and no
// `.rdata$r` is emitted here — this is the `.gl` reader's cell, not #1107's.
// `walign_data_poly_object.cpp`'s comment records what happens when it is
// defined.
//
// Pairs with `walign_dyninit_poly_double.cpp`: same size (16), different
// natural alignment (4 against 8), so the two objs differ in one
// `Characteristics` nibble and real c2 is the judge of which.
//
// Converts where `/GF` is implied (`/O1`, `/O2`) and stays `NotImplemented` at
// `/Ox` for the same reason `wr1c_dyninit_extern.cpp` does.

struct L { virtual void f(); L(const char* s, int r); int a, b, c; };
L gL("abc", 0);
