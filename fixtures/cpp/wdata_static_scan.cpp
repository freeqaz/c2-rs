// **W-DATA — the static-array scan loop.** The port's first body whose
// translation unit **defines the data its own code reads**.
//
// `src/system/math/Primes.cpp` is `p0`'s program with a 62-element array, and it
// is a FRONTIER TU: its one emitted function is this shape, so it converts on
// this class or on none. This file is the class's FENCE — the workload contains
// exactly one instance and one instance cannot tell a rule from a coincidence
// (board #260).
//
//     lis   r10,a@ha      REFHI  (+PAIR)
//     li    r11,0         j = 0
//     addi  r9,r10,a@l    REFLO  (+PAIR)      r9 = &a
//     lwz   r10,a@l(r10)  REFLO  (+PAIR)      r10 = a[0]   <- the SECOND low half
//     b     .+24          the ROTATION: jump INTO the bottom test
//     cmpw  cr6,r10,r3      <- LOOP TOP
//     bf    24,.+28       cr6.LT false -> the value-return block
//     addi  r11,r11,1     j++
//     slwi  r10,r11,2
//     lwzx  r10,r10,r9    r10 = a[j]
//     cmpwi cr6,r10,0       <- the bottom test, the `b` lands here
//     bf    26,.-24       BACK EDGE to the loop top
//     blr                 fall-out: return i, already in r3
//     slwi  r11,r11,2       <- REMATERIALIZED: r10 already held this
//     lwzx  r3,r11,r9
//     blr
//
// **Sixteen words with ZERO free immediate fields.** Every other transcription
// this port ships has at least two (`ptr_walk_loop`'s `K0` and `K`,
// `if_call_join`'s two literals); this one has none. So nothing in the emitted
// `.text` can vary across the class, and everything that *does* vary is the
// OBJECT: its symbol name, its size, its alignment and its bytes. The three
// cells here vary exactly those, and `work/w-data/GRID.md` froze what each must
// produce before any of them was compiled.
//
// ---- the two things that must NOT change a byte -----------------------------
//
// `p2` differs from `p0` in the function name, the array name and every array
// value, and its `.text` must be **byte-identical**. If it ever is not, the
// class has a field the emitter is not modelling.
//
// ---- and the one that must ---------------------------------------------------
//
// `p1` is the only cell that crosses `coff::placement_align`'s 64-byte
// promotion. Its `.data` is 256 bytes and takes **ALIGN_8** where `p0`'s
// 32-byte one takes ALIGN_4 — read off c2's own obj, and the reason
// `Primes.cpp`'s 248-byte array is `0xC0401040` and not `0xC0301040`. Without
// this cell the whole class is graded on one side of that boundary.
//
// The relocation shape is the other thing one instance could not have told:
// **one REFHI and TWO REFLOs** against one symbol, because c2 materialises the
// high half once and spends it twice. A 1:1 carrier emits five records where c2
// emits six — see `coff::DataDef`.

// p0 — the dc3 body's program, small array (32 B, ALIGN_4).
int p0(int i) {
    static int a[8] = { 3, 5, 7, 11, 13, 17, 19, 0 };

    for (int j = 0; a[j] != 0; j++) {
        if (a[j] >= i)
            return a[j];
    }

    return i;
}

// p1 — 256 B, across the promotion boundary. `.text` must equal p0's.
int p1(int i) {
    static int b[64] = {
        3,    5,    7,    11,   13,   17,   19,   23,   29,   31,   37,   41,
        43,   47,   53,   59,   61,   67,   71,   73,   79,   83,   89,   97,
        101,  103,  107,  109,  113,  127,  131,  137,  139,  149,  151,  157,
        163,  167,  173,  179,  181,  191,  193,  197,  199,  211,  223,  227,
        229,  233,  239,  241,  251,  257,  263,  269,  271,  277,  281,  283,
        293,  307,  311,  0
    };

    for (int j = 0; b[j] != 0; j++) {
        if (b[j] >= i)
            return b[j];
    }

    return i;
}

// p2 — p0 with every name and every value changed. `.text` must equal p0's.
int p2(int k) {
    static int table[8] = { 100, 200, 300, 400, 500, 600, 700, 0 };

    for (int m = 0; table[m] != 0; m++) {
        if (table[m] >= k)
            return table[m];
    }

    return k;
}
