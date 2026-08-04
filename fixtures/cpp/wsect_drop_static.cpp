// w-sect / board #174 — the NEGATIVE-SPACE fixture: c2 emits the bare
// four-section shell for this TU, with no `.bss` and no symbol.
// An internal-linkage object that is UNINITIALIZED and UNREFERENCED is dropped
// entirely. OBJ_DATA_BSS_SHAPE.md does not have this rule: §5.2's static cells
// are all "8 uninit statics AND ONE FUNCTION EACH", so every object in them is
// referenced and the drop is invisible there. The differential found it.
static int za;
