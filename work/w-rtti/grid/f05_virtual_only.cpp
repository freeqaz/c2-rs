// FRESH: VIRTUAL inheritance with a SINGLE base. The spec only exhibits
// attributes 0, 1 and 3; this should be the 2 it never shows.
struct Vb { Vb(); virtual void f(); int v; };
struct Vd : virtual Vb { Vd(); virtual void g(); int d; };
Vd::Vd(){}
