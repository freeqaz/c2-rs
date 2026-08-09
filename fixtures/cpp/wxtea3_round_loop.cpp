// W-XTEA3 — the XTEA round loop. `?Encipher@XTEABlockEncrypter@@AAA_K_KPAI@Z`
// is the workload instance (`src/system/utl/EncryptXTEA.cpp`, 116 bytes / 29
// words), and `w-xtea` #2339 priced it at ">= 9 mechanisms".
//
// What this fixture grades that no earlier one can:
//
//  * **A SOFTWARE-PIPELINED loop body.** The `addis`/`addi` pair that
//    materialises 0x9E3779B9 is split AROUND an `xor` that does not depend on
//    it, and the second half-round's index word is hoisted above the first
//    half's last use of r11. No pass in this port derives that order; the class
//    is a TRANSCRIPTION and says so in its own module header.
//  * **An indexed load INSIDE a CTR loop.** #1981 defines `counted_accum_loop`
//    to contain no memory reference and declines the update-form pass by name,
//    so this is a new class rather than a widening of that one.
//  * **`rldicl` and `rldimi`** — two of the five encoders board #2344 recorded
//    as missing from `c2-core` entirely.
//  * **The first LEAF class with a non-zero `label_lead`.** Every other term in
//    that sum belongs to a framed class, where the lead moves the function's own
//    `$M` triple; this one has no triple at all and what its `+2` moves is every
//    later function's labels. `work/w-xtea2/LABGRID.txt` row `x-encipher`,
//    stride 3, taken in `LABEL_COUNTER.md` §7.6's in-the-middle form.
//
// At `/Ox` this file is deliberately out of class: the same source is 1,352
// bytes there with a `__savegprlr_28` frame, six relocations and the loop fully
// unrolled. The mode gate lives in the PARSER (board #1638) and in the emitter.

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
        sum += 0x9E3779B9;
        v2 += (v1 + (v1 << 4 ^ v1 >> 5)) ^ sum + key[(sum >> 11) & 3];
    }
    return (static_cast<unsigned long long>(v2) << 32)
         | (static_cast<unsigned long long>(v1) & 0xFFFFFFFF);
}
