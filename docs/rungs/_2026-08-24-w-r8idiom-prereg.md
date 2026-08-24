# PREREG — `w-r8idiom`: the `mr r8,r8` idiom from both sides

> **PROVENANCE — DISASSEMBLY-DERIVED** for everything under §2/§3. See
> [`docs/whitebox/DISCLOSURE.md`](../whitebox/DISCLOSURE.md). Nothing here may
> enter `crates/` without a `DISCLOSURE.md` row naming the address it came from.

**Lane:** `w-r8idiom` · **kind:** characterization lane
(`docs/rungs/README.md` § "Lane kinds" 3) · **Fixtures:** none ·
**Census:** +0 · **predicted reach: 0**, registered here.
**Board rows:** **#3481**–**#3484**, reserved by the seventh-wave ledger
(`docs/BOARD.md`, decision 9 / `#3466`). Next free after the wave is `#3485`;
this lane mints nothing.
**Base:** `e85253cda`. **Zero `crates/` bytes, zero `fixtures/` bytes** — the
fence, to be shown by `git diff --numstat e85253cda..HEAD -- crates fixtures`
being empty in the rung.

**Subject.** `w-tailread`'s top-ranked follow-up
(`docs/rungs/2026-08-23-w-tailread.md` § "Found and not taken" items 1 and 6;
`docs/whitebox/ref/P_OPATTR.md` §6): **3,792 `mr r8,r8` self-moves in 1,206 of
120,000 objs**, all naming `r8`, branch-adjacent, no relocation covering them —
and the sibling handler `0x10c16d83` (peephole arm 14, `mr`) that
`w-tailread` did not read. Optional third: the second byte table `0x10c3b270`.

---

## §0 — Image, cache and addresses, VERIFIED before this file was frozen

**Image.** `compilers/X360/16.00.11886.00/c2.dll` via the main checkout,
sha256 `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
1 347 072 bytes — `sha256sum` run before any address below was touched. Nothing
under `compilers/` is committed by this lane.

**Corpus.** `~/.cache/c2rs/capture`, **386,686 entries at lane start** (`ls |
wc -l`). This is *more* than the 120,000 `w-tailread` walked, and it is the
same growing shared cache: see P1.1.

**The brief's addresses, checked against the image itself.** The coordinator
states in terms that it has not verified them.

| target | brief says | image says (first bytes) | verdict |
|---|---|---|---|
| `0x10c16d83` | arm 14's handler, `mr` | `53 55 56 8b f1 8b 46 2c 8b 4e 28 8b 68 1c 8b 59 1c 3b eb 75` | ✅ a function prologue, and it loads `[esi+0x2c]` / `[esi+0x28]` exactly as §6.1 says arm 6's does |
| `0x10c18373` | arm 14 thunk | `8b ce e8 09 ea ff ff e9 …`; `0x10c1837a + (−0x15f7) = 0x10c16d83` | ✅ the thunk does call it |
| `0x10c1838b` | arm 6 thunk | `8b ce e8 2b ec ff ff e9 …` | ✅ same 12-byte shape |
| `0x10c16fbd` | arm 6's handler | `56 8b f1 8b 4e 28 8b 46 2c 8b 51 1c 39 50 1c 75 3d …` | ✅ matches `P_OPATTR.md` §6.1's read |
| `0x10c3b270` | second byte table | `01 01 01 01 …` (20 bytes) | plausible; unread |

### §0.1 — `[POST]` — what I had already seen before freezing this file

Registered so none of it can be claimed as a discovery later.

* **`[POST]`** `P_OPATTR.md` §6's whole result: 3,792 / 1,206 / 120,000, the
  per-form split (`fmr` 32,569 : 0, `mr.` 150 : 0, `mr` 102,499 : 3,792), "all
  `r8`", "adjacent to branches (`op18`)", "`/Ox`", "no relocation covers those
  offsets". **None of these are my predictions**; where §1 predicts about them
  it predicts about *reproduction on a different corpus slice*, which is a
  different question.
* **`[POST]`** arm 14's handler **does** open with the same-register compare:
  `mov eax,[esi+0x2c] / mov ecx,[esi+0x28] / mov ebp,[eax+0x1c] /
  mov ebx,[ecx+0x1c] / cmp ebp,ebx / jne …`. I read those 20 bytes in §0
  above. So the brief's three-way split ("arm 14 either lacks the elimination,
  guards it, or is not reached") is **already down to two** before I start:
  the elimination test is present. Predictions P2.2–P2.6 are about *which of
  the remaining two*, and about what the `jne` path does.
* **`[POST]`** the capture cache entry format: `entry.bin` carries `src-arg`,
  `flags` and the full c2 argv, so source attribution needs no new capture.
* **`[POST]`** `0x10c3b270`'s first 20 bytes are all `0x01`.

---

## §1 — Obj side. Predictions

### P1.1 — the 3,792 will NOT reproduce as a number
`probe_selfmove.py --limit 120000` re-run today will **not** report 3,792.
`iter_objs` takes the *first* 120,000 in `os.walk` order out of a cache that
has since grown to 386,686 and is written by concurrent lanes; "the first
120,000" is a different set of objs. **Predicted:** a different absolute count,
with the *rate* (self-moves per obj, and % of objs carrying one) within a
factor of 2 of `w-tailread`'s 0.0316 / obj and 1.00 % of objs.
**Grade:** HIT if the number differs and the rate is within 2×; MISS if the
number reproduces exactly (which would mean the walk is stable and my reasoning
is wrong); PARTIAL if it differs and the rate moves more than 2×.

### P1.2 — the parameter that should not matter
Per `w-tailread`'s own transferable rule: I will re-run at a *different*
`--limit` and, separately, over a **shuffled / suffix-sliced** enumeration of
the same cache. **Predicted:** the *rate* is stable across both to within 2×,
i.e. the phenomenon is a property of the corpus and not of the walk order.

### P1.3 — still one register
On the largest corpus I run (target: the whole 386k cache), **every** self-move
still names `r8` and no other GPR. Calibration note: `w-read-r6` recorded that
its misses all predicted the mechanism *tidier* than it is, and `w-tailread`
found that biasing toward mess is a correction and not a law. Here I predict
the **tidy** answer deliberately, because a fixed register is already the
weirdest part of the finding. **Grade:** MISS if any non-`r8` self-move appears.

### P1.4 — position: it terminates a run, it does not sit inside one
"Adjacent to branches" will resolve to: the self-move's **successor** is a
branch far more often than chance, and the dominant single shape is
`self-move` immediately **followed** by an unconditional branch or a `bl`.
**Predicted:** ≥ 60 % of instances have a primary-18 (`b`/`bl`) or primary-16
(`bc`) instruction within the two words *after* them.

### P1.5 — it is not spread evenly over the corpus
The 1,206 objs will trace to a **small number of distinct source files**
(`src-arg`), not to a broad cross-section: **predicted ≤ 20 distinct
`src-arg` basenames** account for ≥ 80 % of self-move-bearing objs.

### P1.6 — this is my own method's registered failure mode
The capture cache is a **fixture** corpus written by ~80 lanes, heavily
dominated by generated fixtures. **Predicted:** the self-moves will trace
predominantly to *machine-generated* fixtures (a sweep/cross generator's
`.cpp`), not to hand-written or dc3-derived TUs — in which case "what source
shapes produce them" is answerable only about that generator, and any claim of
the form "this is what MSVC does for C++ shape X" is unsupported. If that
happens I will say so and **decline to generalise**, rather than publish the
generator's shape as c2's idiom. **Grade:** HIT if ≥ 50 % of bearing objs are
generator-produced.

### P1.7 — the flags claim will widen
`P_OPATTR.md` §6 says *"the unit was compiled `/Ox`"*. **Predicted:** the
bearing objs carry **more than one** flag set, and the self-move appears under
at least one non-`/Ox` optimisation level too — because the cache is
multi-lane. **Grade:** MISS if every bearing obj is `/Ox`.

### P1.8 — the neighbourhood is not a prologue
§6 says *"the surrounding words decode as a coherent prologue"*. **Predicted:**
the self-move is **not** in function prologues generally; the dominant context
is mid-body around a call. **Grade:** graded by measured position-in-section.

### P1.9 — what I think it IS, stated in advance so it can be wrong
Ranked, and I will grade whichever the evidence reaches:

1. **A `nop`-out, not a deletion** — a pass *after* the peephole rewrites a
   now-dead instruction in place to a self-move rather than unlinking it, and
   `r8` is whatever register the rewrite hard-codes. *Most likely.*
2. **A fixed-register idiom with architectural meaning** — PowerPC uses
   `or rN,rN,rN` as a hint form (`or 1,1,1` / `2,2,2` / `3,3,3` priority;
   `or 28..31` as Xenon `dbNcyc` delays). `r8` is **not** in either published
   family I can name, which argues against this — registered anyway so that if
   it turns out to be a documented Xenon hint I cannot claim I meant it.
3. **Padding / alignment** to place a following branch or label.
4. **A genuine missed coalesce** that reaches the object because the peephole
   ran before the allocator assigned both operands the same physical register.

**Predicted:** the obj evidence alone will *narrow* this to at most two, and
will not by itself pick one. Registering that this lane may end at
"deliberately not guessed at", which `w-tailread` established as a valid final
state and which R6's precedent (refusing to publish) vindicated.

---

## §2 — Image side. Predictions about `0x10c16d83` (arm 14)

### P2.1 — `[POST]`, not scored
The same-register compare is present. Already seen (§0.1), **not a prediction**.

### P2.2 — arm 14 is bigger than arm 6
`FUN_10c16fbd` is 191 bytes. **Predicted:** `FUN_10c16d83` is **larger**,
because `mr` is the integer form and carries cases (`Rc`, `XER`, global
register bindings) that `fmr` does not. **Grade:** MISS if it is ≤ 191 B.

### P2.3 — there is a GUARD on the equal path
**Predicted:** on the `cmp ebp,ebx` **equal** path, arm 14 does *not* go
straight to the unlink; there is at least one further test (a flag bit on an
operand descriptor, a check on the containing instruction, or a check of a
global/frame register) that can send it away from the delete. **Grade:** HIT if
≥ 1 additional conditional branch sits between the equality test and the
tail-call to `0x10c16cde`; MISS if the equal path is an unconditional
straight line to the unlink, as arm 6's is.

### P2.4 — the guard names a specific register class
**Predicted:** if P2.3 hits, the guard is about the operand being a
**physical / pre-coloured** register rather than a virtual one (a "this
operand is pinned, do not touch" bit), not about the opcode. Stated so a
vaguer "there is a guard" cannot be scored as this.

### P2.5 — arm 14 IS reached
**Predicted:** arm 14's thunk `0x10c18373` sits in the same peephole jump table
as arm 6's `0x10c1838b` (12 bytes apart in the same run), so "not reached" is
the least likely of the brief's branches. **Grade:** HIT if the two thunks are
entries of the same table and nothing gates arm 14's entry that does not also
gate arm 6's.

### P2.6 — **reading arm 14 will not close the question**
My registered expectation for the lane as a whole: **the read of arm 14 will
show an eliminator that does fire, and the surviving `mr r8,r8` will therefore
have to come from a producer that runs after this peephole** — exactly what
`P_OPATTR.md` §6's closing note already suspects. **Predicted:** the lane ends
with the *mechanism* still not identified, or identified only from the obj
side. **Grade:** HIT if I cannot exhibit a path from arm 14 to the surviving
self-moves; MISS if arm 14 turns out to explain them.

### P2.7 — the corroboration path, registered in advance
`[R]` means the instructions were read correctly and **never** what c2 does —
the `.bss`-bump failure mode has struck twice. The corroboration for every
arm-14 claim in this lane is the **obj population** (`[O]`): if arm 14
eliminates unconditionally then `mr rX,rX` for `X ≠ 8` must be absent from the
corpus, and that is checkable. If the read and the objs disagree, the objs win
and the read stays `[R]` with the disagreement printed.

---

## §3 — Optional. `0x10c3b270`, only if cheap after §1 and §2

### P3.1
**Predicted:** it is a **second denormalised column** of the same
`0x10b1b260` mnemonic-table row (stride 12, `{name, form, flags}`) — i.e. the
`form` field flattened to one byte, the way `0x10c3afd8` is the `flags` field
flattened. **Grade:** HIT if byte-identical to some field of the mnemonic
table on ≥ 99 % of 664 entries.

### P3.2
**Predicted:** the out-of-range default `0x64` (100) is **not** a value the
table itself contains at a valid index — it is a sentinel. **Grade:** MISS if
`0x64` appears among the in-range entries.

### P3.3
**Predicted:** §3 is **dropped**. Two full sides plus fences is the budget;
registering the drop in advance so that not doing it is not silently a failure.

---

## §4 — Method fences, registered before use

* Every tool goes in `docs/whitebox/scripts/`, sha256-fenced to the pinned
  image, and **each fence is watched refusing deliberately broken input**
  before any number from it is quoted — truncated image, flipped byte, empty
  corpus. `w-tailread` found a real ordering bug in its own probe this way, and
  four of its own defects came out of *running* things.
* Before quoting any count, change a parameter it should not depend on and
  re-run (P1.2 is that discipline, written as a prediction).
* Claims graded against objs are `[O]`; claims from the image are `[R]`.
* **Refusing to publish is an acceptable outcome.** R6's precedent.

## §5 — What this lane will NOT do

* No source-shape probe grid. Board **#3052** binds: do not build probe grids
  for register-allocation quantities — read and measure the objs already held.
  Any new `.cpp` is out of scope, and `fixtures/` gets zero bytes.
* No `crates/` byte, no adoption of any address into the port.
* No new board numbers beyond `#3481`–`#3484`.
* `WB_EXPAND_FINDINGS.md:79` and board `#3432`'s stale "unrecorded" sentence
  (`w-tailread` follow-up 5) are **out of this lane's reserved rows** and stay
  unfixed.

**Frozen before the first read.** Grades land in
`docs/rungs/2026-08-24-w-r8idiom.md`.
