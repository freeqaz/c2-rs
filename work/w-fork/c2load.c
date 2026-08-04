/* Loads c2.dll and exits — isolates "process spawn + PE load + DLL init"
   from the compile itself. */
#include <windows.h>
#include <stdio.h>
int main(int argc, char **argv){
    if (argc < 2) return 2;
    HMODULE h = LoadLibraryA(argv[1]);
    if (!h) { fprintf(stderr,"LoadLibrary failed %lu\n", GetLastError()); return 3; }
    if (!GetProcAddress(h, "_InvokeCompilerPass@12")) return 4;
    return 0;
}
