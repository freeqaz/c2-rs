// w-xtea3 — the `?Encrypt` class, isolated: a framed `__savegprlr_26` body
// whose loop calls a same-TU callee and whose back edge is `addic.`/`bf 2`
// rather than CTR (the call clobbers CTR's usefulness).

struct Blk { unsigned long long mData[2]; };

struct Enc {
    unsigned long long mNonce[2];
    unsigned int mKey[4];
    unsigned long long Encipher(unsigned long long, unsigned int *);
    void Encrypt(const Blk *in, Blk *out);
    void Encrypt4(const Blk *in, Blk *out);
};

struct EncOff {
    unsigned int mPad[4];
    unsigned long long mNonce[2];
    unsigned int mKey[4];
    unsigned long long Encipher(unsigned long long, unsigned int *);
    void Encrypt(const Blk *in, Blk *out);
};

unsigned long long Enc::Encipher(unsigned long long nonce, unsigned int *key) {
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

// THE TARGET, verbatim.
void Enc::Encrypt(const Blk *in, Blk *out) {
    unsigned int *key = mKey;
    unsigned long offset = (char *)out - (char *)in;
    for (int i = 0; i < 2; i++) {
        *(unsigned long long *)(offset + (char *)in) =
            *(unsigned long long *)in ^ Encipher(mNonce[i], key);
        mNonce[i] += 1;
        in = (const Blk *)((char *)in + 8);
    }
}

// FOUR trips instead of two: does anything but the `li r29` immediate move?
void Enc::Encrypt4(const Blk *in, Blk *out) {
    unsigned int *key = mKey;
    unsigned long offset = (char *)out - (char *)in;
    for (int i = 0; i < 4; i++) {
        *(unsigned long long *)(offset + (char *)in) =
            *(unsigned long long *)in ^ Encipher(mNonce[i], key);
        mNonce[i] += 1;
        in = (const Blk *)((char *)in + 8);
    }
}

unsigned long long EncOff::Encipher(unsigned long long nonce, unsigned int *key) {
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

// Both members moved: mNonce at 16, mKey at 32.
void EncOff::Encrypt(const Blk *in, Blk *out) {
    unsigned int *key = mKey;
    unsigned long offset = (char *)out - (char *)in;
    for (int i = 0; i < 2; i++) {
        *(unsigned long long *)(offset + (char *)in) =
            *(unsigned long long *)in ^ Encipher(mNonce[i], key);
        mNonce[i] += 1;
        in = (const Blk *)((char *)in + 8);
    }
}
