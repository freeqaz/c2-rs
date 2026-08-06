# w-emitp2 — PREREGISTRATION

    Lane:    w-emitp2, 2026-08-08
    Branch:  wt-w-emitp2, off master `039b718`
    Scope:   MEASUREMENT.  Nothing ships under `crates/`.
    Written: BEFORE any probe exists.  No script in `work/w-emitp2/` has been
             run, and no corpus-wide number has been computed on this branch.

---

## 0. The question, and the brief's premise

The coordinator's brief asks: **now that `w-tag02` made the initializer
reference graph readable — `InInitResidue::SymbolAddress` refused 913,136
elements before and refuses 0 after, with 1,466,832 symbol-address elements over
453,022 records readable — what does the data-initializer emit-predicate channel's
precision become, and what per-TU exact does a no-truth-conditioning model
reach?**

**I have read the code before writing this, and I believe the premise is
false**, so I am registering that as the claim that can lose.

The number `0.96 recall at 0.27 precision` is the `INIT` row of
`rungs/_2026-08-04-w-emitp-findings.md` §2.2 (precision **0.27289**, recall
**0.95991**, F1 **0.42496**, per-TU exact **34 of 850**). It was produced by
`work/w-emitp/scan.py`, whose `.in` node source is `work/w-skip/marks.py`
`parse_records` → `work/w-mark/instream.py` `node`. That reader has read element
tag `02` since w-mark:

    instream.node, k == 0x02:   varU token ; i32c addend ; i32c width

`i32c` (`work/w-roots/glflags.py`, transcribed from `0x10c1f9e9`) is *signed
byte, or `0x80` escape then LE32*. `w-tag02`'s measured grammar is
`02 <target-token> <offset> <n>` with `<offset>` a varint — short form `00..7F`,
escape `80` + LE32 — and `<n>` = `04`.

**On the escape and on every short form in `00..7F` the two consume identical
bytes.** They can differ only on an offset byte in `0x81..0xFF`, which
`instream` would read as a negative one-byte offset and `crates/c2-il`'s
`read_offset` refuses as a desync.

So the residue `w-tag02` closed was in **`crates/c2-il`**, a reader **no
emit-predicate lane has ever used**. If that is right, the re-pricing this lane
was commissioned to do returns **zero movement**, and the useful output is the
negative plus a corrected residual.

**I could be wrong in three ways and each is registered below**: the
`0x81..0xFF` population could be non-empty (P1); `instream`'s whole-stream
`clean` gate could be truncating streams the strict grammar would carry (P3);
and the record-level acceptance the crate applies could differ from the python
channel's in a way that changes the node universe (P8).

---

## 1. The channel definitions used — THEIRS, cited, not re-invented

Every model below is `work/w-emitp/scan.py`'s by value. No definition is
changed; the **only** variable this lane introduces is which `.in` reader
supplies the `02`-node token list.

| model | definition | citation |
|---|---|---|
| `RGL` | `joint.closure(Seed, .gl reference-list edges, U, skip)` | w-refs / w-roots; `scan.py:188` |
| **`INIT`** | `joint.closure(Seed ∪ I, …)` where **`I` = every `U`-name named by any `02` node in any `.in` record** (w-mark's unfiltered reading) | `scan.py:179-189` |
| `SKIP` | `Seed ∪ marks.replay(loose)` — w-skip's instruction-level replay of `0x10b98e26` | `work/w-skip/marks.py`; `scan.py:186,190` |
| `JFP` | w-db's joint fixpoint over merged code edges (`.gl` refs) + data edges (`.in` owner→nodes), entering `W` | `scan.py:65-78, 196` |
| `JFP_ALIAS` | `JFP` with `02`-node targets resolved through the tag-0x10 ALIAS table | `scan.py:215-216`; w-emitp §6 |
| `ORACLE` *(ceiling — conditions on truth `D`)* | w-joint's data fixpoint over `D` | `scan.py:192-195` |
| `ALIAS_IN` *(ceiling)* | `ORACLE` + alias resolution at the `in` `02` site | `scan.py:210` |

Truth `E` = **the reference obj's own code-COMDAT leader symbols**
(`work/w-emit/truth`, 174,417 names over 850 TUs). Truth `D` = the obj's defined
data symbols, regenerated here with w-joint's `truth_data.py` unmodified.
Population = the **850** TUs of `work/w-db/cacheidx.tsv`, of the 871 graded;
that is the same population every recorded baseline was taken over and the
difference from 871 is stated, never elided.

**The new variable — `STRICT`**: a second `.in` reader written from
`work/w-tag02/GRAMMAR.md` and `crates/c2-il/src/func/ininit.rs`'s field layout,
in this lane's own file, importing nothing from `instream`. Element tag `02` is
`<varU token> <strict-varint offset> <n==04>`; an offset byte in `0x81..0xFF`,
or an `<n>` ≠ `04`, is a **refusal**, not a read.

---

## 2. Registered claims — point, interval, and what losing looks like

| # | claim | point | interval | loses if |
|---|---|---:|---|---|
| **P1** | tag-02 elements over the 850 TUs whose offset short form is `0x81..0xFF` — the ONLY bytes on which `instream` and the strict grammar disagree | **0** | [0, 2000] | > 0. Then `instream` desyncs there and the channel *was* truncated, the brief's premise is live, and P4 should move |
| **P2** | strict-reader tag-02 element count over the 850 TUs, as a ratio to `w-tag02`'s crate figure of 1,466,832 over 878 TUs | **0.97** | [0.85, 1.05] | outside. Then the two instruments are not reading the same stream and every conclusion below is void |
| **P3** | `.in` streams consumed to the last byte (`in_clean`) by the **strict** reader, of 850 | **850** | [800, 850] | < 800. A truncated stream is a truncated node list |
| **P4** | **THE HEADLINE.** `INIT` precision under `STRICT` minus the recorded 0.27289 — *the registered expected precision movement, as a range* | **+0.000** | **[−0.005, +0.005]** | outside. Then reading tag 02 strictly *does* move the data-initializer channel and the brief is right |
| **P4′** | `INIT` recall under `STRICT` minus the recorded 0.95991 | **+0.000** | [−0.005, +0.005] | outside |
| **P4″** | `INIT` per-TU exact under `STRICT`, of 850 (recorded 34) | **34** | [30, 60] | outside |
| **P5** | `JFP_ALIAS` per-TU exact under `STRICT`, of 850 (recorded 308) | **308** | [280, 400] | outside |
| **P5′** | TUs **gained** / **lost** by name against the recorded 308-set | **0 / 0** | [0, 25] each | either > 25 |
| **P6** | KA-A: the six incumbents (`RGL`, `INIT`, `SKIP`, `JFP`, `ORACLE`, `ALIAS_IN`) reproduce w-emitp §2.1 **to the digit** on `\|P\|`, precision, recall, F1 and per-TU exact | **6/6** | 6/6 | any digit differs — then this lane's environment is not w-emitp's and nothing here is comparable |
| **P7** | w-emitp §3.2 reproduces: emitted names with **no tag-0x0E `.gl` record at all** / the TUs carrying them / the TUs where they are the **sole** blocker of `ALIAS_IN` | **510 / 162 / 43** | ±10 % each | outside |
| **P8** | emitted names reachable through the python channel but **only** through `.in` records the strict grammar REFUSES — i.e. signal the crate-shaped reader would lose | **0** | [0, 5000] | > 5000; and any non-zero is reported with counts and a class breakdown |
| **P9** | TU match 10 before and after; `git diff 039b718 -- crates/ scripts/ Cargo.toml Cargo.lock fixtures/` is **0 bytes** | **10 / 0 bytes** | exact | either moves |
| **P10** | **the quarantine recommendation, registered before the numbers**: I predict I will recommend **DO NOT SPEND** w-emitpred's one-shot Part-1 gate | do-not-spend | — | I end up recommending spending it — which is fine, but it must then be written as a spec with the 21 outcomes frozen, per the brief |

**The declared bias.** I expect P1 = 0 and P4 = +0.000, i.e. a null result, and
a lane that expects a null has an incentive to look less hard. Three mitigations
are registered: P2 and P3 are *positive* checks with printed counts that would
catch a strict reader that silently read nothing; P6 is a six-way known-answer
control that must reproduce to the digit; and P8 exists specifically to find
signal in the direction my hypothesis does not predict.

**The second declared bias.** P7 asks a prior lane's numbers to reproduce. If
they do not, the honest reading is that something moved between `b6fa935` and
`039b718` — not that P7 was a bad prediction — and I will say which.

---

## 3. Decline clauses

1. **Nothing ships under `crates/`.** This is a measurement lane. If a model
   worth shipping falls out, it is written as a spec and not as a patch.
2. **Do not spend the one-shot Part-1 quarantine gate** (w-emitpred's 21 TUs).
   Every script must check `heldout.txt` by name before touching a TU and print
   the check. If no probe touches a TU at all, that is stated as such.
3. **Report per-TU exact BY NAME, never by count alone** (board #250, and
   `docs/STATUS.md` trap 8). Every table prints per-TU exact *and* micro-F1
   side by side, and every movement is accompanied by the gained/lost name
   lists.
4. **A null is reported as a null.** If P4 lands at +0.000 I do not go looking
   for a different channel to call the headline; I report the negative, name the
   residual with counts, and stop.
5. **No definition may be chosen by looking at `E` or `D`.** The strict reader
   is transcribed from `work/w-tag02/GRAMMAR.md` and `ininit.rs`, both of which
   predate this lane, and is frozen at this commit.
6. **Class removal is a LOWER bound** (`STATUS.md` trap 8's addendum, w-emitp
   §3): nothing here prices a class by subtracting it.
7. **`IlFunction::mangled_name` is positional and disagrees with
   `FnCensus::emit_name` on 74,955 rows (#918).** No number here is keyed on a
   positional name; the channel keys on `.gl` tokens through
   `il.gl_symbol_index`, which is the per-record binding.

---

## 4. What this lane will NOT do, registered so absence cannot read as success

1. It will not compute `|{TU : model exact} ∩ B∧C|` — `gap.rs` still has no
   per-TU `B∧C` listing, and w-emitp §8.1 declined the same extrapolation.
2. It will not decode `0x10b28ca3` (the Mark instruction), `0x10b8ac60`,
   `0x10b3389b` or `0x10b9aa26`.
3. It will not read `.sy`.
4. It will not model **order**; a right set in the wrong order is still a
   mismatch.
5. It will not touch the 21 quarantined TUs.
6. Every statement is at the workload's own `/O1 /EHsc /GR` and nowhere else.
