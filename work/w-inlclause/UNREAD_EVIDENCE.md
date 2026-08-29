# `w-inlclause` — the `R3` evidence file

`work/w-inlclause/PREREG.md` §2: **`R3` is not cheap.** *"A universal negative
over a corpus is the easiest thing in this file to assert and the hardest to
check, so every `R3` row must carry a recorded, reproducible search … This makes
`R3` cost the same as `R1`."*

`work/w-inlclause/read_state.py`'s EVIDENCE check requires a `## C<n>` section
here for every row whose `read` cell is `R3`.

---

## THE POPULATION IS EMPTY, AND THAT IS THE LANE'S ANSWER

**No row of the 24 is `R3` on this tree.** The EVIDENCE check therefore grades
**0 of 24 rows** and says so in those words rather than printing a green — the
`#3470` rule, applied to a check of this lane's own making.

That is not a check that never had work to do. It had two rows and they were
closed inside the lane:

| row | `R3` as dispatched | closed by | now |
|---|---|---|---|
| **C5** | the corpus states *"instruction kind `0x0f` is a call site"* and cites only `FUN_10b600e6`'s **entry** — `work/w-clausefix/REPAIRS.md` marks that citation *"pins the address and not the clause"* itself | `IMAGE_READ.md` §4 — `cmp al,0xf` at **`0x10b6020b`**, on `BYTE [instr+0x8]` | `R2`, `no-instr-stream` |
| **C6** | same entry citation; `WB_INLINE_FINDINGS` §1's seven-opcode list names no address at all | `IMAGE_READ.md` §3 — the dispatch head at `0x10b603ef`–`0x10b60405`, both tables decoded by `jumptable.py`, and the flag word at `0x10b602bc`–`0x10b60347` | `R2`, `no-instr-stream` |

`work/w-inlclause/read_scan.py` records both readings mechanically:

```
PIN-SCAN: 13 of 15 rows have at least one clause-pinning address cited in the
          frozen corpus as dispatched
PIN-SCAN: 15 of 15 rows once this lane's own read is included
```

**So the brief's question — *"absent because unread, or absent because
unadopted?"* — answers 13 to 2 as dispatched and 15 to 0 at this tip.** The
`absent` column was never mostly an ignorance column, and the prereg's **P2**
(*"≥ 5 place `R3`"*) is refuted. The prereg named P2 as the prediction it most
expected to lose, and said why: *"every one of these 24 rows sits inside a
function some lane has listed."*

## The method a future `R3` owes

Kept here because the population being empty is exactly when a procedure gets
forgotten.

1. **Name the clause-pinning address**, which for eight of the fifteen rows is
   **not** the address `CLAUSES.tsv` cites — five rows cite a function entry
   (C1, C5, C6, C20, C21) and three cite a block head (C11, C12, C13).
   `read_scan.py`'s `PINS` table is where it goes, and every entry there is
   verified against the independent objdump listing first.
2. **Run the scan** over the frozen corpus. `.md` prose only: a `*.asm` dump is
   the **input** to a read, and counting it would make every clause in a dumped
   function `R3`-free for nothing. `work/w-inlbudget/FUN_10b600e6.asm` contains
   every address in §3 above and told nobody which of them mattered.
3. **Record the miss list here**, under `## C<n>`, with the addresses searched
   and the corpus as of a named commit.
4. **Then read it**, and say in the rung which of the two states the row moved
   between — because `absent → absent` with the `read` cell moving `R3 → R2` is
   a real result and the `state` column cannot express it.
