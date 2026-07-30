// **Negative** — the boundary of the store leaf. **0 of these may be in class**,
// and the file must never mismatch: every function here emits something the
// store production does not, so admitting any one of them is wrong bytes rather
// than a gap.
//
// Every expected sequence below was read off the reference obj
// (`work/lf/probes/p1.cpp` and this file, `/Ox /GS- /c`), not derived.
//
// `n_f` / `n_d` — a **floating-point** value. `s->f = v` is `d0230014`
//   (`stfs f1,20(r3)`) and `s->d` is `d8230018` (`stfd f1,24(r3)`): the value
//   comes out of the FP argument file, whose register number counts the FP
//   parameters *alone*. That is the same off-by-one `float_leaf_text`'s header
//   records as a live mis-emit, and it is why `store_value_width` asks
//   `value_class`/`sized_ptee` rather than reading the TYPE's width nibble —
//   `86 45 40` and `88 85 41` are 4 and 8 bytes wide and would otherwise select
//   `stw r4` / `std r4`.
//
// `n_widen` — a **conversion of the value**: a `bool` parameter stored into an
//   `int` member carries `2C 86 41 74 00` and emits `548b063e ; 91630000`
//   (`clrlwi r11,r4,24 ; stw r11,0(r3)`). Free-looking and not free. The
//   production admits a `2C` on the value only in the two 4-byte classes, where
//   `w23_store_leaf.cpp`'s `s_pv` grades it byte-exact.
//
// `n_narrow` — the same in the other direction: an `int` parameter narrowed to a
//   `char` member. Also a `2C`, also over a width that is not 4.
//
// `n_add` / `n_lit_add` — a **computed** value: `s->a = x + y` is
//   `7c832214 ; 90830000` — the arithmetic lands in the scratch register first,
//   so the body is two instructions and the third op of the stream is not a bare
//   `Load`/`Lit`.
//
// `n_two` — **two** stores. The production requires the body to end at the first
//   `4B`, and this is the shape that dominates the intrinsic-2117 designator's
//   `calls-0` population on the real workload (a setter that also marks a dirty
//   flag), so it is the neighbour most likely to be reached by a loosened gate.
//
// `n_value` — the store's **result used**: `return s->a = v;`. The `32` yields
//   the stored value and `4B` discards it; a body that goes on to return it is a
//   different shape.
//
// `n_edge` — a displacement of 32768, one past what a store's signed 16-bit `D`
//   field holds. c2 emits `addis` + a store, two instructions.
//
// `n_global` — a store into a **global**, which needs a `.data`/`.bss` symbol and
//   an ADDR32 relocation pair (W14), and whose destination is pushed with `26`
//   rather than through a pointer designator.
//
// `n_index` — a store at a **variable** index (`s->arr[i] = v`), which is
//   `stwx` with a scaled index, not a folded displacement.
//
// `n_aggr` — an **aggregate** assignment, which is a copy loop / `memcpy`, not a
//   store instruction at all.
//
// `n_load` — a store whose value is itself **loaded from memory**
//   (`d->a = s->a`), which is `lwz r11 ; stw r11` — two instructions and a value
//   that is not a register argument.
//
// `n_arg9` — the base in the **ninth** parameter slot, which is stack-homed and
//   needs a frame.

struct S {
    int a;
    char c;
    float f;
    double d;
    int arr[4];
    struct Inner { int p; int q; } in;
};

void n_f(S* s, float v)        { s->f = v; }
void n_d(S* s, double v)       { s->d = v; }
void n_narrow(S* s, int v)     { s->c = (char)v; }
void n_add(S* s, int x, int y) { s->a = x + y; }
void n_lit_add(S* s, int x)    { s->a = x + 1; }
void n_two(S* s, int v)        { s->a = v; s->arr[0] = v; }
int  n_value(S* s, int v)      { return s->a = v; }
void n_index(S* s, int i, int v) { s->arr[i] = v; }
void n_aggr(S* s, S::Inner t)  { s->in = t; }
void n_load(S* d, S* s)        { d->a = s->a; }
void n_arg9(int a, int b, int c, int d, int e, int f, int g, int h, S* s, int v) { s->a = v; }

struct M { int m0; void setb(bool v); };
void M::setb(bool v)           { m0 = v; }

struct Edge { char pad[32768]; int t; };
void n_edge(Edge* p, int v)    { p->t = v; }

int g_i;
void n_global(int v)           { g_i = v; }
