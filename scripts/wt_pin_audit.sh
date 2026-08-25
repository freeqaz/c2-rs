#!/bin/sh
# wt_pin_audit.sh — THE REAP GUARD. Board **#3573**, closing **#3552**.
#
# Needs `git`, `find`, `od` and nothing else. No toolchain, no build, no
# network. ~10 s over six worktrees on this box; the denominator it examined is
# printed on every exit path.
#
# ---- why this exists ---------------------------------------------------------
#
# **Three consecutive actors destroyed pinned measurement artifacts by removing
# a worktree.** `w-adjacency` §7.6 named it (its three cost arms reaped, so its
# own numbers could not be re-derived). `w-hygiene` §2.1 paid it a second time
# and wrote it down. The coordinator then paid it a **third** time — one turn
# after quoting `w-hygiene` saying *"the arms stay pinned"* — by running
# `git worktree remove --force`, destroying three pinned experiment-F arms
# (**#3552**, and `#3523` is `w-permute`'s binaries going the same way).
#
# A named failure that three consecutive actors walk into is a **missing check,
# not three lapses** — the argument `#3156` makes about `git add -f`.
#
# ---- WHAT THE GUARD HAD TO BE, AND WHY IT IS NOT A SCRIPT CHECK -------------
#
# `#3552` sketches *"before a `worktree remove --force`, refuse if the tree
# holds a pinned binary"*. Read literally that is a check inside
# `scripts/wt_reap.py`, and **it could not have fired on any of the three
# occurrences**: `wt_reap.py` never passes `--force` (its own docstring says
# so, and git's refusal on a dirty tree is its stated backstop). All three
# losses came from a **hand-typed `git worktree remove --force`**. A check that
# lives where the failure does not is `#1236` exactly — a guard that passes
# precisely when it matters.
#
# So the enforcement here is **git's own refusal**, and the guard's job is to
# arm it. Measured live on git 2.55.0 before this file was written:
#
#     $ git worktree lock --reason "pinned experiment-F arms: c2rs-b1" wt1
#     $ git worktree remove --force wt1
#     fatal: cannot remove a locked working tree, lock reason: pinned ...
#     use 'remove -f -f' to override or unlock first          [exit 128]
#     $ git worktree remove --force --force wt1               [exit 0, GONE]
#
# **A single `--force` is refused; only `-f -f` gets through, and the refusal
# prints the pin reason.** That is a real fence on the exact command that did
# the damage, and it needs no cooperation from any wrapper. This script finds
# the trees that should be locked and locks them; `git` does the refusing.
#
# ---- the detector, and the two numbers it had to be re-scoped by -------------
#
# `#3545`'s lesson is to **measure the wide prescription before scoping to it**
# (its own prescription read 8,041 and could not ship). Both wide numbers here
# are printed on every run for the same reason.
#
#   WIDE-1  executable regular files outside `target/` — **3,053** in the
#           primary and **336** per worktree at the time of writing. Useless:
#           it is `compilers/X360` (Microsoft binaries, gitignored, restored by
#           `scripts/fetch_compilers.sh`) plus every tracked `.sh`/`.py`.
#
#   WIDE-2  ELF binaries outside `target/`, `.git/`, `compilers/` and outside
#           any NESTED worktree — **28** in the primary, **1** in every one of
#           the six live worktrees. Still not the guard: the per-worktree 1 is
#           `work/w-biquad/c2rs.base`, which `scripts/setup_worktree.sh`
#           reflinks into every tree it creates. **Removing that worktree
#           destroys nothing** — the primary's copy survives.
#
# The predicate that survives is therefore about **uniqueness, not shape**:
#
#   > an artifact is at risk from a reap only if it exists ONLY in the tree
#   > being reaped.
#
# So class P1 keeps an ELF binary only when the primary repo has no file at the
# same relative path with the same size. That is exactly what drops the six
# inherited `c2rs.base` copies and exactly what keeps a cost arm built in a
# lane's own scratch tree — which is what all three losses were.
#
# ---- class P2: prose is not machine-readable ---------------------------------
#
# Two of the three losses were of things a rung had **declared** pinned in
# words — *"the arms stay pinned"*, *"registered and unrun"*. No tool can read
# that. Class P2 is the channel: a lane that pins anything a reap would destroy
# — a corpus snapshot (`w-3475`'s `dc3-pin`), a gate base table, an unrun
# experiment — drops a **`.c2rs-pin`** file naming it. The file's presence is
# the declaration; its contents become the lock reason, so the refusal a future
# `--force` prints names what it is protecting.
#
# ---- the output contract -----------------------------------------------------
#
# Denominators first, because only a denominator catches an absence (`#3470`,
# `#1002`): every run prints the number of worktrees examined and the number of
# candidate files examined, and a run that examined **zero worktrees** FAILS
# rather than reporting a clean estate.
#
#   exit 0  every worktree holding a pin is locked (or holds none)
#   exit 1  at least one UNLOCKED tree holds a pin — the list is on stdout
#   exit 2  the audit could not run (not a git repo, or zero worktrees examined)
#   exit 3  --self-test found a class whose detector does not fire
#
# Usage:
#   scripts/wt_pin_audit.sh                audit every non-primary worktree
#   scripts/wt_pin_audit.sh --lock         audit, then arm git's refusal on
#                                          every violating tree
#   scripts/wt_pin_audit.sh --self-test    plant one violation per class in a
#                                          THROWAWAY repo and require the audit
#                                          to go red on each — including a live
#                                          `worktree remove --force` that must
#                                          be REFUSED
#   scripts/wt_pin_audit.sh --pin DIR "reason"
#                                          declare DIR pinned: write .c2rs-pin
#                                          and lock the worktree it lives in

set -eu

# `C2RS_PIN_AUDIT_ROOT` points the audit at another repository. It exists so
# `crates/c2-harness/tests/wt_pin_audit.rs` can run THIS script and
# `scripts/wt_reap.py` over one planted throwaway tree and require them to
# agree — two implementations of one predicate that nobody compares are two
# predicates. It is not a convenience flag and nothing in the funnel sets it.
REPO="${C2RS_PIN_AUDIT_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"

PIN_FILE=".c2rs-pin"

# ---- ELF magic without `file`(1), which is not guaranteed to be installed ----
is_elf() {
    [ -f "$1" ] || return 1
    [ -L "$1" ] && return 1
    [ "$(head -c 4 "$1" 2>/dev/null | od -An -tx1 | tr -d ' \n')" = "7f454c46" ]
}

# The list of worktree paths, primary first. `git worktree list --porcelain`
# emits the primary as its first record; every later record is a linked tree.
wt_paths() { git -C "$1" worktree list --porcelain | sed -n 's/^worktree //p'; }

is_locked() {
    # `git worktree list --porcelain` prints a bare `locked` line, or
    # `locked <reason>`, for a locked tree. Matching the record is fiddlier
    # than asking git directly about this one path.
    git -C "$1" worktree list --porcelain | awk -v p="$2" '
        $1=="worktree" { cur=$2 }
        $1=="locked"   { if (cur==p) { found=1 } }
        END { exit(found?0:1) }'
}

# ---- the candidate walk ------------------------------------------------------
#
# Prunes: `target/` (cargo output, reproducible by construction), `.git/`,
# `compilers/` (WIDE-1's whole mass; MS binaries, gitignored, re-fetchable),
# `node_modules/`, and `.claude/` — which is where NESTED worktrees live, so
# without that prune the primary's audit reports every other tree's files as
# its own. That last one is not hygiene: it is the difference between 28 and 34
# in the numbers above, and it would have made every worktree look pinned by
# every other worktree.
candidates() {
    find "$1" \
        -type d \( -name target -o -name .git -o -name compilers \
                   -o -name node_modules -o -name .claude \) -prune -o \
        -type f -executable -print 2>/dev/null
}

# ---- class P1: an ELF that exists ONLY in this tree --------------------------
#
# `same_in_primary` is a path+size comparison rather than a content hash on
# purpose: `setup_worktree.sh` reflinks, so an inherited copy is byte-identical
# AND at the same relative path, and hashing 6 MB binaries per worktree per run
# would cost seconds to separate cases that cannot arise. A same-path
# same-size file that is NOT the same bytes is a false NEGATIVE this accepts,
# and it is named here rather than left for someone to find: use --pin for
# anything whose value is its exact bytes at a path the primary also has.
same_in_primary() {
    _rel="$1"; _size="$2"; _primary="$3"
    _p="$_primary/$_rel"
    [ -f "$_p" ] || return 1
    [ "$(stat -c %s "$_p" 2>/dev/null || echo -1)" = "$_size" ]
}

audit() {
    root="$1"
    do_lock="${2:-0}"
    rc=0

    if ! git -C "$root" rev-parse --git-dir >/dev/null 2>&1; then
        echo "ERROR: $root is not a git repository — the audit did not run." >&2
        return 2
    fi

    primary="$(wt_paths "$root" | head -1)"
    trees="$(wt_paths "$root" | tail -n +2)"
    ntrees="$(printf '%s' "$trees" | grep -c . || true)"

    echo "primary: $primary"
    echo "worktrees examined: $ntrees"
    if [ "$ntrees" -eq 0 ]; then
        echo "ERROR: the audit examined ZERO worktrees. A clean estate report over" >&2
        echo "  nothing is not a clean report (board #3470, #1002)." >&2
        return 2
    fi

    wide1=0; wide2=0; examined=0
    viol_trees=""

    for wt in $trees; do
        [ -d "$wt" ] || continue
        pins=""
        for f in $(candidates "$wt"); do
            wide1=$((wide1 + 1))
            is_elf "$f" || continue
            wide2=$((wide2 + 1))
            examined=$((examined + 1))
            rel="${f#"$wt"/}"
            sz="$(stat -c %s "$f" 2>/dev/null || echo -1)"
            if same_in_primary "$rel" "$sz" "$primary"; then continue; fi
            pins="$pins    P1 unique-binary  $rel
"
        done
        # class P2 — the explicit declaration.
        for pf in $(find "$wt" -type d \( -name target -o -name .git \
                        -o -name compilers -o -name .claude \) -prune -o \
                        -type f -name "$PIN_FILE" -print 2>/dev/null); do
            examined=$((examined + 1))
            pins="$pins    P2 pin-manifest   ${pf#"$wt"/}  — $(head -1 "$pf" 2>/dev/null)
"
        done

        [ -z "$pins" ] && continue

        if is_locked "$root" "$wt"; then
            echo "  LOCKED  $wt"
            printf '%s' "$pins" | sed 's/^    /      ok /'
        else
            echo "  UNLOCKED AND PINNED  $wt"
            printf '%s' "$pins"
            viol_trees="$viol_trees$wt
"
            rc=1
        fi
    done

    echo "candidate files examined: $examined"
    echo "WIDE-1 executables outside target/compilers: $wide1"
    echo "WIDE-2 ELF binaries among them: $wide2"
    echo "  Both printed every run (#3545): WIDE-1 is compilers/ plus every"
    echo "  tracked script, WIDE-2 still counts the inherited work/w-biquad"
    echo "  copy that setup_worktree.sh reflinks into every tree. Neither is"
    echo "  the guard; uniqueness against the primary is."

    if [ "$rc" -ne 0 ] && [ "$do_lock" = "1" ]; then
        for wt in $viol_trees; do
            reason="pinned measurement artifacts (wt_pin_audit.sh) — run \
scripts/wt_pin_audit.sh to list; unlock deliberately before reaping"
            if git -C "$root" worktree lock --reason "$reason" "$wt" 2>/dev/null; then
                echo "  LOCKED NOW  $wt"
            else
                echo "  LOCK FAILED $wt" >&2
            fi
        done
        # Re-audit: --lock claims to have fixed the estate, so it has to prove
        # it rather than assert it. An exit that says "fixed" without re-reading
        # is the shape this project keeps paying for.
        echo "  re-auditing after --lock:"
        # The re-audit runs in a SUBSHELL. `audit` is recursing into itself and
        # every variable it uses (`rc`, `wide1`, `examined`, `viol_trees`) is
        # global in POSIX sh, so the inner call would otherwise overwrite the
        # outer one's state mid-function. It happens to be harmless today —
        # `rc` is reassigned explicitly on both branches below and nothing else
        # is read afterwards — and "happens to be harmless" is not a property
        # worth relying on in the one code path that decides whether the estate
        # is safe. `( … )` makes it impossible rather than merely unlikely.
        if ( audit "$root" 0 ) >/dev/null 2>&1; then
            echo "  re-audit CLEAN"
            rc=0
        else
            echo "  re-audit STILL RED — --lock did not fix the estate" >&2
            rc=1
        fi
    fi

    return "$rc"
}

# ---- --pin: the declaration channel -------------------------------------------
do_pin() {
    dir="$1"; reason="$2"
    [ -d "$dir" ] || { echo "wt_pin_audit.sh: $dir is not a directory" >&2; exit 2; }
    dir="$(cd "$dir" && pwd)"
    printf '%s\n' "$reason" > "$dir/$PIN_FILE"
    printf 'pinned %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$dir/$PIN_FILE"
    echo "wrote $dir/$PIN_FILE"
    wt="$(git -C "$dir" rev-parse --show-toplevel)"
    if git -C "$dir" worktree lock --reason "$reason" "$wt" 2>/dev/null; then
        echo "locked worktree $wt — 'git worktree remove --force' will now REFUSE"
    else
        echo "note: $wt is the primary worktree or is already locked; nothing to lock"
    fi
}

# ---- --self-test --------------------------------------------------------------
#
# A guard nobody has seen fail is a guard nobody has tested (`#1236`, which is
# this repo's canonical instance: a NUL check that could not fire was quoted as
# clean 20+ times). Everything below runs in a THROWAWAY repository, never in
# this one, so a killed run cannot leave a plant or a stray lock behind.
#
# The last case is the load-bearing one and it is not a plant at all: it runs
# the real destroying command against a locked tree and requires git to refuse.
# Every other case in this file is about FINDING a pin; that one is the only
# evidence that finding it accomplishes anything.
self_test() {
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/c2rs-pin-selftest-XXXXXX")"
    trap 'chmod -R u+w "$tmp" 2>/dev/null || true; rm -rf "$tmp"' EXIT INT TERM
    m="$tmp/main"
    git init -q "$m"
    git -C "$m" config user.email a@b.c
    git -C "$m" config user.name t
    mkdir -p "$m/work/shared"
    printf 'ok\n' > "$m/keep.txt"
    # An inherited binary: present in the primary AND in every worktree, at the
    # same relative path and size. This is the FALSE POSITIVE the real estate
    # is full of (six copies of work/w-biquad/c2rs.base) and the control that
    # proves the uniqueness rule is doing work rather than just passing.
    printf '\177ELF-inherited-payload\n' > "$m/work/shared/inherited.bin"
    chmod +x "$m/work/shared/inherited.bin"
    git -C "$m" add -A >/dev/null
    git -C "$m" commit -qm base

    git -C "$m" worktree add -q "$tmp/lane" -b lane1
    mkdir -p "$tmp/lane/work/shared"
    cp "$m/work/shared/inherited.bin" "$tmp/lane/work/shared/inherited.bin"

    fails=0

    if audit "$m" 0 >/dev/null 2>&1; then
        echo "  control: a tree holding only an INHERITED binary   GREEN"
    else
        echo "  CONTROL FAILED: the inherited-binary tree is flagged. The" >&2
        echo "  uniqueness rule is not working and the guard would be red on" >&2
        echo "  all six live worktrees." >&2
        fails=$((fails + 1))
    fi

    # ---- class P1: a binary that exists ONLY in the lane's tree --------------
    arm="$tmp/lane/work/lane/c2rs-b1"
    mkdir -p "$(dirname "$arm")"
    printf '\177ELF-cost-arm-b1\n' > "$arm"
    chmod +x "$arm"
    if audit "$m" 0 >/dev/null 2>&1; then
        echo "  PLANT P1 unique cost-arm binary   *** STAYED GREEN ***" >&2
        fails=$((fails + 1))
    else
        echo "  plant P1 unique cost-arm binary                    -> RED"
    fi
    rm -f "$arm"
    if ! audit "$m" 0 >/dev/null 2>&1; then
        echo "  PLANT P1 removed but audit STILL RED — not reversible" >&2
        fails=$((fails + 1))
    fi

    # ---- class P1b: same NAME as an inherited file, different SIZE -----------
    # The uniqueness rule keys on path+size, so a tree that overwrote an
    # inherited artifact with its own build must still be caught.
    printf '\177ELF-rebuilt-locally-and-bigger-than-the-inherited-one\n' \
        > "$tmp/lane/work/shared/inherited.bin"
    if audit "$m" 0 >/dev/null 2>&1; then
        echo "  PLANT P1b rebuilt-in-place binary *** STAYED GREEN ***" >&2
        fails=$((fails + 1))
    else
        echo "  plant P1b binary rebuilt in place (same path, new size) -> RED"
    fi
    cp "$m/work/shared/inherited.bin" "$tmp/lane/work/shared/inherited.bin"
    if ! audit "$m" 0 >/dev/null 2>&1; then
        echo "  PLANT P1b removed but audit STILL RED — not reversible" >&2
        fails=$((fails + 1))
    fi

    # ---- class P2: the explicit declaration ---------------------------------
    # NOT an ELF file, and that is the point: `w-3475`'s 14 GB corpus snapshot
    # and every registered-unrun experiment are invisible to any binary sniffer.
    mkdir -p "$tmp/lane/work/lane"
    printf 'registered-unrun experiment F: three arms + the rotation design\n' \
        > "$tmp/lane/work/lane/$PIN_FILE"
    if audit "$m" 0 >/dev/null 2>&1; then
        echo "  PLANT P2 .c2rs-pin manifest       *** STAYED GREEN ***" >&2
        fails=$((fails + 1))
    else
        echo "  plant P2 .c2rs-pin manifest (no binary anywhere)        -> RED"
    fi

    # ---- the lock clears it, and the LOCK IS THE WHOLE POINT ----------------
    git -C "$m" worktree lock --reason "self-test pin" "$tmp/lane"
    if audit "$m" 0 >/dev/null 2>&1; then
        echo "  same tree, now LOCKED                                  -> GREEN"
    else
        echo "  LOCKED TREE STILL RED — the guard cannot be satisfied" >&2
        fails=$((fails + 1))
    fi

    # ---- THE CASE THAT MATTERS: the real command, refused -------------------
    if git -C "$m" worktree remove --force "$tmp/lane" >/dev/null 2>&1; then
        echo "  *** 'git worktree remove --force' SUCCEEDED ON A LOCKED TREE ***" >&2
        echo "  The enforcement this guard relies on does not exist on this git." >&2
        fails=$((fails + 1))
    elif [ -d "$tmp/lane" ]; then
        echo "  'git worktree remove --force' on the locked tree       -> REFUSED"
    else
        echo "  *** the tree is GONE after a refused remove ***" >&2
        fails=$((fails + 1))
    fi
    # ...and that `-f -f` still works, so the lock is a fence and not a trap.
    git -C "$m" worktree remove --force --force "$tmp/lane" >/dev/null 2>&1 || true
    if [ -d "$tmp/lane" ]; then
        echo "  *** 'remove -f -f' did NOT remove the locked tree ***" >&2
        echo "  The lock would be unremovable, which is a worse defect than the" >&2
        echo "  one this guard fixes." >&2
        fails=$((fails + 1))
    else
        echo "  'git worktree remove --force --force' still works       -> GONE"
    fi

    # ---- the denominator guard ---------------------------------------------
    solo="$tmp/solo"
    git init -q "$solo"
    audit "$solo" 0 >/dev/null 2>&1 && st=0 || st=$?
    if [ "$st" -eq 2 ]; then
        echo "  a repo with ZERO linked worktrees -> exit 2 (not a clean estate)"
    else
        echo "  ZERO-WORKTREE repo reported exit $st, expected 2" >&2
        fails=$((fails + 1))
    fi

    if [ "$fails" -gt 0 ]; then
        echo "SELF-TEST FAIL: $fails case(s) did not behave as required." >&2
        return 3
    fi
    echo "SELF-TEST PASS: 3 planted classes red, 2 controls green, the real"
    echo "  'worktree remove --force' REFUSED on a locked tree, '-f -f' still"
    echo "  works, and a zero-worktree estate is refused."
    return 0
}

case "${1:-}" in
    --self-test) self_test ;;
    --lock)      audit "$REPO" 1 ;;
    --pin)       shift; [ $# -ge 2 ] || { echo "usage: $0 --pin DIR \"reason\"" >&2; exit 2; }
                 do_pin "$1" "$2" ;;
    "")          audit "$REPO" 0 ;;
    *)           echo "usage: $0 [--lock | --self-test | --pin DIR \"reason\"]" >&2; exit 2 ;;
esac
