// **W-VSNPRNC — the four axes this lane widened on `guard_chain_shared_tail`,
// in one TU, ordered so a wrong compiler-label charge is LIVE.**
//
// **NO `c2rs-profile:` line, deliberately.** The class is `/O1`-only in the
// PARSER (board #1638), so at the fixture path's default `/Ox` every cell here
// is `NotImplemented` — which is correct and is what the shipped
// `wextdata_guard_chain_shared_tail.cpp` does too. **The grading happens in
// `scripts/gate.sh`'s `/O1` mode lanes**, which put all 323 fixtures through
// `c2rs gap --flags-file` at `/O1`, `/O1 /EHsc`, `/O1 /Oi`, `/O1 /Oi /EHsc` and
// `/O1 /Oi /EHsc /GR`. Declaring `/O1` here would be worse than useless: the
// fixture-profile path (`c2rs diff` honouring a declared profile) grades
// `Port=Mismatch` at `/O1` on a source as simple as
// `int pz(int a,int b){return g(a,b);}` — at master, before this lane — while
// the same source through `c2rs gap` at the same flags is `match`. That defect
// is in the declared-profile seam, not in `/O1` emission, and it is not this
// lane's to fix.
//
// ## What each cell widens
//
// `src/xdk/LIBCMT/vsnprnc.cpp::_vsprintf_s_l` is `p0` almost verbatim. That TU
// does not convert — its other function tail-calls it and the inline fence
// refuses the TU wholesale — so these axes have no workload TU that grades them
// end to end, and this file is where they are graded.
//
// ## THE FUNCTION ORDER IS THE LABEL TEST
//
// A wrong charge on the LAST function of a TU moves nothing after it and the
// cell is inert (w-blockir §6). `p0` and `p1` charge **different** leads — 0 and
// 1 — on the same thirty-eight-word block plan, so the one that charges 0 comes
// FIRST and a wrong charge on it shifts everything below:
//
//     p0   five formals, BYTE store, SUBSCRIPT base, SUNK arms    lead 0
//     p1   three formals, HALFWORD store, DEREF base, INLINE arms lead 1
//     p2   a bare leaf                                            charges 0
//
// ## STRUCTURAL BLIND SPOT
//
// Both guard-chain cells have **three** guards in the same order (`params[2]`,
// `params[0]`, `params[1]`), an **external** callee set, and a function's
// address in argument 0. The file cannot see a rule depending on the guard
// count, the guard order, or a data symbol in slot 0. The arity axis is graded
// here at **5 and 3** — the workload's own and the floor; 4, 6, 7 and the n = 8
// refusal are graded against the real `c2.dll` in `work/w-vsnprnc/GRID-N.md` and
// by `#[test]` against two reference objs.

typedef unsigned int usz;
typedef int (*outfn_t)(void);

int *lasterr_alpha(void);
void report_alpha(void);
int helper5_alpha(outfn_t, char *, usz, const char *, void *, void *);
int outfn_alpha(void);

int *lasterr_beta(void);
void report_beta(void);
int helper3_beta(outfn_t, wchar_t *, usz, usz);
int outfn_beta(void);

int bare_target_alpha(int, int);

// p0 — FIVE formals (one rotate step above the `lis`, where the shipped class's
// six put two), a BYTE store (`stb` — the shipped emitter wrote `sth` here for
// every width its reader let through, which was a live `Port=Mismatch`), a
// SUBSCRIPT store base (`buffer[0]`, not `*buffer`), and the SUNK arm spelling
// (two locals per arm merging in the SOURCE). Charges lead 0.
int p0(char *buffer, usz sizeInBytes, const char *format, void *locale, void *argptr) {
    int result;
    int *err_ptr;
    int err_val;
    if (format == 0 || buffer == 0 || sizeInBytes == 0) {
        err_ptr = lasterr_alpha();
        err_val = 22;
    } else {
        result = helper5_alpha(outfn_alpha, buffer, sizeInBytes, format, locale, argptr);
        if (result < 0) {
            buffer[0] = 0;
        }
        if (result != -2) {
            return result;
        }
        err_ptr = lasterr_alpha();
        err_val = 34;
    }
    *err_ptr = err_val;
    report_alpha();
    return -1;
}

// p1 — the opposite corner on all four axes, and the ARITY FLOOR. At n = 3 the
// hoist takes one of the three moves that would otherwise follow the `lis`, so
// only TWO do: this is the cell that refutes "the `lis` is three steps before
// the last". Charges lead 1, where p0 charges 0.
int p1(wchar_t *buffer, usz sizeInWords, usz count) {
    int result;
    if (count == 0 || buffer == 0 || sizeInWords == 0) {
        *lasterr_beta() = 0x16;
        report_beta();
        return -1;
    }
    result = helper3_beta(outfn_beta, buffer, sizeInWords, count);
    if (result < 0) {
        *buffer = 0;
    }
    if (result != -2) {
        return result;
    }
    *lasterr_beta() = 0x22;
    report_beta();
    return -1;
}

// p2 — a bare leaf, LAST, so p1's charge has somewhere to land.
int p2(int a, int b) {
    return bare_target_alpha(a, b);
}
