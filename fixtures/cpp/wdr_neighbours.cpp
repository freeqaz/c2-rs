// wdr — the NEGATIVE side of virtual dispatch and the by-value return.
//
// `wdr_virtual_byval.cpp` grades the two constructs. This file grades what they
// are NOT, because the failure mode a decode-only widening has is a width that
// swallows a neighbouring token and lands on a plausible tail anyway.
//
// Two populations:
//
//   * `ctl_*` — bodies that stay IN class and must keep reading
//     `cflow-straight`. If decoding `67` / `9A` / `64` had desynchronized
//     anything, the damage would show first as a control-group body changing
//     shape, because a desync that lands on a legal token keeps walking.
//
//   * `near_*` — one construct away from a virtual call or a by-value return,
//     and each must refuse at a DIFFERENT key than its neighbour in the other
//     file. That is what says the census is measuring the construct rather than
//     the neighbourhood.

struct Val {
    int a, b, c;
};

struct Plain {
    int Get();                 // NOT virtual — direct dispatch, `99` not `67`
    Val* PtrMake();            // returns a POINTER, so no temporary, no `64`
    const Val& RefMake();      // returns a REFERENCE, likewise
    Val v;
    int m;
};

// ---- the control group: in class, single basic block ---------------------
int ctl_add(int a, int b) { return a + b; }
int ctl_lit(int a) { return a + 1; }
void ctl_void() {}
int ctl_deref(int* p) { return *p; }

// ---- one construct away from `67` ----------------------------------------
// A non-virtual member call. `IL_CALL_IN_EXPR.md` §3's claim is that a `99` bind
// site is direct dispatch BY CONSTRUCTION; this is the witness that keeps it
// falsifiable, because it differs from `virt_ptr` only by the word `virtual`.
int near_direct(Plain* p) { return p->Get(); }
// A call through a function POINTER — indirect, but not through a vtable, so it
// has no `67` and no `9A`.
typedef int (*FP)(int);
int near_fnptr(FP f, int a) { return f(a); }
// A pointer-to-member-function call: indirect, and dispatched on a value rather
// than a slot.
typedef int (Plain::*PMF)();
int near_pmf(Plain* p, PMF m) { return (p->*m)(); }

// ---- one construct away from `64` ----------------------------------------
// The same call shape, returning a pointer and a reference instead of a value.
// Neither materializes a temporary, so neither may carry `64`.
int near_ptr_ret(Plain* p) { return p->PtrMake()->a; }
int near_ref_ret(Plain* p) { return p->RefMake().a; }
// An aggregate copy with no call in it at all.
void near_copy(Val* d, const Val* s) { *d = *s; }
// A by-value ARGUMENT rather than a by-value return: the temporary is the
// caller's, and it is bound and pushed, not materialized by a `64`.
int takes(Val v);
int near_byval_arg(Plain* p) { return takes(p->v); }
