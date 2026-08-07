// **W-ALIGN discriminating partner — same size, different ALIGNMENT.**
//
// `walign_dyninit_poly.cpp` and this cell both define a 16-byte `.bss` object.
// They differ in one thing: this one's natural alignment is **8**, so
// `coff::container::align_nibble(16, 8)` is 4 (ALIGN_8) where the sibling's
// `align_nibble(16, 4)` is 3 (ALIGN_4). `container.rs`'s own doc states the
// rule the pair exercises — *"a `double` member gives ALIGN_8 at n = 8 where a
// `char[8]` gives ALIGN_4"*.
//
// Two objs that differ in ONE nibble, judged by real c2. Without this cell "the
// tag is the alignment" and "the tag is 4 for every wide aggregate" are
// indistinguishable on the fixture corpus.
//
// It is also a NEGATIVE cell for the wide form: a polymorphic class whose
// natural alignment is 8 spells its `.gl` tag **`88`, not `C8`** — the wide bit
// is not "the class has a vtable", whatever `docs/IL_TYPE_WIDE_TAG.md` §2.1
// establishes for `.ex`. This cell therefore passed *before* board #1110's arm
// as well as after, which is what makes it a control.

struct L { virtual void f(); L(const char* s, int r); double d; };
L gL("abc", 0);
