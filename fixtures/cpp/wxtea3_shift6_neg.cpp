// W-XTEA3 `_neg` — the round loop with `v >> 6` where the class requires
// `v >> 5`. One `rlwinm` field moves in each half-round (`srwi r7,r9,5` becomes
// `srwi r7,r9,6`, i.e. `5527d97e` becomes `5527d1be`), and the class emits the
// measured word rather than computing it from the source.
//
// `xtea_round_loop::SHR_K` is the clause under test, and it is FIXED rather than
// carried precisely because no cell separates it from the schedule: this file is
// the cell that says a body carrying a different shift must REFUSE instead of
// being emitted with the witnessed one. ONE refusing body in the TU.

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
        v1 += (v2 + (v2 << 4 ^ v2 >> 6)) ^ sum + key[sum & 3];
        sum += 0x9E3779B9;
        v2 += (v1 + (v1 << 4 ^ v1 >> 6)) ^ sum + key[(sum >> 11) & 3];
    }
    return (static_cast<unsigned long long>(v2) << 32)
         | (static_cast<unsigned long long>(v1) & 0xFFFFFFFF);
}
