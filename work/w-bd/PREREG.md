# w-bd — PREREG

Written and committed **before the first capture**. Lane `w-bd`, branch
`wt-w-bd`, off master `04727f37`.

## 0. What the rung is

`w-front3` (board **#1289**) closed with: *"Pinning opcode `0xBD`'s payload
width from a capture extends 7 of the 17 ladders at once. It converts
nothing."* Seven-or-so of the seventeen frontier ladders terminate at
`expr-chain-noform-0xBD` — the **instrument** running out, not the TU. This lane
removes that floor or declines with evidence.

**I am not expected to convert a TU and I do not expect to convert one.**

## 1. The width I expect

    CALL := BD  <TYPE ret>  <flags:1 raw byte>  <varint fn-type-id>

i.e. a form `<TYPE> <1 raw byte> <varint>`, total 8–13 bytes, every field
self-delimiting. `SkipForm` has no variant of that shape today
(`TypeVarint` is `<TYPE> <varint>`), so I expect to add one —
`TypeByteVarint` — rather than to reuse an existing one.

**P-WIDTH:** `chain_skip_form(0xBD) == Some(TypeByteVarint)`, reading the flags
byte as a *raw byte* and not as a varint.

## 2. Where I expect the file to be

The brief names `crates/c2-il/src/func/body/shapes/control_flow.rs`. I expect
that to be **the wrong file**: `control_flow.rs`'s `operand()` table already has
a `0xBD` arm, and the `noform-0xNN` key is minted by `chain_skip_form` /
`chain_step_with` in `crates/c2-il/src/func/body/expr.rs`. **P-FILE:** the edit
lands in `expr.rs`, and `control_flow.rs` needs no change.

If P-FILE holds, the brief's shared-semantics hazard is also mis-aimed: the
chain sink is `C2RS_SINK_CHAIN`-gated, defaults OFF, and pushes no `IlOp`, so
widening it cannot move the census, the blocking histogram or the completeness
walk. **P-QUIET:** all 139 `gap-metric` lines are byte-identical base vs tip,
`fn_blockers` / `emit_blockers` move **zero** keys, and `peerkeys.py` reports no
key family gained or lost. The brief predicts the `gap-metric` block **WILL**
move; I predict it will not. This is a registered disagreement and one of us is
wrong.

## 3. How many ladders, and by how much

Read off `work/w-front3/lad/ladder.json` + `lad-kg/ladder.json` before touching
anything, **9** of the 17 ladders name `expr-chain-noform-0xBD` in their exit:

    sole terminal (5)   osfinfo · undname · vswprnc · vsnprnc · negate_test
    one of several (4)  wordwrap (+0x1C) · mmio (+0x4C)
                        EncryptXTEA (+0x00,+0x4C) · keygen_xbox (+0x13, hatch withheld)

**So the headline "7" is itself unverified** and is a third thing this rung
measures. **P-COUNT:** the true base figure is **9**, not 7.

**P-EXTEND:** of those 9, **all 9** gain at least one rung; the 5 sole-terminal
rows gain more than the 4 multi-terminal ones, whose chains stop at the *other*
noform almost immediately. Registered point estimates (`rungs_net`, from
`ladder.py`, not predicted from prose):

| TU | base | I predict | I predict the new exit is |
|---|---:|---:|---|
| undname | 5 | 8 | another `noform` |
| vswprnc | 5 | 8 | another `noform` |
| vsnprnc | 6 | 9 | another `noform` |
| negate_test | 9 | 12 | another `noform` |
| osfinfo | 12 | 15 | another `noform` |
| mmio | 7 | 8 | `noform-0x4C` |
| wordwrap | 17 | 18 | `noform-0x1C` |
| EncryptXTEA | 16 | 17 | `noform-0x00` or `-0x4C` |
| keygen_xbox | 15 | 16 | `noform-0x13` |

**Total rungs added: +19.** Zero TUs converted; TU match 10 → 10.

## 4. The direction I expect to be wrong in

**Optimistic.** Board **#770** records that estimates on this project have
missed in the optimistic direction **nine consecutive times**, and this lane
registers no reason to be the tenth exception. Concretely, the two ways §3 is
most likely too generous:

1. **Fewer than 9 extend.** A ladder whose functions each stop at a *different*
   noform gains 0 net rungs when only one of those noforms is pinned, because
   `rungs_net` is a per-TU number over a per-function set. The 4 multi-terminal
   rows are the exposed ones and I expect at least one of them to move by 0.
2. **The extensions are shorter than +3.** `BD` is a call token; the byte after
   the argument region is `4C`, which is *already* in the table as `Bare` —
   but `mmio` and `EncryptXTEA` exit at `noform-0x4C` today, which means the
   sink's `4C` is reached at a position the table refuses, so pinning `BD` may
   walk straight into `0x4C` after one rung.

If either fires I will say so and score it a MISS in the rung doc, the way
`w-front3` scored its own P-DIR.

## 5. The evidentiary standard I have to meet

`w-divsplit`/board **#820** is the precedent and it is **two** independent
confirmations, not one:

1. a **captured token stream** showing the shape, graded against real `c2.dll`;
2. a **workload-wide** check over every site, showing the rival readings are
   excluded by the bytes.

**P-STANDARD:** I expect to meet both. If `0xBD` turns out genuinely ambiguous
from captures I will decline and publish the ambiguity, the way `0x14` is
carried as a deliberate unresolved case with a test pinning it.

The rival readings the workload check must exclude:

* **R1** `<TYPE> <varint>` (`TypeVarint`, no flags byte) — the reading that
  would be right if the flags byte did not exist.
* **R2** `<TYPE>` alone (`Type`).
* **R3** `<TYPE> <varint flags> <varint id>` — flags read as a varint rather
  than a raw byte. **I expect this one to be INDISTINGUISHABLE** on this
  corpus, because `IL_CALL_GRAMMAR.md` §2.2 already records every observed
  flags value (`00`, `04`, `40`) as `< 0x80`, where the two readings agree. I
  register that up front: R3 is not excluded by the corpus and the rung will
  say so rather than claim it is. The tie is broken by *consistency with the
  accepting parser* (`mcall::eat_call_and_args` reads a raw byte), because a
  sink that read a field differently from the acceptor would report a successor
  the acceptor can never reach — which is the whole failure mode this
  instrument exists to avoid.

## 6. Controls, and the positive question

**Would this control go red if my width were wrong in the most likely way?**
The most likely way to be wrong is *off by one byte* — omitting the flags byte
(R1). A stream read one byte short lands mid-varint on `80`, which is not an
operand opcode, so:

* **C1 — the corpus walk.** For every `BD` site in the dc3 corpus I decode
  under P-WIDTH, R1 and R2 and record the byte each reading lands on and
  whether it opens a legal token. C1 goes red iff P-WIDTH lands on an illegal
  byte anywhere. It goes red **on R1 by construction** — which is the check
  that it can go red at all, and I will publish R1's own red count as the
  calibration rather than assert C1 is meaningful.
* **C2 — a fresh capture.** An `e19.cpp`-shaped probe (three externals
  differing only in calling convention, identical return type) recompiled at
  this master, its IL captured, the three `BD` tokens read out, and the obj
  graded byte-exact against real `c2.dll` under wibo. Red if the flags byte is
  not the only byte that moves.
* **C3 — the null step.** `chain_skip_form` for every byte NOT `0xBD` is
  unchanged, asserted byte for byte over `0..=255`. Red if the edit widened
  anything else.
* **C4 — the sink stays OFF.** The existing `chain sink must default OFF` test
  extended to assert `0xBD` claims nothing while OFF. Red if the new arm is
  reachable without `C2RS_SINK_CHAIN`.
* **C5 — the poison holds.** A walk that used the `0xBD` step still refuses.
  Red if decoding became accepting.

## 7. Tests

**P-TESTS:** `+4` to `+8` `#[test]` bodies in `expr.rs`; the workspace target
count unchanged at 36; `the_unpinned_opcodes_refuse_rather_than_guess_a_width`
loses `0xBD` from its list, which is the assertion that has to be *deleted*
rather than a blank line, per its own header.

## 8. The secondary finding, and its ordering

`w-front3` §4.4: lifting `call-arg-outer-formal` panics on `keygen_xbox.cpp`.
The brief cites `crates/c2-core/src/codegen/calls.rs:71`; `lad/ladder.json`'s
own `SCANFAIL` trace names `crates/c2-il/src/func/body/s…`, i.e. **c2-il**, so
the cited location is itself unverified. **I do not start this until `0xBD` is
finished or declined**, and if I do not reach it I will say so rather than
half-land it.

## 9. What would make this rung a decline

Any of: the corpus walk shows P-WIDTH landing on an illegal byte at a
non-trivial number of sites; the fresh capture disagrees with the tree's three
existing readers; or the two confirmations reduce to one. A decline on
measurement is a good outcome here and is the more likely of the two.
