// w-fence2 — THE DECLINE SIDE OF THE INLINE FENCE: an intra-TU call c2 KEEPS.
//
// `w-inlfence` shipped one clause — *a callee this TU also defines, of which
// the port has no model, refuses the function* — and it was right as a safety
// property. But it refuses on *"could c2 have inlined?"* where the question that
// decides an obj is *"did it?"*, and `src/xdk/LIBCMT/vsnprnc.cpp` is the
// measured price: **both its functions byte-exact against real c2 and the TU
// still not matching** (`docs/rungs/2026-08-09-w-vsnprnc.md` §5, isolated four
// ways).
//
// This file is that TU's shape as a fixture, with nothing from the workload in
// it. `wf2_big` is a straight-line integer chain the port lowers to well over
// `c2_core::comdat::INLINE_DECLINE_BYTES`; `wf2_wrap` tail-calls it with an
// argument setup, so `c2_core::splice` declines (S3) and the port really does
// emit the `bl`. c2 keeps it, and this is a whole-TU byte-exact match.
//
// **THE EVIDENCE FOR "c2 KEEPS IT" IS OBJ-MEASURED AND IT IS THE DECLINE SIDE
// ONLY.** `work/w-fence2/GRID-W.md` reads the *reference* obj's own REL24
// targets for every one of 7,552 intra-TU call edges in the 878-TU workload:
// 1,101 kept, 6,451 inlined, and **the largest callee c2 inlines anywhere is 80
// bytes**. 96 B and up is 955 kept against 0 inlined. The bound is 128 — 48
// bytes above the last measured inline, and above both published `/O1`
// first-declined points (`WB_INLINE_FINDINGS` F2's 116, GRID-J's 120).
//
//   P1  `wf2_big`  — the callee. A plain-external, non-`inline` leaf whose
//                    lowered body is over the bound.
//   P2  `wf2_wrap` — the caller. One REL24, against a name THIS TU DEFINES,
//                    which no obj this port emitted had ever carried until this
//                    lane (it read `Port=Mismatch @ offset 12`,
//                    `NumberOfSymbols`, until `coff::writer` learned to resolve
//                    such a relocation against the callee's own defined symbol
//                    instead of minting an undefined external for it).
//
// **`/O1` ONLY, and that is a correctness gate rather than a coverage one.**
// The inline ceilings move with the favour-speed bit — `WB_INLINE_FINDINGS`
// F1/F2 put them at `(212,252]` and `(156,164]` at `/O2` — so 128 is *below*
// the `/O2` external ceiling and the rule would be wrong there. The parser
// refuses the whole TU at any mode but `/O1` (board #1638), which is why this
// fixture declares no profile and grades `NotImplemented` at the default `/Ox`.
//
// The negative half is `wfence2_local_callee_neg.cpp`.
//
// Board rows #2470–#2478; `docs/rungs/2026-08-09-w-fence2.md`.

typedef unsigned int wf2_usz;
typedef int (*wf2_outfn_t)(void);

int *wf2_lasterr(void);
void wf2_report(void);
int wf2_helper(wf2_outfn_t, char *, wf2_usz, const char *, void *, void *);
int wf2_outfn(void);

// P1 — the CALLEE. A plain-external, non-`inline` guard chain whose lowered
// body is 152 bytes, comfortably over `INLINE_DECLINE_BYTES`. This is
// `guard_chain_shared_tail`, the only class on the 878-TU workload the port
// lowers to more than 47 bytes at all (GRID-W's port-side table).
int wf2_big(char *buffer, wf2_usz sizeInBytes, const char *format, void *locale, void *argptr) {
    int result;
    int *err_ptr;
    int err_val;
    if (format == 0 || buffer == 0 || sizeInBytes == 0) {
        err_ptr = wf2_lasterr();
        err_val = 22;
    } else {
        result = wf2_helper(wf2_outfn, buffer, sizeInBytes, format, locale, argptr);
        if (result < 0) {
            buffer[0] = 0;
        }
        if (result != -2) {
            return result;
        }
        err_ptr = wf2_lasterr();
        err_val = 34;
    }
    *err_ptr = err_val;
    wf2_report();
    return -1;
}

// P2 — the CALLER. One REL24 against a name THIS TU DEFINES.
int wf2_wrap(char *buffer, wf2_usz sizeInBytes, const char *format, void *argptr) {
    return wf2_big(buffer, sizeInBytes, format, 0, argptr);
}
