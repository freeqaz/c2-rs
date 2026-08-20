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

/* Called from the thunk. Returns the real call target for this site.
 *
 * MUST NOT call back into c2 and MUST NOT do I/O (see the header comment on
 * why the payload is buffered). Counting only, in this revision. */
unsigned int tap_enter(unsigned int retaddr, unsigned int ecx,
                       unsigned int edx, unsigned int callee_esp)
{
    int i;
    (void)ecx;
    (void)edx;
    (void)callee_esp;
    for (i = 0; i < N_SITES; i++) {
        if (g_armed[i] && g_retaddr[i] == retaddr) {
            g_hits[i]++;
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
    fflush(stderr);
}
