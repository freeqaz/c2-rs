# PREREG — lane `w-encarms`, wave 18 (2026-08-28)

**Committed before the image was opened.** Board `#3756`–`#3761`.
Charter: `docs/ADOPTION_BRIEF_2026-08-28.md` §L1, `docs/DECISIONS_2026-08-22.md`
§ Decision 22 §2.

## 0. What is already known before this lane starts, so a re-read is not sold as a read

`docs/whitebox/ref/P_ENCODE.md` §10.4 **already** classifies the 52 unmapped
arms (25 VMX/VMX128 over 243 opcodes; 27 non-VMX over 100), and §3.2 already
says the default arm `0x10bfae1b` is an encoding and the refusal is
`0x10bfa81d` (ICE line 985, 7 forms, 19 opcodes). The brief's framing —
*"'104 opcodes, one form' is as consistent with a table-driven general encoder
as with a refusal path"* — is **already answered on the page** for
`10bf9f91` (form 78 = VMX three-register, an encoding).

So the read this lane owes is **not** "what are the arms". It is the one
question §10.4 explicitly does not answer and the brief names as the deciding
one:

> **which of the 52 does the 878-TU workload actually need?**

`subsys.rs:669`'s own caveat states the hole in those words: the `encode` row's
`exercised` cell *"Says nothing about which of the 79 arms the workload takes"*.

## 1. The instrument

`work/w-encarms/armhist.py` — for every executable `.text` word of the real-`c2`
reference objs of the workload, attribute the word to a **c2 opcode**, hence a
**form** (table `0x10c39b18`), hence an **arm** (`ENCODE_ARMS.txt`), and
histogram over the 79 arms.

* **Population, named up front.** `build/373307D9/src/<tu>.obj` of the sibling
  `dc3-decomp` tree, intersected with the 871 `src` rows of
  `work/w-bss/census/sections.jsonl`. **861 of 871** census TUs have such an
  obj; the 10 that do not are printed, not dropped silently (`#1002`).
  These are `c2.dll`'s own output over the graded workload — a strictly better
  denominator than `P_ENCODE` §8.2's 500 `dc3-decomp` objs, which the page
  itself flags as *"not the 878-TU workload manifest"*.
* **Attribution** is by skeleton: clear the operand-field bits implied by the
  word's PPC form, and look the residue up in c2's own base-word table
  (`0x10c3a578`). A word whose skeleton is not a base-word table row is
  **unattributed and counted as such** — this instrument prints its own
  denominator and its own residue.

## 2. Predictions — registered, and scored in the rung whether they hit or miss

**P1.** Over the workload objs, **strictly fewer than 20 of the 52 unmapped
arms are reached at all**, and **more than half of the 52 are reached zero
times**. Registered because the brief's implicit premise — 52 unported arms is
52 conversion opportunities — is the `#3723` trap in a different costume.

**P2.** The two most-reached unmapped arms are `10bfa285` (form 7, `bl`) and
`10bfa76a` (form 54, `mfspr` / `mflr`), each ≥ 0.5 % of attributed words.
These are the two `P_ENCODE` §10.5 already flags as *"the port DOES emit this
and does not go through the arm"*.

**P3 — the sharp one, and it is a control on my own decoder.** Arm
`10bfa81d`'s 19 opcodes (the eight `cr*` logicals, `mcrf`, `mcrfs`, `mcrxr`,
`mtfsb0/.`, `mtfsb1/.`, `mtfsfi/.`, `lswi`, `stswi`) appear **zero** times in
the workload's `.text`, because `P_ENCODE` §3.2 reads that arm as an
unconditional ICE. **A non-zero count here falsifies either §3.2's reading or
my decoder, and I must say which before quoting any other number from it.**

**P4.** `10bf9f91` (form 78, 104 opcodes) is an **encoding**, not a refusal —
confirmed by reading the arm body in the image, not by quoting §3.2 — and it is
reached by **< 1 %** of attributed workload words.

**P5 — adoption.** The set of arms this lane can adopt **under the byte judge**
is small and I name its size now: **2**, being `10bfa285` and `10bfa76a`. Both
qualify on a criterion nothing else meets — *the port already emits that
instruction on a live path, from a hand-baked word, so the corpus exercises it
and the byte judge can grade the adoption.* I predict `[encode] ported` moves
**27 / 79 → 29 / 79** and the byte delta is **zero**.

**P6.** No other arm of the 52 is adoptable this wave, and the reason is
uniform: the port has no lowering that produces the instruction, so adding a
plan would move the published numerator without the port being able to reach
the arm from any emit path — decision 21 §4's `#3505` hazard in miniature.
**If P6 holds, the honest headline is 2 adopted of 52 characterized, not 52
opportunities.**

## 3. Controls, each of which must be WATCHED RED before its verdict is quoted (`#3336`)

| control | planted defect | expected RED |
|---|---|---|
| **C-A** attribution | clear one bit too many in the `RB` field mask | attributed fraction drops measurably |
| **C-B** ICE prediction | force `crand`'s form to `10bfa81d`'s and re-run P3 | P3's count goes non-zero, so P3 can fail |
| **C-C** byte judge | flip one bit of the adopted `bl` field plan | `scripts/gate.sh` / suite goes RED |
| **C-D** surface registry | perturb the adopted arm's row | `surface/DOMAIN.txt` test goes RED |

C-C is the load-bearing one: it is the assertion that the corpus **does**
exercise what I adopted, which is exactly what `#3723` says a green byte delta
does not establish on its own.

## 4. State changes each outcome licenses

* **P5 holds** → adopt the 2 arms into `mop.rs`, file 2 `DISCLOSURE.md` rows
  naming `0x10bfa285` and `0x10bfa76a`, register one `c2_core::surface` row
  whose domain covers **all 79 arms** (so the 52 refusals are enumerated text a
  reviewer can read), and report `converted`… *no* — the rung kind is
  **construct**, so the outcome word is `built`.
* **P5 misses low (0 adopted)** → the outcome word is `FAILED` if nothing else
  landed, or `instrument` if the histogram landed and the adoption did not.
  I will not manufacture an adoption to avoid `FAILED`.
* **P3 fails** → I stop and adjudicate §3.2 against my decoder before any other
  number from the histogram is published.

## 5. Fences

OWN: `crates/c2-core/src/codegen/mop.rs` and the encoder path,
`crates/c2-core/src/surface.rs` (**additive block at the end only** —
`w-inlbudget` owns this file too), `work/w-encarms/`, `docs/whitebox/DISCLOSURE.md`
(append only), board `#3756`–`#3761`, `docs/rungs/2026-08-28-w-encarms.md`.

MUST NOT TOUCH: `codegen/splice.rs`, `docs/whitebox/ref/P_INLINE.md`,
`work/w-inlmetric/CLAUSES.tsv`, `docs/whitebox/ref/P_GLOBREGS.md`,
`docs/STATUS.md`, `docs/rungs/INDEX.md`, `scripts/gate.sh` (`#3691` — no 22nd
count-bearing row, from anyone).

**Open question I am registering rather than deciding now:** `calls.rs` and
`frame.rs` are the eleven duplicate word producers of `P_ENCODE` §10.5 /
board `#3637`. Routing them through `mop` is the change that makes the
adoption *live* rather than latent. Neither file is in another lane's fence
this wave. I will take it **only** if the byte delta is zero and the gate is
green, and I will say so explicitly in the rung either way.
