// W21 — the census/gate boundary, NEGATIVE side (roadmap #44).
//
// Every function here decodes cleanly and is refused, and each one was refused
// **in codegen** until the gate was moved into the IL parser — so each was
// counted by `function_census` and then declined by `PortC2`. This file exists
// to keep that from coming back: it must census **0/N**, and `c2rs census` must
// print no `census/gate DISAGREEMENT` line for it.
//
// Nothing here is a candidate for widening. They are refusals with byte
// evidence behind them (`docs/CODEGEN_W5_SCRATCH.md`, `docs/CODEGEN_W6_O1.md`).

// --- operand-stack depth 3: needs a second scratch --------------------------
// The §24.7 case itself. `a`, `b` and `c` are all live when the `*` fires.
int add_then_mul(int a, int b, int c) { return a + b * c; }
int sub_then_mul(int a, int b, int c) { return a - b * c; }
int deep_tree(int a, int b, int c, int d, int e) { return (a + b) * (c + d) * e; }

// --- the depth-2 tree's two characterized rewrites --------------------------
// N1, product flattening: c2 re-linearizes a `*` root over a `*` child.
int n1_product(int a, int b, int c, int d) { return (a * b) * (c * d); }
// N2, additive canonicalization: an additive root over an additive child is
// reassociated into a chain, not emitted as a tree.
int n2_additive(int a, int b, int c, int d) { return (a + b) + (c + d); }

// --- comparison leaves the difference spine cannot encode -------------------
// `==`/`!=` form `a - k` as `addi r11,a,-k`, which needs the literal's UNSIGNED
// value to fit the immediate; against a large unsigned c2 materializes the
// constant and subtracts instead, one instruction more.
int u_eq_max(unsigned a) { return a == 4294967295u; }
int u_ne_m5(unsigned a) { return a != 4294967291u; }
// `-(-32768)` does not fit the immediate.
int s_eq_i16min(int a) { return a == -32768; }
