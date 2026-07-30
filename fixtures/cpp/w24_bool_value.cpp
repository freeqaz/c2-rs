// **Positive** — the one-byte-unsigned value class. Every function here must
// emit, and the whole obj must be byte-exact.
//
// `docs/IL_STORE_LEAF.md` §7 (1). `bool` and `unsigned char` share the operand
// TYPE `82 12 <id>`, and **inside** the class a value is a plain register value
// that costs no instruction at all:
//
//   bool k_false()                    38600000  li r3,0   ; blr
//   bool b_id(bool b)                           blr                 (already r3)
//   bool b_r4(int k, bool b)          7c832378  mr r3,r4  ; blr
//   unsigned char k_uc()              386000c8  li r3,200 ; blr
//
// So the rung is a decode widening with **no new emitter**: `[Lit(k)]` and
// `[Load(t)]` are what the ordinary integer selector has lowered since the MVP,
// and the W18 register move covers every argument slot.
//
// ## The one thing this file exists to pin
//
// A conversion **out of** the class is not free. `unsigned u(bool b)
// { return b; }` is `5463063e` — `rlwinm r3,r3,0,24,31`, a real mask — and it is
// spelled with the same `2C … 00` that is free between the two width-4 classes
// (`w20_convert.cpp`). So `ValueClass::Int1u` is its own class rather than a
// spelling of `Int4`, the `41` result annotation is required to **restate** it,
// and every conversion out of it lives in `w24_bool_value_neg.cpp`. That is the
// whole boundary; the positives below are the part that costs nothing.
//
// ## What each function discriminates
//
// `k_false` / `k_true` / `k_uc` / `k_uc0` / `k_uc255` — a LITERAL of the class,
//   at both `bool` values and across the `unsigned char` range. `li r3,k` is the
//   same word the int literal leaf emits, which is the point: the class reaches
//   no instruction.
//
// `b_id` / `uc_id` — the identity from r3, a bare `blr`.
//
// `b_r4` … `b_r10` and `uc_r4` — the identity from every other argument slot,
//   which is the W18 register move (`mr r3,rN`). `b_r10` is the eighth slot, the
//   last one that is not stack-homed; a lowering that hardcoded r4 would pass
//   every two-parameter case in this file.
//
// `x_get` / `x_set` / `x_int` are the accepted NEIGHBOURS, present so this file
//   fails if the new class ever leaks into one of them: a `bool` **member** read
//   is the T3 narrow getter (`lbz`), a `bool` member written is the W23 store
//   leaf (`stb`), and an `int` literal beside them must keep emitting exactly
//   what it always did.

bool          k_false() { return false; }
bool          k_true()  { return true; }
unsigned char k_uc()    { return 200; }
unsigned char k_uc0()   { return 0; }
unsigned char k_uc255() { return 255; }

bool          b_id(bool b)                 { return b; }
unsigned char uc_id(unsigned char c)       { return c; }

bool          b_r4(int k, bool b)                  { return b; }
bool          b_r5(int k, int l, bool b)           { return b; }
bool          b_r6(int k, int l, int m, bool b)    { return b; }
unsigned char uc_r4(int k, unsigned char c)        { return c; }
bool          b_r10(int a, int b, int c, int d, int e, int f, int g, bool h) { return h; }

// The accepted neighbours, one token away in each direction.
struct S { int i; bool f; unsigned char u; };
bool x_get(S* s)            { return s->f; }
void x_set(S* s, bool v)    { s->f = v; }
int  x_int()                { return 1; }
