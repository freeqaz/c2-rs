// **W-ORDER3 / board #1179 — SLOT `C`: a static `.bss` kept alive by a FUNCTION
// sits AFTER the code groups, and Rule S1's three insertion points cannot say
// so. One line of C++, and nobody had written it.**
//
// `OBJ_DATA_BSS_SHAPE.md` §2.2's Rule S1 gives the shell three slots — before
// `.XBLD$W(C2)` (a `/GF` string `.rdata`), between the watermarks (the
// uninitialized section), after `.XBLD$W(C1)` (the initialized section, then the
// code groups, then `.CRT$XCU`). There is a fourth position, and an eager `.bss`
// can take it:
//
//     .drectve .debug$S .XBLD$W(C2) .XBLD$W(C1) .text .bss
//
// The doc already had this position for the **dyninit** `.bss` (§2.1 row 11) and
// read it as a property of deferred objects. It is not: this file's `g` is an
// ordinary uninitialized static with no dynamic initializer at all. What decides
// the slot is **which contributor materialised the section first** —
//
//     A  a STATIC first reached from a `.data` initializer  -> before C2
//        (`wa16_bss_static_reloc.cpp`)
//     B  an EAGER EXTERNAL                                  -> between them
//        (Rule S1's middle clause; `worder3_bss_slot_extern.cpp`)
//     C  a STATIC first reached from a FUNCTION body, and every DEFERRED
//        object whatever its linkage                        -> after the code
//
// — and a static is materialised lazily, at its first reference. Change this
// file's reference from the function body to a `.data` initializer and the same
// section moves from slot `C` to slot `A`; give it both and slot `A` wins, the
// earlier of the two (`work/w-order3/cells/O06`).
//
// **This is the workload's common case, not a curiosity.** On the 871-obj
// census, 109 objs put a non-COMDAT `.bss` after the code groups and 33 of those
// sections hold a static — against **0** in slot `A`. Two out-of-sample
// predictions of the model hold there: no purely-static `.bss` is ever in slot
// `B` (0 of 25), and every one of the 138 sections that IS in slot `B` contains
// an external.
//
// **This file is a boundary cell and grades as a gap today**, because the port's
// function decode refuses this TU for unrelated reasons. That is exactly its
// value: whichever lane teaches the decoder this body will emit a `.bss` for a
// TU with functions, and if it reaches for Rule S1 it will place it between the
// watermarks and produce wrong bytes. `coff::data::emit_data_obj` cannot: it
// serves functionless TUs only, and slot `C` is outside it by construction.

struct A{int a;};
static A g;
void f(){ g.a = 1; }
