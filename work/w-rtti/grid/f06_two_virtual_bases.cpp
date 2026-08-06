// FRESH: two virtual bases, no diamond.
struct P { P(); virtual void f(); int p; };
struct Q { Q(); virtual void g(); int q; };
struct R : virtual P, virtual Q { R(); virtual void h(); int r; };
R::R(){}
