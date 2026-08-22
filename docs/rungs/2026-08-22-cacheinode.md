# w-cacheinode — the capture cache stops being a traversal hazard: 9 inodes per entry become 3, at a root outside every checkout

    Tag:       w-cacheinode
    Slug:      cacheinode
    Date:      2026-08-22
    Kind:      construct — capture-cache container, root resolution, reclamation
    Outcome:   built
    Fixtures:  none — construct rung: capture-cache container and reclamation
    Census:    +0
    Record:    this file · board rows `#3406`–`#3409` · design-doc corrections
               `docs/CAPTURE_CACHE_DESIGN.md` `[C4]`–`[C6]` · format
               `crates/c2-il/src/cachefmt.rs`

Board rows **#3406**–**#3409**. Follows board **#3265** (the cache is unbounded
and nothing evicts) and **#1388** (an entry served from the wrong path fakes a
`mismatch`).

The question put to this lane was the user's: *"can we fix the storage format so
that we don't just have a ton of inodes? a packed file or sqlite dbs or
something?"* — with the goal stated as fixing the footgun rather than adopting
any particular storage.

**Answer: the storage engine was already the right one and `docs/CAPTURE_CACHE_DESIGN.md`
§4.2/§4.3 had priced both alternatives correctly. The *container* was not, and
neither was the *location*.** Three things landed: one live correctness bug found
on the way in, a reclamation path, and a fold plus relocation.

---

## 1. What was actually wrong

Not the format, and not primarily the bytes. `[C1]` in the design doc had already
established that deleting 98.7 % of 4.94 M entries returned **~17 GiB, not
~266 GB** — btrfs inlines sub-`max_inline` files. The problem is **metadata and
traversal**: 22,576,454 entries × 9 inodes ≈ 203 M inodes sitting *inside the
checkout*, where every `find`, `du`, `rg` or `**` glob walks them.

That has cost real time:

| when | what |
|---|---|
| 2026-08-04 | two kernel OOM kills, 62 GB and 72 GB of anon RSS in a `zsh` |
| 2026-08-21 | box wedged ~1 h at load 37.8, IO pressure **23 % *full*** stall |

The 08-21 event was two independent sessions tripping it inside the same hour.
Killing one 59-minute `work/**/*.txt` glob took load 37.8 → 23.5 and IO *full*
stall 23.3 % → 2.8 %.

## 2. Why pack-file and SQLite stay rejected — and where that argument stops

`cl.exe` runs under wibo and writes to a **path**; that path is baked into the
obj as `S_OBJNAME` in `.debug$S`, where even its *length* shifts bytes. So the
payload cannot leave the filesystem, and §4.2's conclusion holds verbatim: *"a
pack does not remove the directory; it adds a second copy of it."* SQLite
reclaims nothing, and the workspace is zero-dependency by hard constraint.

**But that argument constrains `out.obj` only.** The other seven files —
`key.bin`, `meta.txt` and the five `_CL_*` IL streams — are under no such
constraint: `IlBundle` is fully in-memory and replay materialises IL to a scratch
directory, never to the cache. §4.2's reasoning was right and its scope was too
wide. Recorded as `[C4]` in the design doc.

## 3. F1 — a live correctness bug, found while mapping the seam

`gap/scan.rs` restores the cached obj after replay, with a comment saying that
without it *"the scan would poison its own cache with the thing it was checking
for."* **`differential_tail` had no such restore**, while `replay`'s own doc says
it overwrites the path it is given.

So `c2rs diff --cache` left the *replay's* obj in the cache entry whenever the
replay diverged, and returned `ReferenceReplayMismatch` **having already
overwritten**. The next run serves those bytes as a hit → a false `mismatch` on a
byte-exact TU → an ALARM pointing at the port while the port is fine. Board
#1388's failure shape from a new cause, live in the gate's sweep row since
2026-08-18.

Fixed unconditionally rather than on the mismatch path: the uncached
`ref_obj_path` is scratch, where rewriting bytes that are already there is a
no-op — one rule, one implementation.

The test (`replay_overwrites_its_output_path`) pins the **premise**, not the
divergence. A non-diverging replay rewrites byte-identical content bar the COFF
`TimeDateStamp`, so a test of the leak itself would only fire when the two calls
straddle a second boundary — a race, not a check.

## 4. The GC, and the finding that redirects it

`c2rs cache stat | index | generations | show <key> | gc`. `gc` is a **dry run
unless `--apply`**; `--min-age` (default 1 h) is age used to *protect*, never to
evict, which is the whole of `[C3]`'s correction.

Enumeration is bounded by construction — `read_dir` is a lazy `getdents64`
iterator, never `.collect()`ed, never recursed, one `open` per entry.

Two measurements over the live 22.58 M-entry tree:

**(a) The source-orphan predicate reclaims ~nothing.** 50 k sample, 8.5 s:

```
live 47,423   UNREACHABLE 0   unknown 2,577
```

The design doc aimed §4.1(a) at a population that is not there: the sources are
tracked fixtures and workload TUs, and they still exist.

**(b) The churn is wibo rebuilds, not workload commits.** 1-in-400 sample,
56,441 entries read: **107 generations, of which the top four hold 99.6 % and
differ ONLY in the wibo version string** — identical `cl.exe`/`c1xx.dll`/`c2.dll`
digests, identical tree token, identical cache root. ~16.5 % is keyed on wibo
builds that no longer exist.

So the reclamation is the **relocation**, not the predicate: `cache-root` is in
the key, so moving the root makes 100 % of the old tree unreachable at once —
one `rm -rf`, no predicate to get wrong. The GC's value is **preventing
recurrence** at the new root, plus `--drop-generation`, which makes the 08-04
manual cleanup scriptable and counted.

The three-way verdict is the `[C3]` doctrine ported: parent readable + file
absent ⇒ DELETE; file present ⇒ KEEP; parent absent, unreadable, `EIO`, `ESTALE`
⇒ **KEEP, counted `unknown`.** *Unknown must not mean delete.*

`.locks` gets its own bounded pass, because `KeyLock`'s staleness break only runs
on **contention** — a lock abandoned on a key nobody wants again is never looked
at. Reaped only when the pid is dead *and* `/proc/<pid>/cmdline` is not a `c2rs`;
161 leaked lockfiles were found. `GcOptions` uses `try_acquire` — one
`create_new`, no wait, no stale-break — never `acquire`, whose 600 s wait and
1800 s break are right for a capture and wrong for a GC, where breaking a lock
means deciding a capture is dead.

## 5. Scale, proven

The claim that a bounded walk is safe at this size, measured rather than
asserted:

| | `zsh` glob (2026-08-04) | bounded `read_dir` |
|---|---|---|
| memory | **62–72 GB** → OOM-killed the box | **6.2 MB, flat** |
| result | machine down | **22,576,454 entries**, ~110 s, exit 0 |

**The OOM is a property of globbing, not of the entry count.** That is why the
design doc's appendix forbids globs and not `readdir`, and why the GC's comments
say in as many words that a future `.collect()` reintroduces it.

## 6. The fold — `entry.bin`

Nine inodes (dir + 8 files) → **three** (dir + `out.obj` + `entry.bin`).
Container in `c2_il::cachefmt`, a zero-dep leaf both the harness and
`tests/gl_alias_corpus.rs` can reach.

```
off  size  field
  0     8  MAGIC     b"C2RSCAP\x02"
  8     4  VERSION   u32 = 2
 12     4  N_SECT    u32, <= 16
 16     8  TOTAL_LEN u64, whole-file length including header
 24    32  DIGEST    32 ASCII hex = digest128(&file[56..])
 56          SECT[N] — 24 B each: TAG[8] NUL-padded, OFF u64, LEN u64
```

Tags in the fixed order `key, meta, ex, gl, sy, in, db`; canonical form
**enforced on read** (strictly ascending tag ordinals, contiguous payloads, no
slack), so `encode_entry` is a deterministic pure function and a whole class of
valid-but-weird blobs cannot exist.

**The fold is a strict strengthening, not a neutral inode trade.** v1's
completion marker was "`meta.txt` exists", written last — and `std::fs::write` is
create+truncate+write_all, not atomic, so a crash after two lines landed left a
*parseable* `base`+`arg` pair: a Hit with a truncated argv. v2 checks
`TOTAL_LEN == len` **before anything else in the file is trusted**, digests the
body including the section table, and appears by `rename(2)`.

**No migration is possible, only invalidation.** A v1 `out.obj` has the old
root's path baked into `S_OBJNAME`; rewriting the container cannot rewrite the
obj. A "migrated" entry is `EntryRead::Foreign` by construction, and suppressing
that check to make migration work re-creates #1388 exactly.

The pre-#1388 compat arm (`objpath` line absent ⇒ serve anyway) is **retired**:
after a hard invalidation it has provably zero members, so it was dead code that
read as a tested branch. A format bump is the one moment retiring it is free.

**No byte-win is claimed.** btrfs inlines sub-`max_inline` files, so five ~1 KB
streams may be inlined today while one ~6 KB blob gets its own extent. The inode
win is unambiguous; the byte effect wants measuring, not predicting.

## 7. Relocation

`--cache` → `$C2RS_GAP_CACHE` → `$XDG_CACHE_HOME/c2rs/capture` →
`$HOME/.cache/c2rs/capture` → **loud error**. Never a silent fallback into the
repo, which would resurrect the footgun invisibly. Callers on the capture path
degrade to no-cache and say so — a cache is a speedup, never a grading input,
and an unresolvable root became reachable at v2 (`sudo`, containers, read-only
`$HOME`).

Fold and relocation land **together** on purpose: the fold alone would make v1
dirs immortal garbage *inside the root you still have to GC*. Together, v2 starts
at a fresh root and 100 % of v1 sits at a root deleted with one `rm -rf`.

`~/.cache/c2rs` collapses *clones* as well as worktrees — more sharing than
before, which is the intent, but `rm -rf ~/.cache/c2rs` is now the blast radius
for every clone on the box.

## 8. Consumers

- `tests/gl_alias_corpus.rs` was the one that would have gone **silently wrong**:
  its `_CL_*gl` scan finds nothing after the fold, and it is env-gated, so it
  SKIPs when unset and would have found zero when set — absence read as success.
  Now reads the blob (v1 loose files still work, for archived corpora), and
  asserts loudly that a corpus yielding nothing is a failure.
- `scripts/gt_frame_class.py` had `glob.glob(<cache>/*/out.obj)` — the footgun in
  Python, materialising one string per entry. Now a bounded `os.scandir`, with
  the source filter going through `c2rs cache index`.
- `work/w-{db,joint}/cacheindex.py` and `work/w-quar/cachekey.py` each carried
  their own `digest128` and their own idea of the layout. Bannered dead and
  pointed at `c2rs cache index` / `show` — the supported readers. `c2rs cache
  index` exists precisely so nothing needs a second implementation.
- `capture_cache.rs`'s *"the default root is `<repo>/work/`, which `.gitignore`
  covers"* was the sentence making "captured IL is never committed" true. It is
  false now, and the guarantee is structural instead: there is nothing to ignore
  because there is nothing in the tree.

## 9. What this lane did NOT do

- **No enforcement of the no-traverse rule** (a `PreToolUse` hook, a
  `CACHEDIR.TAG`). Offered and declined. Relocation makes the rule mostly moot
  for the default root; it does not protect an operator who points a root back
  into a repo on purpose.
- **No byte measurement of the fold** (see §6).
- **No age-based eviction**, ever. `[C3]`.
