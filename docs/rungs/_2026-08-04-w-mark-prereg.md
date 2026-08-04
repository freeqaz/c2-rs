# w-mark — PRE-REGISTRATION

    Lane:    w-mark, 2026-08-04, worktree `wt-w-mark` off master `451f1bd`
             (the merge of `wt-w-refs`)
    Question: what marks a symbol at each of the OTHER `Mark` call sites, and do
             those roots close the recall gap?
    Ships:   NOTHING under `crates/`.
    Frozen:  this file is committed BEFORE the first measurement against truth.
             Every number in §3 is a belief, not a result.

---

## 0. Provenance, fixed before anything is measured

| | |
|---|---|
| c2-rs branch | `wt-w-mark`, based on master **`451f1bd`** |
| c2.dll read | `compilers/X360/16.00.11886.00/c2.dll`, sha256 `c80981c0…a66258`, image base `0x10b00000`, `.text` VA `0x1000` @ file `0x400` |
| **dc3-decomp HEAD BEFORE** | **`940d07dcb0960964ad61aa5f025658f993eb46b2`** (`work/w-mark/prov_before.txt`) — the same rev w-emit, w-roots and w-refs measured at |
| **wibo** | **`1.0.1-23-g4a9dd6f`**, checked at lane start, **not stale**. (`.claude/worktrees/wibo` is a **symlink** to the sibling checkout, so a worktree-relative resolution reaches the same binary — verified, not assumed.) |
| `gl` / `ex` / truth | **reused from w-emit unchanged** (`work/w-emit/{il,truth}` in the main repo) |
| **`in`** | **captured by this lane** (`work/w-mark/capture_in.py`), same `cl /Bd /d2nop` invocation w-emit used, which aborts in `p2` with `C1007` and so produces **no c2 output** — quarantine-safe |
| the `gl` join | the re-capture reproduces w-emit's cached `gl` **byte-identically** (`cmp` on `src/system/utl/PoolAlloc.cpp`, run before this file was written), which is why the token spellings agree across the two caches |
| flags | the unmodified workload line, `work/dc3-workload/flags.txt` |
| denominators | **850** graded TUs, `work/emitpred/magnitude/truthlist.txt`; **174 417** emitted names; **1 506 586** gate-clean records; **14 662** seeds |

**The 21-TU quarantine of `_2026-08-02-w-emitpred-prereg.md` stays in force and
will be honoured. w-emitpred's one-shot Part-1 gate will NOT be spent** — §5.

---

## 1. The decode, done BEFORE this file and stated so it can be scored

Everything in this section was read out of `c2.dll` with `objdump` before any
prediction below was written. It is **not** a prediction; it is the instrument.
`work/w-mark/dis.sh <va> <n>` reproduces each address.

### 1a. There are SIX external `Mark` call sites, not seven

`work/w-mark/xrefs.py 0x10b276e4` scans **every** `E8`/`E9 rel32` in `.text` and
**every** little-endian occurrence of the absolute address in the whole image.
It finds seven `call` sites and **zero** absolute references — so `Mark` is
reached by direct call only, and the enumeration is complete for this image.

One of the seven, **`10b27731`, is `Mark`'s own recursion** over the reference
list (it is inside `10b276e4 .. 10b2773e`). **`_2026-08-04-w-refs-findings.md`
§1b counts it as one of "seven other `Mark` call sites"; it is not other.** The
external count is **6**, in **4** enclosing functions.

### 1b. Two gates w-refs' pseudocode omits, one of which is a mode switch

    Mark(ecx = sym, edx = force):                       ; 10b276e4
      if (sym[0x4c] & 0x20) return;                     ; 10b276ea  already marked
      if (ds:0x10c462c4 && !force) return;              ; 10b276ee
      sym[0x4c] |= 0x20;                                ; 10b276fb  THE SEED BIT
      if (!ds:0x10c3cf68) return;                       ; 10b27701  <-- OMITTED
      ecx = sym[0x80]; if (!ecx) return;                ; 10b2770a
      for (node = ecx[0xc]; node; node = node[0]):      ; 10b27715
          tgt = node[4][4]
          if (tgt[0x37] & 0x400) continue;              ; 10b27720
          if (tgt[0x4c] & 0x20) continue;               ; 10b27729
          Mark(tgt, force)                              ; 10b27731

* **`ds:0x10c3cf68` is the head of the global list of reference-list objects**
  (`10b27dbd: mov [esi+0x80],eax ; mov ecx,ds:0x10c3cf68 ; mov [eax],ecx ; mov
  ds:0x10c3cf68,eax`), so it is non-null as soon as **one** symbol has a list.
  The gate is a cheap "did this IL carry any reference lists at all" and is
  **benign on this workload** (w-refs measured the list bit on 1 506 591 of
  1 506 595 records). Recorded because an omitted gate is a decode defect
  whether or not it fires.
* **`ds:0x10c462c4` is a MODE, and it is 0 for an ordinary compile.** The p2
  driver at `10b7f022` reads the `ex` and `sy` sub-streams **only** when it is
  zero (`10b7f026: cmp ds:0x10c462c4,edi ; jne`), and reads `in` only when it is
  zero (`10b7f2e0`). So on this workload every site's `force` argument is
  **inert**, and `force` is the flag that keeps a site alive in the other mode.

### 1c. The phase order, from the p2 driver `10b7f022`

    10b7f09d  call 10b97f98
    10b7f0a2  if (ds:0x10c3cf68) call 10b27f3c   -> 10b2773f, the prune/fixpoint
              else               call 10b9937a
    10b7f0d2  call 10b34113      -> 10b98e26, THE INITIALIZER WALK (S3/S4/S5)
    10b7f15f  the COMPILE LOOP:
                for (p = &ds:0x10c4630c; (s = *p); )
                    if ((s[0x4c] & 0x20) && !(s[0x4c] & 2)) { s[0x4c] |= 2;
                        unlink s; compile s;   ; 10b7f199 .. 10b7f1c5
                        goto 10b7f15f; }       ; 10b7f1e5 RESTART FROM THE HEAD
                    else p = &s[0x78];

**The compile loop restarts at the list head after every compiled function.** It
is a worklist run to a fixpoint, not a pass over a precomputed set: **codegen of
one function can mark another and that one is then compiled.** That is what
makes S2 (below) a live root channel and not a late no-op.

### 1d. The table — every external `Mark` call site

| # | site | enclosing fn / TU | reached from | trigger, transcribed | can it mark what reference-following cannot? |
|---|---|---|---|---|---|
| **S1** | `10b28ca3` | `10b28a9b` `coff.c` — the COFF symbol writer | itself (`10b28cb9`) and `10b29050` | kind-1 symbol whose COMDAT-selection field `(+0x37 >> 5) & 0xF == 2` takes `[sym+0x3f]` as a token, resolves it (`10b9860d`), sets `[tgt+0x32] |= 4`, and marks it when `[tgt+0x37] & 0x200000`. A second entry at `10b28b02` reaches the same code for a kind-4 symbol with `+0x37 & 0x400000`, taking `[sym+0x4c]` as the token | **YES in the symbol table, NO in the emit set** — it runs after the compile loop (`10b7f186`), so a mark here cannot cause codegen |
| **S2** | `10b3389b` | `10b33647` `dag.c` | `10b3421b` ← `10b7e032` ← `10b7e6af` = **compile-one-function** | walks the tuple/DAG operand chain (`[insn+0x2c]`, then `[insn+0x28]`); operand kind `[op+8]` 2/3 → `edi = *[[op+0x18]+8]`, kind 4 → `[op+0x18]` with `[edi+0x30]==3 && [op+4]==0x2a7`, kind 5/6 → `*[[op+0x24]+8]`. Marks when `[edi+0x30]==4`, `!(+0x37 & 0x400)`, `+0x37 & 0x200000`, `!(+0x4c & 0x20)` | **NO — this IS reference-following**, at codegen time, over the same operands w-emit's `.ex` 26-token proxy reads. It is the mechanism behind the propagation half both earlier lanes validated |
| **S3** | `10b98be8` | `10b98b00` `p2symtab.c` | `10b98e26` ← `10b34113` ← the driver, **before the compile loop** | walks the owner's **initializer node list** (`[sym+0x33]`, built by `10b9893b` from `[sym+0x28]`); a node with `[n]==2 \|\| [n]==0x14` resolves `[n+8]` as a token; recurses through data symbols and marks the target when it is a function | **YES — this is the address-take in a DATA INITIALIZER** |
| **S4** | `10b98c08` | `10b98b00` | same | the early-out arm of the same walk: target is a function, `!(+0x4c & 2)`, and `[[owner+0xc]+0x4d] == 0x1d` | **YES**, same channel |
| **S5** | `10b98c7f` | `10b98c0f` `p2symtab.c` | `10b98e26` (`10b98ee7`), and `10b9ac38` | the recursive form of the same walk; when the symbol reached **is itself** a function (`[+0x30]==4`, `+0x37 & 0x200000`, `!(+0x37 & 0x400)`) it is marked directly | **YES**, same channel |
| **S6** | `10b9aa26` | `10b9a897` `p2symtab.c` — intern-symbol-**by-name** | `10b9ae7e`/`10b9ae89`, called from **all of codegen** (`lower.c`, `code.c`, `cgintrin.c`, `mod.c`, `misc.c`, `globregs.c`, `ltcg.c`) | hash the name (`10b8a01b`) into the 128-bucket table at `0x10c67db8`; create kind-4 with `+0x37 \|= 0x87` if absent; then when `(+0x37 & 0x1e0) != 0x80` and `!(+0x4c & 2)`, **`Mark(sym, force=1)`** and `[+0x20] \|= 0x2000`. In the other mode the same arm first emits diagnostic `0x10c` (`10c1ef6f`) or ICEs at `p2symtab.c:5447` | **YES — a name c2 mints during lowering is not in the IL at all.** But a minted name normally has no body, and the compile loop skips a symbol whose body lookup (`10b7ef55`) returns 0 |

**S3/S4/S5 are one channel** — the initializer walk — and it is the only one of
the four that is both (a) unreachable by reference-following and (b) upstream of
the compile loop. **That is the channel this lane measures.**

### 1e. Why that channel is exactly the hole the last three lanes left

`10b98e26` fills its list from `ds:0x10c67db4`, which the driver loads from the
**`in` sub-stream** (`10b7f311: mov edx,0x10b13380 ("in") ; call 10b7e276`).
w-emit's capture **kept only `gl` and `ex` and deleted `in`**, so every number in
w-emit, w-roots and w-refs is blind to this channel by construction — w-refs
§9.2 names it as uncapturable from the cached corpus. And w-refs §3 proves the
`.gl` reference list is a **tag-`0x0E` field** (`cmp [ebp-0x78],0xe` at
`10b9bf46`), so a vftable is a *target* of that list and a dead end in it, and
**54.6 %** of what the list misses is referenced by nothing in `.gl` at all.

### 1f. The `in` grammar, and its known-answer gate

Decoded from the bytes with c2's own primitives and gated by **exact
consumption** (`work/w-mark/instream.py`):

    record := 0x07 <byte> <varU owner> <i32c 0> node*
            | 0x00        <varU owner> <i32c 0> node*      (the leading __C1_<build>)
    node   := 0x01 <i32c count> <i32c width> <value>       scalar run
            | 0x02 <varU token> <i32c addend> <i32c width> SYMBOL REFERENCE
            | 0x03 <i16c len> <len bytes>                  blob
            | 0x08 <i32c len>                              zero fill

A lone trailing `0x07` is the end-of-stream marker. On `src/system/utl/PoolAlloc.cpp`
the walk consumes all 6 434 bytes exactly and the records read as their own
proof: `??_R0?AVrange_error@stlpmtx_std@@@8 -> ??_7type_info@@6B@`, and
`_CT??_R0?AVrange_error@…??0range_error@…@Z268 -> [??_R0?AVrange_error@…,
??0range_error@stlpmtx_std@@QAA@ABV01@@Z]` — a catchable-type record naming its
own copy constructor, with the `268` in its decorated name reproduced as the
`sizeOrOffset` field. A wrong width would have produced garbage names.

---

## 2. What is measured

Per TU, joining w-emit's cached `gl`/`ex`/truth with this lane's `in`:

    U, E, Seed, RGL, P_RGL   exactly as `work/w-refs/scan.py` computes them
                             (the INCUMBENT, reproduced to the digit — KA-A)
    I  = { f in U : f is named by a 0x02 node anywhere in this TU's `in` }
    P_INIT = closure_RGL(Seed union I) intersect U

One variable changes: the root set. The edge relation, the truth reader, the
name binding and the closure operator are w-refs'/w-roots' as landed.

**The incumbent I must beat, named:** w-refs' `RGL` at **precision 1.00000,
recall 0.74307, micro-F1 0.85260, per-TU exact 132 / 850**, and w-roots' root
coverage **0.18762** (0.18796 recomputed over `RGL`) of a root floor of
**36 141**. A threshold with no baseline cannot tell an improvement from a
regression.

---

## 3. The frozen predictions

**Point estimate and interval are registered SEPARATELY. The decline clauses key
on the explicit thresholds in §4, never on an interval edge.**

| # | quantity | **point** | interval |
|---|---|---|---|
| **M1** | **recall** of `P_INIT` (incumbent **0.74307**) | **0.88** | [0.78, 0.96] |
| **M2** | **precision** of `P_INIT` (incumbent **1.00000**) | **0.97** | [0.90, 1.00] |
| **M3** | **micro-F1** of `P_INIT` (incumbent **0.85260**) | **0.925** | [0.85, 0.97] |
| **M4** | per-TU exact `P_INIT == E` (incumbent **132/850 = 0.15529**) | **0.30** | [0.10, 0.55] |
| **M5** | `\|I\|` — INIT roots that are functions in `U`, over 850 TUs | **45 000** | [15 000, 120 000] |
| **M6** | `\|I \ P_RGL\|` — names the channel adds that the incumbent misses | **25 000** | [5 000, 70 000] |
| **M7** | **soundness** `\|I ∩ E\| / \|I\|` — is an initializer-named function emitted? | **0.97** | [0.85, 1.00] |
| **M8** | **root coverage** `\|(Seed ∪ I) ∩ Rfloor\| / \|Rfloor\|`, `Rfloor` over `RGL` = 36 141 (w-roots measured **0.18796** with `Seed` alone) | **0.75** | [0.40, 0.98] |
| **M9** | **terminus gate** — TUs whose `in` stream is consumed to the last byte | **0.99** | [0.90, 1.00] |
| **M10** | vtable-slot share of the residual `E ∖ P_INIT` (w-refs measured **0.37262** for `E ∖ P_RGL`) | **0.10** | [0.00, 0.30] |
| **M11** | **KA-MUT, sufficiency** — retarget one `02` node's token to an unmarked, unemitted function of equal varU width and replay through real `c2.dll`: the retargeted function GAINS its COMDAT | **4/5** | ≥ **3/5** to pass |
| **M12** | **KA-MUT, necessity** — in the same replays the original target LOSES its COMDAT | **4/5** | ≥ **3/5** to pass |

**Declared bias.** I believe the initializer walk is the missing channel and I
have registered M1, M7 and M8 high so that being wrong costs me. M8 is the
sharpest: w-roots measured 0.188 and I am predicting **four times** that.

**The single outcome I most expect to be wrong about:** M2. Adding every
initializer-named function as a root ignores `10b98e26`'s own skip
(`([owner+0x20] & 0x60) == 0x20`) and F3's `[[owner+0xc]+0x4d] == 0x1d` arm, so
the root set is an over-approximation and precision should fall below 1.00000.
If it does not fall at all, the extraction is probably too small, not too good.

---

## 4. Decline clauses — literal, and honoured literally

1. **M9 < 0.95 ⇒ decline to quote M1–M8 at all.** Publish §1d's table, the gate
   as a count, and nothing that rides on the decode.
2. **M7 < 0.50 ⇒ the extraction is dominated by token coincidence.** Decline to
   quote M1/M3 as a model; publish them as an upper bound with a coincidence
   calibration in w-emit P-e's shape, and say so in the first line.
3. **F1 gain < +2.0 pp over 0.85260 ⇒ WASH.** Report it as a null in the first
   line, **do not go looking for a further channel**, and name every channel not
   decoded so absence cannot read as success.
4. **No instrument tuning after truth is read.** The `in` grammar, the `02`-node
   rule and the root definition are fixed by §1f *before* any truth is read. The
   terminus gate reads **no c2 output**, so fixing the grammar against it is not
   tuning; any such fix will be recorded here with its numbers.
5. **Nothing ships under `crates/`.** `PortC2` still returns `NotImplemented`
   outside its class; the gate is not widened; no `DISCLOSURE.md` row is owed
   because nothing from `docs/whitebox/` is adopted into `crates/`.
6. **The one-shot Part-1 gate is NOT spent and the 21-TU quarantine is
   honoured.** Registered now, before any number exists, for a reason about the
   object: this lane's model has **zero fitted parameters** — every field, width
   and trigger is transcribed from a named instruction — so there is nothing for
   a held-out population to catch. **Reversal condition:** if I end up choosing
   *any* rule by looking at truth, I must say so and the gate becomes owed.
7. **A refuted §1d is reported first.** If M11/M12 fail, the reading of S3/S4/S5
   is wrong and that goes in the first line ahead of any recall number.

---

## 5. Known-answer controls, registered with their pass marks

| # | control | pass mark |
|---|---|---|
| **KA-A** | reproduce the incumbent exactly: `\|U\|` 1 506 586, `\|E\|` 174 417, `\|Seed\|` 14 662, `\|P_RGL\|` 129 604, precision 1.00000, recall 0.74307, F1 0.85260, per-TU exact 132 | all eight identical |
| **KA-B** | **terminus gate** (M9), reported as a count of clean files **and** a count of decoded `02` nodes | ≥ 0.95, nodes > 0 |
| **KA-C** | the published witness: `PoolAlloc.cpp`'s `_CT…@Z268` record names its own copy constructor and its `??_R0` names `??_7type_info@@6B@` | exact, zero extra |
| **KA-D** | **KA-MUT through the SOLE JUDGE** (M11 + M12), on non-quarantined TUs, byte-length-preserving | ≥ 3/5 each |
| **KA-E** | incumbent gate on the unmodified tree: 687 passed / **0 failed** / 25 targets, 219 selftest PASS, 12/12 gate lanes 2 628 verdicts 0 mismatch, TU match 8 / 0 / 863 / 7, A/B/C/D/E 28/338/114/8/2, FRONTIER 17, census-gate disagreement 0 | every one reproduced, **compared on the FAILED count and the target count, never the passed count** |
| **KA-F** | dc3 HEAD before/after, wibo version disclosed | no mid-run move |
| **KA-POS** | **positive check** — `P_INIT` and `P_RGL` must DISAGREE, printed as a count of discriminating names. **A run whose discriminating count is 0 graded nothing and is a FAILURE, not a pass.** | > 0 |

---

## 6. Registered before the numbers exist

* **TU match stays 8.** Root work converts no TU on its own; factor A is
  necessary and not sufficient.
* **`census/gate disagreement` stays 0.**
* **A high recall would not make a shippable predicate.** A fail-closed
  `Emit/Skip/Unknown` needs precision at the ceiling *and* order, and order is
  untouched here.
* **`Rfloor` is a floor, not a target.** M8 is reported because w-roots was
  graded on it and the comparison is owed, not because closing it is the goal.
  The goal is `E`.

## 7. What this lane will NOT measure — named now, so absence cannot read as success

1. **S1's second entry** (`10b28b02`, kind-4 with `+0x37 & 0x400000`, taking
   `[sym+0x4c]` as a token while `+0x4c` is the emit flag word on tag-0x0E
   records). Either the field is a union or the bit selects another layout.
   **Not decided, and not decided by assertion.**
2. **S6's effect on the emit set.** A name minted during lowering usually has no
   body, so the compile loop skips it; whether any workload symbol is *reached*
   by S6 is untested here.
3. **`10b98e26`'s own skips** — `([owner+0x20] & 0x60) == 0x20`,
   `[[owner+0xc]+0x4d] == 0x1d`, `[owner+0x20] & 0x4000`. `I` ignores all three,
   which is why M2 is registered below the ceiling.
4. **The `0x14` node kind.** `[n] == 2 || [n] == 0x14` in memory; only the `0x02`
   byte kind is decoded. If a byte kind for `0x14` exists the terminus gate will
   fail on it, which is the point of the gate.
5. **`db` and `sy`.** Still uncaptured.
6. **`-optref`** (`FUN_10b27b7f`), the only path that clears `0x20`.
7. **Order.** A right set in the wrong order is still a mismatch.
8. **The 21 quarantined TUs.**
