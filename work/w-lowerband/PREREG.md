# `w-lowerband` — PREREG

    Lane:      w-lowerband
    Kind:      characterization
    Charter:   docs/DECISIONS_2026-08-22.md decision 21, the `w-lowerband` row
    Board:     #3731–#3736 (reserved; rows only in that range)
    Base:      8213c7b77
    Image:     compilers/X360/16.00.11886.00/c2.dll
               sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
               (verified on this tree before this file was written)

> **Registered BEFORE the image was opened.** Nothing under
> `~/ghidra-projects/export/c2/` has been read at the time of this commit — no
> `grep`, no `objdump`, no `peread.py`. The only files read so far are the
> repo's own prose (`CLAUDE.md`, `docs/DECISIONS_2026-08-22.md` decision 21,
> `docs/whitebox/ref/P_INLINE.md`, `docs/rungs/2026-08-27-w-inlfit.md`,
> `docs/rungs/README.md`, `docs/BOARD.md`) and the two `work/w-inlfit/` helper
> scripts (`grab.py`, `peread.py`), which are tooling and quote no addresses
> beyond ones `P_INLINE` already publishes.

---

## 1. The subject, stated as the previous lane left it

`P_INLINE.md` §6.6.1 closes C8 with two named missing links, **neither in the
inliner band `0x10b5b86d`–`0x10b62b00`**:

1. *"`[sym+0x50]` is initialised from the `.gl` `SIZE` field at `0x10b9bf6c` and
   is **reduced by every pass that runs between there and `0x10b5fc8a`**. §2.1b
   measures the consequence … and **nothing yet located reads that
   reduction.**"*
2. count→bytes, *"the whole of lowering"*.

**Link 1 is this lane's whole subject. Link 2 is OUT OF SCOPE** by decision 21
and by this lane's brief. If the chain runs into lowering I stop at the boundary
and name the address where I stopped; I do not price it, model it, or fit it.

The measured consequence I am trying to explain, from §2.1b:

| cell | `.gl` `SIZE` | emitted `.text` | real c2 |
|---|---:|---:|---|
| `arith_012_O1` | **115** | 28 | **inlined** |
| `mix_008_O1` | **115** | 132 | **kept** |

Identical `SIZE`, opposite verdicts, at `/nologo /c /GR /O1 /Oi /EHsc`.

## 2. The tension I am registering as my starting hypothesis

§2.1a states, in the same document, that **"There is exactly ONE 16-bit store to
`[reg+0x50]` in the whole image"** — at `0x10b9bf6c`, the `.gl` decode.

§2.1a and §6.6.1 cannot both be read literally. If the field has exactly one
writer, nothing *reduces* it, and §2.1b's separation is carried by something
other than `[sym+0x50]`. **I register in advance that I think §2.1a's "exactly
ONE" is a claim about an INSTRUMENT** — a text pattern over one listing — and
not about the image. That is this repo's most repeated defect class (the brief
names four prior instances, most recently `w-regcells`' 213 cells).

## 3. Predictions

Each is registered with the outcome it licenses, so no outcome is chosen after
seeing the answer.

### P1 — the write set is LARGER THAN ONE

A properly enumerated write set over the whole image finds **between 2 and 6**
distinct sites that can store to this field, counting `0x10b9bf6c`.

I name in advance the four forms whose omission would produce §2.1a's "exactly
one", so that finding one of them is a HIT and not a retrofit:

* **(a) read-modify-write arithmetic** — `sub`/`add`/`dec`/`inc`/`or`/`and`
  `WORD PTR [r+0x50]`, which is not a `mov` and would miss a `mov`-shaped grep;
* **(b) a 32-bit store at `+0x50`**, which also covers `+0x52` and would miss a
  16-bit-width grep;
* **(c) a store through a computed pointer** — `lea r,[base+0x50]` or a pointer
  advanced by `0x50`, after which the displacement `0x50` is not in the
  instruction at all;
* **(d) a block copy** of the whole record (`rep movs`, or a field-by-field
  clone), where the field is written without `0x50` ever appearing.

**HIT** if ≥1 site besides `0x10b9bf6c` is found writing this struct's `+0x50`.
**MISS** if the enumeration, run over the independent objdump boundary set and
cross-checked against Ghidra's xrefs, confirms exactly one.

### P2 — `arith_012` and `mix_008` are separated by **ONE** site, not by many

I predict the reduction is a **single recompute at a pipeline boundary** — one
site that both cells reach, differing only in the value it computes — and not a
chain of per-pass decrements. Formally: **at most 2** distinct value-changing
sites lie on the `/O1`, non-POGO, non-`__forceinline` path the workload takes.

**REFUTED** if ≥3 distinct value-changing sites lie on that path. Per the
brief, *"if the chain is longer or more conditional than the fixture pair
suggests, that is the result"* — so a refutation here is a deliverable, not a
failure, and I will report the full enumerated chain rather than the shortest
one that explains the pair.

### P3 — the mechanism is a RECOMPUTE, not a DECREMENT

The reducing site stores a freshly counted value obtained by walking the
function's body, rather than subtracting a delta from the field's current value.

**REFUTED** if the located site is form (a) above — an RMW arithmetic on the
field itself.

P1(a) and P3 are deliberately in tension: P1(a) is a prediction about what
§2.1a's grep would have missed, P3 is a prediction about what the real
mechanism is. If the answer is an RMW site, P1 scores a HIT and P3 scores a
MISS, and both are reported.

### P4 — no reducing site is inside the inliner band

No site that changes `[sym+0x50]` lies in `0x10b5b86d`–`0x10b62b00`. This is
§6.6's own claim, restated so that it can fail.

**REFUTED** if one is in-band — which would contradict `P_INLINE` §6.6 and
would be worth more than a confirmation.

### P5 — the negative branch, registered so it cannot be dressed up

It is possible the enumeration is honest and the answer is **exactly one
writer**. In that case:

* §2.1b's separation is **not** carried by `[sym+0x50]`, and §6.6.1's
  *"reduced by every pass"* is wrong as written;
* I will say so, name the enumeration and the population it ran over, and name
  the alternative carriers I would test next (the site's own flag word, the
  per-site record `[site+…]`, `[sym+0x4c]`'s other bits, a second count field
  such as `WORD [sym+0x52]`);
* I will **NOT** fit a formula, will **NOT** propose a constant, and will
  **STOP**.

### P6 — reach

**Predicted reach 0.** Zero `crates/` bytes, no `DISCLOSURE.md` row, no
`gate.sh` row (`#3691`), no change to `INLINE-P` or `splice.rs`, and **no clause
row of §6.1 added, removed, renumbered or restated** (`w-inlmetric`'s table is
another lane's frozen instrument; new clauses discovered here are filed as named
follow-ups, `w-inlfit`'s §6.5 convention).

## 4. The enumerations I will run, and their populations, fixed now

Every "nothing reads X" sentence in the findings must name which of these it
rests on and over what population. Counts are reported even when they are large.

| id | population | what it enumerates |
|---|---|---|
| **E1** | every instruction start in `objdump_intel.asm` (independent of Ghidra) | the boundary set, so no address is quoted mid-instruction (`#3721`) |
| **E2** | E1 ∩ instructions whose memory operand has displacement `0x50` | split by **width** (byte/word/dword) and by **direction** (read/write), reported as a table |
| **E3** | Ghidra `xrefs.tsv` / `decomp_all.c` | independent cross-check of E2's write set; disagreement is reported, not silently reconciled |
| **E4** | functions containing a reference to **both** `+0x4c` and `+0x50` | the struct-identity filter: `+0x4c` is `ATTR` and is confirmed from the container side (C13, `w-mmioclose`) |
| **E5** | every `lea` with displacement `0x50`, and every `rep movs`/field-clone in E4's functions | P1 forms (c) and (d), which carry no `0x50` at the store itself |

## 5. The control, and I will watch it FAIL before quoting it (`#3336`)

The enumerator is worthless if it cannot miss. Before any verdict is quoted:

* **positive** — the known store at `0x10b9bf6c` must appear in E2's write set.
  If it does not, the enumerator is broken and no count from it is reported.
* **negative, watched RED** — the same enumerator run over a copy of the
  listing in which that one line's displacement is edited `0x50` → `0x51` must
  report the site **gone**. A control I have not watched go red is decoration.

Both transcripts land in `work/w-lowerband/`.

## 6. What I may not do

* **No count→bytes.** Out of scope by decision 21 and by the brief. If the
  chain reaches lowering I stop and name the address.
* **No change to the port's inline predicate.** `INLINE-P` is frozen by content
  hash; any change is an EMIT change needing a two-sided price. **Adopting 128
  is specifically forbidden** — §6.6 says the port's constant is in the wrong
  unit and the converter is two subsystems away.
* **No new count-bearing `gate.sh` row** (`#3691`).
* **No rows outside `#3731`–`#3736`**, and none in another lane's block or in
  the reservation ledger line.
* **Amend-beside only** in `P_INLINE.md`: §1–§6.6 stay as written, including
  §2.1's struck block and §2.1a's "exactly ONE" sentence, whatever this lane
  finds. A correction is bannered next to the original, never over it.
