// **Negative** — the boundary of the one-byte-unsigned value class. **0 of these
// may be in class**, and the file must never mismatch.
//
// Every expected sequence below was read off the reference obj
// (`work/lf/probes/p2.cpp`, `p4.cpp` and this file, `/Ox /GS- /c`), not derived.
// This file is the more load-bearing half of the rung: the positives cost no
// instruction, so everything that can go wrong is here.
//
// `n_u_from_b` / `n_i_from_b` — the conversion **out of** the class, which is
//   `5463063e` (`rlwinm r3,r3,0,24,31`, i.e. `clrlwi r3,r3,24`) in both
//   directions of spelling. It arrives as the same `2C … 00` token that is free
//   between the two width-4 classes, and it is the reason `ValueClass::Int1u`
//   exists and the reason `eat_value_type(Int1u)` demands the target restate the
//   class. A widening that reused `Int4` here would emit a bare `blr`.
//
// `n_uc_add` — arithmetic. C++ promotes both operands to `int` first, so the
//   body is `clrlwi ; clrlwi ; add ; clrlwi` — four instructions, and its IL has
//   the conversions the guard refuses. `parse_expr`'s `expr-int1u-arith` guard is
//   the second lock: a chain over raw `bool` operands with no `2C` at all has no
//   witness in any capture, so it refuses rather than being assumed free.
//
// `n_not` — `!b` is `546b063e ; 7d6a0034 ; 5543dffe` (mask, `cntlzw`, extract),
//   nothing like a move.
//
// `n_and` — `a && b` is a ten-instruction branchy sequence.
//
// `n_char` / `n_schar` — `char` and `signed char` (`82 11`), the *other* one-byte
//   class. Both are a bare `blr` here, so admitting them would be free **today**
//   — and that is exactly why they refuse: a signed narrow value widened to `int`
//   costs an `extsb` where an unsigned one costs a `rlwinm` or nothing, so the
//   two classes part company one token later and one predicate per fact is the
//   rule. What this refusal costs is written down rather than quietly taken.
//
// `n_local` — a `bool` through a **local**, which reaches the assignment-body
//   parser rather than the straight-line arm. That path calls `parse_expr` (which
//   discards the class) and then the shared `41` gate, so it refuses one token
//   later, honestly. Emitted, it is a bare `blr`; it is a sized handoff, not a
//   silent hole.
//
// `n_tail` — a `bool`-returning **tail call**, whose `BD` return type and `41`
//   annotation are gated in the call path this rung did not widen. c2 emits the
//   ordinary `b <callee>`. 809 functions on the real workload.
//
// `n_ptr` — a `bool*` **pointee** in an arithmetic position rather than a `bool`
//   value, which is a different question entirely (the T3 narrow getter owns the
//   load, and this one feeds it to `+`).

unsigned      n_u_from_b(bool b)  { return b; }
int           n_i_from_b(bool b)  { return b; }
unsigned char n_uc_add(unsigned char a, unsigned char b) { return a + b; }
bool          n_not(bool b)       { return !b; }
bool          n_and(bool a, bool b) { return a && b; }
char          n_char(char c)      { return c; }
signed char   n_schar(signed char c) { return c; }
bool          n_local(bool b)     { bool x = b; return x; }

bool g_call();
bool          n_tail()            { return g_call(); }

int           n_ptr(bool* p)      { return *p + 1; }
