// w-fence2 NEGATIVE — the LINKAGE clause. `Port=NotImplemented`, at every mode.
//
// **One field away from `wfence2_kept_local_callee.cpp`**, which is a whole-TU
// byte-exact match: the callee is `static`. Nothing else differs — same guard
// chain, same 152-byte body, same wrapper, same tail call.
//
// `WB_INLINE_FINDINGS` F1/F2 measure two different size ceilings for the two
// linkage classes at `/O1` — `(300,308]` for a STATIC callee against
// `(100,116]` for an EXTERNAL one — so `INLINE_DECLINE_BYTES` (128), which is
// fitted above the EXTERNAL one, is **three times too small** here. c2 inlines
// this callee, and a port that kept its `bl` would emit a call c2 does not.
//
// The clause is `c2_il::func::gl::plain_external_defined_names`, which reads the
// `.gl` defined record's linkage byte and admits only `05`: `03` is `static` and
// `09` is `__declspec(dllexport)` (`work/w-fence2/GRID-K.md`, fifteen defined
// records, one per linkage form).
//
// **This cell is the reason the clause exists rather than a restatement of it**:
// with the linkage test removed the port emits, and the obj is wrong.
//
// Board rows #2470–#2478; `docs/rungs/2026-08-09-w-fence2.md`.

typedef unsigned int wf2s_usz;
typedef int (*wf2s_outfn_t)(void);

int *wf2s_lasterr(void);
void wf2s_report(void);
int wf2s_helper(wf2s_outfn_t, char *, wf2s_usz, const char *, void *, void *);
int wf2s_outfn(void);

static int wf2s_big(char *buffer, wf2s_usz sizeInBytes, const char *format, void *locale,
                    void *argptr) {
    int result;
    int *err_ptr;
    int err_val;
    if (format == 0 || buffer == 0 || sizeInBytes == 0) {
        err_ptr = wf2s_lasterr();
        err_val = 22;
    } else {
        result = wf2s_helper(wf2s_outfn, buffer, sizeInBytes, format, locale, argptr);
        if (result < 0) {
            buffer[0] = 0;
        }
        if (result != -2) {
            return result;
        }
        err_ptr = wf2s_lasterr();
        err_val = 34;
    }
    *err_ptr = err_val;
    wf2s_report();
    return -1;
}

int wf2s_wrap(char *buffer, wf2s_usz sizeInBytes, const char *format, void *argptr) {
    return wf2s_big(buffer, sizeInBytes, format, 0, argptr);
}
