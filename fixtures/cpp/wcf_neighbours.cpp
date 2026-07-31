// wcf — the control-flow boundary, from BOTH sides.
//
// `wcf_shapes.cpp` grades the shapes. This file grades the *edge*: the bodies
// that sit one construct away from control flow and must NOT move.
//
// Two populations, and the census prints both axes so each is checked on its
// own column:
//
//   * `ctl_*` — straight-line bodies that stay IN class. They are the control
//     group for the control-flow axis: every shape the port accepts is a single
//     basic block, so all of them must read `cflow-straight`. A `cflow-if-1`
//     among them would mean the scanner is inventing branches, and a
//     `cflow-loop` would mean the port had been handed a back edge.
//
//   * `edge_*` — bodies that decode as control flow and must keep refusing.
//     A rung that lowered control flow would move these; a rung that only
//     DECODES it must not, and the named key each one refuses at is the
//     assertion.
//
// The pairing is the point: `ctl_max_expanded` and `edge_max` compute the same
// function, and exactly one of them is in class.

// ---- the control group: in class, and single-basic-block ------------------
int ctl_add(int a, int b) { return a + b; }
int ctl_chain(int a, int b, int c) { return a + b + c; }
int ctl_lit(int a) { return a + 1; }
void ctl_void() {}
int ctl_scopes(int a) { { int x = a + 1; { return x + 2; } } }

// The branchless comparison LEAF (W6) — a relation against a literal whose
// value is RETURNED. c2 lowers it with carry-bit arithmetic and no `cmpw` and
// no branch at all, so it is one basic block even though it reads like a
// condition. The SAME relation used as an `if` condition is a branch
// (`edge_if_rel` below, and `wcf_shapes.cpp`'s `cf_if_rel`). Keeping the pair
// adjacent is what stops "a comparison" being treated as one fact.
int ctl_cmp(int a) { return a > 0; }

// ---- the edge: decoded as control flow, and still refused -----------------
// The neighbour of `ctl_cmp`, one construct away: the same relation against the
// same literal, consumed by a branch instead of returned. In class as a value,
// out of class as a condition.
int edge_if_rel(int a) { if (a > 0) return 1; return 2; }

// …and the same thing over two operands, which is the shape a `max` takes.
int edge_max(int a, int b) { if (a > b) return a; return b; }

// Two `return`s, one epilogue label: every return in a function is a `3A` to
// the SAME label, and the label is defined once after the body scope closes.
// The braces add a scope and change nothing else, which is what separates the
// scope stack from the control flow riding on it.
int edge_two_returns(int a) { if (a) { return 1; } return 2; }

// A `goto` and nothing else — no conditional anywhere. This is the only way to
// reach a body with several exits and no branch, and it is why the shape has a
// name of its own even though the workload turned out to hold none of it.
int edge_goto_only(int a) { goto out; out: return a; }

// A loop whose body is empty. The smallest possible back edge: everything the
// block IR would need and nothing else.
int edge_empty_loop(int a) { while (a) { a = a - 1; } return a; }

// The conditional EXPRESSION, which is control flow the statement layer cannot
// see: `43 42` is an operand-stream token, so this body reads `cflow-straight`
// and refuses on the expression axis instead. Pinned here so the limit is a
// tested fact rather than a remark — a reader of the control-flow histogram
// must know that it does not count these.
int edge_ternary(int a, int b) { return a > b ? a : b; }
