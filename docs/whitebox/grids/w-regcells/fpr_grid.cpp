// fpr_grid.cpp — lane w-regcells (L3 of docs/REGALLOC_BRIEF_2026-08-27.md).
//
// THE FIRST OBJ CELLS FOR THE FPR ALLOCATION ORDER AT 0x10c37f20.
//
// `docs/whitebox/ref/P_REGALLOC.md` §7: "The FPR order at 0x10c37f20 is read
// and never obj-checked — no cell in any grid uses floating point."  Every
// register-order cell this project has ever compiled (wb-live's 10,
// wb-regalloc's 15, w-dagorder's 20) is integer.  These are the FP ones.
//
// Predictions are frozen in work/w-regcells/PREREG.md §1 BEFORE this file was
// compiled.  The read under test:
//
//     0x10c385c4[1] = 0x10c37f20 = { fp0, fp13, fp12, ..., fp1,
//                                    fp31, fp30, ..., fp14 }, 32 entries
//
// Shapes mirror grids/wb-regalloc/regorder_grid.cpp's G-series with `double`
// in place of `int`, so a difference between the two grids is attributable to
// the register CLASS and not to the shape.  Values come from GLOBALS on
// purpose: nothing arrives pre-coloured, so no copy preference biases the
// selector's cost and the LIST ORDER is the only thing left to decide.
//
// Compile:  scripts/gt_capture.sh docs/whitebox/grids/w-regcells/fpr_grid.cpp \
//               /nologo /c /GR /O1 /Oi /EHsc          (mode W, the workload)
//           scripts/gt_capture.sh docs/whitebox/grids/w-regcells/fpr_grid.cpp \
//               /nologo /Ox /GS- /c                   (mode X, the fixtures)
// Read:     scripts/gt_dump.py <obj>

extern "C" {

extern double fpg0, fpg1, fpg2, fpg3, fpg4, fpg5, fpg6, fpg7;
extern double fpg8, fpg9, fpg10, fpg11, fpg12, fpg13, fpg14, fpg15;

// ---- G-series: no arrival register anywhere, so the list order is the only
// ---- thing deciding.  PREREG §1.1 predicts f0; {f0,f13}; {f0,f13,f12};
// ---- {f0,f13,f12,f11} as the values pile up.

double fpc_g1(void) { return fpg0 + 1.0; }
double fpc_g2(void) { return (fpg0 + 1.0) * (fpg1 + 2.0); }
double fpc_g3(void) { return (fpg0 + 1.0) * (fpg1 + 2.0) + fpg2 * 3.0; }
double fpc_g4(void)
{
    return ((fpg0 + 1.0) * (fpg1 + 2.0)) + ((fpg2 + 3.0) * (fpg3 + 4.0));
}

// ---- L3: THE SHARPEST CELL.  Three values live across a call lose every FP
// ---- volatile at once (the kind-0x0b clobber operand, WB_LIVE §6.1), so the
// ---- selector must walk into the list's non-volatile TAIL.  FR0 says that
// ---- tail is f31, f30, f29 — DESCENDING.  Every rival that reads the table
// ---- ascending says f14, f15, f16.  This is the cell that separates them.

extern void fpsink(void);

double fpc_l3(void)
{
    double a = fpg0, b = fpg1, c = fpg2;
    fpsink();
    return (a + b) * c;
}

// ---- P1: pressure.  Sixteen values from globals, combined in a balanced tree
// ---- so they are simultaneously live at the leaves.  FR0 says the used set is
// ---- a PREFIX of the list: volatiles first, and the first non-volatile
// ---- reached is f31.

double fpc_p1(void)
{
    double v0 = fpg0, v1 = fpg1, v2 = fpg2, v3 = fpg3;
    double v4 = fpg4, v5 = fpg5, v6 = fpg6, v7 = fpg7;
    double v8 = fpg8, v9 = fpg9, v10 = fpg10, v11 = fpg11;
    double v12 = fpg12, v13 = fpg13, v14 = fpg14, v15 = fpg15;
    return (((v0 * v1) + (v2 * v3)) + ((v4 * v5) + (v6 * v7)))
         + (((v8 * v9) + (v10 * v11)) + ((v12 * v13) + (v14 * v15)));
}

// ---- P2: the same pressure WITH a call in the middle, so every one of the
// ---- sixteen is live across the clobber and the whole non-volatile tail has
// ---- to be walked.  FR0 predicts f31 down to f16 and NOT f14/f15 first.

double fpc_p2(void)
{
    double v0 = fpg0, v1 = fpg1, v2 = fpg2, v3 = fpg3;
    double v4 = fpg4, v5 = fpg5, v6 = fpg6, v7 = fpg7;
    double v8 = fpg8, v9 = fpg9, v10 = fpg10, v11 = fpg11;
    double v12 = fpg12, v13 = fpg13, v14 = fpg14, v15 = fpg15;
    fpsink();
    return (((v0 * v1) + (v2 * v3)) + ((v4 * v5) + (v6 * v7)))
         + (((v8 * v9) + (v10 * v11)) + ((v12 * v13) + (v14 * v15)));
}

// ---- A1: the preference term for the FP file.  Formals arrive in f1, f2 and
// ---- have a copy preference to them; the loaded global has NO copy relation
// ---- to anything, so PREREG §1.1 says it takes the HEAD of the list (f0) —
// ---- not the next free argument register.  This is the FPR restatement of
// ---- WB_REGALLOC_FINDINGS §7.3's recorded miss, registered in advance so the
// ---- same mistake cannot be made twice.

double fpc_a1(double a, double b) { return a * b + fpg0; }

// ---- A2: four formals, so the arrival registers reach f4, and one loaded
// ---- global.  If the global takes f0 while f5..f13 are free, the preference
// ---- reading and the list head are both confirmed on one cell.

double fpc_a2(double a, double b, double c, double d)
{
    return ((a * b) + (c * d)) * fpg0;
}

// ---- W1: `float` rather than `double`.  The class is the same (nibble 5 ->
// ---- class 1, 0x10b022cc) so the SAME list must be walked.  A different
// ---- register set here would mean the class map is width-sensitive, which
// ---- nothing in the read allows.

extern float fsg0, fsg1, fsg2;
float fpc_w1(void) { return (fsg0 + 1.0f) * (fsg1 + 2.0f) + fsg2; }

}  // extern "C"
