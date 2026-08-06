P = "docs/rungs/2026-08-06-w-fnbyte.md"
s = open(P, encoding="utf-8").read()
anchor = "### 6.1 Mutations — 31 total, 31 RED, 0 green"
assert s.count(anchor) == 1
new = """### 6.0 The oracle's own verdict, applied to the NEW population

`fnbyte-match-tu-differs 0` is a *negative* control — no newly-graded body
contradicts a certified obj. The **positive** form is stronger and this lane can
state it, because the widening moved functions *into* that population:

On the **10 TUs the differential graded `match`**, the per-TU FBM buckets read

```
plain/exact 4 · cond-pair/exact 3 · parse-refused/refused 2      (denominator 9)
```

**Three `cond-pair` bodies on `match` TUs were `Partial` at the baseline and are
`Exact` now.** Those are functions inside objs the *sole judge* has already
certified byte-identical to real c2's, so their correct answer was known in
advance — and the reconstruction produced it, word for word, from the port's own
emitter. The `whole_tu` credit falling **5 → 2** is exactly these three moving
from "credited on the oracle's whole-obj verdict" to "credited on their own
bytes"; the residual 2 are `TomCryptLicense.cpp` and `ZlibLicense.cpp`, the
`??__E` whole-TU route that `select_function` never sees
(`FUNCTION_BYTE_MATCH.md` §5, board #323). Nothing else changed on those TUs.

"""
s = s.replace(anchor, new + anchor, 1)
open(P, "w", encoding="utf-8").write(s)
print("control 6.0 added")
