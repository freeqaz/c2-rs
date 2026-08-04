# w-roots — the emit ROOT set is READABLE. Bit `0x20` is a perfectly SOUND root oracle and an incomplete one.

    Lane:      w-roots, 2026-08-04, worktree `wt-w-roots` off master `9aeded2`
    Prereg:    rungs/_2026-08-04-w-roots-prereg.md, committed at `0663018`
               BEFORE any measurement of the headline quantity. Scored in §6.
    Ships:     NOTHING under `crates/`. No fixture, no codegen, no widening.
    Status:    FINDINGS. TU match is 8 at both ends.

**One-line answer:** ***PARTIALLY — and the partition is sharp.* Bit `0x20` of
the `.gl` flag word at `sym+0x4c` is a **perfectly sound** root oracle —
**14 662 of 14 662** seeds across 850 real TUs are emitted, **zero exceptions**,
and 19 obj-level mutations through the real `c2.dll` agree (**KA-B 7/8, KA-C
12/12**) — but it supplies only **8.4 %** of `|E|` directly and covers only
**18.8 %** of w-emit's 20.4 % root floor, so my registered root-coverage floor
of 0.55 is **MISSED** and this lane's decline clause fires. Seed + direct-call
closure reaches **74.2 %** of every emitted name at precision **0.99991**
(F1 0.852, **+64.4 pp** over emit-everything). **The `p2/main.c` reading is
CONFIRMED, not corrected** — but `C2_MAP.md` §3E's one open question is resolved
**against** its static reading, and this lane **retracts its own prereg's**
claim to the contrary.**

---

## 0. Provenance — every number on this page

| | |
|---|---|
| c2-rs branch | `wt-w-roots`, based on master **`9aeded2`** |
| c2-rs HEAD at measurement | **`0663018`** (the prereg), clean — **no `crates/` change exists in this lane** |
| harness binary | `14bed9911e10`, tree `0663018` |
| **dc3-decomp HEAD BEFORE the run** | **`940d07dcb0960964ad61aa5f025658f993eb46b2`** |
| **dc3-decomp HEAD AFTER the run** | **`940d07dcb096…`** — **it did not move** |
| c2.dll read | `compilers/X360/16.00.11886.00/c2.dll`, image base `0x10b00000`, `.text` VA `0x1000` @ file `0x400` |
| IL + truth | **reused from w-emit unchanged** (`work/w-emit/{il,truth}`), 876 IL / 850 truth, captured at the same dc3 rev |
| flags | the unmodified workload line, `work/dc3-workload/flags.txt` |
| scratch | `work/w-roots/` (gitignored); scripts force-added as text records |

### 0a. The wibo warning — I changed the host, deliberately, and here is exactly when

At lane start `../wibo/build/wibo --version` reported **`1.0.1-7-g3b0f71c-dirty`**
while its own source tree described **`1.0.1-23-g4a9dd6f`** — **the checked-out
build was STALE**, which is a provenance fact worth recording on its own, since
w-emit reported running under `1.0.1-23`.

* **The headline scan (§2–§4) runs no c2 at all.** It reads w-emit's cached IL
  and cached truth. Its wibo provenance is w-emit's, not mine.
* **The mutation control (§5) does run c2**, and the stale build **could not**:
  it aborts with `missing import lstrcpynA from kernel32`. I **rebuilt wibo** to
  `1.0.1-23-g4a9dd6f` — the same version w-emit measured under — and every
  number in §5 is from that build. The rebuild is a change of the host and is
  disclosed here rather than buried.
* A second `missing import lstrcatW` failure turned out **not** to be wibo: it
  was **my** replay driver failing to create the output obj's parent directory,
  and c2's error path building the message. Recorded because it looked exactly
  like a host limitation for an hour and was not one — **suspect your invocation
  before you believe the result.**

### 0b. Denominators, stated once

* **850** TUs graded, from `work/emitpred/magnitude/truthlist.txt`.
* **The 21-TU quarantine of `_2026-08-02-w-emitpred-prereg.md` is still in force
  and was honoured.** No held-out TU was read. **w-emitpred's one-shot Part-1
  gate is UNSPENT** — see §7, where I pre-registered that I would not spend it.

---

## 1. The claim was verified against the binary before anything was measured

I re-read every load-bearing instruction with `objdump` rather than taking the
lane summary's word. **All three confirm, byte for byte:**

| VA | bytes | meaning |
|---|---|---|
| `10b7f16b` | `8b 50 4c  f6 c2 20  74 05  f6 c2 02  74 21` | `mov edx,[eax+0x4c]; test dl,0x20; je skip; test dl,0x2; je COMPILE` — the walk loop in `p2/main.c` |
| `10b7f199` | `83 48 4c 02` | `or [eax+0x4c],0x2` — the loop sets the DONE bit itself, exactly as `C2_MAP.md` says |
| `10b9bf70` | `e8 a6 39 08 00  83 e0 fb  89 46 4c` | `call varU; and eax,~0x4; mov [esi+0x4c],eax` — **the flag word, verbatim from the IL** |
| `10b9c02b` | `e8 eb 38 08 00  89 46 4c` | tag `0x10` also sets `+0x4c`, without the `~0x4` clear |

**The codec primitives, decoded from the disassembly** (stream pointer at
`ds:0x10c46310`), because no prose description of them existed:

| fn | name | encoding |
|---|---|---|
| `10c1f8fc` | `GetByte` | one raw byte |
| `10c1f90a` | `skipvar` | consume bytes while the high bit is set |
| **`10c1f91b`** | **`varU`** | `b0 \| (b1<<8)` when `b1 & 0x80 == 0`; else 4 bytes, `b0 \| ((b1&0x7f)<<8) \| (b2<<15) \| (b3<<23)` |
| `10c1f9a6` | `i16c` | `movsx` of one byte, unless that byte is **exactly** `0x80` → LE16 follows |
| `10c1f9e9` | `i32c` | `movsx` of one byte, unless exactly `0x80` → LE32 follows |
| `10c1fae7` | `i64c` | same shape, LE64 |
| `10c1fcef` | `blob` | `i16c` length, then that many bytes |
| `10c1fc5b` | `GetCStr` | NUL-terminated, **and it consumes the NUL** |

The record dispatcher was read too: the tag byte indexes `0x10b9c615`, which
indexes `0x10b9c5d5`. **Tags `0x04`, `0x0E` and `0x10` share the handler at
`0x10b9bdcf`** — confirming `C2_MAP.md` §3E's own correction — and tags
`0x0C/0x0F/0x11/0x13`–`0x17` all land on the **fatal-error** arm `0x10b9c5ca`.

## 1a. The one correction, and it goes AGAINST the static read — and against me

`C2_MAP.md` §3E's open item is *"the gate on the owner-index `varU` at the
tag-`0x0e` gap"*: Ghidra and raw asm both say `10b9be6b` reads
`test eax,0x200 / je`, i.e. the owner field is read **only** when the `+0x20`
flags word has bit `0x200` set — and `+0x20` decodes to `0x0005`/`0x0105`/`0x0405`
on **every** workload record, never with `0x200`.

**The workload IL says the field is read unconditionally.** Measured, not
argued — the metric is how many `.gl` name runs chain forward onto a `0x80
<LE32>` that `.ex` independently confirms is a `4F 1F` function start:

| layout | `src__App.cpp` | `Game.cpp` |
|---|---:|---:|
| owner `varU` **gated** on `+0x20 & 0x200` (the static read) | **60** | **41** |
| owner `varU` read **unconditionally** | **6 208** | **6 784** |
| (`model.named_bodies`, for scale) | 6 237 | 6 827 |

Over the whole workload the unconditional layout gates **1 506 595** records
clean. This is `C2_MAP.md`'s own rule applied to itself — *a rule read off the
disassembly that disagrees with the data is wrong, however clean the code
looked* — and here the data is c1xx's own output.

> **I also RETRACT this lane's own prereg §1.** I wrote there that §3E's byte
> walk was "off by one field" at `0x0c4` because the owner `varU` could not have
> been read. **§3E's byte walk was right and my reading of the gate was wrong.**
> Recorded rather than quietly dropped, because the prereg is a dated record.

**What is still unexplained:** *why* the gate does not gate. Three candidates,
none distinguished: a caller inside `10b984c3`/`10b978fd` consumes the field; the
value tested is not the value I think is at `+0x20`; or the arm is reached from a
second entry. **Open, `low`, and named — not closed by assertion.**

---

## 2. The instrument, and why it is a decode and not a model

    Seed(t) = { name of a gate-clean tag-0x0e `.gl` record :
                (flags4c & 0x20) and not (flags4c & 0x02) }

A record is accepted **only** when, decoding forward from the end of its own
name, the field chain lands **exactly** on a `0x80 <LE32>` whose value is a real
`4F 1F` function start in `.ex`. That is a per-record known-answer gate: a
coincidental `0x80 <LE32>` is rejected because nothing chains to it, and a
mis-modelled field desyncs and lands nowhere.

**This is not decoration.** A first cut that located the flag word by the loose
`0x80 <LE32>` value-membership scan alone (the `named_bodies` rule) put **4 of
15** App.cpp "seeds" outside `E` — including `llrintl`, a CRT function with no
body in that TU. All four were **decode false positives**, and the chain gate
removes all four. **The unchained scan would have reported seed precision 0.73
and this lane would have published it.**

**There are no free parameters.** Every field, width and escape is transcribed
from a named instruction. Nothing was chosen by looking at truth. That is the
whole reason §7 declines to spend the held-out gate.

---

## 3. The result — 850 TUs, 174 417 emitted names

| | |
|---|---:|
| TUs graded | **850** |
| `\|U\|` (gate-clean records) | **1 506 586** |
| `\|E\|` (truth) | **174 417** |
| `\|E ∩ U\|` | 173 907 (**99.71 %**) |
| **`\|Seed\|`** | **14 662** (17.2 / TU, median 12, max 170) |
| **`\|Seed ∩ E\|`** | **14 662** |
| **`\|Seed ∖ E\|`** | **0** |

> ### **Seed ⊆ E, exactly, with zero exceptions over 850 TUs.**
> Every single name c1xx marks `0x20` is emitted by c2. The predicate the
> disassembly names is *sound* on real headers — STLport, templates over
> templates, multiple inheritance and all — which is precisely the population
> `PHASE7_PLAN.md` §2's standing caveat says its 172 synthetic cells could not
> reach.

And it is **not** trivially sound by being trivially small: `0x20` is set on
only **0.97 %** of `U`, and the `0x02` DONE mask removes **2** records of
14 664 (**0.014 %**) — so **the DONE bit is c2's own bookkeeping, essentially
never present in the IL**, exactly as `10b7f199` says.

### 3.1 The closure, over w-emit's `26`-edges

| | |
|---|---:|
| `\|P\| = \|closure₂₆(Seed)\|` | **129 430** |
| precision `\|P ∩ E\| / \|P\|` | **0.99991** |
| recall `\|E ∩ P\| / \|E\|` | **0.74200** |
| **micro-F1** | **0.85186** |
| incumbent **emit-everything** (the port's behaviour today) | precision 0.11577, recall 1.0, **F1 0.20752** |
| **delta** | **+64.43 pp** |
| incumbent **never-emit** | F1 **0** |
| per-TU **exact set** `P == E` | **132 / 850 (15.5 %)** |
| closure multiplier over the seed | **8.83×** |

**12 false positives in 129 430.** A seed set closed under direct calls almost
never predicts a name c2 did not emit.

---

## 4. Where it fails, and the failure is on the EDGE side of the boundary

**S3, the registered headline, is a MISS and it is a big one:** of w-emit's
**36 228** root-floor names (20.77 % of `|E|` at my population — w-emit measured
35 608 / 20.4 %, reproduced to within 1.7 %), `Seed` contains only **6 797**,
i.e. **0.188** against a registered floor of **0.55**.

**Before reading that as "`0x20` misses roots", note what `Rfloor` is.** It is
*"emitted, and not reached by a `26`-edge from an emitted body"* — a floor
defined **against the proxy**, not against c2's own reference relation. c2's
closure runs over the **`.gl` per-symbol reference list** (`10b276e4`, *"recurses
over the func record reference list"*), which `PHASE7_VALIDATION.md` §7 records
as carrying **data-symbol references and vftable-to-slot links**. The `.ex`
`26`-token scan carries **neither**.

**So I classified the residual rather than guessing at it** (150 TUs,
characterization only — no predicate changed, no edge kind added, nothing
fitted; MSVC access code = the char after the `@@` closing the qualified name,
virtual ∈ `{E,F,M,N,U,V}`):

| `E ∖ P` — never reached by `closure₂₆(Seed)` | n = 10 097 | |
|---|---:|---:|
| free / file-scope function | 4 503 | **44.6 %** |
| **VIRTUAL member** (reachable only through a vtable slot) | 3 147 | **31.2 %** |
| **`??_G`/`??_E` deleting dtor** (vtable slot, synthesized — `#152`) | 710 | **7.0 %** |
| `$` in the qualified name (template instantiation **or** adjustor thunk) | 565 | 5.6 % |
| other `$` | 446 | 4.4 % |
| undecorated (`extern "C"` / CRT) | 265 | 2.6 % |
| non-virtual member | 264 | 2.6 % |
| static member | 148 | 1.5 % |
| adjustor thunk (access code) | 48 | 0.5 % |

> **38.2 % of everything the closure misses is structurally a vtable slot** — a
> reference kind `PHASE7_VALIDATION.md` §8a repair #1 **deliberately excluded**
> from the `26` extractor because a virtual call ODR-uses the *slot*, not the
> definition. Another **44.6 %** are free functions, the class whose canonical
> unreached example in this corpus is `?EaseBackInOut@@YAMMMM@Z` — a member of
> `Easing.h`'s `gEaseFuncs[]` **static function-pointer table**, i.e. reached by
> an **address-take in a data initializer**, which has no `.ex` body and
> therefore no `26`-token at all. `glgraph.py`'s docstring names that exact
> symbol table as the case the `.gl` reference list *does* carry.

**And the mutation control found the same boundary independently.** The single
KA-B miss (§5) is `?RawAlloc@FixedSizeAlloc@@MAAPAHH@Z` — access code `M`, a
**protected virtual**. Clearing its own seed changed nothing because its vtable
keeps it. That is `#168`'s own amendment reproducing itself inside my control
sample, from a completely different instrument.

For contrast, `Seed` itself is overwhelmingly **out-of-line definitions**:

| `Seed` — what c1xx marks `0x20` | n = 2 759 | |
|---|---:|---:|
| non-virtual member | 1 836 | 66.5 % |
| virtual member | 775 | 28.1 % |
| free / file-scope function | 101 | 3.7 % |
| static member | 39 | 1.4 % |
| everything else | 8 | 0.3 % |

### 4.1 The honest statement of what S3 does and does not settle

**What is settled:** `0x20` alone does not supply w-emit's root floor. 0.188,
measured, against a registered 0.55. **The prediction lost.**

**What is NOT settled, and I decline to settle it here:** whether the shortfall
is `0x20` missing roots or the `26`-proxy missing edges. The residual's shape
says the second is at least a large part of it, but **that is an inference from
a classification, labelled, not a measurement.** The clean discriminator is the
**`.gl` per-symbol reference list** — and §6.1 of the prereg named that as this
lane's *not-measured* list **before any of this was known.** Reaching for it now,
after the registered prediction failed, is precisely the move decline clause 4
forbids. **It is named as the single highest-value next measurement, and left
undone.**

> **A corollary that is worth more than the miss:** *w-emit's "roots must supply
> 20.4 % of every emitted name" is a statement about the `26`-proxy, not about
> the roots.* `Rfloor` counts everything the proxy cannot reach — including every
> vtable slot and every address-taken free function — so it is an **upper bound
> on the root set of unknown tightness**, and no root oracle should be graded
> against it as though it were a target. This lane graded itself against it
> anyway, because that is what it registered, and lost.

---

## 5. Known-answer controls — including 19 obj-level mutations through the real c2

| # | control | registered pass | measured | |
|---|---|---|---|---|
| **KA-A** | reproduce w-emit's population | `\|E\|` and `\|U\|` within 0.5 % | `\|E\|` = **174 417 exactly**; `\|U\|` = 1 506 586 vs 1 508 530 (**−0.13 %**, my gate is stricter) | **PASS** |
| **KA-B** | **clear `0x20` at MY decoder's byte offset on a seeded root leaf, replay through real `c2.dll`** | ≥ **4/6** lose exactly that COMDAT | **7/8** | **PASS** |
| **KA-C** | **set `0x20` on an unseeded, unemitted record, replay** | ≥ **2/3** gain exactly that COMDAT | **12/12** | **PASS** |
| **KA-D** | decode chain gate, fail-closed | ≥ 95 % of TUs gate-clean | **850/850**; `varU` re-encode mismatches **0 of 1 506 595**; duplicate-`.ex` refusals **4** | **PASS** |
| **KA-E** | incumbent gate on the unmodified tree | every incumbent | all reproduced exactly, §8 | **PASS** |
| **KA-F** | dc3 HEAD before/after; wibo recorded | no mid-run move | `940d07dcb096` → `940d07dcb096`; wibo disclosed in §0a | **PASS** |

**KA-B / KA-C in full, because the headline rides on them.** Three TUs, workload
flags, the harness's own capture recipe, replayed through `c2host` under wibo
`1.0.1-23-g4a9dd6f`. **The baseline replay reproduces the pipeline obj with the
COMDAT-leader set identical and the only byte difference being the `-Fo` path
string c2 records in `.debug$S`** (59 bytes on `SHA1.cpp`; 3 bytes when the
basenames match). Every mutation is a **single bit** at the offset `record.py`
reports, and the `.gl` is restored between runs.

| TU | baseline leaders | KA-B | KA-C |
|---|---:|---|---|
| `src/system/utl/PoolAlloc.cpp` | 77 | **1/2** | **4/4** |
| `src/system/math/SHA1.cpp` | 11 | **4/4** | **4/4** |
| `src/system/utl/BeatMap.cpp` | 51 | **2/2** | **4/4** |
| | | **7/8** | **12/12** |

**KA-C is the sharper direction and it is perfect.** Setting one bit on a record
c1xx did *not* mark makes exactly that COMDAT appear in the obj — `12/12`,
across `??$min@H@stlpmtx_std@@…`, `??0?$allocator@D@…`, `??$MakeString@…` and a
`StlNodeAlloc` constructor. **`0x20` is sufficient, not merely correlated**, and
the byte offset my decoder reports is the right byte.

**The one KA-B miss is characterized, not excused** (§4): a protected virtual,
selected as a "leaf" only because the `26`-proxy cannot see the vtable edge that
keeps it. I score it as a miss.

---

## 6. Scoring the pre-registration — 5 hits, 2 misses, and the headline is a miss

| # | registered | measured | |
|---|---|---|---|
| **S1** | seed containment 0.97, [0.90, 1.00] | **1.00000** (14 662/14 662) | **HIT**, at the ceiling |
| **S2** | seed share 0.25, [0.10, 0.60] | **0.08406** | **MISS below** |
| **S3** | **root coverage 0.85, [0.55, 1.00]** | **0.18762** | **MISS, far below → decline clause 3 FIRED** |
| **S4** | closure F1 0.90, [0.70, 0.98]; must beat emit-everything by ≥ 20 pp | **0.85186**, **+64.43 pp** | **HIT** on both terms |
| **S5** | per-TU exact 0.10, [0.01, 0.45] | **0.15529** (132/850); 0.16235 on `E ∩ U` | **HIT** |
| **S6** | seed density 0.03, [0.005, 0.15] | **0.04778** | **HIT** |
| **S7** | `0x20` share of `U` 0.03, [0.005, 0.15] | **0.00973**; `0x02` mask moves **0.014 %** | **HIT**, and the mask is **inert** |

**The declared bias was that the reading is right, and I registered S1 and S3
high so a miss would cost me. S1 hit at the ceiling and S3 missed by a factor of
four.** Both are reported, the miss first.

### 6.1 The decline clauses — one fired, and it is honoured literally

* **Clause 3 (`S3 < 0.55`) FIRED.** Honoured: `0x20` is declared **refuted as a
  *complete* root oracle** in the first line, and **I did not scan the other 31
  bits of `flags4c` against `E`.** Not one. A bit found that way would be a
  coincidence with 31 degrees of freedom, and it is exactly the third-lane
  failure this lane was briefed to avoid. **No other bit is proposed, adopted,
  or reported.**
* **Clause 4 (no instrument tuning) HONOURED.** The name binding, the folding
  rule, the strict/local split and the `26` edge kind are w-emit's as landed. I
  changed **nothing** after seeing S3 fail. The `.gl` reference list — the one
  change that would plainly raise recall, and which the *binary* motivates —
  is **declined** (§4.1) precisely because the motivation to reach for it
  arrived after the number did.
* **Clause 1 (decode trust) NOT triggered:** KA-B 7/8 ≥ 4/6, KA-D 850/850.
* **Clause 2 (`S1 < 0.90`) NOT triggered:** S1 = 1.000, so S4/S5 are reportable.
* **Clause 5 (nothing ships) HONOURED.** No `crates/` change. `PortC2` still
  returns `NotImplemented` outside its class; the gate was not widened.

### 6.2 Registered before the numbers existed, restated against them

* **TU match stays 8.** It did — 8 at both ends, and that was the *pre-registered
  outcome of root work*, not a shortfall.
* **A sound seed does not make a shippable predicate.** 74.2 % recall means one
  emitted name in four is unexplained; a fail-closed `Emit/Skip/Unknown` built on
  this today would refuse most TUs, which is correct behaviour and zero
  conversions.
* **`S1 = 1.000` is not a proof of the predicate.** It is 850 TUs of one corpus
  at one flag set with `-optref` absent — the only path in the image that
  *clears* `0x20` (`FUN_10b27b7f`) never ran here, and is untested.

---

## 7. The one-shot Part-1 gate — NOT spent, as pre-registered

The 21-TU quarantine is intact and w-emitpred's Part-1 gate is **still runnable
exactly once**. The prereg registered this decision *before any number existed*,
for a reason about the object: **a decode has no free parameters**, so there is
nothing for a held-out population to catch, and 850 TUs beat 21.

**The registered reversal condition did not trigger, and I checked it honestly.**
I chose nothing by looking at dev truth: the layout was fixed by the disassembly
and gated on `.ex`, the edge kind and attribution are w-emit's as landed, and
after S3 failed I changed nothing. **The gate is therefore still owed by whoever
first ships a root model that *does* have parameters — and this lane deliberately
did not become that lane.**

---

## 8. Gate — every incumbent reproduced, on a tree with no `crates/` change

| | incumbent | this tree |
|---|---|---|
| `cargo test --workspace --release` | 687 passed, **0 failed**, 25 targets | **687 passed, 0 failed, 25 targets** |
| `cargo build --release` | 0 warnings | **0 warnings** |
| `c2rs selftest` | 219 PASS | **219 PASS** |
| `scripts/gate.sh --jobs 6` | 12/12 PASS, 2 628 verdicts, 0 mismatch | **12/12 PASS, 2 628 verdicts, 0 mismatch** |
| TU match / mismatch / vocab-gap / capture-fail | 8 / 0 / 863 / 7 | **8 / 0 / 863 / 7** |
| A / B / C / D / E | 28 / 338 / 114 / 8 / 2 | **28 / 338 / 114 / 8 / 2** |
| `A∧B∧C` / `A∧B∧C∧(D∨E)` / `B∧C` | 25 / 8 / 107 | **25 / 8 / 107** |
| FRONTIER | 17 | **17** |
| **`census/gate disagreement`** | **0** | **0** |

*Compared on the FAILED count, never the passed count.* Capture cache: 871 hit,
7 miss, **0 POISONED**.

---

## 9. What this lane did NOT measure — named, so absence never reads as success

1. **The `.gl` per-symbol reference list.** c2's closure runs over *that*, not
   over `.ex` `26`-tokens. **Every recall, F1 and root-coverage number here is a
   statement about the proxy as much as about the seed.** Declined post-hoc on
   purpose (§4.1); it is the next lane's whole job.
2. **Which other bits of `flags4c` mean anything.** Deliberately not scanned
   (§6.1).
3. **Tag-`0x10` records**, which also carry a `+0x4c` (`10b9c02b`) but no body.
4. **Why the `0x200` gate at `10b9be6b` does not gate** (§1a). Open, `low`.
5. **`-optref`** (`FUN_10b27b7f`), the only path in the image that *clears*
   `0x20`. The workload never passes it; the pruner is entirely untested here.
6. **Why c1xx sets `0x20`.** This lane reads what c1xx wrote. The rule that
   produces it lives in `c1xx.dll` and is untouched.
7. **Order, and the 21 quarantined TUs.** A right set in the wrong order is
   still a mismatch, and the held-out population is unspent.
8. **The 510 emitted names (0.29 % of `E`) with no gate-clean record.** They
   count against recall here, which is the conservative direction, but they are
   uncharacterized.

---

## 10. Proposed board rows — **numbers NOT minted**

`#196`–`#205` are claimed; next free is `#206` with w-prov's five proposed
there. Same discipline as w-afail and w-emit: **no number minted, no `#N` pinned
in code, `BOARD.md` / `ROADMAP.md` / `rungs/INDEX.md` untouched.** Assign at
merge.

| proposed | item | claim | where |
|---|---|---|---|
| **P-a** | **The emit ROOT set is READABLE, and bit `0x20` is SOUND on real headers** — `Seed ⊆ E` with **14 662/14 662** and **zero** exceptions over 850 workload TUs | `#168`'s reading confirmed at three independent levels: disassembly, workload containment, and 19 single-bit obj mutations through real `c2.dll` | this file §1, §3, §5 |
| **P-b** | **`0x20` is SUFFICIENT, not merely correlated** — setting the bit on an unmarked record makes exactly that COMDAT appear, **12/12** | the sharper direction of `#168`'s mutation test, run on real workload TUs at the byte offset a decoder reports rather than on hand-built leaves | §5 |
| **P-c** | **`0x20` is INCOMPLETE as a root oracle: 8.4 % of `\|E\|`, and 18.8 % of w-emit's root floor** — registered floor 0.55, measured 0.188 | the lane's own headline prediction, refuted; no other bit was scanned in response | §4, §6 |
| **P-d** | **Seed + direct-call closure reaches 74.2 % of every emitted name at precision 0.99991** — F1 **0.852**, **+64.4 pp** over emit-everything, **12** false positives in 129 430 | the first end-to-end emit-set number on real TUs with **zero fitted parameters**; per-TU exact sets on 132 of 850 | §3.1 |
| **P-e** | **w-emit's "roots must supply 20.4 %" is a fact about the `26`-PROXY, not about the roots** — `Rfloor` counts every vtable slot and every address-taken free function, so it is an upper bound of unknown tightness | 38.2 % of the unreached residual is structurally a vtable slot and 44.6 % is free functions of the `gEaseFuncs[]` address-taken class | §4 |
| **P-f** | **The `.gl` owner-index `varU` is read UNCONDITIONALLY** — resolving `C2_MAP.md` §3E's open gate **against** its own static reading (`test eax,0x200`) | 6 208 vs 60 chain-clean records on one TU, 1 506 595 over 850; *why* the gate does not gate stays open at `low` | §1a |
| **P-g** | **An `.ex`-value-membership scan alone is NOT enough to locate the flag word** — it put 4 of 15 App.cpp "seeds" outside `E`, including a CRT symbol with no body; the forward **record chain** removes all four | without the chain gate this lane would have published seed precision 0.73 instead of 1.000 | §2 |

---

## 11. Reproducing every number here

```sh
# 0. the binary reads (no corpus needed)
work/w-roots/dis.sh 0x10b7f15f 70        # the p2/main.c walk loop
work/w-roots/dis.sh 0x10b9bf5c 60        # the flag word, verbatim from the IL
work/w-roots/dis.sh 0x10c1f91b 100       # varU

# 1. the headline scan — reads w-emit's cached IL + truth, runs NO c2
python3 work/w-roots/scan.py  work/w-emit/il work/w-emit/truth \
        work/emitpred/magnitude/truthlist.txt work/w-roots/seed.jsonl 12
python3 work/w-roots/score.py work/w-roots/seed.jsonl

# 2. the boundary characterization (150 TUs)
python3 work/w-roots/boundary2.py 150

# 3. KA-B / KA-C — RUNS real c2.dll under wibo on non-quarantined TUs
python3 work/w-roots/mutate.py src/system/math/SHA1.cpp     6 4
python3 work/w-roots/mutate.py src/system/utl/BeatMap.cpp   6 4
python3 work/w-roots/mutate.py src/system/utl/PoolAlloc.cpp 6 4
```

All scripts are **stdlib-only** and read-only against the corpus (`mutate.py`
writes only inside its own `work/w-roots/mut/` scratch and restores the `.gl`
between runs). `work/` is gitignored; the scripts are force-added as text
records, and no IL, obj or `_CL_*` artifact is committed.
