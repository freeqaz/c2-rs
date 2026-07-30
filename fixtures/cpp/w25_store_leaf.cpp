// **Positive** — the store leaf. Every function here must emit, and the whole
// obj must be byte-exact.
//
// `docs/IL_STORE_LEAF.md`. A body whose entire content is one store into a
// sub-object — `s->m = v;`, `p->Base::m = v;`, `s->arr[2] = v;`, `*p = v;`,
// `s->m = 7;` — is one instruction, and it is the same instruction whichever of
// the two designators spelled the object:
//
//   void f(S* s, int v)  { s->a = v; }   90830000  stw r4,0(r3)  ; blr
//   void f(S* s, int v)  { s->b = v; }   90830004  stw r4,4(r3)  ; blr
//   void D::sb1(int v)   { b1 = v; }     90830004  stw r4,4(r3)  ; blr   (2117)
//   void f(S* s)         { s->a = 7; }   39600007 91630000       li r11,7 ; stw r11
//
// It is the **third** consumer of the designator `try_parse_indirect_load_leaf`
// (`lwz`) and `try_parse_addr_leaf` (`addi`) already share, and the three emit
// three different instructions — which is why the shape is its own production
// and its own census key rather than a flag on either of them.
//
// ## Why this file is a cross product and not a list
//
// The same reason `w16_addr_leaf.cpp` is: the two designators reach the same
// address by different routes, and on the real workload the *plain* one is
// **29x** the intrinsic one (21,269 + 811 against 740 whole-body-complete
// functions). A fixture set that exercised one would say nothing about the
// other, so every axis below is crossed against both spellings.
//
// ## What each function discriminates
//
// `s_a` / `s_b` / `i_b0` / `i_b1` — the zero and nonzero displacements at both
//   designators. Unlike the address leaf, a zero offset here is NOT free: the
//   store still happens, so `stw r4,0(r3)` is a real instruction and this pair
//   separates "fold the offset" from "emit at offset 0".
//
// `s_c` / `s_sh` / `s_uc` / `s_q` / `s_bo` and `i_wc` / `i_ws` / `i_wl` — the
//   stored value's width, 1 / 2 / 8, at both designators. This is the field that
//   picks the opcode (`stb` / `sth` / `stw` / `std`), which is the exact reverse
//   of the address leaf, where the same field reaches nothing at all. A lowering
//   that took the width from the *designator's* pointer tag rather than from the
//   stored TYPE would still pass `s_a`.
//
// `s_p` / `s_pv` — a **pointer** value stored, which is `stw` exactly as an int
//   is. The two 4-byte classes c2 keeps in a GPR share one instruction, which is
//   why `store_value_width` asks `value_class` first.
//
// `s_e2` / `i_t2` / `i_t0` — the `28 00 00` subscript add. `i_t2` carries two of
//   them, so the offset-add run must be unbounded here exactly as it is for the
//   address leaf.
//
// `s_deref` / `s_cast` — no offset add at all (`*p = v;`), and the `2C` cv-strip
//   / cast applied to the **address** (`*(int*)s = v;`), which emits nothing.
//   `s_deref` is also the shape that blocks at `expr-op-0x32` rather than
//   `expr-op-0x27` — a separate census row this production drains.
//
// `s_k` / `s_k0` / `s_kneg` / `s_kwide` / `i_k` / `s_kbo` — a **literal** value,
//   which goes through the scratch register: `li r11,k ; stw r11,d(rB)`. r11 and
//   not r3, read off the capture — a `void` function's r3 holds nothing the ABI
//   cares about, so `li r3` would have been just as plausible. `s_kwide` is the
//   `lis`+`ori` form (70000 does not fit `li`), `s_kneg` a negative one, and
//   `s_kbo` a `bool` literal, whose store is `stb`.
//
// `s_arg2` / `s_arg3` / `i_r4` — the base and the value in registers other than
//   r3/r4. `stw r5,4(r4)` moves BOTH fields, and a lowering that hardcoded either
//   would pass every two-parameter case in this file.
//
// `m_set0` / `m_set1` / `m_setc` — a member function, where `this` is in r3 and
//   is not in the `2D` formals list, so every explicit formal is one register
//   higher than its index implies.
//
// `s_edge` — offset 32764, the largest that still fits the signed 16-bit
//   displacement. 32768 is two instructions and lives in the negative file.
//
// `i_q` — two inheritance steps, so the intrinsic's class descriptor is `66 03`
//   rather than `66 02`.
//
// `loc1` / `loc2` / `loc3` are byte-identical bodies at varied file positions
//   with different neighbours between them (`docs/IL_CALL_IN_EXPR.md` §17.3's
//   locality tell): all three must emit the same `90830004`.
//
// `x_load` / `x_addr` / `x_id` are the three accepted NEIGHBOURS, present so this
//   file fails if the new production ever swallows one of them: a `30` before the
//   `41` is a load (`lwz`), an offset add ending in `41` is an address (`addi`),
//   and no add at all is the identity (bare `blr`).

struct S {
    int a;            // 0
    int b;            // 4
    void* p;          // 8
    char c;           // 12
    short s;          // 14
    unsigned char uc; // 16
    bool bo;          // 17
    long long q;      // 24
    int arr[4];       // 32
};

void s_a  (S* s, int v)            { s->a  = v; }
void s_b  (S* s, int v)            { s->b  = v; }
void s_p  (S* s, void* v)          { s->p  = v; }
void s_pv (S* s, S* v)             { s->p  = v; }
void s_c  (S* s, char v)           { s->c  = v; }
void s_sh (S* s, short v)          { s->s  = v; }
void s_uc (S* s, unsigned char v)  { s->uc = v; }
void s_bo (S* s, bool v)           { s->bo = v; }
void s_q  (S* s, long long v)      { s->q  = v; }
void s_e2 (S* s, int v)            { s->arr[2] = v; }
void s_deref(int* p, int v)        { *p = v; }
void s_cast (S* s, int v)          { *(int*)s = v; }

void s_k    (S* s)                 { s->a  = 7; }
void s_k0   (S* s)                 { s->a  = 0; }
void s_kneg (S* s)                 { s->a  = -3; }
void s_kwide(S* s)                 { s->a  = 70000; }
void s_kbo  (S* s)                 { s->bo = true; }

void s_arg2(int x, S* s, int v)            { s->b = v; }
void s_arg3(int x, int y, S* s, int v)     { s->b = v; }

struct Edge { char pad[32764]; int t; };
void s_edge(Edge* p, int v)        { p->t = v; }

// The accepted neighbours, one token away in each direction.
int  x_load(S* s)                  { return s->b; }
int* x_addr(S* s)                  { return &s->b; }
S*   x_id(S* s)                    { return s; }

// ---- a member function: `this` in r3, formals one register up --------------
struct M {
    int m0;
    int m1;
    char mc;
    void set0(int v);
    void set1(int v);
    void setc(char v);
};
void M::set0(int v) { m0 = v; }
void M::set1(int v) { m1 = v; }
void M::setc(char v) { mc = v; }

// ---- the intrinsic-2117 designator: a member inherited from a base ---------
struct A { int a0; int a1; };
struct B { int b0; int b1; };
struct D : A, B { int d; };

void i_b0(D* p, int v)         { p->b0 = v; }
void i_b1(D* p, int v)         { p->b1 = v; }
void i_r4(int x, D* p, int v)  { p->b1 = v; }
void i_k (D* p)                { p->b1 = 5; }

struct W { char wc; short ws; long long wl; };
struct DW : B, W {
    void swc(char v);
    void sws(short v);
    void swl(long long v);
};
void DW::swc(char v)      { wc = v; }
void DW::sws(short v)     { ws = v; }
void DW::swl(long long v) { wl = v; }

struct AR { int t[4]; };
struct DR : B, AR { };
void i_t0(DR* p, int v)        { p->t[0] = v; }
void i_t2(DR* p, int v)        { p->t[2] = v; }

struct E : D { int e; };
void i_q(E* p, int v)          { p->b1 = v; }

// ---- the locality tell: three byte-identical bodies, spread out ------------
void loc1(S* s, int v)         { s->b = v; }
int  pad1(int a)               { return a + 1; }
void loc2(S* s, int v)         { s->b = v; }
int  pad2(int a, int b)        { return a - b; }
void loc3(S* s, int v)         { s->b = v; }
