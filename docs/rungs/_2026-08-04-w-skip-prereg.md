# w-skip — PRE-REGISTRATION

    Lane:    w-skip, 2026-08-04, worktree `wt-w-skip` off master `e57e641`
    Question: do `0x10b98e26`'s OWNER SKIPS — the three w-mark decoded and
              refused to use — recover w-refs' precision while holding w-mark's
              recall?
    Committed BEFORE any measurement against truth.  Scored in the findings doc.

---

## 0. Provenance, fixed before the first number

| | |
|---|---|
| c2-rs branch | `wt-w-skip`, based on master **`e57e641`** (the merge of `wt-w-mark`) |
| **dc3-decomp HEAD BEFORE** | **`940d07dcb0960964ad61aa5f025658f993eb46b2`** (`work/w-skip/prov_before.txt`) |
| wibo | **`1.0.1-23-g4a9dd6f`**, checked at lane start, **not stale** (`work/w-skip/wibo.txt`) |
| c2.dll | `compilers/X360/16.00.11886.00/c2.dll`, sha256 `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`, image base `0x10b00000`, `.text` VA `0x1000` @ file `0x400` |
| `gl` / `ex` / truth | **w-emit's cache, unchanged** — 876 IL, 1700 truth files, 850 graded TUs from `work/emitpred/magnitude/truthlist.txt` |
| `in` | **w-mark's capture, unchanged and re-verified** — 876 TUs, same dc3 rev, same `cl /Bd /d2nop` recipe. A re-capture of a random sample must reproduce it **byte-identically** (control KA-F2) |
| flags | the unmodified workload line, `work/dc3-workload/flags.txt` |
| scratch | `work/w-skip/` (gitignored); scripts and text outputs force-added, no IL or obj committed |

**The 21-TU quarantine of `_2026-08-02-w-emitpred-prereg.md` is in force and will
be honoured. w-emitpred's one-shot Part-1 gate is UNSPENT and this lane will not
spend it** — §8.

---

## 1. THE INCUMBENTS — registered as a table, not as a threshold

Every number below is graded against **`E`, the real emitted set** (COMDAT
leaders of code sections in the pipeline obj), on the **same 850 TUs**.
`Rfloor` is a floor, not a target (w-refs §4); it is reported for comparability
with w-roots and it is **not** a decline key.

| | **w-refs `RGL`** (best F1) | **w-mark `RGL+INIT`** (best recall) |
|---|---:|---:|
| \|P\| | 129 604 | 613 532 |
| **precision** | **1.00000** | 0.27289 |
| **recall** | 0.74307 | **0.95991** |
| **micro-F1** | **0.85260** | 0.42496 |
| per-TU exact `P == E` | **132 / 850** | 34 / 850 |
| soundness `\|I∩E\|/\|I\|` | — | 0.14086 |
| root-floor coverage | 0.18796 | 0.86926 |

**Which incumbent on which axis.** This lane tries to **beat w-mark on
precision** while **holding w-mark's recall**, and its overall claim is measured
against **w-refs' F1 = 0.85260**, which is the number to beat for a *shippable*
predicate. Both are named in every table.

**What a WASH looks like.** w-refs registered a ±2.0 pp decision band and
declined into it honestly. This lane keeps the same band:

> **F1 within ±2.0 pp of `max(0.85260, 0.42496)` = 0.85260 is a WASH.**
> Below 0.83260 is a **REGRESSION against the best incumbent**; above 0.87260 is
> an improvement. A precision gain that does not carry F1 past 0.87260 is
> reported as *"precision recovered, F1 not"*, never as a win.

---

## 2. THE DECODE — re-verified by address with `objdump`, not inherited

`work/w-skip/dis.sh <va> [n]` reproduces every line. I re-read all of
`0x10b98e26`, `0x10b98b00`, `0x10b98c0f`, `0x10b276e4`, `0x10b9bdcf`,
`0x10b9b945`, `0x10b9bf99`, `0x10b27f3c`, `0x10b27ec7` and the p2 driver
`0x10b7f022` from the bytes. Two of w-mark's readings are **corrected** and one
is **confirmed** — §3, §4.

### 2a. The record header, both kinds, so the skip fields are readable at all

The tag dispatch is `0x10b9b922` (GetByte) → `0x10b9b937` index table at
`0x10b9c615` → `0x10b9b93e` jump table at `0x10b9c5d5`; `0x10b98521`
(`mov BYTE PTR [esi+0x30],bl` in the allocator `0x10b984c3`) shows `[sym+0x30]`
is the **tag** unless an arm overwrites it.

    tag 0x04 / 0x0e / 0x10 -> 0x10b9bdcf    KIND 4   (0x10b9bdfb writes 4)
    tag 0x01 / 0x02 / 0x1a -> 0x10b9b945    KIND 1   (0x10b9b95f writes 1)
    tag 0x09               -> 0x10b9c212    KIND 9   the TYPE record

**KIND 4** (w-refs' `refs.head`, unchanged):

    <tag> <varU tok -> +0x28> <byte -> +0x31> <name>
          <byte -> +0x37 sclass, 0x10b9be0e> <i32c -> +0x40>
          <varU -> +0x20>                      0x10b9be63   THE FLAG WORD
          [ if (+0x20 & 0x200): <varU tok> -> +0x0c ]    0x10b9be6b

**KIND 1** — decoded by this lane; the 0xa0-byte template is memset at
`0x10b9b945` from `[ebp-0x6c]`, so `[ebp-0x1f]`=`+0x4d`, `[ebp-0x3c]`=`+0x30`,
`[ebp-0x44]`=`+0x28`, `[ebp-0x4c]`=`+0x20`:

    <tag> <byte -> +0x4d, 0x10b9b957 saved / 0x10b9b9d4 stored>
          <varU tok -> +0x28> <byte -> +0x31> <name>
          <byte, READ AND DISCARDED at 0x10b9b9cc>
          <byte -> +0x37 bits 21..23, 0x10b9b9d7>
          <byte -> +0x37 bits 5..8 sclass, 0x10b9b9ee>
          <i32c -> +0x1c>
          <varU -> +0x20>                      0x10b9ba0d   THE FLAG WORD
          [ if (+0x20 & 0x200): <varU tok> -> +0x0c ]    0x10b9ba5f

**KIND 9** (the type): `<0x09> <varU tok -> +0x28> <NUL-terminated string>
<byte -> +0x4d>` (`0x10b9c221`, `0x10b9c237`, `0x10b9c23c`, `0x10b9c247`).
It has **no separator byte before its string**, so a separator-anchored scan
cannot see it; this lane finds it by searching for `09 <enc(tok)>` for exactly
the tokens the owners ask about, **fail-closed** (0 or >1 hits → unknown).

### 2b. The three skips, by address, with what each tests

`0x10b98e26` is the initializer walk. Its loop body, transcribed:

    for (rec = *(0x10c67db4); rec; rec = [rec+8]):
      G0   0x10b98e4a  if ([rec+0x20] & 1) continue;
                       owner = Resolve([rec+0x10], curmodule);   0x10b98e67
                       if (!owner) continue;
                       if (kind in (1,4) && [owner+0x37]&0x400)
                            owner = Redirect(owner);             0x10b98e8e
           0x10b98e98  if (!([owner+0x32] & 4)) {
      S1   0x10b98e9f     if (([owner+0x20] & 0x60) == 0x20) continue;   <<< SKIP 1
                          [owner+0x32] |= 4;
                          r = WalkInit(owner);                   0x10b98eae
                          [owner+0x32] &= ~4;
                          if (r == 0) { 0x10b98de4(rec); continue; }
                       }
      S2   0x10b98ecd  if ([[owner+0xc] + 0x4d] == 0x1d) continue;      <<< SKIP 2
      S3   0x10b98ed9  if ([owner+0x30]==1 && ([owner+0x20] & 0x4000))
                            continue;                                    <<< SKIP 3
           0x10b98ee7  RecurseSym(owner, [owner+8]);            (S5, 0x10b98c0f)

| # | site | field tested | what it means structurally | static, or codegen state? |
|---|---|---|---|---|
| **SKIP 1** | `0x10b98e9f` | `([owner+0x20] & 0x60) == 0x20` | the owner's `.gl` flag word, `varU` at `0x10b9ba0d`. Skips the record **entirely** — neither the `0x10b98b00` walk nor the `0x10b98c0f` pass runs | **STATIC.** Read verbatim from the `.gl` byte stream; nothing writes `+0x20` after the reader |
| **SKIP 2** | `0x10b98ecd` (and `0x10b98ba8` inside the walk) | `[[owner+0xc] + 0x4d] == 0x1d` | the owner's **type**: `+0x0c` is a symbol resolved from a `varU` token when `+0x20 & 0x200`, else the module default `[0x10c472e8 + 0x2cc]`; `+0x4d` is that record's kind byte. Skips only the `0x10b98c0f` pass. **Inside `0x10b98b00` the same test is not a skip but an early-out: it Marks that one target and returns 0**, aborting the rest of the node list (`0x10b98ba8` → `0x10b98c04` → `0x10b98c08`) | **STATIC.** Both `+0x0c` and `+0x4d` come from the stream |
| **SKIP 3** | `0x10b98ed9` | `[owner+0x30]==1 && ([owner+0x20] & 0x4000)` | kind is the record **tag**; the bit is in the same flag word. Skips only the `0x10b98c0f` pass | **STATIC** |

### 2c. TWO GATES w-mark DID NOT NAME, and they are the operative filter

`WalkInit` = `0x10b98b00` opens with two tests that gate the whole node walk and
that no lane has recorded:

| # | site | test | effect |
|---|---|---|---|
| **W1** | `0x10b98b09` | `[owner+0x30] != 1` → return 1 | only **kind-1** (tag-0x01/0x02) owners' initializers are walked at all |
| **W2** | `0x10b98b14` | `!([owner+0x20] & 0x480)` → return 1 | the owner must carry `0x80` or `0x400` in the same flag word |

`0x10b98c0f`'s kind-1 arm repeats W2 verbatim at `0x10b98c89`. **Both are static
reads of the same `+0x20` word.** A pilot on three TUs, run before this prereg
and disclosed here (§7.2), says W2 is where the filtering actually happens and
SKIP 1 never fires on this workload — which is why §5 registers what it does.

### 2d. The mark rule, so nothing is asserted

`0x10b98b00`, node loop (`0x10b98b4f` .. `0x10b98bf2`):

    for (n = [[owner+0x33]+4]; n; n = [n+4]):
        if ([n] != 2 && [n] != 0x14) continue;
        t = Resolve([n+8], [owner+8]);      if (!t) return 0;     0x10b98b6e
        if (kind(t) in (1,4) && [t+0x37]&0x400) t = Redirect(t);
        if (kind(t)==4 && [t+0x37]&0x200000 && !([t+0x4c]&2)
              && [[owner+0xc]+0x4d]==0x1d) { Mark(t); return 0; }  0x10b98c08
        if ([t+0x32] & 4) continue;
        [t+0x32] |= 4;  r = WalkInit(t);  [t+0x32] &= ~4;
        if (r == 0) return 0;
        0x10be70cc(t);
        if (kind(t)==4 && [t+0x37]&0x200000 && !([t+0x37]&0x400)) Mark(t);  0x10b98be8

`0x10b98c0f` (S5) marks a kind-4 owner directly at `0x10b98c7f` and, for a kind-1
owner passing W2, recurses `RecurseSym` into every resolved node target.

---

## 3. THE WORKLIST QUESTION — answered from the disassembly, BEFORE fitting

**w-mark's R-d is CONFIRMED**: `0x10b7f1e5: je 0x10b7f15f` re-reads
`ds:0x10c4630c` after every compiled function, so the compile loop is a worklist
run to a fixpoint during codegen (and the restart is itself gated on the mode
flag `ds:0x10c462c4`, which is 0 for an ordinary compile).

**And it does not apply to this channel.** `work/w-mark/xrefs.py` (a full
`E8`/`E9` plus absolute-address scan) finds:

* `0x10b98e26` has **exactly one caller**: `0x10b3413d`, inside `0x10b34113`;
* `0x10b34113` has **exactly one caller**: `0x10b7f0d2` — **before** the compile
  loop at `0x10b7f15f`;

so the initializer walk runs **once per module, before any function is
compiled**, and is never re-entered by the worklist.

**Therefore the model does NOT need to be a worklist for this channel.** Every
field the three skips and W1/W2 read — `+0x20`, `+0x30`, `+0x0c`, `+0x4d` — is
written by the `.gl` reader and by nothing else. The one field that *is* codegen
state, `[t+0x4c] & 2` (the "compiled" bit, set at `0x10b7f199`), is read by the
SKIP-2 early-out at `0x10b98b9f` at a time when **nothing has been compiled yet**,
so its value there is the `.gl` stream's DONE bit. `[owner+0x32] & 4` is the
walk's own DFS re-entrancy marker, set and cleared inside the walk.

**The static roots-plus-closure shape therefore survives for the initializer
channel.** It does *not* survive for the other two live channels, and this lane
does not model either: `0x10b3389b` (`dag.c`, reached from compile-one-function)
adds *edges* during codegen, and `0x10b9aa26` (the by-name intern) adds *roots*
during codegen. **If the measurement below fails, "the model has the wrong
shape" is not available as an excuse for this channel** — that is why this
question is answered here and not afterwards.

---

## 4. Corrections and confirmations of landed claims, registered before scoring

* **CONFIRMS w-refs' edge relation, and names the mechanism it did not.** The
  `.gl` reader pushes `(token, refcount)` nodes onto `[head+0x14]`
  (`0x10b9bffb`/`0x10b9c000`), but `Mark` walks `[head+0xc]`
  (`0x10b27715`) whose nodes hold a *pointer* (`[[n+4]+4]` is the target).
  These are two lists. `0x10b27f3c` — called at `0x10b7f0b5`, **before**
  `0x10b98e26` — is the pass that resolves `+0x14` into `+0xc`, and it keeps an
  edge only when the target is `[+0x30]==4 && [+0x37]&0x200000` (a tag-`0x0E`
  function record, i.e. a member of `U`) with a non-zero use count
  (`0x10b27fd1`) and a token `>= 0x20` (`0x10b27f8b`), via `0x10b27ec7`, which
  itself drops the edge when the target has no head object. **So w-refs' `∩ U`
  restriction and its zero-use drop are not conveniences — they are the pass.**
  w-refs' pseudocode prints `ecx[0xc]` next to a reader that writes `+0x14`
  without noting they differ; this lane records why they agree.
* **CORRECTS w-mark §1d S5's reach.** `0x10b98c0f`'s callers are **three**:
  `0x10b98d2e` (its own recursion), `0x10b98ee7` (the driver walk) and
  `0x10b98fa6`. w-mark lists "`0x10b9ac38`"; that is two frames away
  (`0x10b9ac38` → `0x10b98f0a` → `0x10b98fa6`).
* **NAMES a near-clone nobody has recorded.** `0x10b98f0a` is a second walk of
  the same shape over a *different* list (`[module+0x28]`), carrying SKIP 1 and
  SKIP 3 but **not** SKIP 2, reached from `0x10b9ac38` ← `0x10b34026`. This lane
  does **not** model it (§9).

---

## 5. THE REGISTERED NUMBERS — point and interval SEPARATELY

**The decline clause keys on the POINT for M3 and on the INTERVAL for nothing.**
Stated once, so it cannot be mis-quoted the way w-roots' 0.55 floor was: below,
*point* is my belief and *interval* is my uncertainty; a value inside the
interval but far from the point is a **miss on the point** and is reported as
one.

The model under test, `P_SKIP`, changes exactly one variable against w-mark:
the root set `I` becomes `I_skip`, the marks a faithful replay of §2b/§2c/§2d
produces. Edges, seed, truth reader, name binding and closure are w-refs'/
w-roots' as landed.

| # | quantity | **point** | **interval** |
|---|---|---:|---|
| **M1** | **precision** of `P_SKIP` vs `E` | **0.35** | [0.27, 0.80] |
| **M2** | **recall** of `P_SKIP` vs `E` | **0.93** | [0.78, 0.96] |
| **M3** | **micro-F1** of `P_SKIP` vs `E` | **0.51** | [0.40, 0.86] |
| **M4** | per-TU exact `P == E` | **0.06** (51/850) | [0.02, 0.30] |
| **M5** | soundness `\|I_skip ∩ E\| / \|I_skip\|` | **0.20** | [0.13, 0.70] |
| **M6** | `\|I_skip\|` | **185 000** | [60 000, 246 000] |
| **M7** | share of w-mark's `I` that survives the filter, `\|I_skip\|/\|I\|` | **0.75** | [0.25, 1.00] |
| **M8** | fraction of `in` records whose owner passes **W1∧W2** | **0.60** | [0.35, 0.85] |
| **M9** | number of `in` records SKIP 1 fires on, corpus-wide | **0** | [0, 20 000] |
| **M10** | number of `in` records SKIP 3 fires on, corpus-wide | **9 000** | [500, 60 000] |
| **M11** | fraction of owners whose type kind is decodable (`+0x0c` resolvable) | **0.90** | [0.40, 1.00] |
| **M12** | root-floor coverage `(Seed ∪ I_skip) ∩ Rfloor / \|Rfloor\|` | **0.80** | [0.30, 0.90] |
| **M13** | **MUTATION, positive arm** — set SKIP 1 on a walked owner: its initializer's functions lose their COMDATs | **5/5** | pass ≥ 3/5 |
| **M14** | **MUTATION, discriminating control** — set `0x60` (both bits) on the *same* owner at the *same* byte: **nothing is lost** | **5/5** | pass ≥ 4/5 |

**The single outcome I most expect to be wrong about** is **M1**. I have
registered it at 0.35 — barely above w-mark's 0.27289 — because the decode says
the skips remove *owners*, not *targets*, and my pilot (§7.2) says they remove
about a third of the nodes. If M1 lands near 1.0 I was wrong about the shape of
the answer, and I will say so first.

**The declared bias.** I am registering M1/M3 **low**, i.e. I am predicting this
lane's own hypothesis fails. That is the direction that costs me least, so the
honest counterweight is M13/M14: I have registered the *causal* claim at
**5/5 and 5/5**, where a single red control refutes the decode this whole page
rests on.

### 5.1 DECLINE CLAUSES — literal, and named in advance

1. **If M3 < 0.87260 (the +2.0 pp band over w-refs' 0.85260): DECLINE.**
   The model half of the findings is published as a **measurement of the skips'
   effect, not as a shippable predicate**, the first line says so, and **I stop
   looking for a further channel.** Everything I did not decode is *named* in
   the "not measured" section and left undecoded, as w-refs and w-mark did.
2. **If M5 < 0.50: publish a coincidence calibration** in w-mark's exact shape
   (its increment was 4.00× the uniform-coincidence expectation against
   w-emit's disqualified 1.07×), or the recall row is not a claim.
3. **If M13 or M14 fails its pass mark, that is reported FIRST**, above every
   observational number, and §2b is marked "decode not causally confirmed".
   M14 is the arm that can go red in the most likely failure mode — if my
   reading of `& 0x60` is wrong and *any* write to that byte breaks emission,
   M14 goes red and M13 is worthless. **A red M14 invalidates M13.**
4. **No instrument tuning after truth.** The `.gl` header reader, the `in`
   grammar, the type-kind lookup and every gate above are fixed by the
   disassembly and by gates that read **no c2 output**. After the first number
   against `E` I change nothing. If I must change something, the change and both
   numbers are published.
5. **Nothing ships.** No `crates/` change, no fixture, no widening, no
   `DISCLOSURE.md` row. `PortC2` still returns `NotImplemented` outside its
   class.
6. **`Rfloor` is a floor, not a target** (w-refs §4). M12 is reported for
   comparability with w-roots and w-mark and is **not** a decline key.
7. **If §2b or §2c is refuted by the data, that is reported before any headline.**

### 5.2 Registered before the numbers exist

* **TU match stays 8.** Nothing ships, so it must.
* **`census/gate disagreement` stays 0.**
* **A high recall is not a shippable predicate.** If M1 stays low, a fail-closed
  `Emit/Skip/Unknown` on this is wrong about most names it claims.
* **Order is untouched.** A right set in the wrong order is still a mismatch.

---

## 6. KNOWN-ANSWER CONTROLS — including a POSITIVE check

| # | control | registered pass |
|---|---|---|
| **KA-A** | reproduce **both** incumbents exactly on the same 850 TUs: `\|U\|` 1 506 586, `\|E\|` 174 417, `\|E∩U\|` 173 907, `\|Seed\|` 14 662, `\|P_RGL\|` 129 604, precision 1.00000, recall 0.74307, F1 0.85260, per-TU exact 132; and w-mark's `\|P_INIT\|` 613 532 / 0.27289 / 0.95991 / 0.42496 / 34 | all, to the digit |
| **KA-B** | the `in` terminus gate is unchanged from w-mark: **876/876 clean, 1 885 700 `02` nodes** | exact |
| **KA-C** | the `.gl` owner-header reader: the `+0x20` flag word must **round-trip** to the same bytes it was read from, and its value histogram must be **concentrated** (top 8 values ≥ 0.80 of kind-1 records) rather than noise | round-trip ≥ 0.999, concentration ≥ 0.80 |
| **KA-D** | **coverage**: fraction of `in`-record owner tokens that bind to a decoded `.gl` record | ≥ 0.50, reported as a count |
| **KA-E** | **MUTATION against the SOLE JUDGE** — M13 and M14 | §5, M13 ≥ 3/5 and M14 ≥ 4/5 |
| **KA-F1** | dc3 HEAD before/after; wibo version | no mid-run move |
| **KA-F2** | **re-capture 8 random non-quarantined TUs' `in` and byte-compare against w-mark's cache** | 8/8 identical |
| **KA-G** | incumbent gate on the unmodified tree: `cargo test --workspace --release` **FAILED count and target count** (re-measured, not transcribed — w-mark's tables quote a stale 687), `scripts/gate.sh`, `c2rs selftest`, `cargo build --release` warnings, the `c2rs gap` block | 0 FAILED, 25 targets, 12/12, 0 mismatch, TU match 8 |
| **KA-POS** | **POSITIVE CHECK — the run must have GRADED something.** `P_SKIP` and `P_INIT` must **disagree**, printed as a count of discriminating names, and `P_SKIP` and `P_RGL` likewise. **A run reporting zero discriminating names is a FAILURE, not a pass** | both > 0, both printed |

---

## 7. Disclosures

### 7.1 What is reused, and why that is not a hidden variable

`gl`, `ex` and truth are w-emit's cache; `in` is w-mark's. KA-A grades the reuse
by reproducing both incumbents to the digit, and KA-F2 grades the `in` cache by
re-capturing a sample and comparing bytes. Reuse that is *checked* is not an
assumption.

### 7.2 THE PILOT — three TUs, run BEFORE this prereg, disclosed in full

I ran `work/w-skip/glowner.py` on `PoolAlloc.cpp`, `HttpReq.cpp` and `App.cpp`
and joined it to their `in` records **before** writing §5, and no truth file was
opened. What it showed, and what it moved:

| | PoolAlloc | HttpReq | App |
|---|---:|---:|---:|
| `in` records | 241 | 176 | 3 628 |
| owner token binds to a `.gl` record | 159 | 133 | 3 044 |
| of those, **all** are kind 1 | 159 | 133 | 3 044 |
| **SKIP 1 fires** | **0** | **0** | **0** |
| SKIP 3 fires | 2 | 2 | 96 |
| owner has a `+0x0c` type token (`+0x20 & 0x200`) | 14 | 9 | 342 |
| **W1∧W2 pass (walk-enabled)** | 112 | 93 | 1 840 |
| `02` nodes total / under walk-enabled owners | 332 / 241 | 286 / 214 | 8 456 / 5 379 |

This is why M9 is registered at a point of **0** and why M1 is registered at
**0.35** rather than near 1.0. **Registering high after seeing a pilot that says
low would be the dishonest move; registering low is what the pilot supports, and
I am saying out loud that it makes this lane's own hypothesis likely to fail.**
The tag-0x07 flags byte in the `in` stream is **0 on every record** of all three,
so G0 (`[rec+0x20] & 1`) is not that byte and is registered nowhere.

### 7.3 The stratification `#152` forces

w-mark showed the channel covers free functions **99.60 %** and ordinary virtuals
**96.25 %** but `??_G`/`??_E` deleting destructors only **12.67 %**, taking them
from 10.2 % to **60.9 %** of the residual — because c2 *synthesizes* those and no
`02` node can name them (`#152`). **Every residual table in the findings will be
stratified by w-roots' `boundary2.kind` classifier with `??_G`/`??_E` broken
out**, and the headline precision/recall will additionally be reported with that
class **excluded from `E`**, so it cannot silently dominate either direction.

---

## 8. The one-shot Part-1 gate — NOT to be spent

The 21-TU quarantine is intact and w-emitpred's Part-1 gate is runnable exactly
once. **This lane will not spend it**, and that is registered here before any
number exists. The reason is about the object, not about convenience: this
lane's model has **zero fitted parameters** — every field, mask and constant is
transcribed from a named instruction — so a held-out set cannot tell it anything
an in-sample set cannot, and the lane is comparative by construction
(`P_SKIP` against `P_INIT` and `P_RGL` on the *same* 850 TUs).

**The registered reversal condition**: if, after seeing the numbers, I introduce
*any* parameter that was chosen by looking at `E` — a bit mask, a threshold, a
class exclusion — then the model is no longer parameter-free, and I must
(a) say so in the first line, (b) **not** spend the gate anyway, and (c) hand the
gate to the coordinator with the model described. I will check this honestly and
report the check.

---

## 9. What this lane will NOT measure — named in advance

1. **`0x10b98f0a`**, the near-clone over `[module+0x28]` (§4).
2. **`0x10b98de4`**, called when the walk returns 0.
3. **G0** — `[rec+0x20] & 1`. The stream byte it might come from is 0 everywhere
   (§7.2); where it is actually written is undecoded.
4. **S1 (`0x10b28ca3`, the COFF writer) and S6 (`0x10b9aa26`, the by-name
   intern)** — w-mark's two other channels, untouched.
5. **S2 (`0x10b3389b`, `dag.c`)** — the codegen-time edge channel. §3 says the
   worklist matters *there*; this lane does not model it.
6. **`db` and `sy`.** Still uncaptured. w-mark's three necessity survivors say a
   second channel reaches the EH copy-constructor family.
7. **Node kind `0x14`.** `[n]==2 || [n]==0x14` in memory; only the stream's
   `0x02` byte kind is decoded.
8. **`0x10be7006`** (`0x10be70cc`/`0x10be70d4`), called between the recursion and
   the Mark.
9. **`-optref`** (`0x10b27b7f`), the only path that clears `0x20`. Absent here.
10. **Order.** A right set in the wrong order is still a mismatch.
11. **The 21 quarantined TUs.** Untouched.

---

## 10. Reproducing it

```sh
work/w-skip/dis.sh 0x10b98e26 260     # the walk and its three skips
work/w-skip/dis.sh 0x10b98b00 280     # WalkInit, W1/W2 and the mark rule
work/w-skip/dis.sh 0x10b98c0f 200     # RecurseSym (S5)
work/w-skip/dis.sh 0x10b9b945 200     # the KIND-1 record header
work/w-skip/dis.sh 0x10b27f3c 180     # +0x14 -> +0xc, the edge relation's pass
python3 work/w-skip/glowner.py <ildir>            # the owner-side fields
python3 work/w-skip/scan.py  <il> <in> <truth> <tulist> <out.jsonl> [jobs]
python3 work/w-skip/score.py <out.jsonl>
C2RS_DC3=<dc3> C2RS_WIBO=<wibo> python3 work/w-skip/mutate_skip.py <src> 5
```

All stdlib-only, read-only against the corpus; the mutation writes only inside
`work/w-skip/mut/` and restores between runs. `work/` is gitignored; scripts and
text outputs are force-added, no IL or obj is committed.
