// FRESH: a class nested inside a CLASS (the spec covers a namespace, `A@N@`).
struct Outer { struct Inner { Inner(); virtual void f(); int i; }; };
Outer::Inner::Inner(){}
