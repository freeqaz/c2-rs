// WCO — one DESIGNATOR STEP on a chained member call's pointer result:
// `return p->a()->b()->m;` and `return &p->a()->b()->m;`.
//
// WCH shipped the chain that ENDS at its outermost call and WCL added the
// arguments; this rung adds the one instruction that may follow it. Both cells
// were read off the reference obj (`work/WCO/probe/p1.cpp`, `/O1 /GS- /c`):
//
//   int  f(O* p) { return  p->Next()->gf()->m; }   bl ; bl ; lwz  r3,4(r3)
//   int  f(O* p) { return  p->Next()->gf()->a; }   bl ; bl ; lwz  r3,0(r3)
//   int* f(O* p) { return &p->Next()->gf()->m; }   bl ; bl ; addi r3,r3,4
//   int* f(O* p) { return &p->Next()->gf()->a; }   bl ; bl ;   (nothing)
//
// The ADDRESS form is `SeqTail::CallValue`, shipped since #35 rung 1 — a
// recognizer and nothing else, and it folds `+0` away by itself. The LOAD form
// is one new tail, `SeqTail::CallLoad`, and it does NOT fold at offset 0. Those
// two middle rows are the whole content of the rung and they are four bytes of
// `.text` apart.
//
// Every function here must be in class: `c2rs census` N/N.

struct In {
    int x;
    int y;
};

struct M {
    int a;      // offset 0
    int m;      // offset 4
    int b;      // offset 8
    int* pi;    // offset 12 — a POINTER member, the second width-4 class
    In in;      // offset 16
    int arr[8]; // offset 24
};

struct O {
    int n;
    O* Next();
    O* Self();
    M* gf();
    M* gfa(int k);
    int* gpi();
};

// ---- the load form, at several displacements --------------------------------
// One `lwz r3,off(r3)` each. Offset 0 emits the instruction; the address form
// below at the same member emits none, which is the pair that separates them.
int f_off0(O* p) { return p->Next()->gf()->a; }
int f_off4(O* p) { return p->Next()->gf()->m; }
int f_off8(O* p) { return p->Next()->gf()->b; }

// A POINTER member is the other class c2 lowers with the identical bare `lwz`.
int* f_ptr(O* p) { return p->Next()->gf()->pi; }

// ---- the offset RUN folds ---------------------------------------------------
// A nested member and a constant subscript are `27 · 27` and `27 · 28`; both
// sum into one displacement, exactly as the indirect-load leaf has folded them
// since W35. A single-add recognizer would refuse all four of these.
int f_nest(O* p) { return p->Next()->gf()->in.y; }
int f_arr(O* p) { return p->Next()->gf()->arr[2]; }
int* f_arr_addr(O* p) { return &p->Next()->gf()->arr[3]; }
In* f_nest_addr(O* p) { return &p->Next()->gf()->in; }

// ---- no offset add at all ---------------------------------------------------
// A bare `30` with nothing in front of it — the same `lwz r3,0(r3)`.
int f_deref(O* p) { return *p->Next()->gpi(); }
int f_sub(O* p) { return p->Next()->gpi()[3]; }

// ---- the address form -------------------------------------------------------
// `addi r3,r3,k`, and NOTHING at k = 0.
int* f_addr0(O* p) { return &p->Next()->gf()->a; }
int* f_addr4(O* p) { return &p->Next()->gf()->m; }
int* f_addr8(O* p) { return &p->Next()->gf()->b; }

// ---- depth, and arguments on the links --------------------------------------
// The tail is independent of everything in front of it: a third link is one
// more `bl`, and WCL's link marshalling still runs before the last call.
int f_three(O* p) { return p->Self()->Next()->gf()->m; }
int f_link_arg(O* p, int k) { return p->Next()->gfa(k)->m; }   // Class B
int f_link_lit(O* p) { return p->Next()->gfa(7)->m; }          // Class A
int* f_link_arg_addr(O* p, int k) { return &p->Next()->gfa(k)->b; }

// ---- the receiver is `this` --------------------------------------------------
struct H {
    O* Nx();
    int q();
    int* r();
};
int H::q() { return Nx()->Next()->gf()->m; }
int* H::r() { return &Nx()->Next()->gf()->b; }
