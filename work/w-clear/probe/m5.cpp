// M5 = ?mmioGetInfo verbatim, with the xdk typedefs spelled out.
extern "C" void *memcpy(void *, const void *, unsigned);
typedef unsigned MMRESULT;
typedef void *HMMIO;
struct MMIOINFO { unsigned pad[18]; };
typedef MMIOINFO *LPMMIOINFO;
typedef unsigned UINT;
MMRESULT mmioGetInfo(HMMIO hmmio, LPMMIOINFO pmmioinfo, UINT fuInfo) {
    if (hmmio == 0) return 5;
    if (pmmioinfo == 0) return 11;
    memcpy(pmmioinfo, hmmio, 0x48);
    return 0;
}
