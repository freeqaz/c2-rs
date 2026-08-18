# PREREG — lane `w-glattrs`, the `.gl` function record `SIZE` field's `0x80` escape

    Lane:   w-glattrs
    Base:   master `666fe6eb7`
    Date:   2026-08-18
    Kind:   construct — a container-reader decode, graded against the oracle
    Seam:   crates/c2-il (the `gl.rs` / `gl_function_attrs` region) + docs/whitebox/

**Frozen before any probe compiled, any scan ran, and any line of `crates/`
changed.** Committed as this branch's first commit. Everything after this file is
scored against it.

---

## 0. The dispatched question

`w-sizebracket` §2.3 / board **#3274**: `gl_function_attrs` **refuses the whole
file** when the `.gl` function record's `SIZE` byte is `>= 0x80`, and that shape
fires on **309 of 7,667** graded workload call edges (4.0 %). The lane is sent to
decode the escape correctly, confirm it against real `c2.dll` and real `.gl`
bytes, ship it, and **score the decode against the oracle** — not against
`fnbyte-exact`.

The binding methodological fact, from the same lane and now in
`docs/rungs/README.md`: **a predicate can be 39.6 % wrong about c2 and free in
the metric used to choose it.** `fnbyte-exact Δ` is evidence about **reach**,
never about correctness.

---

## 1. What is admissible as evidence, registered BEFORE anything is read

The brief requires this list to be frozen so that a constant cannot be admitted
after it flatters a number.

### 1.1 ADMISSIBLE — and disclosed

| fact | where it comes from | status |
|---|---|---|
| the **control flow** of `il-read-varint16` at `0x10c1f9a6` — i.e. **how many bytes each arm consumes** | the disassembly of the pinned image (`C2_MAP_METHOD.md` §0, sha256 `c80981…6258`) | **adoption** — a `DISCLOSURE.md` row in the same commit as the `crates/` change |
| the **field order** of the `.gl` tag-`0x0e` arm at `0x10b9bf57`…`0x10b9bf78` | already in `docs/whitebox/ref/ADDR.tsv`, written down by `w-emitp`/`w-roots` before this lane existed | navigation |
| the **payload endianness** of the escape | the disassembly, **and** a mutation replay against real `c2.dll` (GRID-B). **The port cannot observe it** — see §6 M2 | reported, and its unobservability reported with it |

### 1.2 NOT ADMISSIBLE

* **Any threshold constant** — `DAT_10c46318`, `INLINE_DECLINE_BYTES`, or any
  cut on `SIZE`. This lane decodes a *field width*; it does not use the field's
  *value* as a predicate. `w-sizebracket` §5.4 already measured that the best
  `SIZE` cut is 0-for-330 on the workload **and** killed by a three-line `.cpp`,
  and the dispatch explicitly forbids building a size-dependent rule on
  `[sym+0x50]`.
* **Any constant fitted to `fnbyte-exact`, `match`, or any gate count.**
* **Any semantic claim about what `SIZE` means** beyond "it is the field the
  reader must step over to reach `ATTR`". `w-sizebracket` §4 established the
  value is an upper bound on a post-fold count that the container does not
  carry; nothing here may re-open that.
* **A record shape witnessed nowhere.** If a decode arm has zero witnesses in
  the 878-TU workload and zero in this lane's constructed probes, it is
  **refused**, not guessed — `label_counter`'s rule, one field over.

### 1.3 The decision rule for the `0x81`–`0xff` direct-byte arm

Registered now because it is the one place the answer could be tuned after the
fact:

> **D1.** If the direct-high-byte form (`SIZE` byte in `0x81..=0xff`) has
> **zero** witnesses across the 878 workload `.gl` files under the framing
> `gl_function_attrs` actually walks, the reader **keeps refusing it**. It stays
> a desync canary and an unwitnessed shape, and the rung says so.
>
> **D2.** If it has ≥ 1 witness, the reader decodes it as c2 does (one byte) and
> the rung reports the witness count and the population.
>
> Either way the choice is stated in the rung with the count that drove it.

---

## 2. The decision rule for shipping

| id | clause |
|---|---|
| **S1** | The escape's width is established from the image **and** confirmed by an independent black-box observable (a `.gl` byte diff across a `__declspec(noinline)` twin at `SIZE >= 0x80`, GRID-A). Two sources or no ship. |
| **S2** | The decode is scored **against real `c2.dll`** on the population it applies to — the escaped-`SIZE` call edges — and not against `fnbyte-exact`. |
| **S3** | `mismatch` is **0** on the 878-TU scan at the tip. Non-negotiable. |
| **S4** | **A `fnbyte-exact` DECREASE is treated as evidence of a WRONG DECODE**, investigated per symbol and never netted against an increase. The mechanism: this change can only make `inlinable` *readable*, and a readable `Some(false)` only ever makes `splice`/`comdat` **refuse more**. A body that is byte-exact today by being spliced is a body c2 inlined, so a correct decode cannot refuse it. Any such loss is a decode defect until shown otherwise. |
| **S5** | If S1 fails — the two sources disagree, or the black-box observable is not produced — **DECLINE** and say so in those words, with the measurement. `Outcome: declined` is a full deliverable here. |

---

## 3. The grids, frozen by content hash

| input | sha256 |
|---|---|
| `work/dc3-workload/files.txt` (878 TUs) | `4996839bf89780a2dea9ed005450d8953961355a9eb2292cc1bc22572a6853b6` |
| `work/dc3-workload/flags.txt` | `fa8ba48aa21229773116bf0decff3b7e9e5e7f7ee356c3e347c506038ffbcb48` |
| `dc3-decomp` head | `ccd4c8036` (clean) |
| `compilers/X360/16.00.11886.00/c2.dll` | `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` |

Identical to `w-sizebracket`'s and to `w-dataseam`'s, so every figure this lane
quotes from either is quoted against the same corpus it was measured on.

**Three grids, defined before a cell compiled:**

* **GRID-A — the constructed twin grid.** Callees generated so that `.gl SIZE`
  spans both sides of `0x80`, each compiled **with and without**
  `__declspec(noinline)`, at `/O1` (the workload profile) and `/Ox`.
  Observables, per cell: (a) the `.gl` byte positions that differ between the
  twins; (b) real c2's own obj — does the caller's `.text` COMDAT carry a
  `REL24` naming the callee (`w-fence2` GRID-W's observable)?
* **GRID-B — mutation replay.** A cell's captured `.gl` is edited **in place, at
  byte granularity** and replayed through real `c2.dll` under wibo. This is the
  only instrument that can decide the payload's endianness and width from c2's
  own output rather than from its instructions.
* **GRID-C — the workload oracle grid.** All 878 TUs. Per IL call edge to a
  callee this TU defines: `kept` / `inlined` / `unknown` from the reference
  obj's `REL24` target set, crossed with (i) whether the callee's `.gl SIZE`
  uses the escape, (ii) the `ATTR` byte the new decode reads, (iii) the
  `FN_FLAG_INLINABLE` bit. Scaffold reverted before the gate; the deliverable is
  the grid, not the code that measured it.

**Results tables are DERIVED FROM THE LOGS, never accumulated**
(`docs/rungs/README.md` probe rule 2).

---

## 4. Registered baseline, at `666fe6eb7`

From the dispatch. Re-read **back to back** with the tip per **#3249**; the
cache state and `dc3-decomp` head are stated with every reading.

| metric | value |
|---|---|
| `match` | 26 |
| `mismatch` | 0 |
| `codegen-gap` | 0 |
| `vocab-gap` | 844 |
| `capture-fail` | 8 |
| `fnbyte-exact` | 35,899 |
| `fnbyte-refused-parse` | 113,447 |
| anchored `gap-metric` keys (`grep -cE '^ *gap-metric \S+ \S+$'`) | 394 |
| suite (`C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast`) | 1,671 / 0 / 46 |

---

## 5. Predictions, probability-form. Ceilings carry NO discount factor.

| id | P | prediction |
|---:|---:|---|
| **P1** | 0.75 | The base re-read reproduces `fnbyte-exact` **35,899** exactly. (±2 floor; anything larger is reported as drift, #3249.) |
| **P2** | 0.95 | The base re-read reproduces `26 / 0 / 0 / 844 / 8`. |
| **P3** | 0.85 | Anchored key count is **394**. |
| **P4** | 0.90 | **The `0x80` byte is a LENGTH ESCAPE introducing exactly TWO little-endian payload bytes — three bytes total — and `0x81..=0xff` are NOT escapes but single sign-extended bytes.** It is neither a flag nor a sentinel. |
| **P5** | 0.80 | GRID-A confirms the width **black box**: the `.gl` of a `__declspec(noinline)` twin whose `SIZE` uses the escape differs from its plain twin at a byte **exactly 3 after the `0x80`**, and the difference is **bit `0x40`** alone. |
| **P6** | 0.55 | GRID-B confirms the payload is **little-endian** from c2's own obj: rewriting `80 lo hi` so the LE reading is small flips the callee to inlined, while the BE reading of the same three bytes stays large. (Lower P: `[sym+0x50]` may be recomputed rather than trusted, in which case the mutation is inert and the prediction is a MISS for a reason that is itself a finding.) |
| **P7** | 0.70 | **Zero** witnesses of the `0x81..=0xff` direct-byte form in `SIZE` across the 878 workload `.gl` under the incumbent framing, so **D1 fires and the reader keeps refusing it**. |
| **P8** | 0.60 | The number of workload TUs on which `gl_function_attrs` goes `None → Some` under the fix is **strictly smaller than 309** and **strictly smaller than the count of TUs whose FIRST refusal is the `SIZE` clause** — the first-blocker inflation the dispatch warns about. Ceiling, undiscounted: **878**. |
| **P9** | 0.50 | That realized `None → Some` count is in **[1, 200]**. |
| **P10** | 0.75 | `fnbyte-exact` Δ over the whole change is **≥ 0**, and any decrease triggers S4. |
| **P11** | 0.60 | `fnbyte-exact` Δ is **exactly 0** — the escape population lives in TUs refused for other reasons, so the fix buys warranty and not reach. Registered as the *expected* outcome, and registered as **NOT evidence of correctness** in advance. |
| **P12** | 0.97 | `mismatch` is 0 at the tip. |
| **P13** | 0.70 | On the escaped-`SIZE` population, the decoded `ATTR` satisfies *bit 6 clear ⇒ c2 kept the call* with **0 counterexamples** on GRID-C. |
| **P14** | 0.45 | At least one workload TU currently refused by the `SIZE` clause contains a function the fix decodes as `noinline` (bit 6 clear) — i.e. the refusal is costing a real correctness signal, not only reach. |
| **P15** | 0.65 | The gate (`scripts/gate.sh --jobs 16 --require-graded`) is **BLIND to this change in both directions**: every per-lane count identical at base and tip. Registered as a *prediction about the instrument*, and it is the reason the oracle grid exists. |
| **P16** | 0.50 | Most-likely Outcome: **`built`**. Alternatives registered: `declined` 0.30, `FAILED` 0.20. |
| **P17** | 0.40 | The incumbent `gl_offset_framed` truncation (#2783 — `36 of 811` records on a 65 KB `.gl`) makes the *realized* escape-refusal population materially smaller than `w-sizebracket`'s relaxed-framing measurement of 309 edges. Reported as found-and-not-taken either way; **this lane does not touch the framing**. |

---

## 6. Mutant colours, registered up front

Each is a deliberate one-line corruption of the **shipped** decode. The colour
is what `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release
--no-fail-fast` does, **plus** whether `scripts/gate.sh` moves.

| id | mutation | predicted suite | predicted gate |
|---|---|---|---|
| **M1** | escape consumes **5** bytes (`q += 5`) instead of 3 | **RED** | **GREEN** |
| **M2** | escape payload read **big-endian** | **GREEN** | **GREEN** |
| **M3** | `0x81..=0xff` decoded as a 3-byte escape instead of refused | **GREEN** (predicted, conditional on P7) | **GREEN** |
| **M4** | the escape arm deleted — i.e. the incumbent restored | **RED** | **GREEN** |

**M2 and M4 are the important rows and they are predicted colours, not
oversights.**

* **M2 GREEN is a statement about the port's blindness, not about the finding.**
  The port needs the escape's **width** and never its **value**, so endianness
  is unobservable in `crates/` by construction. It is therefore graded by
  GRID-B against real c2 or it is not graded at all — which is exactly the
  discipline `w-sizebracket` §5.2 established.
* **M4 GREEN on the gate is the honest form of "this change is invisible to the
  standing instruments"** (P15). A lane that reported a green gate as evidence
  *for* this change would be committing STATUS trap 1 in its purest form.

**A colour taken in an environment whose executed-test count and differential
duration were not asserted is VOID, not provisional** (#3219 / #3231): discard,
re-run, keep the invalid log.

**The environment control, pinned BY NAME and not by count:** the suite must
report `SKIP: toolchain absent` **0 times**, and `census_gate` must take a
non-zero wall time. An unprovisioned worktree is byte-identical in every printed
count.

---

## 7. What this lane does NOT do

* It does **not** touch `gl_offset_framed` / `gl_offset_framed_relaxed`
  (#2783). That is a different widening with a different price and it is
  measured here only as found-and-not-taken.
* It does **not** use `SIZE`'s value as a predicate anywhere in `crates/`.
* It does **not** touch `crates/c2-harness/tests/` or `src/gap/tests.rs` (peer
  `w-witness7`'s seam) except for scratch that is reverted before the gate.
* It does **not** rebase, merge or push.

## 8. Board rows

`#3289`–`#3293`, allocated by the coordinator. The next-free pointer in
`BOARD.md` is **not** read.
