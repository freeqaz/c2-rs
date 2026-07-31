// wda — the control group for the data-symbol boundary.
//
// `wda_dataaddr_neg.cpp` holds the bodies that refuse. This file holds their
// **minimal pairs**: the same call shapes with the string-literal address
// replaced by an int literal, and nothing else changed. Every one of them is in
// class, so the census reads N/N here and 0/N there.
//
// The pairing is the assertion. `nb_one_lit` and the negative's `neg_one_sym`
// are the same statement — an assignment whose right-hand side is one ordinary
// call — and they differ by exactly one operand token:
//
//     nb_one_lit    26 <uci> BD … 33 <int> 07 55 …        an int literal
//     neg_one_sym   26 <uc>  BD … 26 <$SG> 2C … 55 …      a data symbol's ADDRESS
//
// One is in class and one is 1,058 workload functions of
// `expr-call-in-expr-data-addr-1sym-then-plain-call-whole`. That is the whole
// distance the rung would have had to cover, stated as source rather than as a
// row count, and it is why WDA is a measurement and not a lowering: the missing
// piece is not a token in the expression grammar, it is a `.rdata` string pool,
// a `$SG<n>` / `??_C@` symbol and a REFHI/REFLO relocation quad in the object
// writer (`docs/IL_CALL_IN_EXPR.md` §17.2).
//
// WDA changes no acceptance at all, so every function here was in class before
// it and must still be: this file is the under-claiming check, which is the
// direction a census-key rung can break without any byte compare noticing.

int uci(int);
void v0(int);
int uci2(int, int);

// The minimal pair of `neg_one_sym` — one ordinary call as an assignment's
// right-hand side, its argument an int literal instead of a symbol address.
int nb_one_lit() { int x; x = uci(7); return x; }

// Two operands rather than one: the pair of `neg_two_sym`'s arity, so that
// "two arguments" cannot be what the negative is refusing. Both are formals,
// because a *literal* in a non-leading argument slot is its own refusal
// (`call-arg-computed`) and would put a second reason in this file.
int nb_two_arg(int a, int b) { return uci2(a, b); }

// The same call in a return position rather than an assignment — the statement
// position that `docs/IL_CALL_IN_EXPR.md` §9.2 shows can move a whole function
// between buckets. It must not move this one.
int nb_tail_lit() { return uci(7); }

// A formal forwarded rather than a literal, so the control group is not made
// only of constants.
int nb_forward(int a) { return uci(a); }

// The void flavour, matching `neg_two_sym`'s return type.
void nb_void_lit() { v0(7); }

// The straight-line leaf under all of them — no call at all. If this ever left
// class the file would be measuring the harness, not the boundary.
int nb_leaf(int a, int b) { return a + b; }
