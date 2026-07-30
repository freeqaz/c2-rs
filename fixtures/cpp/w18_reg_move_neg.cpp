// W18 — the register-move boundary (negative fixture).
//
// Every function here must be **out of class**, and the file must never
// mismatch. Each is one production away from `w18_reg_move.cpp`, and each would
// cost c2 something other than the single `mr r3,rN` that fixture grades.
//
// The one that matters most is `agg_*`: a by-value aggregate wider than 8 bytes
// takes more than one argument register, so the formal's *index* stops being its
// *register number* and the move would name the wrong source. That is
// `docs/GAPS.md` §6's fourth instance — the same pair of facts that emitted
// `lwz r3,0(r4)` for `lwz r3,0(r6)` — and it is why the move is gated behind
// `.sy`'s declared widths (`formals_are_one_register_each`) rather than computed
// from the list position alone.
//
// Freestanding, include-free.

struct S { int a; int b; };
struct Big { int a[8]; };          // 32 bytes: more than one argument register
struct Pair { int x, y; };         // 8 bytes: one register, by hidden reference

// ---- value classes that are not one plain GPR word in this parser -----------
//
// Each refuses on its **operand type**, ahead of any question about the move.
// They are listed so the boundary is a measurement: admitting the move must not
// have quietly admitted a narrow or a wide load with it.

short     n_short(int a, short b)         { return b; }
long long n_ll(int a, long long b)        { return b; }
float     n_float(int a, float b)         { return b; }
double    n_double(int a, double b)       { return b; }
char      n_char(int a, char b)           { return b; }
bool      n_bool(int a, bool b)           { return b; }

// ---- the index is not the register ------------------------------------------

int  agg_after(Big v, int b)              { return b; }
int  agg_before(int a, Big v, int b)      { return b; }
Big* agg_ptr(Big v, Big* p)               { return p; }

// The ninth argument is stack-homed, so the value is not in a register at all.
int  nine(int a, int b, int c, int d, int e, int f, int g, int h, int i)
                                          { return i; }

// ---- the value is not an argument -------------------------------------------

int  g_global;
static int g_static;
int  ret_global(int a, int b)             { return g_global; }
int  ret_static(int a, int b)             { return g_static; }

// ---- one production more than a move ----------------------------------------
//
// The neighbours that stay *in* class — `b + 1` (`addi`), `*p` (a load),
// `&s->b` (`addi r3,r4,4`) and an 8-byte by-value aggregate ahead of the moved
// formal — are graded in `w18_reg_move.cpp`, where their bytes are compared
// rather than merely refused.

int  ternary(int a, int b, int c)         { return a ? b : c; }
int* addr_local(int a, int b)             { return &g_global; }
S*   next(int a, S* s)                    { return s + 1; }   // pointer arithmetic
