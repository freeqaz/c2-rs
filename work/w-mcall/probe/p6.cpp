struct Val { int a, b; };
struct Ret { Val gv(); };
struct Obj { void v1(int); };
void n_ret_struct(Ret *r) { r->gv(); }
void n_float_arg(Obj *o, float f) { o->v1((int)f); }
