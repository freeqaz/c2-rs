// W4b2: framed non-leaf call. `f` calls the external `g`, USES the result
// (`+ 1`), and returns it — so `f` allocates a stack frame (non-leaf) and the
// obj gains a `.pdata` unwind section plus the compiler label symbols
// $M2545/$M2546/$T2547. First fixture with a `.pdata` section and an ADDR32
// relocation. See docs/CODEGEN_PPC_MVP.md (W4b2) and docs/OBJ_FORMAT_MVP.md.
int g(int);
int f(int a) { return g(a) + 1; }
