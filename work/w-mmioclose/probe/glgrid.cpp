// w-mmioclose — THE ATTRIBUTE GRID.  Eight functions that differ ONLY in the
// declaration attribute, so whichever `.gl` byte moves is the attribute and
// nothing else.  Every body is `return 0`, so the `.ex` segments are identical
// and no body feature can be confounded with the flag.
//
// Prediction, frozen before the capture: `g_noinl` differs from `g_plain` in
// exactly one bit, and it is **bit 6 (0x40)** — `WB_INLINE_FINDINGS.md` §1 read
// the legality test at `0x10b5c06b` as *"requires bit 6 of `[sym+0x4c]`"* off
// c2's own disassembly, and this is that field arriving from the other side.

int g_plain(int a) { return 0; }
__declspec(noinline) int g_noinl(int a) { return 0; }
inline int g_inl(int a) { return 0; }
__forceinline int g_finl(int a) { return 0; }
static int g_static(int a) { return 0; }
static __declspec(noinline) int g_static_noinl(int a) { return 0; }
__declspec(noinline) int g_noinl_body(int a) { return a * 3 + 1; }
int g_plain_body(int a) { return a * 3 + 1; }

// Keep every one of them referenced so none is dropped, and keep the callers
// out of the way: their own records are in the dump too and must not be
// mistaken for the callees'.
int use_all(int a) {
    return g_plain(a) + g_noinl(a) + g_inl(a) + g_finl(a) + g_static(a) +
           g_static_noinl(a) + g_noinl_body(a) + g_plain_body(a);
}
