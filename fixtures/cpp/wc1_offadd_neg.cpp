// **Phase 1 slice C1 — the BOUNDARY of the byte-offset add.** Lane `w-c1`,
// 2026-08-24. The negative half of `wc1_offadd.cpp`; every body here must keep
// reporting `NotImplemented`, and no body here may ever produce bytes.
//
// A class is defined by its boundary, and C1's boundary is unusually
// load-bearing because **`27` is not the only way to add to an address**. The
// three neighbours that would look identical under a plausible wrong rule are
// each held here:
//
//   1. `02` — SCALED pointer arithmetic. `p + 1` on an `int*` is `addi r3,r3,4`,
//      the pointee width multiplied in. `27` carries the scaling already. One
//      opcode admitted and the other refused is the whole of §A.
//   2. `28` — the SUBSCRIPT offset add, which does NOT re-type the address.
//      `designator.rs` folds it beside `27` for the four leaf consumers;
//      `parse_expr` has no arm for it and must keep refusing rather than
//      assuming it behaves like its neighbour.
//   3. The **multi-argument call** with a computed address — board **#149**.
//      This is the row C1 did not ship and the reason it did not: the argument's
//      position in the permutation walk was searched over three grids and the
//      fitted rule was **REFUTED at 656/754 = 87.0 %** (`ROADMAP.md` §9.19.5),
//      i.e. it would mis-emit roughly one call in eight, silently. §9.13.1's
//      ALARM is that the `n <= 1` case — which `wc1_offadd.cpp` DOES claim —
//      is exactly where a wrong ordering rule ships green. §C is the six
//      neighbours that separate the two.
//
// Also here: the successors the widening exposed rather than removed. Board
// **#150** measured that unblocking `expr-op-0x27` *renames* far more than it
// converts; §D is that at fixture scale — bodies that were `expr-op-0x27` at
// base and are now `expr-op-0x30` / `expr-op-0x28` / an out-of-class chain key.
// They still refuse. The key moving is the fall-through thesis, not progress.

struct S { int a; int b; int c; };
struct T { int x; S s; int y; };
struct A { int v[8]; };
struct N { A arr; int z; };

extern void a1(int*);
extern void a2(int*, int);
extern void a2b(int, int*);
extern void a3(int, int, int*);
extern void a3b(int*, int, int);
extern void a3c(int, int*, int);
extern void a4(int*, int*, int, int);

// ---------------------------------------------------------------------------
// A — `02`, the SCALED add. `expr-ptr-arith`.
// ---------------------------------------------------------------------------

int n_scaled(int* p, int i) { return (int)(p + i); }
int n_scaled_lit(int* p) { return (int)(p + 1); }

// ---------------------------------------------------------------------------
// B — `28`, the subscript offset add. No arm in `parse_expr`; `expr-op-0x28`.
// ---------------------------------------------------------------------------

int n_index(S* s, int i) { return (int)&s[i]; }
int n_index_const(N* n) { return (int)&n->arr.v[3]; }

// ---------------------------------------------------------------------------
// C — the off-add as a call argument at **two or more arguments**: board #149.
// `call-arg-computed`, in every slot position, because the position is exactly
// what is unknown. If a later lane ships #149 these convert; until then a green
// verdict on any of them is a mis-emit, not a win.
// ---------------------------------------------------------------------------

void n_arg2_first(T* t, int k) { a2(&t->s.b, k); }
void n_arg2_second(T* t, int k) { a2b(k, &t->s.b); }
void n_arg3_last(T* t, int k, int m) { a3(k, m, &t->s.b); }
void n_arg3_first(T* t, int k, int m) { a3b(&t->s.b, k, m); }
void n_arg3_mid(T* t, int k, int m) { a3c(k, &t->s.b, m); }
void n_arg4_two(T* t, N* n, int k) { a4(&t->s.b, &n->z, k, 3); }

// ---------------------------------------------------------------------------
// D — the SUCCESSORS. Each was `expr-op-0x27` at base `e85253cda` and refuses
// under a different key here. Board #150's 201,618 re-filing bodies, in
// miniature.
// ---------------------------------------------------------------------------

// the indirect load standing behind the designator — `expr-op-0x30`
int n_then_load(T* t, int i) { return t->s.b + i; }
int n_then_load_deref(T** pt) { return (int)&(*pt)->s.b; }
// the chain shapes the straight-line selector does not model
int n_mul_by_lit(T* t) { return (int)&t->s.b * 3; }
int n_bitwise(T* t) { return (int)&t->s.b & 15; }
int n_shift(T* t) { return (int)&t->s.b >> 2; }
int n_tree_depth(T* t, T* u) { return (int)&t->s.b - (int)&u->s.a; }
int n_repeated_leaf(T* t) { return (unsigned)&t->y - (unsigned)t; }
int n_mixed(T* t, int i) { return (int)&t->s.b + i * 4; }
