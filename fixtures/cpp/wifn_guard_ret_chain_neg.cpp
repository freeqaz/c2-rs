// W-IFN — the separating controls for `guard_ret_chain`. Every body here is
// ONE clause away from `wifn_guard_ret_chain.cpp` and every one must REFUSE, at
// every mode, in the READER.
//
// The point of the file is not that it refuses — a file of nonsense would do
// that. It is that each cell trips a **different** named clause, so the fence is
// exercised rather than merely present. `work/w-ifn/NEG_CLAUSES.md` records the
// key each cell actually reached, read off `c2rs census` at the workload's own
// flags and not predicted.
//
// Compile at the workload's profile like its positive twin; at `/Ox` the whole
// file is out of class on the mode gate alone, which is a second, coarser
// refusal and not the one these cells are for.

typedef unsigned int uint;

extern "C" void *memcpy(void *, const void *, unsigned int);

struct wifn_ninfo {
    unsigned pad[7];
    char *next;
    char *end_read;
    unsigned tail[9];
};

// ---- n1: the first guard's operand is an INT, not a pointer ---------------
// c2 emits `cmpwi` here and `cmplwi` in the positive — one word apart, and it
// links either way, which is why the reader pins the operand's type.
extern "C" long wifn_n1(uint h, wifn_ninfo *p, uint flags) {
    if (h == 0) {
        return 5;
    }
    if (p == 0) {
        return 11;
    }
    memcpy(p, &h, 0x48);
    return 0;
}

// ---- n2: the copy is BELOW the measured expansion step ---------------------
// 25 cells put the boundary at n = 6; at 4 c2 expands the copy into loads and
// stores and emits no call at all.
extern "C" long wifn_n2(void *h, wifn_ninfo *p, uint flags) {
    if (h == 0) {
        return 5;
    }
    if (p == 0) {
        return 11;
    }
    memcpy(p, h, 4);
    return 0;
}

// ---- n3: the guards test formal 1 and then formal 0 ------------------------
// Which formal a guard tests decides which register its compare reads once the
// park has run, so the order is not a presentation detail.
extern "C" long wifn_n3(void *h, wifn_ninfo *p, uint flags) {
    if (p == 0) {
        return 11;
    }
    if (h == 0) {
        return 5;
    }
    memcpy(p, h, 0x48);
    return 0;
}

// ---- n4: THREE guards ------------------------------------------------------
// A third arm is one more four-word block in source order and this class has
// never been graded on one.
extern "C" long wifn_n4(void *h, wifn_ninfo *p, void *q) {
    if (h == 0) {
        return 5;
    }
    if (p == 0) {
        return 11;
    }
    if (q == 0) {
        return 13;
    }
    memcpy(p, h, 0x48);
    return 0;
}

// ---- n5: TWO formals, not three -------------------------------------------
// A different arity moves the argument registers.
extern "C" long wifn_n5(void *h, wifn_ninfo *p) {
    if (h == 0) {
        return 5;
    }
    if (p == 0) {
        return 11;
    }
    memcpy(p, h, 0x48);
    return 0;
}

// ---- n6: the first guard is not against null -------------------------------
// `cmplwi cr6,rX,1` is a different immediate and, more to the point, a
// different program.
extern "C" long wifn_n6(void *h, wifn_ninfo *p, uint flags) {
    if (h == (void *)4) {
        return 5;
    }
    if (p == 0) {
        return 11;
    }
    memcpy(p, h, 0x48);
    return 0;
}

// ---- n7: the copy's destination is the THIRD formal ------------------------
// Neither of the two graded register plans covers it: formal 2 arrives in r5
// and no witness moves it anywhere.
extern "C" long wifn_n7(void *h, wifn_ninfo *p, wifn_ninfo *q) {
    if (h == 0) {
        return 5;
    }
    if (p == 0) {
        return 11;
    }
    memcpy(q, h, 0x48);
    return 0;
}

// ---- n8: the tail returns a non-zero literal -------------------------------
// The emitter's last word before the epilogue is a hard `li r3,0`.
extern "C" long wifn_n8(void *h, wifn_ninfo *p, uint flags) {
    if (h == 0) {
        return 5;
    }
    if (p == 0) {
        return 11;
    }
    memcpy(p, h, 0x48);
    return 7;
}

// ---- n9: the clamp stores a member the test did not read -------------------
// Sub-shape S's `lwz`/`lwz`/`cmplw`/`stw` quartet is driven from ONE pair of
// offsets; a third member is a plan this class does not have.
extern "C" long wifn_n9(void *h, wifn_ninfo *p, uint flags) {
    if (h == 0) {
        return 5;
    }
    if (p == 0) {
        return 11;
    }
    memcpy(h, p, 0x48);
    wifn_ninfo *ni = (wifn_ninfo *)h;
    if (ni->end_read < ni->next) {
        ni->next = ni->end_read;
    }
    return 0;
}

// ---- n10: the guard arm's literal is wider than a `li` immediate -----------
extern "C" long wifn_n10(void *h, wifn_ninfo *p, uint flags) {
    if (h == 0) {
        return 70000;
    }
    if (p == 0) {
        return 11;
    }
    memcpy(p, h, 0x48);
    return 0;
}
