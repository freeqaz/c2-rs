// A polymorphic class whose constructor is declared and NOT defined here, with
// a pointer to it. Nothing mints a vftable, so this should be 0 records — a
// negative control that says the trigger really is the definition.
struct A { A(); virtual void f(); int a; };
A* p;
void use(A* q){ p = q; }
