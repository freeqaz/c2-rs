// w-xtea2 probe — the FOUR `_neg` cells, one per shipping clause of
// `try_parse_memcpy_tail`, compiled through real c2 so that each is verified to
// be a DIFFERENT BODY before it is claimed as a refusal.
extern "C" void *memcpy(void *, const void *, unsigned long);

struct P {
    unsigned char a[16]; // 0x00
    unsigned char b[16]; // 0x10
    unsigned long long n[2];
    unsigned char big[64];   // 0x30
    unsigned char big2[64];  // 0x70
};

// N1 — the direction reversed: the member is the SOURCE.
void n1(P *p, unsigned char *out) { memcpy(out, p->b, 0x10); }

// N2 — a length below the measured call window: c2 expands it inline.
void n2(P *p, const unsigned char *s) { memcpy(p->b, s, 0x4); }

// N3 — the SOURCE carries a member offset: `addi r4,r4,16`.  The length is
// 0x40 and not 0x10 because at 0x10 c2 EXPANDS the copy (two ld/std pairs, 24 B,
// no call at all) and the cell would never reach the recognizer — a vacuous
// `_neg`, which is the shape w-pool2 found in a predecessor's grid.
void n3(P *d, const P *s) { memcpy(d->big, s->big2, 0x40); }

// N4 — a SECOND statement after the copy: the body no longer ends at the call.
void n4(P *p, const unsigned char *s) {
    memcpy(p->b, s, 0x10);
    p->n[0] = 0;
}
