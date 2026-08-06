// A function-local static of a polymorphic type: the vfptr write is inside a
// real `.text` body, so this is expected to have one — a control for z01/z02.
struct A { virtual void f(); int a; };
A* get(){ static A a; return &a; }
