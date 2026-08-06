// KNOWN-ANSWER CONTROL — the spec's §5 worked example, `struct D:B`.
struct B { B(); virtual void f(); int b; };
struct D : B { D(); virtual void f(); int d; };
D::D(){}
