#!/bin/sh
# tracked_artifact_audit.sh — does the INDEX contain anything CLAUDE.md says is
# never committed?  Board **#3545**, closing **#3156**'s standing prescription.
#
# Needs `git` and nothing else. No toolchain, no build, no network. Runs in
# well under a second on 9.6k tracked files.
#
# ---- why this exists ---------------------------------------------------------
#
# `#3156` was filed on 2026-08-15 for a pre-existing tracked artifact, and by
# 2026-08-25 the count had grown to **21** forbidden binaries under
# `work/w-biquad/` — 11 `.obj` and 10 `_CL_*` captures the standing row did not
# even count. The owner removed them from `HEAD` in `2c0de2ad4`. **Nothing
# stopped the next `git add -f`**, which is what `#3156` itself said:
#
#   > "a one-line check — `git ls-files` against `.gitignore`'s own patterns —
#    would have caught all nineteen, and belongs beside `board_audit.sh` rather
#    than in any lane's head."
#
# ---- AND THAT PRESCRIPTION, RUN LITERALLY, DOES NOT WORK ---------------------
#
# Measured at `a8593651b` before this file was written:
#
#     git ls-files -c --ignored --exclude-standard | wc -l   ->  8041
#
# **8,041 tracked files match `.gitignore`.** Essentially all of them are under
# `/work` (`work/w-mmio` 918, `work/emitpred` 277, `work/w-prod` 209, …) —
# `.gitignore:24` ignores `/work` wholesale, and ~200 lanes have force-added
# their evidence there deliberately for months. A gate scoped to `.gitignore`
# entire is therefore **red at HEAD by 8,041** and could only ship with an
# 8,041-entry allowlist, which is not a one-line check and is not a gate either.
#
# So the guard that CAN ship is scoped to the classes `CLAUDE.md` actually names
# under "Never commit", which read **0** at HEAD. The `.gitignore`-wide number is
# still printed every run, as an ADVISORY, so that nobody re-proposes the wide
# version without seeing what it costs.
#
# ---- two traps, both hit while building this --------------------------------
#
#   1. **`git check-ignore` does not report TRACKED files unless given
#      `--no-index`.** A guard built on bare `git check-ignore` reports clean
#      over exactly the population it exists to police. Verified both ways here.
#   2. **`grep` cannot test for a NUL byte** and is not used to. `grep -c $'\0'`
#      counts LINES; `LC_ALL=C grep -qP '\x00'` does not fire at all. (`#3513`,
#      board `#3544` — same wave as this file.)
#
# ---- the output contract ----------------------------------------------------
#
# `board_audit.sh` prints a count and a list and never a status, because a regex
# that matched nothing and a board that covered everything look identical from
# the outside. This file keeps the count and the list **and adds an exit code**,
# because the brief that commissioned it is explicit that a validating script's
# exit status must gate what follows it or the validation is decorative.
#
# The two are reconciled by a **denominator**: the number of tracked files
# examined is printed every run, and a run that examined **zero** files FAILS.
# Only a denominator can catch an absence (`#3470`, `#1002`) — an empty
# `git ls-files`, a wrong `-C`, or a `--`-pathspec typo would otherwise report
# a clean audit over nothing at all, which is this project's most-repeated
# failure and has now happened at least nine times.
#
#   exit 0  every class clean
#   exit 1  at least one violation — the list is on stdout
#   exit 2  the audit could not run (not a git repo, or zero files examined)
#   exit 3  --self-test found a class whose detector does not fire
#
# Usage:
#   scripts/tracked_artifact_audit.sh              audit the index
#   scripts/tracked_artifact_audit.sh --self-test  plant one violation per class
#                                                  in a THROWAWAY repo and require
#                                                  the audit to go red on each

set -eu

REPO="$(cd "$(dirname "$0")/.." && pwd)"

# ---- the allowlist, PRINTED EVERY RUN ---------------------------------------
# One line per entry, each with its reason. `board_audit.sh`'s convention: the
# filter is a LIST that is printed, never a silent regex, so an entry that stops
# being justified shows up as a suppressed line rather than vanishing.
allow_reason() {
    case "$1" in
        crates/c2-harness/src/provenance.rs)
            echo "a DOC COMMENT quoting a historical worktree path as evidence \
(:477, the #3048/w-fork record) — documentary, not a resolved path" ;;
        *) echo "" ;;
    esac
}
ALLOWLIST="crates/c2-harness/src/provenance.rs"

audit() {
    root="$1"
    rc=0

    if ! git -C "$root" rev-parse --git-dir >/dev/null 2>&1; then
        echo "ERROR: $root is not a git repository — the audit did not run." >&2
        return 2
    fi

    files="$(git -C "$root" ls-files)"
    denom="$(printf '%s' "$files" | grep -c . || true)"
    echo "tracked files examined: $denom"
    if [ "$denom" -eq 0 ]; then
        echo "ERROR: the audit examined ZERO files. A clean report over nothing" >&2
        echo "  is not a clean report (board #3470, #1002)." >&2
        return 2
    fi

    # ---- class 1: forbidden file NAMES ---------------------------------------
    # CLAUDE.md § Commits, "Never commit": captured/generated IL (_CL_*, *.il),
    # build artifacts (*.obj, *.o, /target), plus the gitignore's own binary
    # classes. Anchored per path SEGMENT so `foo.object` and `libo/` do not hit.
    name_re='(^|/)(_CL_[^/]*|[^/]*\.(obj|o|il|profraw|profdata|pyc))$|^(target|corpus|compilers|paint)/'
    hits="$(printf '%s\n' "$files" | grep -E "$name_re" || true)"
    n="$(printf '%s' "$hits" | grep -c . || true)"
    echo "forbidden artifact names: $n"
    if [ "$n" -gt 0 ]; then
        printf '%s\n' "$hits" | sed 's/^/    VIOLATION /'
        rc=1
    fi

    # ---- class 2: absolute machine paths in the CODE surfaces ----------------
    # CLAUDE.md: "absolute machine paths (/home/<user>/… — use C2RS_* env /
    # relative-to-repo defaults; toolchain location is env-driven by design)".
    #
    # SCOPED, and the scope is a measurement rather than a preference: at
    # a8593651b, 489 files under work/ and 16 under docs/ carry such a path as
    # recorded EVIDENCE (a rung quoting the directory a measurement ran in is
    # doing its job). Only 2 files under the code surfaces did, and one of those
    # is a doc comment. So the rule bites where it was written to bite.
    apaths=""
    for f in $(git -C "$root" ls-files -- crates scripts fixtures c2host c1host); do
        case " $ALLOWLIST " in *" $f "*) continue ;; esac
        if grep -qI '/home/[a-z][a-z0-9_-]*/' "$root/$f" 2>/dev/null; then
            apaths="$apaths$f
"
        fi
    done
    n2="$(printf '%s' "$apaths" | grep -c . || true)"
    echo "absolute machine paths in code surfaces: $n2"
    if [ "$n2" -gt 0 ]; then
        printf '%s' "$apaths" | sed 's/^/    VIOLATION /'
        rc=1
    fi

    # ---- the allowlist, printed whether or not it suppressed anything --------
    echo "allowlisted (printed every run, never a silent filter):"
    for f in $ALLOWLIST; do
        echo "    $f — $(allow_reason "$f")"
    done

    # ---- ADVISORY: the wide number #3156 asked for --------------------------
    wide="$(git -C "$root" ls-files -c --ignored --exclude-standard | grep -c . || true)"
    echo "ADVISORY — tracked files matching .gitignore entire: $wide"
    echo "  #3156 prescribed gating on THIS number. It is not gated on, and the"
    echo "  count is why: essentially all of it is deliberate lane evidence under"
    echo "  /work. Printed so the wide version is re-proposed with its price visible."

    return "$rc"
}

# ---- --self-test: every class watched going RED on a planted violation -------
#
# A guard nobody has seen fail is a guard nobody has tested. This plants one
# violation per class in a THROWAWAY repository — never in this one, so a killed
# run cannot leave a plant behind — and requires the audit to go red on each and
# green again with the plant removed.
self_test() {
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/c2rs-artifact-selftest-XXXXXX")"
    trap 'rm -rf "$tmp"' EXIT INT TERM
    git -C "$tmp" init -q
    git -C "$tmp" config user.email a@b.c
    git -C "$tmp" config user.name t
    mkdir -p "$tmp/crates" "$tmp/scripts"
    printf 'ok\n' > "$tmp/crates/keep.rs"
    printf '/work\n' > "$tmp/.gitignore"
    git -C "$tmp" add -A >/dev/null
    git -C "$tmp" commit -qm base

    if ! audit "$tmp" >/dev/null 2>&1; then
        echo "SELF-TEST FAIL: the clean control is not green." >&2
        return 3
    fi
    echo "  control (clean repo)                        GREEN"

    fails=0
    # path, content — one plant per class this guard CLAIMS to cover.
    for plant in \
        "crates/a.obj:binary" \
        "crates/a.o:binary" \
        "crates/_CL_abc123:capture" \
        "crates/a.il:il" \
        "crates/a.profraw:cov" \
        "target/release/c2rs:tree" \
        "scripts/hard.sh:main=/home/someuser/code/thing"
    do
        p="${plant%%:*}"; c="${plant#*:}"
        mkdir -p "$tmp/$(dirname "$p")"
        printf '%s\n' "$c" > "$tmp/$p"
        git -C "$tmp" add -f "$p" >/dev/null
        if audit "$tmp" >/dev/null 2>&1; then
            echo "  PLANT $p                                  *** STAYED GREEN ***" >&2
            fails=$((fails + 1))
        else
            echo "  plant $p -> RED"
        fi
        git -C "$tmp" rm -q --cached "$p" >/dev/null
        rm -f "$tmp/$p"
        if ! audit "$tmp" >/dev/null 2>&1; then
            echo "  PLANT $p removed but audit STILL RED — not reversible" >&2
            fails=$((fails + 1))
        fi
    done

    # And the denominator guard: an empty index must FAIL, not report clean.
    empty="$tmp/empty"
    mkdir -p "$empty" && git -C "$empty" init -q
    audit "$empty" >/dev/null 2>&1 && st=0 || st=$?
    if [ "$st" -eq 2 ]; then
        echo "  empty index -> exit 2 (an audit over 0 files is not a pass)"
    else
        echo "  EMPTY INDEX reported exit $st, expected 2" >&2
        fails=$((fails + 1))
    fi

    if [ "$fails" -gt 0 ]; then
        echo "SELF-TEST FAIL: $fails class(es) whose detector does not fire." >&2
        return 3
    fi
    echo "SELF-TEST PASS: 7 planted classes red, control green, empty index refused."
    return 0
}

case "${1:-}" in
    --self-test) self_test ;;
    "") audit "$REPO" ;;
    *) echo "usage: $0 [--self-test]" >&2; exit 2 ;;
esac
