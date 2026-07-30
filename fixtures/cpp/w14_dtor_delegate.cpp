// **Positive** — the compiler-generated **empty destructor** that does nothing
// but destroy its one non-virtual base at offset 0. It is the largest coherent
// sub-shape of `expr-call-in-expr`, the #1 blocking feature on the real dc3
// workload (`docs/IL_CALL_IN_EXPR.md` §5), and it needs **no new codegen**: the
// reference lowers every function in this file to the same four bytes as a bare
// void tail call.
//
//   ??1D1@@QAA@XZ:  48000000  b ??1B1@@QAA@XZ     one REL24, nothing else
//
// The body is a member call, not a leaf, and it reaches the port only because
// each of its three cost centres is provably zero:
//
//   * the receiver is intrinsic **2113 `this-adjust`** with offset **0**, so the
//     adjustment emits nothing and `this` is already in r3;
//   * the bind is `99`, which is DIRECT dispatch — virtual dispatch is opcode
//     `67` with a `9A` bind and a double indirect load through the vtable — so
//     the call is a direct branch;
//   * the call is `void` with zero explicit arguments, and nothing follows it,
//     so there is no result to place and no frame to build.
//
// Captured body (`d1`, from the `LO` marker; the wide type refs vary per class
// but nothing else does):
//
//   53                                SS
//   33 86 41 74 00                    LIT int 0        (role UNKNOWN)
//   26 <??1B1>                        the BASE destructor, pushed first
//   33 86 41 74 80 41 08 00 00        LIT 2113
//   40 86 43 8e 20                    intrinsic call, pointer result
//   66 02 80 20 82 20                 class-pair descriptor, two refs
//   55 86 41 74                       selector argument terminator
//   33 86 41 74 00  55 86 41 74       the adjust OFFSET — zero
//   b9 <this> a6 43 81 20  55 …       the object pointer
//   4c                                -> the adjusted receiver
//   2c a6 43 84 20 00                 cv strip (pointer->pointer: no code)
//   99 86 43 85 20 00                 member bind
//   bd 82 07 03 00 80 05 10 00 00     CALL void, cdecl, fn-type id
//   4c                                ZERO explicit arguments
//   5c 86 41 74 01                    opaque statement trailer
//   4b                                statement end
//   3a <lbl> 54 02 29 <lbl>           return plumbing
//   5e 01 21                          opaque SUB-OBJECT trailer — see below
//   4b  4f 12 47 54 01 54 00          function tail
//
// The `5E` payload is the one field that was worth chasing, because it **varies**
// and it varies with exactly the thing that would make this lowering wrong: a
// two-base destructor emits `5E 02 21` (and two calls, the second with a nonzero
// adjust). So `01` is a real discriminator, not padding, and it is required
// literally. `5C`'s `01` never varied across any witness and is required
// literally for the opposite reason — a field that never varied is
// indistinguishable from a constant (`docs/GAPS.md` §6).
//
// The five cases below vary everything the grammar leaves free: base and derived
// data layout, an empty base, a two-level inheritance chain, and the brace
// placement (`d3`'s `{`/`}` are on their own lines, which inserts a `4F 01 <line>`
// marker inside the return plumbing that a one-line probe never shows).
//
// The base destructors are declared and not defined on purpose: a callee that is
// also defined in the TU is refused wholesale, because c2 may inline it.

struct B1 { ~B1(); int x; };
struct D1 : B1 { ~D1(); int y; };
D1::~D1() {}

// A wider base and a derived class whose own members are of mixed width — the
// class layout reaches the type refs in the descriptor and nothing else.
struct B2 { ~B2(); int a, b, c, d; };
struct D2 : B2 { ~D2(); double e; char f; };
D2::~D2() {}

// Braces on their own lines: the `}`'s source line appears as a `4F 01 <line>`
// marker between the statement end and the `3A` branch of the return plumbing.
struct B3 { ~B3(); int x; };
struct D3 : B3 { ~D3(); };
D3::~D3()
{
}

// Two inheritance levels. The destructor still delegates exactly ONE step, so
// the class-pair descriptor is still `66 02` — which is why that count is
// required literally rather than skipped structurally.
struct G4 { ~G4(); int g; };
struct B4 : G4 { ~B4(); int b; };
struct D4 : B4 { ~D4(); int d; };
D4::~D4() {}

// An empty base. Offset 0 either way, so the adjust literal is still 0.
struct B5 { ~B5(); };
struct D5 : B5 { ~D5(); int d; };
D5::~D5() {}
