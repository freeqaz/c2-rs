// W8, the VALUE-returning arm: `return g(...)` where the void form of
// w8_cond_tail.cpp has `g(...); return;`. This is the other half of
// `src/xdk/nuispeech/xboxmem.cpp` — `?MemSize`, whose `.text` payload is the
// same 36 bytes as `?MemFree`'s up to its two relocation targets.
//
// The IL differs in the arm's terminator and in nothing else:
//
//     void  arm:  … 4C 4B                        discard, end statement
//     value arm:  … 4C [2C <T> 00] 41 <T>        convert, result type
//
// and neither spelling changes an emitted byte — the value is already in r3 and
// the `2C` is an int-width conversion. `fs` forces the `2C` by returning a
// wider-spelled type than its callees do (`?MemSize` gets one for the same
// reason); `fp` has matching types and gets none. Both must emit the identical
// branch layout, which is what makes this fixture a control on the terminator
// rather than a second copy of w8_cond_tail.cpp.
int gi2(void *, unsigned long);
int hi3(void *, unsigned long, void *);
void *gp2(void *, unsigned long);
void *hp3(void *, unsigned long, void *);

unsigned long fs(void *v1, void *v2, unsigned long ul) {
    if (v1 == 0) {
        return gi2(v2, ul);
    }
    return hi3(v1, 0, v2);
}

void *fp(void *v1, void *v2, unsigned long ul) {
    if (v1 == 0) {
        return gp2(v2, ul);
    }
    return hp3(v1, 0, v2);
}
