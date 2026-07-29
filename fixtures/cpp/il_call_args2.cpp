// Characterization: argument ORDER, and the CALL header's per-TU type id.
//
// Part 1 — order. `fwd` and `rev` call the same two-parameter callee with the
// same two values in opposite order, which is the only way to tell "the argument
// region is in source order" from "it is reversed": with one argument, or with
// two arguments passed straight through, the two hypotheses produce the same
// bytes.
//
//   fwd: formals stream 2d e7, 2d e6  ->  args stream b9 e7, b9 e6
//   rev: formals stream 2d eb, 2d ea  ->  args stream b9 ea, b9 eb
//
// Read on its own that is ambiguous — it is equally consistent with "formals
// ascending, args in source order" and "formals descending, args reversed". The
// tie-breaker is `parse_formals`, whose convention is already byte-verified by
// the accepted add-chain class: it collects the `2D` tokens in stream order and
// then REVERSES, so `params[0]` (the r3 argument) is the LAST `2D` token. Under
// that anchor both rows read the same way: **arguments are listed rightmost
// first**.
//
// Part 2 — the type id. The CALL header was previously believed to be a fixed
// ten bytes, `BD <3-byte return type> 00 80 01 10 00 00`. The trailing value is
// really an escaped-varint **function-type id**, deduplicated by signature and
// assigned per TU in declaration order of distinct function types:
//
//   int g1(int)         -> 0x1001
//   int g2(int,int)     -> 0x1003
//   int g3(int,int,int) -> 0x1005
//   int h1(int)         -> 0x1001   <- same *signature* as g1, so it is keyed on
//                                      the type, not the symbol
//   g1 called again     -> 0x1001
//
// Every other fixture in this corpus has exactly one callee signature, which is
// the only reason a literal match ever worked. Real TUs have many signatures, so
// the literal is itself a large-scale decode blocker — and one that fails closed,
// so it costs coverage rather than correctness.

int g2(int, int);
int g1(int);
int g3(int, int, int);
int h1(int);

int fwd(int a, int b) { return g2(a, b); }
int rev(int a, int b) { return g2(b, a); }

int t1(int a) { return g1(a); }
int t3(int a, int b, int c) { return g3(a, b, c); }
int t_same(int a) { return h1(a); }
int t_again(int a) { return g1(a); }
