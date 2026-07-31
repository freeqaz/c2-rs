// WFL — the chained member call whose result's member is FLOATING POINT:
// `float f(O* p) { return p->a()->b()->m; }` is one `lfs f1,off(r3)` after the
// last `bl`, and a `double` member is `lfd`.
//
// WCO shipped the integer form of this designator step (`lwz r3,off(r3)`) and
// refused this one by name, at a cost it MEASURED by counterfactual rather than
// estimated: `mcall-chain-tail-load-class` was 717 functions on the 878-TU dc3
// workload. This rung is that key.
//
// Read off the reference obj (`work/WFL/probe/p1.cpp`, `/O1 /GS- /c`), base
// already in r3 after the outermost call:
//
//   float  f(O* p) { return  p->Next()->gf()->f; }   lfs  f1,4(r3)    c0230004
//   double f(O* p) { return  p->Next()->gf()->d; }   lfd  f1,16(r3)   c8230010
//   float  f(O* p) { return *p->Next()->gpf();   }   lfs  f1,0(r3)    — no fold
//   double f(O* p) { return  p->Next()->gf()->f; }   lfs  f1,4(r3)    — IDENTICAL
//   float* f(O* p) { return &p->Next()->gf()->f; }   addi r3,r3,4     — CallValue
//
// TWO facts make this a different tail rather than a flag on the integer one:
// the value lands in **f1**, the other register file, and the obj acquires the
// undefined external `_fltused` — which `IlFunction::touches_floating_point`
// produces and which W36 lost a symbol by missing on a shape exactly this
// integer-looking.
//
// Every function here must be in class: `c2rs census` N/N.

struct In {
    float  u;   // +0
    double v;   // +8
};

struct M {
    int    a;      // 0
    float  f;      // 4
    float  g;      // 8
    double d;      // 16
    float  arr[4]; // 24
    In     in;     // 40  (u at 40, v at 48)
};

struct N { M m; };

struct O {
    O* Next();
    O* Self();
    M* gf();
    M* gfa(int k);
    N* gn();
    float*  gpf();
    double* gpd();
};

// ---- the load form, both widths, several displacements ----------------------
float  f_f    (O* p) { return p->Next()->gf()->f; }      // lfs f1,4(r3)
float  f_g    (O* p) { return p->Next()->gf()->g; }      // lfs f1,8(r3)
double f_d    (O* p) { return p->Next()->gf()->d; }      // lfd f1,16(r3)

// ---- displacement 0 does NOT fold -------------------------------------------
// The address form of the same designator emits nothing at 0; this one emits
// the load, because `*(r3 + 0)` is a memory read that has to happen.
float  f_arr0 (O* p) { return p->Next()->gf()->arr[0]; } // lfs f1,24(r3)
float  f_deref(O* p) { return *p->Next()->gpf(); }       // lfs f1,0(r3)
double f_dderef(O* p) { return *p->Next()->gpd(); }      // lfd f1,0(r3)
float  f_sub  (O* p) { return p->Next()->gpf()[2]; }     // lfs f1,8(r3)

// ---- the offset RUN folds, exactly as it does for the integer form ----------
float  f_arr2 (O* p) { return p->Next()->gf()->arr[2]; } // lfs f1,32(r3)
float  f_nest (O* p) { return p->Next()->gn()->m.g; }    // lfs f1,8(r3)
double f_nest2(O* p) { return p->Next()->gf()->in.v; }   // lfd f1,48(r3)

// ---- the PROMOTION is free, and that is measured, not assumed ---------------
// `lfs` loads AND converts, so a `float` member returned as a `double` is the
// identical single instruction and the opcode follows the MEMBER's width. The
// reverse direction is `lfd f0 ; frsp f1,f0` and is in the negative file.
double f_promote(O* p) { return p->Next()->gf()->f; }    // lfs f1,4(r3)
double f_promote_arr(O* p) { return p->Next()->gf()->arr[1]; }

// ---- depth, and arguments on the links --------------------------------------
// The tail is independent of everything in front of it.
float  f_three  (O* p) { return p->Self()->Next()->gf()->f; }
float  f_link_arg(O* p, int k) { return p->Next()->gfa(k)->f; }  // Class B
double f_link_lit(O* p) { return p->Next()->gfa(7)->d; }         // Class A

// ---- the receiver is `this` --------------------------------------------------
struct H {
    O* Nx();
    float  q();
    double r();
};
float  H::q() { return Nx()->Next()->gf()->f; }
double H::r() { return Nx()->Next()->gf()->d; }
