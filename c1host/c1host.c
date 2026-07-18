/* c1host — the front-end analogue of c2host.
 *
 * Drives `c1xx.dll` (the MSVC C++ *front end*) standalone under wibo, the same
 * way c2host drives `c2.dll` (the back end). cl.exe normally LoadLibrary's the
 * front end and calls its stdcall export to turn C++ source into the temp
 * `_CL_*` IL bundle (ex/gl/sy/in/db) that c2 then consumes. This stub does that
 * one call in isolation, so the bundle a captured pipeline produced can be
 * reproduced from source alone — the front-end replay oracle (P-F0.1).
 *
 * Two things differ from c2host, both mechanical:
 *   1. The export is the WIDE variant `_InvokeCompilerPassW@16` — c1xx/c1 do not
 *      export the narrow `_InvokeCompilerPass@12` that c2 does. So argv is
 *      converted to UTF-16 (CP_ACP) here.
 *   2. cl.exe reserves the compiler's heap arena itself and hands the base to
 *      the pass via `-zm<base>`; the pass only MEM_COMMITs inside it. Without
 *      that reservation c1xx dies `C1060 out of heap space`. We mirror cl by
 *      reserving at the `-zm` base before invoking.
 *
 * The pass also resolves `<host-exe-dir>/1033/clui.dll` (diagnostics resources)
 * via GetModuleFileNameW(NULL); the caller (ensure_c1host) places a `1033`
 * symlink next to this exe, and runs it with cwd = the toolchain dir so the
 * sibling DLLs (TLBREF.dll / mspdb*.dll) resolve.
 *
 *   usage: c1host <c1xx.dll> <argv0> [args...]
 *
 * Built on demand into a gitignored cache; the `.c` is tracked, the `.exe`
 * never is.
 */
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>

typedef int(__stdcall *InvokeW)(int argc, wchar_t **argv, int flag, void *block);

/* The pass writes a small context value into this block; cl.exe passes a
 * pointer to a zero-initialized global. A zeroed 64 KiB buffer suffices. */
static char block[0x10000];

/* Size cl.exe reserves for the -zm arena (0x4b00000 = 75 MiB at the default
 * /Zm scale — enough for the trivial front-end MVP class). */
#define ARENA_RESERVE 0x4b00000

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "usage: c1host <c1xx.dll> <argv0> [args...]\n");
        return 2;
    }
    HMODULE h = LoadLibraryA(argv[1]);
    if (!h) {
        fprintf(stderr, "[c1host] LoadLibrary(%s) failed err=%lu\n", argv[1], GetLastError());
        return 3;
    }
    InvokeW fn = (InvokeW)GetProcAddress(h, "_InvokeCompilerPassW@16");
    if (!fn) {
        fprintf(stderr, "[c1host] GetProcAddress(_InvokeCompilerPassW@16) failed err=%lu\n",
                GetLastError());
        return 4;
    }

    /* Reserve the pass heap arena at the base cl handed us via -zm<base>. */
    for (int i = 2; i < argc; i++) {
        if (argv[i][0] == '-' && argv[i][1] == 'z' && argv[i][2] == 'm') {
            unsigned long base = strtoul(argv[i] + 3, NULL, 0);
            if (base && !VirtualAlloc((void *)base, ARENA_RESERVE, MEM_RESERVE, PAGE_READWRITE)) {
                fprintf(stderr, "[c1host] arena reserve @0x%lx failed err=%lu\n",
                        base, GetLastError());
                return 5;
            }
        }
    }

    /* Convert the compiler argv (everything after this exe + the dll path) to
     * UTF-16 for the wide entry point. */
    int cargc = argc - 2;
    wchar_t **wargv = (wchar_t **)malloc(sizeof(wchar_t *) * (cargc + 1));
    if (!wargv) {
        fprintf(stderr, "[c1host] out of memory building wide argv\n");
        return 6;
    }
    for (int i = 0; i < cargc; i++) {
        const char *s = argv[i + 2];
        int n = MultiByteToWideChar(CP_ACP, 0, s, -1, NULL, 0);
        wargv[i] = (wchar_t *)malloc(sizeof(wchar_t) * n);
        MultiByteToWideChar(CP_ACP, 0, s, -1, wargv[i], n);
    }
    wargv[cargc] = NULL;

    return fn(cargc, wargv, 0, block);
}
