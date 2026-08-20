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
 * ------------------------------------------------------------------------- */
#define ARENA_BYTES (4u * 1024u * 1024u)
static char         g_arena[ARENA_BYTES];
static unsigned int g_arena_len = 0;
static int          g_arena_full = 0;

static void ap(const char *s)
{
    while (*s) {
        if (g_arena_len >= ARENA_BYTES - 1) { g_arena_full = 1; return; }
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

static void tap_walk_tuples(unsigned int t)
{
    unsigned int i = 0;
    unsigned int first = t;
    if (!plausible(t)) {
        ap("REFUSE region walk-implausible-head\n");
        return;
    }
    while (i < WALK_MAX) {
        const unsigned char *b = (const unsigned char *)(uintptr_t)t;
        unsigned int opcode;
        unsigned int next;
        unsigned int d = (t > first) ? (t - first) : (first - t);
        if (d > WALK_SPAN) { ap("REFUSE region walk-span\n"); return; }

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
        if (g_arena_full) { ap("REFUSE region arena-full\n"); return; }

        i++;
        if (next == 0) return;               /* end of list: a real terminus */
        if (!plausible(next)) { ap("REFUSE region walk-implausible-next\n"); return; }
        t = next;
    }
    ap("REFUSE region walk-overrun\n");
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
            if (strcmp(g_sites[i].name, "region") != 0) {
                g_phase = g_sites[i].name;
                if (strcmp(g_phase, "sched1") == 0) g_fn++;
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
        g_payload = (pl && *pl && *pl != '0');
        g_raw = 0;
        if (rw && *rw) {
            unsigned int v = 0;
            while (*rw >= '0' && *rw <= '9') { v = v * 10u + (unsigned int)(*rw++ - '0'); }
            if (v > 256u) v = 256u;
            g_raw = v;
        }
    }
    g_slide = slide_export;
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
