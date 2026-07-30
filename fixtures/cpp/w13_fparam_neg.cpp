// W13a — the floating-point PARAMETER boundary (negative fixture).
//
// Every function here MUST be out of class, and the file must never mismatch.
//
// ## What this file used to be, and why it shrank
//
// It held nineteen functions and pinned the gate `try_parse_float_leaf` used to
// close `docs/GAPS.md` §6's sixth and seventh live wrong-bytes emits — **every
// formal must appear as an FP operand of the body**, which is correct and which
// refused every mixed parameter list outright. Sixteen of those nineteen are now
// **emitted and byte-exact**: W27 replaced the gate with the actual register
// numbering, read from `.sy`'s type kind, and they moved verbatim into
// `fixtures/cpp/w27_fp_reg.cpp` (33/33 in class, `Port=Match`). The over-refusal
// was MEASURED at 1,005 functions on the 878-TU workload before it was taken
// (`docs/IL_CALL_IN_EXPR.md` §23.1) and this file is what it cost locally.
//
// What is left is the three that were never about parameters at all. They are
// kept here because a negative fixture whose subject has been implemented is a
// gate that grades nothing, and the honest repair is to shrink it to the cases
// that still refuse rather than to delete it and lose the boundary.
//
// ## The rule they actually pin: a REPEATED LEAF
//
// A parameter read twice licenses c2's algebraic rewriter, and the rewrite lands
// in `.rdata`:
//
//   float f(float a, float b) { return b * b; }
//
// is not simply `fmuls f1,f2,f2` — with one operand appearing twice, c2 is free
// to reassociate and to fold, and the constant path (W13b) is a pooled `.rdata`
// COMDAT with a REFHI/REFLO relocation pair. `try_parse_float_leaf` refuses any
// chain whose LOAD tokens are not distinct, and that refusal is orthogonal to
// which register a parameter occupies — which is exactly why these three did not
// move when the numbering did.
//
// Freestanding, include-free, leaf-only. Compiled by `c2rs bench` and by every
// `scripts/mode_lane.sh` lane.

// A repeated leaf in a division, with a non-FP formal ahead of it — so the
// parameter numbering is exercised and is *not* what refuses.
float  mix_i_f(int a, float b)                  { return b / b; }

// A repeated leaf beside an unused FP parameter: `a` occupies f1 and is never
// read, `b` is f2 and is read twice.
float  fp_unused(float a, float b)              { return b * b; }

// The same in a member function, where `this` takes r3 and the FP file is
// unchanged.
struct C { float mf(float x) const; };
float  C::mf(float x) const                     { return x * x; }
