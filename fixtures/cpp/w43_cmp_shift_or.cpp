// **W43 — `return ((unsigned)(P != 0) << SH) | C;`**, the shift and the OR
// folded into one `rlwimi`. This is `?GetXAllocAttributes@NUISPEECH@@YAKH@Z`
// from `src/xdk/nuispeech/xboxmem.cpp` — the LAST of that TU's four functions,
// and the one that converts it.
//
//     ?GetXAllocAttributes@NUISPEECH@@YAKH@Z        .text COMDAT, 0x18 B, nrel 0
//       addic  r11,r3,-1        the `!= 0` fold, unchanged from W6
//       lis    r10,0x249b       the constant, materialized INTO the gap between
//       subfe  r11,r11,r3         the carry producer and its consumer
//       rlwimi r10,r11,30,0,1   the shift and the OR, one word
//       mr     r3,r10
//       blr
//
// ## Why the class is drawn where it is
//
// c2 has at least three lowerings for this expression and picks between them on
// a property of `C`'s **bit pattern**:
//
//     slwi rT,rS,SH ; oris r3,rT,C>>16     C's low half 0 and SH <= msb(C)
//     slwi rT,rS,SH ; ori  r3,rT,C         C's high half 0
//     lis rT,C>>16 ; rlwimi rT,rS,SH,0,31-SH ; mr r3,rT
//
// and the third is the LONGEST of the three, so it is not an instruction count
// that chooses it. **288 cells** were compiled by real `c2` at the workload's
// own `/O1 /Oi /EHsc /GR` — `C` × `SH ∈ 0..=31` over nine constants
// (`work/w-tu1/p/grid_kc.cpp`) — and the boundary does **not** reduce to any
// clean rule this lane could state: `C = 0x80000000` takes `rlwimi` for SH 1..30
// and something else at 31, and `C = 0x00030000` crosses one column early.
//
// So `c2_il::shift_or_rlwimi` does not claim the boundary. It claims a region
// strictly inside it — `C_low16 == 0 && SH > msb(C)` — where all 288 cells
// agree and none disagrees, and it **refuses** everything else, the two
// anomalous rows included. `w43_cmp_shift_or_neg.cpp` holds the refusals with
// their measured bytes.
//
// ## The register is the INCUMBENT `/O1` rule, not a new one
//
// `subfe` writes r11 at `/O1` and r9 at `/Ox`. That is `docs/CODEGEN_W6_O1.md`'s
// rule — *a temp whose defining instruction makes the last use of the value in
// r11 takes r11 rather than a fresh descending number* — already graded across
// that doc's 108-cell matrix, applied to one more spine. Nothing new is fitted.
//
// `led` at the bottom is the **label-stride control**: a framed call sharing the
// TU, so a W43 body that charged the compiler-label counter the wrong number of
// slots would give `led` the wrong `$M` numbers and this obj would not match.

int gg(int);

// The workload's own function, reduced to its externals.
unsigned long GetXAllocAttributes(int i) {
    return (unsigned int)(i != 0) << 0x1e | 0x249b0000;
}

// The whole admitted region: every `(C, SH)` with `C_low16 == 0` and
// `SH > msb(C)`, over eleven constants — 57 cells.
//
// `0x9abc0000` and `0xffff0000` were in the generator's constant list and
// contribute **zero** cells, which is the point: `SH > msb(C)` with `SH <= 31`
// forces `msb(C) <= 30`, so `C`'s bit 31 is always clear and `lis` never gets a
// negative immediate. The class cannot reach the `addis` sign-flip at all.
unsigned g000(int i) { return ((unsigned)(i != 0) << 31) | 0x40000000u; }
unsigned g001(int i) { return ((unsigned)(i != 0) << 30) | 0x249b0000u; }
unsigned g002(int i) { return ((unsigned)(i != 0) << 31) | 0x249b0000u; }
unsigned g003(int i) { return ((unsigned)(i != 0) << 29) | 0x10000000u; }
unsigned g004(int i) { return ((unsigned)(i != 0) << 30) | 0x10000000u; }
unsigned g005(int i) { return ((unsigned)(i != 0) << 31) | 0x10000000u; }
unsigned g006(int i) { return ((unsigned)(i != 0) << 28) | 0x8000000u; }
unsigned g007(int i) { return ((unsigned)(i != 0) << 29) | 0x8000000u; }
unsigned g008(int i) { return ((unsigned)(i != 0) << 30) | 0x8000000u; }
unsigned g009(int i) { return ((unsigned)(i != 0) << 31) | 0x8000000u; }
unsigned g010(int i) { return ((unsigned)(i != 0) << 18) | 0x30000u; }
unsigned g011(int i) { return ((unsigned)(i != 0) << 19) | 0x30000u; }
unsigned g012(int i) { return ((unsigned)(i != 0) << 20) | 0x30000u; }
unsigned g013(int i) { return ((unsigned)(i != 0) << 21) | 0x30000u; }
unsigned g014(int i) { return ((unsigned)(i != 0) << 22) | 0x30000u; }
unsigned g015(int i) { return ((unsigned)(i != 0) << 23) | 0x30000u; }
unsigned g016(int i) { return ((unsigned)(i != 0) << 24) | 0x30000u; }
unsigned g017(int i) { return ((unsigned)(i != 0) << 25) | 0x30000u; }
unsigned g018(int i) { return ((unsigned)(i != 0) << 26) | 0x30000u; }
unsigned g019(int i) { return ((unsigned)(i != 0) << 27) | 0x30000u; }
unsigned g020(int i) { return ((unsigned)(i != 0) << 28) | 0x30000u; }
unsigned g021(int i) { return ((unsigned)(i != 0) << 29) | 0x30000u; }
unsigned g022(int i) { return ((unsigned)(i != 0) << 30) | 0x30000u; }
unsigned g023(int i) { return ((unsigned)(i != 0) << 31) | 0x30000u; }
unsigned g024(int i) { return ((unsigned)(i != 0) << 17) | 0x10000u; }
unsigned g025(int i) { return ((unsigned)(i != 0) << 18) | 0x10000u; }
unsigned g026(int i) { return ((unsigned)(i != 0) << 19) | 0x10000u; }
unsigned g027(int i) { return ((unsigned)(i != 0) << 20) | 0x10000u; }
unsigned g028(int i) { return ((unsigned)(i != 0) << 21) | 0x10000u; }
unsigned g029(int i) { return ((unsigned)(i != 0) << 22) | 0x10000u; }
unsigned g030(int i) { return ((unsigned)(i != 0) << 23) | 0x10000u; }
unsigned g031(int i) { return ((unsigned)(i != 0) << 24) | 0x10000u; }
unsigned g032(int i) { return ((unsigned)(i != 0) << 25) | 0x10000u; }
unsigned g033(int i) { return ((unsigned)(i != 0) << 26) | 0x10000u; }
unsigned g034(int i) { return ((unsigned)(i != 0) << 27) | 0x10000u; }
unsigned g035(int i) { return ((unsigned)(i != 0) << 28) | 0x10000u; }
unsigned g036(int i) { return ((unsigned)(i != 0) << 29) | 0x10000u; }
unsigned g037(int i) { return ((unsigned)(i != 0) << 30) | 0x10000u; }
unsigned g038(int i) { return ((unsigned)(i != 0) << 31) | 0x10000u; }
unsigned g039(int i) { return ((unsigned)(i != 0) << 31) | 0x7fff0000u; }
unsigned g040(int i) { return ((unsigned)(i != 0) << 18) | 0x20000u; }
unsigned g041(int i) { return ((unsigned)(i != 0) << 19) | 0x20000u; }
unsigned g042(int i) { return ((unsigned)(i != 0) << 20) | 0x20000u; }
unsigned g043(int i) { return ((unsigned)(i != 0) << 21) | 0x20000u; }
unsigned g044(int i) { return ((unsigned)(i != 0) << 22) | 0x20000u; }
unsigned g045(int i) { return ((unsigned)(i != 0) << 23) | 0x20000u; }
unsigned g046(int i) { return ((unsigned)(i != 0) << 24) | 0x20000u; }
unsigned g047(int i) { return ((unsigned)(i != 0) << 25) | 0x20000u; }
unsigned g048(int i) { return ((unsigned)(i != 0) << 26) | 0x20000u; }
unsigned g049(int i) { return ((unsigned)(i != 0) << 27) | 0x20000u; }
unsigned g050(int i) { return ((unsigned)(i != 0) << 28) | 0x20000u; }
unsigned g051(int i) { return ((unsigned)(i != 0) << 29) | 0x20000u; }
unsigned g052(int i) { return ((unsigned)(i != 0) << 30) | 0x20000u; }
unsigned g053(int i) { return ((unsigned)(i != 0) << 31) | 0x20000u; }
unsigned g054(int i) { return ((unsigned)(i != 0) << 29) | 0x12340000u; }
unsigned g055(int i) { return ((unsigned)(i != 0) << 30) | 0x12340000u; }
unsigned g056(int i) { return ((unsigned)(i != 0) << 31) | 0x12340000u; }
// The label-stride control.
int led(int a) { return gg(a) + 1; }
