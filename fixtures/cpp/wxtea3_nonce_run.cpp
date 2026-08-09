// W-XTEA3 — the two-element 64-bit member run whose addend is a zero-extended
// 32-bit formal. `?SetNonce@XTEABlockEncrypter@@QAAXPB_KI@Z` is the workload
// instance (`src/system/utl/EncryptXTEA.cpp`, 32 bytes, one of that TU's three
// remaining blocked bodies after `w-xtea2`).
//
//   ld     r10,0(r4)      clrldi r11,r5,32     add r10,r10,r11   std r10,0(r3)
//   ld     r10,8(r4)      add    r11,r10,r11   std r11,8(r3)     blr
//
// What this fixture grades that no earlier one can:
//
//  * **A COMMON SUBEXPRESSION ACROSS TWO STATEMENTS.** The `clrldi` is emitted
//    ONCE, before the first `add`, and the second statement reads its result.
//    A per-statement lowering of the same two source lines is one word long in
//    an obj that still links.
//  * **TWO SCRATCH REGISTERS THAT ARE NOT SYMMETRIC.** The first `add` targets
//    r10 and the second r11, because r11 is live across the first statement and
//    dead after the second. `wxtea3_nonce1_neg.cpp` is the same body with ONE
//    element, where c2 EXCHANGES the two registers — which is why the run
//    length is a constant of the class and not a parameter.
//  * **The first 64-bit rotate/mask word this port emits.** `encode_rldicl`,
//    which board #2344 recorded as missing from `c2-core` entirely.
//
// At `/Ox` this file is deliberately out of class: c2 emits eight words there
// too, six of them identical, and the second `add` targets **r9**. The mode gate
// lives in the PARSER (board #1638) and in the emitter, and
// `scripts/mode_lane.sh` compiles every fixture at both.

struct XteaLike {
    unsigned long long mNonce[2];
    unsigned int mKey[4];
    void SetNonce(const unsigned long long *nonce, unsigned int shift);
};

void XteaLike::SetNonce(const unsigned long long *nonce, unsigned int shift) {
    mNonce[0] = nonce[0] + shift;
    mNonce[1] = nonce[1] + shift;
}
