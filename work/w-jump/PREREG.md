# w-jump — PREREG

**Frozen and committed BEFORE the first workload scan and before the first line
of the scratch instrument.** Lane `w-jump`, worktree branch `wt-w-jump` off
master **`154c8580`** (the w-bdnz merge). Board rows **#2000**–**#2019**.

The commission (ROADMAP §10.26.5, board **#1988**, `docs/rungs/2026-08-09-w-bdnz.md`
§8): **decompose the `expr-jump` refusal family** so the loop seam's reader gate
is PRICED instead of guessed. w-bdnz says a reverted scratch instrument keyed on
*the byte after the `0x3A`* prices it in one scan. This lane ships **no
`crates/` change**; the instrument is a scratch that is reverted and recorded as
a quoted patch.

---

## §0 — what is known before the scan, and where it came from

Nothing below is a scan result. Two of the three numbers are *inherited*, and
they are registered as predictions precisely so that the re-derivation can score
them rather than absorb them.

| fact | claimed value | claimed by |
|---|---:|---|
| `expr-jump`, bodies | 2,286 | w-bdnz §1 table / board #1985 |
| `expr-jump`, emitted | 302 | ditto |
| base commit | `154c8580` | this tree, `git log` |

### §0.1 What the `3A` operand actually is — read from the code, not scanned

`docs/whitebox/WB_READER_FINDINGS.md` §3.1 puts opcode **`0x3A` in operand class
`02`**, and class `02` (§3) is *"`if (DAT_10c67fc0 == 0 && op == 0x42) nothing;
else → class 08`"*, where class **`08`** is **`varU`→`sym`→`[0x20]`** — a single
unsigned varint that is looked up in the TU's own symbol table. So the token
after a `3A` is a **label symbol id**.

`crates/c2-il/src/func/body/expr.rs` agrees on the port's side: the
`BranchSink::Cflow` arm for `0x29 | 0x3A` does `*p += 1` and then
`read_token_var`.

And `work/w-bdnz/PREREG.md` §0.1 shows one real window:

```text
  1 x expr-jump   … 32 86 41 74 4b >3a< e8 09 29 e9 09 …
```

`e8 09` is a **two-byte** varint, so the byte immediately after the `3A` is a
continuation byte of a large id. That is the shape of an **index into a
per-TU table**, which is the exact failure mode
`crates/c2-il/src/func/body/mod.rs` documents at length for the TYPE id
(*"putting that id in the bucket name shattered one construct into 256 shards,
and a ranked histogram cannot show a shattered construct at all"*).

**So this lane registers, before scanning, that the instrument it was handed is
probably an artifact-generator** — and it registers the alternative axes it will
measure instead, so that the alternative is not chosen after seeing the answer.

### §0.2 The instrument, frozen

A scratch in `crates/c2-harness/src/gap/scan.rs` only, at the **two** sites that
count blockers (`fn_blockers`, `emit_blockers`). When
`FnCensus::verdict.key() == "expr-jump"` the key is replaced by

```text
expr-jump|<cflow>|<dispatch>|<calls>|<seg_len>|<hex_mark>|<hexwindow>|<index>|<name>
```

so **every axis is decided offline from one scan** and both columns sum to the
family total **by construction**. Nothing else in `crates/` is touched, and the
patch is reverted before the lane's first commit of any tracked file other than
this PREREG.

---

## §1 Predictions, in registered-direction form

### Re-derivation at base

* **P1** — `expr-jump` at `154c8580` re-derives to **exactly 2,286 bodies /
  302 emitted**. *p = 0.85.* A miss in **either** direction is reported as a
  seventh wrong inherited price.
* **P11** — under the instrument, the sub-key counts sum **exactly** to the
  family totals in both columns. *p = 0.95* — a **check**, not a gamble;
  registered so that a failure is visible rather than silently absorbed.

### Axis A — the instrument w-bdnz specified: the byte AFTER the `3A`

* **P2** — that byte is the first byte of a `varU` **label symbol id** (§0.1),
  i.e. an identifier and not a construct. *p = 0.90.*
* **P3** — it takes **≥ 100 distinct values** over the 2,286 bodies. *p = 0.70.*
* **P4** — its **largest single value holds < 15 %** of the family. *p = 0.70.*

> P2–P4 jointly register the finding **in advance**: the instrument the
> commission was handed is the ranking-instruments failure (five prior
> instances) caught at *design* time rather than after a lane spent itself on
> it. If P3/P4 miss — if the axis does concentrate — the finding is instead that
> it concentrates on a *label numbering artifact*, which is a different failure
> and will be reported as such rather than as a decomposition.

### Axis B — the byte immediately BEFORE the `3A`

* **P5** — the top value is **`4B`** (statement end) with **≥ 40 %** share.
  *p = 0.55.*
* **P6** — **≤ 12 distinct values cover 90 %** of the family. *p = 0.60.*

### Axis C — the `cflow` control-flow class (already decoded, already a field)

* **P7** — **`cflow-loop`** is the largest `cflow` sub-key. *p = 0.65.*
* **P8** — its share is in **[25 %, 70 %]**. *p = 0.60.*
* **P9** — **≥ 3 distinct `cflow` classes each hold ≥ 5 %**, i.e. the family is
  **not** one construct. *p = 0.75.*

### The emitted column

* **P10** — the 302 emitted are **MORE** loop-heavy than the 2,286 bodies:
  `cflow-loop` share higher by **≥ 5 percentage points**. *p = 0.50.*
  (Registered with a direction so that "the two columns differ" cannot be
  claimed after the fact whichever way it goes.)

### Reading the bodies, not just counting them

* **P12** — **at least one** of the top-3 sub-keys on the best axis is
  *different constructs wearing one key*: ≥ 2 distinct source-level constructs
  among ≥ 3 sampled bodies. *p = 0.80.*
* **P13** — the counted-loop class's **real neighbours** — bodies w-bdnz's
  recognizer could reach under board #1988's named extensions (a)–(c) — are
  **≤ 10 %** of the 2,286. *p = 0.70.*

### The recommendation

* **P14** — the priced next rung on this family converts **< 20 emitted
  functions**, and the honest answer is that **no lane is worth it on this
  family alone**. *p = 0.60.* Registered pessimistic on purpose: board #770's
  streak is optimistic misses, and this is the direction that breaks it if it is
  wrong.

### Hygiene

* **P15** — **no `crates/` change is committed**, and
  `cargo test --workspace --release` is unchanged from master. *p = 0.95.*
* **P16** — **≥ 1 unnamed refusal**, pre-armed on two places:
  1. **WINDOW TOO SHORT** — `CENSUS_HEX_BACK = 16` / `CENSUS_HEX_FWD = 24` does
     not reach far enough to separate the constructs the top sub-keys turn out
     to hold, so a sample has to be re-read from the `.ex` rather than from the
     window; or
  2. **THE KEY ITSELF COSTS** — the compound key inflates the `--jsonl` or the
     scan enough to matter, or a name/field in it needs escaping the JSON writer
     does not do.

  *p = 0.75.*

---

## §2 What this lane will NOT do

* **No `crates/` change is shipped.** The instrument is reverted; its diff is
  quoted in the rung.
* **No fixture is authored.** `Fixtures: none`, `Census: +0` literally.
* **No lowering, no recognizer, no widening.** This is a pricing lane. If the
  decomposition says a rung is worth taking, it is *priced and handed on*, not
  taken here.
* **No new top-level doc.** The findings live inside
  `docs/rungs/2026-08-09-w-jump.md`.
* **No `DISCLOSURE.md` row.** `WB_READER_FINDINGS.md` is used for
  **navigation** — which operand class opcode `3A` is in — and any constant it
  supplies is quoted in prose only, never in `crates/`.
