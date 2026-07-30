// **Characterization** — the three spellings of "add a byte offset to a pointer",
// which are three *different* IL productions that lower identically.
//
// This fixture exists because the obvious model — "c1xx desugars `p[k]` to
// `*(p + k)`" — is false, and because `27` and `28` look interchangeable until you
// hold them side by side.
//
//   x_sub   p[1]        b9 <p> 86 43 f4 08  33 86 41 12 04  28 00 00  30 86 41 74
//   x_add   *(p + 1)    b9 <p> 86 43 f4 08  33 86 41 12 04  02        30 86 41 74
//   x_mem   s->b        b9 <s> 86 43 89 20  33 86 41 74 04  27 86 43 f4 08
//                                                            30 86 41 74
//
// All three emit `lwz r3,4(r3)`. So:
//
// * a **subscript** is its own opcode, `28`, with a **two-byte trailing field**;
// * an explicit **pointer add** is the ordinary binary `02` ADD — `02` is
//   polymorphic over (pointer, integer), not integer-only;
// * a **member** offset is `27 <TYPE>`, which re-types the designator (`S *` →
//   `int *`) where `28` leaves the type alone and needs no operand.
//
// The offset is always in **bytes** and already scaled: `p[1]` on an `int *` is
// literal 4, `p[3]` is 12, `p[-1]` is `fc` (−4 in the signed short form), and on a
// `double *` it is 8 (`x_dsub`). With a variable index the scaling is an explicit
// `04` MUL by the element size (`x_var`), which is why an index by a `char *`
// multiplies by a literal 1 that does nothing. So the element size is nowhere in
// the `28` token — it is in the operand.
//
// The offset literal's own type differs between the two forms and is not the
// pointer's: a subscript offset is `86 41 12` (`long`), a member offset is
// `86 41 74` (`int`). A parser that hardcoded `int` there would lose every
// subscript.
//
//   UNKNOWN: the two trailing bytes of `28`. They are `00 00` at every site
//   captured — constant and variable indices; 1-, 4- and 8-byte elements; negative
//   indices; two-dimensional arrays (`p[i][j]`, which chains two `28`s);
//   `w->v[2]`, which chains a `27` then a `28`; a bitfield base; a string literal
//   subscript; and offsets past the 16-bit displacement. Nothing here makes them
//   move, so the port requires exactly `00 00` and refuses otherwise. **A fixture
//   that would separate them:** unknown — I could not construct a source that
//   changes them, which is itself the reason this is recorded as UNKNOWN rather
//   than as "a fixed two-byte pad".
//
// `x_dsub` and `x_var` are also *negatives* for the emission gate — the first
// loads a `double` (`lfd`) and the second needs `slwi r11,r4,2 ; lwzx r3,r11,r3` —
// so this fixture as a whole is out of class and `c2rs diff` reports
// `Port=NotImplemented`. Its job is the byte comparison between its own functions,
// not emission.

struct S { int a; int b; };

int x_sub(int* p) { return p[1]; }
int x_add(int* p) { return *(p + 1); }
int x_mem(S* s) { return s->b; }
int x_sub3(int* p) { return p[3]; }
int x_subneg(int* p) { return p[-1]; }
int x_addneg(int* p) { return *(p - 1); }
double x_dsub(double* p) { return p[1]; }
int x_var(int* p, int i) { return p[i]; }
char x_cvar(char* p, int i) { return p[i]; }
int x_diff(int* p, int* q) { return (int)(p - q); }
