# `w-paramfill` — PREREG

**Committed BEFORE the image is opened.** Lane `w-paramfill`, wave 19,
2026-08-29. Brief `docs/ADOPTION_BRIEF_2026-08-29.md` §L3. Board
**#3802**–**#3807**. Characterization lane (`docs/rungs/README.md` kind 3):
**predicted reach 0**, `git diff master..HEAD -- crates/` must be empty at the
tip, no `DISCLOSURE.md` row, no `scripts/gate.sh` row (`#3691`).

Base: master `12d3c0558`, branch `wt-w-paramfill`.
Image: `compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
(verified equal to `~/ghidra-projects/bin/c2dll` before this file was written —
that is a hash of a file, not a read of its code).

---

## 0. What is already published, and therefore is NOT a prediction

Three statements were relayed to this lane by `w-inlswitch`
(`WB_INLSWITCH_FINDINGS.md` §9 item 1, `P_INLINE.md` §6.8.2). They are the
lane's **inputs**, and every one of them is to be **re-derived, not relayed**
(`#3470`, and this repo's own "board rows decay" rule):

* `DAT_10c462c4` is tested at `0x10b5e4f7` (`cmp DWORD PTR ds:0x10c462c4,0x0` /
  `je 0x10b5e52e`) inside `FUN_10b5e4cc`, and the `je` skips the module-size
  trim, the table select and the `rep movsd` into the live record.
* It is *"compared against zero in ~110 places image-wide."*
* It has *"two writers: `0x10bec3e4` stores `1` unconditionally in what looks
  like driver startup, and `0x10b84bba` stores `1` when `ds:0x10c45fa0` is
  non-zero."*

**A relayed count is exactly the thing this repo has been wrong about five
times** (`#3505`), and `w-inlswitch` itself found **19** writers of
`DAT_10c3de20` where `w-lowerband` had published **10**. So the counts above
are treated as claims to be tested, not as facts.

---

## 1. Predictions — registered with the observation that refutes each

### P1 — the gate is TAKEN on this workload, and §3 survives

> `DAT_10c462c4 != 0` on every compilation this project runs, so the
> `je 0x10b5e52e` at `0x10b5e4fe` is **not** taken, the `rep movsd` runs, and
> the live record at `0x10c3f510` holds table A's contents.

Route: the unconditional store at `0x10bec3e4` reaches `FUN_10b5e4cc` on the
c2 driver path, or the conditional one at `0x10b84bba` fires under the argv
`work/w-inlswitch/cl_argv_modes.out` already measured.

**Refuted if**: the unconditional store is not on the path that precedes
`FUN_10b5e4cc` (call-graph order), or its owner function is unreachable from
c2's entry, or `0x10c45fa0`'s switch is not in the measured argv **and** no
other writer runs. Registered confidence: **0.7**.

### P2 — the gate bounds §3's `live` column ONLY, not its `defA`/`defB` columns

> The two filler calls at `0x10b5e4ed` (table B) and `0x10b5e4f2` (table A)
> execute **before** the gate test at `0x10b5e4f7`. Therefore the 33+33
> zero-guarded default stores run unconditionally with respect to this gate,
> and `WB_INLSWITCH_FINDINGS.md` §3's `defA` and `defB` columns need **no**
> added condition. What the gate bounds is the `live` column — and hence what
> every reader in the `readers` column observes.
>
> Corollary registered with it: `DAT_10c46318` is computed at `0x10b5e4d2`,
> also **before** the gate, so **`P_INLINE` §6.6.1's `16 << k` ceiling is not
> gated by `DAT_10c462c4`** and `#3734`/`#3732` are untouched by this lane
> either way.

**Refuted if**: the fillers are themselves gated (an earlier test in
`FUN_10b5e4cc` that the relayed excerpt omitted), or `FUN_10b5e4cc` is called
from more than one site with different preconditions, or either filler
early-returns on the same word. Registered confidence: **0.8**.

### P3 — the writer count is MORE than two

> `DAT_10c462c4` has **≥ 3** write instructions in the image.

Registered because this lane's stated hazard is a count relayed from one
instrument, and the base rate here is bad: 10→19 last wave, and `#3505` is
five for five. The direction is registered, not hedged.

**Refuted if**: three independent instruments (objdump linear listing, Ghidra
`xrefs.tsv`, decode-independent byte scan) all yield exactly 2 write
instructions. Registered confidence: **0.55**.

### P4 — `DAT_10c462c4` is NOT inline-specific

> The ~110 zero-tests image-wide mean this word is a **global** compiler
> condition, not an inliner one; `0x10b5e4f7` is one consumer of ~110. I
> register the specific guess: **it is the "optimizations are on" flag**, i.e.
> the one `-Og` (or its equivalent) sets, and `0x10c45fa0` is `-Og`'s value
> word or a near neighbour of it.

**Refuted if**: `0x10c45fa0` resolves to a descriptor whose name is not `-Og`
and not an optimisation-enable spelling; or the read census concentrates
inside the inliner band `0x10b5b86d`–`0x10b62b00` rather than spreading across
the image. Registered confidence: **0.45** on the specific `-Og` guess, **0.85**
on the weaker "global, not inline-specific" half. **Both halves are scored
separately** — a lane that registers a conjunction and claims the easy half is
scoring itself.

### P5 — the scatter and the two sweeps re-derive exactly

> Re-derived independently in this tree: `FUN_10b5b88f` scatters **37** value
> words into the 46-dword record; `FUN_10b5ba71` and `FUN_10b5bc6e` each run
> **33** zero-guarded default stores; `46 − 33 = 13` fields get no default in
> either table.

**Refuted if**: any of 37 / 33 / 33 / 46 differs from `w-inlswitch`'s published
figure, or a store in either sweep is found **not** zero-guarded. Registered
confidence: **0.85**. A miss here is a finding about `w-inlswitch`, and it is
reported as one.

### P6 — §3's verdict split

> Of the statements in `WB_INLSWITCH_FINDINGS.md` §3 (the 24-row table's four
> content columns, plus the six prose claims in §3 / §3.1 / §3.2): **0 are
> false**, **≥ 1 needs a stated condition** (the `live` column, per P2), and
> the remainder survive unchanged.

**Refuted if**: any §3 statement is found false — in which case that is the
lane's headline, not a footnote. Registered confidence: **0.6**. This is the
one the brief actually asked for and it is registered before the read.

---

## 2. Controls — all three watched before any verdict is quoted (`#3336`)

| id | kind | population | required |
|---|---|---|---|
| **C1** | GREEN | the objdump listing's decoded instruction starts | the enumerator, pointed at `DAT_10c46318`, must return the set `P_INLINE` §6.6.1 established **independently of this lane**: writers `0x10b5e4d7`/`0x10b5e4e8`, reader `0x10b5fc8a`. Anything else = the instrument is broken and no absence claim from it may be quoted |
| **C2** | RED | same | a planted address `0xdeadbe00` must return **0** references. A non-zero result means the matcher is matching text, not operands |
| **C3** | RED | the raw `.text` bytes | the decode-independent scan, pointed at a byte pattern that cannot occur (`0xdeadbe00` little-endian), must return **0** hits, while the same scan for `0x10c462c4` returns ≥ the listing's count. A byte scan that finds nothing anywhere is not a control, it is a broken loop |
| **C4** | CROSS | Ghidra `xrefs.tsv` (control-flow-driven, 146,818 refs) | the write set must agree with the listing's **to the address**. Disagreements are **reported, not reconciled** — `w-inlswitch` §5.1's one-in-390 objdump desynchronisation at `0x10bd5d2f` is the precedent |

**The byte scan is not optional.** c2 has a ~150 KB data block at the head of
`.text`; `objdump` sweeps linearly and anything inside a desynchronised run is
invisible to it. `w-lowerband`'s `bytescan.py` exists for exactly this and this
lane's hazard is the same one.

**Denominators beside numerators, every time** (`#3470`): a count of writers is
reported as *N of M decoded instruction starts* / *N of M raw bytes scanned*,
never bare.

---

## 3. What each outcome licenses

| outcome | state change |
|---|---|
| **P1 holds** | `WB_INLSWITCH_FINDINGS.md` §3 and `P_INLINE` §6.8.1–§6.8.3 stand as written; §6.8.2's `GATE 1 — not read` annotation is replaced by the read, with the condition named. **No number moves.** |
| **P1 refuted** (gate can be 0 here) | §3's `live` column and every `readers` claim downstream of it are **conditional**, and `P_INLINE` §6.8.3's *"read and dead"* becomes *"dead for a second, stronger reason"*. Still no adoption — a dead knob that is dead twice is not more adoptable |
| **P3 refuted** (exactly 2 writers) | a registered miss, published as one; and it is evidence **against** this lane's own base-rate reasoning, which is worth more than the count |
| **any §3 statement false** | the lane's headline, its own board row, and an amendment to `P_INLINE` that says which |
| **any control RED-fails** | **no absence claim is published at all.** The verdict is `FAILED` in those words (`docs/rungs/README.md`), not a compound headline |

## 4. Out of scope, stated in advance

* **No adoption.** No `crates/` byte, no `DISCLOSURE.md` row, no `gate.sh` row.
* **128 is not adopted and is not restated as the settled inline ceiling**
  (`#3732`, `#3734`). `k = 3` at run time is `w-inlswitch`'s settled result and
  is cited, not re-litigated.
* **`work/w-inlmetric/CLAUSES.tsv` is not touched** — `w-inlclause` owns it
  this wave.
* **No new `ported` numerator** for the inliner (decision 21 §4).
* `w-clausefix`'s ten address patches for `P_INLINE` §6.1 are a **separate**
  deliverable, applied only if independently re-derived in this tree. If they
  are not re-derived they are not applied, and the reason is stated.
