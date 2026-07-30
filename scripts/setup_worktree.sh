#!/bin/bash
#
# setup_worktree.sh — create a buildable + gradeable git worktree, cheaply (CoW).
#
# Mirrors the sibling `../dc3-decomp/scripts/setup_worktree.sh` and
# `../rb3-xenon/scripts/setup_worktree.sh`. Here the problem is smaller but real: a
# naive `git worktree add` produces a worktree that BUILDS FROM COLD and CANNOT
# GRADE ANYTHING, because the two things that make this repo work are gitignored —
# `compilers/` (the MS binaries, fetched by scripts/fetch_compilers.sh and never
# committed) and `target/` (678 MB of cargo output). Without the first, every
# integration test reports `SKIP: toolchain absent` and the differential is
# vacuous — the worst failure mode this repo has, because it looks like success.
#
# Usage:
#   scripts/setup_worktree.sh [path] [branch-name] [base-ref] [--cold-cache]
#
#   path          Where to create it (default: .claude/worktrees/wt-<timestamp>)
#   branch        Branch name (default: wt-<basename of path>)
#   base-ref      Ref to branch from (default: current HEAD)
#   --cold-cache  Do NOT reflink `target/`. Use for a guaranteed-clean A/B timing
#                 test, or if a warm cargo cache triggers a full rebuild anyway.
#
# What is shared, and WHY symlink vs reflink-copy
# ----------------------------------------------------------------------------
# The rule from the siblings, and it is the load-bearing one: anything the build
# WRITES TO must be a real (reflinked) copy, never a symlink into the main tree — a
# symlink lets this worktree's build corrupt the shared one, which with several
# agents running concurrently is exactly the kind of failure that wastes a day.
# Anything only READ can be a symlink, which is cheapest.
#
#   compilers/          symlink        read-only toolchain (6 MB, MS binaries)
#   target/             reflink copy   cargo WRITES here; reflinking gives a warm
#                                      build cache for free on btrfs/xfs
#   work/dc3-workload/  reflink copy   the workload manifest + scan JSONLs. Small
#                                      and read-mostly, but a gap scan writes a
#                                      new JSONL beside them, so it is a copy.
#
# wibo needs nothing: it is resolved from `C2RS_WIBO`, a sibling `../wibo` build,
# or `PATH`, and the worktree sits at the same depth, so the sibling lookup still
# finds it. Scratch dirs go to `$TMPDIR`, not into the tree.
#
# After setup:
#   cd <worktree>
#   cargo build --release -p c2-harness
#   ./target/release/c2rs census fixtures/cpp/w5_chain.cpp   # must print 4/4
#   ./target/release/c2rs bench                              # the fixture gate
#   C2RS_JOBS=16 scripts/mode_lane.sh /O1                    # 0 mismatch or bust
#
# Prerequisite: the main repo must have `compilers/` populated
# (scripts/fetch_compilers.sh) and ideally have been built once.

set -euo pipefail

MAIN_REPO="$(cd "$(dirname "$0")/.." && pwd)"

# ---- args -------------------------------------------------------------------
POSITIONAL=()
WARM_CACHE=1
for arg in "$@"; do
    case "$arg" in
        --cold-cache) WARM_CACHE=0 ;;
        *) POSITIONAL+=("$arg") ;;
    esac
done

WORKTREE_PATH="${POSITIONAL[0]:-$MAIN_REPO/.claude/worktrees/wt-$(date +%s)}"
# A bare name is the natural way to call this ("setup_worktree.sh sy-bind") and it
# used to mean a path relative to the CWD — so from the repo root it created the
# worktree at `<repo>/sy-bind`, which `.gitignore` covers only under
# `/.claude/worktrees/`. The result was a full second checkout showing up as one
# untracked directory in `git status`, one `git add -A` away from being committed,
# plus a stray `wibo` symlink at the repo root (correctly placed for that location,
# since wibo resolves from the worktree's `../wibo`). Anything that is not already
# an explicit path is treated as a name under the ignored directory.
case "$WORKTREE_PATH" in
    */*) ;;
    *) WORKTREE_PATH="$MAIN_REPO/.claude/worktrees/$WORKTREE_PATH" ;;
esac
BRANCH="${POSITIONAL[1]:-wt-$(basename "$WORKTREE_PATH")}"
BASE_REF="${POSITIONAL[2]:-HEAD}"

BASE_COMMIT="$(git -C "$MAIN_REPO" rev-parse --short "$BASE_REF" 2>/dev/null)" || {
    echo "ERROR: cannot resolve ref '$BASE_REF'" >&2
    exit 1
}
BASE_BRANCH="$(git -C "$MAIN_REPO" rev-parse --abbrev-ref HEAD 2>/dev/null || echo detached)"

# ---- prerequisite sanity ----------------------------------------------------
# `compilers/` absent is the dangerous case: the harness degrades to
# `SKIP: toolchain absent` rather than failing, so a worktree without it grades
# every change green. Refuse to create one.
if [ ! -d "$MAIN_REPO/compilers" ]; then
    echo "ERROR: $MAIN_REPO/compilers is missing — run scripts/fetch_compilers.sh first." >&2
    echo "  Without it the differential SKIPs instead of failing, and a worktree" >&2
    echo "  built this way would report every change as passing." >&2
    exit 1
fi

DEST_FSTYPE="$(findmnt -no FSTYPE --target "$(dirname "$WORKTREE_PATH")" 2>/dev/null || echo unknown)"
case "$DEST_FSTYPE" in
    btrfs|xfs|zfs) : ;;
    *) echo "WARN: $(dirname "$WORKTREE_PATH") is on '$DEST_FSTYPE'; reflinks may be" >&2
       echo "      unavailable, so copies will be full — slow and space-hungry." >&2 ;;
esac

# ---- reflink helper ---------------------------------------------------------
# `cp --reflink=auto` falls back to a full copy transparently off CoW. Retried
# because `target/` may be written by a concurrent build in the main tree, and
# `cp -a` aborts with "file changed as we read it" when that happens.
reflink_dir() {
    local src="$1" dst="$2" tries="${3:-4}" i
    mkdir -p "$(dirname "$dst")"
    for ((i = 1; i <= tries; i++)); do
        rm -rf "$dst"
        if cp -a --reflink=auto "$src" "$dst" 2>/dev/null; then
            return 0
        fi
        sleep "$i"
    done
    return 1
}

# Best-effort for a REGENERABLE cache: a partial copy is fine, cargo just rebuilds
# whatever is missing. Fails only if the copy produced nothing at all.
reflink_dir_besteffort() {
    local src="$1" dst="$2" i
    mkdir -p "$(dirname "$dst")"
    rm -rf "$dst"
    for i in 1 2 3 4; do
        if cp -a --reflink=auto "$src" "$dst" 2>/dev/null; then return 0; fi
        sleep "$i"
    done
    [ -d "$dst" ] && return 0
    return 1
}

# ---- worktree (idempotent) --------------------------------------------------
if [ -e "$WORKTREE_PATH/.git" ]; then
    echo "==> Worktree already exists at $WORKTREE_PATH (reconfiguring in place)"
else
    echo "==> Creating worktree at $WORKTREE_PATH"
    echo "    branch=$BRANCH  base=$BASE_REF ($BASE_COMMIT, on $BASE_BRANCH)"
    if git -C "$MAIN_REPO" show-ref --verify --quiet "refs/heads/$BRANCH"; then
        git -C "$MAIN_REPO" worktree add "$WORKTREE_PATH" "$BRANCH"
    else
        git -C "$MAIN_REPO" worktree add "$WORKTREE_PATH" -b "$BRANCH" "$BASE_REF"
    fi
fi

WORKTREE_PATH="$(cd "$WORKTREE_PATH" && pwd)"

exec "$MAIN_REPO/scripts/configure_existing_worktree.sh" "$WORKTREE_PATH" \
    $([ "$WARM_CACHE" -eq 1 ] || echo --cold-cache)
