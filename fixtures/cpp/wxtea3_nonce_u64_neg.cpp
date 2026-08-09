// W-XTEA3 `_neg` — the same two-element run with a **64-bit** addend, which
// needs no zero-extension at all:
//
//   unsigned int       ld r10,0(r4) · clrldi r11,r5,32 · add r10,r10,r11 · std r10
//                      ld r10,8(r4) · add r11,r10,r11  · std r11
//   unsigned long long ld r11,0(r4) · add r11,r11,r5   · std r11
//                      ld r11,8(r4) · add r11,r11,r5   · std r11
//
// (`work/w-xtea3/probe/nonce.cpp`, cell `SetNonceU64`.) Seven words against
// eight, no `clrldi`, and a third register plan throughout — so the
// zero-extension is minted by the ADDEND's width and not by the store's.
//
// The clause under test is `nonce_add_run::eat_widen8`, which requires the `2C`
// conversion the 32-bit addend carries and the 64-bit one does not. ONE
// refusing body in the TU.

struct XteaLike {
    unsigned long long mNonce[2];
    unsigned int mKey[4];
    void SetNonce(const unsigned long long *nonce, unsigned long long shift);
};

void XteaLike::SetNonce(const unsigned long long *nonce, unsigned long long shift) {
    mNonce[0] = nonce[0] + shift;
    mNonce[1] = nonce[1] + shift;
}
