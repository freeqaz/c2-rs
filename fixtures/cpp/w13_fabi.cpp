// W13a — the floating-point calling convention (characterization fixture).
//
// Pins: float/double parameters land in f1..f13 numbered by *float-parameter
// order* (not positional slot), the result comes back in f1 for both widths,
// float and double share the same registers, and a positional slot past the
// 8th still burns a GPR home slot. Freestanding, include-free, leaf-only.

float  fp_pass1(float a)                            { return a; }
float  fp_pass2(float a, float b)                   { return b; }
double dp_pass2(double a, double b)                 { return b; }
double dp_pass3(double a, double b, double c)       { return c; }

// float and int parameters are numbered in separate sequences: `c` is the 2nd
// float, so it is f2 even though it is the 3rd positional parameter.
float  fp_skip(float a, int b, float c)             { return c; }

// The 9th float parameter is still an FPR (f9); the FPR file is deeper than the
// 8-slot GPR home area.
float  fp_nine(float a, float b, float c, float d, float e,
               float f, float g, float h, float i)  { return i; }

// 13 doubles exhaust f1..f13; the 14th spills to the stack home area.
double dp_thirteen(double a1, double a2, double a3, double a4, double a5,
                   double a6, double a7, double a8, double a9, double a10,
                   double a11, double a12, double a13) { return a13; }
double dp_fourteen(double a1, double a2, double a3, double a4, double a5,
                   double a6, double a7, double a8, double a9, double a10,
                   double a11, double a12, double a13,
                   double a14)                        { return a14; }

// Float parameters still consume the positional GPR home slot: `z` is the 9th
// positional parameter, so it is read off the stack even though r3..r10 look
// free.
int    ip_after_floats(float a, float b, float c, float d, float e,
                       float f, float g, float h, int z) { return z; }
