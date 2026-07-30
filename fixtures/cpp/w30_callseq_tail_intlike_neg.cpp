// **Negative** — the neighbours of W30's int-like call-tail literal
// (`w30_callseq_tail_intlike.cpp`), each one step outside it and each refused in
// the IL parser so the census and the emission gate cannot disagree.
//
// `eat_int_like` admits a TYPE only when the tag says **4-byte alignment** and
// the kind says **4-byte size**, so every narrow type, the 8-byte types, the FP
// types and the pointers stay out. That boundary is load-bearing rather than
// decoration: `bool f(){ g0(); return false; }` is not `li r3,0` in general (the
// value class has its own extension rules, `docs/CODEGEN_W6_O1.md` / W26), and a
// pointer null is a different production entirely.
//
// The two `_wide` rows are the *other* gate: `li rD,k` carries a signed 16-bit
// immediate, so a literal outside it is `lis`+`ori` and is refused by name
// (`callseq-tail-lit-wide`) rather than truncated into a valid-looking `li` of
// the wrong value.
//
// Decode is all-or-nothing per TU, so the whole file must refuse.

extern int g0();
extern int g1(int);

// --- narrower than 4 bytes: the tag's width nibble refuses ------------------
bool           n_bool()   { g0(); return false; }
char           n_char()   { g0(); return 1; }
unsigned char  n_uchar()  { g0(); return 1; }
short          n_short()  { g0(); return 1; }
unsigned short n_ushort() { g0(); return 1; }
wchar_t        n_wchar()  { g0(); return 1; }

// --- wider than 4 bytes, or not an integer at all ---------------------------
__int64        n_i64()    { g0(); return 1; }
float          n_float()  { g0(); return 1; }
double         n_double() { g0(); return 1; }
void*          n_ptr()    { g0(); return 0; }

// --- an int-like type, but an immediate `li`/`addi` cannot carry -----------
unsigned       n_wide()      { g0(); return 70000; }
long           n_widelong()  { g0(); return -70000; }
unsigned       n_wide_post(int a) { return g1(a) + 70000; }
