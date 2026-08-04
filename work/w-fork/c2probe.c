/* c2probe — lane w-fork's per-TU-state probe.
 *
 * Question: does c2.dll capture per-TU state (cwd, TMP/TEMP, MSC_CMD_FLAGS,
 * INCLUDE/LIB) at LoadLibrary/DllMain time?  If it does, a fork-server that
 * forks *after* LoadLibrary cannot vary that state per compilation and the
 * fork point has to move earlier (which is where the win lives).
 *
 * Three modes, all reaching the SAME state at the moment InvokeCompilerPass
 * is called, differing only in when that state was established:
 *
 *   early  : real state set BEFORE LoadLibrary, untouched after.   (baseline)
 *   late   : DECOY state before LoadLibrary, real state set AFTER. (fork-server
 *            semantics: the DLL initialised under somebody else's environment)
 *   never  : DECOY state before LoadLibrary and never corrected.   (control —
 *            if this also matches, the probe has no power over these variables
 *            and that is itself the finding)
 *
 * usage: c2probe <early|late|never> <decoy_dir> <real_dir> <c2.dll> <argv0> [c2 args...]
 */
#include <windows.h>
#include <stdio.h>
#include <string.h>

typedef int(__stdcall *InvokeFn)(int argc, char **argv, int unk);

static void set_state(const char *dir, const char *tag) {
    SetCurrentDirectoryA(dir);
    SetEnvironmentVariableA("TMP", dir);
    SetEnvironmentVariableA("TEMP", dir);
    SetEnvironmentVariableA("MSC_CMD_FLAGS", tag);
    SetEnvironmentVariableA("MSC_IDE_FLAGS", tag);
    SetEnvironmentVariableA("INCLUDE", dir);
    SetEnvironmentVariableA("LIB", dir);
    SetEnvironmentVariableA("LIBPATH", dir);
    SetEnvironmentVariableA("_CL_", tag);
}

int main(int argc, char **argv) {
    if (argc < 6) {
        fprintf(stderr, "usage: c2probe <early|late|never> <decoy> <real> <c2.dll> <argv0> [args...]\n");
        return 2;
    }
    const char *mode = argv[1];
    const char *decoy = argv[2];
    const char *real = argv[3];
    const char *dll = argv[4];

    int is_early = strcmp(mode, "early") == 0;
    int is_late = strcmp(mode, "late") == 0;
    int is_never = strcmp(mode, "never") == 0;
    int is_reverse = strcmp(mode, "reverse") == 0;
    if (!is_early && !is_late && !is_never && !is_reverse) {
        fprintf(stderr, "[c2probe] bad mode %s\n", mode);
        return 2;
    }

    if (is_early || is_reverse)
        set_state(real, "");
    else
        set_state(decoy, "-DECOY-");

    HMODULE h = LoadLibraryA(dll);
    if (!h) {
        fprintf(stderr, "[c2probe] LoadLibrary(%s) failed err=%lu\n", dll, GetLastError());
        return 3;
    }

    if (is_late)
        set_state(real, "");
    if (is_reverse)
        set_state(decoy, "-DECOY-");

    InvokeFn fn = (InvokeFn)GetProcAddress(h, "_InvokeCompilerPass@12");
    if (!fn)
        fn = (InvokeFn)GetProcAddress(h, "InvokeCompilerPass");
    if (!fn)
        fn = (InvokeFn)GetProcAddress(h, (LPCSTR)(uintptr_t)3);
    if (!fn) {
        fprintf(stderr, "[c2probe] GetProcAddress failed err=%lu\n", GetLastError());
        return 4;
    }

    int rc = fn(argc - 5, &argv[5], 0);
    fprintf(stderr, "[c2probe] mode=%s returned %d\n", mode, rc);
    return rc;
}
