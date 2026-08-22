# The capture cache: measured, and what to do about it

Lane **w-cache**, 2026-08-04. Read-only investigation — nothing was deleted,
moved, or changed. This document is a measurement and a recommendation.

The question put to this lane was: *does `work/capture-cache` need a new storage
structure, or just a delete?*

**Answer: just a delete, plus a retention policy and a one-line environment
change. No new storage structure is warranted, and the database is a clear
no.** The measured facts that force that answer are below; the design
alternatives are costed at the end so the decision is reviewable rather than
asserted.

---

## CORRECTION NOTE (added 2026-08-04 by lane **w-land4**, at merge)

This document was written on 2026-08-04 and **merged later the same day**, after
the cleanup it recommended had already been carried out. That cleanup **refuted
three of its claims**. Per this project's convention a dated record is corrected
visibly, never silently rewritten: **the original text below is unchanged**, and
every affected passage carries an inline marker — **[C1]**, **[C2]**, **[C3]** —
pointing back here. Read both. What was believed and what turned out to be true
are each part of the record.

Authority for all three: **ROADMAP §10.22** ("W-GC — the concurrent-capture bug
was LIVE, and the cache was never a disk problem") and **BOARD #182** (REFUTED).

### [C1] The `~266 GB` is WRONG by more than an order of magnitude — do not requote it

Deleting **98.7 % of the 4.94 M entries returned ~17 GiB to `df`**, not ~266 GB.
Three compounding reasons, each of which will recur:

1. **`du -s` reports blocks × 512**, so every file rounds up to the 4 KB block.
   These files average **~850 B**. On a corpus of millions of sub-block files
   `du` is not a noisy estimate of bytes — it is measuring a different quantity.
   §2.3 below *sees* this ("4.8× block-rounding amplification") and then still
   propagates the block figure as the byte total.
2. **btrfs inlines files under `max_inline` directly into metadata.** Files that
   small never occupy a data extent at all, so the data-space saving from
   deleting them is near zero by construction.
3. Therefore **the caches were an inode/metadata problem, not a data problem.**
   The cost was millions of metadata records and the walk time over them — which
   is why the standing constraint is "never recursively walk
   `work/capture-cache`" and not "watch the disk".

The verification the cleanup brief suggested could not have caught it either:
**`df -i` reports nothing useful on btrfs**, which has no fixed inode table.
§10.22 records this as a *recurrence* of §6s's `/tmp` misreading in the opposite
direction — there a metadata limit was read as a space limit, here a metadata
cost was priced as a space cost. **A space-shaped number is not evidence of a
space-shaped problem.**

Every GB figure in this document is `du`-derived and carries the same error.
The **entry**, **file** and **inode** counts are unaffected and stood up.

### [C2] "47 of 50 sibling caches ≥ 2 days old" was **44** when measured strictly

Against a hard 48-hour cutoff the count is **44**, not 47. The cleanup lane
deliberately **declined to round the gate** to make its own precondition true.

### [C3] The age-based GC is **NOT safe as stated** — age was replaced, not tuned

§4.1(a) argues age is a sound proxy because "hits do not update mtime … age is a
proxy for *write* recency only". That is the correct mechanism and the wrong
conclusion. A directory's mtime is its **creation** time; a cache **hit never
rewrites it** and `/home` is `noatime` — so an entry that has served a hit on
**every gate run since 07-31 still reads as three days old**. The population this
hits hardest is exactly the one that matters: gate lanes scan repo `fixtures/`
with no `--cwd` and key as `unknown+dirty-unknown`, so a 48 h age GC would have
evicted the live gate working set while it was still being used.

The cleanup lane therefore replaced age with a **provably-unreachable**
predicate — the source file is gone, so `key_material` returns `None` and the key
**cannot be formed**; or a stale workload-tree token, verified absent in a
2,321-entry sample — and deliberately **kept 27,451 entries older than 48 h**.

### What this document got RIGHT, and what the correction does not touch

The three corrections above are about **magnitudes and one retention predicate**.
They do not reach the document's conclusions, which stood up in full:

- **"Just a delete."** No new storage structure was warranted and none was built.
- **The pack-file rejection (§4.2).** It rests on the `-Fo` / `S_OBJNAME`
  constraint — the cache directory *is* the capture directory — not on the byte
  arithmetic, so [C1] leaves it untouched. A pack is a second copy of the
  directory it was meant to replace, and still needs the GC.
- **The SQLite rejection (§4.3).** The payload cannot live in the engine;
  atomicity and integrity are already handled; the one gap it appeared to close
  it does not close.
- **Both one-line follow-ups were real and both landed** in `wt-w-gc`
  (`c72a2a6`): canonicalise `cwd` in `key_material`, and the `O_EXCL` per-entry
  lockfile. The race §4.3 called "rare but real" was **live** under
  `gate.sh --jobs N`, which runs N processes against one root.

A lane that reaches the right decision through one bad estimate is worth
recording accurately in both directions.

---

## SECOND CORRECTION NOTE (added 2026-08-22 by lane **w-cacheinode**)

Eighteen days later the count was back: **22,576,454 entries**, past the 21.5 M
this document measured, and the footgun fired twice more — an hour-long wedge of
the whole box at load 37.8 with IO pressure at 23 % *full* stall, from two
independent sessions tripping it inside the same hour. Three findings, one of
which reverses a conclusion above.

### [C4] "Just a delete" was right about the *storage engine* and wrong about the *container*

The pack-file (§4.2) and SQLite (§4.3) rejections **hold in full** and were
re-checked before anything was built: `cl.exe` writes to a *path*, that path is
baked into the obj as `S_OBJNAME` where even its length shifts bytes, so the
payload cannot leave the filesystem — *"a pack does not remove the directory; it
adds a second copy of it."*

But that argument constrains **`out.obj` only**. The other seven files —
`key.bin`, `meta.txt` and the five `_CL_*` IL streams — are under no such
constraint: `IlBundle` is fully in-memory and replay materialises IL to a scratch
directory, never to the cache. They are now **one `entry.bin` blob**
(`c2_il::cachefmt`), taking an entry from **nine inodes to three**. §4.2's
reasoning was correct and its scope was too wide.

The fold is also a **strict strengthening**, not a neutral trade. This document's
completion marker was "`meta.txt` exists", written last — and `fs::write` is
create+truncate+write_all, not atomic, so a crash after two lines landed left a
*parseable* `base`+`arg` pair: a Hit with a truncated argv. `entry.bin` carries
`TOTAL_LEN` and a digest and appears by `rename(2)`.

### [C5] The GC's own predicate reclaims ~nothing — the *root* was the lever

§4.1 recommended a GC and the 08-04 cleanup applied "the source file is gone"
by hand. Built as code and measured over the live 22.58 M-entry tree
(`c2rs cache gc`, 50 k sample): **47,423 live, 0 unreachable, 2,577 unknown.**
The sources are tracked fixtures and workload TUs — they still exist. The
predicate was aimed at a population that is not there.

What the churn actually is, from a 1-in-400 generation histogram
(`c2rs cache generations`): **107 generations, of which the top four hold 99.6 %
and differ ONLY in the wibo version string** — identical `cl.exe`/`c1xx`/`c2`
digests, identical tree token, identical root. ~16.5 % is keyed on wibo builds
that no longer exist. The driver is **wibo rebuilds**, not workload commits as
§0 assumed.

So the reclamation is the **relocation**, not the predicate: `cache-root` is in
the key, so moving the root makes 100 % of the old tree unreachable at once,
retired with one `rm -rf` and no predicate to get wrong. The GC's value is
**preventing recurrence** at the new root, plus `--drop-generation`, which makes
the 08-04 manual cleanup scriptable and counted.

### [C6] The root is out of the repo, so the appendix's rule is no longer load-bearing

The appendix below ("how to touch this directory without taking the box down")
is correct and stays. But it is a **rule**, and rules do not bind the traversals
that actually cause this: a shell glob, an editor's indexer, another agent's
`du -sh .`. The default root is now `$XDG_CACHE_HOME/c2rs/capture` (then
`$HOME/.cache/…`, then a **loud error** — never a silent fallback into the tree).
A `find` rooted at the checkout no longer reaches it at all. Treat the appendix
as advice for anyone who points a root back into a repo on purpose.

The scale claim in the appendix is now measured rather than asserted: a bounded
`read_dir` walked all 22,576,454 entries in **6.2 MB RSS, flat**, in ~110 s. The
OOM is a property of *globbing*, not of the entry count.

---

## 0. The headline numbers

| | main cache | the 50 sibling caches | total |
|---|---:|---:|---:|
| entries | 944,936 | 3,996,458 | **4,941,394** |
| files (8/entry) | 7.56 M | 32.0 M | **39.5 M** |
| inodes (files + dirs) | 8.50 M | 36.0 M | **44.5 M** |
| on-disk bytes **[C1]** | ~112 GB | ~154 GB | **~266 GB** |

**[C1] — the whole bytes row is WRONG.** Deleting 98.7 % of these entries
returned **~17 GiB**, not ~266 GB. See the correction note above. The entry,
file and inode rows are unaffected.

The volume holding all of it is 3.7 TB, 93 % full, with **262 GB free**. The
capture caches are approximately equal to every remaining free byte on the
disk. **[C1] — this inference falls with the number: they were ~6 % of it.**

Of the main cache:

- **93.9 %** of entries name a source file that **no longer exists**.
- **≤ 2.7 %** of entries could produce a hit today even in principle.
- **98.5 %** of entries were written on two days, 2026-07-31 and 2026-08-01.
- Measured hit rate across every scan report still on disk: **33,358 hits /
  117,733 misses = 22.1 %**, and it is bimodal — see §3.

---

## 1. Method, and why sampling is sound here

Entry names are the 32-hex `digest128` of the key material
(`capture_cache::digest128`, two FNV-1a-64 passes). A hash name is by
construction uncorrelated with the entry's content, its provenance, its age and
its size, so the `ls`-sorted ordering is an arbitrary permutation with respect to
every property measured here. **Systematic every-*k*-th sampling over that
ordering is therefore equivalent to simple random sampling**, and it is far
cheaper than shuffling a 945k-line list.

That claim was not left implicit. Two **disjoint** systematic samples were drawn
from the same 944,936-name snapshot:

- **Sample A** — offsets 1, 201, 401, … → N = 4,725
- **Sample B** — offsets 101, 301, 501, … → N = 4,725

and every headline proportion was computed on both:

| quantity | sample A | sample B |
|---|---:|---:|
| `/tmp/c2rs-cross-*` sweep entries | 73.42 % | 72.93 % |
| `/tmp/c2rs-swm-*` sweep entries | 5.78 % | 6.35 % |
| dc3 workload TUs | 3.03 % | 3.43 % |
| est. total on-disk bytes | 111.0 GB | 112.5 GB |
| est. bytes held by dc3 TUs | 81.0 GB | 82.6 GB |

The two agree inside their sampling error everywhere. At N = 4,725 the 95 %
interval on a proportion near 90 % is about ±0.9 pp and near 3 % about ±0.5 pp;
the byte totals are heavy-tailed and carry a 1σ of ≈ 9 GB (see §2.3), which is
why they are quoted to two significant figures and no further.

Every command was `ls`-streamed or `xargs`-bounded. No glob was ever expanded
inside the cache directory — see the warning at the end of this document.

---

## 2. Composition

### 2.1 Provenance

Classified from each sampled entry's `meta.txt` (`arg -f <source>`) and its
`key.bin` (`cwd` field, needed to resolve relative source arguments).

| provenance | share (A) | share (B) | est. entries | est. GB | source exists? |
|---|---:|---:|---:|---:|---|
| `/tmp/c2rs-cross-*` generated sweep | 88.1 %¹ | 87.1 %¹ | ~832,000 | 27 | **0 %** |
| `/tmp/c2rs-swm-*` generated sweep | 5.8 % | 6.4 % | ~55,000 | 2 | **0 %** |
| dc3 workload TU | 3.03 % | 3.43 % | ~29,000 | **81** | ~100 % |
| `<repo>/work/sweeps` fixtures | 1.48 % | 1.27 % | ~14,000 | 0.5 | 100 % |
| worktree-local fixtures | 1.06 % | 1.12 % | ~10,000 | 0.35 | 100 % |
| `<repo>/fixtures/cpp` | 0.36 % | 0.47 % | ~3,400 | 0.15 | 100 % |
| `/tmp/w-r1b-baseline` | 0.21 % | 0.25 % | ~2,000 | 0.1 | 100 % |

¹ `cross-sweep` + `cross-wcb` + `cross-wch` combined.

Existence was tested by resolving each sampled `-f` argument (`z:\…` → host
path; relative → joined to the entry's own recorded `cwd`) and `ls`-ing it.
**258 of 4,725 resolved paths still exist**; a further 28 are dc3 TUs recorded
under a *relative* cwd (`../dc3-decomp`, `../../../../dc3-decomp`) that cannot be
resolved out of context and are almost certainly alive. Taking those as alive
gives **290 / 4,725 = 6.1 % live sources, 93.9 % orphaned**.

**The entry-count problem and the byte problem are two different populations.**
94 % of the *entries* are dead sweep residue holding 29 GB; 3 % of the entries
are dc3 TUs holding **81 GB, 72 % of all the bytes**. Any policy that optimises
only for entry count will leave almost all the storage in place, and vice versa.

**[C1] — the split is real, the framing is not.** There was no byte problem to
be a second population of: the entire cache was ~17 GiB of reclaimable space,
and the cost was the metadata records and the walk time over 44.5 M inodes. The
correct reading is that the *entry-count* problem was the only problem, which
makes the entry-count-optimising policy the right one after all — for a reason
this section argues against.

### 2.2 Generations

29 distinct full key-contexts appear in sample A. Decomposed:

| component | distinct values in sample A | note |
|---|---:|---|
| `tool cl.exe / c1xx.dll / c2.dll` digests | 1 | the toolchain never moved |
| `wibo <version>` | 2 | 4,701 current, 24 on `1.0.1-7-g3b0f71c-dirty` |
| `tree <HEAD>+<clean\|DIRTY>` | 19 | 18 dc3 HEADs + `unknown+dirty-unknown` |
| `tree-dirty <digest>` | 11 | content digest of the dirty working set |
| `cache-root <path>` | 1 present… | …but **absent from 1,026 / 4,725 entries** |
| recorded compile `cwd` | 4 non-empty | 4 spellings of the *same* directory |

Four separate generation multipliers are visible, and each is a full orphaning
event:

1. **`cache-root` was added to the key after these entries were written.**
   21.7 % of the cache (≈ 205,000 entries) predates that component. The current
   binary cannot construct their keys at all — they are unreachable by
   construction, forever.
2. **The dc3 tree identity.** 18 distinct HEADs × 11 dirty-digests in a sample
   that contains only 143 dc3 entries. Every commit *and every uncommitted edit*
   to `../dc3-decomp` mints a new generation of all 878 TUs. ~29,000 dc3 entries
   ÷ 878 ≈ **33 full generations**, of which at most one can ever hit.
3. **Four spellings of one cwd.** `/…/dc3-decomp`, `../dc3-decomp`,
   `/…/c2-rs/../dc3-decomp`, `../../../../dc3-decomp` are the same directory and
   four different keys. `key_material` stringifies `cwd` without canonicalising
   it — a 1-line fix that would fold four generations into one. (`cache-root`
   *is* canonicalised; `cwd` is not. The asymmetry looks unintentional.)
4. **`cache-root` itself, across lanes** — see §2.5.

### 2.3 Sizes

Per-entry, on disk (`du -s`), sample A (N = 4,725):

```
apparent   mean 91,866 B   p10 5,711   p50 6,803   p90 8,655   p99 2.64 MB   max 10.2 MB
on-disk    mean 117,509 B                p50 32,768              p90 32,768
```

The median entry is **6.8 KB of data occupying 32 KB of disk** — 8 files, each
rounded to a 4 KB block, on a filesystem that is already compressing with
`zstd:3`. **4.8× block-rounding amplification.** Across the ~900,000 small
entries that is ≈ 22 GB of pure padding. Total apparent bytes for the main cache
are ~87 GB against ~112 GB occupied.

Extrapolated totals from the two disjoint samples: **111.0 ± 9.0 GB** and
**112.5 ± 8.9 GB** (1σ; heavy tail).

**[C1] — the two samples agree with each other and both are wrong.** Every
figure in this section is `du`-derived, i.e. blocks × 512 over ~850 B files, on
a filesystem that also *inlines* files that small into metadata. The reported
"4.8× block-rounding amplification" is the error being observed and then carried
into the total anyway. Reproducibility across disjoint samples measures sampling
noise, not the systematic error in what the tool counts.

### 2.4 Age

`meta.txt` mtime, sample A extrapolated:

| day | est. entries | share | est. GB |
|---|---:|---:|---:|
| 2026-07-31 | ~422,000 | 44.7 % | 46.1 |
| 2026-08-01 | ~508,600 | 53.8 % | 27.7 |
| 2026-08-02 | ~8,000 | 0.85 % | 23.1 |
| 2026-08-03 | 0 | 0 % | 0 |
| 2026-08-04 (today) | ~6,400 | 0.68 % | 14.3 |

**98.5 % of the cache was written on two days**, by the cross-product sweeps
whose `/tmp` corpus has since been deleted. Today's five running lanes account
for ~6,400 entries and ~14 GB — 0.68 % of the entries and 13 % of the bytes.

### 2.5 The 50 sibling caches — the finding that dwarfs the original question

`main.rs` defaults the cache root to `provenance::repo_root().join("work/capture-cache")`,
and `repo_root()` is `env!("CARGO_MANIFEST_DIR")/../..` — resolved **at compile
time**. A binary built inside a worktree therefore writes to *that worktree's*
`work/capture-cache`. There are 83 worktrees under `.claude/worktrees/`, and **50
of them hold their own cache**:

```
530,390  agent-ad606bd5c3fca6eac      17.4 GB
529,543  agent-a01a1acdd2f78afa0      17.4 GB
529,422  w-r1                         17.3 GB
188,025  class-b                       6.2 GB
183,796  w-fp                          6.0 GB
144,265  lane-registry                 4.7 GB
…44 more…
─────────────────────────────────────────────
3,996,458 entries                    ~154 GB
```

The top three are **not copies** — their key names are disjoint. Three lanes each
ran the same ~530k-entry cross-product sweep and each stored its own complete
copy of the result, because `cache-root` is part of the key and their roots
differed. The IL bundles in those three caches are byte-identical to one another
(the `.gl` embeds the *source* path, which was the same `/tmp` corpus); only
`out.obj` genuinely differs, and only because c2 bakes its `-Fo` path into
`.debug$S`. **≈ 85 % of ~35 GB is a three-way duplicate of the same bytes.**

Last-write times say almost all of it is abandoned:

| last written | caches | entries | est. GB |
|---|---:|---:|---:|
| 2026-07-31 | 31 | — | — |
| 2026-08-01 | 13 | — | — |
| 2026-08-02 | 3 | — | — |
| 2026-08-04 (live) | 3 | 8,575 | ~4 |

**47 of 50 sibling caches, ≈ 3.99 M entries and ≈ 150 GB, have not been touched
in ≥ 2 days.** Their lanes are finished. When their worktrees are removed the
caches go with them; until then they are pure carry.

**[C2] — 44, not 47**, measured against a hard 48-hour cutoff; the cleanup lane
declined to round the gate. **[C1] — the ≈ 150 GB is `du` blocks**, not
reclaimable bytes.

---

## 3. The hit rate

Every `cache_hits` / `cache_misses` pair recorded in a scan report still on disk
(`work/**.jsonl`, `work/**.log`, excluding the cache itself), deduplicated:

| hits | misses | what it was |
|---:|---:|---|
| 0 | 42,719 | cold cross-product sweep |
| 0 | 27,956 | cold cross-product sweep |
| 0 | 18,051 | cold sweep |
| 1 | 14,121 | cold sweep |
| 1 | 13,706 | cold sweep |
| 0 | 878 | **cold dc3 workload scan** (×5 occurrences) |
| 0 | 201 | cold probe grid |
| 0 | 1 | — |
| 1 | 0 | — |
| 198 | 4 | warm re-run |
| 202 | 0 | warm re-run |
| 871 | 7 | **warm dc3 workload scan** |
| 14,033 | 89 | warm sweep re-run |
| 18,051 | 0 | warm sweep re-run |
| **33,358** | **117,733** | **22.1 %** |

The aggregate 22.1 % is the least useful number in the table. **The distribution
is bimodal and there is no middle:** a scan re-run with *nothing whatsoever
changed* hits ~100 % (`18051/0`, `14033/89`, `871/7`, `202/0`), and every other
run hits ~0 % (`0/42719`, `0/27956`, `0/878` five separate times).

That is not an accident, it is the key design working exactly as documented. The
key includes the workload tree's `HEAD` **and** a content digest of every dirty
path (`dirty_digest`). Between two dc3 scans, somebody edits the tree — which is
the entire point of the project — so the key moves and all 878 entries miss.
The reported `0 hits / 878 misses` is not a malfunction; **it is the normal
steady state for the workload the cache was built for.**

So the cache is genuinely useful in exactly one situation — *re-running the same
scan back-to-back within a session, to compare an instrument change* — and it is
worth roughly 36 s of wall time (~450 s CPU at `--jobs 16`) each time that
happens. That is a real benefit and the cache should be kept. It is not worth
266 GB and 44 M inodes of permanent retention. **[C1] — it was never worth
266 GB because it never occupied 266 GB; the 44 M inodes is the half of this
sentence that was true, and is the whole of why the cleanup was still right.**

### 3.1 How much of it could ever hit again

Cross-tabulating source-liveness against key-context currency, sample A:

| | count | share |
|---|---:|---:|
| source dead, current-era context | 3,436 | 72.7 % |
| source dead, pre-`cache-root` context | 999 | 21.1 % |
| **source alive, current context, no workload git** | **123** | **2.6 %** |
| source alive, current era, dc3 git context | 116 | 2.5 % |
| source alive, stale era, dc3 git context | 27 | 0.6 % |
| source alive, stale era, no workload git | 24 | 0.5 % |

The 116 dc3 entries are spread across ~18 HEAD tokens, so at most ~1/18 of them
(≈ 6 sampled entries, ≈ 0.14 %) belong to the current generation. The **upper
bound on entries that could serve a hit today is ≈ 2.7 %, ≈ 26,000 entries** —
and that is an upper bound, because it assumes the surviving source files still
have their original bytes.

**97.3 % of the cache is provably incapable of ever producing a hit.**

---

## 4. Recommendation

### 4.1 Does a GC solve it? — Yes, completely.

A retention policy is the whole fix. Three predicates, in increasing order of
sophistication and *decreasing* order of value:

**(a) Age. [C3] — REFUTED as a safe primary predicate; do not implement this
one.** The mechanism stated in its own "Risk" bullet is right and the conclusion
drawn from it is wrong: because a hit never rewrites mtime, an entry that has
served a hit on every gate run since 07-31 still reads as three days old, so
this predicate evicts the **live** gate working set (repo `fixtures/`, keyed
`unknown+dirty-unknown`). The cleanup lane replaced it with a
provably-unreachable predicate and kept 27,451 entries older than 48 h. See the
correction note.

**(a) Age.** `mtime(meta.txt) < now − 48 h` → delete the entry.
- Reclaims ≈ 938,500 entries (99.3 %) and ≈ 97 GB (87 %) from the main cache,
  and ≈ 3.99 M entries / ≈ 150 GB from the 47 abandoned sibling caches.
- Leaves ~6,400 entries / ~14 GB — today's live working set.
- Cost: one `find -mindepth 1 -maxdepth 1 -mtime +2 -exec rm -rf` per cache root,
  or ~30 lines of `std::fs` in a `c2rs cache gc` subcommand.
- Risk: an entry created 3 days ago that a lane would still hit gets evicted.
  §3.1 bounds that risk at ≤ 2.7 % of entries even before the age filter, and
  the cost of a wrong eviction is a re-capture, never a wrong answer. Hits do
  not update mtime (`/home` is mounted `noatime,nodiratime`), so age is a proxy
  for *write* recency only — which, for a write-once cache whose value is
  entirely short-horizon, is the right proxy.

**(b) Orphaned source.** the entry's `-f` argument, resolved against its recorded
`cwd`, does not exist → delete.
- Reclaims ≈ 887,000 entries (93.9 %) but only ≈ 29 GB (26 %).
- Strictly weaker than (a) on bytes and needs to parse `meta.txt` + `key.bin`.
- Worth having as a *second* predicate for a longer age window, not as the
  primary one.

**(c) Stale generation.** context ≠ the context this binary would compute → delete.
- Needs to run in-process (`CaptureCache::new` computes the context) and would
  reclaim the ~24,000 orphaned dc3 entries / ~67 GB that (b) misses because their
  sources *do* still exist.
- This is the only predicate that reaches the 72 %-of-bytes population without
  an age cutoff. If the retention window is ever widened beyond a few days, this
  is the one to add.

**Recommended policy: `(a) age > 7 days` OR `(b) source missing` OR `(c) context
not current`, run at the start of every scan, bounded to the scan's own cache
root.** On today's data that reclaims ~97 % of entries and ~99 % of bytes in the
main cache and leaves the instrument exactly as fast.

**[C3] — what was actually done: the disjunction was cut down to (b) and (c).**
Age was dropped entirely rather than widened, because widening the window does
not fix the mechanism (§4.1(a) [C3]) — an entry can be arbitrarily old and still
be hit every run. (b) and (c) are *provably-unreachable* predicates: the source
is gone so `key_material` returns `None` and the key cannot be formed, or the
workload-tree token is stale (verified absent in a 2,321-entry sample).
**[C1] — "~99 % of bytes" is `du` bytes**; the delete returned ~17 GiB.

**And one change that costs nothing at all:** point every lane at a single shared
cache root. `C2RS_GAP_CACHE` already exists and already overrides the default.
Exporting one absolute path for all lanes would have prevented the entire 154 GB
of sibling caches, including the three-way duplicate of the same 530k-entry
sweep — because `cache-root` would then be constant across lanes and the *same*
key would be produced. This is a one-line environment change with no code diff,
and it is the single highest-value action in this document after the delete.

### 4.2 If a structure were still warranted, what? — Challenging the pack-file prior

The prior offered was a git-style append-only pack plus a sorted fixed-width
index binary-searched by `seek`, ~200 lines of std. It is a good design for the
problem it solves. **It does not fit this cache, for a reason that is specific to
this project and easy to miss.**

**The blocking constraint: the cache directory *is* the capture directory.**
From `capture_cache`'s own module documentation:

> c2 bakes its `-Fo` path into the obj (`S_OBJNAME` in `.debug$S`), so a capture
> is only reproducible at a **fixed output path**. Each entry is therefore
> captured directly into `<cache>/<key>/`, and a hit hands back
> `ref_obj_path = <cache>/<key>/out.obj` — the same path the bytes were made at.

Two consequences follow, and they are not cosmetic:

1. **A pack cannot be the target of `-Fo`.** `cl.exe` and `c2.dll` are external
   Windows binaries running under wibo; they write files to paths. A miss must
   therefore still capture into a real directory, and that directory's absolute
   path lands in the obj bytes. This project has already been burned by exactly
   this: an unkeyed cache root produced six phantom `mismatch` reports —
   "ref 740 B vs port 768 B, diverging at offset 8: exactly the path-length
   delta" — an ALARM pointing at the port while the port was fine.
2. **A hit must hand back a real path too.** The differential consumes
   `ref_obj_path`, and the port is handed the same path via `S_OBJNAME` so both
   sides agree. A packed entry must therefore be *materialised back to
   `<cache>/<key>/`* — the same path it was captured at — on every hit.

So a pack does not remove the directory; it adds a second copy of it. The honest
shape of the design is: capture into `<cache>/<key>/` → append to pack →
`rm -rf <cache>/<key>/` → on a hit, re-materialise `<cache>/<key>/` from the
pack, and GC the materialisation later. That is:

- pack append with a cross-process lock and crash-safe ordering,
- a sorted index with an atomic rebuild (you cannot binary-search an
  append-ordered index; you must sort, and sorting a 5 M-entry index is its own
  write amplification),
- a materialiser,
- a **GC for the materialisation directory** — i.e. the exact retention policy of
  §4.1, still needed, now with a second consistency invariant to keep,
- and recovery for a pack truncated mid-append.

Realistically 400–700 lines plus tests, in the one seam every correctness claim
in the project routes through. Against that:

| what it buys | measured value |
|---|---|
| collapses 8.5 M files into a few | real, but so does the GC — a 6,400-entry cache is 51,000 files |
| removes 4 KB block rounding on small entries | ≈ 22 GB, worth having |
| makes the glob hazard structurally impossible | true, but a 6,400-entry directory is not a hazard |
| does **not** reclaim the orphaned generations | the 81 GB of dead dc3 entries stays |

**A GC reclaims ~97 GB; the pack reclaims ~22 GB.** The GC wins by more than 4×,
at a tenth of the effort, with no risk to the differential. **[C1] — both
figures are `du` blocks and neither survives; the ratio between two wrong
numbers is not an argument.** The verdict does survive, on the two grounds this
section actually rests on: the pack cannot be the target of `-Fo`, and the entry
and inode counts — the quantity that was really costing — fall to the GC and not
to the pack. Packing is the right
answer for a cache that is *legitimately* millions of live entries; this one is
6,400 live entries wearing a 945,000-entry coat. **Recommend: do not build it.**

If a structural change is wanted anyway, the higher-value one is far smaller:
**canonicalise `cwd` in `key_material`** (one line, folds 4 spellings of
`dc3-decomp` into 1 generation) and **share one cache root across lanes** (one
environment variable, §4.1). Those two together eliminate more duplicate entries
than the pack file eliminates bytes.

### 4.3 The database question — no, and here is the actual argument

The constraint is `std` only, zero external crates. SQLite means a C dependency
(or a large pure-Rust reimplementation). `CLAUDE.md` says to stop and discuss
when a dep looks unavoidable. Having looked: **it is not close to unavoidable,
and I would argue against it even if the constraint did not exist.**

What a relational engine actually offers, checked one by one against what this
cache does:

**Exact-key lookup — no gain.** The access pattern is `get(key) -> bytes`, with
no queries, no joins, no ordering, no range scans. The filesystem is already a
persistent hash table; `open("<root>/<key>/key.bin")` on btrfs is one b-tree
probe, the same asymptotics and roughly the same constant as SQLite's. There is
no query planner to help because there are no queries.

**The payload cannot live in the database anyway — this is the decisive point.**
Per §4.2, `c2.dll` writes `out.obj` to a filesystem path and bakes that path into
the output bytes, and the harness hands the path onward to the differential.
SQLite would therefore end up storing *metadata about files that still live in
directories*. That is strictly worse than today: the same 39 M inodes, plus a
second source of truth that can disagree with them, plus a dependency.

**Atomicity — already solved, correctly.** An entry is 8 files with `meta.txt`
written last as the completion marker, and `read_entry` requires `meta.txt` to
parse before serving. A crash mid-write leaves an entry that reads as a miss.
Sampling found `meta.txt` present in 4,725 / 4,725 entries; there is no observed
partial-entry problem to fix.

**Integrity — already stronger than SQLite's default.** Every entry stores its
complete key material verbatim in `key.bin`, and a hit is served only when those
bytes compare equal, so a hash collision degrades to a miss rather than to a
wrong answer. `--validate-cache N` re-captures every Nth hit and byte-compares
five IL streams, the argv and the obj. SQLite's page checksums (a non-default
compile option) protect against a different and less relevant failure.

**Concurrency — one genuine gap, and `std` closes it.** This is the one place
where I found something worth reporting. `CaptureCache`'s per-key locks are
`Mutex<HashMap<String, Arc<Mutex<()>>>>` — an **in-process** structure. Five
`c2rs` processes are running against these caches right now. Two processes that
compute the same key would both capture into `<cache>/<key>/`, interleaving two
`cl.exe` invocations' output files in one directory. In practice this is rare
(different worktrees have different `cache-root` values, hence different keys —
which is *why* the 154 GB of duplication exists; the duplication and the safety
are the same mechanism), but it is unguarded and it is real.

SQLite would give a cross-process write lock — over the *metadata*, while
`cl.exe` and `c2.dll` still race on the directory. It does not actually fix the
race. **`File::create_new("<cache>/<key>/.lock")` does**, in ~20 lines of `std`,
with `O_EXCL` semantics that are exactly a cross-process mutex, and it guards the
thing that is actually racing. That is the fix to make.

**Verdict: no. Not a close call.** Write-once, read-many, exact-key, no queries,
no joins, no cross-entry transactions, and a payload that cannot be stored in the
engine because an external process writes it to a path. A relational engine would
add a build dependency, an FFI surface, a second source of truth, and a
constraint violation, in exchange for nothing this workload asks for. If the
counter-argument is "we want to query the cache to GC it," the answer is that the
GC predicates in §4.1 are `mtime`, `stat`, and a string compare, and a 945,000-line
append-only sidecar journal (`<key> <mtime> <source> <context-digest>`) is ~100 MB
of text that greps in seconds — if a journal even proves necessary, which it does
not, because `find -maxdepth 1 -mtime +7` already answers the only question.

### 4.4 Sharding (`ab/cd/rest`) — reject, and the prior is right

Agreed, reject. The reasoning offered ("fixes directory *width* but not file
count, and is subsumed by packing") is correct, and the measurements add three
more objections:

1. **Width is not the problem.** btrfs indexes directory entries in a b-tree;
   the 944,936-entry directory's dirent extent is 60 MB and lookups are
   O(log n). `open()` on a key in this cache is not slow. Sharding optimises a
   metric nobody is paying.
2. **It makes the inode problem worse.** 8.5 M inodes becomes 8.5 M + ~66,000
   shard directories, for zero byte reduction and zero entry reduction.
3. **It does not fix the glob hazard — it hides it.** `work/capture-cache/*` stops
   being lethal, and `work/capture-cache/*/*/*` becomes lethal instead, while
   `find` and `du` walks get *slower* because there are more directories to
   traverse. A hazard that requires one more `*` to trigger is a worse hazard
   than one that is obvious.

The real fix for the glob hazard is the GC. A 6,400-entry directory can be
globbed all day.

---

## 5. Costed summary

Every figure in the **reclaims** column is `du`-derived and therefore wrong by
~5–15× on bytes — **[C1]**; the entry counts in that column are sound, and they
are what mattered. The whole-table reading after the fact: **the right actions,
priced in the wrong unit.**

| option | effort | risk to the differential | reclaims (main + siblings) |
|---|---|---|---|
| **`rm -rf` the 47 **[C2]** abandoned sibling caches** | minutes | none — those lanes are finished | **~3.99 M entries, ~150 GB [C1]** |
| **age-based GC, 48 h–7 d window** | ~30 lines or one `find` | negligible; worst case a re-capture **[C3] — no: it evicts the live gate working set, which age cannot distinguish from abandoned residue** | **~938 k entries, ~97 GB [C1]** |
| **share one cache root via `C2RS_GAP_CACHE`** | one env var, no diff | none | prevents recurrence of ~150 GB |
| canonicalise `cwd` in `key_material` | 1 line | none | folds 4 generations into 1 |
| `O_EXCL` per-entry lockfile | ~20 lines | closes a real cross-process race | — |
| source-orphan + stale-generation GC predicates | ~80 lines | low | the residual ~67 GB of dc3 generations |
| pack file + sorted index | 400–700 lines + tests | **touches the capture seam** | ~22 GB of block rounding |
| `ab/cd/` sharding | ~20 lines | low | **nothing** |
| SQLite | dependency + FFI | new source of truth | **nothing** |

**Recommended, in order:** delete the abandoned sibling caches; add an age-based
GC to the scan; export one shared `C2RS_GAP_CACHE`; canonicalise `cwd`; add the
`O_EXCL` lock. Build no new storage structure. Take no dependency.

**What actually happened, in the same order:** the abandoned sibling caches were
deleted (**44** of them by the strict cutoff, **[C2]**, returning ~17 GiB and not
~150 GB, **[C1]**); the age GC was **not** implemented and was replaced by
unreachability predicates (**[C3]**); the shared root was resolved **in code**
via `provenance::main_repo_root()` rather than exported as an environment
variable, because an env var would have pointed already-built lane binaries at a
shared root before they had the lock (§10.22); `cwd` canonicalisation and the
`O_EXCL` lock both landed in `c72a2a6`. **No new storage structure was built and
no dependency was taken** — the last two sentences of this recommendation are
the ones that held.

Deletion is the user's call and was not performed by this lane. Five lanes were
running during this investigation and their entries — everything written on
2026-08-04, ~6,400 entries in the main cache and ~8,575 across `w-r1b`, `w-r1c`
and `w-factors` — must be preserved by any window that is applied.

---

## Appendix — how to touch this directory without taking the box down

Twice on 2026-08-04 an agent expanded a shell glob inside `work/capture-cache`
and the OOM killer took every process on the machine with it (62 GB and 72 GB of
anonymous RSS in a `zsh`). **The shell, not the tool, expands the glob**: zsh
materialises all 945k paths in its own heap, `lstat`s each, and sorts them —
roughly 65 KB of arena per match.

Never, inside a cache root:

- any glob at all — `…/capture-cache/*`, `*/`, `**`
- `grep -r`, `find` without `-maxdepth`, `du -s` on the root, `ls -R`, `rsync`
- any of the above run from the repo root, which also walks 83 worktrees

Safe, and sufficient for everything in this document:

```sh
ls <root> | wc -l                                   # ls streams; the shell never sees the names
ls <root> | head -N
ls <root> | sed -n '1~200p'                         # systematic sample, bounded consumer
find <root> -mindepth 1 -maxdepth 1 -printf '\n' | wc -l    # O(1) memory
xargs -a <bounded-list> -d '\n' -n 300 du -s -- ...  # xargs chunks the argv; the shell does not
```

Put a `timeout` on all of it. A 32 GiB `RLIMIT_DATA` cap in `~/.zshenv` should now
turn this mistake into one failed tool call rather than a dead box — it is
per-process and a shared-anon mmap escapes it, so do not lean on it.
