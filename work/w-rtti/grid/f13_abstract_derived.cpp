// FRESH: an abstract base with a concrete derived — _purecall in the base's
// vftable and a real slot in the derived's.
struct Ab { Ab(); virtual void f() = 0; virtual void g(); int a; };
struct Cn : Ab { Cn(); virtual void f(); int c; };
Ab::Ab(){}
Cn::Cn(){}
