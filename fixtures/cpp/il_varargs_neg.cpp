// **Negative** — a variadic function, refused on its **name**, because its body IL
// carries no trace of the ellipsis.
//
// This was a live wrong-bytes emit on the port's *oldest* accepted shapes, found by
// an adversarial reviewer:
//
//   int va(int a, ...) { return a; }
//   c2:   std r4,0x18(r1) … std r10,0x48(r1) ; blr    + a .pdata entry  (6 sections)
//   port: blr                                                          (5 sections)
//   Port=Mismatch @ offset 2   — NumberOfSections, 06 against 05
//
// It fired on `straight-line` identities and add chains, `indirect-load-leaf`
// getters, `__stdcall`, and member functions with or without an explicit formal, in
// every optimization mode including the workload's own `/O1` family.
//
// MEASURED, and the reason this gate is where it is: the `.ex` and `.sy` streams of
// `int va(int a, ...) { return a; }` and `int va(int a) { return a; }` are
// **byte-identical** — 2745 B and 30 B, compared whole. Only `.gl` differs, by the
// one byte of the mangled name, and `.db`. So no body-level gate can see this, and
// there is no production to decode. The mangled name is the signal, and it is an
// ABI fact rather than a heuristic: MSVC terminates the argument list with `@` (a
// list ended), `X` (no arguments) or `Z` (an ellipsis), then closes with a final
// `Z`, so a variadic name ends `ZZ`.
//
// The whole point of this fixture is the neighbours, because two obvious readings of
// that rule are wrong:
//
//   z_type    int z_type(Z*)          ?z_type@@YAHPAUZ@@@Z    a type NAMED Z
//   no_args   int no_args()           ?no_args@@YAHXZ         ends XZ, not @Z
//   enum_arg  int enum_arg(E)         ?enum_arg@@YAHW4E@@@Z    (see below)
//   v_free    int v_free(int, ...)    ?v_free@@YAHHZZ         VARIADIC
//   v_only    int v_only(...)         ?v_only@@YAHZZ          VARIADIC
//   v_mem     int C::v_mem(int, ...)  ?v_mem@C@@QBAHHZZ       VARIADIC
//
// `z_type` is why the test is not "contains a Z", and `no_args` is why it is not
// "does not end `@Z`". Both are **in class** and must stay in class; if this gate
// ever over-reaches, they are what says so. `enum_arg` carries a `Z`-free name with
// a `4` in it and is a useful third spelling, but it refuses on its own merits —
// `expr-load-type-864185`, an enum-typed load, a different rung — so it witnesses
// the *name* rule and not the gate's boundary. Said explicitly because a reader who
// assumed all three neighbours were in class would think this fixture proves more
// than it does.
//
// The variadic functions make the whole TU refuse, so nothing here is byte-graded
// and the **census** is what grades it: `z_type` and `no_args` report a shape, and
// exactly the three variadics report `fn-varargs`.
//
// One asymmetry, recorded so the census is not over-read: on a *real* translation
// unit the census cannot see this at all. It pairs names positionally and only when
// `.gl` yields exactly one per body, which real TUs do not satisfy, so it has no
// name to test and `fn-varargs` never fires there — measured 0 across 878 TUs. The
// *gate* does see it, because `functions` binds names per record. The asymmetry is
// therefore in the safe direction (the gate is stricter than the census claims) and
// it is the pre-existing name-pairing limitation, not a new one — but it means this
// key measures the fixture corpus and nothing else until names bind on real input.
//
// An `extern "C"` variadic function has an undecorated name and is invisible to this
// rule. It is covered today for an unrelated reason — `gl_defined_names` accepts
// only `?…@@…` forms, so a TU containing one binds no names and refuses whole
// (measured: `extern "C" int cva(int, ...)` is `Port=NotImplemented`) — and that
// coupling is recorded in `mangled_is_varargs` so a future loosening of the name
// rules cannot silently uncover this.

struct Z { int z; };
enum E { E0, E1 };

struct C {
    int m;
    int v_mem(int x, ...) const;
};

int z_type(Z* p) { return p->z; }
int no_args() { return 1; }
int enum_arg(E e) { return (int)e; }

int v_free(int x, ...) { return x; }
int v_only(...) { return 1; }
int C::v_mem(int x, ...) const { return x; }
