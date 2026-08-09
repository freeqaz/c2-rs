# w-xtea2 — the `_neg` cells and their MUST-FAIL mutations, RUN

Lane `w-xtea2`, 2026-08-09. Reproduce with `work/w-xtea2/mutate.sh`, which
always reverts (including on failure) and rebuilds before and after.

Each mutation deletes **exactly one shipping clause** of
`try_parse_memcpy_tail` and grades that clause's own `_neg` fixture — plus the
positive — against real `c2.dll` under wibo at the workload's own `/O1 /Oi`.

```text
   work/w-xtea2/mutate.sh
   M1 match 1 mismatch 1 codegen-gap 0 vocab-gap 3 port-error 0
   M2 match 1 mismatch 1 codegen-gap 0 vocab-gap 3 port-error 0
   M3 match 1 mismatch 1 codegen-gap 0 vocab-gap 3 port-error 0
   M4 match 1 mismatch 1 codegen-gap 0 vocab-gap 3 port-error 0
   reverted; baseline:
      match 1 mismatch 0 codegen-gap 0 vocab-gap 4 port-error 0
```

**4 of 4, and the positive stays `match` under every one of them** — so each
mutation is graded against the class it widens and not against a build that
broke.

| # | fixture | clause deleted | what the port then emits | verdict |
|---|---|---|---|---|
| **M1** | `wxtea2_mcpy_rev_neg.cpp` | `mcpytail-operands-are-not-already-in-the-argument-registers` | one word where c2 emits five; the copy runs backwards | **FAIL, `mismatch 1`** |
| **M2** | `wxtea2_mcpy_short_neg.cpp` | `mcpytail-length-outside-the-call-window` | `li r5,4 · b memcpy` for a body c2 expanded inline (40 B, no call at all) | **FAIL, `mismatch 1`** |
| **M3** | `wxtea2_mcpy_srcoff_neg.cpp` | `mcpytail-source-carries-a-member-offset` | three words where c2 emits four — the source `addi` is simply missing and every relocation still resolves | **FAIL, `mismatch 1`** |
| **M4** | `wxtea2_mcpy_stmt_neg.cpp` | the `eat_return_plumbing` gate after the call's `4B` | a 12-byte LEAF tail branch for a 60-byte FRAMED body, dropping the store and the whole `.pdata` record | **FAIL, `mismatch 1`** |

---

## The two things this grid cost, and both are method

### 1. A `_neg` fixture with several refusing bodies CANNOT go `mismatch`

The first draft was ONE file holding all four cells. Every mutation came back
`match 1 mismatch 0 vocab-gap 1` — the mutated cell parsed, and the TU still
read `vocab-gap` because the other three refused. **A TU verdict is a
conjunction over its functions**, so a `_neg` file that holds more than one
refusal has no way to express "this one cell is now wrong". The grid proved
nothing until the cells were split one per TU.

That is `CEILING.md` §11.4 item 8's mechanism in the `_neg` direction: the
instrument that would have shown the wrong bytes (`fnbyte-differs`) is keyed on
a different predicate from the one the fixture gate reports.

### 2. A `_neg` cell fenced by THREE clauses grades NONE of them

M1's cell was originally `memcpy(out, p->b, 0x10)` — the direction reversed
**and** a member offset on the source **and** a receiver in play. Deleting the
destination-base clause left the source-register clause and the source-offset
clause still refusing it, so `mismatch` stayed 0 and the cell said nothing about
any of the three.

Two repairs, and both are in the shipping code rather than in the harness:

* the cell was re-derived from a probe that isolates the register plan and
  nothing else (`work/w-xtea2/probe/mcpyswap.cpp` — two plain pointer formals,
  no offsets, one length, and c2 keeps the call: 8 B accepted against 20 B
  swapped);
* the two register clauses were **merged into one**, because the fact is the
  conjunction and the emitter's words depend on it as a conjunction. Split in
  half, neither half is gradeable.

`w-pool2` §5 found a predecessor's `_neg` cell had become a *positive*; this is
the adjacent failure — a `_neg` cell that is over-fenced and therefore
**vacuous as evidence**, while looking exactly like a working one.

### 3. …and a third, on the cell that would have been vacuous outright

`wxtea2_mcpy_srcoff_neg.cpp` was written at length `0x10`. At that length c2
does not call `memcpy` at all — it expands the copy as two `ld`/`std` pairs, 24
bytes — so the body never reaches the recognizer and the clause under test is
never asked. The length is `0x40` **because of what the obj said**, not because
0x40 was chosen. Compiled first, claimed second.

---

## The mode halves, on the same five fixtures

```text
   /O1 /Oi /GS- /c    match 1  mismatch 0  vocab-gap 4     the class's own mode
   /O1     /GS- /c    match 0  mismatch 0  vocab-gap 5     no /Oi: memcpy is an
                                                           ORDINARY call, not the
                                                           `40` intrinsic
   /Ox     /GS- /c    match 0  mismatch 0  vocab-gap 5     the parser's mode gate
```

The middle row is worth keeping: without `/Oi` the copy arrives as a `26` callee
push rather than as intrinsic selector 172, so the class is not merely refused
there — it is not the same IL. `scripts/lanes.txt` carries four `/O1 /Oi` lanes
(`O1-Oi`, `O1-Oi-EHsc`, `O1-Oi-GR`, `O1-Oi-EHsc-GR`), which is what grades this
class in the merge gate at all.
