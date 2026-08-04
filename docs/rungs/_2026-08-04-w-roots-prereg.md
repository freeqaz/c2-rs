# w-roots — pre-registration. Frozen BEFORE the first measurement.

    Lane:      w-roots, 2026-08-04, worktree `wt-w-roots` off master `9aeded2`
    Question:  does bit `0x20` of the `.gl` flag word at `sym+0x4c` reproduce
               the ROOT set of c2's emit fixpoint?
    Status:    PREREG. Nothing below has been measured. Scored in the findings
               file, hit or miss, with the misses reported first.

---

## 0. Why this lane exists, in one paragraph

w-emit measured `PHASE7_PLAN.md` §2's **Propagation** clause on 850 real TUs and
failed to refute it (470 contradictions raw, **2 after an artifact filter**,
against a registered 250 000). It then located the whole remaining risk in the
half nobody has built: **the roots must supply 20.4 % of every emitted name**
(35 608 of 174 417, ~42/TU), the transitive closure over direct edges adds only
1.7 %, and the root clause is the one `PHASE7_VALIDATION.md` §6b proved
internally inconsistent on its face. **Two lanes have declined to fit a root
model.** This lane does not fit one either. It tests whether the root set is
something to **read** rather than model.

## 1. The claim under test, verified against the binary before this file was written

Board `#168` and `docs/whitebox/C2_MAP.md` §3E state the reading. **I confirmed
every load-bearing instruction myself with `objdump` on
`compilers/X360/16.00.11886.00/c2.dll` (image base `0x10b00000`, `.text` VA
`0x1000` @ file `0x400`) before freezing this file.** What I confirmed:

| where | bytes | what it does |
|---|---|---|
| `10b7f16b` | `8b 50 4c` / `f6 c2 20` / `74 05` / `f6 c2 02` / `74 21` | `mov edx,[eax+0x4c]; test dl,0x20; je skip; test dl,0x2; je COMPILE` — the walk-loop test in `p2/main.c` |
| `10b9bf70` | `e8 a6 39 08 00` / `83 e0 fb` / `89 46 4c` | `call varU; and eax,~0x4; mov [esi+0x4c],eax` — **the flag word read verbatim from the IL**, in `FUN_10b9b8e9` (`p2symtab.c`) |
| `10b9c02b` | `e8 eb 38 08 00` / `89 46 4c` | tag `0x10` records also set `+0x4c` from a `varU`, **without** the `~0x4` force-clear |

**The codec primitives, decoded from the binary rather than taken from the
docs** (all read through the stream pointer at `ds:0x10c46310`):

| fn | name | encoding |
|---|---|---|
| `10c1f8fc` | `GetByte` | one raw byte |
| `10c1f91b` | **`varU`** | `b0 | (b1<<8)` when `b1 & 0x80 == 0`; else 4 bytes, `b0 | ((b1&0x7f)<<8) | (b2<<15) | (b3<<23)` |
| `10c1f9a6` | `i16c` | signed byte, unless the byte is exactly `0x80` → LE16 follows |
| `10c1f9e9` | `i32c` | signed byte, unless the byte is exactly `0x80` → LE32 follows |
| `10c1fc5b` | `GetCStr` | NUL-terminated name |

**The tag-`0x0e` tail, read off `10b9bf57`–`10b9bf80`, is the decode this lane
relies on:**

    i32c -> +0x54   (.ex body-start offset)      <- the ANCHOR
    i32c -> +0x58   (.sy offset)
    i16c -> +0x50
    varU -> +0x4c   <<<< THE FLAG WORD
    i16c -> +0x52
    [ i32c/i16c count + count x (varU,i16c) , only if +0x4c & 0x1000 ]

So the flag word sits at **no fixed offset**, but it is exactly **three fields
past a value that `.ex` can independently confirm**: `+0x54` must be one of the
`4F 1F` function-start offsets. That cross-check is the instrument's gate and it
is not negotiable (§4, KA-D).

**One correction to `C2_MAP.md` §3E already found and recorded here, before any
measurement:** §3E's byte-level walk of `?zero_test@@YAII@Z` labels
`0x0c4 00 00` as `varU -> +0x0c owner idx` and flags the gate as unresolved. The
binary resolves it: at `10b9be6b` the owner `varU` is read **only if the `+0x20`
flags word has bit `0x200` set**, and §3E itself records `+0x20` decoding to
`0x005/0x105/0x405` in every record across three bundles. **Those two bytes are
therefore not the owner index**, and §3E's chain is off by one field there. This
lane decodes from the binary, not from §3E's walk.

## 2. Definitions, frozen

    Seed(t)  = { name of a tag-0x0e .gl record in TU t :
                 (flags4c & 0x20) and not (flags4c & 0x02) }
    E(t)     = truth: COMDAT leaders of every IMAGE_SCN_CNT_CODE section
               (w-emit's definition, byte-identical reader, unchanged)
    U(t)     = names with a `.gl`-named `.ex` body (model.named_bodies)
    26-edge  = exb[p-1] == 0x26, the direct call/reference edge kind
               (w-emit's TIGHT extractor, 99.663 % of its targets emitted)
    Rfloor(t)= { f in E(t) : no 26-edge from any emitted body reaches f }
               — w-emit's ROOT FLOOR, 35 608 names / 20.4 % of |E|

Population: the **850** TUs of `work/emitpred/magnitude/truthlist.txt`.
**The 21-TU quarantine of `_2026-08-02-w-emitpred-prereg.md` is honoured and
NOT spent** — see §5.

**Instruments are w-emit's as landed and are NOT to be tuned**:
`pipeline/model.named_bodies`, `pipeline/il.gl_symbol_index`, strict
(`.gl`-named owner) attribution, `26`-edges only. Variants may be reported
*beside* a headline, never instead of it.

## 3. Frozen predictions — points, with intervals, against named incumbents

**Incumbents** (the model must be compared to these, never to a bare
threshold): **emit-everything** = predict all of `U`, the port's behaviour
today, micro-precision **0.11562**, recall 1.0, **F1 0.2073**; **never-emit** =
predict nothing, F1 **0**, ~93 % per-body accuracy on this workload.

| # | quantity | registered point | interval | refuted if |
|---|---|---:|---|---|
| **S1** | **seed containment** — `\|Seed ∩ E\| / \|Seed\|` over the workload | **0.97** | [0.90, 1.00] | **< 0.90** ⇒ the `p2/main.c` reading or my decode is wrong; §5 decline clause 2 fires |
| **S2** | **seed share** — `\|Seed\| / \|E\|` | **0.25** | [0.10, 0.60] | ≥ 0.95 ⇒ `0x20` is not a seed but the whole answer and the closure is a no-op; ≤ 0.02 ⇒ not the root channel |
| **S3** | **ROOT COVERAGE — the headline** — `\|Rfloor ∩ Seed\| / \|Rfloor\|` | **0.85** | [0.55, 1.00] | **< 0.55** ⇒ `0x20` is REFUTED as a root oracle; §5 decline clause 3 fires |
| **S4** | **closure F1** — micro-F1 of `closure_26(Seed)` against `E` | **0.90** | [0.70, 0.98] | F1 does not beat emit-everything's **0.2073** by ≥ **20 pp** ⇒ refuted as a model regardless of anything else |
| **S5** | **per-TU exact sets** — fraction of 850 TUs where `closure_26(Seed) == E` | **0.10** | [0.01, 0.45] | reported, not gated |
| **S6** | **seed density** — mean `\|Seed\| / \|U\|` per TU | **0.03** | [0.005, 0.15] | reported, not gated |
| **S7** | **`0x20`-set share of `U`** — `\|{flags4c & 0x20}\| / \|U\|` before the `0x02` mask | **0.03** | [0.005, 0.15] | reported; if the `0x02` mask changes this by > 1 % the mask is load-bearing and must be said so |

**Bias declaration.** I was briefed that a refutation is worth more than a
partial model, and that "the binary is the source of truth and it is
underused". That is a bias **towards** the reading being right. I have
therefore registered S1 and S3 **high** — where a miss costs me — rather than
hedging low. If S3 lands below 0.55 the reading is refuted and I say so in the
first line of the findings.

**What I predict does NOT move:** TU match stays **8**. `census/gate
disagreement` stays **0**. No `crates/` change, no fixture, no widening —
`PortC2` keeps returning `NotImplemented` outside its class.

## 4. Known-answer controls — registered pass marks

| # | control | pass mark |
|---|---|---|
| **KA-A** | reproduce w-emit's population on the same 850 TUs with the same readers: `\|E\|` = 174 417, `\|U\|` = 1 508 530 | within **0.5 %** of both |
| **KA-B** | **the mutation test, against the SOLE JUDGE** — pick 6 leaf functions in a real workload TU whose decoded `flags4c` has `0x20` set, clear that bit at the byte offset **my decoder** reports, replay the mutated IL through real `c2.dll` under wibo, compare objs | **≥ 4/6** lose exactly that COMDAT. This validates the byte position *and* the semantics with the real oracle, not with a model |
| **KA-C** | **the inverse mutation** — pick 3 records whose `flags4c` lacks `0x20`, set it, replay | **≥ 2/3** gain that COMDAT. Sufficiency, not just necessity. A miss here is informative and will be reported as such, not buried |
| **KA-D** | **decode chain gate, fail-closed** — the anchors found must be 1:1 and in order with the `.ex` `4F 1F` offsets, and every field my codec consumes must **re-encode to the exact bytes it consumed** | ≥ **95 %** of TUs gate-clean; TUs that fail are **excluded and counted**, never decoded anyway |
| **KA-E** | incumbent gate on the unmodified tree (`cargo test --workspace --release`, `gate.sh --jobs 6`, `selftest`, `cargo build --release`) | every incumbent reproduced; **compare the FAILED count, never the passed count** |
| **KA-F** | dc3-decomp HEAD recorded **before and after** the run; wibo `--version` recorded | a mid-run corpus move is a **hard error**, not a footnote |

## 5. Decline clauses — priced, and each says what I will conclude

1. **KA-B < 4/6, or KA-D fails on > 5 % of TUs** ⇒ **the decode is not
   trustworthy. Publish no precision/recall number at all.** Report only that
   the decode failed, at which field, and on what population. A number from an
   ungated decoder is worse than no number.
2. **S1 < 0.90** ⇒ **decline to report S4 and S5.** A seed set not contained in
   `E` means the reading or the decode is wrong; any downstream F1 computed on
   top of it would be an artifact of whichever half is broken. Report instead
   the *structure* of `Seed ∖ E` — mangling classes, sizes, section fates.
3. **S3 < 0.55** ⇒ **declare `0x20` REFUTED as a root oracle, say so in the
   first line, and STOP.** Specifically: **I will not scan the other bits of
   `flags4c` against `E` looking for one that correlates better.** That is
   fitting, it is exactly the "third lane to fit a root model by accident" the
   brief names, and a bit found that way would be a coincidence with 31
   degrees of freedom. **I may adopt a different bit only if the BINARY names
   it** — a different instruction, at a named address, reading a different bit
   — and I will quote the instruction if I do.
4. **No instrument tuning to improve a number.** The name-binding window, the
   folding rule, the edge kind and the strict/local split are w-emit's as
   landed. If I report a variant it goes *beside* the headline with its own
   label, never in place of it.
5. **Nothing ships under `crates/`.** Even if S1–S5 all hit, this lane is a
   measurement: one lane's decode is not grounds for widening the gate, and the
   port must keep returning `NotImplemented` outside its class. Implementation
   is a separate lane with its own prereg.

### 5a. The one-shot Part-1 gate — I pre-register that I will NOT spend it

w-emitpred's 21-TU held-out quarantine is unspent and is the root model's gate.
**I register, before seeing any number, that I do not intend to spend it**, for
a reason that is about the object and not about convenience:

> A **decode has no free parameters.** The held-out gate exists to catch a model
> fitted on the population it is then scored against. `Seed` is read out of the
> IL by a chain of five field reads taken from the disassembly; there is nothing
> in it that dev truth could have tuned. 850 TUs with zero fitted parameters is
> stronger evidence than 21 TUs, and spending the gate here would foreclose it
> for the first model that actually *has* parameters.

**The condition that would reverse this, registered now:** if at any point I
choose *anything* — a bit, an edge kind, a tie-break, an exclusion — by looking
at dev truth, then the result is fitted, I will **say so explicitly**, and the
gate becomes **owed** by whoever ships it. **I will not spend it myself in
either case**, because a lane that discovers it has fitted something is the
worst-placed party to also run its only clean test.

## 6. What this lane will NOT measure — named now, so absence never reads as success

1. **The `.gl` per-symbol reference list** (`PHASE7_VALIDATION.md` §7). c2's
   closure runs over *that* list, not over `.ex` `26`-tokens. S4 uses w-emit's
   `26`-edges as a **proxy**, and any S4 number is a statement about the proxy
   as much as about the seed. Decoding the real list is a separate job.
2. **Tag-`0x10` records**, which also carry a `+0x4c` (`10b9c02b`). They have no
   `.ex` body so they cannot be compiled, but they are unmeasured here.
3. **The 21 quarantined TUs.** Untouched. See §5a.
4. **Whether `0x20` is *set* correctly by c1xx.** This lane reads what c1xx
   wrote. Why c1xx wrote it is `c1xx.dll`'s question and is out of scope.
5. **The `-optref` pruner** (`FUN_10b27b7f`), which is the only path that
   *clears* `0x20`. The workload does not pass `-optref`; unverified here.
6. **Order.** A right set in the wrong order is still a mismatch. Not measured.
