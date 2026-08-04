// **W-R1c positive cell — the EXTERNAL-linkage dynamic initializer.**
//
// `fixtures/cpp/il_dyninit_static.cpp` is the *static* cell, and it is the only
// one this project had. The two workload TUs board #158 targets differ from each
// other in exactly one structural axis — `OBJ_DYNINIT_SHAPE.md` §7.2: "The only
// structural difference between the two workload TUs is the `.bss` symbol's
// linkage" — and with only the static fixture that axis had no fixture at all.
//
// So this is `ZlibLicense.cpp`'s shape and the other one is
// `TomCryptLicense.cpp`'s:
//
//   static L sL(...)  ->  `.gl` `$sL`,  COFF `sL`,  STATIC (3), undecorated
//          L gL(...)  ->  `.gl` `?gL@@3UL@@A` == COFF name, EXTERNAL (2)
//
// The `$` is the discriminator and it is a `.gl` **name separator**, not part of
// the name — which is why the internal-linkage symbol comes out undecorated. It
// is also why the two workload TUs reported *different* census blockers
// (`data-sym-unresolved` vs `data-sym-not-extern`) from **byte-identical** `.ex`
// files: `gl_symbol_index` opens a name run only after `00` or `26`, so a
// `24`-introduced name was never seen at all.
//
// Three members, so `sizeof` is 12 and alignment is 4 — the aggregate cell that
// separates "the TYPE tag is the size" from "the TYPE tag is the ALIGNMENT".
// `IL_TYPE_TAGS.md` §1 tabulates the tag under the heading `size`, which is true
// only for scalars, where size *is* alignment. Here the tag is `86` (align 4)
// while the size field says `0c`.
//
// `<name>$initializer$` stays STATIC and undecorated for BOTH linkages (§4.3),
// so this cell also pins that the initializer symbol is read from `.gl` rather
// than derived from the object's decorated name — `?gL@@3UL@@A` still yields
// `gL$initializer$`.
//
// Graded at every lane. It converts only where `/GF` is implied (`/O1`, `/O2`)
// and must stay `NotImplemented` at `/Ox`, where the literal is a non-COMDAT
// `$SG<n>` `.rdata` placed BEFORE `.text` with 5 relocations instead of 9.

struct L { L(const char* s, int r); int a, b, c; };
L gL("abc", 0);
