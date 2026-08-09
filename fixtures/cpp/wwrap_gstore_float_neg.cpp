// w-wordwrap `_neg` — a FLOAT object, which is stored out of the other register
// file.
//
//     void T_f(float x) { t_f = x; }
//       lis 11,0
//       stfs 1,0(11)   <== f1, not r3
//       blr
//
// Twelve bytes and the same three-word shape, and the value word names a
// floating-point register. `params` is the GPR mapping; the FP one is
// `IlFunction::fp_arg_sources`, a field this class does not carry. GRID T's
// `86 45 …` and `88 85 …` rows are absent from `STORE_WIDTHS` by name for
// exactly this cell.
//
// `fnbyte-exact` reads **0**.

float t_f;

void SetFloat(float x) { t_f = x; }
