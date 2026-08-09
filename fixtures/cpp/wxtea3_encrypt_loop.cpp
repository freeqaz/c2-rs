// W-XTEA3 — the framed XTEA block loop, and the workload TU
// `src/system/utl/EncryptXTEA.cpp` less one body. FOUR bodies, four classes: the
// store run, the two-element 64-bit member run, the XTEA round loop, and the
// framed block loop that calls it.
//
// **`?SetKey` is deliberately absent**, and its absence is a mode fact rather
// than a scope choice: `w-xtea2`'s `memcpy` tail branch exists only at
// `/O1 /Oi`, because at plain `/O1` the copy arrives in the IL as a `26` callee
// push instead of the `40` intrinsic — a DIFFERENT STREAM, not a refused one.
// With it here this file would be `vocab-gap` at every lane but the four `/Oi`
// ones, and the four classes below would be graded by none of them. The
// five-body composition is graded where it actually lives: the workload TU
// itself, in the 878-TU scan.
//
// What this fixture grades that no earlier one can:
//
//  * **A `__savegprlr_N` frame WITH an IL-named callee.** `w-xlr` and `w-json`
//    are the port's only other helper-framed classes; the first has two IL-named
//    callees and no defined one, the second has none at all. This is the first
//    with a frame helper PAIR and a callee this same obj DEFINES — three REL24
//    sites of two different kinds in one function.
//  * **THE INLINE FENCE'S LOOP CLAUSE.** With every body byte-exact the workload
//    TU still refused, because `?Encipher` is 116 emitted bytes against
//    `INLINE_DECLINE_BYTES`' 128 and the fence read "the port cannot prove c2
//    kept this call". `WB_INLINE_FINDINGS` §7's MAY table licenses the answer —
//    "a loop-bodied callee > 80 bytes is never inlined at /O1", F9 + the anchor,
//    62 cells — and this file is the only cell that grades it.
//  * **THE WHOLE LABEL TRIPLE OF A MULTI-FUNCTION TU.** Here the charge is
//    `1 + 1 + 3 + 4 = 9` slots ahead of the framed function's own `$M`, of which
//    SIX come from the two `label_lead` terms this lane added (`+2` for the
//    round loop, `+4` for the framed one). The workload TU's own charge is 11 —
//    the same nine plus `?SetKey`'s leaf-and-`memcpy` pair. A wrong count here
//    is six wrong bytes in an obj that still links.
//  * **`stdu`, `stdx` and `addic.`** — the last three of the five encoders
//    board #2344 recorded as missing, and #2567 as unpaid.
//
// At `/Ox` this file is deliberately out of class: c2 inlines `?Encipher` into
// `?Encrypt` there and the round loop is fully unrolled. Every mode gate lives
// in the PARSER (board #1638) and is re-asked in the emitter.

struct XteaBlockLike { unsigned long long mData[2]; };

class XteaLike {
private:
    unsigned long long mNonce[2];
    unsigned int mKey[4];
    unsigned long long Encipher(unsigned long long, unsigned int *);

public:
    XteaLike();
    void SetNonce(const unsigned long long *, unsigned int);
    void Encrypt(const XteaBlockLike *, XteaBlockLike *);
};

XteaLike::XteaLike() {
    mNonce[0] = 0;
    mNonce[1] = 0;
}

void XteaLike::SetNonce(const unsigned long long *nonce, unsigned int shift) {
    mNonce[0] = nonce[0] + shift;
    mNonce[1] = nonce[1] + shift;
}

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
        mNonce[i] += 1;
        in = (const XteaBlockLike *)((char *)in + 8);
    }
}
