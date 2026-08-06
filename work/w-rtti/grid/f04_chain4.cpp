// FRESH: a four-deep single-inheritance chain. numBaseClasses = 4.
struct L0 { L0(); virtual void f(); int a; };
struct L1 : L0 { L1(); virtual void f(); int b; };
struct L2 : L1 { L2(); virtual void f(); int c; };
struct L3 : L2 { L3(); virtual void f(); int d; };
L3::L3(){}
