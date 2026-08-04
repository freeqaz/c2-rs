/* c2forkd — fork-server guest stub for the real MSVC X360 backend.
 *
 * Same contract as c2host, but the expensive part happens once:
 *
 *   1. LoadLibraryA(c2.dll)  + GetProcAddress(_InvokeCompilerPass@12)
 *   2. WiboForkServe(...)    -- wibo extension; the PARENT never returns from
 *                               this, it forks once per request. We return here
 *                               only in a freshly-forked CHILD, holding that
 *                               request's argv and already chdir'd.
 *   3. InvokeCompilerPass(argv) and ExitProcess with its return code.
 *
 * If WiboForkServe is absent (stock wibo) or returns 0 ($WIBO_FORK_SOCKET
 * unset) this falls back to c2host's behaviour: compile the argv given on the
 * command line, once. That fallback is what keeps the stub honest — it is the
 * *same* real c2.dll doing the *same* real work in both arms of the benchmark.
 *
 * usage: wibo c2forkd.exe <c2.dll> [<arg0> <c2 argv...>]
 */
#include <windows.h>
#include <stdio.h>

typedef int(__stdcall *InvokeFn)(int argc, char **argv, int unk);
typedef DWORD(WINAPI *ForkServeFn)(char *buf, DWORD n);

#define MAXARGS 512
static char reqbuf[262144];
static char *cargv[MAXARGS];

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: c2forkd <c2.dll> [<argv0> <c2 args...>]\n");
        return 2;
    }
    HMODULE h = LoadLibraryA(argv[1]);
    if (!h) {
        fprintf(stderr, "[c2forkd] LoadLibrary(%s) failed err=%lu\n", argv[1], GetLastError());
        return 3;
    }
    InvokeFn fn = (InvokeFn)GetProcAddress(h, "_InvokeCompilerPass@12");
    if (!fn) fn = (InvokeFn)GetProcAddress(h, "InvokeCompilerPass");
    if (!fn) fn = (InvokeFn)GetProcAddress(h, (LPCSTR)(uintptr_t)3);
    if (!fn) {
        fprintf(stderr, "[c2forkd] GetProcAddress failed err=%lu\n", GetLastError());
        return 4;
    }

    /* Gate on the environment, NOT on GetProcAddress returning NULL: stock wibo
     * answers an unknown kernel32 name with a trampoline that aborts the process
     * when called ("call reached missing import WiboForkServe from kernel32"),
     * so a NULL check is not a portability test and costs a core dump. */
    DWORD n = 0;
    char sockcheck[8];
    if (GetEnvironmentVariableA("WIBO_FORK_SOCKET", sockcheck, sizeof(sockcheck)) > 0 ||
        GetLastError() == ERROR_INSUFFICIENT_BUFFER) {
        HMODULE k32 = GetModuleHandleA("kernel32.dll");
        ForkServeFn serve = k32 ? (ForkServeFn)GetProcAddress(k32, "WiboForkServe") : NULL;
        if (serve) {
            n = serve(reqbuf, (DWORD)sizeof(reqbuf));
        }
    }

    int cargc;
    if (n > 0) {
        /* CHILD of the fork server: reqbuf holds NUL-separated argv. */
        cargc = 0;
        DWORD off = 0;
        while (off < n && cargc < MAXARGS - 1) {
            cargv[cargc++] = reqbuf + off;
            while (off < n && reqbuf[off] != '\0') off++;
            off++;
        }
        cargv[cargc] = NULL;
    } else {
        /* Stock-wibo / no-socket fallback: exactly c2host's behaviour. */
        if (argc < 4) {
            fprintf(stderr, "[c2forkd] no fork socket and no inline argv\n");
            return 2;
        }
        cargc = argc - 2;
        for (int i = 0; i < cargc && i < MAXARGS - 1; i++) cargv[i] = argv[2 + i];
        cargv[cargc] = NULL;
    }

    int rc = fn(cargc, cargv, 0);
    ExitProcess((UINT)rc);
    return rc;
}
