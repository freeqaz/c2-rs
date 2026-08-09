// W-XTEA3 `_neg` — the round loop with the `sum` update REMOVED. c2 emits a
// DIFFERENT BODY throughout, not the same body one statement shorter:
//
//   with `sum +=`      116 B, 29 words, the key load INSIDE the loop (`lwzx`)
//   without it          84 B, 21 words, the key load HOISTED to the prologue
//                       (`lwz r9,0(r5)`), the trip-count register moved and the
//                       schedule collapsed
//
// (`work/w-xtea3/probe/enc.cpp`, cell `EncipherNoSum`, real c2.dll at `/O1 /Oi`.)
//
// So the round body is NOT composable from its two halves, and the clause under
// test is the `sum += <delta>` statement `xtea_round_loop` requires between
// them. ONE refusing body in the TU: a TU verdict is a conjunction and a
// multi-cell `_neg` file can never go `mismatch` (`w-xtea2` #2664).

struct XteaLike {
    unsigned long long mNonce[2];
    unsigned int mKey[4];
    unsigned long long Encipher(unsigned long long nonce, unsigned int *key);
};

unsigned long long XteaLike::Encipher(unsigned long long nonce, unsigned int *key) {
    unsigned long v1 = nonce & 0xFFFFFFFF;
    unsigned long v2 = nonce >> 32;
    unsigned int sum = 0;
    for (int i = 0; i < 4; i++) {
        v1 += (v2 + (v2 << 4 ^ v2 >> 5)) ^ sum + key[sum & 3];
        v2 += (v1 + (v1 << 4 ^ v1 >> 5)) ^ sum + key[(sum >> 11) & 3];
    }
    return (static_cast<unsigned long long>(v2) << 32)
         | (static_cast<unsigned long long>(v1) & 0xFFFFFFFF);
}
