// w-ifn — the three blocked `mmio.cpp` bodies, self-contained.
//
// `src/xdk/nuispeech/mmio.cpp` needs the dc3 tree's `xdk/nui/mmio.h` and its
// `win_types.h`; this file restates only what the three bodies touch, so the
// cell compiles from this repo.  The success condition is that the three
// `.text` COMDATs are BYTE-IDENTICAL to the ones in the workload obj
// (`work/w-ifn/ref/mmio.dis.txt`) — checked, not assumed.
//
// `extern "C"` because the real header wraps its declarations in it, and the
// obj's symbols are undecorated.  `__declspec(noinline)` on `mmioFlush` and
// `mmioSetBuffer` is in the original and is load-bearing for the elided-call
// question.

typedef void *HANDLE;
typedef HANDLE HMMIO;
typedef char *LPSTR;
typedef LPSTR HPSTR;
typedef long LRESULT;
typedef LRESULT MMRESULT;
typedef unsigned long DWORD;
typedef unsigned int UINT;
typedef long LONG;
typedef void *LPVOID;
typedef DWORD FOURCC;
typedef HANDLE HTASK;
typedef unsigned int uint;

extern "C" void *memcpy(void *, const void *, unsigned int);

extern "C" {

typedef LRESULT (*LPMMIOPROC)(LPVOID lpmmioinfo, UINT uMsg, LONG lParam1, LONG lParam2);

typedef struct _MMIOINFO {
    DWORD dwFlags;      // 0x00
    FOURCC fccIOProc;   // 0x04
    LPMMIOPROC pIOProc; // 0x08
    UINT wErrorRet;     // 0x0c
    HTASK hTask;        // 0x10
    LONG cchBuffer;     // 0x14
    HPSTR pchBuffer;    // 0x18
    HPSTR pchNext;      // 0x1c
    HPSTR pchEndRead;   // 0x20
    HPSTR pchEndWrite;  // 0x24
    LONG lBufOffset;    // 0x28
    LONG lDiskOffset;   // 0x2c
    DWORD adwInfo[4];   // 0x30
    DWORD dwReserved1;  // 0x40
    DWORD dwReserved2;  // 0x44
    HMMIO hmmio;        // 0x48
} MMIOINFO, *LPMMIOINFO;
typedef const LPMMIOINFO LPCMMIOINFO;

void FreeHandle(HANDLE);

MMRESULT mmioGetInfo(HMMIO hmmio, LPMMIOINFO pmmioinfo, UINT fuInfo) {
    if (hmmio == 0) {
        return 5;
    }
    if (pmmioinfo == 0) {
        return 11;
    }
    memcpy(pmmioinfo, hmmio, 0x48);
    return 0;
}

MMRESULT mmioSetInfo(HMMIO hmmio, LPCMMIOINFO pmmioinfo, UINT fuInfo) {
    if (hmmio == 0) {
        return 5;
    }
    if (pmmioinfo == 0) {
        return 11;
    }
    memcpy(hmmio, pmmioinfo, 0x48);
    LPMMIOINFO new_info = (LPMMIOINFO)hmmio;
    if (new_info->pchEndRead < new_info->pchNext) {
        new_info->pchEndRead = new_info->pchNext;
    }
    return 0;
}

__declspec(noinline) MMRESULT mmioFlush(HMMIO hmmio, UINT fuFlush) { return 0; }

__declspec(noinline) MMRESULT mmioSetBuffer(HMMIO hmmio, LPSTR pchBuffer, LONG cchBuffer,
                                            UINT fuBuffer) {
    return 0;
}

MMRESULT mmioClose(HMMIO hmmio, UINT fuClose) {
    if (hmmio == 0) {
        return 5;
    }
    uint flush_ret = mmioFlush(hmmio, 0);
    if (flush_ret != 0)
        return flush_ret;
    LPMMIOINFO info = (LPMMIOINFO)hmmio;
    uint proc_ret = info->pIOProc(info, 4, fuClose, 0);
    if (proc_ret != 0)
        return proc_ret;

    mmioSetBuffer(hmmio, 0, 0, 0);
    FreeHandle(hmmio);

    return 0;
}

}
