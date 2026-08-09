// w-fltret probe v1 — the float value tail of a statement-position member-call
// sequence, reproduced from `float Timer::SplitMs() { Split(); return Ms(); }`
// (src/system/os/Timer.h:137 in the dc3 tree).
struct O {
    void  Poll();
    float Level();
    double DLevel();
    int   ILevel();
    float Fv(int a);
};

// THE TARGET: two member calls on `this`, the second's float result returned.
float v_float(O *o) { o->Poll(); return o->Level(); }

// the same with a double result
double v_double(O *o) { o->Poll(); return o->DLevel(); }

// the integer sibling — this one the port already accepts (CallValue add_k 0)
int v_int(O *o) { o->Poll(); return o->ILevel(); }

// three statements, then the float value tail
float v_float3(O *o) { o->Poll(); o->Poll(); return o->Level(); }

// the float value tail with an argument
float v_float_arg(O *o, int k) { o->Poll(); return o->Fv(k); }
