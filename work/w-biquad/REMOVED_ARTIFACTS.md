# The 21 binary artifacts this directory used to track — removed 2026-08-25

**Owner decision, 2026-08-25: remove from the tree, do NOT rewrite history.**
So these files are gone from `HEAD` and still reachable in history at
`fea91947` and its descendants up to the removal commit. They remain on the
lane author's disk (untracked — `/work` is `.gitignore`d); nothing was
destroyed, and this note exists so a citation to one of them can still be
resolved.

`CLAUDE.md` § Commits forbids both classes — *"Never commit: … captured or
generated IL (`_CL_*`, `*.il`); build artifacts (`*.obj`, `*.o`, `/target`)"*
— and `.gitignore` forbids them twice over (line 16 `_CL_*`, line 20 `*.obj`,
line 24 `/work`). All 21 were force-added past those rules with `git add -f`.

Board **#3156** is the standing row. **It undercounted: it names 11 `.obj`
and does not count the 10 `_CL_*` captures, which appeared on no row at all
until this removal.** Its own prescribed fix is the one applied here:
**track the command, not the artifact.**

## What was removed, and how to regenerate each

| removed | count | regenerate from |
|---|---:|---|
| `probe/{labstride,park_extern,park_local,pool1,pool1ox,pool2}.obj` | 6 | the `.cpp` beside each, **still tracked**, through the reference toolchain (`pool1ox` is `pool1.cpp` at `/Ox`) |
| `real.obj`, `port.obj`, `peak.obj` | 3 | workload TU `src/system/synth_xbox/Biquad.cpp` — the reference side via `Toolchain::replay`, the port side via `c2rs`. The tracked fixture `wbiquad_fp_store_diamond.cpp` is that TU verbatim (rung §"the `_neg` file", line 299) |
| `fdedup_port.obj`, `fdedup_ref.obj` | 2 | `w13b_fdedup.cpp` at `/Ox` — the pair behind board **#2533**'s live wrong emit at relocation record 5 |
| `il/_CL_78764cab.{db,ex,gl,in,sy}` | 5 | a capture run over `Biquad.cpp` |
| `il_bq/_CL_72ee57bf.{db,ex,gl,in,sy}` | 5 | a capture run over the same TU at the lane's other arm |

Every `.cpp`, script and text artifact in this directory is **untouched** —
`scan.sh`, `fixscan.sh`, `PREREG.md`, the `.jsonl` scans, the verdict tables
and all six `probe/*.cpp` are still tracked.

## The citations that pointed at the objs

Four `crates/` doc comments cite `work/w-biquad/real.obj` as byte evidence,
and two cite `probe/*.cpp` (unaffected — those sources stay):

- `crates/c2-il/src/func/body/shapes/fp_store_diamond.rs:49`
- `crates/c2-core/src/codegen/fp_store_diamond.rs:387`
- `crates/c2-core/src/codegen/ctor_forward_call.rs:116`
- `crates/c2-core/src/codegen/fp_store_diamond.rs:331` (probe `.cpp`, fine)

**They are left as written.** Each names the *fact* it read (the 35 words,
the nine words) and the obj it read it from; that is a provenance statement
about a regenerable artifact, and the row above says how to regenerate it.
Rewriting six source comments to point at this note would be churn in the
byte-producing crate for a documentation gain — and the rung
(`docs/rungs/2026-08-09-w-biquad.md`) holds the derivations in full.

**What this removal does NOT do:** it does not purge history (owner's call,
declined deliberately — a rewrite would move every commit hash since
2026-08-09), and it does not add the funnel check that would have prevented
the force-add. `#3156` names that check — one `git ls-files` against
`.gitignore`'s own patterns, beside `board_audit.sh` — and it is **still not
built**. Nothing stops the next `git add -f`.
