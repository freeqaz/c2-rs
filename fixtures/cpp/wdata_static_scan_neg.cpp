// **W-DATA's negative cells.** Six programs one axis away from
// `wdata_static_scan.cpp`'s `p0`, each of which must come back
// `NotImplemented` and **never** `Mismatch` — a refusal becoming a wrong emit is
// the one direction the correctness rule forbids (board #232).
//
// **Every cell is braced exactly like the positive file** and every cell keeps
// `p0`'s arity, relation, stride and element type except the one thing it
// varies. w-cfgclass's §6.2 was bitten by a `_neg` file whose cells failed for a
// clause an earlier cell already held, and the file looked as complete before
// the fix as after — `c2rs census` reports only the fall-through blocker, so two
// cells tripping one gate is invisible from outside. The check here is
// mechanical: the recognizer's clause names are distinct strings and
// `work/w-data/GRID.md` names which one each cell must produce.
//
//   n0  namespace-scope `static`   -> non-COMDAT      resolve_data_def, !comdat
//   n1  uninitialized              -> .bss COMDAT     resolve_data_def, !initialized
//   n2  `short` elements           -> scale 2         scan-test-subscript
//   n3  `>` instead of `>=`        -> relation byte   scan-guard-not-ge
//   n4  two formals                -> arity           scan-formals-not-1
//   n5  index starts at 1          -> init literal    scan-index-init-not-zero
//
// **n0 and n1 refuse programs c2 emits perfectly well**, and that is recorded
// rather than left to be discovered. A namespace-scope `static` is a
// **non-COMDAT** `.data` placed *before* `.text` (lane w-cfg2's GRID A cell
// `a4`, board #1682) and an uninitialized function-local `static` is a `.bss`
// COMDAT (cell `a3`); this lane graded a writer for neither. The fence is
// narrower than the class, which is the safe direction, and widening either
// needs its own graded cells — not a relaxed clause.

// n0 — the array is at namespace scope. Non-COMDAT `.data`, placed BEFORE
// `.text`, which is a different section order and a different obj.
static int n0a[8] = { 3, 5, 7, 11, 13, 17, 19, 0 };

int n0(int i) {
    for (int j = 0; n0a[j] != 0; j++) {
        if (n0a[j] >= i)
            return n0a[j];
    }

    return i;
}

// n1 — the function-local static is UNINITIALIZED, so it is a `.bss` COMDAT.
int n1(int i) {
    static int a[8];

    for (int j = 0; a[j] != 0; j++) {
        if (a[j] >= i)
            return a[j];
    }

    return i;
}

// n2 — `short` elements. The subscript's scale is 2 and the load is `lhz`, so
// two words of the sixteen would have to change and the emitter has a field for
// neither.
int n2(int i) {
    static short a[8] = { 3, 5, 7, 11, 13, 17, 19, 0 };

    for (int j = 0; a[j] != 0; j++) {
        if (a[j] >= i)
            return a[j];
    }

    return i;
}

// n3 — `>` in the guard. A different relation byte, and the emitted `bf` reads a
// different CR bit.
int n3(int i) {
    static int a[8] = { 3, 5, 7, 11, 13, 17, 19, 0 };

    for (int j = 0; a[j] != 0; j++) {
        if (a[j] > i)
            return a[j];
    }

    return i;
}

// n4 — two formals. Everything about the register plan is measured at arity 1,
// where `return i` emits no instruction at all because the formal is already in
// r3.
int n4(int i, int lo) {
    static int a[8] = { 3, 5, 7, 11, 13, 17, 19, 0 };

    for (int j = lo; a[j] != 0; j++) {
        if (a[j] >= i)
            return a[j];
    }

    return i;
}

// n5 — the index starts at 1. `li r11,0` is a literal word in the emitter.
int n5(int i) {
    static int a[8] = { 3, 5, 7, 11, 13, 17, 19, 0 };

    for (int j = 1; a[j] != 0; j++) {
        if (a[j] >= i)
            return a[j];
    }

    return i;
}
