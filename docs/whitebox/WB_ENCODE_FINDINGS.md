# WB_ENCODE — FINDINGS for read R2 (the instruction encoder)

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md).
> **No `crates/` change was made by this lane and no constant was adopted**, so
> no `DISCLOSURE.md` row is owed. A later rung that adopts anything from
> [`ref/P_ENCODE.md`](ref/P_ENCODE.md) owes the row in the same commit.

**Lane** `w-read-r2` · **kind** characterization · **Fixtures** none ·
**Census** +0 · **reach** 0, registered and delivered ·
**prereg** [`WB_ENCODE_PREREG.md`](WB_ENCODE_PREREG.md), committed at
`f663fd27b` before any byte of either table or any arm body was read.

**The spec is [`ref/P_ENCODE.md`](ref/P_ENCODE.md).** This file is the grade:
what was predicted, what happened, and the four things in the tree that this
read contradicts.

**Headline.** **73 of 79 arms read** (92.4 %); both tables established with a
control that could have failed and did not; **99.38 %** of 634,457 real `.text`
words predicted from the tables + arms, with all four deliberately-broken
variants of the reading scoring measurably worse.

---

## 1. The prereg, scored

Tier **PREREG** throughout (committed to git before the answer existed in this
lane). **10 HIT · 1 half-HIT · 6 MISS · 0 ungraded** over 17 predictions.
Misses are below in full, and one of them (P4.3) is the lane's most valuable
result. A 10/17 hit rate on a scheme whose whole point is that the guesses
were written down first is the number, not an embarrassment; the four
predictions about *shape* (P1.1–P1.3, P2.3) all landed, and every miss is
about a **mechanism nobody in this tree had looked at**.

| # | prediction | grade | what happened |
|---|---|---|---|
| **P1.1** | ≥80 % of `encode.rs`'s encoders reproduce `base_word[op]` at zero operands | **HIT** | **82/89 = 92.1 %** |
| **P1.2** | every residual explained by a field the port bakes and c2's arm supplies; **0 unexplained** | **HIT** | 7 residuals, 7 explained (`P_ENCODE.md` §8.1) |
| **P1.3** | 0 disagreements in a primary opcode or an `XO` | **HIT** | **0** |
| **P2.1** | the form table holds `1..=0x6f` for every real instruction | **MISS** | range is `0..113`; **21 named machine opcodes carry form 0/112/113** and route to the default. The default is an encoding, not a hole — §2 below |
| **P2.2** | `base_word == 0` and "routes to default" are the same set | **MISS** | they overlap in 3 and differ in both directions: 5 zero base words, 21 default-routed. `emit` and `DCD` have zero base words and *real* arms |
| **P2.3** | all four tables share one opcode index space | **HIT** | confirmed, and the base-word table **detects** the stride-12 trap past `0x297` (`P_ENCODE.md` §2.1) |
| **P2.4** | the survey's 104 distinct forms / top form covering 104 opcodes reproduce exactly | **HIT** | both reproduce exactly |
| **P3.1** | ≥60 of 79 arms are pure field composition — no call, no global branch, no store | **MISS** | **50 of 79**, graded mechanically against the literal criterion. The 29 that are not: 14 with a call, 13 with a branch on a global (`DAT_10c2e978` ×12, `DAT_10c6fd9c` ×1), 4 with a store — overlapping. §6.1 records the segmentation caveat that makes this an upper bound on impurity |
| **P3.2** | every register operand reached by `[op+0x1c]+0x28`, **0 exceptions** | **HIT** | 0 exceptions in 73 arms |
| **P3.3** | exactly two join points, **no third** | **MISS** | **six** distinct `or …,ebx` composition sites. The invariant that survives is stronger and was not what was registered: **one store, one exit, one word, return 4** |
| **P3.4** | immediates take a different path from registers, and a helper is on it | **HIT** (half) | different path confirmed — `[op+0x18]`, never `[op+0x1c]+0x28`. But **neither** helper is on it; the immediate path is inline, and `0x10bf983a` is something else entirely (P4.2) |
| **P4.1** | `DAT_10c2e978` is a read-only mode flag; VMX128 or 32/64-bit named as candidates | **HIT** | 12 reads, 0 writes, registered by address in an option table; it gates the VMX→VMX128 opcode substitution `FUN_10bf98ec` (72 of 84 `*128` opcodes appear in it) |
| **P4.2** | `0x10bf983a` / `0x10bf98ec` are operand-value extractors, not relocation emitters | **MISS** | *neither* is an extractor. `0x10bf983a` is the **condition-code → (`BO`,`BI`) composer** (13-row table, `P_ENCODE.md` §5.2); `0x10bf98ec` is the **VMX128 opcode substitution**. The "not relocation emitters" half is right and useless — the relocation emitters are seven *other* helpers the prediction did not know existed |
| **P4.3** | **relocations are not emitted by `FUN_10bf9f15`** | **MISS — and it is the lane's most valuable result** | §3 below |
| **P5.1** | `w-ildecode`'s standing "20–40 forms cover ≥99 %" | **HIT** | **27 forms cover 99.0 %** |
| **P5.2** | this lane's sharper "≤12 forms cover ≥90 %" | **MISS** | **15** forms cover 90.0 % |
| **P6.1** | ≥95 % of probe `.text` words predicted from tables + arms | **HIT** | 46/46 on the purpose-built probe; **99.3839 %** of 634,457 words on 500 real objs |
| **P6.2** | residuals concentrated at relocated operands | **MISS** | **0** of 124,700 relocated words is a residual. Every residual is a form this lane did not read a mask for |

---

## 2. The four corrections to standing documents

Each is a claim in a live file that this read contradicts, with the address.

**2.1 `READ_PLAN_2026-08-21.md` §4 — "not encodable by this path" is wrong.**
The spec-shape asked for default-routed rows to be *"marked **not encodable by
this path**"*. `0x10bfae1b` is the store-and-return tail; reaching it emits the
base word with no operand fields, which is the complete and correct encoding
for `isync`, `sync`, `eieio`, `sc`, `rfi`, `tlbsync` and 18 others. The
**refusal** is a different address — `0x10bfa81d`, `edx = 0x3d9`, an ICE at
line 985 covering 19 opcodes (the `cr*` logicals, `mcrf`, `mtfsb0/1`, `lswi`,
`stswi`). Two further ICE sites are inside arms: line 702 in form 51, line
1025 in form 65.

**2.2 `WB_MIDDLE_INTERFACES.md` §5.6 — "relocations, 0 cells" is superseded.**
See §3.

**2.3 `READ_PLAN_2026-08-21.md` §3's tail measurement undercounts.** The
survey (and the brief that quoted it) states the residual tail is *"3 call
sites of `0x10bf983a` + 1 of `0x10bf98ec` + 12 `DAT_10c2e978` references"*.
The `DAT_10c2e978` count reproduces exactly (12). The call-site count does
not: the encoder body `0x10bf9f15..0x10bfae2a` contains **15 call sites over
13 distinct targets, plus one indirect** (`ds:0x10c433f8`) —
`0x10bf9788`, `0x10bf97c8`, `0x10bf96d9`, `0x10bf96ea`, `0x10bf9721`,
`0x10bf9758`, `0x10bf976d`, `0x10bf983a` ×3, `0x10bf98ec`, `0x10bf9e55`,
`0x10bf9eb5`, `0x10b33526`, `0x10bd470b`.
The survey measured **two named targets**, not the tail; the difference is
almost exactly the relocation seam of §3. **This does not make the tail
unbounded** — the survey's real claim, that `WB_MIDDLE_INTERFACES.md:644`'s
*"unbounded"* is false, stands and is strengthened: the tail is 13 named
functions, all short, all inside one page of the image.

**2.4 `WB_MIDDLE_INTERFACES.md` §3.3 — `t+0xa` is packed, not scalar.** §3.3
reports `+0xa` as *"a SIZE, not a condition code — and it is `[O]`"*. The
measurement is not contradicted; the wording is too strong. Three arms read
`+0xa` three ways: `&0x1f` as the condition code fed to `0x10bf983a`
(`0x10bfa2b0`, `0x10bfa2d1`, `0x10bfa326`), the whole 16-bit word compared
against `0x1008` (`0x10bfa381`, `0x10bfa3e4`), and `&0xfff` compared against
`8` on the *operand*'s `+0xa` (`0x10bfa393`). It is a packed field whose low 5
bits are a condition code on branch tuples.

---

## 3. The result the lane did not expect: the encoder drives relocations

**Registered prediction P4.3: "the encoder produces a word and nothing else;
the relocation/label half of the emit seam lives elsewhere." It is REFUTED.**

`FUN_10bf9f15` takes a fourth argument at `[ebp+0xc]` — a **relocation sink**.
Every relocation path is guarded by `cmp [ebp+0xc],0`, and with a `NULL` sink
the encoder emits exactly the same word and asks for nothing. Seven helper
paths issue relocations through one indirect vector `ds:0x10c433f8`:

| type code | `IMAGE_REL_PPC_*` | issued by | for |
|---:|---|---|---|
| `0x06` | `REL24` | `0x10bf976d` | `bl` to an external symbol (form 7) |
| `0x0d` | `IFGLUE` | `0x10bf96d9` | the glue slot after that call (form 37, opcode `0x280`) |
| `0x10`+`0x12` | `REFHI` + `PAIR` | `0x10bf96ea` | `addis sym@ha` (form 51) and `lau` (form 30) |
| `0x11`+`0x12` | `REFLO` + `PAIR` | `0x10bf9721`, `0x10bf9808` | `lal` (form 29) and both `D`-form memory composers |
| `0x0f` | `SECREL16` | `0x10bf9758` | `loffs` (form 34) |
| `0x02` | `ADDR32` | form 65 inline | `DCD` |

`0x10b2930b`, called between the halves of every hi/lo pair, is inside
`coffemit.c`'s recovered range.

**Why the tree did not have this, and why that is the interesting part.**
`WB_MIDDLE_INTERFACES.md` §5.6 is explicit and honest: *"`mvp_add3` and
`mvp_two` have **zero** `.text` relocations, so this lane observed the
relocation/label half of the emit seam **not at all**."* Its two fixtures
could not reach any of the seven paths — every one is behind either a
non-`NULL` sink or an external symbol. **The absence was a property of the
fixture set, and it read as a property of the compiler for a day.** This is
the same shape as `ref/README.md` §2's second failure mode ("a claim can be
obj-checked, reproduce perfectly, and still be wrong as stated, because the
fixture lacked the structure that would expose the gap"), and it is why this
lane's own P6 probe was specified with four *required* structural features
before it was built.

**What it costs and what it buys.** It costs P4.3 and it costs
`READ_PLAN` §4's negative section its generality. It buys the first cells the
project has ever had on the relocation half of the emit seam — an item the
read-plan's own §1 lists as **UNREAD, "0 cells"**, and which no probe grid was
funded to attack.

---

## 4. The control that could have gone red

Two independent derivations of the same 32-bit constants — `encode.rs`'s 89
words, recovered one captured obj at a time and never looking at `c2.dll`, and
c2's own `0x10c3a578` — agree on **82 of 89**, with **0 disagreements in a
primary opcode or an `XO`** and all 7 residuals explained by a field the port
bakes into its encoder that c2 contributes from the arm. The seven are in
`ref/P_ENCODE.md` §8.1; `work/w-read-r2/control_p1.txt` is the full listing.

Three of the seven are more than bookkeeping, because c2's arm says out loud
what the port's comment says it derived:

* `encode_mtctr`'s split five-and-five `SPR` field, *"low half first"*, is
  `0x10bfa7a3`'s `(spr & 0x1f)<<16 | (spr >> 5)<<11`.
* `encode_logical_x`'s warning that the destination is in `RA` and not `RT` is
  form 39's arm at `0x10bfa53b`, field for field.
* `encode_bclr`/`BO_ALWAYS` is form 55's one-instruction arm `or ebx,0x2800000`.

**The control went green, and green is the weaker outcome.** It is reported
because it was registered, and because a control that *cannot* fail is not one:
§5 is the version of this that was built to fail and did.

---

## 5. The confirmation probe, and the mutation that only a big corpus could see

`[R]` is a hypothesis. So every arm rule was turned into a bit-mask claim —
*these are the bits arm A owns* — and tested against real c2 output by asking
whether `word & ~armmask(form[op]) == base_word[op]` for some opcode. A
misread field width leaves a bit outside the mask and the residual stops
matching any base word.

| population | words | explained | residuals at a relocation site |
|---|---:|---|---:|
| purpose-built probe (`work/w-read-r2/probe/p6.cpp`, real c2 under wibo) | 46 | **46 (100 %)** | 0 of 7 |
| 500 `dc3-decomp` reference objs | 634,457 | **630,548 (99.3839 %)** | **0 of 124,700** |

The 3,909 residuals are all words of forms with no mask written (`rld*`,
`tdi`/`twi`, `mftb`) — a coverage statement, not a disagreement.

**Four deliberately-broken readings, to show the test can fail:**
`D` field 16→12 bits **91.40 %**; `RB` 5→4 bits **92.32 %**; drop the `RA`
field **73.49 %**; `SPR` unsplit **95.66 %** — against 99.38 % as read.

**The one worth carrying away is `RB`.** On the 46-word purpose-built probe
that mutation changed **nothing at all** — no word there used a register
≥ 16, so a 4-bit `RB` and a 5-bit `RB` are the same hypothesis on that corpus.
The probe was designed against four *named* failure modes (a displacement, a
sign bit, a relocation, an `Rc`/`LK` bit) and it still could not see a fifth
nobody had named. **A control is only capable of failing on the population you
ran it on**, and the 500-obj run is what made this one capable. Recorded
because the small probe would have been quoted as a 100 % result.

**And the limit of the whole method, stated plainly.** §5 confirms *which bits
an arm owns*. It does not confirm *which operand the arm read them from*: a
rule that puts the right bits in the right place from the wrong operand passes
every cell above. Closing that needs the tuple stream, which is read **R5**.
Everything in `P_ENCODE.md` §5 stays `[R]` on that axis.

---

## 6. Two numbers for the roadmap, and one it does not get

**The `w-ildecode` (b)/(c) split is now priceable** — `WB_MIDDLE_INTERFACES.md`
§8.1 and `ROADMAP_SLICING_2026-08-21.md`'s encoder row deliberately left (c)
unpriced *"until (a) runs"*. (a) has now run:

* **(a) dump both tables + histogram** — done. Half a day was the estimate;
  the tables took under an hour, the histogram needed a 500-obj decode.
* **(b) read the arms** — estimated *"~2 days for the stereotyped majority,
  **unbounded** for the tail"*. Actual: **73 of 79 in one session**, and the
  tail is not unbounded (§2.3). The 6 remaining are one family.
* **(c) grade them** — **27 forms cover 99.0 % of emitted words, 15 cover
  90.0 %**, and §5's mask test is a per-word grading surface that needs **no
  fixture per form**: it grades 634,457 words against objs that already exist.
  That is the number (c) was waiting on.

### 6.1 The arms-read denominator, and what "read" means here

**73 of 79.** An arm counts as read when this lane can state its
field-composition rule or its refusal, which is what `ref/P_ENCODE.md` §5 does.
The 6 that do not are named in `P_ENCODE.md` §7 — all single-opcode VMX128
arms of one already-read family. In opcode terms, **653 of 660 machine
opcodes** reach an arm with a stated rule (98.9 %) — the 6 unread arms serve
**7** opcodes (`vcfpsxws128`, `vcfpuxws128`, `vperm128`, `vpkd3d128`,
`vrlimi128`, `vsldoi128`, `vupkd3d128`); in emitted-word terms they cover 0
words of the 634,457 measured.

**The segmentation caveat, because the numbers above rest on it.** Arms were
cut at *"start of this arm, to start of the next arm by address"*. That is an
over-approximation in one direction — a span can swallow a few instructions
that belong to a neighbour (`0x10bfa84e`'s span contains `0x10bfa884` and
`0x10bfa88f`, which are join fragments) — and an under-approximation in the
other, because 34 arms end by jumping into a **shared tail** whose
instructions are then not in their span (`P_ENCODE.md` §4 lists the seven
tails). So P3.1's 50/79 is an upper bound on impurity: an arm can be scored
impure for a neighbour's `call`. The rules in `P_ENCODE.md` §5 were read by
following the jumps, not by trusting the spans.

**What the roadmap does not get from this lane: a re-price of I2.** A read
produces a spec, not an implementation (`DECISIONS_2026-08-22.md` decision 1's
own warning), and `P_ENCODE.md` §9 lists six things I2 needs that this spec
does not contain — starting with the tuple stream, which is R5 and unstarted.
Quoting "the encoder is read" as "I2 is cheaper" would be `#1767`'s error with
a bigger denominator.
