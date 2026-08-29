// W13c — the floating-point MULTIPLY-ADD contraction, as positives.
//
// c2 fuses every `*` that feeds a `+`/`-`; there is no mode in which it emits
// `fmuls`+`fadds` for `a*b+c` (`docs/CODEGEN_W13_FLOAT.md` §3.3). Until lane
// `w-fmadd` these were the whole of `w13_fneg.cpp`'s N1 group — refused,
// because the port's encoder had no field plan for c2's form 24 and its
// lowering had no rule for which operand becomes the addend.
//
// The two facts every function here pins, and both are silent failures:
//
// 1. **Which field the addend lands in.** c2's form-24 arm (`0x10bfa49a`,
//    read by `w-fmadd`) puts the second multiplicand in the `C` field at bit 6
//    and the ADDEND in the `B` field at bit 11 — the mnemonic's order
//    `fmadd FRT,FRA,FRC,FRB`, not the bit layout's. Swapping the two gives a
//    word that disassembles, and that computes `(a*c)+b`.
//
// 2. **Which side of the `-` the product was on.** `fnmsub` computes
//    `B − A*C`, not `A*C − B`, so `a - b*c` is `fnmsubs` and `a*b - c` is
//    `fmsubs` — one opcode apart, opposite sign.
//
// Every body below has strictly ascending leaves and no parentheses, because
// the port models neither c2's FP commutative canonicalization (`b*a + c`
// reaches c2 as the canonicalized `fmadds f1,f1,f2,f3`) nor the `0x59`
// parenthesis marker. `w13_fneg.cpp` keeps the refusing neighbours.

// --- one fused instruction, product on the LEFT ---
float f_fma(float a, float b, float c)            { return a * b + c; }
float f_fms(float a, float b, float c)            { return a * b - c; }

// --- one fused instruction, product on the RIGHT. `f_nms` is `fnmsubs`. ---
float f_madd_r(float a, float b, float c)         { return a + b * c; }
float f_nms_r(float a, float b, float c)          { return a - b * c; }

// --- the fused instruction is NOT the last one, so it takes a scratch
//     register and the trailing op takes f1. ---
float f_chain(float a, float b, float c, float d) { return a * b + c + d; }
float f_chain_r(float a, float b, float c, float d) { return a + b * c + d; }
float f_chain_sub(float a, float b, float c, float d) { return a - b * c - d; }

// --- a deferred product that a second `*` forces out early: the inner
//     multiply really is an `fmuls`, and only the outer one fuses. ---
float f_mulmul(float a, float b, float c, float d) { return a * b * c + d; }

// --- a product on BOTH sides: the left one is fused, the right one is
//     materialised into f0 FIRST. ---
float f_twoprod(float a, float b, float c, float d) { return a * b + c * d; }
float f_twoprod_sub(float a, float b, float c, float d) { return a * b - c * d; }

// --- TWO live temporaries, which is where c2's FP scratch policy becomes
//     MODE-DEPENDENT and where the port was wrong at master. `/Ox` carries the
//     pool cursor (f0, then f13); `/O1` recycles the register the instruction
//     itself just killed (f0, then f0). Three leaves can never separate the
//     two, because three leaves need only one temporary — which is why no
//     fixture had ever caught it. See `FpTempPolicy`. ---
float f_chain4(float a, float b, float c, float d) { return a + b + c + d; }
float f_mad4(float a, float b, float c, float d, float e) { return a * b + c + d + e; }
double d_chain4(double a, double b, double c, double d) { return a + b + c + d; }

// --- double: primary opcode 63 rather than 59, same four fields. ---
double d_fma(double a, double b, double c)        { return a * b + c; }
double d_fms(double a, double b, double c)        { return a * b - c; }
double d_nms_r(double a, double b, double c)      { return a - b * c; }
double d_twoprod(double a, double b, double c, double d) { return a * b + c * d; }
