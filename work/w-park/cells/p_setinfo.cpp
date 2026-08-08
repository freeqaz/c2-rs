extern "C" void *memcpy(void *, const void *, unsigned int);
struct MI { char pad[28]; char *pchNext; char *pchEndRead; };
unsigned long p_setinfo(void *hmmio, const void *pmmioinfo, unsigned int fuInfo) {
    if (hmmio == 0) return 5;
    if (pmmioinfo == 0) return 11;
    memcpy(hmmio, pmmioinfo, 0x48);
    MI *ni = (MI *)hmmio;
    if (ni->pchEndRead < ni->pchNext) ni->pchEndRead = ni->pchNext;
    return 0;
}
