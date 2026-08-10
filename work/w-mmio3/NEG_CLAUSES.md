# W-MMIO3 — which clause each `_neg` cell actually trips, READ and not predicted

`fixtures/cpp/wmmio3_close_call_chain_neg.cpp` holds ten cells, each one clause
away from the positive fixture. The point of the file is not that it refuses —
a file of nonsense would do that — but that each cell reaches a **different**
named clause, so the fence is exercised rather than merely present.

`c2rs census` cannot answer this, and it is worth seeing why rather than taking
it on trust: the production is non-committal, so a body that declines is
reported under the *dispatch arm's* blocker and all ten read **`expr-cmp-eq`**,
the same key `mmioClose` itself read at this lane's base. The keys below were
read off a **scratch print at the production's dispatch site**, behind
`C2RS_CCC`, committed nowhere and reverted in the same session — `w-ifn`'s
method, itself `w-blockir`'s, board #2087.

```
$ C2RS_CCC=1 c2rs census fixtures/cpp/wmmio3_close_call_chain_neg.cpp \
      --flags-file work/w-mmio3/fx/one-o1/flags.txt
```

| cell | the one clause it is away from the positive | key reached |
|---|---|---|
| `wmmio3_n1` | the guard's operand is a `uint`, not a pointer | `gret-guard-operand-not-a-pointer` |
| `wmmio3_n2` | the first early return is BRACED | `ccc-early-return-arm-operand` |
| `wmmio3_n3` | the first early return tests `== 0`, not `!= 0` | `ccc-early-return-not-cmp-ne` |
| `wmmio3_n4` | the indirect call's third argument is a literal, not formal 1 | `ccc-icall-args-are-not-(lit, formal, lit, base)` |
| `wmmio3_n5` | the ELIDED call's result is USED | `ccc-elided-token` |
| `wmmio3_n6` | THREE formals | `ccc-not-two-formals-free-fn` |
| `wmmio3_n7` | the guard's arm returns `70000`, wider than a `li` immediate | `gret-arm-literal-wider-than-simm16` |
| `wmmio3_n8` | the indirect call's FIRST argument is the formal, not the cast local | `ccc-icall-first-argument-is-not-the-cast-local` |
| `wmmio3_n9` | the second early return returns `r1`, not `r2` | `ccc-early-return-arm-returns-another-value` |
| `wmmio3_n10` | the void call takes TWO arguments | `ccc-void-call-is-not-one-argument` |

**Ten cells, ten distinct keys, and TWO of them are not the key the cell was
written for.** Recorded rather than tidied, exactly as `w-ifn` recorded its
`n4`:

* **`wmmio3_n4`** was written for `ccc-icall-third-argument-is-not-formal-1`
  and reaches the *pattern* clause one step earlier instead: with a literal in
  the third position the argument region is `[Lit, Lit, Lit, Load]` and the
  four-element `[Lit, Load, Lit, Load]` destructuring fails before the
  formal-index test can run. The clause it was written for is therefore
  **unreached by this file**, and it is not left ungraded: the destructuring
  and the index test are the same conjunction and `wmmio3_n8` enters it from
  the other side.
* **`wmmio3_n8`** was written for `ccc-icall-base-is-not-the-cast-local` and
  reaches `ccc-icall-first-argument-is-not-the-cast-local`, because passing `h`
  instead of `t` changes the first ARGUMENT and leaves the receiver base
  alone — the source cannot spell "call through the formal" without also
  changing the base's type. Same shape, one clause later.

**Two cells reach the ladder that are not cells at all.** The scratch print
emits twelve lines for twelve bodies: the file's own two `noinline` helper
leaves reach this production, fail at `gret-guard-operand` and
`ccc-not-two-formals-free-fn`, and are then accepted by a LATER production as
`straight-line`. That is the non-committal contract working — a declining
production leaves the cursor untouched — and it is noted so the twelve is not
read as twelve cells.

## The two INTERPROCEDURAL clauses are NOT in this file, and could not be

They refuse at `c2_il::IlBundle::functions`, which is a **whole-TU** verdict. A
cell for one of them inside a multi-cell file would refuse every other cell with
it and grade none of them — `w-decouple` §8.2's shape, where a cell that grades
nothing looks exactly like nine that work. They are one per file:

| file | the clause | how it is graded |
|---|---|---|
| `wmmio3_close_sibling_neg.cpp` | the ELIDED callee is an EXTERNAL, not a sibling | the differential. `Port=NotImplemented`, and the reference obj (`work/w-mmio3/ref/sib.obj`) carries `bl wmmio3s_setbuf` at `+0x6c` with its REL24 — **144 bytes against the positive's 124** — so a port that dropped it would be 20 bytes and one relocation short. That is a real `Mismatch` and the mutation grid produces it |
| `wmmio3_close_extern_neg.cpp` | the VOID callee is DEFINED here, so its footprint is not the whole volatile set | the differential. `Port=NotImplemented`, on the conservative side: M-RULE says the r31 park is a function of the exact footprints of the calls the value is live across, and this transcription has one witness |

## The mode gate, separately

Every cell here and both bodies of the positive fixture are additionally out of
class at `/Ox` on `ccc-not-o1`, which is asked FIRST and before any body byte
(board #1638). That is a second, coarser refusal and not the one these cells are
for; `scripts/mode_lane.sh` grades it on every fixture at four `/Ox` lanes.
