# W-IFN — which clause each `_neg` cell actually trips, READ and not predicted

`fixtures/cpp/wifn_guard_ret_chain_neg.cpp` holds ten cells, each one clause
away from the positive fixture. The point of the file is not that it refuses —
a file of nonsense would do that — but that each cell reaches a **different**
named clause, so the fence is exercised rather than merely present.

`c2rs census` cannot answer this: the production is non-committal, so a body
that declines is reported under the *dispatch arm's* blocker (`expr-cmp-eq` for
all ten) and the clause is invisible. The keys below were read off a **scratch
print at the production's dispatch site**, behind `C2RS_GRET`, committed
nowhere and reverted after this file was written — `w-blockir`'s method, board
#2087. Its full text is `work/w-ifn/scratch.patch`.

```
$ C2RS_GRET=1 c2rs census fixtures/cpp/wifn_guard_ret_chain_neg.cpp \
      --flags-file work/w-ifn/o1flags.txt
```

| cell | the one clause it is away from the positive | key reached |
|---|---|---|
| `wifn_n1` | the first guard's operand is an `int`, not a pointer | `gret-guard-operand-not-a-pointer` |
| `wifn_n2` | the copy is 4 bytes — below the measured expansion step | `gret-copy-length-outside-the-call-window` |
| `wifn_n3` | the guards test formal 1 then formal 0 | `gret-guards-are-not-formals-0-then-1` |
| `wifn_n4` | THREE guards | `gret-copy-selector` |
| `wifn_n5` | TWO formals, not three | `gret-not-three-formals-free-fn` |
| `wifn_n6` | the first guard is against `4`, not null | `gret-guard-not-against-null` |
| `wifn_n7` | the copy's destination is the THIRD formal | `gret-copy-formals-are-not-a-graded-pair` |
| `wifn_n8` | the tail returns `7`, not `0` | `gret-tail-is-not-return-zero` |
| `wifn_n9` | the clamp stores the member the test READ, not the one it tested into | `gret-clamp-store-is-not-the-tested-pair` |
| `wifn_n10` | the guard arm returns `70000`, wider than a `li` immediate | `gret-arm-literal-wider-than-simm16` |

**Ten cells, ten distinct keys, and one of them is not the key the cell was
written for.** `wifn_n4`'s third guard does not reach a "too many guards"
clause at all: the second guard parses, and the *copy* parser then meets the
third `if`'s `53` where it wants the intrinsic's selector literal. That is the
production refusing correctly for a reason one clause earlier than intended,
and it is recorded rather than tidied — the emitter carries its own
`a_third_guard_is_refused` test for the clause the fixture does not reach, so
both are graded, in two places, by two instruments.

## The mode gate, separately

All ten cells (and both positive bodies) are additionally out of class at
`/Ox` on `gret-not-o1`, which is asked FIRST and before any body byte (board
#1638). That is a second, coarser refusal and not the one these cells are for;
`scripts/mode_lane.sh` grades it on every fixture at four `/Ox` lanes.

## The must-fail mutations

Three, and the first two are the inverse of the last four lanes' — this class's
finding is that a charge **exists** where those lanes found none, so the
mutation to run is removing it, not adding it.

| # | mutation | verdict |
|---|---|---|
| **M1** | `coff::Function::mints_memcpy` forced to `false` (drop the once-per-TU label slot) | `Port=Mismatch` on `wifn_guard_ret_chain.cpp` — this was the tree's ACTUAL state before the fix and it is what the differential caught (§ the rung's §4) |
| **M2** | `helper_externals` left empty (place `memcpy` in the callee region) | `Port=Mismatch` on the same fixture, four symbol indices apart |
| **M3** | `guard_ret_chain`'s `is_framed()` arm removed | the whole TU refuses — a `.pdata`-less obj two sections short |

M1 and M2 were both **observed as live mismatches** rather than induced: the
class shipped with both wrong, the fixture graded `mismatch`, and each was
fixed against the reference obj's own symbol table. The rung records the
sequence rather than presenting the finished state as if it had been designed.
