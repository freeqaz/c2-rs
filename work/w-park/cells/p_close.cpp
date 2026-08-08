typedef unsigned int uint;
struct MI2;
typedef unsigned long (*IOPROC)(MI2 *, unsigned int, unsigned int, unsigned int);
struct MI2 { char pad[8]; IOPROC pIOProc; };
void FreeHandle2(void *);
__declspec(noinline) unsigned long p_flush(void *h, unsigned int f) { return 0; }
__declspec(noinline) unsigned long p_setbuf(void *h, char *b, long c, unsigned int f) { return 0; }
unsigned long p_close(void *hmmio, unsigned int fuClose) {
    if (hmmio == 0) return 5;
    uint flush_ret = p_flush(hmmio, 0);
    if (flush_ret != 0) return flush_ret;
    MI2 *info = (MI2 *)hmmio;
    uint proc_ret = info->pIOProc(info, 4, fuClose, 0);
    if (proc_ret != 0) return proc_ret;
    p_setbuf(hmmio, 0, 0, 0);
    FreeHandle2(hmmio);
    return 0;
}
