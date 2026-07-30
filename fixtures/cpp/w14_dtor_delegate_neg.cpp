// **Negative** — the destructors next door to `w14_dtor_delegate.cpp`. Every
// function here must keep refusing, and each one is a case where the accepted
// shape's lowering (a single 4-byte `b <base-dtor>`) would be *wrong bytes*
// rather than merely incomplete. Measured at the workload's own
// `/O1 /Oi /EHsc` against the live 16.00.11886.00 toolchain.
//
// n1 — TWO non-virtual bases. The generated body has **two** member-call
//      statements, the second with adjust offset `04`, and closes with
//      `5E 02 21` instead of `5E 01 21`. That is the witness that `5E`'s first
//      payload byte counts destroyed sub-objects: requiring `01` is what makes
//      this refuse.
//        -> addi r3,r3,4 ; bl ??1M2 ; … two branches and a frame
//
// n2 — a destructor with a real statement (`h()`). The skeleton's first
//      statement matches; a second `26` then stands where the return plumbing
//      must begin. Two calls, a frame.
//
// n3 — a **virtual** destructor. A different production entirely: the body opens
//      on intrinsic 2117 `base-member-addr` (the vtable pointer store), and c2
//      also emits the `??_E`/`??_G` deleting-destructor thunks. Blocks as
//      `expr-intrinsic-base-member-addr` / `expr-load-type-A643A4`.
//
// n4 — a base with a destructor **and** a member with one. Two member-call
//      statements again (the second reaching its member through a `27`
//      byte-offset add rather than the 2113 intrinsic) and `5E 02 21`.
//
// The base destructors are declared and not defined for the same reason as in the
// positive fixture.

struct M1 { ~M1(); int a; };
struct M2 { ~M2(); int b; };
struct N1 : M1, M2 { ~N1(); };
N1::~N1() {}

struct M3 { ~M3(); int a; };
struct N2 : M3 { ~N2(); int v; void h(); };
N2::~N2() { h(); }

struct M4 { virtual ~M4(); int a; };
struct N3 : M4 { virtual ~N3(); };
N3::~N3() {}

struct M5 { ~M5(); int a; };
struct M6 { ~M6(); int b; };
struct N4 : M5 { ~N4(); M6 m; };
N4::~N4() {}
