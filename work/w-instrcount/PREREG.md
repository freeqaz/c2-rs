# PREREG — lane `w-instrcount`, 2026-08-29

**Kind:** characterization. **Outcome will be exactly one of** `built` / `FAILED`.
**Predicted reach: 0. Census: +0. Required byte delta: 0.** This lane writes no
`crates/` code, and does not touch `docs/whitebox/ref/P_INLINE.md` or
`work/w-inlmetric/CLAUSES.tsv` (both are `w-clausegen`'s this wave, per
`docs/WAVE20_BRIEF_2026-08-29.md` §4).

Board rows reserved to this lane: **#3824–#3830**. Base commit `c5bfe89d9`.
Image: `compilers/X360/16.00.11886.00/c2.dll`,
sha256 `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.

**Frozen before the image was opened.** The only material read before this file
was committed is prose already in the tree: `CLAUDE.md`,
`docs/WAVE20_BRIEF_2026-08-29.md`, `docs/WHITEBOX_LEVERAGE_2026-08-21.md`,
`docs/rungs/README.md`, `docs/rungs/_TEMPLATE.md`,
`work/w-inlmetric/CLAUSES.tsv`, `docs/whitebox/ref/P_INLINE.md` §1–§5 and
`docs/whitebox/WB_INLINE_FINDINGS.md` §4. No disassembler has been run.

---

## 1. The question

Four `absent` clauses (C2, C4, C16, C17) and the `fitted` pin on C20 all name
one missing link: `no-instr-count` — *the port has no pre-codegen instruction
count*. C2 says c2 seeds `DAT_10c3f5cc = (ushort)[fn+0x50]` at `0x10b62703`.

**What is `[fn+0x50]`?** When is it written, by what, in what unit, and could
anything in `crates/` carry it?

## 2. What the tree already claims, and which parts I am treating as open

`P_INLINE.md` §2.1a (`[O]`, lane `w-sizebracket`, 2026-08-18) claims:

* `[sym+0x50]` is the `.gl` function record's `SIZE` field, arriving verbatim
  from the IL, read by `il-read-varint16` (`0x10c1f9a6`) and stored at
  `0x10b9bf6c` inside `FUN_10b9b8e9`;
* **"there is exactly ONE 16-bit store to `[reg+0x50]` in the whole image."**

`P_INLINE.md` §2.1b (`[O]`, same lane) claims the opposite-facing thing:

* `[sym+0x50]` is **initialized** from `SIZE` and is then **reduced by whatever
  runs before the inliner**, so `SIZE` is an upper bound and not the quantity.
  Witness: `arith_012_O1` and `mix_008_O1`, both `SIZE = 115`, opposite
  verdicts.

**These two cannot both be true as written**, and that tension is this lane's
centre. If §2.1a's census is complete there is no reducer, and §2.1b's
inference needs a different mechanism. If §2.1b's inference is right there is a
second writer §2.1a's census missed — which is exactly `#3505`'s sharpest
instance (an xref census that returned *0 writes* correctly, because the write
went through `rep movsd` and `EDI`). **I treat §2.1a's universal negative as
UNVERIFIED until I have re-run it with the blind spots named.**

## 3. Registered predictions

Each is falsifiable and I will report the verdict of each, including misses.

**P1 — the writer census.** A write census over the whole image for stores
that can land on `sym+0x50` — 16-bit stores, 32-bit stores at `+0x50`
(which would cover `+0x50..+0x53` and so also `WORD [sym+0x52]`), stores
through a rebased pointer (`lea`/`add` then a small displacement), and
block-copy paths (`rep movs*`, `memcpy`/`memmove` thunks) whose destination is
a symbol record — will find **exactly one initializing writer**
(`0x10b9bf6c`) and **at most one further writer**.
*Falsified if:* I find ≥ 2 further direct writers, or if I cannot bound the
block-copy path at all (in which case I report the census as **unclosable by
this method** and say so, rather than restating §2.1a's negative).

**P2 — the unit.** The value is a **front-end-supplied count of IL
instructions/tuples for the function body**, produced by `c1xx` and transported
in the `.gl` record — *not* machine instructions, *not* bytes, and *not*
computed by c2 from its own tuple stream. Support I expect to find: the
`"INL:\tInlining %s (%d instrs) into "` diagnostic formats this same field, and
the field's measured linearity in source statements (`w-sizebracket`: empty
`int f(int)` = 19, `s ^= a;` +4, `if (s>3) s=1;` +13).
*Falsified if:* the diagnostic reads a different field, or the seed at
`0x10b62703` is a computed quantity (a loop counting tuples) rather than a
field load.

**P3 — the `ushort` truncation.** The seed at `0x10b62703` narrows to 16 bits,
so a caller at ≥ 65,536 count units wraps modulo 65,536 rather than saturating.
I predict the field itself is 16-bit-wide at rest (so the truncation is a
no-op at the store and the real ceiling is imposed upstream, in the `.gl`
reader / the front end), rather than a 32-bit quantity being narrowed here.
*Falsified if:* `[sym+0x50]` is read anywhere as a DWORD, or the `.gl` reader
can deposit a value that a `movzx`/`ushort` consumer would read differently
from the value the producer meant.

**P4 — the F7 answer.** F7 (*"a 48-byte caller and a 5,640-byte caller give
identical verdicts on 12 cells"*) is **not** evidence that the count is not an
input. I register two candidate explanations and predict which one the read
settles:

* **P4a — the clamp floor.** `B = clamp(2 × caller_instrs, 1000, 35000)`, so
  every caller under 500 count-units produces the *identical* budget `B = 1000`.
  If both of F7's callers are under 500 units **in the read unit**, the axis
  was never varied at all. I further predict that the published *"`B` from
  1000 to ~2,820"* arithmetic is **in the wrong unit** — it looks like source
  or emitted **bytes** divided by 2, and the tested quantity is neither.
* **P4b — the budget is structurally unreachable on a one-site grid.** C17
  declines only when `budget < instrs && instrs > 0x28`. Candidacy (C8) has
  already refused any callee at or above the ceiling `DAT_10c46318`, so
  `instrs` at C17 is bounded by that ceiling; with `B ≥ 1000` the budget cannot
  bind until accumulated charges (C19) have drained it, which needs many
  charged sites in one caller. A grid with one call site per cell cannot
  reach it **for any caller size**.

**I predict P4b is the dominant term and P4a is a real second term**, and that
the honest statement is *"F7 varied an axis that is clamped, and measured it
through a predicate that a one-site cell cannot reach."* If instead the read
shows the budget binding on a single site, P4 is a miss and I say so.

**P5 — the clause verdicts.** Before reading, my prediction of what the read
unblocks:

| clause | predicted |
|---|---|
| C2 | **unblocked** — the port already decodes the producing field and discards it |
| C4 | **not unblocked by this read** — its blocker is the absent driver/recursion, not the count |
| C16 | **unblocked as unreachable** — a 35,000-unit caller does not exist in the workload; the row becomes a bounded refusal, not an adoption |
| C17 | **partly** — the predicate is readable, but it needs C19's accumulation, which is a separate absence |

*Falsified* row by row. Overstating this list costs a wave, so a row I cannot
settle is reported as *unsettled*, never as unblocked.

## 4. Read before probe — the price I am registering now

The read is: one function body (`FUN_10b9b8e9`, the `.gl` reader), one seed
site (`0x10b62675`'s prologue through `0x10b6276e`), and an image-wide write
census. Estimated at **well under a lane**.

The probe that would answer the same question is a caller-size grid at
sufficient resolution to find the clamp knee — which needs the unit *first* to
know where to place cells, so **the probe cannot be designed without the read**.
That is the read-before-probe justification and it is registered here. If I run
any cell at all, it is a **confirmation** probe against a prediction written
down before it, per `WHITEBOX_LEVERAGE_2026-08-21.md` §1.

## 5. What I refuse to conclude

* **I will not conclude the port can carry the count.** Adoption is a later
  wave's, off `w-clausegen`'s repaired screen. "Derivable" is the strongest
  word available to this lane, and it requires every field to carry a
  `PROV[R]` address.
* **I will not claim `.gl SIZE` *is* the tested value.** §2.1b's matched pair
  is `[O]` and refutes it; whatever I find must be consistent with 115/115
  giving opposite verdicts, or must explain that pair away with evidence.
* **I will not assert a universal negative** ("no other writer exists") without
  publishing the census method, its query set, and the classes of write it
  cannot see. `#3505` is six for six.
* **I will not report a clause as unblocked** on the strength of the address
  alone. Unblocked means: a named link from c2's quantity to something
  `crates/` can compute, with the addresses to write it from.
* **I will not touch** `crates/**`, `P_INLINE.md`, `CLAUSES.tsv`,
  `P_GLOBREGS.md`, `P_DAG.md`, `docs/STATUS.md`, `docs/rungs/INDEX.md`, or any
  board row outside **#3824–#3830**.

## 6. Gate evidence owed

`scripts/gate.sh --jobs 16 --require-graded` (unqualified `GATE: PASS`) and
`C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast`
with **both the target count and the pass count**. Byte delta must be zero and
is zero by construction: this lane changes no compiled file.
