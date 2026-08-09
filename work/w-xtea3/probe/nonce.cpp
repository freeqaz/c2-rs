// w-xtea3 — the `?SetNonce` class, isolated. One TU per cell would be ideal but
// the register plan is what is under test and it is per-function, so the cells
// live together and each is read off its own `.text` COMDAT.
//
// Compiled through real cl.exe/c2.dll under wibo at `/O1 /Oi /GS- /c` and again
// at `/Ox /GS- /c`; every claim in `crates/c2-il/.../nonce_add_run.rs` and
// `crates/c2-core/.../nonce_add_run.rs` is read off these objs.

struct Enc {
    unsigned long long mNonce[2];
    unsigned int mKey[4];
    void SetNonce(const unsigned long long *nonce, unsigned int shift);
    void SetNonceRev(const unsigned long long *nonce, unsigned int shift);
    void SetNonce1(const unsigned long long *nonce, unsigned int shift);
    void SetNonce3(const unsigned long long *nonce, unsigned int shift);
    void SetNonceU64(const unsigned long long *nonce, unsigned long long shift);
};

struct EncOff {
    unsigned int mPad[2];
    unsigned long long mNonce[2];
    void SetNonce(const unsigned long long *nonce, unsigned int shift);
};

// THE TARGET, verbatim.
void Enc::SetNonce(const unsigned long long *nonce, unsigned int shift) {
    mNonce[0] = nonce[0] + shift;
    mNonce[1] = nonce[1] + shift;
}

// The addend on the LEFT of the `+`, which is the same arithmetic and may or may
// not be the same register plan.
void Enc::SetNonceRev(const unsigned long long *nonce, unsigned int shift) {
    mNonce[0] = shift + nonce[0];
    mNonce[1] = shift + nonce[1];
}

// ONE element: does the second statement's `add 11,10,11` exist because it is
// the last use, or is it an unconditional alternation?
void Enc::SetNonce1(const unsigned long long *nonce, unsigned int shift) {
    mNonce[0] = nonce[0] + shift;
}

// THREE elements — out of the class as written, and the cell that says what the
// allocator does past two.
void Enc::SetNonce3(const unsigned long long *nonce, unsigned int shift) {
    mNonce[0] = nonce[0] + shift;
    mNonce[1] = nonce[1] + shift;
    mKey[0] = (unsigned int)(nonce[0] + shift);
}

// A 64-bit addend: no `clrldi` at all, because nothing needs zero-extending.
void Enc::SetNonceU64(const unsigned long long *nonce, unsigned long long shift) {
    mNonce[0] = nonce[0] + shift;
    mNonce[1] = nonce[1] + shift;
}

// A NON-ZERO member offset: the destination's designator sums to 8 and 16.
void EncOff::SetNonce(const unsigned long long *nonce, unsigned int shift) {
    mNonce[0] = nonce[0] + shift;
    mNonce[1] = nonce[1] + shift;
}
