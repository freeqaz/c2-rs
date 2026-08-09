# w-nc PREREG — frozen before the first scan

Lane: **w-nc** (PRICING/AUDIT). Board rows **#2380–#2399**.
Frozen at tree `9e67561f` (master tip at lane start), branch `w-nc`.
**Nothing below this line was written after a scan ran.** The first `c2rs gap`
invocation of this lane happens after this file is committed.

---

## 0. The commission, restated as a measurable question

Three consecutive conversion lanes found their **last** blocker was not codegen:

* `w-bdnz` (#1980–#1988) — half the class byte-exact all along, blocked by
  `.sy`'s `plain_int` type list (**#764**).
* `w-blockir` (#2300–#2311) — `fnbyte-exact 4 / differs 0` on every body and the
  **whole obj** graded `mismatch`, one `_fltused` short, because
  `IlFunction::touches_floating_point` had no arm (**#2301**). One line.
* `w-main` (#2260–#2266) — an R1 published as a "formals header" refusal that was
  actually `ex_exit_label` wanting a corroborating byte the segment does not
  carry. A **mislocated** blocker.

The question this lane asks is whether that is a **class with population** or
three accidents. The registered discriminator is:

> **ALL-EXACT-NO-MATCH** — a graded TU with `fnbyte-denominator > 0` whose every
> graded emitted function is `fnbyte-exact` (so `differs`, `refused`, `unbound`,
> `nobytes`, `partial`, `reloc-differs` are all **0**) and whose `class` is not
> `Match`.

That is exactly `w-blockir`'s shape one day before it converted, and it is
computable from a single `c2rs gap --jsonl` run without any new instrument in
`crates/`.

## 1. Registered numeric predictions

| id | quantity | registered value |
|---|---|---:|
| **G1** | `fnbyte-tus-full` on my scan (TUs where `exact == denominator`, match-TUs overridden to full) | **15** |
| **G2** | of the 19 `match` TUs, how many have `fnbyte-denominator > 0` (and so are counted in G1 by the override, not by measurement) | **13** |
| **G3** | **ALL-EXACT-NO-MATCH** = G1 − G2 — the gold population | **2** |
| **G4** | of the FRONTIER 8, how many are ALL-EXACT | **0** |
| **G5** | of the FRONTIER 8, how many carry ≥1 `fnbyte-refused` function | **8** |
| **G6** | of the `≤10` blocked-function band (STATUS reads 27), how many are ALL-EXACT | **1** |
| **G7** | distinct whole-obj / TU-level obligations enumerable from the port's writer + TU gates | **7** |
| **G8** | of the gold population (G3), how many collapse to fewer demangled STEMs than bodies (template replication, `w-band` #2246) | **≥ half** |

Probability form, because a point estimate on a population this small is not a
claim:

* `P(G3 == 0)` = **0.30**
* `P(1 ≤ G3 ≤ 3)` = **0.50**
* `P(G3 ≥ 4)` = **0.20**
* `P(G4 == 0)` = **0.75**

## 2. The registered call on deliverable 4

Deliverable 4 offers two outcomes: a ranked list of finds, **or** the honest
statement that the three known instances were the only ones (the class is
exhausted rather than systematic). Registered, in probability form:

* **P(the sweep finds ≥1 previously-unrecorded instance of the class)** = **0.45**
* **P(the class is exhausted — no new instance anywhere in 871 graded TUs)** = **0.55**
* **P(≥1 find converts a TU for ≤5 lines of `crates/` change)** = **0.20**
* **P(≥1 find converts a TU at all, at any price)** = **0.35**

I am registering the negative as **more likely than the positive**. The reason
is `w-readpx` (#2280–#2293): blocked rows are `fnbyte-refused` **by
construction**, so the discriminating population is only the ADMITTED side, and
the admitted side is small — `factor-d` is 19 and `|D∨E|` is 21, which is
`CEILING.md` §10's "non-codegen headroom has been exactly 2 at every reading".
**If §10's invariant is right, G3 ≤ 2 is forced.** A G3 of 4 or more would be
evidence *against* that invariant and is the outcome I would most want to be
wrong about.

## 3. What would make this lane's result an ARTIFACT

Nine rankings in a row were artifacts (`ranking-instruments-measure-themselves`).
The specific ways this one could be:

1. **The instrument grades what it can reach.** `fnbyte-refused == 0` may mean
   "the port emits every body" or "the instrument declined to ask". Registered
   control: I will print the `fnbyte-decline|*` stage split for every candidate
   and assert the partition sums.
2. **`bodies == TUs` is blind to templates** (#2246). Registered control: a
   demangled **STEM** column on every table, plus a TU-replication column
   (#2000).
3. **A census reading is not a byte reading** (`CEILING.md` §10.2; `w-inlfence`
   #2220–#2227 — fail-open on 845 of 871). Registered control: every verdict in
   the sweep is read from `fnbyte-*`, never from `fn_in_class`.
4. **A columns-do-not-sum table.** Registered control: every table asserts its
   columns against the population total, script-counted.
5. **Reading ≤2 bodies.** Registered control: ≥3 actual bodies per top find.

## 4. Scope and exclusions

* `mmio.cpp` is owned by concurrent lane **`w-ifn` (#2350–#2379)**. I cite its
  mechanism list; I do not re-derive it.
* Docs and `work/` only. Any scratch instrument is reverted and its diff quoted
  in the rung.
* No IL, no objs, no absolute paths committed.
* `work/capture-cache` and `.claude/worktrees` are never globbed or walked.
