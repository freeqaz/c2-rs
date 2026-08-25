# PERMUTER_POPULATION — the failure a permuter faces is not the failure the port makes

**The one-sentence answer.** They are opposite populations, measured with one
lens on one day: of the port's **7,912** substituted words **99.87 %** differ in
their **opcode** and **0** of its 1,968 bodies are a pure reordering, while in
the decomp near-miss band a permuter is actually run on — **405** bodies at
`decomp.db` ≥ 99 % — **2.14 %** of substituted words differ in opcode,
**52.50 %** differ in a **register**, **7.90 %** of bodies are a **pure
reordering**, and the port's one mechanism (c2 inlined a callee where a call was
emitted) appears in **0 of 405**. **So `crates/c2-core/src/splice.rs`'s inline
cost model is the wrong first knob for a permuter, and the decision-surface
clause's own list — allocation order, scheduling tie-breaks — is the right one.**

Lane `w-permeasure`, 2026-08-25. Boards **#3534**–**#3538**;
[`rungs/2026-08-25-w-permeasure.md`](rungs/2026-08-25-w-permeasure.md);
prereg [`rungs/_2026-08-25-w-permeasure-prereg.md`](rungs/_2026-08-25-w-permeasure-prereg.md).

> **What this page is NOT.** It reaches no numerator, appears in no
> accept/refuse path, and grades no emit. It does not license a permuter; it
> says which one would be searching a non-empty space. The judge is still real
> `c2` under wibo plus a byte-exact obj compare, and nothing here touches it.
>
> **And it is not a paired study.** The two populations are different
> functions — the port's are `c2-rs`'s own emissions over the 878-TU workload,
> the decomp's are hand-written sources over 979 `objdiff` units. Both are
> dc3, and that is the whole of the relationship. This compares two failure
> **shapes**, and every claim below is a claim about a shape.

---

## 1. Why the question existed

`docs/GOAL_DECISION_2026-08-21.md` § AMENDED names a consumer of the port:

> *"build a better permuter to 'brute force' fixing code that is close, but
> wrong because of opaque compiler internal state"*

`docs/DIFF_STRUCTURE.md` had measured a wrong-body population and found it was
**one mechanism**. Board **#3369** recorded the conflation that made that
measurement look like an answer:

> the owner's permuter use case is *matching pretext for hand-written decomp
> source*, a **different population** from the port's own refused bodies, and
> nothing in this tree has measured the two against each other.

`DIFF_STRUCTURE`'s reading — *"a search over allocation or scheduling would
point at nothing"* — is **true of the population it measured** and was about to
be spent on a population it never saw.

---

## 2. Both sides, re-measured on one tree with one lens

### 2.1 The lens

`crates/c2-harness/src/gap/fndiff.rs`: LCS over 4-byte big-endian words, adjacent
insert/delete runs paired into substitutions, per-substitution decoded-field
classification under the re-encode-or-refuse rule, a `same_multiset` reordering
bit, relocation-site awareness. It ships, it is unconditional, and **it was not
rebuilt** (#3369).

It cannot be pointed at two arbitrary objs without a `crates/` edit, which this
lane was not permitted to make, so the decomp side runs through
`work/w-permeasure/permeasure.py` — a re-expression **graded before use** on
three control arms (§5).

### 2.2 The port side — rescanned, not quoted

`c2-rs` `a8593651b`, 878-TU dc3 workload, `/GR /O1 /Oi /EHsc`, `DIFF STRUCTURE`
block of a full `c2rs gap` scan:

| | |
|---|---:|
| signatures | **1,968** in 8 clusters |
| accounting breaks (known answer 0) | 0 |
| LCS-capped rows | **0** |
| **pure reorderings** | **0** |
| first word already wrong | **1,826 / 1,968 = 92.78 %** |
| substituted words | **7,912**, 0 undecoded |
| … `opcode` | **7,902 = 99.87 %** |
| … `mixed:reg+disp` | 7 = 0.09 % |
| … `reg` | 3 = 0.04 % |
| substitutions / deletions under a relocation | 157 / 3,057 |

**The rescan the brief asked for, and its result: `DIFF_STRUCTURE.md`'s SHAPE is
confirmed and its COUNTS have moved.** The page is at tree `0c8a185` — 3,195
bodies, 5,189 words, opcode 99.7 %, reorderings 0, first-word 94.3 %. At
`a8593651b` it is 1,968 / 7,912 / **99.87 %** / **0** / 92.78 %. The population
nearly halved, the substituted-word count rose by half again, and **the opcode
share went up**. Nothing about the conclusion moved. The page is not edited —
it is a dated record and this is the rescan beside it (#3369's own rule).

### 2.3 The decomp side — the population, defined by bytes

`../dc3-decomp` @ `15a64d92f1975868e55a1c670d312a8e464074c3`, **0 dirty files**,
`objdiff.json` 2,224 units of which **979** carry both a target and a base obj.

| | |
|---|---:|
| `P` — symbols naming a `.text` COMDAT body in **both** objs | **29,163** |
| `P-identical` — bytes and relocation targets agree | **20,475** |
| `P-reloc-differs` — bytes agree, a callee **name** does not | **7,158** |
| **`N` — the reachable near-miss population (bytes differ)** | **1,530** |
| `N90` — `N` ∧ `decomp.db current_percent` ≥ 90 | **1,098** |
| `N99` — `N` ∧ ≥ 99 | **405** |
| COFF reads refused (fail-closed) | 0 |

`N` is **1,530 of 29,163 pairable bodies = 5.25 %**. The decomp is mostly done;
the near-misses are the residue, and that residue *is* the permuter's input.

> **`P-reloc-differs` (7,158) is broken out and NOT folded into `N`, and that is
> the port side's own convention rather than a convenience.** `DIFF STRUCTURE`
> profiles `fnbyte-differs` (1,968); the byte-identical / wrong-target bodies are
> `fnbyte-reloc-differs` (530), credited nowhere and not part of the population
> the cluster table describes (#884, #986). Measured here, that class is
> **template-instantiation naming under COMDAT folding** — ours calls
> `??H?$_Bit_iter@_NPB_N@…` where the target names
> `??H?$_Bit_iter@U_Bit_reference@…`, bodies identical word for word;
> `decomp.db` carries a `merged_symbols` table for exactly this. It is not a
> compiler-state problem and it is not a permuter case. **It is also 4.7× larger
> than `N`**, so anyone quoting "the decomp near-miss population" owes which of
> the two they mean.

---

## 3. THE COMPARISON

Same lens, same day, same box. Word-class rows are over **non-capped rows only**
— a capped row aligns positionally and manufactures substitutions between
*equal* words; the port side has **0** capped rows and folding 31 of them into
the decomp side would compare a fallback against a real LCS.

| | **port** 1,968 | **N** 1,530 | **N90** 1,098 | **N99** 405 |
|---|---:|---:|---:|---:|
| **pure reorderings** | **0.00 %** | 3.53 % | 4.64 % | **7.90 %** |
| first word already wrong | **92.78 %** | 1.24 % | 0.36 % | **0.00 %** |
| substituted words | 7,912 | 34,504 | 16,454 | 2,520 |
| … **`opcode`** | **99.87 %** | 23.63 % | 12.11 % | **2.14 %** |
| … **`reg`** | 0.04 % | 46.18 % | 54.45 % | **52.50 %** |
| … `disp` | 0 | 7.68 % | 9.82 % | **20.20 %** |
| … `imm` | 0 | 5.78 % | 7.19 % | **16.23 %** |
| … `branch-target` | 0 | 9.48 % | 11.66 % | 7.34 % |
| … `mixed:reg+disp` | 0.09 % | 5.02 % | 3.15 % | 0.52 % |
| … `undecoded` | **0** | 0.29 % | — | — |
| **opcode-implicated words** | **99.87 %** | 23.63 % | 12.11 % | **2.14 %** |
| **operand-level words** | **0.13 %** | 76.37 % | 87.89 % | **97.86 %** |

**Per body, jointly — never by multiplying the marginals above:**

| | **N** 1,499 | **N90** 1,074 | **N99** 397 |
|---|---:|---:|---:|
| no substitutions at all (pure ins/del) | 4.7 % | 6.0 % | 6.8 % |
| substitutions, **none** an opcode difference | 34.8 % | 46.6 % | **82.6 %** |
| at least one opcode difference | 60.4 % | 47.5 % | 10.6 % |
| **PERMUTER-REACHABLE** — no opcode-class substitution **and** the callee set agrees | **39.6 %** | **52.5 %** | **89.4 %** |
| … of those, a pure reordering | 8.6 % | 8.7 % | 8.7 % |

### 3.1 The inlining signature is ZERO, and the denominator is in the sentence

`DIFF_STRUCTURE` §3's own predicate — *transfers control anywhere other than by
its own terminal `blr`*, primary 16 or 18, or primary 19 with XO 16/528 — applied
to both sides of every near-miss body, **jointly on the row**:

| | **N** 1,530 | **N90** 1,098 | **N99** 405 |
|---|---:|---:|---:|
| our body contains a call or branch | 100.0 % | 100.0 % | 100.0 % |
| the target body contains one | 100.0 % | 100.0 % | 100.0 % |
| **the two DISAGREE** | **0.0 %** | **0.0 %** | **0.0 %** |
| linked-call presence disagrees | 0.8 % (12) | 0.0 % | **0.0 %** |

**0 of 1,530.** The mechanism that *is* the port's population — 78.9 % of c2's
counterparts making no call at all at `0c8a185` — does not occur once here.

**And there is a reason, which is why the null is not a surprise to be
explained away.** Both sides of a decomp near-miss came out of the **real**
compiler. c2 makes the inlining decision from whatever source it was handed, so
a near-miss is a difference in the *source*, not in the *compiler model*. The
port's population is the opposite case: the same source, and a model that has
not yet learned the decision. **A permuter is defined over the first case.**

### 3.2 Two worked examples, both at ≥ 99 %

**A pure reordering** — `default/system/gesture/BaseSkeleton`,
`?BoneLength@BaseSkeleton@@UBAMW4SkeletonBone@@W4SkeletonCoordSys@@@Z`, 99.875 %.
16 words each, multisets equal, two words swapped:

```
        ours                              target
w5      c0010050  lfs f0,80(r1)      |    c0010054  lfs f0,84(r1)
w8      c0010054  lfs f0,84(r1)      |    c0010050  lfs f0,80(r1)
```

Two loads, scheduled the other way round. **This is the class
`DIFF_STRUCTURE.md` measured at exactly 0 in 3,195 bodies**, and it is 32 of
405 here.

**A single operand choice** — `default/system/char/Character`,
`?Normalize@@YAXABVVector3@@AAV1@@Z`, 99.565 %. One substituted word in the
whole body:

```
w18     ec0002f2                     |    ec0b0032
        fmuls, fields FRA/FRC differ  —  a commutative operand order
```

Neither is reachable by a search over an inline cost model, and both are the
first thing a classic permuter tries.

---

## 4. THE CONSEQUENCE — which permuter is worth building

**Do not build the inline-decision permuter first.**
`crates/c2-core/src/splice.rs`'s S7 clause and `INLINE_PREDICATE.md`'s cost model
(graded 0.9716 with a 2.84 % NOT-MODELLED residual) are the right knob for the
**port's** population and measurably the wrong one for the permuter's: in `N99`
an opcode difference appears in **10.6 %** of bodies and **2.14 %** of
substituted words, and the inlining signature is **0 of 405**.

**Build the operand-level search.** Ranked by `N99`'s substituted words, with
the denominator (2,520 words over 397 non-capped bodies of 405):

| rank | decision point | share of `N99` words |
|---|---|---:|
| 1 | **register assignment** | **52.50 %** |
| 2 | **stack-slot displacement** | **20.20 %** |
| 3 | **immediate / literal choice** | **16.23 %** |
| 4 | branch target / block layout | 7.34 % |
| — | *instruction schedule* (a body-level class, not a word class) | **7.90 % of bodies are pure reorderings** |
| 5 | everything opcode-implicated | 2.14 % |

**This list is already written down in this repo, and the measurement says it
was right.** `docs/rungs/README.md` § "Lane kinds" 2, the decision-surface
clause adopted 2026-08-22 from the owner's re-ranking, requires a general layer
to expose *"allocation order, scheduling tie-breaks, label counters"* as named,
enumerable parameters whose default reproduces c2 byte-exactly. Items 1, 2 and
the schedule row are that clause's own three, in the order the population puts
them. **The clause needs no amendment; it needs the layers.**

**What this does not say.** It does not say a permuter will succeed, does not
price one, and does not say the port is ready to host one — `89.4 %` of `N99` is
*reachable in principle by an operand-level search*, which is a statement about
the shape of the target, not about any search's hit rate. And `N99` is **405
bodies of 29,163 pairable**; the bands are reported whole precisely so nobody
quotes 89.4 % against the wrong denominator.

**One honest selection effect, stated because it cuts toward the conclusion.**
`N99` is selected by `decomp.db`'s own score for *near-ness*, so it is
enriched for small differences by construction. That is not a defect of the
measurement — it is the definition of the population a permuter runs on — but
it does mean `N99`'s 2.14 % opcode share must never be quoted as "how MSVC
differs from a naive decomp". `N` (1,530, unselected except by "the bytes
differ") is the row for that question, and there the opcode share is **23.63 %**
and the reachable fraction **39.6 %**.

---

## 5. THE CONTROLS, and the three artifacts they caught

`work/w-permeasure/permeasure.py` refuses to print a single decomp number until
arms 1 and 2 pass. Board **#2064**: *a rescoring harness that cannot reproduce
the published scores is measuring something else.*

| arm | what it grades | result |
|---|---|---|
| **1** | re-derive `fndiff.rs`'s own `first`/`equal`/`sub`/`ins`/`del`/`same_multiset`/`capped`/`classes`/`csig`/`sig`/`prefix`/`suffix`/`accounting` from its own `port_hex`/`ref_hex` | **1,968 / 1,968 = 100.0000 %**, 0 excluded |
| **2** | replay `fndiff.rs`'s own `mod tests` assertions through the re-expression | **38 / 38** |
| **3** | `decomp.db`'s independent 100.0 verdict against this lens's bytes | 6,850 contradictions → **15** |

Both arms were **watched failing on deliberately broken input** before being
trusted (CLAUDE.md): `pair_runs` disabled → 7.06 % / 33-of-38; #977's classify
ordering reversed → 7.06 %; `bits()` shifted → 7.06 %; edit order scrambled →
87.70 %; `disp` folded into `imm` → 37/38.

> **ARM 1 WAS GREEN THROUGH ALL THREE ARTIFACTS, AND THAT IS THE LESSON.** It
> grades the **lens**; all three defects were in the **input**. A control that
> cannot see the inputs is a statement about the population it can reach —
> arm 1's 7,912 words are `opcode` 7,902, `mixed:reg+disp` 7, `reg` 3, and it
> never once produces `imm`, a bare `disp`, `branch-target`, `spr` or
> `cr-field`: **precisely the classes this page's answer is made of.** Arm 2
> exists for that gap and arm 3 for the inputs.

### 5.1 The three artifacts, each caught by a shape too systematic to be real

1. **Whole-section bodies.** The first run took a COMDAT `.text` section's whole
   raw data as the body, the way `c2_obj::text_comdat_entries` does — sound for
   objs where c2 under `/Gy` puts one function per COMDAT at offset 0, wrong
   here. `src/App.obj` section 72 is 128 bytes holding the section definition at
   0, `??0FilePath@@QAA@PBD@Z` at **Value = 8**, `__unwind$275902` at 88 and five
   interior `$M` line labels. The published headline was **84.8 %
   `port-longer|sub+ins|branch-target` over 8,313 bodies** — an artifact of
   *section offset*. **Caught by arm 3**: a function `decomp.db` scores 100.0
   read as 32 words against 20.
   **Fix:** body = `[symbol Value, next boundary)`; `IMAGE_SYM_CLASS_LABEL` is
   interior and must never truncate a body.

2. **Link state read as compiler behaviour — board #984 in the mirror.** Under
   `/Gy` a call's placeholder displacement is `-(offset of the branch word)`, so
   #984's finding is that byte equality **credits** a relocated word it has not
   checked. Run the same fact the other way and it **penalises**: the same call
   compiled at a different section offset is different bytes.
   `??0FilePath`'s four `bl`s each differed by exactly 8.
   **Fix:** `normalize()` zeroes the bits the linker overwrites (REL24 `LI`,
   REFHI/REFLO low 16, ADDR32) and compares relocation **target names**
   separately, never summed into the byte verdict.

3. **Alignment padding read as inserted code.** With extents fixed, one cluster
   held **531 bodies, 25.8 % of N**, `port-longer|ins-only|-` — and **530 of
   them inserted exactly two words**. Sampling showed 399 of 400 inserting
   `00000000 00000000` at the **end**: section padding, because a function that
   is last in its COMDAT runs to the padded section end. `0x00000000` is not a
   legal PPC instruction, so a real body cannot end in one.
   **Fix:** trailing zero words trimmed on both sides. `N` 2,062 → **1,530**,
   and *first word already wrong* fell 26.62 % → **1.24 %**.

**The common shape of all three: a cluster too clean to be a compiler
decision.** 84.8 % in one signature; exactly 2 extra words in 530 of 531; a
100 %-scored function reading as 12 words short. **None was caught by a green
control and all three were caught by asking what a suspiciously tidy number was
made of.**

---

## 6. Reproduce

```sh
# port side — the shipped instrument, unmodified
cargo build --release -p c2-harness
./target/release/c2rs gap --list work/dc3-workload/files.txt \
    --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --jobs 8 \
    --fnbyte-diff-jsonl work/w-permeasure/port_fndiff.jsonl

# controls alone (exit 1 if either arm fails)
python3 work/w-permeasure/permeasure.py control work/w-permeasure/port_fndiff.jsonl

# both sides; refuses to print anything if the controls fail
python3 work/w-permeasure/permeasure.py measure ../dc3-decomp \
    work/w-permeasure/port_fndiff.jsonl --json work/w-permeasure/decomp_rows.jsonl
```

Corpus stamp: `../dc3-decomp` `15a64d92f1975868e55a1c670d312a8e464074c3`, 0
dirty files, obj manifest digest
`45a06795a83cf6e770bc7a5a9d1769f8c1d7818ca84b37b077baa7b3c0032329`. **#3500's
rule applies and is not satisfied by the commit alone** — that tree has been
observed to move its corpus without moving its commit, and this lane's numbers
are valid against that manifest and no other.
