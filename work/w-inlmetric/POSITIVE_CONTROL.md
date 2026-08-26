# The conformance grader, watched FAILING

`work/w-inlmetric/check_table.py`'s green is quoted in this lane's rung. PREREG
§6 forbids quoting it until the grader has been watched taking a failure, so
three wrong verdicts were planted — one per check — and reverted.

| plant | what was changed | which check must catch it |
|---|---|---|
| 1 | `C14`'s `absent` flipped to `R-derived` with witness `splice.rs:INLINE_MAX_DEPTH`, a token that does not exist | WITNESS |
| 2 | `C8`'s address moved `10b5fc8a` → `10b5fe14` — **`w-sizebracket`'s exact defect, re-injected** | ADDRESS |
| 3 | `C7`'s `none:DAT_10c46318` changed to `none:INLINE_UNBOUNDED_BYTES`, a token that IS in `crates/` | ABSENCE |

Observed, one run, all three planted at once:

```
  FAIL C7: state absent but token 'INLINE_UNBOUNDED_BYTES' IS PRESENT in crates/
  FAIL C8: ADDRESS 0x10b5fe14 is in FUN_10b5fcd8, table claims FUN_10b5fb5f
  FAIL C14: WITNESS 'INLINE_MAX_DEPTH' NOT FOUND in crates/c2-core/src/splice.rs

CONFORMANCE-CHECK: RED  (3 failure(s) over 24 rows)
```

Reverted, same command:

```
CONFORMANCE-CHECK: GREEN  (0 failure(s) over 24 rows)
```

**Each check caught exactly its own plant and no other**, so the three are
independent and a green is not one check masking two. Plant 2 is the one worth
keeping: `P_INLINE.md` §2.1's CORRECTION block is that comparison done by hand,
once, after four addresses had already been published in the wrong function.
It is now a program that runs over every row.
