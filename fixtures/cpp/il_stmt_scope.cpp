// **Lexical scopes** — `53` opens one, `54 <k>` closes the one at depth `k`.
//
// This was `body-0x53`, the largest single blocking feature on the real dc3
// workload at 170,401 functions, and it is pure decode: c2 register-allocates
// across braces and emits nothing at one, so every body here lowers exactly as its
// brace-free equivalent does. Clearing it took the bucket to 0 and moved those
// functions on to their next blocker, of which 865 turned out to be fully in class.
//
// `54 <k>`'s operand is the **depth of the scope being closed**, not a count of
// anything, and two of the bodies below are what pin that reading:
//
//   nested    { x…  { …return } }   plumbing closes `54 03 54 02`
//   two_blocks{ {x…} {…return} }    first block closes `54 03`, then REOPENS at 3
//
// Under "k = scopes still open after the pop" the second one would have to number
// its two blocks differently, and it does not. The function body itself is depth 2
// — the same numbering `.sy` uses, where 1 is the formals scope — and the segment
// tail's own `47 54 01 54 00` is the identical scheme two levels further out, which
// is the third witness.
//
// Because the depth is tracked rather than skipped, the return plumbing can require
// the close run to descend exactly from the depth the statement parse counted. That
// is the difference between checking the nesting and merely tolerating it: a body
// whose braces do not balance refuses (`return-scope-close`) instead of being read
// as some shorter body that happens to parse.
//
// Deliberately out of class still, and each refuses cleanly: a scope containing a
// declaration of non-`int` type (the `.sy` reader will not vouch for it), and any
// scope introduced by control flow rather than a bare brace — an `if` or a loop
// opens a scope too, but it also emits `29` label definitions and `38`/`39`/`3A`
// branches, which is the next layer and needs codegen, not just decode.
//
// `deep` is here because the close run is a loop over depths: at three levels the
// plumbing closes `54 05 54 04 54 03 54 02`, which no two-level body can
// distinguish from a hardcoded pair.
//
// **The braces are on their own lines on purpose.** Each close is preceded by its
// own `4F 01 <line>` marker — the source line of the `}` — so the run is
// `4F 01 <l> 54 05 4F 01 <l> 54 04 …`. A probe written one function per line emits
// no intervening markers at all, parses with a single marker consumed ahead of the
// run, and hides the whole thing; that is exactly what happened while this was
// being written, and reformatting the same seven functions turned 3/3 in class into
// 3/7. Source layout changes the IL, so a fixture that is formatted unlike real
// code is testing something real code does not do.

int braced(int a) {
    {
        return a + 1;
    }
}

int nested(int a) {
    int x = a + 1;
    {
        int y = x + 2;
        return y;
    }
}

int two_blocks(int a) {
    {
        int x = a + 1;
    }
    {
        int y = a + 2;
        return y;
    }
}

int deep(int a) {
    {
        int x = a + 1;
        {
            int y = x + 2;
            {
                return y + 3;
            }
        }
    }
}

int close_then_return(int a) {
    {
        int x = a + 1;
    }
    return a + 2;
}

int scope_after_scope(int a, int b) {
    {
        int x = a + b;
    }
    {
        int y = a + 1;
    }
    return a + b;
}

int empty_scope(int a) {
    {
    }
    return a + 1;
}
