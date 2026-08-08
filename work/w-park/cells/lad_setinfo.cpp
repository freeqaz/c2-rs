// CONSTRUCT LADDER — from the class this lane's widening reaches to
// `mmio.cpp`'s `mmioSetInfo`, one construct per rung. Board #401's method.
void c3(void *, void *, unsigned int);
extern "C" void *memcpy(void *, const void *, unsigned int);
struct MI { char pad[28]; char *pchNext; char *pchEndRead; };

// S0 — the head: two guards, a three-slot call with one literal, formals in
// place. In class at this lane's tip.
unsigned long S0(void *a0, const void *a1, unsigned int a2) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    c3(a0, (void *)a1, 0x48);
    return 0;
}

// S1 — S0 with the POST-CALL conditional member store. `SeqTail` has three
// forms (Void, Lit, Cmp) and this is none of them: two member loads, a compare
// on the loaded values and a store, all AFTER the call.
unsigned long S1(void *a0, const void *a1, unsigned int a2) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    c3(a0, (void *)a1, 0x48);
    MI *ni = (MI *)a0;
    if (ni->pchEndRead < ni->pchNext) ni->pchEndRead = ni->pchNext;
    return 0;
}

// S1b — S1 without the conditional, to separate "a member store after the call"
// from "a BRANCH after the call".
unsigned long S1b(void *a0, const void *a1, unsigned int a2) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    c3(a0, (void *)a1, 0x48);
    MI *ni = (MI *)a0;
    ni->pchEndRead = ni->pchNext;
    return 0;
}

// S1c — S0 with the formal merely READ after the call, no store: the smallest
// thing that makes a0 live across the call and therefore Class B (`std r31`).
unsigned long S1c(void *a0, const void *a1, unsigned int a2) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    c3(a0, (void *)a1, 0x48);
    MI *ni = (MI *)a0;
    return (unsigned long)ni->pchNext;
}

// S2 — S1 with the callee named `memcpy`: `mmioSetInfo` itself.
unsigned long S2(void *a0, const void *a1, unsigned int a2) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    memcpy(a0, a1, 0x48);
    MI *ni = (MI *)a0;
    if (ni->pchEndRead < ni->pchNext) ni->pchEndRead = ni->pchNext;
    return 0;
}
