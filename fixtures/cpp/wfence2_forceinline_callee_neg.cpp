// w-fence2 NEGATIVE — the `__forceinline` clause. `Port=NotImplemented`, at every mode.
//
// **One field away from `wfence2_kept_local_callee.cpp`**, which is a whole-TU
// byte-exact match: the callee is `__forceinline`. Nothing else differs.
//
// `WB_INLINE_FINDINGS` F4 measured `__forceinline` inlining a **980-byte**
// callee at `/O1` *and* `/O2`: it bypasses every size test there is, so no
// size-keyed decline rule may be applied to such a callee at all. c2 expands
// this one, and a port that kept its `bl` would emit a call c2 does not.
//
// **THE LINKAGE BYTE CANNOT SEE IT** — `extern "C" void f()` and
// `extern "C" __forceinline void f()` both read `05`. What separates them is
// one byte later: the flags byte at `name_nul + 5`, `00` against `20`, set for
// `inline`, for `__forceinline` and for a member defined in-class
// (`work/w-fence2/GRID-K.md` K2). That byte is the whole content of this cell.
//
// `w-inlfence`'s decline **D9** said a `__forceinline` cell *"would grade
// nothing: such a callee is refused by the same clause as any other"*. Under a
// narrowed fence it grades a wrong obj, which is why it exists now.
//
// Board rows #2470-#2478; `docs/rungs/2026-08-09-w-fence2.md`.

typedef unsigned int wf2f_usz;
typedef int (*wf2f_outfn_t)(void);

int *wf2f_lasterr(void);
void wf2f_report(void);
int wf2f_helper(wf2f_outfn_t, char *, wf2f_usz, const char *, void *, void *);
int wf2f_outfn(void);

__forceinline int wf2f_big(char *buffer, wf2f_usz sizeInBytes, const char *format, void *locale,
                    void *argptr) {
    int result;
    int *err_ptr;
    int err_val;
    if (format == 0 || buffer == 0 || sizeInBytes == 0) {
        err_ptr = wf2f_lasterr();
        err_val = 22;
    } else {
        result = wf2f_helper(wf2f_outfn, buffer, sizeInBytes, format, locale, argptr);
        if (result < 0) {
            buffer[0] = 0;
        }
        if (result != -2) {
            return result;
        }
        err_ptr = wf2f_lasterr();
        err_val = 34;
    }
    *err_ptr = err_val;
    wf2f_report();
    return -1;
}

int wf2f_wrap(char *buffer, wf2f_usz sizeInBytes, const char *format, void *argptr) {
    return wf2f_big(buffer, sizeInBytes, format, 0, argptr);
}
