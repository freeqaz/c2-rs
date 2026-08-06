// FRESH CONTROL: __declspec(novtable) on the BASE only; the derived still
// mints. The spec's novtable cell has no derived class.
struct __declspec(novtable) Nv { Nv(); virtual void f(); int n; };
struct Nd : Nv { Nd(); virtual void f(); int d; };
Nv::Nv(){}
Nd::Nd(){}
