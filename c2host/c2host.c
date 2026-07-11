#include <windows.h>
#include <stdio.h>
typedef int (__stdcall *InvokeFn)(int argc, char **argv, int unk);
int main(int argc, char **argv){
    if (argc < 3){ fprintf(stderr,"usage: c2host <c2.dll> <arg0> [args...]\n"); return 2; }
    HMODULE h = LoadLibraryA(argv[1]);
    if (!h){ fprintf(stderr,"[c2host] LoadLibrary(%s) failed err=%lu\n", argv[1], GetLastError()); return 3; }
    InvokeFn fn = (InvokeFn)GetProcAddress(h, "_InvokeCompilerPass@12");
    if (!fn) fn = (InvokeFn)GetProcAddress(h, "InvokeCompilerPass");
    if (!fn) fn = (InvokeFn)GetProcAddress(h, (LPCSTR)(uintptr_t)3); /* ordinal 3 */
    if (!fn){ fprintf(stderr,"[c2host] GetProcAddress failed err=%lu\n", GetLastError()); return 4; }
    int cargc = argc - 2;
    char **cargv = &argv[2];
    fprintf(stderr,"[c2host] InvokeCompilerPass @%p argc=%d argv0=%s\n", (void*)fn, cargc, cargv[0]);
    int rc = fn(cargc, cargv, 0);
    fprintf(stderr,"[c2host] returned %d\n", rc);
    return rc;
}
