// WEC (negative) — the neighbours of the empty base-delegating constructor, one
// per refusal, each of which really does emit something this rung cannot.
//
// `c2rs census` must report **0 of N** in class here. Each of these is a live
// wrong-bytes emit if admitted, not a conservative gap:
//
//   Nv   a polymorphic derived class — the base moves to offset 4 (`addi
//        r3,r3,4`) AND the vfptr store is a second statement (64 B, not 48).
//   Nm   two destructible bases — two `bl`s in reverse declaration order.
//   Nl   a LITERAL base-constructor argument — `li r4,3` before the `bl`.
//   Np   a PERMUTED forwarding — `B0(b, a)` needs two `mr`s through a temp, and
//        beside a callee-saved copy c2 breaks the cycle through the callee-saved
//        register rather than r11 (uncharacterized).
//   Nq   an argument that is a formal PAST the argument count — `B0(b)` from
//        `(a, b)` is `mr r4,r5`, not a permutation.
//   Nw   a WIDENING conversion on the forwarded argument — `int` into a
//        `long long` parameter is a real instruction, and the two TYPEs differ.
//   Nf   a forwarded FLOATING-POINT argument. MEASURED as a mismatch before it
//        was refused: the obj carries `_fltused` and the port emitted one symbol
//        short, `Port=Mismatch @ offset 12` in all five modes. An *unused* FP
//        formal costs nothing; passing the value is what does it.
//   Ni   a member initializer beside the base — a second statement.

struct B0 { B0(); B0(int); B0(int, int); B0(long long); B0(float); int x; };
struct B1 { B1(); ~B1(); int x; };
struct M1 { M1(); ~M1(); int y; };

struct Nv : B1 { Nv(); virtual void v(); };
Nv::Nv() {}

struct Nm : B1, M1 { Nm(); };
Nm::Nm() {}

struct Nl : B0 { Nl(); };
Nl::Nl() : B0(3) {}

struct Np : B0 { Np(int a, int b); };
Np::Np(int a, int b) : B0(b, a) {}

struct Nq : B0 { Nq(int a, int b); };
Nq::Nq(int a, int b) : B0(b) {}

struct Nw : B0 { Nw(int a); };
Nw::Nw(int a) : B0((long long)a) {}

struct Nf : B0 { Nf(float f); };
Nf::Nf(float f) : B0(f) {}

struct Ni : B1 { Ni(); int m; };
Ni::Ni() : m(0) {}
