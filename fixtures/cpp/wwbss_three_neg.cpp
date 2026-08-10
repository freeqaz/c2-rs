// w-wordwrap2 — THREE objects in one non-COMDAT `.bss`, which is ABOVE the
// measured bound and must be REFUSED.
//
// `docs/OBJ_DATA_BSS_SHAPE.md` §8.1, quoted rather than restated: a `.bss` with
// exactly two objects is right on **47 of 48** real sections and anything larger
// is **38 of 62**. The residue is the WALK ORDER (board #184) and emphatically
// not the arithmetic — of the 64 real sections whose walk needs no alignment
// padding anywhere, where every candidate allocator coincides by construction,
// **10 are still wrong**.
//
// `work/w-wordwrap2/probe/p7.obj` is one of those walks and it is worth reading:
// c2 lays out `g3@0 g1@4 g2@8` for a file that declares `g1 g2 g3`, so the walk
// is `.gl` order and it is neither declaration order nor its reverse. The port
// has that order; what it does not have is a cell saying the two-object result
// generalizes, and #184 says it does not.
//
// This is `fixtures/cpp/wwrap_gstore_widths.cpp`'s shape too — three objects,
// three functions — which is why that file grades `codegen-gap` rather than
// `match` even though all three of its bodies are `fnbyte-exact`.

unsigned int g1;
unsigned int g2;
unsigned int g3;

void S1(unsigned int x) { g1 = x; }
void S2(unsigned int x) { g2 = x; }
void S3(unsigned int x) { g3 = x; }
