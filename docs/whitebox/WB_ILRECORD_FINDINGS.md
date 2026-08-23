# WB_ILRECORD — FINDINGS for read R5 (the IL-record → codegen dispatch)

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md).
> **No `DISCLOSURE.md` row is owed** — this lane adopted nothing into
> `crates/`; it changed no `crates/` byte at all.

**Lane** `w-read-r5` · characterization · **Fixtures:** none · **Census:** +0 ·
**predicted reach 0, delivered 0.** Prereg
[`WB_ILRECORD_PREREG.md`](WB_ILRECORD_PREREG.md), committed **first** as
`52fa9d7bd`, before the jump table was parsed. Spec
[`ref/P_ILRECORD.md`](ref/P_ILRECORD.md). Instrument
[`scripts/dump_ilrecord.py`](scripts/dump_ilrecord.py). Board
**#3415**–**#3421**.

---

## 0. The two sentences

**The read plan asked for 189 arms; there are 62, and 94 of the 189 opcodes
route to a single arm that raises C1001 — the real subject is 61 arms serving
95 opcodes, and all 62 were read.**

**The select-vs-decode boundary the read plan said "has never been located"
is located, and it is a number:** every node opcode `FUN_10bc2d7a` mints is
≥ `0x2af`; the encoder's opcode space ends at `0x294`; a probe whose
predicate comes from read R2 and is marked `[O]` accepts **660 of 660** real
machine opcodes and **0 of 25** of the opcodes this dispatch mints.

---

## 1. The prereg, scored

**11 HIT · 5 MISS · 1 SPLIT · 4 UNGRADED.** Misses are listed first and are
not smoothed.

### The misses

| # | registered | measured | why it matters |
|---|---|---|---|
| **P1.2** | "189 is an entry count; distinct targets **[100, 170]**" | The qualitative half is right — 189 is an *opcode* count. The bracket is **wrong by a factor of ~2**: there are **62** distinct arms | I applied R2's 111→79 lesson but assumed the same *ratio*. The two-level byte-index switch collapses far harder than a one-level table. **The registered bracket was itself a fitted extrapolation from a single prior case** — instance six of "ranking instruments measure themselves", applied to my own prediction |
| **P2.3** | "#3359 CONFIRMED — selection happens at or below this dispatch; the boundary does **not** fall between it and the encoder" | **The boundary does fall exactly there, and it is sharper than anyone expected** (§3) | The most useful miss in the lane. I registered the pessimistic reading and the binary disagreed |
| **P4.1** | "`0x27`'s arm tests the `0x4000` bit `0x10b3d581` set" | **Nothing in all 5,080 bytes reads bit `0x4000` of `node+6`.** The constant occurs exactly once in the body, and it is a *function*-flag write on arm 24's intrinsic chain | The reader's `0x27` special case and this dispatch are **not** the same mechanism at two addresses. The consumer is below arm 9 — which is why `0x10bbfebb` is now the highest-value unread function this read exposes |
| **P4.3** | "the ten constructs touch **>10 and <60** distinct arms" | Only **three** constructs have published opcodes; they touch **5** arms | Ungradeable as framed, and scored MISS rather than excused. The prediction silently assumed all ten constructs were keyable to opcodes; `ROADMAP_SLICING` pools C4–C10 into one 241,297-body row with **no opcode list anywhere in the record** (§4.3) |
| **P5.2** | "the construct arms are individually **larger** than the median arm" | S-A median **35 B** vs all-arms median **42.5 B** | Backwards. The mass-weighted constructs have *smaller* arms than average, because the biggest one is a trampoline |

### The split

| # | registered | measured |
|---|---|---|
| **P4.4** | (a) C1 and C3 do not share an arm · (b) C3's three opcodes **do** share one | **(a) HIT** — `0x27`→arm 9, C3→arms 52/53/54. **(b) MISS** — the three bind opcodes get **three distinct arms**, and they do three unrelated things: `0x99` sets bind mode 1 under an option gate, `0x9a` sets bind mode 2 in 12 bytes with no call, `0x9b` mints a symbol |

### The hits

| # | registered | measured |
|---|---|---|
| **P1.1** | extent 5,080 B ± 0, body ends where the jump table begins | **HIT, exactly.** `0x10bc2d7a + 5080 = 0x10bc4152` |
| **P1.3** | every target inside the body | **HIT, 62/62** |
| **P1.4** | dense `op−1` index, `cmp`/`ja` guard, a default arm | **HIT.** `lea eax,[edx-1]` / `cmp eax,0xbc` / `ja 0x10bc4143` |
| **P2.1** | **≤ 40 %** of arms are pure DECODE | **HIT — 17 of 61 = 27.9 %** |
| **P2.2** | `C2_MAP.md:1012`'s *"mechanical, recipe is exact"* is **wrong** | **HIT** (§2.2) |
| **P2.4** | the boundary is interleaved, not a prefix of the opcode space | **HIT.** `0x8d`–`0x90` are DECODE between SELECT neighbours `0x8b` and `0x99` |
| **P3.1** | **≥ 25 %** of arms read a global | **HIT — 17 of 62 = 27.4 %** |
| **P3.2** | **≤ 12** globals cover **≥ 80 %** of references | **HIT — top 12 = 82.1 %** of 28 references over 17 distinct globals |
| **P3.3** | ≥ 1 global already named elsewhere in `docs/whitebox/` | **HIT — `0x10c472e8`**, and at the *same* `+0xcac` offset `WB_READER_FINDINGS` §3.2 step 5 uses. Shared context, not a private one |
| **P4.2** | a `0x27` node carries no `size_index` and no composed type, so its arm must get them elsewhere or not need them | **HIT, and now grounded**: arm 9 *cannot* branch on type or size because the reader's classification tail never ran. Its being a trampoline is forced by the record's shape, not a choice |
| **P6.4** | depth-1 bound; DEFER counted, never guessed | **Delivered. 19 of 61 arms are DEFER**, over **76 distinct callees / 174 call sites** |

### The ungraded, and why

| # | status |
|---|---|
| **P5.1** | **UNGRADED — the stratification became unnecessary.** §P5 fixed three strata (construct-keyed, frequency-ranked, and a seeded random control) against an expected partial read. The population turned out to be 62 arms, not 189, so **the lane read 62 of 62 and 189 of 189 opcodes**. A full-population read is strictly stronger than any sampling plan, and S-C's purpose — giving P2.1 a denominator not chosen for being interesting — is served better by there being no sample at all. `--sample` remains in the instrument, unused |
| **P6.1** · **P6.2** · **P6.3** | **VOIDED — the registered probes rest on a premise this read refuted.** All three predict counts of a *machine opcode* per IL opcode. The read found the arms mint **IR** opcodes and never name a machine opcode, so there is no predicted machine opcode per arm to count. The registered design silently assumed the two-stage model. Replaced by a probe with an independent `[O]`-graded criterion (§5), and the substitution is reported rather than presented as the original plan |

---

## 2. Corrections to standing documents

### 2.1 "189 arms" — five documents, one number, wrong in all five

| document | says | is |
|---|---|---|
| `READ_PLAN_2026-08-21.md` §3, §4 | "the **189-arm** IL-record→codegen dispatch" | **62 arms** |
| `C2_MAP.md:1012` | "needs reading 189 arms of `FUN_10bc2d7a`" | 62, of which 61 are real |
| `STEP5_PRICING_2026-08-21.md:139` | "189 arms … zero arms read today" | 62 |
| `WHITEBOX_LEVERAGE_2026-08-21.md:89` | same | 62 |
| `ARCHITECTURE_PROPOSAL_2026-08-20.md:968` | same | 62 |

`189 = 0xBD − 0x01 + 1`. It was read off `labels/W-IL.tsv:36`'s *"table
0x10bc4152, ops 0x01..0xBD"* as if the table were opcode-indexed. The switch
is two-level: a **189-entry byte table** at `0x10bc424a` mapping `op−1` to an
index in `0..61`, then a **62-entry DWORD table** at `0x10bc4152`.

**Nobody had parsed it.** `ref/ADDR.tsv:755` records `0x10bc4152` as `data`,
**size 4, `unknown`** — that row is corrected by this lane's read.

### 2.2 `C2_MAP.md:1012`'s difficulty claim, graded

The row's difficulty column reads **"mechanical, recipe is exact"**. It
carries no board number and had never been tested. **It is wrong**, and
`ref/P_ILRECORD.md` §3 is the evidence:

- **17 of 62 arms read a `.data` global**; arm 29's *entire* behaviour is
  chosen by the global `0x10c3cf96`, and arms 24 and 52 gate on the
  compiler-options block `0x10c472e8`.
- **Six frame slots carry state across tokens** (`ref/P_ILRECORD.md` §2.1) —
  bind mode, scope depth, terminate flag. An arm is not a function of its
  record.
- **Arm 20 recurses into `FUN_10bc2d7a`**, saving and restoring the global
  `0x10c2e2ec` around the nested walk.
- **Arm 56 (880 B) is a peephole optimizer**, matching node shapes and
  rewriting them.

A recipe is not exact when a quarter of its steps consult mutable state and
one of them re-enters the recipe.

### 2.3 New facts about a table board #1591 already knew

`0x10b25f10`, the per-opcode `u16` attribute table, was read by `wb-reader`
at bit `0x400`. This dispatch uses two more:

- **bit `0x1000` at `0x10bc4023` is the loop-exit bit** — it decides whether
  the token walk iterates or returns.
- **bit `0x8000` at `0x10bc2dda`** gates a pre-dispatch call to `0x10bc0b93`.

### 2.4 Board #3359, refined in both directions

Its **observational half is upheld and now explained**: no tap can see this
seam because the intermediate is an in-memory tree in an opcode range that
never reaches an obj. Its **structural half — "there is no intermediate" — is
wrong**; the intermediate exists and is separable by a numeric test on one
field. Its **pessimistic conclusion survives on different grounds**: a
general IL decode is still not "read the records", but because of 76 callees
and six pieces of cross-token state, **not** because selection is entangled
here.

---

## 3. The result the lane was funded for

`READ_PLAN` §4: *"Per arm: does it select or merely decode — that boundary is
the I1/I2 split the whole 15–45 eng-mo estimate rests on, and it has never
been located."*

**Located.** Full detail at `ref/P_ILRECORD.md` §6. In brief:

1. Every immediate in `[0x100, 0x400]` across all 62 arms was collected
   mechanically — **26 distinct values**. Exactly one is ≤ `0x294`, and
   `0x200` (with `0x400`) appears only as a flag bitmask in an `or`.
2. The constants are confirmed to be *opcodes* by a round trip inside the
   read: arm 20 mints `0x2af`; arms 14 and 56 test for it at `node+4` — the
   field `P_ENCODE.md` §9.2 calls *"opcode-or-address-mode"* on a machine
   tuple. **One field, one numbering, two disjoint ranges.**
3. The probe (§5) confirms it against an independent criterion.

**There are three stages where the project's framing has two.** The dispatch
decodes `.ex` records into an IR tree in the `≥ 0x295` space; something
lowers `≥ 0x295` → `≤ 0x294`; the encoder turns `≤ 0x294` into PPC words.
This *explains* rather than merely notes that `wb-select`'s addresses
(`0x10b022cc`, `0x10b1b1f0`, `0x10bf7c59`, `0x10bfee89`) are a disjoint set
from `0x10bc2xxx`.

---

## 4. Three things the lane did not expect

### 4.1 Half the opcode space refuses

**94 of 189 opcodes route to arm 61**, which is
`ICE("…\be\p2\reader.c", 3295)` through `0x10b33526` — the same C1001 entry
the operand-class `0B` arm uses. They are legal `.ex` tokens the reader
parses; they are not legal *here*. `FUN_10bc2d7a` is **one** consumer of the
`.ex` stream, not **the** consumer.

### 4.2 The largest construct is a 16-byte trampoline

C1 `off-add` (`0x27`) is **696,164 bodies, 33.3 % of the residue**. Its arm is
four instructions: two `lea`s, a `call 0x10bbfebb`, a `jmp`. It decides
nothing.

More generally, **three of the ten constructs — 75.2 % of the residue mass —
reach arms that build no IR node in the arm itself**: C1 defers wholesale, C2
sets five flag bits on the function record and mints nothing, and C3 sets
walk state in two of its three opcodes.

### 4.3 C4–C10 cannot be keyed to arms at all

`ROADMAP_SLICING_2026-08-21.md:162-169` publishes opcodes for C1, C2 and C3
and pools **C4 load-type · C5 temp · C6 lit-type · C7 compare · C8 bitwise ·
C9 materialize-64 · C10 virtual-slot** into a single 241,297-body row. **No
opcode list for those seven exists anywhere in `docs/`.** Keying them needs an
IL-census pass, not a read — reported for a follow-up lane.

---

## 5. The confirmation probe

`READ_PLAN` §5.3: `[R]` means *"the instructions were read correctly"*, not
*"this is what c2 does"*.

**The predicate is not this lane's.** `P_ENCODE.md` §2.1, marked `[O]` by
read R2, states that the encoder's tables are exactly `0x001..0x294` long and
that the base-word table *"is a cheap validity check on any claim that opcode
N is a machine opcode"*; §3 fixes the encode-form range at `0..113`. So
`machine_opcode(N) ⇔ form_table[N] ≤ 113`.

```
CONTROL    machine space 0x001..0x294 : 660/660 = 100.0% pass
TEST       opcodes FUN_10bc2d7a mints :   0/25  =   0.0% pass
BACKGROUND indices 0x295..0x400       :   4/364 =   1.1% pass
```

Re-run: `python3 docs/whitebox/scripts/dump_ilrecord.py <c2.dll> --probe`.

**It could have failed.** Had the minted opcodes been machine opcodes, all 25
would have passed at the control's 100 % rate. The separation is total.

### 5.1 The first probe was wrong, and its trap was already documented

The **first** probe attempted was the mnemonic table `0x10b1b260`. It returns
`blectr` at `0x2b0`, `bltlr` at `0x2b4`, `twle` at `0x2e4`, `twlgt` at
`0x2f4` — apparently **refuting** §3 outright.

Those are the **stride-16 extended-mnemonic table read through a stride-12
walk**: exactly the trap `P_ENCODE.md` §2.1 names, with the same signature it
records (`0x02020202` filler; `cmpwi`/`mtctr` at `0x2c0`/`0x2cc`, neither a
c2 machine opcode). Independently reproducing a peer lane's documented trap
is the reason that section exists, and it is recorded here because a lane
that quoted the first result would have published a false refutation of its
own central claim.

### 5.2 What this probe CANNOT catch — the honest list

Registered in the prereg and re-stated now that the numbers exist:

1. **It tests the range claim, not the arm readings.** It proves no minted
   opcode is encodable. It says **nothing** about whether arm 39 mints
   `0x2fb` rather than `0x2fc`, or whether arm 54's operand order is right.
   **Every per-arm row in `ref/P_ILRECORD.md` §3 remains `[R]`.**
2. **Right bits, wrong operand.** `P_ENCODE.md` §9.6's bound, unweakened.
3. **The structural bar, which no probe in this lane can clear.** This seam's
   output is an in-memory tree in a private opcode range and **appears in no
   artifact**. `READ_PLAN` §3's *"the tap cannot see this seam"* is not a
   tooling gap. This is the prereg's failure mode 1 — named in advance,
   and still not fixable inside the lane.
4. **Static reachability ≠ semantic reachability.** 62/62 targets lie inside
   the body; which arms real IL reaches is unmeasured.
5. **Path-dependence.** No fixture varied target/ABI, PGO, LTCG or `/GL`. Arms
   gated on those are read as if unconditional.
6. **The DECODE/SELECT rule is a reproducible judgement, not a measurement.**
   The rule was fixed before any arm was read and every classification is
   published per-arm so a reader can re-grade it — but P6.1, the only control
   that would have tested it against reality, was voided (§1).

---

## 6. Two numbers for the roadmap, and the one it does not get

**It gets:** the subject is **61 real arms over 95 opcodes**, not 189 arms.
And: **76 distinct direct callees over 174 call sites** sit one level below,
including `0x10bc0fcc` (2,763 B), `0x10bc00a1` (2,282 B), `0x10bbec18`
(1,174 B) and `0x10bbf8f1` (1,128 B); **19 of 61 arms are DEFER**, their
semantics entirely below the depth-1 bound.

**It does not get a re-price.** The two numbers push in opposite directions —
the arm count is ~3× smaller than assumed, the callee surface is unbounded by
this read — and combining them is precisely what #1767 forbids. R2's
discipline governs: it delivered a complete encoder spec and then declined to
lower I2's estimate. **This lane delivers a decode spec and declines to move
I1**, and notes that the re-pricing is not the owner's decision yet either.

---

## 7. Coverage, with its denominator

| population | read | denominator |
|---|--:|--:|
| distinct arms | **62** | **62** (100 %) |
| opcodes dispatched | **189** | **189** (100 %) |
| … real arms (excluding the refusal) | 61 | 61 |
| callees, depth 1 named | 76 | 76 |
| callees, **bodies read** | **0** | 76 |

**The read is complete at the level it claims and empty one level down**, and
that boundary is the honest statement of what R5 bought. The prereg
anticipated a partial read of 189 and fixed three strata to keep the
denominator honest; the population was 62 and the lane read all of it, so the
strata went unused rather than being quietly re-purposed.

**What the unread remainder contains:** not arms — there are none left — but
the 76 callee bodies. On the evidence of the 19 DEFER arms and the four
1 KB-plus callees, that is where the IR tree is actually constructed, and it
is a strictly larger read than this one was.
