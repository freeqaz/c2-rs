// W-XTEA3 `_neg` — the framed block loop with `mNonce[i] += 1;` REMOVED. That
// statement is the one c2 folds into the loop's induction step: it is emitted as
// `stdu r11,8(r30)`, ONE word for the store AND the base's post-increment
// (wb-loop's update-form pass, which #1981 declines by name for the counted
// class). Without it the biased base has no reason to exist and the whole
// register plan changes.
//
// The clause under test is `xtea_encrypt_loop`'s second statement. ONE refusing
// body in the TU — `?Encipher` beside it is in class, which is what makes the
// refusal attributable: a TU verdict is a conjunction, so a `_neg` file holding
// SEVERAL refusing bodies can never go `mismatch` (`w-xtea2` #2664).

struct XteaBlockLike { unsigned long long mData[2]; };

class XteaLike {
private:
    unsigned long long mNonce[2];
    unsigned int mKey[4];
    unsigned long long Encipher(unsigned long long, unsigned int *);

public:
    void Encrypt(const XteaBlockLike *, XteaBlockLike *);
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

void XteaLike::Encrypt(const XteaBlockLike *in, XteaBlockLike *out) {
    unsigned int *key = mKey;
    unsigned long offset = (char *)out - (char *)in;
    for (int i = 0; i < 2; i++) {
        *(unsigned long long *)(offset + (char *)in) =
            *(unsigned long long *)in ^ Encipher(mNonce[i], key);
        in = (const XteaBlockLike *)((char *)in + 8);
    }
}
