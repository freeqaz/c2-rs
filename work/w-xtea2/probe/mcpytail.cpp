// w-xtea2 probe — the shape of `?SetKey@XTEABlockEncrypter`: a member function
// whose WHOLE body is one `memcpy` into a member array, lowered by c2 as a TAIL
// BRANCH to `memcpy` rather than a framed `bl`.
//
// The cells vary one thing each so the emitter's three words can be attributed:
//   the destination member's offset, the length, and which formal is the source.
extern "C" void *memcpy(void *, const void *, unsigned long);

struct A {
    unsigned long long n[2];
    unsigned int k[4];
    void off16(const unsigned char *uc);
    void len8(const unsigned char *uc);
    void off0(const unsigned char *uc);
    void rev(unsigned char *uc);
};

// THE SUBJECT — byte-for-byte `?SetKey@XTEABlockEncrypter`'s shape.
void A::off16(const unsigned char *uc) { memcpy(k, uc, 0x10); }

// Same, a different length: does the `li` move and nothing else?
void A::len8(const unsigned char *uc) { memcpy(k, uc, 0x8); }

// Destination at offset 0: is the `addi` emitted at all?
void A::off0(const unsigned char *uc) { memcpy(n, uc, 0x10); }

// The direction REVERSED — the member is the source. r3 must become the formal
// and r4 the member address, which is not the subject's register plan.
void A::rev(unsigned char *uc) { memcpy(uc, k, 0x10); }

// A free function with two pointer formals, no `this`: the same call with a
// different argument origin.
void freefn(unsigned char *d, const unsigned char *s) { memcpy(d, s, 0x10); }
