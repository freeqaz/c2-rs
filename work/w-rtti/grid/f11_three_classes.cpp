// FRESH: three independent polymorphic classes in one TU. The spec's §6
// non-interleave rule is measured on two; this is three, in a scrambled order.
struct C1 { C1(); virtual void f(); int a; };
struct C2 { C2(); virtual void f(); int b; };
struct C3 { C3(); virtual void f(); int c; };
C2::C2(){}
C3::C3(){}
C1::C1(){}
