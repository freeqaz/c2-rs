# PREREG — lane `w-cache`, 2026-08-08

Written **before the first change**, at base `119af05f`. Scored in the rung.

Two items, both in the verification apparatus. Neither touches codegen, so
neither can move a rung count except by fixing an instrument that was lying.

---

## ITEM 1 — board #1388: a relative `--cache` path grades a byte-exact TU `mismatch`

### What I expect the mechanism to be

Read off the source before running anything, so the reproduction can falsify it:

* `CaptureCache::new` puts the **canonicalized** cache root in the key
  (`cache-root <canonical>`), but stores `self.root` **as given** — the raw,
  possibly relative, spelling.
* On a **MISS**, `capture_reference_with(src, &dir, …)` calls `absolute(dir)`
  internally, so `-Fo` and the returned `ref_obj_path` are **absolute** whatever
  the spelling. Both sides of the differential then agree. *A first run is always
  green — the defect cannot fire on a miss.*
* On a **HIT**, `read_entry` returns `ref_obj_path = self.root.join(key)/out.obj`
  — **relative**, because `self.root` was never absolutised. `to_wibo_path` of a
  relative path returns it **unchanged and without the `Z:` prefix**, so the port
  bakes a different, shorter `S_OBJNAME` than the cached reference obj carries.
  The compare then diverges inside `.debug$S` and the objs differ in length.

So: the key already folds the two spellings onto **one entry** (canonicalized),
while the *served path* does not. The two disagree, and only on a hit.

### Direction I expect to be wrong in

**Safe** — a false `mismatch`, never a false `match`. The port's obj is *shorter*
because the relative spelling is shorter; a longer path could never make two
different captures compare equal. I expect to confirm this and I expect no rung
count anywhere to move.

### What I will do

Absolutise the root at the point it enters the cache (`CaptureCache::new`), so
the served path and the keyed path are the **same** string. I register in advance
that I expect **not** to need `crates/c2-obj` — if the compare turns out to need
relaxing, that is a refusal, not a fix, and I will stop and say so.

Second, belt and braces: record the capture's own `-Fo` path in the entry's
`meta.txt` **provenance** and make a served entry whose recorded path is not the
path it is being served from a **MISS**. I expect this guard to fire **zero**
times in normal operation and I register that a zero is the expected reading, not
a null result.

### Registered predictions

1. On the **base** binary, `--cache <relative>` on a byte-exact TU reports
   `mismatch` on the **second** run and `match` on the first. **Confident.**
2. The two objs differ in **length**, by the number of characters the absolute
   prefix adds (rounded up by `.debug$S` record alignment). **Confident.**
3. On the **tip** binary, relative and absolute spellings give the **same**
   verdict, and it is `match`. **Confident.**
4. The `cache-root` line in the key is unchanged by my fix, so **existing cache
   entries stay valid** and no peer pays a re-capture. **Moderately confident** —
   this is the claim most likely to be wrong, because bumping `CACHE_FORMAT` is
   the documented reflex for a layout change and I am deliberately not doing it.
   If old entries do go cold, peers pay ~450 s of CPU once and I will say so.
5. `mismatch 0` everywhere at both ends. **Confident.**

---

## ITEM 2 — board #1406: `hatch_red.py` has no automated test

### What I will do

One additive `gate.sh` row, run **synchronously before `pin_harness`** — i.e.
before the single `cargo build` the gate does and before any lane starts — so
that the arms' writes into `crates/` can never race a compile.

### The thing I expect to get wrong, and pre-empt

**Reddening a peer.** `hatch_red.py` writes to six `crates/` files and restores
them with `git checkout --`; on a tree that already differs from `HEAD` under
`crates/` it would eat the difference, and on a tree whose commit has moved a
hatch needle `apply` refuses (#1389's shape) and the arms cannot be built. Both
are properties of the **tree**, not of `hatch.py`, and both are conditions a
peer mid-wave is likely to be in.

So I register the outcome set in advance:

| condition | row verdict | gate |
|---|---|---|
| all 11 arms pass | `PASS` | contributes 11 arms |
| an arm's guard did not fire | `FAIL` | **gate FAIL** |
| `crates/` differs from `HEAD` | `REFUSED` | qualified PASS, files named |
| the hatch will not apply to this tree | `REFUSED` | qualified PASS, cause named |
| `crates/` left dirty afterwards | `FAIL` | **gate FAIL** |
| ran, no recognizable verdict | `NO-RESULT` | **gate FAIL** |

I expect the loudest criticism of this to be that `REFUSED` is a fig leaf. My
answer in advance: it exits 0 but it **cannot print an unqualified `GATE: PASS`**
— same treatment `SAMPLED` and `SKIPPED` already get, for the same reason.

### Registered predictions

6. `hatch_red.py` passes 11/11 on `119af05f` with a clean tree. **Confident**
   (measured before writing this: exit 0, 9 red + 2 green, 8 distinct words).
7. The existing 18 lanes' counts, the sweep's and the cross's are **unchanged**
   digit for digit by the new row. **Confident** — the row is prepended, shares
   no state, and restores the tree before the build.
8. Mutating the new row's classifier makes it go red with **its own leading
   word**, and no other case's expectation is satisfied by it. **Moderately
   confident** — this is where the two observed traps live and where I expect to
   find a bug in my own work.
9. `git grep -c '#\[test\]'` under `crates/` moves by **+0 or a small positive**;
   the new coverage is in `gate.sh --selftest`, not in `cargo test`. **Confident.**

---

## What would make me stop

* Item 1 needing a change in `crates/c2-obj`'s compare. That would mean the
  differential's normalization is wrong, not the cache's path handling, and it
  is not a thing to fix quietly inside a cache lane.
* Item 2's row failing on a peer's clean tree for any reason I cannot name.
