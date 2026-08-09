// w-wordwrap GRID G — the file-scope-global store leaf, one cell per clause the
// recognizer draws.  Compiled by real cl.exe/c2.dll under wibo; every fence in
// `global_store_leaf.rs` is a reading off one of these and not a caution.
//
//     work/w-wordwrap/probe.sh probe/gstore.cpp /O1 /Oi /GS- /c
//     work/w-wordwrap/probe.sh probe/gstore.cpp /Ox /GS- /c

unsigned int g_u;
int g_i;
unsigned short g_us;
unsigned char g_uc;
unsigned long long g_ull;
volatile unsigned int g_vol;
static unsigned int g_static;
extern unsigned int g_ext;          // defined in another TU
unsigned int g_arr[4];

// The target's own shape, verbatim.
void G_u(unsigned int x) { g_u = x; }

// Same bytes, different declared type of the object and the formal.
void G_i(int x) { g_i = x; }

// A NARROWER object: the store opcode changes.
void G_us(unsigned short x) { g_us = x; }
void G_uc(unsigned char x) { g_uc = x; }

// A WIDER object: a 64-bit store, and the formal arrives in a register pair.
void G_ull(unsigned long long x) { g_ull = x; }

// `volatile` — the object is the same width and the store is the same word,
// but the qualifier travels in the IL TYPE.
void G_vol(unsigned int x) { g_vol = x; }

// INTERNAL linkage: still `.bss`, but the symbol is STATIC rather than EXTERNAL.
void G_static(unsigned int x) { g_static = x; }

// A global this TU does NOT define — the `.text` is identical and only the
// symbol's storage class differs, which is what makes the class safe to admit
// on `.text` bytes alone.
void G_ext(unsigned int x) { g_ext = x; }

// The value is the SECOND formal: the stored register moves.
void G_second(unsigned int a, unsigned int x) { g_u = x; }

// The value is a LITERAL, not a formal: an `li` appears.
void G_lit() { g_u = 7u; }

// TWO statements: a second `lis` or a reused base?
void G_two(unsigned int x, unsigned int y) { g_u = x; g_i = (int)y; }

// A subscripted destination: the address is computed, not a bare `lis`/`stw`.
void G_arr(unsigned int i, unsigned int x) { g_arr[i] = x; }

// A CONSTANT subscript — the displacement folds into the `stw`.
void G_arr2(unsigned int x) { g_arr[2] = x; }

// The value is WIDENED on the way in: a `clrlwi`/`extsh` may appear.
void G_widen(unsigned char x) { g_u = x; }

// The value is NARROWED on the way in.
void G_narrow(unsigned int x) { g_us = (unsigned short)x; }

// A LOAD of the same global, for contrast — the same `lis` with a `lwz`.
unsigned int G_load() { return g_u; }
