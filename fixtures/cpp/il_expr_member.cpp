// **Positive** — the member accessor, which is the same indirect-load leaf reached
// through an implicit `this`. Must emit, and the obj must be byte-exact.
//
// A member function's `this` is **not** in the `2D` formals list. The pre-body
// region carries it separately:
//
//   53 53 26 <fn> b9 <this> a6 43 82 20 99 86 43 84 20 00 46 2d <q> 4c 4F 11
//                 ^^^^^^^^^^^^^^^^^^^^^ ^^^^^^^^^^^^^^^^^^^^
//                 LOAD this (C * const)  bind-member, offset 0
//
// so `parse_formals` sees only `q` and would map it to r3. It is r4. Captured:
//
//   int C::g(int* q) const        { return *q; }  ->  80640000  lwz r3,0(r4)
//   int C::i(int v, int* q) const { return *q; }  ->  80650000  lwz r3,0(r5)
//   int D::s(int* q)              { return *q; }  ->  80630000  lwz r3,0(r3)
//
// `this` takes r3 and every explicit formal shifts up one; a *static* member
// function has no `this` and does not shift. `gp`/`gpv` and `st_p` are that
// three-way separator: a rule that ignored `this` would emit `lwz r3,0(r3)` for
// `gp` — plausible-looking, wrong register, wrong bytes. `parse_this_token` finds
// the binding by requiring `B9 <tok> <TYPE> 99 <TYPE> 00` to land **exactly** on
// the `46` formals marker, and refuses if no candidate or more than one does.
//
// `get_a`/`get_b` are the shape this class exists for — a `const` getter, which is
// most of a game engine's function count. `const` on `this` propagates into the
// load type, so the body carries a conversion the non-const sibling does not:
//
//   int C::get_b() const  b9 <this> a6 43 82 20  33 86 41 74 04
//                         27 a6 43 8e 20  30 a6 41 8d 20  2c 86 41 74 00
//                         41 86 41 74                    ->  80630004
//   int C::nc_b()         b9 <this> a6 43 81 20  33 86 41 74 04
//                         27 a6 43 f4 08  30 86 41 74
//                         41 86 41 74                    ->  80630004
//
// Same instruction; the `2C` is a cv-strip and costs nothing. Note the *pointer*
// operand's tag is `a6` in both, because `this` is `C * const` either way — the
// difference is in the pointee (`a6 41 …` const int vs `86 41 74` int).
//
// `0x9B` is NOT how any of this works, even though the census bucket `body-0x9B`
// invites that reading: member access is a composition of a byte-offset add
// (`27 <TYPE>`) and an indirect load (`30 <TYPE>`), with no member opcode at all,
// and `0x9B` is a *temporary* designator (`il_expr_temp.cpp`). The two are also
// easy to confuse structurally, and their trailing fields differ:
// `99 <TYPE> <varint>` against `9B <TYPE> <token>`.

struct C {
    int a;
    int b;
    int get_a() const;
    int get_b() const;
    int nc_b();
    int gp(int* q) const;
    int gpv(int v, int* q) const;
};

int C::get_a() const { return a; }
int C::get_b() const { return b; }
int C::nc_b() { return b; }
int C::gp(int* q) const { return *q; }
int C::gpv(int v, int* q) const { return *q; }

struct D {
    int a;
    static int st_p(int* q);
};

int D::st_p(int* q) { return *q; }
