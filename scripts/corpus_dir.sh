# Sourceable helper: resolve the SHARED, CONTENT-ADDRESSED generated corpus.
#
#     . "$repo_root/scripts/corpus_dir.sh"
#     resolve_corpus "$repo_root" "$private_cases_dir"
#     cases="$C2RS_CORPUS_DIR"      # shared when verified, private otherwise
#
# ---- the number this exists to remove -----------------------------------------
#
# A lane's FIRST gate run in a fresh worktree paid a **1,261 s** cold
# `mode_cross` leg and a **117 s** cold sweep fill (lane `w-gateperf`, 2026-08-18,
# rung §2 and §4). Nothing about that work is new: it is 19,556 `cl.exe`
# invocations producing bytes some other worktree on this box computed already.
# The reason the cache misses is one component of its key — **the source path**,
# which `c2` bakes verbatim into `.gl` and `.debug$S`, so it must be in the key
# (`crates/c2-harness/src/capture_cache.rs`). Two worktrees generating the same
# corpus at two different paths therefore mint two disjoint cache generations of
# ~110,000 entries each.
#
# `w-gateperf` §11.2 priced pointing both case directories at the main repo and
# **declined it**, on the grounds that both drivers `rm -f` and regenerate that
# directory, so 3-4 concurrent lanes would contend on one mutable corpus and the
# `mkdir` lock's private-cold fallback would fire under exactly the condition
# that is normal here. That reasoning is correct **about a mutable directory**.
# This file makes the directory immutable, and the contention goes away with it.
#
# ---- why this is safe, stated as properties and not as intentions --------------
#
# **1. Content-addressed, so it is never mutated.** The directory name carries a
# digest of `scripts/sweep_gen.py` and every file in `scripts/sweep.d/`. A lane
# whose generator inputs differ by one byte gets a *different directory*, so it
# cannot overwrite anyone's corpus and nobody can overwrite its own. That closes
# the hazard `expr_sweep.sh`'s own header names ("two lanes with different
# `scripts/sweep.d` would overwrite each other's corpus between runs — board
# #3249's hazard"), which the per-worktree arrangement only avoided by not
# sharing at all.
#
# **2. Published by `rename(2)`, so a reader never sees a half-built corpus.**
# Generation goes into a sibling `.tmp-*` and is renamed into place. A directory
# rename within one filesystem is atomic: the generation either exists complete
# or does not exist. Two lanes racing to publish both generate, one rename wins,
# the loser discards its copy — and the two copies were byte-identical anyway,
# because generation is deterministic.
#
# **3. NO LOCK IS TAKEN OR NEEDED.** Readers only read. This is the whole point:
# the lock in `mode_cross.sh` and `expr_sweep.sh` exists because those drivers
# *delete and regenerate* their case directory, and it is held for the entire run
# (30-1,300 s) to protect a regeneration that takes **0.65 s**. Nothing here is
# deleted or regenerated after publication, so there is no window to protect.
#
# **4. VERIFIED ON EVERY RUN, by regeneration and a full byte compare.** The
# caller has already generated the corpus into its own private directory; this
# helper `diff -rq`s the shared generation against it and adopts the shared paths
# **only if all 19,556 files compare equal**. Measured cost: the generation is
# 0.65 s and the compare brings the pair to **1.2 s**. So the claim "the corpus I
# am grading is byte-identical to the one my own tree produces" is a *measurement
# taken on every run*, not a property inferred from the directory's name.
#
# **That check does not exist today.** Both drivers regenerate their corpus and
# grade it with no verification of any kind beyond `count > 0`. This helper is
# therefore coverage-INCREASING on the corpus itself, in the same run in which it
# removes the cold start.
#
# **5. A disagreement is a loud MISS, never a wrong verdict.** If the compare
# fails for any reason — a foreign generation, a half-deleted directory, a
# concurrent `rm -rf` — the run silently keeps its own private cases and says so
# in one line. It gets the slow, correct answer. This is `capture_cache`'s own
# standing rule for a foreign entry ("a stale or foreign entry must be a MISS or
# a loud refusal, never a silent wrong verdict") applied one layer up, and it is
# also why a stale generation is safe to `rm -rf`: the worst case is one slow run.
#
# ---- what is NOT shared, which is the question that matters --------------------
#
# **Only the case SOURCES are shared. No verdict, no port output, and nothing
# derived from any tree under test.** The sources are a pure function of
# `sweep_gen.py` + `sweep.d`, both of which live in the *reading lane's own
# tree* and both of which are in the digest. A lane cannot pass on a peer's
# evidence, because:
#
#   * `PortC2::compile_to` runs on every case of every run, against a fresh
#     `PortC2`, from the run's own pinned binary. It is never cached and never
#     shared.
#   * the obj compare runs on every case of every run.
#   * `expr_sweep.sh`'s P0.1 replay (standalone `c2.dll` under wibo) runs on
#     every case of every run.
#   * what a warm cache serves is `c2`'s **own** obj and IL bundle, keyed over
#     source bytes, the source argument, flags, cwd, the `cl.exe`/`c1xx.dll`/
#     `c2.dll` contents, the wibo version and the cache root — a set that does
#     **not** and cannot contain the port. It is oracle *input*, not a result.
#
# And the sharing it enables is not new. `work/capture-cache` has been resolved
# through `provenance::main_repo_root()` — "the same directory from every linked
# worktree" — since board #181, and the 878-TU workload scan has run fully warm
# in a fresh worktree that whole time, because its sources live in
# `../dc3-decomp`, one path from every worktree. This file gives the generated
# corpus the property the workload corpus already had.
#
# ---- what bounds it -------------------------------------------------------------
#
# **One directory per distinct corpus content**: 19,556 files, 12 MB apparent /
# 83 MB allocated. Not per lane, not per run, not per worktree — a new generation
# appears only when `sweep_gen.py` or `sweep.d` changes, and the count of
# generations on this box is the count of distinct corpora anyone has gated.
# `corpus: SHARED` prints how many exist so growth is visible rather than
# discovered.
#
# It is bounded in the way `work/capture-cache` is **not** (board #3265,
# ~21.5 M entries and no eviction), and it *reduces* that cache's growth rather
# than adding to it: today every worktree mints ~110,000 fresh entries
# (19,556 sweep + 90,812 cross) at its own paths, order 770 MB, and every
# worktree keeps two byte-identical 83 MB copies of the corpus besides. Under
# this helper a repeat corpus mints none of them.
#
# ---- escape hatches -------------------------------------------------------------
#
#   C2RS_NO_SHARED_CORPUS=1     never share; grade the private directory (cold).
#                               This is the A/B control and is what the cold
#                               numbers in the rung were measured with.
#   C2RS_CROSS_CASES / C2RS_SWEEP_CASES
#                               an explicit case directory keeps its documented
#                               meaning — a PRIVATE, COLD case set. The callers
#                               skip sharing when either is set.
#   C2RS_CORPUS_ROOT=DIR        put the shared generations somewhere else. This
#                               is what `gate.sh --selftest` drives the REAL
#                               `resolve_corpus` with, against fabricated trees
#                               and with no toolchain — a copy of this logic in
#                               the selftest would only prove the copy agrees.

# The digest over this tree's generator inputs. Computed with RELATIVE names
# under `scripts/`, so two worktrees holding identical generators produce an
# identical digest — which is the entire mechanism.
#
# Every file in `sweep.d` counts, not only `*.py`: a stray file that the loader
# happens to ignore still changes the digest, which costs one cold generation and
# never costs a wrong corpus. The fail-safe direction is deliberate.
# The inputs are checked to EXIST before they are hashed, because a pipeline
# whose first stage dies still runs its last: `( exit 1 ) | sha256sum | cut`
# prints the digest of the empty string, which is a perfectly stable-looking
# 16 hex digits for "this tree has no generator". That cannot produce a wrong
# corpus — publication needs a successful generation and adoption needs a full
# byte compare — but it would name two unrelated broken trees with one digest,
# and a digest that means two things is the beginning of a bad afternoon.
corpus_digest() {
    [ -f "$1/scripts/sweep_gen.py" ] || return 1
    [ -d "$1/scripts/sweep.d" ] || return 1
    ( cd "$1/scripts" 2>/dev/null || exit 1
      sha256sum sweep_gen.py 2>/dev/null || exit 1
      find sweep.d -maxdepth 1 -type f | LC_ALL=C sort | xargs sha256sum 2>/dev/null || exit 1
    ) | sha256sum | cut -c1-16
}

# resolve_corpus <repo_root> <private_cases_dir>
#
# Sets C2RS_CORPUS_DIR (the directory to grade) and C2RS_CORPUS_KIND
# (`shared` | `private`). NEVER fails: every error path leaves the private
# directory selected, which is the behaviour this repo had before this file.
resolve_corpus() {
    _rr="$1"; _priv="$2"
    C2RS_CORPUS_DIR="$_priv"; C2RS_CORPUS_KIND=private

    if [ "${C2RS_NO_SHARED_CORPUS:-0}" = "1" ]; then
        echo "corpus: PRIVATE $_priv (C2RS_NO_SHARED_CORPUS=1 — the cold control)"
        return 0
    fi

    # The MAIN repository root, the same directory from every linked worktree —
    # the shell counterpart of `provenance::main_repo_root()`, resolved through
    # git's own back-pointer rather than by string surgery on the path.
    #
    # This USED to be the same resolution the capture cache applied, which is why
    # the two could not end up rooted in different repositories. Since 2026-08-22
    # it is not: the cache root moved out of the checkout entirely
    # (`$XDG_CACHE_HOME/c2rs/capture`), because 22.6 M entries inside the tree is
    # what every `find` and `du` walks. The corpus stays here — it is 19,556
    # bounded files that a human is meant to find, not an unbounded cache — so
    # the two now resolve independently. That is fine and is not an oversight:
    # the cache KEYS on the corpus path, so a corpus rooted elsewhere is a
    # different key and a miss, never a wrong hit.
    if [ -n "${C2RS_CORPUS_ROOT:-}" ]; then
        _root="$C2RS_CORPUS_ROOT"
    else
        _gcd=$(git -C "$_rr" rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || _gcd=""
        if [ -z "$_gcd" ] || [ ! -d "$_gcd" ]; then
            echo "corpus: PRIVATE $_priv (no git common dir — cannot locate the main repo)"
            return 0
        fi
        _root="$(dirname "$_gcd")/work/corpus"
    fi

    _dig=$(corpus_digest "$_rr" 2>/dev/null) || _dig=""
    if [ -z "$_dig" ]; then
        echo "corpus: PRIVATE $_priv (the generator digest could not be computed)"
        return 0
    fi
    _dir="$_root/gen-$_dig"

    if [ ! -f "$_dir/MANIFEST" ]; then
        mkdir -p "$_root" 2>/dev/null || {
            echo "corpus: PRIVATE $_priv (cannot create $_root)"; return 0; }
        # Generate into a sibling and publish by rename. Never generate into
        # `$_dir` directly: a reader would then see a partially written corpus,
        # and `diff` would adopt it the instant the last file landed.
        _tmp="$_root/.tmp-$_dig-$$"
        rm -rf "$_tmp" 2>/dev/null
        if mkdir -p "$_tmp" 2>/dev/null &&
           python3 "$_rr/scripts/sweep_gen.py" "$_tmp" "$_rr/scripts/sweep.d" >/dev/null 2>&1
        then
            _n=$(find "$_tmp" -maxdepth 1 -name '*.cpp' | wc -l)
            if [ "${_n:-0}" -gt 0 ]; then
                printf 'corpus %s\ncases %s\n' "$_dig" "$_n" > "$_tmp/MANIFEST"
                # `-T` is load-bearing: a plain `mv` onto an existing directory
                # moves the source INSIDE it, which would publish
                # `gen-<d>/.tmp-<d>-<pid>/` and leave `MANIFEST` unfound forever.
                # `[ ! -d ]` first so the ordinary loser-of-the-race path does not
                # depend on `mv` refusing.
                if [ -d "$_dir" ] || ! mv -T "$_tmp" "$_dir" 2>/dev/null; then
                    rm -rf "$_tmp" 2>/dev/null
                fi
            else
                rm -rf "$_tmp" 2>/dev/null
            fi
        else
            rm -rf "$_tmp" 2>/dev/null
        fi
    fi

    if [ ! -f "$_dir/MANIFEST" ]; then
        echo "corpus: PRIVATE $_priv (no shared generation could be published)"
        return 0
    fi

    # THE CHECK. The caller has already generated its own corpus into `$_priv`;
    # the shared generation is adopted only if every file compares equal.
    if diff -rq --exclude=MANIFEST "$_priv" "$_dir" >/dev/null 2>&1; then
        _gens=$(find "$_root" -maxdepth 1 -type d -name 'gen-*' | wc -l)
        C2RS_CORPUS_DIR="$_dir"; C2RS_CORPUS_KIND=shared
        echo "corpus: SHARED $_dir"
        echo "  verified byte-identical to this tree's own generation in $_priv"
        echo "  ($_gens generation(s) present; each is immutable and safe to rm -rf when idle)"
    else
        echo "corpus: PRIVATE $_priv — REFUSED the shared generation $_dir"
        echo "  It is NOT byte-identical to what this tree's own generator just"
        echo "  produced, so this run grades its own cases, COLD. That is slow and"
        echo "  correct. A shared generation is content-addressed and should never"
        echo "  diverge; if this repeats, it is foreign: rm -rf '$_dir'"
    fi
    return 0
}
