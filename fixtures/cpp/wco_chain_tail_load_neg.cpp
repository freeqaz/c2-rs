// WCO negative neighbours — one case per refusal row, each with its own census
// key. `c2rs census` must report **0/N**: every one of these is a chained member
// call followed by a designator step, i.e. this rung's own production, stopped
// by a gate this rung declares rather than by a decode failure.
//
// The width table these draw the line on was measured in one TU
// (`work/WCO/probe/p6.cpp`, `/O1 /GS- /c`), base already in r3:
//
//   int / int* / nested / subscripted   lwz  r3,k(r3)      — the accepted class
//   char, unsigned char                 lbz  r3,k(r3)
//   short, unsigned short               lhz  r3,k(r3)
//   long long                           ld   r3,k(r3)
//   char widened to int                 lbz  r11,k(r3) ; extsb r3,r11
//   float / double                      lfs / lfd  f1,k(r3)

struct Wide {
    char pad[40000];
    int far;
};

struct M {
    int a;
    char c;
    unsigned char uc;
    short s;
    unsigned short us;
    long long ll;
    float f;
    double d;
};

struct O {
    O* Next();
    M* gf();
    Wide* gw();
};

// ---- the displacement --------------------------------------------------------

// PAST THE 16-BIT DISPLACEMENT — `addis`+`addi` or an indexed load with a
// scratch register, two words rather than one. The gate is on the SUM, the same
// boundary the indirect-load leaf draws.  `mcall-chain-tail-off-wide`
int n_far(O* p) { return p->Next()->gw()->far; }
char* n_far_addr(O* p) { return &p->Next()->gw()->pad[39000]; }

// ---- the loaded width --------------------------------------------------------

// A NARROW member is `lbz` / `lhz`, not `lwz` — a different instruction, and
// the two spell the same designator.  `mcall-chain-tail-load-width`
char n_char(O* p) { return p->Next()->gf()->c; }
unsigned char n_uchar(O* p) { return p->Next()->gf()->uc; }
short n_short(O* p) { return p->Next()->gf()->s; }
unsigned short n_ushort(O* p) { return p->Next()->gf()->us; }

// AN 8-BYTE member is `ld`, which is DS-form besides.
// `mcall-chain-tail-load-width`
long long n_ll(O* p) { return p->Next()->gf()->ll; }

// A WIDENING to `int` is `lbz r11 ; extsb r3,r11` — TWO instructions, and for a
// signed halfword the count is optimization-mode dependent. The width gate
// above catches it first; the case is here because the widening is what a
// caller actually writes.  `mcall-chain-tail-load-width`
int n_char_widened(O* p) { return p->Next()->gf()->c; }

// ---- the register file -------------------------------------------------------

// A FLOATING-POINT member is `lfs`/`lfd` into **f1**, a different register file,
// and the TU acquires `_fltused` besides.  `mcall-chain-tail-load-class`
float n_float(O* p) { return p->Next()->gf()->f; }
double n_double(O* p) { return p->Next()->gf()->d; }

// ---- what may follow the step ------------------------------------------------

// A POST-OP on the loaded value puts the load in r11 and adds — the same reason
// the indirect-load leaf refuses `*p + 1`. It is not this tail.
int n_postop(O* p) { return p->Next()->gf()->a + 1; }
