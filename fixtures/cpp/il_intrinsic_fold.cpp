// **Negative** — what c2 does to an intrinsic call whose arguments are constant,
// and how an intrinsic result composes with the surrounding expression. All eight
// must keep refusing; the fixture exists so that a future lowering cannot be
// written from the IL bytes alone.
//
// **c1xx does NOT fold intrinsics.** `abs(-5)` reaches c2 intact —
// `33 86 41 74 0f  40 86 41 74  33 86 41 74 >fb<  55 86 41 74  4C`, selector 15
// applied to the literal −5 — so every constant-folding decision here belongs to
// the backend, exactly as it does for `a + a` and `a / 2.0f`
// (`fixtures/cpp/il_repeated_leaf.cpp`, `w13b_ffold.cpp`).
//
// **And the fold is per-intrinsic, not per-shape.** This is the split that only a
// fixture holding both halves can separate:
//
//   f_abs_k     abs(-5)      -> 38600005  li r3,5            FOLDED
//   f_rotl_k    _rotl(1u,4)  -> 38600010  li r3,16           FOLDED
//   f_fabs_k    fabs(-1.5)   -> lis r11 ; lfd f0,0(r11) ; fabs f1,f0
//                                                            NOT folded — and it
//                               pools `__real@bff8000000000000`, i.e. c2 emits
//                               the *unfolded* constant and applies `fabs` at
//                               run time
//   f_sqrt_k    sqrt(4.0)    -> lis ; lfd ; fsqrt f1,f0      NOT folded, pools
//                               `__real@4010000000000000`
//
// So "an intrinsic over literals is a constant" is wrong for the floating-point
// half of the table and right for the integer half. A lowering that folded
// `sqrt(4.0)` to a pooled `2.0` would be a mis-emit; one that emitted a runtime
// `abs` sequence for `abs(-5)` would be a different one.
//
// **The argument region can contain another `0x40`.**
// docs/IL_CAST_CONVERT.md §6 listed this as an open question ("not observed at an
// aligned site, but not excluded"). `c_abs_twice` settles it — the selector/token
// pair nests exactly the way `26 <callee> BD` does, outer first:
//
//   33 86 41 74 0f  40 86 41 74          <- outer abs
//     33 86 41 74 0f  40 86 41 74        <- inner abs
//       b9 <a> 86 41 74  55 86 41 74
//     4C                                 <- inner apply
//     55 86 41 74                        <- inner result pushed as outer's arg
//   4C
//
// **The result register is chosen by the consumer, not by the intrinsic.** This is
// what makes even the one-instruction cases unlowerable today:
//
//   c_fabs_add   fabs(a) + b   -> fabs f0,f1 ; fadd f1,f0,f2   (into f0)
//   c_sqrt_fabs  sqrt(fabs(a)) -> fabs f0,f1 ; fsqrt f1,f0     (into f0)
//   (a bare `fabs(a)`, in il_intrinsic_bits.cpp)  -> fabs f1,f1  (into f1)
//
//   c_abs_plus   abs(a) + 1    -> srawi r11 ; xor r10 ; subf >r11<,r11,r10 ; addi r3,r11,1
//   c_abs_twice  abs(abs(a))   -> srawi r11 ; xor r10 ; subf >r9< ,r11,r10 ; srawi r8,r9,31 ; …
//
// Same three-instruction expansion of the same selector, different destination
// register, chosen by what consumes it — the W5 scratch ladder, which the port
// models for arithmetic leaves and not for intrinsic results.
//
// `c_fabs_arg` and `c_abs_of_sum` are the mirror case (the intrinsic's *argument*
// is a computed expression rather than its result being consumed) and are here so
// a gate written around only the consumer side still has a case that separates it.

extern "C" {
int abs(int);
double fabs(double);
double sqrt(double);
unsigned int _rotl(unsigned int, int);
}

int f_abs_k() { return abs(-5); }
unsigned int f_rotl_k() { return _rotl(1u, 4); }
double f_fabs_k() { return fabs(-1.5); }
double f_sqrt_k() { return sqrt(4.0); }

int c_abs_twice(int a) { return abs(abs(a)); }
double c_sqrt_fabs(double a) { return sqrt(fabs(a)); }
int c_abs_plus(int a) { return abs(a) + 1; }
double c_fabs_add(double a, double b) { return fabs(a) + b; }
double c_fabs_arg(double a) { return fabs(a + 1.0); }
int c_abs_of_sum(int a, int b) { return abs(a + b); }
