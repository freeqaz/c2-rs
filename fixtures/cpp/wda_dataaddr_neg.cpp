// wda — the data-symbol row, refused, and the pair the census could not tell
// apart until WDA.
//
// Every function here is 0/N in class and must stay that way: WDA lowers
// nothing. What it changes is the **census key** each one prints, and the two
// `neg_recv_*` bodies are why. Before WDA they printed one identical key.
//
//   neg_recv_one_sym   expr-call-in-expr-recv-load-then-call-data-addr-1sym-whole
//   neg_recv_two_syms  expr-call-in-expr-recv-load-then-call-data-addr-2sym-whole
//
// That key — 10,540 functions over 828 TUs at the post-WVB workload, the
// largest `-whole` member-call row on the board — carried **no symbol count**,
// because D5 gated the count on the body's own `CallForm` and here the
// designator arrives as the *second* blocker. The count is what decides the
// rung: one symbol is a REFHI/REFLO relocation pair the port could emit
// (`docs/IL_CALL_IN_EXPR.md` §17.2), two symbols is
// `addi rD, rAnchor, <difference of .rdata pool offsets>` and needs a whole-TU
// pool layout visible to instruction selection (§17.3 (a)) — a phase, not a
// rung.
//
// The `neg_*_sym` bodies below are the four keys of the plain-call family, kept
// here as the reproduction of the sizing: the assigned row's name was checked
// against hand-written source before any of it was believed, and
// `neg_two_sym_assert` is §17.1's shape 1 (`f(T*, "…", <line>, "…")`, 1,211 of
// 2,730 in the argument-shape walk) written out as C++.

struct T;
struct O {
    int M1(const char*);
    int M2(const char*, const char*);
};

int uc(const char*);
int u3(T*, const char*);
void d1(const char*, const char*);
void a1(T*, const char*, int, const char*);

// ---- the plain-call family: the data designator IS the body's form ---------

// -> expr-call-in-expr-data-addr-1sym-then-plain-call-whole            (1,058)
int neg_one_sym() { int x; x = uc("hi"); return x; }

// -> expr-call-in-expr-data-addr-1sym-then-plain-call-and-type-ptr-whole2
//                                                                     (1,660)
// One construct more than `neg_one_sym` and the suffix goes `-whole` ->
// `-whole2`: the extra construct is the POINTER operand type, not the string.
int neg_one_sym_ptr(T* p) { int x; x = u3(p, "cc"); return x; }

// -> expr-call-in-expr-data-addr-2sym-then-plain-call-whole                (5)
// One string MORE than `neg_one_sym` and the suffix does NOT move: two
// designators are one construct. Only the `-Nsym` class separates them, which
// is exactly why D5 had to put the count in the key.
void neg_two_sym() { d1("aa", "bb"); }

// -> expr-call-in-expr-data-addr-2sym-then-plain-call-and-type-ptr-whole2
//                                                                    (18,926)
// THE ASSIGNED ROW — the largest key on the board carrying a whole-body
// completeness bit, and §17.1's assert-macro shape.
void neg_two_sym_assert(T* p) { a1(p, "expr", 42, "file"); }

// ---- the member-call family: the designator is the SECOND blocker ---------
// The pair WDA exists to separate. Same receiver form, same second blocker,
// same grant count, same `-whole` suffix; one string apart.

// -> expr-call-in-expr-recv-load-then-call-data-addr-1sym-whole
int neg_recv_one_sym(O* p) { int x; x = p->M1("hi"); return x; }

// -> expr-call-in-expr-recv-load-then-call-data-addr-2sym-whole
int neg_recv_two_syms(O* p) { int x; x = p->M2("aa", "bb"); return x; }
