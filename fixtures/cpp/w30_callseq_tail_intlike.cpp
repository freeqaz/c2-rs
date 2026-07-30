// W30 — the int-like LITERAL at a call tail, by spelling and not by exact triple.
//
// Three positions in the call productions read a literal `33 <TYPE> <k>` and each
// of them required the TYPE to be **exactly** `86 41 74` (`int`):
//
//   * `return <literal>;` after a statement call  (`SeqTail::Lit`      -> `li r3,k`)
//   * `return g(a) + k;`  the single framed call  (`FramedCall::add_k` -> `addi r3,r3,k`)
//   * `g1(); return g2() + k;` the sequence's value call (`SeqTail::CallValue`)
//
// `eat_int_like` — the locator `2C`, `41`, `30` and W22's operand positions all
// already agree through — admits any width-4 integer on its tag/kind nibbles, so
// `unsigned`, `long`, `unsigned long`, an `enum`, a `const int` and a
// `volatile int` are the same instruction and were refused only because their
// third byte is a per-TU type id. On the 878-TU workload the tail position alone
// is **+7,771 functions** (the whole `callseq-tail-lit` bucket, one cause), and
// the two post-op positions are 0 there — they are widened with it because
// leaving one copy of a rule on a narrower gate than the other two is exactly the
// defect shape `docs/GAPS.md` §6 #9 records.
//
// Every function here must census in class and the TU must be `Port=Match`;
// the neighbours that must keep refusing are in `w30_callseq_tail_intlike_neg.cpp`
// (a positive sharing a file with a refused sibling grades nothing — §6).

extern int g0();
extern int g1(int);
extern int g2(int, int);
// The post-op rows below need the CALLEE's result type to be the literal's type
// too: `unsigned f(int a){ return g1(a) + 1; }` with an `int` callee inserts a
// `2C` conversion after the add and blocks on `result-type-0x2C`, which is a
// different production and not what this fixture is about.
extern unsigned gu(int);
extern long gl(int);
extern unsigned gu0();
extern long gl0();

enum E { Em1 = -1, E0 = 0, E1 = 1, Ebig = 32767 };

// --- SeqTail::Lit: `li r3,k` after the last `bl`, over the spellings ---------
unsigned       t_uint()      { g0(); return 3; }
long           t_long()      { g0(); return 4; }
unsigned long  t_ulong()     { g0(); return 5; }
const int      t_cint()      { g0(); return 6; }
volatile int   t_vint()      { g0(); return 7; }
E              t_enum()      { g0(); return E1; }
int            t_int()       { g0(); return 8; }   // the control: the old class

// --- the immediate's boundaries, which the `li` field pins ------------------
E              t_enum_neg()  { g0(); return Em1; }
E              t_enum_max()  { g0(); return Ebig; }
unsigned       t_max()       { g0(); return 32767; }
long           t_min()       { g0(); return -32768; }

// --- more calls, and arguments through the shared marshalling locators ------
unsigned       t_two()       { g0(); g0(); return 3; }
unsigned       t_three()     { g0(); g0(); g0(); return 3; }
unsigned long  t_arg(int a)  { g1(a + 1); g0(); return 6; }
E              t_perm(int a, int b) { g2(b, a); g0(); return E1; }

// --- FramedCall::add_k: `addi r3,r3,k` with an int-like post-op type --------
unsigned       p_uint(int a) { return gu(a) + 1; }
long           p_long(int a) { return gl(a) + 32767; }
unsigned       p_mink(int a) { return gu(a) + 0xFFFF8000u; }  // -32768 as unsigned

// --- SeqTail::CallValue: the sequence's last call, plus an int-like k -------
// (nullary callees: a formal read by the *second* call would be live across the
// first, which is Class B and refused by name — see `mvp_call_seq_neg.cpp`.)
unsigned       s_uint()      { g0(); return gu0() + 2; }
long           s_long(int a) { g1(a); return gl0() + 7; }
unsigned       s_two()       { g0(); g0(); return gu0() + 30000; }
