// FRESH: a diamond whose virtual base is itself derived — five records deep,
// attributes MI|VI at more than one level.
struct Gb { Gb(); virtual void f(); int g; };
struct Vbase : Gb { Vbase(); virtual void f(); int v; };
struct Lft : virtual Vbase { Lft(); virtual void l(); int l_; };
struct Rgt : virtual Vbase { Rgt(); virtual void r(); int r_; };
struct Dia : Lft, Rgt { Dia(); virtual void f(); int d; };
Dia::Dia(){}
