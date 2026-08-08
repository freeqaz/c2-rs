// PROBE CELL — a transliteration of `src/xdk/nuispeech/mmio.cpp`'s
// `mmioGetInfo`, the smallest of that TU's three blocked bodies (84 B).
// Measurement only; never a gate fixture unless promoted.
extern "C" void *memcpy(void *, const void *, unsigned int);

typedef void *HMMIO;
typedef void *LPMMIOINFO;
typedef unsigned int UINT;
typedef unsigned long MMRESULT;

MMRESULT p_getinfo(HMMIO hmmio, LPMMIOINFO pmmioinfo, UINT fuInfo) {
    if (hmmio == 0) {
        return 5;
    }
    if (pmmioinfo == 0) {
        return 11;
    }
    memcpy(pmmioinfo, hmmio, 0x48);
    return 0;
}
