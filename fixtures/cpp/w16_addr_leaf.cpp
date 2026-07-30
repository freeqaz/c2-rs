// **Positive** — the address leaf. Every function here must emit, and the whole
// obj must be byte-exact.
//
// `docs/IL_CALL_IN_EXPR.md` §19. A body whose entire value is a sub-object
// *address* — `return &s->m;`, `return &p->Base::m;`, `return s->arr;` — is one
// instruction, and it is the same instruction whichever of the two designators
// spelled the object:
//
//   int* f(S* s)        { return &s->b; }   38630004  addi r3,r3,4  ; blr
//   int* D::pb1()       { return &b1; }     3863000c  addi r3,r3,12 ; blr   (2117)
//   int* f(int x, S* s) { return &s->b; }   38640004  addi r3,r4,4  ; blr
//   int* f(S* s)        { return &s->a; }             blr                   (off 0)
//
// It is exactly the shape `w12_ptr_leaf_neg.cpp` was built to keep OUT of the
// pointer-identity leaf: an identity that skipped an optional offset add would
// emit a bare `blr` where c2 emits an `addi`. That gate stays; this is the
// separate production that emits the `addi`, and `n_addr_of` moved out of the
// negative file into `a_off4` below.
//
// ## Why this file is a cross product and not a list
//
// The two designators reach the same value by different routes and the port
// refuses(-ed) them at two different sites — `try_parse_ptr_identity_leaf` for
// the plain one, `try_parse_base_member_load` for the intrinsic one. A fixture
// set that exercised one would say nothing about the other, and the *plain* form
// is the larger population of the two. So every axis below is crossed against
// both spellings.
//
// ## What each function discriminates
//
// `a_off0` / `a_off4` / `i_off0` / `i_off12` — the zero and nonzero offsets, at
//   both designators. Zero emits **nothing**; the two cases are one `if` apart in
//   `codegen::addr_leaf_text` and a fixture that only had nonzero offsets would
//   not separate "emit an addi" from "emit an addi with a zero immediate".
//
// `a_ofc` / `a_ofd` / `a_ofe` / `a_ofg` and `i_ac` / `i_as` / `i_al` / `i_ad` —
//   the member's own width, 1 / 2 / 8 / 8-aligned. The point is that it does NOT
//   reach the instruction: all of them are the same `addi`, where the *load* leaf
//   beside them picks `lbz` / `lhz` / `lwz` / `ld` from exactly this field. That
//   is why the address path uses `is_ptr_any` rather than `is_ptr_to_4` — and it
//   is a real discriminator, not a formality: `char*` here carries the TYPE tag
//   `86 43` while `short*` carries `84 43`, so the tag's width nibble is not even
//   a dependable statement of the pointee width in this position.
//
// `a_arr2` / `i_arr2` / `i_arr0` — the `28 00 00` subscript add, which the LOAD
//   leaf admits at most **one** of (a second means `slwi`/`lwzx`) and which this
//   shape admits any number of, because every one folds into the displacement.
//   `i_arr2` carries two of them (`LIT(0) 28 · LIT(8) 28`) and `a_arr2` carries a
//   `27` and a `28`. If the run were capped at one, both would refuse.
//
// `a_decay` / `i_decay` — the array-to-pointer decay, which arrives as a
//   `2C <ptr> 00` after the adds and emits nothing.
//
// `a_const` / `i_cb1` — a `const` object, whose designator tag moves from `86` to
//   `A6`. `a_vol` is the volatile spelling.
//
// `a_r4` / `a_r5` / `i_r4` — the base in r4 and r5. The `addi`'s rA field is the
//   base register, not r3, and a lowering that hardcoded r3 would pass every
//   single-argument case in this file.
//
// `a_edge` — offset 32764, the largest that still fits the signed 16-bit
//   immediate. 32768 is `addis`+`addi` and lives in the negative file.
//
// `i_qb1` — two inheritance steps, so the intrinsic's class descriptor is
//   `66 03` rather than `66 02`. `i_nest` is an inherited member of a nested
//   aggregate member, which adds a `27` after the intrinsic.
//
// `x_load` and `x_id` are the two accepted NEIGHBOURS, present so this file
//   fails if the new production ever swallows one of them: a `30` in front of the
//   `41` is a load (`lwz`), and no offset add at all is the identity (bare `blr`).

struct S {
    int a;
    int b;
    char c;
    short d;
    long long e;
    double g;
    int arr[4];
};

int*        a_off0(S* s)              { return &s->a; }
int*        a_off4(S* s)              { return &s->b; }
char*       a_ofc(S* s)               { return &s->c; }
short*      a_ofd(S* s)               { return &s->d; }
long long*  a_ofe(S* s)               { return &s->e; }
double*     a_ofg(S* s)               { return &s->g; }
int*        a_arr2(S* s)              { return &s->arr[2]; }
int*        a_decay(S* s)             { return s->arr; }
int*        a_r4(int x, S* s)         { return &s->b; }
int*        a_r5(int x, int y, S* s)  { return &s->b; }
const int*  a_const(const S* s)       { return &s->b; }
volatile int* a_vol(volatile S* s)    { return &s->b; }
void*       a_void(S* s)              { return &s->b; }

struct Edge { char pad[32764]; int t; };
int*        a_edge(Edge* p)           { return &p->t; }

// The accepted neighbours, one token away in each direction.
int         x_load(S* s)              { return s->b; }
S*          x_id(S* s)                { return s; }

// ---- the intrinsic-2117 designator: a member inherited from a base ----------
struct A { int a0; int a1; };
struct B { int b0; int b1; };
struct D : A, B { int d; };

int*        i_off0(D* p)              { return &p->a0; }
int*        i_off12(D* p)             { return &p->b1; }
int*        i_r4(int x, D* p)         { return &p->b1; }

struct W { char wc; short ws; int wi; long long wl; double wd; };
struct DW : B, W {
    char*       ac();
    short*      as();
    long long*  al();
    double*     ad();
    const int*  cb1() const;
};
char*      DW::ac() { return &wc; }
short*     DW::as() { return &ws; }
long long* DW::al() { return &wl; }
double*    DW::ad() { return &wd; }
const int* DW::cb1() const { return &b1; }

struct AR { int t[4]; };
struct DR : B, AR { };
int*        i_arr0(DR* p)             { return &p->t[0]; }
int*        i_arr2(DR* p)             { return &p->t[2]; }
int*        i_decay(DR* p)            { return p->t; }

struct E : D { int e; };
int*        i_qb1(E* p)               { return &p->b1; }

struct N { int n0; int n1; };
struct NB { N n; };
struct DN : B, NB { };
int*        i_nest(DN* p)             { return &p->n.n1; }
