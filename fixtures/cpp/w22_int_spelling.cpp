// W22 — the int-like operand type by SPELLING, not by exact triple.
//
// `eat_int_like` was an exact four-triple whitelist (`86 41 74` int, `86 42 75`
// unsigned, `86 41 12` long, `86 42 22` unsigned long), so a width-4 integer
// carrying a *per-TU type id* — an enum, a typedef, a `const`/`volatile`
// qualification — refused, even though `is_int4_type` admits it on the nibbles
// and c2 emits the same instruction. That over-refusal was recorded as ~5,684
// functions attributed from key names; measured properly by counterfactual it is
// **+15,924** (`docs/ROADMAP.md` §6d).
//
// Every function here must census in class and the TU must be `Port=Match` —
// the negatives, which must keep refusing, are in `w22_int_spelling_neg.cpp`.
// The two halves are two files because the port emits an obj only when every
// function in the TU is in class, so a positive sharing a file with a refused
// sibling grades nothing (`docs/GAPS.md` §6).

enum E { E0, E1 };
enum Big { B0 = 0, Bmax = 4294967295u };   // unsigned underlying type
typedef int MyInt;
typedef unsigned MyUns;
typedef long MyLong;

struct S { E e; MyInt mi; const int ci; MyUns mu; volatile int vi; Big b; };

// --- the member getter: the `27` byte-offset add plus the `30` load ---------
E     get_e (S* s) { return s->e; }
MyInt get_mi(S* s) { return s->mi; }
int   get_ci(S* s) { return s->ci; }
MyUns get_mu(S* s) { return s->mu; }
int   get_vi(S* s) { return s->vi; }
Big   get_b (S* s) { return s->b; }

// --- the identity leaf and the `41` result annotation ----------------------
int    id_mi(MyInt a) { return a; }
E      id_e (E a) { return a; }
MyLong id_ml(MyLong a) { return a; }

// --- arithmetic over the same spellings ------------------------------------
MyInt  sum  (MyInt a, MyInt b) { return a + b; }
MyUns  usum (MyUns a, MyUns b) { return a + b; }
int    mixed(MyInt a, MyUns b) { return a + b; }
MyInt  addk (MyInt a) { return a + 7; }
