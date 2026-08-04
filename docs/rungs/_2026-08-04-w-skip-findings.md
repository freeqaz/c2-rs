# w-skip — the three owner SKIPS are decoded, and the sole judge says they are INERT. The filter is the OWNER'S OWN FATE: 10/10 against 0/10.

    Lane:      w-skip, 2026-08-04, worktree `wt-w-skip` off master `e57e641`
    Prereg:    work/w-skip/PREREG.md (= rungs/_2026-08-04-w-skip-prereg.md),
               committed at `d13efdc` BEFORE any measurement against truth.
               Scored in §7.
    Ships:     NOTHING under `crates/`. No fixture, no codegen, no widening,
               no DISCLOSURE.md row (nothing is adopted).
    Status:    FINDINGS. TU match is 8 at both ends.

**Decline clause 3 fires first, so its result goes first.** ***The registered
causal test M13 FAILED, and it failed the useful way: `([owner+0x20] & 0x60) ==
0x20` — SKIP 1, at `0x10b98e9f` — was set on real owners, replayed through the
real `c2.dll`, and the functions their initializers name were **not** lost, 0/4.
Neither were they lost for SKIP 3 (`0x4000`, 0/3) or for the two gates w-mark did
not name (`0x10b98b09` kind==1, `0x10b98b14` `+0x20 & 0x480`, 0/3 and 0/3). A
whole VALUE SWEEP of `+0x20` leaves the emit set identical — while a wild value
at the same byte **SIGSEGVs c2**, so the field is located and c2 acts on it.***

**And the experiment that says what the filter actually is came back
discriminating on the first try.** Split w-mark's retarget by whether the
initializer's **owner is itself a defined symbol in the obj**:

> ### **owner emitted → the retargeted function APPEARS, 10/10. Owner not emitted → it does NOT, 0/10.** Two TUs, real `c2.dll`, same mutation shape, opposite outcomes.

`+0x20 = 0x1c01` occurs in **both** arms, which is as sharp a refutation of a
flag-based filter as this corpus can produce. Modelling the skips moves F1 from
w-mark's 0.42496 to **0.50761** — still **−34.5 pp against w-refs' 0.85260**, so
the model half of this page is published as a **refuted hypothesis**, not as a
predicate. **w-mark's §9 item 6 — the one thing it named and did not test — is
the answer, and it makes the emit set a joint DATA+CODE fixpoint that no root set
over functions can express.**

---

## 0. Provenance — every number on this page

| | |
|---|---|
| c2-rs branch | `wt-w-skip`, based on master **`e57e641`** (the merge of `wt-w-mark`) |
| c2-rs HEAD at the prereg | **`d13efdc`**, clean — **no `crates/` change exists in this lane** |
| **dc3-decomp HEAD BEFORE** | **`940d07dcb0960964ad61aa5f025658f993eb46b2`** |
| **dc3-decomp HEAD AFTER** | **`940d07dcb096…`** — **it did not move** (`work/w-skip/prov_{before,after}.txt`) |
| wibo | **`1.0.1-23-g4a9dd6f`**, checked at lane start, **not stale** (`work/w-skip/wibo.txt`) |
| c2.dll read | `compilers/X360/16.00.11886.00/c2.dll`, sha256 `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`, image base `0x10b00000`, `.text` VA `0x1000` @ file `0x400` |
| `gl` / `ex` / truth | **w-emit's cache unchanged**, 876 IL / 1700 truth, 850 graded |
| `in` | **w-mark's capture**, re-verified: 8 random non-quarantined TUs re-captured with the same `cl /Bd /d2nop` recipe are **byte-identical, 8/8** (KA-F2) |
| flags | the unmodified workload line, `work/dc3-workload/flags.txt` |
| scratch | `work/w-skip/` (gitignored); scripts and text outputs force-added, no IL or obj committed |

**The 21-TU quarantine is intact and w-emitpred's one-shot Part-1 gate is
UNSPENT** — §9. Every mutated TU was checked against `heldout.txt` by name.

---

## 1. The decode, re-verified by address — and the header that makes it possible

`work/w-skip/dis.sh <va> [n]` reproduces every line below from the bytes. I
re-read `0x10b98e26`, `0x10b98b00`, `0x10b98c0f`, `0x10b276e4`, `0x10b9bdcf`,
`0x10b9b945`, `0x10b9bf99`, `0x10b27f3c`, `0x10b27ec7`, `0x10b984c3` and the p2
driver `0x10b7f022` rather than inherit them, and the re-read changed three
things.

### 1a. The KIND-1 `.gl` record header, decoded here for the first time

Nothing in `work/` could read `[owner+0x20]` before this lane, because nobody had
decoded the record kind that owns an initializer. The tag dispatch is
`0x10b9b922` (GetByte) → index table at `0x10b9c615` → jump table at
`0x10b9c5d5`, and `0x10b98521` (`mov BYTE PTR [esi+0x30],bl`, inside the
allocator `0x10b984c3`) shows **`[sym+0x30]` is just the record tag** unless an
arm overwrites it:

    tag 0x04 / 0x0e / 0x10 -> 0x10b9bdcf   KIND 4   (0x10b9bdfb writes 4)
    tag 0x01 / 0x02 / 0x1a -> 0x10b9b945   KIND 1   (0x10b9b95f writes 1)
    tag 0x03               -> 0x10b9bd3d   KIND 3
    tag 0x09               -> 0x10b9c212   KIND 9   the TYPE record

**KIND 1**, from the 0xa0-byte stack template memset at `0x10b9b945`
(`lea eax,[ebp-0x6c]`, so `[ebp-0x1f]` = `+0x4d`, `[ebp-0x3c]` = `+0x30`,
`[ebp-0x44]` = `+0x28`, `[ebp-0x4c]` = `+0x20`):

    <tag> <byte -> +0x4d, saved 0x10b9b957 / stored 0x10b9b9d4>
          <varU tok -> +0x28>  <byte -> +0x31>  <name>
          <byte, READ AND DISCARDED at 0x10b9b9cc>
          <byte -> +0x37 bits 21..23>   <byte -> +0x37 bits 5..8, storage class>
          <i32c -> +0x1c>
          <varU -> +0x20>                          0x10b9ba0d  THE FLAG WORD
          [ if (+0x20 & 0x200): <varU tok> -> +0x0c ]     0x10b9ba5f

Hand-decoded against the bytes on `HttpReq.cpp`: `??_7exception@std@@6B@` reads
`… 02 00 ac 15 26 <name> 00 | 86 06 00 04 01 0c …` → tag `0x02`, `+0x4d` = 0,
token `0xac15`, then three bytes, `i32c` 4, `varU` **`0x0c01`**; the next record
header follows at exactly the predicted offset. The corpus gate is stronger:
**the `+0x20` varU round-trips to the same bytes on 2 744 921 of 2 744 921
records (1.00000)** and its value histogram is **concentrated, top-8 = 0.81225 of
kind-1 records** — a desynced read would be neither (KA-C).

**This also finds a decode defect in w-refs' `refs.head`**, which reads the
`+0x0c` token *unconditionally* while `0x10b9be6b` gates it on `+0x20 & 0x200`.
It is harmless there — `head` only has to locate the `+0x54` anchor and the
`0x80 <LE32>` / `.ex`-offset gate catches a miss — but it is recorded, because an
omitted gate is a decode defect whether or not it fires.

### 1b. The three skips, and the two gates w-mark did not name

`0x10b98e26`'s loop, transcribed:

    for (rec = *(0x10c67db4); rec; rec = [rec+8]):
      G0   0x10b98e4a  if ([rec+0x20] & 1) continue;
                       owner = Resolve([rec+0x10], curmodule);        0x10b98e67
                       if (!owner) continue;
      S1   0x10b98e9f  if (([owner+0x20] & 0x60) == 0x20) continue;   <<< SKIP 1
                       r = WalkInit(owner);                           0x10b98eae
                       if (r == 0) { 0x10b98de4(rec); continue; }
      S2   0x10b98ecd  if ([[owner+0xc] + 0x4d] == 0x1d) continue;    <<< SKIP 2
      S3   0x10b98ed9  if ([owner+0x30]==1 && ([owner+0x20] & 0x4000)) continue;  <<< SKIP 3
           0x10b98ee7  RecurseSym(owner, [owner+8]);

| # | site | field tested | what it means structurally | static, or codegen state? |
|---|---|---|---|---|
| **SKIP 1** | `0x10b98e9f` | `([owner+0x20] & 0x60) == 0x20` | the owner's `.gl` flag word (the `varU` at `0x10b9ba0d`). Skips the record **entirely** — neither pass runs | **STATIC**; read verbatim from the stream |
| **SKIP 2** | `0x10b98ecd`, and `0x10b98ba8` inside the walk | `[[owner+0xc] + 0x4d] == 0x1d` | the owner's **type**: `+0x0c` is a symbol resolved from a `varU` token when `+0x20 & 0x200`, else the module default `[0x10c472e8+0x2cc]`; `+0x4d` is that record's kind byte (written from the stream at `0x10b9c247` for a kind-9 record, and from a parameter at `0x10be7496` for a synthesized one). At `0x10b98ecd` it skips the `0x10b98c0f` pass; **inside `0x10b98b00` the same test is not a skip but an early-out — it Marks that one target and returns 0**, aborting the rest of the node list | **STATIC** |
| **SKIP 3** | `0x10b98ed9` | `[owner+0x30]==1 && ([owner+0x20] & 0x4000)` | kind is the record tag; the bit is the same flag word. Skips the `0x10b98c0f` pass | **STATIC** |
| **W1** *(unnamed by w-mark)* | `0x10b98b09` | `[owner+0x30] != 1` → return 1 | only kind-1 (tag-0x01/0x02) owners' initializers are walked at all | **STATIC** |
| **W2** *(unnamed by w-mark)* | `0x10b98b14`, repeated at `0x10b98c89` | `!([owner+0x20] & 0x480)` → return 1 | the owner must carry `0x80` or `0x400` | **STATIC** |

**All five are static properties of the owner as the `.gl` reader wrote it.**
Nothing between the reader and `0x10b98e26` writes `+0x20`, and `0x10b98e26` runs
before any function is compiled (§2). The only field in the walk that *is*
codegen state — `[t+0x4c] & 2`, the "compiled" bit set at `0x10b7f199` — is read
by SKIP 2's early-out at `0x10b98b9f` at a time when nothing has been compiled,
so its value there is the stream's DONE bit. `[owner+0x32] & 4` is the walk's own
DFS re-entrancy marker, set and cleared inside the walk.

### 1c. Two corrections and one confirmation of landed claims

* **CONFIRMS w-refs' edge relation and names the mechanism it did not.** The
  `.gl` reader pushes `(token, refcount)` nodes onto **`[head+0x14]`**
  (`0x10b9bffb`/`0x10b9c000`), but `Mark` walks **`[head+0xc]`** (`0x10b27715`),
  whose nodes hold a *pointer* — `[[n+4]+4]` is the target symbol. These are two
  different lists on one head object (`0x10b27db2`: `[head+4] = sym`,
  `[sym+0x80] = head`). `0x10b27f3c`, called at `0x10b7f0b5` **before**
  `0x10b98e26`, is the pass that resolves `+0x14` into `+0xc`, and it keeps an
  edge only when the target is `[+0x30]==4 && [+0x37]&0x200000` — a tag-`0x0E`
  function record, i.e. a member of `U` — with a non-zero use count
  (`0x10b27fd1`) and a token `>= 0x20` (`0x10b27f8b`), via `0x10b27ec7`, which
  drops it again if the target has no head object. **So w-refs' `∩ U` restriction
  and its zero-use drop are not conveniences of the model — they are that pass.**
  w-refs prints `ecx[0xc]` next to a reader that writes `+0x14` without noting
  they differ; this is why they agree.
* **CORRECTS w-mark §1d on S5's reach.** `0x10b98c0f` has **three** callers:
  `0x10b98d2e` (its own recursion), `0x10b98ee7` (the driver) and `0x10b98fa6`.
  w-mark lists `0x10b9ac38`; that is two frames away
  (`0x10b9ac38` → `0x10b98f0a` → `0x10b98fa6`).
* **NAMES a near-clone nobody has recorded.** `0x10b98f0a` is a second walk of
  the same shape over a *different* list (`[module+0x28]`), carrying **SKIP 1**
  (`0x10b98f76`) and **SKIP 3** (`0x10b98f98`) but **not** SKIP 2, reached from
  `0x10b9ac38` ← `0x10b34026`. Because SKIP 1 appears in *both* drivers, §4's
  arm G1 tests both at once.

---

## 2. THE WORKLIST QUESTION, answered from the disassembly before any fitting

**w-mark's R-d is CONFIRMED.** `0x10b7f1e5: je 0x10b7f15f` re-reads
`ds:0x10c4630c` after every compiled function, and `0x10b7f199` unlinks the
symbol first, so the compile loop is a worklist run to a fixpoint during codegen.
(The restart is itself gated on the mode word `ds:0x10c462c4`, which is 0 for an
ordinary compile.)

**And it does not reach this channel.** A full `E8`/`E9`-plus-absolute scan
(`work/w-mark/xrefs.py`) finds `0x10b98e26` with **exactly one caller**,
`0x10b3413d` inside `0x10b34113`, which itself has **exactly one caller**,
`0x10b7f0d2` — *before* the compile loop at `0x10b7f15f`. The initializer walk
runs once per module, before any function is compiled, and the worklist never
re-enters it.

> ### **So the answer registered in the prereg was: the model does NOT need to be a worklist for this channel, because every field the skips read is written by the `.gl` reader and by nothing else. That half stands. The other half — that a static root set is therefore the right SHAPE — is REFUTED by §5, and not by ordering.**

The refutation is not about time; it is about what the predicate ranges over.
The gate that decides whether an initializer contributes roots is **whether the
owning data symbol is emitted**, and *that* is itself part of the emit fixpoint.
A model can be unordered and still need to be a **joint fixpoint over data and
code symbols** rather than a root set over functions closed under reference.
Those are different claims and the prereg conflated them; this is the correction.

**What the worklist *is* needed for is named and not modelled here:**
`0x10b3389b` (`dag.c`, reached from compile-one-function) adds *edges* during
codegen, and `0x10b9aa26` (the by-name intern) adds *roots* during codegen.

---

## 3. THE MEASUREMENT — 850 TUs, 174 417 emitted names, one variable changed

`work/w-skip/marks.py` replays `0x10b98e26` / `0x10b98b00` / `0x10b98c0f`
instruction for instruction with all five gates; `scan.py` swaps only the root
set and recomputes both incumbents in the same pass.

| | **`RGL`** w-refs (best F1) | **`RGL+INIT`** w-mark (best recall) | **`RGL+INIT_skip`** THIS LANE |
|---|---:|---:|---:|
| \|P\| | 129 604 | 613 532 | **400 998** |
| **precision** | **1.00000** | 0.27289 | **0.36420** |
| **recall** | 0.74307 | **0.95991** | **0.83732** |
| **micro-F1** | **0.85260** | 0.42496 | **0.50761** |
| per-TU exact `P == E` | **132 / 850** | 34 / 850 | 34 / 850 |
| roots \|I\| | — | 245 148 | **147 599** (0.60208 of w-mark's) |
| root soundness \|I∩E\|/\|I\| | — | 0.14086 | **0.09512** |
| root-floor coverage | 0.18796 | 0.86926 | 0.53626 |

> ### **F1 0.42496 → 0.50761: +8.27 pp over w-mark, −34.50 pp against w-refs.**
> The registered wash band was ±2.0 pp around **0.85260**. 0.50761 is a
> **REGRESSION against the best incumbent**, and decline clause 1 fires.

The `STRICT` variant — dropping, rather than passing through, the 20.1 % of `in`
records whose owner token this decoder cannot bind — is **400 775 / 0.36388 /
0.83613 / 0.50709**, i.e. indistinguishable. The headline uses the inclusive
variant on purpose: a decoder's blind spot must never be allowed to look like a
filter.

### 3.1 The coincidence calibration decline clause 2 demands

| | measured | expected under uniform coincidence | ratio |
|---|---:|---:|---:|
| w-mark, unfiltered `I ∖ P_RGL` (242 118 names, 31 502 emitted) | 0.13011 | 0.03254 | **4.00×** |
| **w-skip, `I_skip ∖ P_RGL` (146 349 names, 12 789 emitted)** | **0.08739** | 0.03254 | **2.69×** |
| w-emit's disqualified loose non-`26` scan (P-e) | 0.0277 | 0.0260 | 1.07× |

And against the base rate `|E|/|U|` = **0.11577**, i.e. the precision of
predicting every function:

* w-mark's roots: soundness 0.14086 = **1.22× the base rate**;
* **this lane's filtered roots: soundness 0.09512 = 0.82× — BELOW chance.**

> ### **The filter removes 40 % of the roots and makes what is left LESS enriched than picking names out of `U` at random.** That is not a filter fitting badly; it is a filter selecting against the target.

### 3.2 Stratified, so `#152` cannot dominate either direction

`??_G`/`??_E` deleting destructors are **5 344 = 3.06 %** of `E`. Removing them
from both `E` and `P`:

| model | \|P\| | precision | recall | F1 |
|---|---:|---:|---:|---:|
| RGL | 128 781 | 1.00000 | 0.76169 | **0.86473** |
| INIT | 612 136 | 0.27123 | 0.98200 | 0.42506 |
| **SKIP** | 399 710 | 0.36215 | 0.85617 | **0.50900** |

**Every conclusion is unchanged with `#152` excluded** — the ordering, the sign
and the magnitude of every gap survive, so no result on this page rests on that
class. And the residual `E ∖ P_SKIP` (27 864 names) is **53.4 % free /
file-scope functions** — the class w-mark's unfiltered channel closed at
**99.60 %**. The filter's damage lands squarely on the population the channel was
found for; `#152` is 14.6 % of this residual rather than w-mark's 60.9 %, purely
because the other classes came back.

### 3.3 What the gates actually did, corpus-wide

| gate | fires on |
|---|---:|
| `in` records total | 879 377 |
| owner token bound to a decoded `.gl` record | 702 262 (**0.79859**, KA-D) |
| **SKIP 1 — `([owner+0x20] & 0x60) == 0x20`** | **0** |
| SKIP 2 — `[[owner+0xc]+0x4d] == 0x1d` | **0, because the owner's type is UNRESOLVABLE on all 702 263** (§8.1) |
| SKIP 3 — kind-1 with `0x4000` | 17 919 |
| **W1 ∧ W2 pass (the operative filter)** | 432 137 (**0.49141**) |
| `0x10b98ba8` early-out taken | 0 |
| tag-`0x07` flags byte non-zero in the `in` stream | **0** |

**SKIP 1 never fires on this workload.** That was registered at a point of 0 in
the prereg from a disclosed three-TU pilot, and it is why §4 had to *construct*
the condition rather than observe it.

---

## 4. KA-E — the SKIPS against the SOLE JUDGE, in the direction that can go RED

### 4.1 First: is a write at this offset even reaching c2?

Every direct arm came back inert, and inert is exactly what a mislocated write
looks like. `work/w-skip/probe_f20.py` separates the two, on `HttpReq.cpp`:

| probe | result |
|---|---|
| **P1** — write the same bytes back | the obj is **byte-identical** to the baseline. The replay is deterministic (with the output path held fixed; a per-arm path lands *in* the obj and made this red on the first run) |
| **P2** — set `0x200` at the same offset, which makes `0x10b9ba5f` read an extra `varU` | **the obj CHANGES.** The byte is consumed by the `.gl` reader |
| **P2c** — write `0x7DFF` there | **c2 SIGSEGVs.** `wibo: caught SIGSEGV` |
| **P3** — w-roots' seed bit `0x20` at `+0x4c`, in the same script | the target COMDAT **disappears**, 1/1 |

> ### **A wild value at this offset crashes c2 and a no-op reproduces the obj byte for byte. The field is located, and c2 acts on it.** Anything inert after that is a fact about the gate, not about the instrument.

### 4.2 The gate arms — w-mark's retarget, run again with one bit changed

`work/w-skip/mutate_gate.py` reproduces w-mark's KA-D mutation (retarget one `02`
node's `varU` to a function c2 was not going to emit) and then re-runs *the same
retarget* with one bit of the owner's `+0x20` changed. `G0` is the positive
control; `G2` is the discriminating control that says a write to that byte is not
simply destructive.

| arm | owner `+0x20` | prediction | **measured** |
|---|---|---|---|
| **G0** retarget only | `0x5801` | F_new APPEARS | **APPEARS 1/1** — and F_old is lost, so w-mark's 15/15 replicates in both directions |
| **G1** `\|= 0x20` (SKIP 1 fires, in *both* drivers) | `0x5821` | suppressed | **APPEARS — MISS** |
| **G2** `\|= 0x60` (SKIP 1 must not fire) | `0x5861` | APPEARS | **APPEARS — control green** |
| **G4/G6** `\|= 0x4000` / `&= ~0x4000` (SKIP 3) | `0x5801` / `0x1801` | suppressed / APPEARS | **APPEARS both — MISS on G4** |
| **G5** `\|= 0x400` (opens W2) | `0x5c01` | APPEARS | **APPEARS** |
| **S-min** set `0x0001` | `0x0001` | — | **APPEARS** |
| **S-walk** set `0x0481` (W2 wide open) | `0x0481` | — | **APPEARS** |
| **S-max** set `0x7DFF` | — | — | **c2 SIGSEGV** |

and the direct arms, on `EventTrigger.cpp` (`work/w-skip/mutate_skip.py`),
writing the gate bits on owners whose initializers name 9–16 emitted functions
each, all of them outside `closure(Seed)`:

| arm | prediction | measured |
|---|---|---|
| **A** SKIP 1 set | payload lost | **0/3 lost** |
| **B** `0x60` control | nothing moves | **3/3 clean** |
| **C** W2 opened on a refused owner | functions gained | **0/3** |
| **D** W2 closed on a walked owner | payload lost | **0/3** |
| **E** SKIP 3 set | payload lost | **0/3** |

> ### **M13 = 0/4 and M14 = 4/4. The control that could have gone red stayed green while the hypothesis went red, which is the only combination in which a null means anything.**

### 4.3 The experiment that came back discriminating

The one predicate left was the one w-mark **named in its §9 and did not test**:
*whether the `in` owner is itself emitted.* `work/w-skip/mutate_owner.py` runs
the identical retarget, split only by whether the owner's name is a defined
symbol in the baseline obj (widest reading: any symbol with a real section
number).

| TU | baseline leaders | **H+ owner IS in the obj** | **H− owner is NOT** |
|---|---:|---:|---:|
| `src/system/net/HttpReq.cpp` | 63 | **5/5 APPEARS** | **0/5 APPEARS** |
| `src/system/utl/PoolAlloc.cpp` | 77 | **5/5 APPEARS** | **0/5 APPEARS** |
| | | **10/10** | **0/10** |

The owners on either side are the same *kinds* of object — vftables, `??_R0`
type descriptors, `??_R1`/`??_R2` RTTI records, `_CT` catchable types — and

> ### **`+0x20 = 0x1c01` appears in BOTH arms.** `??_R1A@?0A@EA@HttpReq@@8` (0x1c01, emitted) pulls its target in; `??_R0?AVexception@std@@@8` (0x1c01, not emitted) does not.

Two different objects, identical flag word, opposite outcome, same mutation.
That is the refutation of a flag-based filter and the confirmation of an
emission-based one in a single table.

**Could this control have gone red?** Yes, in the most likely failure mode: if
the retarget were carried by something other than the owner's own emission, H−
would have come back green-as-APPEARS and matched H+, which is exactly what
w-mark's unfiltered reading predicts. It did not, 0/10.

---

## 5. What this means for the shape of a Phase-7 model

> ### **c2's emit set is a joint least fixpoint over DATA and CODE symbols. An initializer contributes roots only when its owner is emitted, and whether the owner is emitted is itself part of the answer.**

w-mark reached this conclusion from the disassembly and called it R-e; this lane
**measures it**, and the measurement relocates it: the evidence R-e cited — the
three owner skips — is inert, and the load-bearing predicate is the owner's own
membership in the output. Concretely:

* `??_7HttpReq@@6B@` is emitted because the TU's own class is used, so its slots
  are marked — which is why w-refs' residual is 27 % *virtual members* and why
  w-mark's channel closed them at 96.25 %.
* `??_7exception@std@@6B@` arrives in the `in` stream from a header, is not
  emitted, and its slots are not marked — which is why w-mark's unfiltered
  reading over-predicts by 4.7× and why 86 % of what it names is not emitted.
* `_CT` catchable-type records are emitted when the TU has EH data, which is why
  w-mark's three necessity survivors were all EH copy constructors reached from
  `_CT`.

**This is not expressible as `roots + closure` over functions**, which is the
shape w-emit, w-roots, w-refs, w-mark and this lane all used. A model must carry
data symbols as first-class members of the fixpoint. **It does not, however, need
to be *ordered***: §2 shows the initializer channel reads no codegen-mutated
state, so a fixpoint iterated to convergence is sufficient for it — the ordering
requirement comes from `0x10b3389b` and `0x10b9aa26`, which this lane did not
model.

**And `#152` still stands beyond all of it.** `??_G`/`??_E` deleting destructors
are synthesized by c2 and named by no `02` node, so no initializer model of any
shape reaches them.

---

## 6. Known-answer controls

| # | control | registered pass | measured | |
|---|---|---|---|---|
| **KA-A** | reproduce **both** incumbents exactly | all eight | `\|U\|` **1 506 586**, `\|E\|` **174 417**, `\|E∩U\|` **173 907**, `\|Seed\|` **14 662**; RGL `\|P\|` **129 604** / **1.00000** / **0.74307** / **0.85260** / **132**; INIT **613 532** / **0.27289** / **0.95991** / **0.42496** / **34** | **PASS**, to the digit |
| **KA-B** | the `in` terminus gate unchanged from w-mark | 876/876, 1 885 700 | **850/850 graded clean, 1 885 700 `02` tokens** | **PASS** |
| **KA-C** | owner-header round-trip ≥ 0.999 and `+0x20` top-8 concentration ≥ 0.80 | | **1.00000** (2 744 921/2 744 921) and **0.81225** | **PASS** |
| **KA-D** | `in` owner tokens bound to a decoded record ≥ 0.50 | | **0.79859** (702 262 / 879 377) | **PASS** |
| **KA-E** | **MUTATION against the SOLE JUDGE** | M13 ≥ 3/5, M14 ≥ 4/5 | **M13 0/4 — FAIL**; **M14 4/4 — PASS** | **RED / GREEN**, §4 |
| **KA-F1** | dc3 HEAD before/after; wibo | no mid-run move | `940d07dcb096` → `940d07dcb096`; wibo `1.0.1-23-g4a9dd6f` | **PASS** |
| **KA-F2** | re-capture 8 random non-quarantined TUs' `in`, byte-compare | 8/8 | **8/8 identical, 0 differing** | **PASS** |
| **KA-G** | incumbent gate on an unmodified tree | §10 | §10 | **PASS** |
| **KA-POS** | **positive check** — the run must have GRADED something, printed as counts | both > 0 | `P_SKIP ^ P_INIT` = **212 534**; `P_SKIP ^ P_RGL` = **271 394** | **PASS** |

---

## 7. Scoring the pre-registration — 9 hits, 3 misses, 1 pass, 1 fail

| # | registered **point** | interval | measured | |
|---|---|---|---|---|
| **M1** | precision **0.35** | [0.27, 0.80] | **0.36420** | **HIT** — inside, just above the point |
| **M2** | recall **0.93** | [0.78, 0.96] | **0.83732** | **HIT** — inside, below the point |
| **M3** | **F1 0.51** | [0.40, 0.86] | **0.50761** | **HIT** — essentially *at* the point; **−34.50 pp vs w-refs → decline clause 1 FIRED** |
| **M4** | per-TU exact 0.06 (51/850) | [0.02, 0.30] | **0.04000** (34/850) | **HIT**, below |
| **M5** | soundness **0.20** | [0.13, 0.70] | **0.09512** | **MISS below the interval** → **decline clause 2 FIRED** |
| **M6** | `\|I_skip\|` 185 000 | [60 000, 246 000] | **147 599** | **HIT** |
| **M7** | `\|I_skip\|/\|I\|` 0.75 | [0.25, 1.00] | **0.60208** | **HIT** |
| **M8** | W1∧W2 pass fraction 0.60 | [0.35, 0.85] | **0.49141** | **HIT** |
| **M9** | SKIP 1 fires on **0** records | [0, 20 000] | **0** | **HIT**, exactly at the point |
| **M10** | SKIP 3 fires on 9 000 | [500, 60 000] | **17 919** | **HIT** |
| **M11** | owner type-kind decodable **0.90** | [0.40, 1.00] | **0.00000** | **MISS, at the floor** — §8.1 |
| **M12** | root-floor coverage 0.80 | [0.30, 0.90] | **0.53626** | **HIT** |
| **M13** | **SKIP 1 positive 5/5**, pass ≥3/5 | — | **0/4** | **FAIL** |
| **M14** | **SKIP 1 control 5/5**, pass ≥4/5 | — | **4/4** | **PASS** |

**The declared bias was that this lane's own hypothesis would fail, and I
registered M1/M3 low from a disclosed pilot so that being wrong would cost me.
M1 and M3 landed almost exactly on their points — I was right about the size of
the effect and right about the direction, and the thing I got wrong is M13, the
one number that was a claim about *causation* rather than about a correlation.**
Registering the observational numbers correctly and the causal one at 5/5 when it
is 0/4 is the honest summary of where the decode was strong and where it was not.

### 7.1 The decline clauses — two fired, one control failed, all honoured

* **Clause 3 (M13 fails → report FIRST) TRIGGERED and honoured.** It is the first
  paragraph of this page, §1b is marked as causally refuted rather than
  confirmed, and M14 is reported next to it so the null is readable.
* **Clause 1 (F1 < 0.87260) FIRED.** Honoured: the model half is published as a
  refuted hypothesis. **I did not go looking for a further channel after the
  number arrived** — §4.3's owner experiment was the *named continuation of the
  same measurement* (the only remaining predicate, taken from w-mark's own §9
  list) and it was run as a mutation against the sole judge, not fitted against
  `E`. Everything else is named in §8 and left undecoded.
* **Clause 2 (M5 < 0.50) FIRED.** Honoured: §3.1 publishes the calibration in
  w-mark's exact shape, including the fact that it is **below** the base rate.
* **Clause 4 (no instrument tuning after truth) — HONOURED for every scored
  number, with three disclosures.** `glowner.py`, `marks.py` and `scan.py` were
  **not** changed after any truth was read; the second scan run added the two
  `∖ P_RGL` counters clause 2 needs and left M1/M2/M3 bit-identical
  (0.36420 / 0.83732 / 0.50761 both times). What *did* change after seeing a
  replay's leader set, all of it in the **mutation harness's candidate
  selection**, none of it in a scored definition: (i) `mutate_skip.py`'s payload
  rule widened from "emitted and outside `closure(Seed)`" to "emitted", after the
  narrow rule yielded 0 candidates on `HttpReq.cpp` — the narrow count is printed
  beside every row; (ii) `mutate_gate.py`'s candidate rule dropped the
  "gate-open" restriction after it yielded 0 candidates, which is *itself* the
  first evidence that the load-bearing owners are the ones my gates exclude;
  (iii) arms G5/G6 and the value sweep were added after G0–G4 came back inert.
  All three widen evidence *against* this lane's hypothesis.
* **Clause 5 (nothing ships) HONOURED.** No `crates/` change; `PortC2` still
  returns `NotImplemented` outside its class; no `DISCLOSURE.md` row is owed.
* **Clause 6 (`Rfloor` is not a decline key) HONOURED.** M12 is reported for
  comparability only.
* **Clause 7 (a refuted §2b/§2c goes before any headline) TRIGGERED and
  honoured.**

### 7.2 Registered before the numbers existed, restated against them

* **TU match stays 8.** It did — 8 at both ends.
* **`census/gate disagreement` stays 0.** It did.
* **A high recall is not a shippable predicate.** Precision 0.364 means a
  fail-closed `Emit/Skip/Unknown` built on this would be wrong about two names in
  three it claims.
* **Order is untouched.** A right set in the wrong order is still a mismatch.
* **The single outcome I said I most expected to be wrong about was M1.** I was
  not wrong about M1. I was wrong about M13, which I registered at the ceiling.

---

## 8. What this lane did NOT measure — named, so absence never reads as success

1. **SKIP 2 is UNMODELLED IN FACT, not only in effect.** `type_known` is **0** on
   all 702 263 bound owners: only ~9 % carry `+0x20 & 0x200` at all, and for
   those the fail-closed `09 <enc(tok)>` search never resolved to exactly one
   kind-9 record. `[[owner+0xc]+0x4d]` is therefore **never evaluated** in the
   scan, and every SKIP-2 row in §3.3 reads 0 for that reason and not because the
   test is false. **M11 is a miss at the floor and this is what it means.**
2. **G0 — `[rec+0x20] & 1`.** The tag-`0x07` flags byte in the `in` stream is 0
   on all 879 377 records, so that is not its source; where it is written is
   undecoded.
3. **Which Mark site actually carries the retarget.** §4.2's G1 rules out both
   `0x10b98e9f` sites (`0x10b98e26` and the `0x10b98f0a` clone). The remaining
   candidates — S1 `0x10b28ca3` (the COFF writer's COMDAT-associative channel),
   S2 `0x10b3389b` (`dag.c`, during codegen) and S6 `0x10b9aa26` (the by-name
   intern) — are **not** discriminated. **The lane knows the predicate and not
   the mechanism, and says so.**
4. **The owner-emitted filter corpus-wide.** §4.3 is 10/10 vs 0/10 on two TUs
   through real c2; the 850-TU version needs a truth capture that records
   **defined data symbols**, which `work/w-emit/truth` does not — it holds code
   COMDAT leaders only. **That capture is the next lane's first task**, and no
   number on this page is a corpus-wide measurement of it.
5. **`0x10b98de4`**, called when the walk returns 0.
6. **`0x10be7006`** (`0x10be70cc`/`0x10be70d4`), called between the recursion and
   the Mark.
7. **`db` and `sy`.** Still uncaptured.
8. **Node kind `0x14`.** `[n]==2 || [n]==0x14` in memory; only the stream's `0x02`
   byte kind is decoded.
9. **`-optref`** (`0x10b27b7f`), the only path that clears `0x20`. Absent here.
10. **Order.** A right set in the wrong order is still a mismatch.
11. **The 21 quarantined TUs.** Untouched (§9).

---

## 9. The one-shot Part-1 gate — NOT spent, as pre-registered

The 21-TU quarantine is intact and w-emitpred's Part-1 gate is **still runnable
exactly once**, five lanes running. **This lane came closest to having a claim on
it and has the clearest reason not to spend it: its model was refuted by a
mutation, in-sample, on 850 TUs. A held-out set cannot improve a refutation.**

**The registered reversal condition did not trigger, and I checked it honestly.**
No parameter in `glowner.py`, `marks.py` or `scan.py` was chosen by looking at
`E`: every mask (`0x60`, `0x480`, `0x4000`, `0x1d`, `0x200`, `0x200000`, `0x400`)
is transcribed from a named instruction, the header layout comes from a memset
offset, and after M3 came in at 0.508 I changed nothing in the scan. §7.1 clause
4 discloses the three candidate-selection changes in the *mutation harness* and
each of them widened the evidence against this lane.

**The gate is still owed by whoever first ships a root model with fitted
parameters. §4.3 says that model will be a joint DATA+CODE fixpoint, and its
first version will have parameters — so it is that lane's gate to spend.**

---

## 10. Gate — every incumbent reproduced, on a tree with no `crates/` change

| | incumbent (re-measured, not transcribed) | **this tree** |
|---|---|---|
| `cargo test --workspace --release` | `690 passed, 0 failed, 25 targets` (master `e57e641`) | **690 passed, 0 failed, 25 targets** |
| `cargo build --release` | 0 warnings | **0 warnings**, 0 in the test build too |
| `c2rs selftest` | 219 PASS (stale) | **222 PASS, 0 FAIL** |
| `scripts/gate.sh --jobs 6` | 12/12 PASS, 2 628 verdicts (stale), 0 mismatch | **12/12 PASS, 2 664 verdicts, 0 mismatch, 0 SKIP, 0 NO-RESULT** |
| TU match / mismatch / codegen-gap / vocab-gap / capture-fail | 8 / 0 / 0 / 863 / 7 | **8 / 0 / 0 / 863 / 7** |
| A / B / C / D / E | 28 / 338 / 114 / 8 / 2 | **28 / 338 / 114 / 8 / 2** |
| `A∧B∧C` / `A∧B∧C∧(D∨E)` / `B∧C` | 25 / 8 / 107 | **25 / 8 / 107** |
| FRONTIER | 17 | **17** |
| **`census/gate disagreement`** | **0** | **0** |
| capture cache | 871 hit, 7 miss, **0 POISONED** | **871 hit, 7 miss, 0 POISONED** |

*Compared on the **FAILED** count and the **target** count, never the passed
count — a failing target aborts the run, so a lower passed count reads as green.*
**w-mark's gate table quotes 689 and w-emit/w-roots/w-refs quote 687; both are
stale for `e57e641`, and so are the fixture counts — this tree measures 690 tests,
**222** selftest fixtures (not 219) and **2 664** gate verdicts (not 2 628),
because master grew fixtures under those lanes. FAILED is 0 and targets is 25.
Every number in this column was re-measured, not transcribed.**

---

## 11. Proposed board rows — **numbers NOT minted**

Same discipline as w-roots, w-emit, w-refs and w-mark: **no number minted, no
`#N` pinned in code, `BOARD.md` / `ROADMAP.md` / `rungs/INDEX.md` untouched by
hand** (w-book2 owns the board). Assign at merge.

| proposed | item | claim | where |
|---|---|---|---|
| **T-a** | **The three owner SKIPS of `0x10b98e26` are CAUSALLY INERT for the emit set** — `([owner+0x20]&0x60)==0x20`, kind-1 `&0x4000`, and the two gates w-mark did not name (`0x10b98b09`, `0x10b98b14`) were each constructed on real owners and replayed through the real `c2.dll`: **0/4, 0/3, 0/3, 0/3**, while the `0x60` control stayed clean **4/4** and a wild value at the same byte **SIGSEGVs c2** | the registered M13 at 5/5 measured 0/4; the instrument is graded independently by a no-op write that reproduces the obj byte for byte and by w-roots' seed flip in the same script | this file §4 |
| **T-b** | **The filter is the OWNER'S OWN FATE: an initializer contributes roots only when the owning DATA symbol is itself emitted — 10/10 against 0/10** through real `c2.dll` on two TUs, with `+0x20 = 0x1c01` occurring in **both** arms | w-mark named this in its §9 item 6 and did not test it; this is the test, in the direction that could have failed | §4.3 |
| **T-c** | **The KIND-1 `.gl` record header, decoded** — `0x10b9b945`: `<tag><byte→+0x4d><varU tok→+0x28><byte→+0x31><name><byte discarded><byte→+0x37 hi><byte→sclass><i32c→+0x1c><varU→+0x20>[<varU→+0x0c> iff +0x20 & 0x200]` | round-trips **1.00000** on 2 744 921 records with a top-8 value concentration of **0.81225**; nothing in `work/` could read `[owner+0x20]` before it | §1a |
| **T-d** | **c2's emit set is a joint least fixpoint over DATA and CODE symbols and cannot be a root set over functions** — but it does **not** need to be *ordered* for this channel: `0x10b98e26` has exactly one caller chain and runs before the compile loop, reading only stream-written fields | separates w-mark's R-d (the compile loop *is* a worklist, confirmed at `0x10b7f1e5`) from the shape requirement, which comes from the owner predicate and not from ordering | §2, §5 |
| **T-e** | **CORRECTS w-refs' `Mark`/reader pair: `[head+0x14]` and `[head+0xc]` are two different lists**, and `0x10b27f3c` (at `0x10b7f0b5`, before the initializer walk) is the pass that resolves one into the other — keeping an edge only for tag-`0x0E` function targets with a non-zero use count and a token `>= 0x20`. **w-refs' `∩ U` and zero-use drop are that pass, not modelling choices** | disassembly, `0x10b9bffb` vs `0x10b27715`, plus `0x10b27ec7`'s "no head object, no edge" | §1c |
| **T-f** | **CORRECTS w-mark §1d: `0x10b98c0f` has three callers, and `0x10b9ac38` is not one of them** (`0x10b9ac38` → `0x10b98f0a` → `0x10b98fa6`); and `0x10b98f0a` is an unrecorded near-clone of the initializer walk over `[module+0x28]`, carrying SKIP 1 and SKIP 3 but not SKIP 2 | an `E8`/`E9`-plus-absolute scan, not a reading | §1c |
| **T-g** | **CORRECTS w-refs' `refs.head`: the `+0x0c` token is gated on `+0x20 & 0x200`** (`0x10b9be6b`, `0x10b9ba5f`) and is read unconditionally there | harmless in place — the `0x80 <LE32>` / `.ex`-offset gate catches a miss — and recorded anyway | §1a |
| **T-h** | **Filtering w-mark's roots by the skips makes them LESS enriched than chance** — soundness 0.14086 (**1.22×** the base rate `\|E\|/\|U\|` = 0.11577) becomes 0.09512 (**0.82×**), and the `∖P_RGL` coincidence ratio falls 4.00× → 2.69× | the quantitative form of T-a, in w-mark's own calibration shape | §3.1 |

---

## 12. Reproducing every number here

```sh
# 0. the binary reads (no corpus needed)
work/w-skip/dis.sh 0x10b98e26 260     # the walk and its three skips
work/w-skip/dis.sh 0x10b98b00 280     # WalkInit, W1/W2 and the mark rule
work/w-skip/dis.sh 0x10b98c0f 200     # RecurseSym (S5)
work/w-skip/dis.sh 0x10b9b945 200     # the KIND-1 record header
work/w-skip/dis.sh 0x10b27f3c 180     # +0x14 -> +0xc, the edge relation's pass
python3 work/w-mark/xrefs.py 0x10b98e26 0x10b34113 0x10b98c0f

# 1. the owner-side fields, on one captured TU dir (no truth)
python3 work/w-skip/glowner.py <ildir>

# 2. the headline scan (reads w-emit's cached gl/ex/truth + w-mark's in; runs NO c2)
python3 work/w-skip/scan.py  <main-repo>/work/w-emit/il \
        <main-repo>/.claude/worktrees/w-mark/work/w-mark/in \
        <main-repo>/work/w-emit/truth work/emitpred/magnitude/truthlist.txt \
        work/w-skip/scan.jsonl 20
python3 work/w-skip/score.py work/w-skip/scan.jsonl      # -> score.txt

# 3. KA-E — RUNS real c2.dll under wibo, on non-quarantined TUs
export C2RS_DC3=<dc3-tree> C2RS_WIBO=<wibo>
python3 work/w-skip/probe_f20.py    src/system/net/HttpReq.cpp
python3 work/w-skip/mutate_skip.py  src/system/rndobj/EventTrigger.cpp 3
python3 work/w-skip/mutate_gate.py  src/system/net/HttpReq.cpp 1
python3 work/w-skip/mutate_owner.py src/system/net/HttpReq.cpp 5
python3 work/w-skip/mutate_owner.py src/system/utl/PoolAlloc.cpp 5

# 4. KA-F2 — re-capture 8 TUs' `in` and byte-compare against w-mark's cache
python3 work/w-mark/capture_in.py $PWD/work/w-skip/kaf2 work/w-skip/kaf2_tus.txt 4
```

All scripts are **stdlib-only** and read-only against the corpus; the mutation
scripts write only inside `work/w-skip/mut/` and restore the bundle between runs.
`work/` is gitignored; the scripts and the text outputs are force-added as
records, and no IL, obj or `_CL_*` artifact is committed.
