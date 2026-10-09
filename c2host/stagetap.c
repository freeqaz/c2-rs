/* stagetap — see stagetap.h for the contract and the three invariants.
 *
 * ---------------------------------------------------------------------------
 * WHY A CALL-SITE DETOUR AND NOT AN ENTRY DETOUR
 * ---------------------------------------------------------------------------
 * Every boundary we want is already a 5-byte `e8 rel32` direct call, and the
 * exact address of each is in the flat export. Patching the CALL SITE rather
 * than the callee's prologue means:
 *   - no instruction-length decoder (the site is always exactly 5 bytes),
 *   - no prologue save/restore and no unpatch/repatch window,
 *   - the callee is never touched, so any other caller of the same function
 *     is unaffected (0x10be6382 has two callers; we distinguish them BY SITE,
 *     which an entry detour structurally cannot do).
 *
 * ---------------------------------------------------------------------------
 * ONE THUNK, NOT ONE PER SITE
 * ---------------------------------------------------------------------------
 * PLAN DEFECT (recorded, and the plan was written by an agent that could not
 * compile anything): the design handed to this lane specified per-site
 * `__declspec(naked)` thunks. **GCC does not implement the `naked` attribute
 * on i686** — it is available only on ARM/AVR/MSP430/RL78 and friends — so
 * that design does not build with the one compiler this repo's host stubs use
 * (`i686-w64-mingw32-gcc`, pinned in Toolchain::ensure_c2host).
 *
 * The replacement is smaller than the original, not a workaround: a SINGLE
 * top-level `__asm__` thunk that recovers WHICH site called it from the
 * return address the `call` instruction already pushed. `retaddr == site + 5`
 * identifies the site exactly, so the per-site code generation disappears and
 * the number of arm-able sites stops being a compile-time constant. (This
 * matters for the 57 phase-beacon sites in 0x10b7d85e..0x10b7e300, which all
 * share one callee and are only distinguishable by site.)
 *
 * ---------------------------------------------------------------------------
 * WHY THE PAYLOAD IS BUFFERED AND DUMPED AFTER THE PASS RETURNS
 * ---------------------------------------------------------------------------
 * The obvious design streams each event to a file as it happens. That would
 * put a mingw-static-CRT `fwrite` inside a c2 call frame under wibo, whose
 * safety is an open question ("c2's CRT is msvcr100, separate, so this should
 * be fine" is not a measurement). Buffering into a static arena and flushing
 * from c2host's `main` AFTER InvokeCompilerPass returns removes the question
 * instead of answering it, and costs nothing: the snapshot is bounded by
 * construction anyway.
 */

#include "stagetap.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* The image the whole whitebox record is written against:
 * sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258,
 * PE ImageBase 0x10b00000, DllCharacteristics 0x100 (DYNAMIC_BASE CLEAR).
 * The slide is still computed from the HMODULE on every run and printed —
 * a .reloc directory is present, so a nonzero slide is reachable and must be
 * handled rather than assumed away. */
#define C2_PREFERRED_BASE 0x10b00000u

/* Static VA of the export `_InvokeCompilerPass@12` in that same image
 * (functions.tsv:3948, symbols.tsv:5343 — also Ordinal_3). The slide is
 * derived from this and NOT from the HMODULE; see stagetap.h for the measured
 * reason (wibo hands back HMODULE 0x18). */
#define C2_INVOKE_VA      0x10bebffdu

#define MAX_SITES 128

/* A call site to detour.
 *
 * `target_va` is not decoration: it is the fail-closed check. tap_arm decodes
 * the rel32 actually present at `site_va + slide` and refuses the site unless
 * it resolves to `target_va + slide`. A different c2.dll, a relocated image
 * handled wrongly, or an address typo all land on that check rather than on a
 * patched guess. */
typedef struct {
    const char  *name;
    unsigned int site_va;
    unsigned int target_va;
    const char  *what;
} TapSite;

/* ---------------------------------------------------------------------------
 * THE SITE TABLE
 *
 * Every row was verified byte-for-byte against the flat export
 * ~/ghidra-projects/export/c2/objdump_intel.asm before it was written here,
 * and each has a docs/whitebox/DISCLOSURE.md row. Confidence [R] — read from
 * disassembly, not obj-checked.
 *
 * The six per-function phase sites come from P_DAG.md §1's table; the region
 * finder is the SOLE call site of 0x10be5d4b. The three in-band scheduler
 * calls sit behind `cmp ds:0x10c2e2fc,edi` with edi==0, i.e. the optimizer-on
 * flag, which is why the /Od-vs-/O1 discrimination control is a property of
 * the code rather than a hope.
 * ------------------------------------------------------------------------- */
static const TapSite g_sites[] = {
    { "sched1",   0x10b7dc9fu, 0x10be6382u, "scheduler run 1 (mode 1), P_DAG.md S1" },
    { "globregs", 0x10b7dcb7u, 0x10b57633u, "global register assignment" },
    { "sched2",   0x10b7dcdeu, 0x10be6382u, "scheduler run 2 (mode 1)" },
    { "color",    0x10b7dcf6u, 0x10b31c9au, "THE REGISTER ALLOCATOR (color.c band)" },
    { "sched3",   0x10b7dd1du, 0x10be6382u, "scheduler run 3 (mode 1)" },
    { "sched0",   0x10b7e00cu, 0x10be6382u, "scheduler run 4 (mode 0), LAST" },
    { "region",   0x10be643eu, 0x10be5d4bu, "the region finder — sole call site" },
    /* THE OBSERVATION POINT AFTER RUN 4. `0x10b7df57` is the mode-0 (final)
     * schedule and the `sched0` site `0x10b7e00c` lives inside it
     * (0x10b7df57 + 219 = 0x10b7e032, so the site is in range). In the
     * per-function orchestrator `0x10b7e6af` the call at `0x10b7e701` is the
     * FIRST one after `0x10b7df57` returns:
     *
     *   10b7e6f8: 8b ce                mov ecx,esi        ; esi = function rec
     *   10b7e6fa: e8 58 f8 ff ff       call 0x10b7df57    ; run 4 (sched0)
     *   10b7e6ff: 8b ce                mov ecx,esi
     *   10b7e701: e8 2c f9 ff ff       call 0x10b7e032    ; <-- HERE
     *
     * so ecx is the function record and the final schedule has already been
     * written. The region tap structurally cannot see this: it fires at
     * region-finder ENTRY, run 4 has no successor run, and every `sched0`
     * block is therefore run 4's INPUT. */
    { "after0",   0x10b7e701u, 0x10b7e032u, "after the FINAL schedule, function record in ecx" },
    /* THE CANDIDATE SITES (lane w24-c2rs, DISCLOSURE `W-STAGETAP-7`). All five
     * live in the allocator driver 0x10b31c9a's worklist loop
     * (P_REGALLOC.md §1), and unlike every row above they fire once per
     * CANDIDATE, not once per function — which is why the Rust side keeps them
     * in a SEPARATE list (`c2_reference::stage::CAND_SITES`) and the
     * per-function hit-count tests never arm them.
     *
     *   10b31ed5: 8b cf        mov ecx,edi          ; edi = popped candidate
     *   10b31ed7: e8 26 f5 ff ff call 0x10b31402    ; benefit <= 0: not coloured
     *   10b31efe: 8b cf        mov ecx,edi
     *   10b31f00: e8 28 99 ff ff call 0x10b2b82d    ; RE-INSERT by priority
     *   10b31f16: 8b cf        mov ecx,edi
     *   10b31f18: e8 27 f6 ff ff call 0x10b31544    ; allowed set EMPTY: spill
     *   10b31f53: 8b cf        mov ecx,edi          ; edx = ebx = class
     *   10b31f58: e8 9b c8 ff ff call 0x10b2e7f8    ; THE SELECTOR
     *   10b31f68: 57           push edi             ; cand is [esp+4] at entry
     *   10b31f69: e8 5b fb ff ff call 0x10b31ac9    ; after the selector wrote
     *                                               ; cand+0x10 (0x10b2e92e)
     */
    { "cnocol",   0x10b31ed7u, 0x10b31402u, "candidate popped with benefit <= 0 (not coloured)" },
    { "creq",     0x10b31f00u, 0x10b2b82du, "candidate re-inserted into the worklist" },
    { "cspill",   0x10b31f18u, 0x10b31544u, "candidate popped with an EMPTY allowed set" },
    { "csel",     0x10b31f58u, 0x10b2e7f8u, "THE SELECTOR, candidate in ecx, class in edx" },
    { "casg",     0x10b31f69u, 0x10b31ac9u, "after the selector, candidate at [esp+4]" },
    /* THE REGASG SITES (lane w25-regasg, DISCLOSURE `W-STAGETAP-8`) — the
     * second allocator, `regasg.c`. Like the candidate sites they fire per
     * TUPLE or per TEMP, not per function, so the Rust side keeps them in a
     * separate list (`c2_reference::stage::RASG_SITES`).
     *
     *   10bc648f: 8b ce          mov ecx,esi          ; esi = function record
     *   10bc6491: e8 20 fe ff ff call 0x10bc62b6      ; THE REGASG DRIVER
     *   10c1b867: e8 4a aa fa ff call 0x10bc62b6      ; its re-run (rare)
     *   10bc6416: 8b ce          mov ecx,esi          ; esi = real tuple
     *   10bc6418: e8 d0 e9 ff ff call 0x10bc4ded      ; free dying sources
     *   10bc624d: 8b ce          mov ecx,esi          ; esi = operand, edx = tuple
     *   10bc624f: e8 81 f6 ff ff call 0x10bc58d5      ; SELECTOR: a fresh def
     *   10bc6145: e8 8b f7 ff ff call 0x10bc58d5      ; SELECTOR: re-home
     *   10bc5122: e8 ae 07 00 00 call 0x10bc58d5      ; SELECTOR: reload
     *   10bc6021: e8 88 ee ff ff call 0x10bc4eae      ; no register: spill
     *   10bc6026: 8b 4d e8       mov ecx,[ebp-0x18]   ; the register number
     *   10bc6029: 8b 55 f4       mov edx,[ebp-0xc]    ; the symbol
     *   10bc602c: e8 70 eb ff ff call 0x10bc4ba1      ; THE COMMIT
     */
    /* THE CANDIDATE-MINT SITES (lane w25-regasg, DISCLOSURE `W-STAGETAP-9`):
     * every direct call of the candidate constructor FUN_10b54d32 (ecx = the
     * symbol the candidate is minted from), so a `CAND` line can say WHICH
     * pass minted its id. The constructor stamps `id = DAT_10c400d4++` on a
     * fresh alloc and keeps the old id on a free-list reuse (0x10b54d48), so
     * the id is PREDICTED at the call exactly as the constructor will pick it.
     *
     *   10b56837: 8b ce          mov ecx,esi          ; a NEW kind-0xd symbol
     *   10b56839: e8 f4 e4 ff ff call 0x10b54d32      ; FUN_10b5673e, reached
     *            from FUN_10bfd665's FORWARD tuple walk at the head of
     *            globregs (0x10b57633): constants and addresses an operand
     *            needs in a register, deduplicated by value (DAT_10c3ffd0)
     *   10b55e64: 8b cf          mov ecx,edi
     *   10b55e66: e8 c7 ee ff ff call 0x10b54d32      ; FUN_10b55dbe: the
     *            symbol-arena x version mint (P_GLOBREGS.md §6), AFTER it
     *   10bfd989: 8b ce          mov ecx,esi
     *   10bfd98b: e8 a2 73 f5 ff call 0x10b54d32      ; code.c band
     *   10c2073a: 8b ce          mov ecx,esi
     *   10c2073c: e8 f1 45 f3 ff call 0x10b54d32      ; FUN_10c205fd, under
     *            the allocator's liveness driver FUN_10c20f79
     *   10b2e151 / 10b2e655 / 10b2e732                ; split children
     */
    { "cmk0d",    0x10b56839u, 0x10b54d32u, "candidate mint: globregs constant/address pre-pass" },
    { "cmarena",  0x10b55e66u, 0x10b54d32u, "candidate mint: symbol arena x version" },
    { "cmcode",   0x10bfd98bu, 0x10b54d32u, "candidate mint: code.c band" },
    { "cmlive",   0x10c2073cu, 0x10b54d32u, "candidate mint: under the liveness driver" },
    { "cmsplit1", 0x10b2e151u, 0x10b54d32u, "candidate mint: split child (FUN_10b2dfe2)" },
    { "cmsplit2", 0x10b2e655u, 0x10b54d32u, "candidate mint: split child (FUN_10b2e4ae)" },
    { "cmsplit3", 0x10b2e732u, 0x10b54d32u, "candidate mint: split child (FUN_10b2e4ae)" },
    /* Three call sites of the candidate DESTRUCTOR FUN_10b2efd6 (ecx = the
     * candidate; it memsets the record, keeps `+0x1c`, and pushes it on the
     * free list 0x10c2e3e0 that the next mint pops LIFO):
     *   10b55e0e: 8b ce mov ecx,esi / 10b55e10: call  ; inside the arena mint
     *   10bfd62d: 8b ce mov ecx,esi / 10bfd62f: call  ; FUN_10bfcf7c: a
     *            candidate whose benefit `+0x40` is negative is dropped
     *   10b313b0: 8b cf mov ecx,edi / 10b313b2: call  ; FUN_10b30ff6 (COLOR)
     */
    { "cfarena",  0x10b55e10u, 0x10b2efd6u, "candidate free: inside the arena mint" },
    { "cfben",    0x10bfd62fu, 0x10b2efd6u, "candidate free: negative benefit (FUN_10bfcf7c)" },
    { "cfcolor",  0x10b313b2u, 0x10b2efd6u, "candidate free: FUN_10b30ff6 (COLOR)" },
    { "rafn",     0x10bc6491u, 0x10bc62b6u, "regasg driver, function record in ecx" },
    { "rarerun",  0x10c1b867u, 0x10bc62b6u, "regasg driver re-run (FUN_10c1b808)" },
    { "ratup",    0x10bc6418u, 0x10bc4dedu, "regasg per real tuple, tuple in ecx" },
    { "radef",    0x10bc624fu, 0x10bc58d5u, "regasg selector: a def with no register" },
    { "raevict",  0x10bc6145u, 0x10bc58d5u, "regasg selector: re-home after an evict" },
    { "rareload", 0x10bc5122u, 0x10bc58d5u, "regasg selector: spill reload" },
    { "raspill",  0x10bc6021u, 0x10bc4eaeu, "regasg selector found no register: spill" },
    { "racom",    0x10bc602cu, 0x10bc4ba1u, "regasg commit, register in ecx, symbol in edx" },
    /* THE BENEFIT SITES (lane w25-regasg, DISCLOSURE `W-STAGETAP-10`): the
     * terms of `cand+0x40`, the benefit globregs' negative-benefit pass
     * FUN_10bfcf7c tests (`cmp DWORD PTR [eax+0x40],0x0` / `jge` at
     * 0x10bfcfdd, 0x10bfd0a1). Before COLOR the field has exactly two
     * writers, both in the step-4 walk FUN_10b55eae (all other pre-COLOR
     * writers of `[r+0x40]` are BLOCK records):
     *   10b561fd: ff 48 40 dec DWORD PTR [eax+0x40]  ; a DEF in a tuple with
     *            opcode 0x2b7 (0x10b55fe9), symbol versioned
     *   10b563c3: ff 47 40 inc DWORD PTR [edi+0x40]  ; a USE whose symbol is
     *            versioned (`[[op+0x18]+0x34]+0x18 != 0`, 0x10b563b4..bb)
     * plus 0x10b560df `mov DWORD PTR [eax+0x40],0x10000000` (a pinned value).
     * The two calls immediately before each are the observation points;
     * the candidate is `op+0x1c`, stored just before each call (0x10b55fa9,
     * 0x10b56398).
     *   10b55fb2: 8b d3 mov edx,ebx / 10b55fb7: e8 13 60 0a 00 call 0x10bfbfcf
     *            ; ecx = tuple ([ebp-0x4]), edx = the DEF operand
     *   10b563a7: 8b d3 mov edx,ebx / 10b563ac: e8 4e 5c 0a 00 call 0x10bfbfff
     *            ; ecx = tuple, edx = the USE operand
     *   10b57800: 8b ce mov ecx,esi / 10b57802: e8 75 57 0a 00 call 0x10bfcf7c
     *            ; THE NEGATIVE-BENEFIT PASS, ecx = function record: the
     *            benefit of every candidate is final here
     */
    { "cbdef",    0x10b55fb7u, 0x10bfbfcfu, "benefit term: a def in the step-4 walk" },
    { "cbuse",    0x10b563acu, 0x10bfbfffu, "benefit term: a use in the step-4 walk" },
    { "cbpre",    0x10b57802u, 0x10bfcf7cu, "before the negative-benefit pass" },
    /* Inside that pass, 0x10bfd0fd..0x10bfd3fc is a SECOND free source with
     * no benefit test at all: a candidate of an UNVERSIONED symbol
     * (`[[sym+0x34]+0x18] == 0`) whose every reference lies in one block and
     * which has exactly one qualifying def is rewritten onto a fresh temp
     * (FUN_10bd42ff for one use, FUN_10bd432e otherwise) and its id added to
     * the free set the loop at 0x10bfd636 drains in ASCENDING id order.
     *   10bfd323: 8b 56 2c mov edx,[esi+0x2c]   ; the def operand (-> cand)
     *   10bfd328: 53       push ebx             ; the NEW temp symbol
     *   10bfd32b: e8 d9 a7 fd ff call 0x10bd7b09 ; ecx = the def tuple
     */
    { "cbdem",    0x10bfd32bu, 0x10bd7b09u, "demotion: a one-block local onto a fresh temp" },
};
#define N_SITES ((int)(sizeof(g_sites) / sizeof(g_sites[0])))

/* Armed state, indexed in lockstep with g_sites. */
static int          g_armed[MAX_SITES];
static unsigned int g_retaddr[MAX_SITES];   /* site_va + slide + 5 */
static unsigned int g_realtgt[MAX_SITES];   /* target_va + slide   */
static unsigned int g_hits[MAX_SITES];
static int          g_any_armed = 0;
static unsigned int g_slide = 0;
static int          g_slide_known = 0;
/* Payload on/off. Counts-only is the default; C2RS_STAGE_PAYLOAD=1 turns on
 * the bounded tuple walk. Kept separate from arming so the CHEAP neutrality
 * sweep and the EXPENSIVE content run are the same mechanism at two settings,
 * and so G1 can be re-run at the full table WITH the payload rather than
 * extrapolated from the counts-only run. */
static int          g_payload = 0;

/* PHASE TRACKING — how a pre/post-COLOR pair is obtained WITHOUT knowing the
 * function-record -> tuple-list-head offset.
 *
 * The region tap fires from inside the scheduler driver 0x10be6382, which is
 * itself reached from sched1/sched2/sched3/sched0. c2's per-function order is
 *   sched1 -> globregs -> sched2 -> COLOR -> sched3   (P_DAG.md §1)
 * so a region block tagged `sched2` is the tuple list immediately BEFORE the
 * register allocator ran, and one tagged `sched3` is the same list
 * immediately AFTER. The pre/post-COLOR pair is therefore a by-product of the
 * region tap; no second mechanism is needed and no struct offset is guessed.
 *
 * g_fn counts sched1 entries, which is one per function. */
/* RAW WINDOW — how the "which fields does COLOR write?" question is ANSWERED
 * rather than guessed. C2RS_STAGE_RAW=<n> dumps the first n bytes of every
 * tuple as hex beside the decoded row; diffing the sched2 and sched3 dumps
 * names the offsets the register allocator writes. Off by default and NEVER
 * part of the canonical stream: a raw window can contain pointers, which would
 * make a digest stable only because the allocator happened to be. */
static unsigned int g_raw = 0;

/* C2RS_STAGE_OPS=1 — walk each tuple's two OPERAND lists and, from each
 * operand, the symbol/candidate record. This is where the register allocator's
 * output lives; the tuple record itself is byte-identical across COLOR
 * (`docs/rungs/2026-08-20-stageoracle.md` §3). Off by default: it is three
 * levels of foreign pointer deeper than the tuple walk and it triples the
 * payload. */
static int          g_ops = 0;

/* C2RS_STAGE_FUNCWALK=1 — at the per-function sites, walk the WHOLE function
 * from the function record in ecx (blocks, then tuples), instead of relying on
 * the region tap. The region walk runs to the end of the LIST, so region k
 * re-reads regions k..n; a function walk visits each tuple exactly once and,
 * unlike the region tap, has an observation point at the `after0` site. */
static int          g_walkfn = 0;

/* C2RS_STAGE_CANDS=1 — the register-allocation candidate payload at the
 * five candidate sites (lane w24-c2rs). Off by default; independent of
 * C2RS_STAGE_PAYLOAD's tuple walk except that the arena is only flushed when
 * the payload is on, so the Rust seam sets both. */
static int          g_cands = 0;
static unsigned int g_cfn = 0;          /* COLOR entries = functions coloured */
#define CSYM_MAX 1024u
static unsigned int g_csym[CSYM_MAX];   /* per-function symbol-pointer ordinals */
static unsigned int g_ncsym = 0;

/* C2RS_STAGE_FNFILTER=<substr> — emit the CAND/RASG payload only for
 * functions whose raw name contains it. Payload only: the obj is unaffected
 * either way, and the per-site hit counts still count every function. */
static char         g_fnfilter[256];
static int          g_show = 1;

static const char  *g_phase = "none";
static unsigned int g_fn = 0;

/* Written by tap_enter, read by the thunk's final indirect jump. Safe with a
 * single global because the window between the write and the jump contains no
 * c2 code at all — the thunk cannot nest inside itself. c2 is single-threaded
 * here (one InvokeCompilerPass on the calling thread). */
unsigned int g_tap_jmp = 0;

/* ---------------------------------------------------------------------------
 * THE THUNK
 *
 * Entry state is whatever c2's `call` left: [esp] = retaddr = site+5, and the
 * callee's own arguments live in ecx/edx (these are __fastcall-shaped sites;
 * none of them passes a stack argument). We must return to c2 with every
 * register and every flag bit as the unpatched path would have had them.
 *
 *   pushfl / pushal          save flags and all eight GPRs
 *   ebx = esp                pushal saved the real ebx, so ebx is scratch now
 *   and esp, -16 / sub 16    GCC on i686 assumes a 16-byte-aligned stack at a
 *                            call; c2's esp is arbitrary, so realign. With 16
 *                            bytes of outgoing args, esp is 0 mod 16 at the
 *                            `call`, i.e. 12 mod 16 at tap_enter's entry —
 *                            exactly the SysV/mingw convention.
 *   4 cdecl args             retaddr, ecx, edx, and the callee-visible esp
 *   call tap_enter           returns the REAL target to jump to
 *   store to g_tap_jmp       written while eax is still live
 *   esp = ebx / popal/popfl  exact restore
 *   jmp *g_tap_jmp           TAIL jump: the callee returns to c2's own caller
 *
 * pushal's stack layout, low to high: edi ebp... (actually edi +0, esi +4,
 * ebp +8, esp +12, ebx +16, edx +20, ecx +24, eax +28), then flags at +32 and
 * the return address at +36.
 * ------------------------------------------------------------------------- */
__asm__(
    ".text\n"
    ".p2align 4\n"
    ".globl _c2rs_stage_thunk\n"
    "_c2rs_stage_thunk:\n"
    "    pushfl\n"
    "    pushal\n"
    "    movl  %esp, %ebx\n"
    "    andl  $-16, %esp\n"
    "    subl  $16, %esp\n"
    "    movl  36(%ebx), %eax\n"   /* retaddr  */
    "    movl  %eax, 0(%esp)\n"
    "    movl  24(%ebx), %eax\n"   /* saved ecx */
    "    movl  %eax, 4(%esp)\n"
    "    movl  20(%ebx), %eax\n"   /* saved edx */
    "    movl  %eax, 8(%esp)\n"
    "    leal  36(%ebx), %eax\n"   /* esp as the callee will see it */
    "    movl  %eax, 12(%esp)\n"
    "    call  _tap_enter\n"
    "    movl  %eax, _g_tap_jmp\n"
    "    movl  %ebx, %esp\n"
    "    popal\n"
    "    popfl\n"
    "    jmp   *_g_tap_jmp\n"
);

extern void c2rs_stage_thunk(void);

/* ---------------------------------------------------------------------------
 * THE PAYLOAD ARENA
 *
 * Appended to from inside a c2 call frame, so it uses NO CRT function at all
 * (not even sprintf, whose mingw implementation is not obviously safe to call
 * reentrantly under wibo). Hex is hand-rolled. Flushed by tap_report() after
 * InvokeCompilerPass has returned.
 *
 * Bounded and FAIL-LOUD: when the arena fills, a REFUSE line is emitted and
 * the payload stops. It must never degrade into a silently short — or empty —
 * block: "deterministic and vacuous" passes every other criterion in this
 * lane trivially, and absence-read-as-success is this project's own signature
 * defect (twelve recorded instances).
 *
 * FIX-ROUND DEFECT (review finding, 2026-08-20): the first version of this
 * arena could NEVER emit its own `REFUSE region arena-full` line. `ap` drops
 * silently once the arena is full, so the very call meant to announce the
 * truncation was itself dropped — the fail-loud path was unreachable, and a
 * truncated payload would have looked exactly like a complete one. The fix is
 * a RESERVE at the tail that only `ap_reserved` may write into, so the refusal
 * always has room. Mutation-demonstrated at ARENA_BYTES 8192 (rung §9).
 * ------------------------------------------------------------------------- */
#define ARENA_BYTES   (4u * 1024u * 1024u)
/* Room for the refusal line and the trailing newline, and nothing else. */
#define ARENA_RESERVE 64u
static char         g_arena[ARENA_BYTES];
static unsigned int g_arena_len = 0;
static int          g_arena_full = 0;

static void ap(const char *s)
{
    while (*s) {
        if (g_arena_len >= ARENA_BYTES - 1u - ARENA_RESERVE) { g_arena_full = 1; return; }
        g_arena[g_arena_len++] = *s++;
    }
}

/* The ONLY writer allowed into the reserve. Used for the arena-full refusal
 * itself: an announcement that cannot be written is not an announcement. */
static void ap_reserved(const char *s)
{
    while (*s) {
        if (g_arena_len >= ARENA_BYTES - 1u) return;
        g_arena[g_arena_len++] = *s++;
    }
}

static void ap_hex(unsigned int v, int digits)
{
    static const char H[] = "0123456789abcdef";
    char buf[9];
    int i;
    if (digits > 8) digits = 8;
    for (i = digits - 1; i >= 0; i--) { buf[i] = H[v & 0xf]; v >>= 4; }
    buf[digits] = 0;
    ap(buf);
}

static void ap_dec(unsigned int v)
{
    char buf[12];
    int i = 11;
    buf[i] = 0;
    if (v == 0) { ap("0"); return; }
    while (v && i > 0) { buf[--i] = (char)('0' + (v % 10u)); v /= 10u; }
    ap(&buf[i]);
}

/* Is `p` plausibly a readable heap pointer in this 32-bit process?
 *
 * A foreign linked list under a foreign allocator, reached from a
 * [R]-confidence field reading, is not trusted. This is a cheap structural
 * filter, not a guarantee: 4-byte aligned, inside the user address range, and
 * (for a walk) within a bounded span of where the walk started. */
static int plausible(unsigned int p)
{
    if (p == 0) return 0;
    if (p & 3u) return 0;
    if (p < 0x00010000u) return 0;
    if (p >= 0x7ff00000u) return 0;
    return 1;
}

/* The same filter for a pointer to BYTES.
 *
 * `plausible` requires 4-byte alignment because every pointer it guards is a
 * record pointer. A `char *` has no such guarantee, and reusing `plausible`
 * for one would refuse three names in four for a reason that has nothing to do
 * with whether the address is readable — a fence that refuses the healthy case
 * is worse than no fence, because its refusals stop being informative. */
static int plausible_bytes(unsigned int p)
{
    if (p == 0) return 0;
    if (p < 0x00010000u) return 0;
    if (p >= 0x7ff00000u) return 0;
    return 1;
}

/* Walk one scheduling region's tuple list and append it to the arena.
 *
 * `t` is the pointer c2 passed in ecx at 0x10be643e, which the region finder
 * 0x10be5d4b immediately dereferences as a tuple: it reads [ecx+0x4] (opcode,
 * compared to 0x30f), walks [ecx] (next) and reads [esi+0x8] (category byte).
 * So the tuple layout used here is read from the callee's own code, not
 * assumed:
 *
 *     +0x0  next          (0x10be5d5c `mov ecx,[ecx]`, 0x10be5d92 `mov esi,[esi]`)
 *     +0x4  opcode        (0x10be5d55 `cmp [ecx+0x4],ebx` with ebx = 0x30f)
 *     +0x8  category byte (0x10be5d6b `movzx edi,BYTE PTR [esi+0x8]`)
 *     +0x9  flags         WB_DAGORDER_FINDINGS.md §2, bit 0 = real-instruction
 *     +0xa  condition code (& 0x1f), same source
 *
 * The first three are [R] from the code path this tap sits on; +0x9 and +0xa
 * are [R] from the whitebox record and are NOT confirmed by anything this tap
 * reads. DISCLOSURE.md carries a row for each.
 *
 * BOUNDED FOREIGN WALK: at most WALK_MAX nodes, all within WALK_SPAN of the
 * first, all 4-byte aligned. An overrun emits REFUSE and never an empty block.
 */
#define WALK_MAX  4096u
#define WALK_SPAN (64u * 1024u * 1024u)

/* Fail-closed field reads. An unreadable field and a zero field MUST render
 * differently: the whole point of this probe is a pre/post-COLOR comparison,
 * and an instrument that prints `0` for "could not read" would manufacture
 * both nulls and differences. Every caller below prints a distinguishable
 * token for each of the three outcomes (a value, a NULL, an unreadable). */
static int rd32(unsigned int p, unsigned int off, unsigned int *out)
{
    if (!plausible(p)) return 0;
    memcpy(out, (const void *)(uintptr_t)(p + off), 4);
    return 1;
}

static int rd8(unsigned int p, unsigned int off, unsigned int *out)
{
    if (!plausible(p)) return 0;
    *out = *(const unsigned char *)(uintptr_t)(p + off);
    return 1;
}

static int rd16(unsigned int p, unsigned int off, unsigned int *out)
{
    unsigned short v;
    if (!plausible(p)) return 0;
    memcpy(&v, (const void *)(uintptr_t)(p + off), 2);
    *out = v;
    return 1;
}

/* One pointer hop that ends in a register number: `base+off` is a descriptor
 * pointer and `descriptor+0x1c` is the register, in the `n = r + 1` encoding
 * (`FUN_10bfebf7` bounds it to 0x0f..0x20 = r14..r31). Three outcomes, three
 * spellings: the number, `none` (a NULL descriptor — NOT YET ASSIGNED), and
 * `unread` (a non-NULL pointer the plausibility filter refused). */
static void ap_regfield(unsigned int base, unsigned int off)
{
    unsigned int d = 0, n = 0;
    ap(" ");
    if (!rd32(base, off, &d)) { ap("unread"); return; }
    if (d == 0)               { ap("none");   return; }
    if (!rd32(d, 0x1c, &n))   { ap("unread"); return; }
    ap_hex(n, 8);
}

/* At most this many operands per list. A list is a foreign singly-linked
 * chain; an overrun is announced, never silently truncated. */
#define OPS_MAX 128u

/* Walk the two operand lists a tuple points to.
 *
 * READ FROM c2's OWN CODE, not assumed — every offset with its witness:
 *   tuple+0x28  operand list D   `FUN_10b2ceb7` walks `piVar13[10]`
 *   tuple+0x2c  operand list S   `FUN_10b2ceb7` walks `piVar13[0xb]`;
 *                                `FUN_10bfebf7` walks `puVar1[0xb]`
 *   op+0x00     next             all of the above (`puVar3 = *puVar3`)
 *   op+0x08     kind (byte)      `*(char *)(puVar3 + 2) == '\x01'`
 *   op+0x0a     type word (u16)  class nibble is `(u16 at op+0xa) >> 12`,
 *                                indexed into `DAT_10b022cc`
 *   op+0x1c     symbol           `piVar18[7]` / `puVar3[7]`
 *   sym+0x04    symbol kind      `cVar1 = *(char *)(iVar9 + 4)`; 1, 2 or 3
 *   sym+0x1c    candidate id     kind-2 arm feeds it to `FUN_10b2c21d`
 *                                (candidate lookup by id)
 *   sym+0x10 -> +0x1c  assigned register   `FUN_10b31ac9`:
 *                                `*(uint *)(*(int *)(param_3 + 0x10) + 0x1c)`
 *   sym+0x08 -> +0x1c  physical register   `FUN_10b2ceb7` kind-1 arm:
 *                                `*(uint *)(*(int *)(iVar9 + 8) + 0x1c)`
 *
 * All `[R]`. DISCLOSURE.md carries a row for each.
 *
 * SCHEMA: no address appears. `id` is a compilation-global monotonic counter
 * (`DAT_10c400d4++`) and the register fields are small integers, so the stream
 * stays canonical — that is asserted by the determinism gate, not assumed. */
static void tap_walk_operands(unsigned int tup)
{
    static const unsigned int list_off[2] = { 0x28u, 0x2cu };
    static const char *const  list_tag[2] = { "D", "S" };
    int L;
    for (L = 0; L < 2; L++) {
        unsigned int op = 0;
        unsigned int j = 0;
        if (!rd32(tup, list_off[L], &op)) {
            ap_reserved("REFUSE operand tuple-unreadable\n");
            return;
        }
        while (op != 0 && j < OPS_MAX) {
            unsigned int nxt = 0, kind = 0, ty = 0, sym = 0;
            if (!plausible(op)) {
                ap_reserved("REFUSE operand walk-implausible\n");
                break;
            }
            rd32(op, 0x0, &nxt);
            rd8(op, 0x8, &kind);
            rd16(op, 0xa, &ty);
            rd32(op, 0x1c, &sym);
            ap("OP ");  ap(list_tag[L]);
            ap(" ");    ap_dec(j);
            ap(" ");    ap_hex(kind, 2);
            ap(" ");    ap_hex(ty, 4);
            /* FOLLOW `op+0x1c` ONLY FOR OPERAND KIND 1, because that is the
             * only kind c2 itself follows it for: `FUN_10bfebf7` guards with
             * `*(char *)(puVar3 + 2) == '\x01'` before reading `puVar3[7]`,
             * and `FUN_10b2ceb7` does the same. Other kinds put other things
             * there — kind `0x0b` (the call-clobber set) carries a physical
             * register BITSET at `op+0x18`, and its `+0x1c` read back a c2
             * static data address on the first run of this probe. Following a
             * field c2 does not follow is how an instrument invents both a
             * finding and a SCHEMA VIOLATION (an address in the canonical
             * stream) in the same read. */
            if (kind != 1) {
                ap(" nofollow");
            } else if (sym == 0) {
                ap(" nosym");
            } else if (!plausible(sym)) {
                ap(" unread");
            } else {
                unsigned int sk = 0, id = 0;
                rd8(sym, 0x4, &sk);
                rd32(sym, 0x1c, &id);
                ap(" ");  ap_hex(sk, 2);
                ap(" ");  ap_hex(id, 8);
                ap_regfield(sym, 0x10);   /* the ASSIGNED register */
                ap_regfield(sym, 0x08);   /* the PHYSICAL register */
            }
            ap("\n");
            if (g_arena_full) { ap_reserved("REFUSE region arena-full\n"); return; }
            j++;
            op = nxt;
        }
        if (j >= OPS_MAX) { ap_reserved("REFUSE operand walk-overrun\n"); return; }
    }
}

/* At most this many blocks per function. */
#define BLK_MAX 4096u

/* Longest identity this tap will publish. Mangled X360 C++ names run long; 512
 * is far above anything in the fixture corpus and far below the 0x1020 buffer
 * c2's own name path (`FUN_10b99dfe`) uses. */
#define SYMNAME_MAX 512u

/* THE FUNCTION IDENTITY — board #3459.
 *
 * Before this, the funcwalk payload carried an ORDINAL and nothing else, and
 * every consumer in the tree paired that ordinal with the i-th `.text` function
 * in address order. That pairing is an ASSUMPTION: `g_fn` counts `sched1`
 * entries, `sched1` is one of the four optimizer-gated sites, and c2's own
 * processing order is not documented to be address order. `w-pwords` §5 found
 * the assumption failing live — mismatches in BOTH directions on one TU, the
 * signature of a permuted pairing — and could only quarantine the TU, because
 * nothing in the payload could name the function.
 *
 * This writes the name. THREE HOPS, each read from the disassembly of
 * c2.dll sha256 c80981c0…a66258, and disclosed as `W-STAGETAP-6`:
 *
 *   func + 0x00  -> the `.gl` SYMBOL record
 *        `FUN_10bc4715`, the function-record constructor:
 *        `pbVar3 = alloc(0xac); *(int *)pbVar3 = param_1;` where `param_1` is
 *        the symbol the work queue dequeued in `FUN_10b7f1ff` (the record with
 *        `+0x4c` emit bits and `+0x78` next).
 *   sym  + 0x04  -> char *, NUL-terminated
 *        `0x10b9acd0` `mov eax,DWORD PTR [ecx+0x4]` in c2's own symbol-name
 *        getter `FUN_10b9acc4`, with ecx = the symbol; and `0x10b97f38`
 *        `mov esi,DWORD PTR [ecx+0x4]` in `FUN_10b97f37`, which NUL-tests it.
 *   sym  + 0x30 / + 0x31  -> kind / sub-kind
 *        `0x10b97f6b` `movzx eax,BYTE PTR [ecx+0x30]`; `P_SYMBOL.md` §1.
 *
 * WHAT IS DELIBERATELY *NOT* DONE HERE. `FUN_10b9acc4` returns `[sym+4]`
 * verbatim only when `FUN_10b97f37` allows it (kind 4, kind 9, or a kind-1
 * `?`-name); for kind 3 it goes through `FUN_10b99dfe`, which copies `[sym+4]`
 * and returns it UNCHANGED the moment it starts with `'?'`, and otherwise
 * applies per-sub-kind decoration (`"__unwind$"` for 'T', `"__catch$"` for 'V',
 * `"$M"` for 'W', a trailing `"$"` for '\0'). This tap reimplements NONE of
 * that. It publishes the RAW field plus the two kind bytes and leaves the
 * consumer to match by exact equality — a name that does not match is reported
 * unmatched and counted, never quietly demoted back to address order. Porting
 * the decoration here would put a guess between the read and the check.
 *
 * FAIL-CLOSED, in the same three-spellings discipline `rd32` already follows:
 * a value, or one of `<unread>` / `<empty>` / `<nonascii>` / `<toolong>`. None
 * of those is a legal C identifier or mangled name, so a consumer cannot mistake
 * a refusal for an identity, and an unreadable field never renders as an empty
 * one. The scan runs TWICE — validate, then emit — so a name that turns out to
 * be unprintable halfway through never appears half-written. */
static void ap_symid(unsigned int func)
{
    unsigned int sym = 0, kind = 0, subkind = 0, nameptr = 0, n = 0, i;
    const unsigned char *p;
    char one[2];

    ap(" sk ");
    if (!rd32(func, 0x0, &sym) || !plausible(sym)) {
        ap("?? ?? nm <unread>");
        return;
    }
    rd8(sym, 0x30, &kind);
    rd8(sym, 0x31, &subkind);
    ap_hex(kind, 2);
    ap(" ");
    ap_hex(subkind, 2);
    ap(" nm ");
    if (!rd32(sym, 0x4, &nameptr) || !plausible_bytes(nameptr)) {
        ap("<unread>");
        return;
    }
    p = (const unsigned char *)(uintptr_t)nameptr;
    /* Pass 1 — validate and measure, appending nothing. */
    for (n = 0; n < SYMNAME_MAX; n++) {
        unsigned char c = p[n];
        if (c == 0) break;
        /* Space is refused along with the non-printables on purpose: the name
         * is emitted as the TAIL of a whitespace-delimited line, so a name
         * containing a space would silently re-split on the reader's side. */
        if (c < 0x21u || c > 0x7eu) { ap("<nonascii>"); return; }
    }
    if (n == 0)           { ap("<empty>");   return; }
    if (n >= SYMNAME_MAX) { ap("<toolong>"); return; }
    /* Pass 2 — emit. */
    one[1] = 0;
    for (i = 0; i < n; i++) { one[0] = (char)p[i]; ap(one); }
}

/* Walk one whole function from its record — the observable the region tap
 * cannot give, and the one `after0` needs.
 *
 * READ FROM `FUN_10b2ceb7`, which the allocator driver `FUN_10b31c9a` calls
 * with the function record straight through:
 *   func+0x08 -> hdr,  hdr+0x04 -> first block
 *                                `iVar14 = *(int *)(*(int *)(param_1 + 8) + 4)`
 *   block+0x04  next block       `iVar14 = *(int *)(iVar14 + 4)` (loop tail)
 *   block+0x20  the LAST tuple   `piVar13 = *(int **)(iVar14 + 0x20)`
 *   block+0x1c  the stop value   `while (piVar13 != *(int **)(iVar14 + 0x1c))`
 *   tuple+0x10  PREV             `piVar13 = (int *)piVar17[4]` (loop tail)
 *
 * THE WALK IS BACKWARD, and that is not a guess: `tuple+0x0` is `next` and
 * `tuple+0x10` is `prev` (`0x10be626c` re-links the list with exactly that
 * pair, `P_DAG.md` §2), so a loop advancing by `+0x10` runs from the end. The
 * rows are therefore emitted in REVERSE order within each block and the
 * `BLK … rev` marker says so; the reader reverses. Emitting them in the
 * apparent order and calling it the schedule would be the whole probe's
 * conclusion, silently inverted.
 *
 * All `[R]`, and CROSS-DERIVED rather than trusted: PREREG B2 requires the
 * function walk's tuple set at `sched2` to contain every distinct row the
 * region walk reports at `sched2`. If it does not, this reading is wrong and
 * the probe says so. */
static void tap_walk_function(unsigned int func)
{
    unsigned int hdr = 0, blk = 0, nb = 0;
    if (!rd32(func, 0x8, &hdr) || !plausible(hdr)) {
        ap_reserved("REFUSE funcwalk header\n");
        return;
    }
    if (!rd32(hdr, 0x4, &blk)) {
        ap_reserved("REFUSE funcwalk first-block\n");
        return;
    }
    while (blk != 0 && nb < BLK_MAX) {
        unsigned int t = 0, stop = 0, i = 0, next_blk = 0;
        if (!plausible(blk)) { ap_reserved("REFUSE funcwalk block-implausible\n"); return; }
        if (!rd32(blk, 0x20, &t) || !rd32(blk, 0x1c, &stop)) {
            ap_reserved("REFUSE funcwalk block-fields\n");
            return;
        }
        ap("BLK ");  ap_dec(nb);  ap(" rev\n");
        while (t != 0 && t != stop && i < WALK_MAX) {
            const unsigned char *b;
            unsigned int prev = 0, opcode = 0;
            if (!plausible(t)) { ap_reserved("REFUSE funcwalk tuple-implausible\n"); return; }
            b = (const unsigned char *)(uintptr_t)t;
            memcpy(&opcode, b + 4, 4);
            memcpy(&prev, b + 0x10, 4);
            ap("FT ");  ap_dec(i);
            ap(" ");    ap_hex(opcode, 8);
            ap(" ");    ap_hex(b[8], 2);
            ap(" ");    ap_hex(b[9], 2);
            ap(" ");    ap_hex((unsigned int)(b[10] & 0x1fu), 2);
            ap("\n");
            if (g_ops && (b[9] & 1u)) tap_walk_operands(t);
            if (g_arena_full) { ap_reserved("REFUSE funcwalk arena-full\n"); return; }
            i++;
            t = prev;
        }
        if (i >= WALK_MAX) { ap_reserved("REFUSE funcwalk tuple-overrun\n"); return; }
        if (!rd32(blk, 0x4, &next_blk)) { ap_reserved("REFUSE funcwalk block-next\n"); return; }
        nb++;
        blk = next_blk;
    }
    if (nb >= BLK_MAX) { ap_reserved("REFUSE funcwalk block-overrun\n"); return; }
}

static void tap_walk_tuples(unsigned int t)
{
    unsigned int i = 0;
    unsigned int first = t;
    if (!plausible(t)) {
        ap_reserved("REFUSE region walk-implausible-head\n");
        return;
    }
    while (i < WALK_MAX) {
        const unsigned char *b = (const unsigned char *)(uintptr_t)t;
        unsigned int opcode;
        unsigned int next;
        unsigned int d = (t > first) ? (t - first) : (first - t);
        if (d > WALK_SPAN) { ap_reserved("REFUSE region walk-span\n"); return; }

        memcpy(&opcode, b + 4, 4);
        memcpy(&next, b + 0, 4);

        ap("TU ");    ap_dec(i);
        ap(" ");      ap_hex(opcode, 8);
        ap(" ");      ap_hex(b[8], 2);
        ap(" ");      ap_hex(b[9], 2);
        ap(" ");      ap_hex((unsigned int)(b[10] & 0x1fu), 2);
        ap("\n");
        if (g_raw) {
            unsigned int k;
            ap("RAW ");  ap_dec(i);  ap(" ");
            for (k = 0; k < g_raw; k++) ap_hex(b[k], 2);
            ap("\n");
        }
        /* ONLY REAL-INSTRUCTION TUPLES HAVE OPERAND LISTS. `FUN_10bfebf7`
         * and `FUN_10b2ceb7` both gate their operand walk on
         * `*(byte *)(tuple + 9) & 1` (WB_DAGORDER_FINDINGS.md §2, bit 0 =
         * real-instruction). Without the gate this walk reads `+0x28`/`+0x2c`
         * out of label and region-marker tuples, where they are not list
         * heads: the first run of this probe produced operand kinds `0xf8`
         * and symbol kinds `0x88` from exactly those rows. */
        if (g_ops && (b[9] & 1u)) tap_walk_operands(t);
        if (g_arena_full) { ap_reserved("REFUSE region arena-full\n"); return; }

        i++;
        if (next == 0) return;               /* end of list: a real terminus */
        if (!plausible(next)) { ap_reserved("REFUSE region walk-implausible-next\n"); return; }
        t = next;
    }
    ap_reserved("REFUSE region walk-overrun\n");
}

/* ---------------------------------------------------------------------------
 * THE CANDIDATE PAYLOAD (lane w24-c2rs) — every offset with its witness, all
 * [R] from c2.dll sha256 c80981c0…a66258 and DISCLOSURE row `W-STAGETAP-7`:
 *
 *   cand+0x00  the local SYMBOL the candidate was minted from
 *              (`FUN_10b54d32(s)`, P_GLOBREGS.md §1 step 3; read back at
 *              0x10b31eba `mov eax,[edi]` / `cmp BYTE PTR [eax+0x4],0xd`)
 *   cand+0x04..+0x07  kind / flag bytes (`test BYTE PTR [edi+0x5],0x4` at
 *              0x10b31ee1)
 *   cand+0x0c  THE PRIORITY (comparator primary key, signed)
 *   cand+0x10  assigned register descriptor = 0x10c2f088 + reg*0x60
 *              (0x10b2e926 `imul eax,eax,0x60` / `add eax,0x10c2f088` /
 *              `mov [ebx+0x10],eax`)
 *   cand+0x1c  candidate id (per-function, dense from 1)
 *   cand+0x20  allowed set: sparse bitset, head at [set]; each chunk is
 *              {base & ~63, lo32, hi32, next} (0x10b26d68), bit i of a chunk
 *              tested against the mask pair at 0x10b014c0 + i*8 (0x10b26f46)
 *   cand+0x24  ref count;  cand+0x40  benefit (popped arm tests `> 0` at
 *              0x10b31ea9);  cand+0x44  THE TIE KEY (tuple-visit ordinal)
 *   0x10c435e8 the selector's cost array, 357 ints, still holding THIS
 *              candidate's costs at the `casg` site (it is memset at the
 *              selector's entry, P_REGALLOC.md §3)
 *
 * Register numbers print in c2's own numbering (P_REGALLOC.md §2.1):
 * 1..32 = r0..r31 as n-1 (2 = sp, 3 = toc), 34..65 = fp0..fp31 as n-34.
 * ------------------------------------------------------------------------- */
#define C2_REGDESC_VA  0x10c2f088u
#define C2_COSTARR_VA  0x10c435e8u
#define C2_BITMASK_VA  0x10b014c0u
#define C2_NREGS       357u

static void ap_sdec(unsigned int v)
{
    if ((int)v < 0) { ap("-"); ap_dec((unsigned int)(-(int)v)); }
    else ap_dec(v);
}

static void ap_regname(unsigned int n)
{
    if (n >= 1u && n <= 32u)       { ap("r");  ap_dec(n - 1u); }
    else if (n >= 34u && n <= 65u) { ap("f");  ap_dec(n - 34u); }
    else                           { ap("#");  ap_dec(n); }
}

static unsigned int csym_ordinal(unsigned int sym)
{
    unsigned int i;
    for (i = 0; i < g_ncsym; i++) if (g_csym[i] == sym) return i;
    if (g_ncsym < CSYM_MAX) { g_csym[g_ncsym] = sym; return g_ncsym++; }
    return 0xffffffffu;
}

static void ap_bitset(unsigned int setp)
{
    unsigned int chunk = 0, n = 0;
    const unsigned int *mask = (const unsigned int *)(uintptr_t)(C2_BITMASK_VA + g_slide);
    if (!rd32(setp, 0, &chunk)) { ap("<unread>"); return; }
    ap("{");
    while (chunk != 0 && n < 64u) {
        unsigned int base = 0, lo = 0, hi = 0, nxt = 0, i;
        if (!plausible(chunk)) { ap("<implausible>"); break; }
        rd32(chunk, 0x0, &base);
        rd32(chunk, 0x4, &lo);
        rd32(chunk, 0x8, &hi);
        rd32(chunk, 0xc, &nxt);
        for (i = 0; i < 64u; i++) {
            if ((lo & mask[i * 2u]) | (hi & mask[i * 2u + 1u])) {
                ap(" "); ap_regname(base + i);
            }
        }
        chunk = nxt;
        n++;
    }
    ap(" }");
}

/* Candidate-id ORIGIN (lane w25-regasg): which mint site produced each id
 * of the function being allocated. Ids restart at 1 per function
 * (0x10b57676) and every mint for a function precedes its COLOR, so an
 * id-indexed table overwritten function by function is exact. */
#define C2_CANDID_VA   0x10c400d4u   /* the per-function id counter */
#define C2_CANDFREE_VA 0x10c2e3e0u   /* the candidate free list head */
#define MINT_MAX 16384u
static unsigned char g_mint_site[MINT_MAX];   /* 1 + index into g_sites */
static unsigned int  g_mint_seq[MINT_MAX];    /* mint order within the function */
static unsigned int  g_mint_n = 0;
/* The function's mint LOG, flushed as `CMINT` lines right after its `CFN`
 * line (every globregs mint precedes COLOR); mints made during COLOR itself
 * (split children, the liveness driver) print as they happen. */
#define MLOG_MAX 4096u
static unsigned int  g_mlog_id[MLOG_MAX], g_mlog_sym[MLOG_MAX];
static unsigned char g_mlog_site[MLOG_MAX], g_mlog_reuse[MLOG_MAX];
static unsigned int  g_mlog_x[MLOG_MAX];      /* CFREE: cand+0x40; CDEMOTE: the new temp */
static unsigned int  g_mlog_op[MLOG_MAX];     /* CDEMOTE: the def tuple's opcode */
static unsigned int  g_mlog_n = 0;
static int           g_in_color = 0;

static void ap_mnem(unsigned int opcode);
static void ap_symk(unsigned int sym)
{
    unsigned int sk = 0;
    if (plausible(sym)) {
        ap_dec(csym_ordinal(sym));
        rd8(sym, 0x4, &sk);
        ap("/k"); ap_hex(sk, 2);
    } else ap("?");
}

static void ap_cmint(unsigned int seq, unsigned int id, int site, int reuse, unsigned int sym)
{
    unsigned int sk = 0;
    ap(reuse == 2 ? "CFREE " : "CMINT "); ap_dec(seq);
    ap(" id ");   ap_dec(id);
    ap(" ");      ap(g_sites[site].name + 2);
    ap(reuse == 2 ? "" : reuse ? " reuse" : " fresh");
    ap(" sym ");
    ap_symk(sym);
    (void)sk;
    ap("\n");
}

static void ap_cmint_x(unsigned int seq, unsigned int id, int site, int reuse, unsigned int sym,
                       unsigned int x, unsigned int op)
{
    if (reuse == 3) {
        ap("CDEMOTE "); ap_dec(seq);
        ap(" id ");     ap_dec(id);
        ap(" sym ");    ap_symk(sym);
        ap(" def ");    ap_mnem(op);
        ap(" -> temp "); ap_symk(x);
        ap("\n");
        return;
    }
    if (reuse == 2) {
        ap("CFREE "); ap_dec(seq);
        ap(" id ");   ap_dec(id);
        ap(" ");      ap(g_sites[site].name + 2);
        ap(" sym ");  ap_symk(sym);
        ap(" b40 ");
        if ((int)x < 0) { ap("-"); ap_dec((unsigned int)(-(int)x)); } else ap_dec(x);
        ap("\n");
        return;
    }
    ap_cmint(seq, id, site, reuse, sym);
}

static void free_note(int site, unsigned int cand)
{
    unsigned int id = 0, sym = 0;
    if (!rd32(cand, 0x1c, &id)) return;
    rd32(cand, 0x00, &sym);
    unsigned int b40 = 0;
    rd32(cand, 0x40, &b40);
    if (g_in_color) {
        if (g_show) { ap("  "); ap_cmint_x(g_mint_n, id, site, 2, sym, b40, 0); }
    } else if (g_mlog_n < MLOG_MAX) {
        g_mlog_id[g_mlog_n] = id; g_mlog_sym[g_mlog_n] = sym;
        g_mlog_site[g_mlog_n] = (unsigned char)site; g_mlog_reuse[g_mlog_n] = 2;
        g_mlog_x[g_mlog_n] = b40; g_mlog_op[g_mlog_n] = 0;
        g_mlog_n++;
    }
    g_mint_n++;
}

/* `cbdem`: ecx = the def tuple, edx = its def operand (whose `+0x1c` is
 * still the candidate being demoted), [esp+4] = the new temp symbol. */
static void demote_note(int site, unsigned int tuple, unsigned int op, unsigned int callee_esp)
{
    unsigned int cand = 0, id = 0, sym = 0, nsym = 0, opc = 0;
    if (!rd32(op, 0x1c, &cand) || !plausible(cand) || !rd32(cand, 0x1c, &id)) return;
    rd32(cand, 0x00, &sym);
    rd32(tuple, 0x4, &opc);
    memcpy(&nsym, (const void *)(uintptr_t)(callee_esp + 4u), 4);
    if (g_mlog_n < MLOG_MAX) {
        g_mlog_id[g_mlog_n] = id; g_mlog_sym[g_mlog_n] = sym;
        g_mlog_site[g_mlog_n] = (unsigned char)site; g_mlog_reuse[g_mlog_n] = 3;
        g_mlog_x[g_mlog_n] = nsym; g_mlog_op[g_mlog_n] = opc;
        g_mlog_n++;
    }
}

static void ben_reset(void);

static void mint_note(int site, unsigned int sym)
{
    unsigned int id = 0, fl = 0;
    fl = *(const unsigned int *)(uintptr_t)(C2_CANDFREE_VA + g_slide);
    if (fl == 0) id = *(const unsigned int *)(uintptr_t)(C2_CANDID_VA + g_slide);
    else if (!rd32(fl, 0x1c, &id)) return;
    if (id == 1u && fl == 0) { g_mint_n = 0; g_mlog_n = 0; g_in_color = 0; ben_reset(); }  /* a fresh function */
    if (id < MINT_MAX) { g_mint_site[id] = (unsigned char)(site + 1); g_mint_seq[id] = g_mint_n; }
    if (g_in_color) {
        if (g_show) { ap("  "); ap_cmint(g_mint_n, id, site, fl != 0, sym); }
    } else if (g_mlog_n < MLOG_MAX) {
        g_mlog_id[g_mlog_n] = id; g_mlog_sym[g_mlog_n] = sym;
        g_mlog_site[g_mlog_n] = (unsigned char)site; g_mlog_reuse[g_mlog_n] = (unsigned char)(fl != 0);
        g_mlog_n++;
    }
    g_mint_n++;
}

/* ---------------------------------------------------------------------------
 * THE BENEFIT TERMS (lane w25-regasg, DISCLOSURE `W-STAGETAP-10`). Before
 * COLOR, `cand+0x40` is written only by the step-4 walk FUN_10b55eae:
 *
 *   per DEF operand  (after 0x10b55fb7): if the symbol is versioned
 *                    (`[[sym+0x34]+0x18] != 0`, 0x10b55fbc..c3) and the
 *                    tuple's opcode is 0x2b7 (0x10b55fe9) -> `dec` (0x10b561fd)
 *   per USE operand  (after 0x10b563ac): if the symbol is versioned
 *                    (0x10b563b1..bb)                       -> `inc` (0x10b563c3)
 *   pinned           0x10b560df writes 0x10000000 (spill-home path)
 *
 * so the benefit the negative-benefit pass FUN_10bfcf7c reads is
 *   ben = (versioned uses) - (versioned defs in 0x2b7 tuples)   [or pinned].
 * The tap counts each term at its call, keyed by candidate id (`op+0x1c` is
 * the candidate, stored just before both calls), and at `cbpre` reads the
 * field the pass is about to test, so every `CBEN` line is CHECKED: the
 * recount and c2's own field must agree, or the line says `DIFF`.
 * ------------------------------------------------------------------------- */
#define BT_MAX 4096u
static unsigned int   g_bt_cand[BT_MAX], g_bt_sym[BT_MAX];
static unsigned short g_bt_use[BT_MAX], g_bt_unc[BT_MAX], g_bt_kill[BT_MAX], g_bt_def[BT_MAX];
static unsigned int   g_bt_max = 0;
/* the CBEN snapshot taken at `cbpre`, flushed after the CMINT log */
static unsigned int   g_bs_id[BT_MAX], g_bs_sym[BT_MAX], g_bs_n = 0;
static int            g_bs_ben[BT_MAX];
static unsigned short g_bs_use[BT_MAX], g_bs_unc[BT_MAX], g_bs_kill[BT_MAX], g_bs_def[BT_MAX];

static void ben_reset(void)
{
    unsigned int k;
    for (k = 0; k <= g_bt_max && k < BT_MAX; k++) {
        g_bt_cand[k] = 0; g_bt_sym[k] = 0;
        g_bt_use[k] = g_bt_unc[k] = g_bt_kill[k] = g_bt_def[k] = 0;
    }
    g_bt_max = 0; g_bs_n = 0;
}

/* `[[sym+0x34]+0x18] != 0` — the versioned test both arms apply. */
static int ben_versioned(unsigned int sym)
{
    unsigned int v = 0, n = 0;
    if (!plausible(sym) || !rd32(sym, 0x34, &v) || !plausible(v)) return 0;
    if (!rd32(v, 0x18, &n)) return 0;
    return n != 0;
}

static void ben_note(int is_def, unsigned int tuple, unsigned int op)
{
    unsigned int cand = 0, id = 0, sym = 0, opc = 0;
    if (!rd32(op, 0x1c, &cand) || !plausible(cand) || !rd32(cand, 0x1c, &id)) return;
    if (id >= BT_MAX) return;
    rd32(op, 0x18, &sym);
    rd32(tuple, 0x4, &opc);
    g_bt_cand[id] = cand;
    if (g_bt_sym[id] == 0) rd32(cand, 0x00, &g_bt_sym[id]);
    if (id > g_bt_max) g_bt_max = id;
    if (is_def) {
        if (ben_versioned(sym) && opc == 0x2b7u) g_bt_kill[id]++;
        else g_bt_def[id]++;
    } else {
        if (ben_versioned(sym)) g_bt_use[id]++;
        else g_bt_unc[id]++;
    }
}

static void ben_snapshot(void)
{
    unsigned int k, b = 0;
    g_bs_n = 0;
    for (k = 1; k <= g_bt_max && k < BT_MAX; k++) {
        if (!g_bt_cand[k] || !rd32(g_bt_cand[k], 0x40, &b)) continue;
        g_bs_id[g_bs_n] = k; g_bs_sym[g_bs_n] = g_bt_sym[k];
        g_bs_ben[g_bs_n] = (int)b;
        g_bs_use[g_bs_n] = g_bt_use[k]; g_bs_unc[g_bs_n] = g_bt_unc[k];
        g_bs_kill[g_bs_n] = g_bt_kill[k]; g_bs_def[g_bs_n] = g_bt_def[k];
        g_bs_n++;
    }
}

static void ap_ben_log(void)
{
    unsigned int k, sk = 0;
    for (k = 0; k < g_bs_n; k++) {
        int pred = (int)g_bs_use[k] - (int)g_bs_kill[k];
        ap("  CBEN id "); ap_dec(g_bs_id[k]);
        ap(" sym ");
        if (plausible(g_bs_sym[k])) {
            ap_dec(csym_ordinal(g_bs_sym[k])); rd8(g_bs_sym[k], 0x4, &sk);
            ap("/k"); ap_hex(sk, 2);
        } else ap("?");
        ap(" ben ");
        if (g_bs_ben[k] < 0) { ap("-"); ap_dec((unsigned int)(-g_bs_ben[k])); }
        else ap_dec((unsigned int)g_bs_ben[k]);
        ap(" = use "); ap_dec(g_bs_use[k]);
        ap(" - kill "); ap_dec(g_bs_kill[k]);
        ap(" (unversioned-use "); ap_dec(g_bs_unc[k]);
        ap(", other-def "); ap_dec(g_bs_def[k]); ap(")");
        if (g_bs_ben[k] == 0x10000000) ap(" pinned");
        else if (g_bs_ben[k] != pred) ap(" DIFF");
        ap(g_bs_ben[k] < 0 ? " NEG\n" : "\n");
    }
}

/* For a kind-0xd symbol (FUN_10b5673e's constants/addresses): the operand it
 * was built from (`sym+0x28`, `FUN_10bd6bfb(param_2)`), its kind byte, and —
 * when that operand points at a symbol record with a printable name at
 * `+0x04` — the name. Constants print their low word. */
static void ap_k0d(unsigned int sym)
{
    unsigned int op = 0, ok = 0, v = 0, np = 0, n;
    if (!rd32(sym, 0x28, &op) || !plausible(op)) { ap(" val ?"); return; }
    rd8(op, 0x8, &ok);
    rd32(op, 0x18, &v);
    ap(" val k"); ap_hex(ok, 2);
    if (ok == 7u) { ap(" #"); ap_hex(v, 8); return; }
    if (plausible(v) && rd32(v, 0x4, &np) && plausible_bytes(np)) {
        const char *s = (const char *)(uintptr_t)np;
        for (n = 0; n < 64u && s[n] > 0x20 && s[n] < 0x7f; n++) {}
        if (n > 0 && (s[n] == 0 || n == 64u)) {
            char one[2]; unsigned int k;
            one[1] = 0; ap(" ");
            for (k = 0; k < n; k++) { one[0] = s[k]; ap(one); }
            return;
        }
    }
    ap(" ?");
}

static void ap_cand(const char *what, unsigned int cand, int with_allowed)
{
    unsigned int v = 0, sym = 0, sk = 0, fl = 0;
    ap("CAND "); ap(what);
    ap(" fn ");  ap_dec(g_cfn);
    if (!plausible(cand)) { ap(" <unread>\n"); return; }
    rd32(cand, 0x1c, &v); ap(" id ");  ap_dec(v);
    rd32(cand, 0x0c, &v); ap(" pri "); ap_sdec(v);
    rd32(cand, 0x44, &v); ap(" tie "); ap_dec(v);
    rd32(cand, 0x40, &v); ap(" ben "); ap_sdec(v);
    rd32(cand, 0x24, &v); ap(" ref "); ap_sdec(v);
    rd32(cand, 0x04, &fl); ap(" fl "); ap_hex(fl, 8);
    rd32(cand, 0x00, &sym);
    ap(" sym ");
    if (plausible(sym)) {
        ap_dec(csym_ordinal(sym));
        rd8(sym, 0x4, &sk);
        ap("/k"); ap_hex(sk, 2);
    } else {
        ap("?");
    }
    {
        unsigned int id = 0;
        rd32(cand, 0x1c, &id);
        ap(" org ");
        if (id < MINT_MAX && g_mint_site[id]) {
            ap(g_sites[g_mint_site[id] - 1u].name + 2);   /* drop "cm" */
            ap("#"); ap_dec(g_mint_seq[id]);
        } else ap("?");
        if (plausible(sym) && sk == 0x0du) ap_k0d(sym);
    }
    if (with_allowed) {
        rd32(cand, 0x20, &v);
        ap(" allowed ");
        if (v == 0) ap("none"); else ap_bitset(v);
    }
    ap("\n");
}

static void ap_assigned(unsigned int cand)
{
    unsigned int d = 0, id = 0, i, base = C2_REGDESC_VA + g_slide;
    const int *cost = (const int *)(uintptr_t)(C2_COSTARR_VA + g_slide);
    ap("CAND asg fn "); ap_dec(g_cfn);
    if (!plausible(cand)) { ap(" <unread>\n"); return; }
    rd32(cand, 0x1c, &id); ap(" id "); ap_dec(id);
    rd32(cand, 0x10, &d);
    ap(" reg ");
    if (d >= base && (d - base) % 0x60u == 0 && (d - base) / 0x60u < C2_NREGS)
        ap_regname((d - base) / 0x60u);
    else if (d == 0) ap("none");
    else ap("<off-table>");
    ap(" cost");
    for (i = 0; i < C2_NREGS; i++) {
        if (cost[i] != 0) { ap(" "); ap_regname(i); ap(":"); ap_sdec((unsigned int)cost[i]); }
    }
    ap("\n");
}

/* ---------------------------------------------------------------------------
 * THE REGASG PAYLOAD (lane w25-regasg, DISCLOSURE `W-STAGETAP-8`) — the
 * SECOND allocator, `regasg.c`, which gives a register to every value COLOR
 * did not promote (short-lived expression temps, symbol kind 3). All `[R]`
 * from c2.dll sha256 c80981c0…a66258; every offset with its witness:
 *
 *   per function   FUN_10b7dd2c calls FUN_10bc6487 (0x10b7dd3f), which calls
 *                  the regasg DRIVER FUN_10bc62b6 at 0x10bc6491, ecx = the
 *                  function record (0x10bc6488 `mov esi,ecx` / 0x10bc648f
 *                  `mov ecx,esi`). 0x10c1b867 is the driver's only other
 *                  caller (FUN_10c1b808's re-run, gated on DAT_10c6fe00).
 *   per tuple      the driver walks the tuple list FORWARD by `tuple+0x0`;
 *                  for each real tuple (`+0x9 & 1`) it calls FUN_10bc4ded at
 *                  0x10bc6418 (ecx = tuple: frees the registers of SOURCE
 *                  operands whose `op+0x15 & 0x10` last-use bit is set), then
 *                  FUN_10bc61bb (DEFS). At a label tuple (`+0x8 == 0x1a`) it
 *                  RESETS the three cursors to their list heads
 *                  (0x10bc642b `cmp BYTE PTR [esi+0x8],0x1a`).
 *   THE SELECTOR   FUN_10bc58d5(op = ecx, tuple = edx, [esp+4]); three call
 *                  sites: 0x10bc624f (FUN_10bc61bb: a def with no register
 *                  yet), 0x10bc6145 (FUN_10bc6038: re-home after an evict),
 *                  0x10bc5122 (FUN_10bc4eae: a spill reload).
 *   the commit     every selector exit funnels through ONE call,
 *                  0x10bc602c -> FUN_10bc4ba1(reg = ecx, sym = edx), which
 *                  stores DAT_10c3d730[reg] = sym.
 *   the evict      0x10bc6021 -> FUN_10bc4eae: no free register; spill code
 *                  inserted before the commit.
 *   DAT_10c3d730   occupancy, 357 pointers indexed by c2 register number
 *                  (memset 0x594 at the driver's entry)
 *   DAT_10c3d198   per-register INTERFERENCE set, indexed by register,
 *                  tested against `sym+0x1c` with FUN_10b26f37 (the same
 *                  sparse bitset as the candidate `allowed` set)
 *   cursors        DAT_10c6fe0c (GPR), DAT_10c6fe10 (FPR when
 *                  DAT_10c2e974), DAT_10c6fe08 (VMX) — pointers INTO the
 *                  zero-terminated order lists DAT_10c6fdf4 (GPR, picked per
 *                  function by frame pointer), 0x10c3bee0 (FPR) and
 *                  DAT_10c6fdf8 (VMX). With DAT_10c2e310 set the selector
 *                  scans from the cursor, wraps once, and leaves the cursor
 *                  on the entry AFTER the one it took (back to the head when
 *                  that entry is not volatile) — round robin within a block.
 *   op+0x0a        type word; class nibble `>> 12` (5 FPR, 0xc VMX, else GPR)
 *   op+0x15 & 0x10 the last-use bit (FUN_10bc4ded)
 *   sym+0x1c       the interference key;  sym+0x30 -> +0x1c  copy-hint reg
 *   0x10b1b260     the machine mnemonic table, stride 12 (W-MID-1)
 * ------------------------------------------------------------------------- */
#define C2_RA_OCC_VA    0x10c3d730u
#define C2_RA_INTF_VA   0x10c3d198u
#define C2_RA_CUR_GPR   0x10c6fe0cu
#define C2_RA_CUR_FPR   0x10c6fe10u
#define C2_RA_CUR_VMX   0x10c6fe08u
#define C2_RA_LST_GPR   0x10c6fdf4u   /* a POINTER to the list */
#define C2_RA_LST_VMX   0x10c6fdf8u   /* a POINTER to the list */
#define C2_RA_LST_FPR   0x10c3bee0u   /* the list itself */
#define C2_RA_FPRFLAG   0x10c2e974u
#define C2_RA_RRFLAG    0x10c2e310u
#define C2_MNEM_VA      0x10b1b260u
#define C2_MNEM_LAST    0x295u
#define RA_LIST_MAX     128u

static int          g_regasg = 0;      /* C2RS_STAGE_REGASG=1 */
static unsigned int g_rfn = 0;         /* regasg driver entries */
static unsigned int g_rfn_cfn = 0;     /* g_cfn when the last RFN fired */
static unsigned int g_rtup = 0;        /* real tuples seen this function */
static unsigned int g_rblk = 0;        /* cursor resets (labels) passed */
static unsigned int g_rlast = 0;       /* the previous real tuple */
static unsigned int g_rcur_tup = 0;    /* the tuple of the open selection */


static int name_matches(unsigned int func)
{
    unsigned int sym = 0, nameptr = 0, i, j;
    const char *p;
    if (!g_fnfilter[0]) return 1;
    if (!rd32(func, 0x0, &sym) || !plausible(sym)) return 0;
    if (!rd32(sym, 0x4, &nameptr) || !plausible_bytes(nameptr)) return 0;
    p = (const char *)(uintptr_t)nameptr;
    for (i = 0; i < SYMNAME_MAX && p[i]; i++) {
        for (j = 0; g_fnfilter[j] && p[i + j] == g_fnfilter[j]; j++) {}
        if (!g_fnfilter[j]) return 1;
    }
    return 0;
}

static unsigned int sreg(unsigned int va) { return *(const unsigned int *)(uintptr_t)(va + g_slide); }

static void ap_mnem(unsigned int opcode)
{
    unsigned int np = 0, n;
    const char *s;
    if (opcode == 0 || opcode >= C2_MNEM_LAST) { ap("#"); ap_hex(opcode, 3); return; }
    np = sreg(C2_MNEM_VA + opcode * 12u);
    if (!plausible_bytes(np)) { ap("#"); ap_hex(opcode, 3); return; }
    s = (const char *)(uintptr_t)np;
    for (n = 0; n < 16u && s[n] > 0x20 && s[n] < 0x7f; n++) {}
    if (n == 0 || n >= 16u) { ap("#"); ap_hex(opcode, 3); return; }
    {
        char one[2]; unsigned int k;
        one[1] = 0;
        for (k = 0; k < n; k++) { one[0] = s[k]; ap(one); }
    }
}

/* An occupant of DAT_10c3d730: a physical register DESCRIPTOR (the static
 * table 0x10c2f088 + n*0x60), or a symbol record (kind 3 = a regasg temp). */
static void ap_occupant(unsigned int occ)
{
    unsigned int base = C2_REGDESC_VA + g_slide, k = 0;
    if (occ >= base && occ < base + C2_NREGS * 0x60u) { ap("phys"); return; }
    if (!plausible(occ)) { ap("?"); return; }
    rd8(occ, 0x4, &k);
    if (k == 3u) { ap("s"); ap_dec(csym_ordinal(occ)); }
    else { ap("k"); ap_hex(k, 2); }
}

/* Membership of `key` in the sparse bitset at `setp` — FUN_10b26f37 /
 * FUN_10b26d68: chunks ascending by base, stop at the first base >= key&~63. */
static int bitset_has(unsigned int setp, unsigned int key)
{
    unsigned int chunk = 0, n = 0;
    const unsigned int *mask = (const unsigned int *)(uintptr_t)(C2_BITMASK_VA + g_slide);
    if (!rd32(setp, 0, &chunk)) return 0;
    while (chunk != 0 && n < 4096u) {
        unsigned int base = 0, lo = 0, hi = 0, nxt = 0;
        if (!plausible(chunk)) return 0;
        rd32(chunk, 0x0, &base);
        if ((key & ~63u) <= base) {
            if (base != (key & ~63u)) return 0;
            rd32(chunk, 0x4, &lo);
            rd32(chunk, 0x8, &hi);
            return ((lo & mask[(key & 63u) * 2u]) | (hi & mask[(key & 63u) * 2u + 1u])) != 0;
        }
        rd32(chunk, 0xc, &nxt);
        chunk = nxt;
        n++;
    }
    return 0;
}

/* An operand list as `{a b~ c}`: `rN`/`fN` for a kind-1 (physical) symbol,
 * `sK` for any other symbol by its per-function ordinal, `~` = last use. */
static void ap_oplist(unsigned int tup, unsigned int off)
{
    unsigned int op = 0, j = 0;
    ap("{");
    if (!rd32(tup, off, &op)) { ap("?}"); return; }
    while (op != 0 && j < OPS_MAX) {
        unsigned int nxt = 0, kind = 0, sym = 0, fl = 0, sk = 0;
        if (!plausible(op)) { ap(" ?"); break; }
        rd32(op, 0x0, &nxt);
        rd8(op, 0x8, &kind);
        rd8(op, 0x15, &fl);
        rd32(op, 0x1c, &sym);
        if (kind == 1u && plausible(sym)) {
            rd8(sym, 0x4, &sk);
            ap(" ");
            if (sk == 1u) {
                unsigned int rr = 0, n = 0;
                if (rd32(sym, 0x8, &rr) && rd32(rr, 0x1c, &n)) ap_regname(n); else ap("p?");
            } else {
                ap("s"); ap_dec(csym_ordinal(sym));
            }
            if (fl & 0x10u) ap("~");
        }
        j++;
        op = nxt;
    }
    ap(" }");
}

/* The class's order list and cursor, exactly as the selector picks them. */
static void ra_class(unsigned int nib, unsigned int *list, unsigned int *cur)
{
    if (nib == 5u && sreg(C2_RA_FPRFLAG) != 0u) {
        *list = C2_RA_LST_FPR + g_slide;
        *cur = sreg(C2_RA_CUR_FPR);
    } else if (nib == 5u || nib == 0xcu) {
        *list = sreg(C2_RA_LST_VMX);
        *cur = sreg(C2_RA_CUR_VMX);
    } else {
        *list = sreg(C2_RA_LST_GPR);
        *cur = sreg(C2_RA_CUR_GPR);
    }
}

static void ra_tuple(unsigned int tup)
{
    unsigned int t = g_rlast, n = 0, k = 0, opc = 0;
    /* Count the label tuples between the previous real tuple and this one:
     * each is a cursor reset in the driver loop. */
    if (g_rlast != 0) {
        rd32(t, 0x0, &t);
        while (t != 0 && t != tup && n < WALK_MAX) {
            if (!plausible(t)) break;
            rd8(t, 0x8, &k);
            if (k == 0x1au) g_rblk++;
            rd32(t, 0x0, &t);
            n++;
        }
    }
    g_rlast = tup;
    if (!(g_payload && g_regasg && g_show)) { g_rtup++; return; }
    rd32(tup, 0x4, &opc);
    ap("RT "); ap_dec(g_rtup);
    ap(" b ");   ap_dec(g_rblk);
    ap(" ");     ap_mnem(opc);
    ap(" d");    ap_oplist(tup, 0x2cu);
    ap(" u");    ap_oplist(tup, 0x28u);
    ap("\n");
    g_rtup++;
}

static void ra_select(const char *via, unsigned int op, unsigned int tup)
{
    unsigned int sym = 0, sk = 0, sid = 0, ty = 0, nib, pref = 0, list = 0, cur = 0, i;
    unsigned int occb = C2_RA_OCC_VA + g_slide, intfb = C2_RA_INTF_VA + g_slide;
    g_rcur_tup = tup;
    if (!(g_payload && g_regasg && g_show)) return;
    ap("RASG sel fn "); ap_dec(g_rfn);
    ap(" via ");  ap(via);
    if (!plausible(op)) { ap(" <unread>\n"); return; }
    rd16(op, 0xa, &ty);
    rd32(op, 0x1c, &sym);
    nib = ty >> 12;
    if (tup == g_rlast) { ap(" t "); ap_dec(g_rtup ? g_rtup - 1u : 0u); }
    else ap(" t ?");
    ap(" b ");    ap_dec(g_rblk);
    if (plausible(tup)) { unsigned int opc = 0; rd32(tup, 0x4, &opc); ap(" "); ap_mnem(opc); }
    ap(" sym ");
    if (plausible(sym)) {
        rd8(sym, 0x4, &sk);
        rd32(sym, 0x1c, &sid);
        rd32(sym, 0x30, &pref);
        ap("s"); ap_dec(csym_ordinal(sym));
        ap("/k"); ap_hex(sk, 2);
        ap(" sid "); ap_dec(sid);
    } else {
        ap("?");
    }
    ap(" cls "); ap_hex(nib, 1);
    ap(" pref ");
    if (pref != 0) {
        unsigned int n = 0;
        if (rd32(pref, 0x1c, &n)) ap_regname(n); else ap("?");
    } else {
        ap("-");
    }
    ra_class(nib, &list, &cur);
    ap(" cur ");
    if (cur != 0) {
        unsigned int n = *(const unsigned int *)(uintptr_t)cur;
        if (n == 0) ap("end"); else ap_regname(n);
    } else {
        ap("?");
    }
    /* The class list in ORDER: `x` free, `x*` occupied (=occupant), `x!`
     * free but interfering with this symbol. */
    ap(" list");
    if (!plausible_bytes(list)) { ap(" ?\n"); return; }
    for (i = 0; i < RA_LIST_MAX; i++) {
        unsigned int n = ((const unsigned int *)(uintptr_t)list)[i], occ, setp;
        if (n == 0 || n >= C2_NREGS) break;
        occ = ((const unsigned int *)(uintptr_t)occb)[n];
        setp = ((const unsigned int *)(uintptr_t)intfb)[n];
        ap(" "); ap_regname(n);
        if (occ != 0) { ap("*"); ap_occupant(occ); }
        else if (plausible(setp) && bitset_has(setp, sid)) ap("!");
    }
    ap("\n");
    if (g_arena_full) ap_reserved("REFUSE regasg arena-full\n");
}

static void ra_commit(unsigned int reg, unsigned int sym)
{
    if (!(g_payload && g_regasg && g_show)) return;
    ap("RASG asg fn "); ap_dec(g_rfn);
    ap(" reg ");  ap_regname(reg);
    ap(" sym ");
    if (plausible(sym)) { ap("s"); ap_dec(csym_ordinal(sym)); } else ap("?");
    ap("\n");
}

/* Called from the thunk. Returns the real call target for this site.
 *
 * MUST NOT call back into c2 and MUST NOT do I/O (see the header comment on
 * why the payload is buffered). Counting only, in this revision. */
unsigned int tap_enter(unsigned int retaddr, unsigned int ecx,
                       unsigned int edx, unsigned int callee_esp)
{
    int i;
    (void)edx;
    (void)callee_esp;
    for (i = 0; i < N_SITES; i++) {
        if (g_armed[i] && g_retaddr[i] == retaddr) {
            g_hits[i]++;
            /* PAYLOAD. Only the region site has a live TUPLE pointer in ecx —
             * at the six per-function phase sites ecx is the FUNCTION record
             * (0x10b7dc59 `mov esi,ecx`, then `mov ecx,esi` before each call),
             * and the function-record -> tuple-list-head offset is not known.
             * Taking the tuple pointer from the region finder's own argument
             * sidesteps that unknown entirely, which is why this site exists. */
            if (g_sites[i].name[0] == 'c' && strcmp(g_sites[i].name, "color") != 0) {
                /* A CANDIDATE site: never a phase, never a function count. */
                if (g_sites[i].name[1] == 'b') {
                    /* A BENEFIT site: bookkeeping only, flushed at `color`. */
                    if (g_payload && g_cands) {
                        const char *nm = g_sites[i].name;
                        if (strcmp(nm, "cbdef") == 0)      ben_note(1, ecx, edx);
                        else if (strcmp(nm, "cbuse") == 0) ben_note(0, ecx, edx);
                        else if (strcmp(nm, "cbdem") == 0) demote_note(i, ecx, edx, callee_esp);
                        else                               ben_snapshot();
                    }
                    return g_realtgt[i];
                }
                if (g_sites[i].name[1] == 'f') {
                    if (g_payload && g_cands) free_note(i, ecx);
                    return g_realtgt[i];
                }
                if (g_sites[i].name[1] == 'm') {
                    /* A MINT site: bookkeeping only, no payload line. */
                    if (g_payload && g_cands) mint_note(i, ecx);
                    return g_realtgt[i];
                }
                if (g_payload && g_cands && g_show) {
                    const char *nm = g_sites[i].name;
                    if (strcmp(nm, "csel") == 0)        ap_cand("sel", ecx, 1);
                    else if (strcmp(nm, "cnocol") == 0) ap_cand("nocol", ecx, 0);
                    else if (strcmp(nm, "cspill") == 0) ap_cand("spill", ecx, 0);
                    else if (strcmp(nm, "creq") == 0)   ap_cand("req", ecx, 0);
                    else if (strcmp(nm, "casg") == 0) {
                        unsigned int cand = 0;
                        /* The stack is c2's own live frame, so it is read
                         * directly: `plausible` is a HEAP filter and refuses
                         * wibo's high stack addresses. */
                        memcpy(&cand, (const void *)(uintptr_t)(callee_esp + 4u), 4);
                        ap_assigned(cand);
                    }
                    if (g_arena_full) ap_reserved("REFUSE cand arena-full\n");
                }
                return g_realtgt[i];
            }
            if (g_sites[i].name[0] == 'r' && g_sites[i].name[1] == 'a') {
                /* A REGASG site: never a phase, never a function count. */
                const char *nm = g_sites[i].name;
                if (strcmp(nm, "rafn") == 0) {
                    g_rfn++;
                    /* Share COLOR's symbol ordinals for the same function so
                     * a RASG `sN` and a CAND `sym N` name one symbol; reset
                     * only when no COLOR ran for this function (/Od). */
                    if (g_cfn == g_rfn_cfn) { g_ncsym = 0; g_show = name_matches(ecx); }
                    g_rfn_cfn = g_cfn;
                    g_rtup = 0; g_rblk = 0; g_rlast = 0; g_rcur_tup = 0;
                    if (g_payload && g_regasg && g_show) {
                        ap("RFN ");  ap_dec(g_rfn);
                        ap(" rr ");  ap_dec(sreg(C2_RA_RRFLAG));
                        ap(" fv ");  ap_dec(sreg(C2_RA_FPRFLAG));
                        ap_symid(ecx);
                        ap("\n");
                    }
                } else if (strcmp(nm, "rarerun") == 0) {
                    g_rtup = 0; g_rblk = 0; g_rlast = 0;
                    if (g_payload && g_regasg && g_show) ap("RFN-RERUN\n");
                } else if (strcmp(nm, "ratup") == 0) {
                    ra_tuple(ecx);
                } else if (strcmp(nm, "radef") == 0) {
                    ra_select("def", ecx, edx);
                } else if (strcmp(nm, "raevict") == 0) {
                    ra_select("rehome", ecx, edx);
                } else if (strcmp(nm, "rareload") == 0) {
                    ra_select("reload", ecx, edx);
                } else if (strcmp(nm, "raspill") == 0) {
                    if (g_payload && g_regasg && g_show) {
                        ap("RASG spill fn "); ap_dec(g_rfn); ap("\n");
                    }
                } else if (strcmp(nm, "racom") == 0) {
                    ra_commit(ecx, edx);
                }
                if (g_arena_full && g_payload && g_regasg) ap_reserved("REFUSE regasg arena-full\n");
                return g_realtgt[i];
            }
            if (strcmp(g_sites[i].name, "color") == 0) {
                g_cfn++;
                g_ncsym = 0;
                g_show = name_matches(ecx);
                if (g_payload && g_cands && g_show) {
                    unsigned int k;
                    ap("CFN ");  ap_dec(g_cfn);
                    ap_symid(ecx);
                    ap("\n");
                    for (k = 0; k < g_mlog_n; k++) {
                        ap("  ");
                        ap_cmint_x(k, g_mlog_id[k], g_mlog_site[k], g_mlog_reuse[k], g_mlog_sym[k],
                                   g_mlog_x[k], g_mlog_op[k]);
                    }
                    if (g_mlog_n >= MLOG_MAX) ap("  CMINT log-truncated\n");
                    ap_ben_log();
                }
                g_mlog_n = 0;
                g_in_color = 1;
            }
            if (strcmp(g_sites[i].name, "region") != 0) {
                g_phase = g_sites[i].name;
                if (strcmp(g_phase, "sched1") == 0) g_fn++;
                /* ecx is the FUNCTION RECORD at every one of these sites
                 * (`0x10b7dc59 mov esi,ecx` then `mov ecx,esi` before each
                 * call; `0x10b7e6b0 mov esi,ecx` then `0x10b7e6ff mov ecx,esi`
                 * at `after0`). */
                if (g_payload && g_walkfn) {
                    ap("FN ");   ap(g_phase);
                    ap(" fn ");  ap_dec(g_fn);
                    /* #3459: the ordinal alone is not an identity. */
                    ap_symid(ecx);
                    ap("\n");
                    tap_walk_function(ecx);
                    ap("END-FN\n");
                }
            } else if (g_payload) {
                ap("SITE region ENTER ");
                ap(g_phase);
                ap(" fn ");
                ap_dec(g_fn);
                ap(" r ");
                ap_dec(g_hits[i]);
                ap("\n");
                tap_walk_tuples(ecx);
                ap("END-REGION\n");
            }
            return g_realtgt[i];
        }
    }
    /* Unreachable by construction: only armed sites route here. Fail LOUD
     * rather than jump through a zero — a fault that degrades into an empty
     * snapshot is failure mode 4 arriving through the back door. */
    fprintf(stderr, "[stagetap] FATAL unknown return address %08x\n", retaddr);
    fflush(stderr);
    ExitProcess(97);
    return 0;
}

/* Case-insensitive-free, allocation-free comma-list membership. "all" matches
 * everything. */
static int wanted(const char *list, const char *name)
{
    size_t n = strlen(name);
    const char *p = list;
    if (strcmp(list, "all") == 0) return 1;
    while (*p) {
        const char *q = strchr(p, ',');
        size_t len = q ? (size_t)(q - p) : strlen(p);
        if (len == n && strncmp(p, name, n) == 0) return 1;
        if (!q) break;
        p = q + 1;
    }
    return 0;
}

int tap_arm(HMODULE h, void *invoke_fn)
{
    const char *list = getenv("C2RS_STAGE_TAPS");
    int i, armed = 0;
    unsigned int slide_export;
    unsigned int slide_vq = 0;
    int vq_ok = 0;
    MEMORY_BASIC_INFORMATION mbi;

    /* INERTNESS: unset means this function is the only thing that ran, and it
     * wrote nothing. */
    if (!list || !*list) return 0;

    if (!invoke_fn) {
        fprintf(stderr, "[stagetap] REFUSE all — no export pointer to derive "
                        "the slide from\n");
        return 0;
    }
    slide_export = (unsigned int)(uintptr_t)invoke_fn - C2_INVOKE_VA;

    /* SECOND DERIVATION of the same number (#3288). VirtualQuery's
     * AllocationBase for an address inside the image is the load base, so
     * base - 0x10b00000 must equal the export delta. Printed even when it
     * disagrees; only the export delta is USED, because it is the one whose
     * input is a fact from the whitebox record. */
    memset(&mbi, 0, sizeof(mbi));
    if (VirtualQuery(invoke_fn, &mbi, sizeof(mbi)) == sizeof(mbi)
        && mbi.AllocationBase) {
        slide_vq = (unsigned int)(uintptr_t)mbi.AllocationBase - C2_PREFERRED_BASE;
        vq_ok = 1;
    }

    {
        const char *pl = getenv("C2RS_STAGE_PAYLOAD");
        const char *rw = getenv("C2RS_STAGE_RAW");
        const char *ops = getenv("C2RS_STAGE_OPS");
        const char *fw = getenv("C2RS_STAGE_FUNCWALK");
        g_payload = (pl && *pl && *pl != '0');
        g_ops = (ops && *ops && *ops != '0');
        g_walkfn = (fw && *fw && *fw != '0');
        {
            const char *cd = getenv("C2RS_STAGE_CANDS");
            const char *ra = getenv("C2RS_STAGE_REGASG");
            const char *ff = getenv("C2RS_STAGE_FNFILTER");
            g_cands = (cd && *cd && *cd != '0');
            g_regasg = (ra && *ra && *ra != '0');
            g_fnfilter[0] = 0;
            if (ff && *ff) {
                size_t n = strlen(ff);
                if (n >= sizeof(g_fnfilter)) n = sizeof(g_fnfilter) - 1u;
                memcpy(g_fnfilter, ff, n);
                g_fnfilter[n] = 0;
            }
        }
        g_raw = 0;
        if (rw && *rw) {
            unsigned int v = 0;
            while (*rw >= '0' && *rw <= '9') { v = v * 10u + (unsigned int)(*rw++ - '0'); }
            if (v > 256u) v = 256u;
            g_raw = v;
        }
    }
    g_slide = slide_export;
    /* C2RS_STAGE_FORCE_SLIDE=<hex> — THE FAIL-CLOSED CHECK'S OWN TEST LEVER.
     *
     * Review finding (2026-08-20): the `+ slide` half of the fail-closed check
     * — the half the first plan defect was about — had never executed at a
     * nonzero slide, because c2.dll loads at its preferred base on every run on
     * this box. A guard nobody has watched fire is a guard nobody has tested.
     * Setting this variable displaces every site address by the given amount so
     * `crates/c2-reference/tests/stage.rs::a_wrong_slide_arms_nothing_and_never_
     * moves_the_obj` can watch all seven sites refuse AGAINST A LIVE IMAGE and
     * the obj come out byte-identical anyway.
     *
     * It is a testing lever and never a production path: unset, this block does
     * nothing, and when set the ONLY reachable outcome is refusal (the displaced
     * bytes are not this table's `e8 rel32`). */
    {
        const char *fs = getenv("C2RS_STAGE_FORCE_SLIDE");
        if (fs && *fs) {
            unsigned int v = 0;
            while (*fs) {
                char c = *fs++;
                if (c >= '0' && c <= '9')      v = v * 16u + (unsigned int)(c - '0');
                else if (c >= 'a' && c <= 'f') v = v * 16u + (unsigned int)(c - 'a' + 10);
                else if (c >= 'A' && c <= 'F') v = v * 16u + (unsigned int)(c - 'A' + 10);
                else break;
            }
            g_slide = slide_export + v;
            fprintf(stderr, "[stagetap] FORCED-SLIDE +%08x on top of the measured "
                            "%08x — every site MUST refuse\n", v, slide_export);
        }
    }
    g_slide_known = 1;
    fprintf(stderr, "[stagetap] hmodule=%08x invoke=%08x slide=%08x "
                    "slide-virtualquery=%s%08x\n",
            (unsigned int)(uintptr_t)h, (unsigned int)(uintptr_t)invoke_fn,
            g_slide, vq_ok ? "" : "unavailable:", slide_vq);
    if (vq_ok && slide_vq != slide_export) {
        fprintf(stderr, "[stagetap] WARN the two slide derivations disagree "
                        "(%08x vs %08x) — using the export delta\n",
                slide_export, slide_vq);
    }

    for (i = 0; i < N_SITES; i++) {
        unsigned char *p = (unsigned char *)(uintptr_t)(g_sites[i].site_va + g_slide);
        unsigned int   want = g_sites[i].target_va + g_slide;
        unsigned int   here;
        int            rel;
        DWORD          old = 0;
        int            newrel;

        if (!wanted(list, g_sites[i].name)) continue;

        /* FAIL-CLOSED CHECK 0 — the address is MAPPED before it is read.
         * Checks 1 and 2 both dereference the site; at a nonzero slide (a
         * relocated image, or the forced slide the fail-closed test uses) the
         * displaced address need not be inside any committed region, and a
         * fault there would be a crash instead of a refusal. */
        {
            MEMORY_BASIC_INFORMATION sm;
            memset(&sm, 0, sizeof(sm));
            if (VirtualQuery(p, &sm, sizeof(sm)) == sizeof(sm)
                && (sm.State != MEM_COMMIT || (sm.Protect & PAGE_NOACCESS))) {
                fprintf(stderr, "[stagetap] REFUSE %s site=%08x unmapped "
                                "(state=%lx protect=%lx) — never read a guess\n",
                        g_sites[i].name, (unsigned int)(uintptr_t)p,
                        (unsigned long)sm.State, (unsigned long)sm.Protect);
                continue;
            }
        }

        /* FAIL-CLOSED CHECK 1 — the site still starts with a direct call. */
        if (p[0] != 0xE8) {
            fprintf(stderr, "[stagetap] REFUSE %s site=%08x opcode=%02x "
                            "(expected e8) — image not the one this table "
                            "was read from\n",
                    g_sites[i].name, (unsigned int)(uintptr_t)p, p[0]);
            continue;
        }
        /* FAIL-CLOSED CHECK 2 — and it still calls what we think it calls,
         * AT THE MEASURED SLIDE. This is the proof the slide was handled. */
        memcpy(&rel, p + 1, 4);
        here = (unsigned int)(uintptr_t)(p + 5) + (unsigned int)rel;
        if (here != want) {
            fprintf(stderr, "[stagetap] REFUSE %s site=%08x target=%08x "
                            "expected=%08x — never patch a guess\n",
                    g_sites[i].name, (unsigned int)(uintptr_t)p, here, want);
            continue;
        }

        if (!VirtualProtect(p, 5, PAGE_EXECUTE_READWRITE, &old)) {
            fprintf(stderr, "[stagetap] REFUSE %s VirtualProtect failed "
                            "err=%lu\n", g_sites[i].name, GetLastError());
            continue;
        }
        newrel = (int)((unsigned int)(uintptr_t)&c2rs_stage_thunk
                       - ((unsigned int)(uintptr_t)p + 5));
        memcpy(p + 1, &newrel, 4);
        {
            DWORD back = 0;
            VirtualProtect(p, 5, old, &back);
        }
        /* No FlushInstructionCache. Two reasons, and the first is measured:
         * wibo does not implement it ("call reached missing import
         * FlushInstructionCache from kernel32" → SIGABRT, seen on this lane's
         * first successful VirtualProtect). The second is why that costs
         * nothing: x86 has a coherent instruction cache, the patched bytes are
         * written before c2 has ever executed them, and there is exactly one
         * thread. */

        g_armed[i]   = 1;
        g_retaddr[i] = (unsigned int)(uintptr_t)(p + 5);
        g_realtgt[i] = want;
        armed++;
        fprintf(stderr, "[stagetap] ARM %s site=%08x -> %08x (%s)\n",
                g_sites[i].name, (unsigned int)(uintptr_t)p, want,
                g_sites[i].what);
    }

    g_any_armed = armed > 0;
    fprintf(stderr, "[stagetap] armed=%d of %d requested-list=%s\n",
            armed, N_SITES, list);
    return armed;
}

void tap_report(void)
{
    int i;
    if (!g_slide_known) return;
    /* SCHEMA: no address, no pointer, no path, no PID, no timestamp and no
     * allocation count may appear on a TAP/SITE/END line. `slide` is printed
     * as 0-or-nonzero, never as a value, so a relocated image does not change
     * the canonical stream. */
    fprintf(stderr, "[stagetap] TAP 1 c80981c0 slide=%s\n",
            g_slide == 0 ? "0" : "nonzero");
    for (i = 0; i < N_SITES; i++) {
        if (!g_armed[i]) continue;
        fprintf(stderr, "[stagetap] END %s hits=%u\n",
                g_sites[i].name, g_hits[i]);
    }
    if (g_payload) {
        /* One `[stagetap] ` prefix per arena line, so the payload cannot be
         * confused with c2's own chatter on the same stream. */
        unsigned int i2 = 0, start = 0;
        for (i2 = 0; i2 < g_arena_len; i2++) {
            if (g_arena[i2] == '\n') {
                fputs("[stagetap] ", stderr);
                fwrite(&g_arena[start], 1, (size_t)(i2 - start + 1), stderr);
                start = i2 + 1;
            }
        }
        fprintf(stderr, "[stagetap] ARENA bytes=%u full=%d\n",
                g_arena_len, g_arena_full);
    }
    fflush(stderr);
}
