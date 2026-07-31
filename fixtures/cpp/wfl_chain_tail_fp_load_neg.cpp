// WFL negative neighbours — one case per refusal row, each with its own census
// key and each with the reference instruction it would have had to emit.
// `c2rs census` must report **0/N**: every one of these is a chained member call
// followed by an FP designator step, i.e. this rung's own production, stopped by
// a gate this rung declares rather than by a decode failure.
//
// Measured in `work/WFL/probe/p4.cpp` (`/O1 /GS- /c`), base already in r3:
//
//   double member -> float  result   lfd f0,k(r3) ; frsp f1,f0
//   float  member -> int    result   lfs f0,k(r3) ; fctiwz f0,f0 ; stfd ; lwz r3
//   int    member -> float  result   lwa r11 ; std ; lfd f0 ; fcfid ; frsp f1,f0
//   volatile float member            lfs f1,k(r3)     — ONE word; see below

struct M {
    int             a;   // 0
    float           f;   // 4
    double          d;   // 8
    volatile float  vf;  // 16
};

struct O {
    O* Next();
    M* gf();
};

// ---- the NARROWING, which is the one direction that is not free -------------
// `lfd f0 ; frsp f1,f0` — two words, and the load's destination is **f0**, the
// FP pool's first scratch, not the result register. The promotion in the other
// direction is byte-identical to the unpromoted body and IS admitted (see the
// positive file); a rule that refused "any conversion" would have been a
// discount applied to a measured-free cell.  `mcall-chain-tail-load-fp-narrow`
float n_narrow (O* p) { return (float)p->Next()->gf()->d; }
float n_narrow2(O* p) { return p->Next()->gf()->d; }

// ---- OUT of the FP file entirely --------------------------------------------
// `fctiwz` plus a spill through the frame and a reload — four words and a stack
// slot, which is a frame model this tail does not have.
// `mcall-chain-tail-load-fp-convert`
int n_f2i(O* p) { return p->Next()->gf()->f; }
int n_d2i(O* p) { return p->Next()->gf()->d; }

// ---- INTO the FP file from an integer member --------------------------------
// The mirror of the row above, and it belongs to WCO's `-load-convert` gate
// rather than to any of this rung's: the load type is a width-4 integer, so
// `value_class` answers and the FP class test never runs. WCO's header records
// this gate as having NO witness ("the width gate fires first on every spelling
// a caller can write"); it has one, and this is it.
// `mcall-chain-tail-load-convert`
float  n_i2f(O* p) { return p->Next()->gf()->a; }
double n_i2d(O* p) { return p->Next()->gf()->a; }

// ---- the residue of `-load-class` after the FP class moved out --------------
// A `volatile float` member. c2 emits the IDENTICAL single `lfs f1,16(r3)` —
// measured, `c_vol` in `work/WFL/probe/p4.cpp` — so this refusal costs coverage
// rather than correctness. It is kept because the predicate this rung asks is
// the SHARED `is_fp_type`, whose volatile refusal is right at the position it
// was written for (a `volatile float` FORMAL is a spill, and `float f(float x,
// volatile float y){ return gf(y); }` is a 40-byte framed body). Widening it
// means splitting that locator by position, which is a rung in `readers.rs` and
// not a line here. Its measured worth on the workload is in the rung doc.
// `mcall-chain-tail-load-class`
float n_volatile(O* p) { return p->Next()->gf()->vf; }

// ---- what may follow the step ------------------------------------------------
// An FP post-op pools a constant and adds; that is a whole second production
// (`float_leaf_text`'s `.rdata` COMDAT and its REFHI/REFLO quad) and the body
// keeps the `-then-…-more` key that names it, not one of this rung's.
float n_postop(O* p) { return p->Next()->gf()->f + 1.0f; }
