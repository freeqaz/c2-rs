// The 32-bit integer types that are **interchangeable** for `+`, `-`, `*` and
// add-immediate folding: `int`, `unsigned`, `long`, `unsigned long`.
//
// A measured equivalence, not an assumption. Each pair below was captured signed
// and unsigned side by side and the words are identical — PPC's `add`/`subf`/
// `mullw` do not distinguish signedness, and neither does c2 for these operators:
//
//   int  a+b+c   7d632214 7c6b2a14   ==   unsigned a+b+c   7d632214 7c6b2a14
//   int  a-b     7c641850            ==   unsigned a-b     7c641850
//   int  a*b*c   7d6321d6 7c6b29d6   ==   unsigned a*b*c   7d6321d6 7c6b29d6
//   int  a+7     38630007            ==   unsigned a+7u    38630007
//   long a+b+c   7d632214 7c6b2a14   ==   int a+b+c
//
// Before this, `parse_expr` accepted `86 41 74` (int) and nothing else, so every
// `unsigned` operand refused — the `expr-load-type-864275` census bucket, 1.6% of
// blocked functions, blocked by an over-narrow type check rather than by anything
// c2 actually does differently.
//
// The equivalence is deliberately scoped. It does NOT extend to division or the
// shift-right family, where signedness genuinely changes the instruction (both
// refused elsewhere), and it does not extend to the narrow types, whose extension
// placement depends on the operator *and* the result type — see
// `il_type_narrow.cpp` and `docs/IL_TYPE_TAGS.md` §3.2.
//
// Mixed signedness is not here: `int + unsigned` puts a `2C` convert in the IL
// and is a separate class. `il_type_narrow.cpp` holds that case.

int s_add3(int a, int b, int c) { return a + b + c; }
int s_sub(int a, int b) { return a - b; }
int s_mul3(int a, int b, int c) { return a * b * c; }
int s_lit(int a) { return a + 7; }

unsigned u_add3(unsigned a, unsigned b, unsigned c) { return a + b + c; }
unsigned u_sub(unsigned a, unsigned b) { return a - b; }
unsigned u_mul3(unsigned a, unsigned b, unsigned c) { return a * b * c; }
unsigned u_lit(unsigned a) { return a + 7u; }

long l_add3(long a, long b, long c) { return a + b + c; }
long l_sub(long a, long b) { return a - b; }
unsigned long ul_add3(unsigned long a, unsigned long b, unsigned long c) { return a + b + c; }
unsigned long ul_lit(unsigned long a) { return a + 7u; }
