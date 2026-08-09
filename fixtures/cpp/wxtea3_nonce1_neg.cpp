// W-XTEA3 `_neg` — the SAME body with ONE element instead of two, and c2
// EXCHANGES the two scratch registers:
//
//   two elements   ld r10,0(r4) · clrldi r11,r5,32 · add r10,r10,r11 · std r10
//   ONE  element   ld r11,0(r4) · clrldi r10,r5,32 · add r11,r10,r11 · std r11
//
// (`work/w-xtea3/probe/nonce.cpp`, cell `SetNonce1`, real c2.dll at `/O1 /Oi`.)
//
// So the register plan is a fact about a run of exactly TWO, and a port that
// emitted this body from the two-element class would put four wrong register
// fields into an obj that still links — board #263's shape.
//
// ONE refusing body in the whole TU, deliberately: a TU verdict is a
// conjunction, so a `_neg` file holding several refusing bodies can never go
// `mismatch` and grades nothing (`w-xtea2` #2664). Loosening
// `nonce_add_run::RUN_LEN`'s fence must turn this file into a live `mismatch`.

struct XteaLike {
    unsigned long long mNonce[2];
    unsigned int mKey[4];
    void SetNonce(const unsigned long long *nonce, unsigned int shift);
};

void XteaLike::SetNonce(const unsigned long long *nonce, unsigned int shift) {
    mNonce[0] = nonce[0] + shift;
}
