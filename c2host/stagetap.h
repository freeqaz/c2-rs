/* stagetap — observe real c2.dll's INTERMEDIATE per-function state.
 *
 * See c2host/README.md § "The stage tap" for the whole argument. In one
 * paragraph: the sole judge of this port is a byte compare of the finished
 * obj, which says "differs" and nothing about WHERE. This installs
 * CALL-SITE detours at c2's own per-function phase boundaries so a
 * divergence can be localized to a pass instead of costing a whole-object
 * byte archaeology session.
 *
 * Three invariants this header's contract exists to hold:
 *
 *   INERTNESS   — with C2RS_STAGE_TAPS unset, tap_arm() is not called, not
 *                 one byte of c2.dll is written, and c2host is exactly what
 *                 it is without this file.
 *   FAIL-CLOSED — tap_arm() refuses to write a site unless the bytes there
 *                 are still `e8 <rel32>` AND the decoded original target
 *                 equals the recorded target plus the measured load slide.
 *                 A refusal prints and leaves the image untouched. This is
 *                 what makes the slide HANDLED rather than assumed, and it
 *                 is the only defence against a different c2.dll.
 *   TRANSPARENCY— the thunk tail-JUMPS to the real target, so the callee's
 *                 register and stack state at entry is bit-identical to the
 *                 unpatched path and the callee returns straight to c2's own
 *                 caller. There is no wrapper frame to get wrong.
 *
 * NOTHING here is a correctness gate. The snapshot never gates an emit and
 * never enters a refusal predicate; the obj byte compare against real c2.dll
 * remains the sole judge.
 */
#ifndef C2RS_STAGETAP_H
#define C2RS_STAGETAP_H

#include <windows.h>

/* Arm the sites named by the environment variable C2RS_STAGE_TAPS, a
 * comma-separated list of site names or the word "all".
 *
 * MEASURED PLAN DEFECT — READ THIS BEFORE "SIMPLIFYING" THE SIGNATURE.
 * The design handed to this lane said "the HMODULE LoadLibraryA returns IS
 * the load base, so slide = (uintptr_t)h - 0x10b00000 is computable and never
 * assumed". That is true on Windows and **FALSE UNDER WIBO**, which returned
 * `HMODULE 0x00000018` for this c2.dll on the first armed run — a small
 * opaque token, not a base. Every site then failed the fail-closed opcode
 * check (`slide=ef500018`, opcode 00) and nothing was patched, which is the
 * invariant doing its job on its first outing.
 *
 * So the slide is derived from a KNOWN EXPORT instead: `invoke_fn` is the
 * pointer GetProcAddress returned for `_InvokeCompilerPass@12`, whose static
 * VA is 0x10bebffd in the image the whitebox record is written against, and
 * `slide = invoke_fn - 0x10bebffd`. That is a measurement on every run, on
 * exactly the same footing the HMODULE was supposed to be.
 *
 * Prints one `[stagetap]` line per site to stderr (armed or refused) plus a
 * `slide=` line carrying BOTH derivations (the export delta and, when wibo
 * answers, VirtualQuery's AllocationBase) so the number is never single-
 * sourced. Returns the number of sites armed; 0 with nothing requested is the
 * inert path.
 */
int tap_arm(HMODULE h, void *invoke_fn);

/* Write the accumulated snapshot to stderr, one `[stagetap]` line each.
 * Called from c2host AFTER InvokeCompilerPass returns, so no I/O ever
 * happens inside a c2 call frame (this removes the mingw-CRT-reentrancy
 * question entirely rather than measuring it). No-op when nothing is armed.
 */
void tap_report(void);

#endif /* C2RS_STAGETAP_H */
