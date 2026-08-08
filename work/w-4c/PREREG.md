# w-4c — PREREG

Written and committed **before the first capture**. Lane `w-4c`, branch
`wt-w-4c`, off master **`2b1c89da`**.

## 0. What the rung is

Board **#1318** (`w-bd`) measured `0x4C` — the CALL-END, `0xBD`'s own closing
bracket — as **payload-free at 26,701 of 26,701 sites, and refused to ship it**,
because those 26,701 sites are **zero-argument calls only**. The `4C` that
closes a call *with* arguments is 2.46 M of the 3.5 M `BD` tokens and appears in
that grid **zero times**. A frozen holdout containing none of the class the
width would be applied to is worth nothing.

So this lane's population is the **argument-bearing** one. Nine of the
seventeen frontier ladders terminate at `expr-chain-noform-0x4C`, and three
frontier TUs (`vswprnc.cpp`, `undname.cpp`, `osfinfo.cpp`) exit there and
nowhere else.

**I am not expected to convert a TU and I do not expect to convert one.** All
three of those TUs also need `cflow-if-n`, which the emitter cannot represent
(board **#1353**). This rung removes an instrument floor.

## 1. The width I expect

    CALL-END := 4C            (payload-free)

**P-WIDTH:** `chain_skip_form(0x4C) == Some(SkipForm::Bare)`, and the same
answer holds on the argument-bearing population as on `w-bd`'s zero-argument
one.

I register the reason I expect this and it is **not** plausibility: **four**
readers in this tree already consume `4C` as exactly one byte, and two of them
are on the argument-bearing path.

* `control_flow.rs`'s `operand()` `0x4C` arm — `s.p += 1` (plus the EH
  bookkeeping that lives on `4C` deliberately).
* `mcall::eat_call_args_region` — `Some(&0x4C) => { *p += 1; return true }`.
  **This is the accepting parser's argument-region loop**, i.e. the
  argument-bearing path itself.
* `codec.rs::try_ex_token` — `0x4C` is `ExToken::Lo`, width 1 (with `4C 4B`
  taken first as `VoidCallEnd`, width 2).
* `codec.rs`'s **`ExToken::IntCallEnd` = `55 86 41 74 4C`** — the edit model
  *emits* an argument-bearing call end as `55 <INT TYPE> 4C` with nothing after
  the `4C`, and the generated sweep grades the resulting objs against real
  `c2.dll`.

This is `w-bd`'s finding in a second opcode: `chain_skip_form`'s `None` means
*"evidence this enum cannot express"* as often as it means *"no evidence"*.
Here the enum **can** express it (`Bare` exists); the row is simply absent.
`4B` is in the table and `4C` is not.

**The rivals the workload check must exclude**, the same four `w-bd` used:

* **P** — payload-free (the claim).
* **B1** — `4C <one raw byte>`.
* **T** — `4C <TYPE>`.
* **K** — `4C <token>`.

**P-RIVALS:** B1, T and K are all refuted on the argument-bearing population,
and I expect the margins to be *narrower* than on the zero-argument one,
because the successor distribution after an argument-bearing `4C` is richer.

## 2. The anchor, and the one way it can be biased

`w-bd`'s anchor was *"the byte a `BD` token ends on, when that byte is `4C`"* —
exact, and exactly the zero-argument case. The argument-bearing anchor cannot
be a constant offset, because the argument region is a token stream of variable
length.

**Anchor A — the forward walk.** From an anchored `BD` (`w-bd`'s two anchors,
`26 <token> BD` and `99 <TYPE> <varint> BD`), step the argument region token by
token with a stepper whose vocabulary is taken **from the tree**, tracking call
depth: a nested `BD` pushes, a `4C` pops, and the `4C` that returns depth to 0
closes this call. An argument-bearing site is one where the walk crossed at
least one `55 <TYPE>` argument terminator before the closing `4C`.

**The bias I have to declare up front, because it is real**: a walk that hits an
opcode of unknown width must abandon the site rather than guess, so Anchor A can
only see calls whose arguments are built from opcodes this tree already knows.
**P-POP:** I expect to anchor **at least 50 %** of the argument-bearing
population, and I will publish the walked fraction and the *reason* each
abandoned site was abandoned, as a histogram of the opcode that stopped it. If
the walked fraction is small, that is itself close to a decline and I will say
so rather than quoting the 0 that comes out of it.

**Anchor B — the last-argument bracket, walk-free.** Independent of A and with
no stepper at all: `eat_call_args_region`'s grammar makes the final argument end
`55 <TYPE>` immediately before the closing `4C`. So a `4C` preceded by a
well-formed `55 <TYPE>` whose own start is reachable is an argument-closing `4C`
candidate located **without** any forward walk. It is the weaker anchor (a `55
<TYPE>` can precede a `4C` by coincidence) and it is here precisely because its
failure mode is *different* from A's: A is biased toward simple arguments, B is
not biased that way at all.

**P-ANCHOR:** A and B agree on the sites they both see. If they disagree the
lane declines and publishes the disagreement.

## 3. Confirmation 1 — a capture, graded

Board **#820** (`w-divsplit`) is the standard and it is **two** confirmations,
not one. Confirmation 1 is a **captured token stream, graded against real
`c2.dll`**, not a reading.

`work/w-4c/probe/` will carry a `.cpp` whose calls **take arguments** — one
argument, two arguments, a nested call as an argument, and a call whose result
is an argument — captured with `c2rs capture` at this master, read out with a
committed reader, and graded with `c2rs diff`.

**P-GRADE:** `ReferenceReplay=ByteExact`, and at least one probe function
`Port=Match`. A `Port=Match` on an argument-bearing call is the load-bearing
row: it means the accepting parser walked the argument region, read `4C` as one
byte, and the obj that came out is byte-exact against real `c2.dll` under wibo.

## 4. How many ladders, and by how much

Read off `w-bd`'s committed `lad_tip/ladder.json` + `lad_tip_kg/ladder.json`,
**9** of the 17 name `expr-chain-noform-0x4C` in their exit:

    sole terminal (6)   osfinfo · undname · vswprnc · negate_test · vsnprnc · mmio
    one of several (3)  wordwrap (+0x1C) · EncryptXTEA (+0x00) · keygen_xbox (+0x13)

**P-COUNT:** my own base re-climb at `2b1c89da` reproduces **9**. Master has
advanced since `w-bd`'s tip (`177d556a` → `2b1c89da`, the `w-instr` merge), so
this is a prediction and not a transcription.

**P-EXTEND**, registered as point estimates from `ladder.py`'s `rungs_net`:

| TU | base | I predict | I predict the new exit is |
|---|---:|---:|---|
| osfinfo | 13 | 15 | another `noform` |
| undname | 7 | 9 | another `noform` |
| vswprnc | 6 | 8 | another `noform` |
| negate_test | 12 | 14 | another `noform` |
| vsnprnc | 7 | 9 | another `noform` |
| mmio | 7 | 8 | another `noform` |
| wordwrap | 19 | 19 | `noform-0x1C`, unmoved |
| EncryptXTEA | 16 | 16 | `noform-0x00`, unmoved |
| keygen_xbox | 18 | 18 | `noform-0x13`, unmoved |
| **total added** | | **+11** | |
| **ladders that extend** | | **6 of 9** | |

**P-NEXT:** the single most likely new floor is **`0x5C`**, the EH-state
trailer. `control_flow.rs`'s own test vectors carry `… 4C 5C 86 41 74 01 4B`,
`operand()` has `5C`/`5D`/`5E` arms and `chain_skip_form` has none of them — the
identical shape of defect one opcode further along. I register that I expect
this floor to be standing on a third floor, because `w-bd` found exactly that
and said so.

## 5. The direction I expect to be wrong in

**Optimistic.** Board **#770** records **ten** consecutive optimistic misses,
`w-bd` registered itself as the tenth and *was*. I register no reason to be the
eleventh exception. The two concrete ways §4 is most likely too generous:

1. **Fewer than 6 extend.** `rungs_net` is a per-TU union over the TU's blocked
   functions; a TU whose other functions stop at a different noform gains 0 net
   even when this one is pinned. `w-bd` predicted 9 of 9 and got 7 of 9.
2. **The extensions are shorter than +2.** If P-NEXT is right and `5C` follows
   the `4C` closely, several rows gain exactly +1.

I also register the *opposite* risk, because it is the one that matters more:
**the argument-bearing population may not settle the width.** If the walked
fraction is small, or if B1/T/K are not clearly refuted on it, this lane
**declines with the counts** the way `w-bd` did. A second, better-founded
decline is a real result and is a better outcome than a width table entry
resting on 3 % of its own population.

## 6. What I expect to move, and what I expect NOT to

**P-QUIET:** every `gap-metric` line byte-identical base vs tip, `fn_blockers`
and `emit_blockers` move **zero** keys, `peerkeys.py` reports no family gained
or lost, TU match unchanged, mismatch 0 at both ends.

The reasoning is `w-bd`'s and it is structural, not hopeful: the chain sink is
`C2RS_SINK_CHAIN`-gated, defaults **OFF**, pushes no `IlOp`, and poisons any
walk that used it (`expr-chain-sink-poison`). A change confined to
`chain_skip_form` therefore cannot move the census, the blocking histogram or
the completeness walk. **The lane brief predicts the `gap-metric` block WILL
move. I predict it will not.** That is a registered disagreement and one of us
is wrong. It would be live if the edit landed in `control_flow.rs`, and
`control_flow.rs` is **not** this lane's file and needs no change (board
**#1320**: two width tables, one vocabulary of hex keys).

**P-FILE:** the whole shipped diff is `crates/c2-il/src/func/body/expr.rs`.

**P-TESTS:** +2 to +4 `#[test]` bodies. `w-bd` predicted +4..+8 and got +3.

## 7. The decline condition, stated so it can fire

This lane **declines** — ships no width — if any of:

* Anchor A walks **< 25 %** of the argument-bearing population and Anchor B does
  not independently cover the shortfall;
* A and B disagree about any site they both see;
* B1, T or K survives on the argument-bearing population at a rate that does not
  separate it from P by orders of magnitude;
* the probe does not grade `ReferenceReplay=ByteExact`.

`0x14` is already carried in the *other* width table as a deliberate unresolved
entry with a test pinning it. **A second honest entry beats a guess**, and
board **#1318** is the precedent for declining while holding the evidence.
