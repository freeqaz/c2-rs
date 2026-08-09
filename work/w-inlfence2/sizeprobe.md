# w-inlfence2 — finding a leaf the port lowers to more than 64 bytes

The first `big_leaf` was an `a + 1 + 2 + ... + 20` constant chain and it lowered
to **8 bytes**. That is `WB_INLINE_FINDINGS` §3.1's own failure, verbatim:

> v1's ladder was `a = a*3 + i` repeated *k* times. **c2 folds the whole chain
> to two words at every k**, so the size axis did not occur: 159 cells that all
> measured the same 28-byte callee.

The port folds it too, so the confound guard in `n2` fired and said so instead
of the cell passing on a callee that was actually small. That guard is the only
reason this was caught — an `n2` written without it would have been a green
cell testing the clause its positive twin tests.

The replacement is `a + a + a + …`, which is opaque to constant folding for the
same reason WB_INLINE's rebuild used `a += tbl[i]`: there is no constant to
fold. The size is **asserted in the test**, not assumed.
