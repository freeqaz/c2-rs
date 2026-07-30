// Reading a member inherited from a non-virtual base — intrinsic **2117**
// `base-member-addr`, the largest single decode bucket in the class-layout family
// (6.3% of blocked functions). All must emit, byte-exact.
//
// A member declared *directly* in the object's own class uses the ordinary `27`
// offset-add and has been in class since the indirect-load leaf landed. A member
// inherited from a base does not: c1xx emits an intrinsic call instead, whose
// three arguments are `(member offset within the base, base offset, object
// pointer)` — and the address is their **sum**, so it lowers to the very same
// `lwz rD, off(rB)`. Nothing new was needed in codegen; this was purely a decode.
//
//   struct A { int a0, a1; }; struct B { int b0, b1, b2; }; struct D : A, B {};
//   p->b2   args (8, 8)   ->  lwz r3,0x10(r3)      16 = 8 + 8
//   p->b0   args (0, 8)   ->  lwz r3,8(r3)
//   p->a1   args (4, 0)   ->  lwz r3,4(r3)
//
// `both_nonzero` is the load-bearing case. Every simpler shape has one of the two
// literals zero, so "the offsets add" and "take whichever is nonzero" agree on all
// of them; only a member at a nonzero offset inside a base at a nonzero offset
// separates the two rules. It is the same discipline `fixtures/README.md` records
// for `w5_chain.cpp` — add the neighbour that would look identical under a
// plausible wrong rule.
//
// `deep_*` are the second axis. The argument header is `66 <n>` followed by *n*
// two-byte type references, and `n` counts inheritance steps: 2 for `D : A, B`,
// 3 for `E : D`. The first version of this decode matched the six-byte `n = 2`
// header as a constant and so silently refused every multi-level case — a bound
// that was invisible from inside the shapes it was written against. The header is
// now skipped structurally, and `n > 3` refuses because nothing past 3 is captured.
//
// Still out of class, deliberately, and each for its own reason: virtual
// inheritance (2116/2118, a vbtable indirection rather than a constant), an upcast
// (2114, null-guarded), a *write* to a base member, and a base member of narrow or
// floating type. Neighbours worth keeping in mind when widening this.

struct A {
    int a0;
    int a1;
};
struct B {
    int b0;
    int b1;
    int b2;
};
struct D : A, B {
    int d;
};
struct E : D {
    int e;
};

int both_nonzero(D* p) { return p->b2; }
int base_only(D* p) { return p->b0; }
int member_only(D* p) { return p->a1; }
int own_member(D* p) { return p->d; }

int deep_base(E* p) { return p->a1; }
int deep_mid(E* p) { return p->b2; }
int deep_own(E* p) { return p->e; }
