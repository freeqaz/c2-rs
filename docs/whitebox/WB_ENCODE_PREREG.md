# WB_ENCODE — PREREG for read R2 (the instruction encoder)

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md).
> Nothing here may enter `crates/` without a `DISCLOSURE.md` row naming the
> address it came from.

**Lane:** `w-read-r2` · **kind:** characterization lane
(`docs/rungs/README.md` § "Lane kinds" 3) · **Fixtures:** none ·
**Census:** +0 · **predicted reach:** 0, registered.

**Subject.** Read R2 of the funded read-plan
(`docs/whitebox/READ_PLAN_2026-08-21.md` §3, funded by the owner 2026-08-22 —
`docs/DECISIONS_2026-08-22.md` decision 1): the instruction encoder
`FUN_10bf9f15`, its two tables (`0x10c3a578` base word, `0x10c39b18` encode
form), the 111-entry jump table at `0x10bfae2d` with **79 distinct arm
targets**, the 4 helper call sites (3× `0x10bf983a`, 1× `0x10bf98ec`) and the
12 references to `DAT_10c2e978`.

**Image.** `compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` —
**verified by this lane before any address was read** (`C2_MAP_METHOD.md` §0),
and re-verified against `~/ghidra-projects/bin/c2dll` (the export's input),
which matches. The Ghidra flat export is dated 2026-08-04; its input digest
matching the pinned image is what licenses quoting its addresses
(`READ_PLAN` §5.4).

**Prior art this lane must not re-derive** (checked before writing this file):
board **#3358** / `WB_MIDDLE_INTERFACES.md` §5.1–§5.4 already hold the two
table VAs, the dispatch shape (`edx = form; edx--; if (edx > 0x6e) default;
jmp [edx*4 + 0x10bfae2d]`), **two arms read instruction for instruction**
(form `0x31` at `0x10bfa456`, form `0x37` at `0x10bfa2a5`), the
`sym+0x08→+0x1c == operand+0x1c→+0x28 + 1` register-path reconciliation, and
9 words reproduced 32 bits of 32 on `mvp_add3` / `mvp_two`. Those two arms are
**inherited, not re-read**, and are excluded from this lane's own
arms-read numerator unless independently re-derived (they were; see
FINDINGS §0).

---

## The grading rule

Registered **before** any byte of either table or any arm body was read.
Tier **PREREG** by `PREREG.md`'s ladder: committed to git before the answer
existed anywhere in this lane. Each prediction below is scored HIT / MISS /
UNGRADED in `WB_ENCODE_FINDINGS.md`; **misses are reported as misses and are
not smoothed**, and a prediction vague enough to be unfalsifiable earns
nothing and is marked UNGRADED rather than counted.

Numerators are reported with denominators. **`arms-read/79` is the lane's
headline denominator** and a partial numerator is the honest outcome; a
claim of 79/79 reached by skimming would be worth less than 30/79 verified.

---

## P1 — the base-word table reproduces the port's own encoders (THE CONTROL)

`crates/c2-core/src/codegen/encode.rs` is a **black-box re-derivation** of
exactly what `0x10c3a578` states plainly (`READ_PLAN` §2, and the file's own
2026-08-22 banner). That makes it this lane's control in the strongest
available sense: two independent derivations of the same 32-bit constants,
one from captured objs and one from c2's own data.

**Method, fixed here so it cannot be tuned afterwards.** For each `encode_*`
in `encode.rs`, evaluate it with **every register/immediate/displacement
operand zero**. By `WB_MIDDLE_INTERFACES.md` §5.1's characterization
(*"the 32-bit PPC encoding with every operand field zero"*) that value must
equal `base_word[op]` for the opcode whose mnemonic-table entry
(`0x10b1b260`, stride 12) names the same instruction.

| # | prediction | grade if |
|---|---|---|
| **P1.1** | Of the `encode_*` functions that map 1:1 to a c2 mnemonic, **≥ 80 %** reproduce `base_word[op]` exactly at zero operands. | HIT at ≥ 80 %, MISS below |
| **P1.2** | **Every** residual is explained by a field the port bakes into the encoder that c2 contributes from the **arm** (`BO`/`BI`, an `SPR` field, a fixed `SH`/`MB`/`ME`, `LK`, `Rc`) — i.e. **zero unexplained residuals**. | HIT at 0 unexplained; **any** unexplained residual is a MISS *and a finding about one of the two derivations* |
| **P1.3** | **No** case where port and table disagree in the **primary opcode** (bits 0–5) or the **extended opcode** (`XO`). Those fields are operand-free in both derivations, so a disagreement there is a genuine defect in `encode.rs` or in the reading. | HIT at 0, MISS at ≥ 1 |

**P1.3 is the prediction that can embarrass the port**, and it is registered
precisely because it can. `encode.rs` is graded byte-exact on the classes its
fixtures fence; an opcode it never emits on a fixture is a constant nobody
checked.

## P2 — the table's shape

| # | prediction | grade if |
|---|---|---|
| **P2.1** | The form table `0x10c39b18` holds values in `1..=0x6f` for every opcode that is a real instruction, and `0` (or an out-of-range value routing to the `edx > 0x6e` default) for the rest. | HIT / MISS on the observed range |
| **P2.2** | Opcodes whose `base_word` is `0` are exactly the ones whose form routes to the default — i.e. *"not encodable by this path"* is a **single** predicate, not two independent ones. | HIT if the two sets coincide; MISS with the counts if not |
| **P2.3** | The tables are indexed by the **same** opcode number as the mnemonic table `0x10b1b260` and the machine table `0x10b202b0` — one opcode space, four tables. Falsifiable: a mnemonic whose base word is architecturally wrong for it. | HIT / MISS |
| **P2.4** | Over `0x001..0x294` the form table holds **104 distinct values** and the top form covers **104 opcodes** — the survey's numbers (`READ_PLAN` §3 banner), re-measured here as a **reproduction check on the survey**, not as a new claim. | HIT if both reproduce exactly |

## P3 — the arms

| # | prediction | grade if |
|---|---|---|
| **P3.1** | **≥ 60 of 79** distinct arms are *pure field composition*: some sequence of `[operand+0x1c]+0x28` loads, shifts, `or`s and constant `or`s onto `ebx`, with **no** call, **no** conditional branch on a global, and **no** store. | HIT at ≥ 60, MISS below |
| **P3.2** | Every arm that reads a register operand reaches it by the **same two-hop path** `operand+0x1c → +0x28`, giving the hardware number directly (`WB_MIDDLE_INTERFACES.md` §5.3's `n = r+1` identity holds on the *symbol* path, not this one). **Zero** arms use a different register access path. | HIT at 0 exceptions; any exception is a finding |
| **P3.3** | The arms converge on **exactly two** join points, `0x10bfae19` (`or ebx,eax` — a computed field bundle) and `0x10bfae1b` (`ebx` already final). Prediction: **no third join**. | HIT / MISS |
| **P3.4** | Immediate-bearing forms (`D`-form displacement, `SIMM`, `UIMM`, branch `BD`) reach their value through a **different** access path than registers, and at least one of the two helpers (`0x10bf983a` / `0x10bf98ec`) is on it. | HIT / MISS |

## P4 — the escapes, decoded and not named

| # | prediction | grade if |
|---|---|---|
| **P4.1** | `DAT_10c2e978` is a **compile-mode / target flag** — read, never written, inside the encoder — and it selects between two encodings of the same opcode (the VMX/VMX128 or the 32/64-bit split are the two named candidates). It is **not** an output cursor and **not** the current instruction address. | HIT if read-only in the encoder and mode-like; MISS if it is a cursor/address/counter |
| **P4.2** | `0x10bf983a` (3 sites) and `0x10bf98ec` (1 site) are **operand-value extractors** — they turn an operand record into an integer immediate — rather than relocation emitters. Registered against the alternative because `WB_MIDDLE_INTERFACES.md` §5.6 has **0 cells** on relocations and would very much like some. | HIT / MISS, with what they actually do |
| **P4.3** | **Relocations are not emitted by `FUN_10bf9f15`.** The encoder produces a word and nothing else; the relocation/label half of the emit seam lives elsewhere. (This is `WB_MIDDLE_INTERFACES.md` §5.6 restated as a falsifiable claim rather than an absence.) | HIT if no arm and no helper writes a relocation record; MISS otherwise |

## P5 — the coverage question the roadmap row rests on

`WB_MIDDLE_INTERFACES.md` §8.1 and `ROADMAP_SLICING_2026-08-21.md`'s encoder
row both register: **"20–40 forms will cover ≥ 99 % of emitted words."** That
expectation was registered by `w-ildecode` and has never been scored.

| # | prediction | grade if |
|---|---|---|
| **P5.1** | Re-registered unchanged and scored here: **20–40 distinct forms cover ≥ 99 %** of the *instruction words the workload's reference objs actually contain*. | HIT if the count lands in `[20, 40]` at ≥ 99 % coverage; MISS with the true count either way |
| **P5.2** | This lane's own added expectation: the coverage curve is **steeper** than that — **≤ 12 forms cover ≥ 90 %**. | HIT / MISS |

**The denominator problem is named here, not discovered later.** The
workload's objs give a distribution over *emitted PPC words*, which this lane
can decode and bucket by (primary opcode, XO) → c2 opcode → form. That is a
**proxy** for "forms the encoder reaches": it cannot see an opcode c2 emits
through a path other than this encoder, and it cannot distinguish two opcodes
that share one form. Both limits are reported with the number.

## P6 — the confirmation probe (the `[R]` → `[O]` step)

`READ_PLAN` §5.3 and `docs/whitebox/ref/README.md:49`: `[R]` means *"the
instructions were read correctly"*, **not** *"this is what c2 does"* — the
`.bss` bump rule was read correctly and was wrong about c2. So every claim in
the findings that is only `[R]` says so, and the lane ends in a probe.

**The probe, specified before it is run.** Take a fixture the port already
emits byte-exact, take its **real c2 obj**, decode `.text`, and predict each
word **from the tables and arms alone** — base word from `0x10c3a578`, fields
composed by the arm the form selects. Compare word for word.

**The control must be capable of failing, and here is how it is made so.**
`mvp_add3`'s three words (#3358) are all register-only and relocation-free;
a fixture of that shape cannot detect a misread displacement field, a
misread immediate, a wrong `Rc`/`LK` bit, or a relocated operand. So the
probe set is fixed here to require **all** of:

1. at least one **D-form memory** instruction with a **non-zero
   displacement** (detects a misread `D` field or a wrong field width);
2. at least one **immediate** instruction with a value whose top bit is set
   (detects sign/zero-extension confusion);
3. at least one instruction carrying a **`.text` relocation** (detects
   exactly the case P4.3 predicts the encoder does *not* handle — if the
   word the tables predict differs from the obj's word at a relocated
   operand, that is a finding about where the addend lives, not a failure);
4. at least one **`Rc`- or `LK`-bearing** instruction (detects a bit the
   base word and the arm could each plausibly own).

| # | prediction | grade if |
|---|---|---|
| **P6.1** | On a probe set meeting (1)–(4), **≥ 95 %** of decoded `.text` words are predicted exactly from the tables + arms, with every residual named. | HIT at ≥ 95 %, MISS below |
| **P6.2** | The residuals, if any, are **concentrated at relocated operands** rather than spread across ordinary words. | HIT / MISS |

---

## What this lane will NOT claim

- **Relocations are out of scope** and a complete encoder is **not** a
  complete emit seam (`READ_PLAN` §4's spec-shape item 5). If the read says
  something about relocations it will be reported, but the spec's negative
  section stands.
- **No `crates/` change.** Docs-only. The findings are an implementation
  spec for **I2** (`docs/STEP5_PRICING_2026-08-21.md`), not an
  implementation, and a read produces a spec, not an implementation
  (`DECISIONS_2026-08-22.md` decision 1's own warning).
- **No adoption without a `DISCLOSURE.md` row.** Nothing in this lane is
  copied into `crates/`; if a later rung adopts a constant from here it owes
  the row in the same commit.
- **No re-pricing of I2 from this read alone.** The read produces the spec;
  what building against it costs is a separate number, and #1767's rule
  (a 3-cell measurement extrapolated to 111 arms is not an estimate)
  applies to this lane too.

## Registered outcome shape

`built` **only if** both tables are established with a control that could
have failed *and* a non-trivial arm count is read and confirmed. If the
control goes red and is not resolved, or if the probe cannot be built to
meet P6's (1)–(4), the outcome word is **FAILED**, in those words.
