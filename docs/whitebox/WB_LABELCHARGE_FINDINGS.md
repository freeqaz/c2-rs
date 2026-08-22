# WB-LABELCHARGE — read R3 graded: the SITES are closed, the CHARGE is not, and `LABEL_SEED_GAP` is not a constant

> **PROVENANCE.** §1–§4 are read from a static analysis of Microsoft's
> `c2.dll`, image sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> **verified before the first VA was quoted**, on the repo path and on the flat
> export's input. §5–§6 are **obj bytes** from real `cl.exe` 16.00.11886.00
> under wibo and stand without §1–§4. The spec is
> [`ref/P_LABEL.md`](ref/P_LABEL.md); the prereg is
> [`WB_LABELCHARGE_PREREG.md`](WB_LABELCHARGE_PREREG.md), committed as the
> **first commit on this branch** (`521175b43`) before any site was read.

Lane `w-read-r3`, 2026-08-22. Board **#3387**–**#3390**.
Read **R3** of [`READ_PLAN_2026-08-21.md`](READ_PLAN_2026-08-21.md) §3, priced
2–4 days, funded by the owner
([`../DECISIONS_2026-08-22.md`](../DECISIONS_2026-08-22.md) decision 1).

---

## 0. The headline, in five sentences

1. **The site population is closed.** The allocator `FUN_10b97dd0`'s VA occurs
   **zero times** as a 4-byte absolute anywhere in the image, so its **31
   direct call sites** are the whole population; the label constructor
   `FUN_10b9a455`'s **132** likewise. Both counts were re-derived from the
   pinned image by a raw `E8 rel32` scan and agree with the Ghidra export
   exactly.
2. **The charge is NOT closed.** **42 of those 163 sites sit on loop back
   edges** — including `0x10b5cee1`, a nested loop over a 1,024-bucket symbol
   table that gives a number to *every* symbol c2 minted. A TU's charge is a
   sum over a data-dependent population, not a per-construct constant.
3. **`LABEL_SEED_GAP = 9` is not a constant.** Measured over 22 cells it is
   **7** without `/Og`, **9** with it, and **10** at `/O1`/`/O2`/`/Ox /GF`
   when a string literal is pooled in the data phase. `/Od` is one of the 18
   graded lanes.
4. **The defect is latent, not live — checked rather than argued.** All 21
   `/Od` matches are data-only TUs that emit no `$M`, and the `/O1`+string
   shape returns `Port=NotImplemented` with the reference replay byte-exact.
5. **Naming never charges.** The formatter `FUN_10b99dfe`'s call subtree is
   201 functions and contains **zero** calls to the allocator.

---

## 1. The prereg, scored

**17 graded predictions, 12 HIT, 4 MISS, 1 UNGRADED.** Misses are the content
and are not smoothed.

| # | prediction | outcome |
|---|---|---|
| **P1.1** | every write to `DAT_10c2edd0` is the increment or a seed install; no third kind | **HIT** — 7 references image-wide, 3 writes: `0x10b97807` (IL directive `0x16`), `0x10b97ca1` (`max(IL, current)`), `0x10b97de5` (`+1`). No `add [mem],k`, no decrement, no reset |
| **P1.2** | one reachable path to `0x10b97de5`, exactly +1 per returning call | **HIT** — the only branch leaves through the non-returning ICE `0x37` call |
| **P1.3** | **no** allocator call site lies on a loop back edge | **MISS — and it is the lane's most valuable miss.** **3 of 31** do: `0x10b5cee1` (`hash.c`, nested, `0x400` buckets × chain), `0x10b9a8d9` (the intern probe), `0x10bdbb37` (a tuple-list walk). And **39 of 132** constructor sites do. See §2 |
| **P1.4** | 31 allocator sites reproduces | **HIT** — 31, twice, by two independent methods |
| **P1.5** | 132 sites / 86 callers reproduces | **HIT** — 132 / 86, exactly |
| **P1.6** | no indirect route; the allocator's address is never taken | **HIT** — 0 occurrences of `10b97dd0` as a 4-byte LE value anywhere in the image; 0 for `10b9a455` too |
| **P2.1** | ≥ 20 of 31 read to *(caller, guard, object kind)* | **HIT — 31 of 31**, `P_LABEL.md` §3 |
| **P2.2** | the sites partition by object kind; the kind predicts the charge, not the construct | **MISS.** They partition by kind *for 28 of 31*, but the three loop-resident sites charge **per element of a walked population**, which is a count and not a kind. The prediction's own falsifier — *"a site whose charge depends on a count"* — fired |
| **P2.3** | `FUN_10b9a455` is the busiest site; the other 30 are a structural minority | **HIT, with the limit named.** 22 of the remaining 30 are once-per-call section/symbol builders and 6 more charge **only off the default segment**; the read cannot count dynamic executions and does not claim to |
| **P2.4** | at least one site is guarded by a first-time / already-minted test | **HIT** — and it is not a flag but a **hash table**: `FUN_10b9a897` at `0x10b9a8d9` charges only in the `bucket == 0` arm of a 128-slot open-address probe on `DAT_10c67db8`, `idx = FUN_10b8a01b(name) & 0x7f`. Plus three explicit first-time flags: `DAT_10c45d6c`, `DAT_10c2e460`, `DAT_10c46b5c` |
| **P3.1** | ≥ 5 of `LABEL_COUNTER.md` §1.1's 7 surcharge rows explained by the sites | **HIT — 6 of 7.** The two zero rows are read (`[R]`), the four positive minting rows are inferred from the intern site (`[I]`) |
| **P3.2** | the two zero rows are *absence of a site*, not a subtraction | **HIT** — an IL-named callee takes `sym[+0x28]` from `FUN_10c1f91b` with no constructor call; a re-used helper hits the intern probe and never reaches the charging arm |
| **P3.3** | the signed `>`/`<` `+2`, which mints nothing, is two `FUN_10b9a455` calls | **MISS — not located.** Registered in advance as the row most likely to miss, and it missed. It remains the one surcharge this read does not explain |
| **P3.4** | ≥ 4 of "the nine" named by call site | **MISS.** The candidate population is *bounded* to five named once-per-TU sites, and the gap is shown to be **mode-dependent** — which refutes "nine fixed allocations" — but no unit is attributed to a site. That needs a live tap on `0x10b97de5`, unbuilt |
| **P3.5** | the `/Gy` `+3` explained by three identified sites | **MISS.** Re-confirmed as an exact `3 × nfuncs` in every `/Gy` cell (`[O]`), but *what* the three are is still unread — the same status `WB_LABEL_FINDINGS.md` §6 open #2 left it in on 2026-08-09 |
| **P4.1** | the formatter's subtree contains zero charges | **HIT** — 201 functions reachable, 0 calls to `FUN_10b97dd0`, 0 to `FUN_10b9a455` |
| **P4.2** | the formatter is a pure function of `+0x30`/`+0x31`/`+0x43`/`+0x4d` | **HIT, with one addition**: the kind-1 **named** arm is additionally gated on the 3-bit linkage field `((sym[+0x37] >> 0x15) & 7) ∈ {1,3}` — the same field `P_SYMBOL.md` §3 found at `0x10b28bb4`. And the `@<function>` suffix is switched by the formatter's **second argument**, not a field |
| **P4.3** | `$LC`/`$LL`/`$LN` off `sym[+0x43]` bits `0x10`/`0x4` | **HIT**, re-derived rather than inherited |
| **P5.1** | ≥ 90 % of absolute `$M`/`$T` numbers predicted from `seed + gap + 3n + Σ` | **UNGRADED.** Superseded by P5.4 going red: the `gap` term is not a constant, so a prediction using a single fitted value would grade the *port's* constant, not the read. §5 reports the seed-gap grid instead, which is the same measurement with the confound removed |
| **P5.2** | the instrument self-test: counterfactual moves, true charge does not | **HIT — GREEN.** `s_loc2` counterfactual **+2**, `s_loc8` **+8** — the banner's two published numbers to the digit — with the true in-TU charge **0** on all five cells. §6 |
| **P5.3** | the dedup cell costs 0 and the read names the mechanism | **HIT** — `gpr3` 7, `gpr3-dup` **5**, `gpr3-dup-wide` 7; `const1-led` 7, `const2-led` 9, `const1-dup-led` **5**. Mechanism: the intern probe at `0x10b9a897` |
| **P5.4** | **the `LABEL_SEED_GAP = 9` MOVES** | **HIT — and this is the finding the lane owes the port.** 7 / 9 / 10 over 22 cells, §5 |
| **P6.1** | the listing stays closed as a route to the charge, *with a reason* | **HIT** — two counters, two increment sites (`0x10b97de5` vs `0x10b9a483`), two fields (`sym[+0x28]` vs `sym[+0x3f]`), two populations |
| **P6.2** | the listing discriminates IL-supplied labels from c2-invented ones | **UNGRADED — not attempted.** Named here rather than quietly dropped |

---

## 2. The miss that matters: closure buys the sites, not the charge

`READ_PLAN` §3 row R3 promised *"the enumerated charge rule for
`DAT_10c2edd0` — **closed by construction** (one increment instruction),
replacing the fitted `+9`/`+3`"*. Three claims are packed in there.

* **One increment instruction** — true, and stronger than stated: the counter
  has **7 references image-wide**, of which **3** are writes.
* **The call sites are the whole population** — true, and this read supplies
  the argument that was missing. A direct `call` encodes a *relative*
  displacement, so a function whose absolute VA never appears as data cannot be
  reached indirectly. `dump_label_sites.py --refs 10b97dd0` returns **zero
  occurrences**. Without that check, "enumerate the 31 sites" is a sample.
* **Therefore the charge is a closed-form constant per construct** — **false.**

```text
  loop-resident charging sites
    allocator   FUN_10b97dd0    3 of  31    0x10b5cee1  0x10b9a8d9  0x10bdbb37
    constructor FUN_10b9a455   39 of 132    incl. 8 in FUN_10be4f28 (except.c)
                              --------
                              42 of 163
```

`0x10b5cee1` is the decisive one. In `FUN_10b5ceb5` (`hash.c`) it sits inside
`do { ... } while (i < 0x400)` over a bucket table, with an inner chain walk,
and it does `sym[+0x28] = FUN_10b97dd0()` for **every** symbol except kind 1
with linkage 3. That is the bulk symbol-numbering pass, and it is why
`LABEL_COUNTER.md`'s `stride == minted` observation holds: c2 gives one number
to each record it minted itself.

> **So the honest statement of the rule is a sum, not a table.**
> `charge(TU) = |objects c2 constructs itself|`, over an object population the
> port would have to reproduce. That is a larger obligation than "replace two
> fitted constants", and a lane pricing Phase 1 off row R3's sentence would
> have under-priced it.

---

## 3. What is genuinely new, beyond the enumeration

| # | finding | where |
|---|---|---|
| 1 | **`$M` is minted at `0x10c21992`** — a 20-byte function in `vlines.c` that calls `FUN_10b9a455`, sets `+0x43 \|= 1`, stamps `+0x31 = 'W'`, and attaches the label to a tuple via `FUN_10c21df3` → `FUN_10bd3824`. **One charge per `$M`, one address.** | `P_LABEL.md` §5.1 |
| 2 | **`$T` is minted at `0x10b9b701`** (`FUN_10b9b6a4`, anonymous kind-1, `+0x31 = 0x26`), reached from the `.pdata` record writer `FUN_10c217fd` through `FUN_10b9c655('\6',8,4,0,'\4',0x80)` | ibid. |
| 3 | **A reserved low-id region.** Six of the 31 sites charge **only** when the section is not the default segment; for the default they use hardcoded ids `0x0d`, `0x0f`, `0x16`, `0x17`, `0x19`, `0x1a`, and `FUN_10c1252c` uses `0x1b`. All far below any seed. **This is why a `.data`/`.bss`/`.rdata` global moves the seed gap by zero** — measured | `P_LABEL.md` §3.1, §4.2 |
| 4 | **`lur.c` holds six label constructors and is unread** — a mechanism for `LABEL_COUNTER.md` §7.7 open #3, the `/Ox` loop charge with four magnitudes and no rule | `P_LABEL.md` §7 |
| 5 | **`fg.c` holds eight** — and `fg.c` `0x10b36133` is where R8 starts. The plan's *"R8 waits for R3 — they are the same deliverable from opposite ends"* now has a concrete overlap | ibid. |
| 6 | **The formatter's kind-1 named arm is linkage-gated** on `((sym[+0x37] >> 0x15) & 7) ∈ {1,3}` — the same field `P_SYMBOL.md` §3 found suppressing COFF records | `P_LABEL.md` §6 |

---

## 4. Corrections to standing documents

| document | claim | correction |
|---|---|---|
| `READ_PLAN_2026-08-21.md` §3 row R3 | *"closed by construction (one increment instruction), replacing the fitted `+9`/`+3`"* | the **sites** are closed; the **charge** is a data-dependent sum (§2). The `+9` is **refuted**, not replaced (§5); the `+3` is re-confirmed and still unexplained |
| `crates/c2-core/src/coff/label.rs:9` | `LABEL_SEED_GAP: u32 = 9` | **not a constant** — 7 / 9 / 10 (§5). Latent, not live (§5.3). **Not edited by this lane**; docs-only |
| `WB_LABEL_FINDINGS.md` §1.4, §6 open #1 | *"nine allocations … whether it moves for a TU with different section needs is **unvaried**"* | **varied, and it moves — but not with section needs.** It moves with `/Og` and with a `/GF`-pooled string. Section needs move it by **zero**, and §3.1 says why |
| `WB_LABEL_FINDINGS.md` §1.2 | the formatter's field table | correct as far as it goes; add the linkage gate on the kind-1 named arm and the second-argument switch on the `@` suffix (§3 row 6) |
| `scripts/gt_capture.sh` header | *"`../wibo/build/wibo` is a **stale 1.0.1-7** build that produces wrong objs — do not point this at it"* | **stale comment.** That path is **wibo 1.0.1-23** on this box today, which is the build `LABEL_COUNTER.md` §0 says its whole table was captured with. Both it and the repo-default `../wibo/build/release/wibo` (1.2.0-c2rs.1) reproduce §1's stride table identically here. Reported, **not edited** |

**No `crates/` file is touched and no `DISCLOSURE.md` row is owed** — nothing
is adopted. Explaining or refuting a black-box-fitted constant incurs no debt;
*replacing* it with a disassembly-derived one does
(`WB_LABEL_FINDINGS.md` §8's own tiering).

---

## 5. The control that went red: `LABEL_SEED_GAP`

`scripts/gt_label_seedgap.py`. Seed read directly out of the captured `.gl`
(`u32_le(.gl[7..11])`), first label read out of the obj from the same flags,
so the seed cannot hide inside the answer. **This is not the counterfactual
form**: nothing is differenced against another source text.

### 5.1 The result `[O]`

Two framed functions in every cell; nothing but data or flags ahead of them.

| mode | base | + `const char* g = "x";` |
|---|---:|---:|
| `/Od`, `/Os`, `/Ot`, `/Oy`, `/Ob2` | **7** | **7** |
| `/Og`, `/Ox`, `/Ox /Gy` | **9** | **9** |
| `/Ox /GF` | **9** | **10** |
| `/O1`, `/O2` | **9** | **10** |

> **`LABEL_SEED_GAP = 7 + 2·[/Og] + 1·[/GF ∧ a string pooled in the data
> phase]`**, over 22 cells.

**Nine of the fourteen source/flag cells move it by nothing**: an initialized
global, an uninitialized global, an externally-visible const, a 64-element
array, a 4 KiB `.bss` array, three globals at once, `/Gy`, `/GS` on, `/EHsc`,
`/GR`, `/Oi`, and the workload's whole `/Oi /EHsc /GR` cluster.

### 5.2 The confound, named because this lane hit it

The **first** version of the grid put a string-returning *leaf* ahead of `f0`
and read gaps of 9, 10, 11, 12, 13, 14 — every one of which is
`9 + the functions in front`, because a function ahead of the first framed one
consumes its own slots. **A moving gap that was nothing of the kind.** The
grid now adds only data or flags, and any cell with a function ahead of `f0`
is reported **VOID** rather than graded; the guard is in the script and the
episode is in its source comment. This is `LABEL_COUNTER.md` §7.6 step 5 —
*"a once-per-TU slot is invisible if the subject is first"* — from the other
side.

### 5.3 Latent, not live — checked, not argued `[O]`

* `scripts/mode_lane.sh /Od` → `LANE-RESULT PASS flags=[/Od /GS- /c ]
  graded=386 total=386 **match=21 mismatch=0**`, and **all 21 matching TUs are
  data-only or empty** — `mvp_empty`, `wa16_bss_*`, `wa16_data_*`,
  `walign_data_*`, `wnpos_provide_*`, `worder3_bss_*`, `wsect_*`. Not one emits
  a `$M`, so the seed never reaches an obj at `/Od`.
* `work/w-read-r3/probe/gapstr.cpp` — a file-scope `const char* g3 = "x";`
  ahead of two framed functions — through `c2rs diff`:
  `ReferenceReplay=ByteExact (ref=1437B replay=1437B) **Port=NotImplemented**`.

> **Nothing the port emits today is wrong, and the constant is still wrong as
> stated.** What is live is the **licence**: `LABEL_SEED_GAP` reads as a
> compilation-independent constant. The first rung to admit a framed function
> at `/Od`/`/Os`, or a `/O1` TU with a file-scope pointer-to-string
> initializer, inherits a wrong `$M` on **every function in the TU** — six
> wrong bytes in an obj that still links, which is the whole reason
> `LABEL_COUNTER.md` exists.

---

## 6. The instrument's own self-test, and what it could not catch

`scripts/gt_label_seedgap.py --selftest`, `/Ox /GS- /c`, both forms side by
side on five cells:

| cell | counterfactual | TRUE in-TU | the banner |
|---|---:|---:|---|
| `s_ctl` | +0 | +0 | — |
| `s_loc2` — 2 unused locals | **+2** | **+0** | published +2 · **OK** |
| `s_loc8` — 8 unused locals | **+8** | **+0** | published +8 · **OK** |
| `s_decl8` — 8 unused `extern` declarations | +8 | +0 | — |
| `s_loc8` with 8 `(void)` casts | **+16** | **+0** | — |

**GREEN.** The two cells `LABEL_COUNTER.md`'s banner publishes by number
reproduce to the digit, and the true charge is 0 on every one. Under the
prereg a MISS here would have voided the whole lane.

### 6.1 What the controls would have caught

* A misread of the **dedup** mechanism. A naive site-count would charge the
  second `__savegprlr_29` user +2; `gpr3-dup` reads **5** against `gpr3`'s
  **7**, and `const1-dup-led` reads **5** against `const2-led`'s **9**. Both
  cells are in the set precisely because they can separate "charges per use"
  from "charges per first introduction".
* A gap constant fitted to one mode. `/Od` reads 7 and would have gone
  unnoticed by any `/Ox`-only measurement — which is every measurement in
  `OBJ_GY_SHAPES.md` §3.5's original 25-TU fit.
* The **confound** in §5.2, which the guard now reports as VOID.

### 6.2 What they could **not** catch, stated rather than discovered later

* **Any site no probe executes.** The enumeration is static; the probes
  exercise a handful of paths. A site guarded by a condition none of these TUs
  meets is `[R]` and stays `[R]`. Concretely: the PGO-gated `$zz`/`$zy` sites,
  the `.cil$` IL-embedding sites, and the intermodule-thunk sites were read
  and **never executed**.
* **Two sites that always fire together.** Nothing here separates "one
  constructor takes two numbers" from "two constructors take one each"; §5's
  `+2` rows are reported as pairs.
* **Order.** No probe here places a label. R8.
* **`/Ox`'s loop charge.** Four magnitudes, no rule, and this lane proposes
  none — it names `lur.c` as the mechanism and stops.
* **A control on a one-label or no-label body could not have failed at all**,
  which is why the probe set was fixed in advance to require a stride ≥ 3
  subject, two functions after it, a dedup cell and an instrument self-test.
  The `/Od` evidence in §5.3 is the sharpest form of this: the reason the port
  is green there is that *every* matching TU is label-free, i.e. the standing
  `/Od` lane is, for this quantity, exactly the structurally-incapable control
  the prereg was written against.

---

## 7. Which `LABEL_COUNTER.md` numbers are charges and which are counterfactuals

Asked for explicitly by the dispatch, and it is the question the banner exists
for.

| numbers | kind | why |
|---|---|---|
| **§1's 28-row stride table, §1.1's surcharge table, §1.2's packed re-run** | **CHARGES** | in-TU differences with `base` measured in the same obj. Re-measured here on 8 rows, all reproducing to the digit |
| **§4.2.1's leaf-loop table, §4.1, §4.2.2's triple** | **CHARGES** | same instrument |
| **§6's 161 inline families and law L′** | **CHARGES** | `stride(N) − stride(0)` of the *same* body with an in-object control on every row (§6.0 names the family-baseline defect it fixed). Not re-measured here |
| **§7.4's `/O1` primitive table and the six holdout compositions** | **CHARGES** | in-the-middle form, `minted` 0 on every row |
| **§7.5's EH row (28 / 25), §7.6a's `label_lead = 7`** | **CHARGES** | in-the-middle, per-symbol, four distinct seeds |
| **§7.2's "counterfactual lead" column** | **COUNTERFACTUALS**, and labelled as such in the source | `Δseed + Δcharge` |
| **`w-json`'s 4, `w-bdnz`'s +7/+8, `w-blockir`'s +10/+13/+11/+15, `w-osfinfo`'s `b`-word rule** | **COUNTERFACTUALS** | reproduced here at `s_loc2` +2 and `s_loc8` +8 with true charge 0 |
| **`ROADMAP_SLICING` §2.3(b)'s +7/+8/+11/+15/+23** | **COUNTERFACTUALS** | board #3368 already struck the ground; `READ_PLAN` §5.1 records it |
| **`OBJ_GY_SHAPES.md` §3.5's `+9`** | a **charge**, and **wrong outside `/Og`** | it is an absolute seed-relative reading, not a displacement — the instrument was sound and the *generalisation* was not. §5 |
| **`w-main` §5's "−2 per TU" for the no-`return` spelling** | **COUNTERFACTUAL** | `LABEL_COUNTER.md` §7.6a already says so and pairs it with the `+1`-per-label in-TU reading |

---

## 8. `DISCLOSURE.md` pre-drafts — nothing is owed today

| tier | finding | what adoption would look like | debt |
|---|---|---|---|
| **TIER 2** | `LABEL_SEED_GAP` is `7 + 2·[/Og] + 1·[/GF ∧ pooled string]` | the *rule* is **black-box measured** (`gt_label_seedgap.py`, 22 cells) and owes nothing; only the **explanation** is disassembly-derived. A rung that ships the mode-dependent gap can cite the script | **none** — but see the note |
| **TIER 2** | `$M` costs one charge, minted at `0x10c21992` | if a rung ever computes a `$M` count from a model of c2's label objects, that model's *shape* is disassembly-derived and owes a row naming `0x10c21992` and `0x10b9a468` | owed **on adoption** |
| **TIER 2** | the reserved low-id region `{0x0d,0x0f,0x16,0x17,0x19,0x1a,0x1b}` | adopting any of those seven integers into `crates/` owes a row naming `0x10be78a8`/`0x10be794d`/`0x10be79fa`/`0x10c1252c` | owed **on adoption** |
| **TIER 1** | the formatter's `$M`/`$T`/`$S`/`$SG`/`$E`/`$LC`/`$LL`/`$LN`/`__unwind$`/`__catch$` selection | observable from any `/FAsc` listing, which ROADMAP §9.8 names as a black-box observable | none |

> **The note on row 1.** The mode-dependence is a *measurement* and the port
> may adopt it as one. What is disassembly-derived is *why* — the reserved
> low-id region, the once-per-TU site set — and that stays in `docs/`.

---

## 9. Reproduce

See [`ref/P_LABEL.md`](ref/P_LABEL.md) §9. Every command there was run by this
lane on the pinned image and the repo-default wibo, and the stride rows were
additionally cross-checked against the older `../wibo/build/wibo` (1.0.1-23,
the build `LABEL_COUNTER.md` §0 was captured with) with identical results.
