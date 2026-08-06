// KNOWN-ANSWER CONTROL — this is OBJ_RDATA_R_SHAPE.md §2's own cell.
// If this one disagrees, the reader is broken, not the spec.
struct A { A(); virtual void f(); int a; };
A::A(){}
