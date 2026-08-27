#!/bin/sh
# tracked_artifact_audit.sh — does the INDEX contain anything CLAUDE.md says is
# never committed?  Board **#3545**, closing **#3156**'s standing prescription;
# widened to the `work/` evidence shelf by **#3675** (owner decision 18).
#
# Needs `git`, and the POSIX text utilities `tr`/`wc`/`cat`/`xargs` that any
# shell on this box already has. No toolchain, no build, no network. Runs in
# about a second on ~9.8k tracked files / 194 MB.
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
# **OWNER DECISION 18 (2026-08-26) SETTLED WHAT THAT NUMBER MEANT.** `#3615`
# asked whether `work/` is an ignored scratch space or a tracked evidence
# shelf. The answer is **shelf**: committing lane evidence there is correct and
# is not a `#3156` regression, `.gitignore`'s `/work` line stays (it governs the
# default for *untracked* scratch), and **two carve-outs are absolute** —
# **no binaries or build artifacts**, and **no absolute machine paths**. The
# `.gitignore`-wide number is still printed as an ADVISORY, so nobody
# re-proposes the wide version without seeing what it costs.
#
# ---- WHAT #3675 CHANGED, AND WHY EACH CHANGE HAS A CORPSE BEHIND IT ----------
#
#   * **Class 2 was scoped to `crates scripts fixtures c2host c1host`, so
#     `work/` was outside the population it examined.** It printed *"absolute
#     machine paths in code surfaces: 0"* while eleven offending files were
#     staged (`#3615`). A guard green because the offender is out of scope is
#     `#1236`'s shape. **Class 3 is the shelf, and it is now in scope.**
#
#   * **Class 3 looks for BOTH spellings.** The reference side runs under wibo,
#     which maps the tree to a DOS drive, so `cl`/`c2` write
#     `z:\home\<user>\…` into every `.cod` listing and oracle log. At
#     `41ca1ee9a`, **36 tracked files carried that form and 26 of them held no
#     forward-slash `/home/` at all** — invisible to `/home/[a-z][a-z0-9_-]*/`,
#     which is the pattern this file and every lane scrubber had been using.
#
#   * **Class 4 is defined by CONTENT, not by name, and that is the whole
#     lesson of the file that caused it.** `work/w-biquad/c2rs.base` — a
#     3,835,864-byte statically-linked ELF — survived a removal of 21 binaries
#     **from the same directory** because that removal enumerated its
#     population by NAME (`.obj`, `.o`, `_CL_*`, `.il`) and verified itself the
#     same way. Its commit message closes *"git ls-files for .obj/.o/_CL_/.il
#     now reads 0 workspace-wide"* — true, and never the question. **A
#     name-shaped test cannot report the file it has no name for.** Class 1
#     (names) is kept because it catches the classes `CLAUDE.md` enumerates
#     even when they are 0 bytes; class 4 catches everything else.
#
# ---- three traps, all hit while building or widening this --------------------
#
#   1. **`git check-ignore` does not report TRACKED files unless given
#      `--no-index`.** A guard built on bare `git check-ignore` reports clean
#      over exactly the population it exists to police. Verified both ways here.
#
#   2. **THE NUL TEST. This file used to assert that `grep` cannot test for a
#      NUL byte. THAT ASSERTION WAS FALSE and is retracted.** `grep` can — with
#      `-a`. Measured 2026-08-27 on `printf 'a\000b\n'` vs `printf 'ab\n'`,
#      both implementations on this box, 0 = fired and 1 = silent:
#
#           detector                            NUL file   text file
#           ugrep 7.5.0   grep -qP  hexNUL        1          1   SILENT
#           GNU  3.12     grep -qP  hexNUL        1          1   SILENT
#           ugrep 7.5.0   grep -q -a -P hexNUL    0          1   correct
#           GNU  3.12     grep -q -a -P hexNUL    0          1   correct
#           tr -d + wc -c byte count              0          1   correct
#
#      and the `grep -c` dollar-NUL spelling is not merely silent, it is
#      **anti-correlated**: GNU prints **2** on the NUL file and **1** on a
#      PLAIN TEXT file, because that pattern is the EMPTY string in argv (the
#      shell truncates at the NUL) so it matches every line and counts lines.
#      **The real defect is that the spelling everyone reaches for omits `-a`
#      and fails quietly** — that is how the 2026-08-26 scan read a clean tree
#      over a 3.8 MB ELF. This file uses the byte count, because it is correct
#      without depending on a flag whose absence is invisible. (`#1236`,
#      `#3513`, `#3544`; the correction to all three is `#3677`.)
#
#   3. **A per-file NUL test over 9.8k files costs 35 s; the aggregate costs
#      0.4 s and is exactly as exact.** `cat` the whole population once and
#      compare its byte count with and without NULs. The per-file walk that
#      NAMES the offenders runs only when that aggregate says there is one.
#      No 8,000-byte window, no `git ls-files --eol` heuristic — whole files.
#
# ---- the output contract ----------------------------------------------------
#
# `board_audit.sh` prints a count and a list and never a status, because a regex
# that matched nothing and a board that covered everything look identical from
# the outside. This file keeps the count and the list **and adds an exit code**,
# because the brief that commissioned it is explicit that a validating script's
# exit status must gate what follows it or the validation is decorative.
#
# The two are reconciled by **a denominator PER CLASS**, and every class prints
# how many files it examined. **A class that examined zero files FAILS the
# run**, even if every other class is clean: only a denominator can catch an
# absence (`#3470`, `#1002`) — an empty `git ls-files`, a wrong `-C`, or a
# `--`-pathspec typo would otherwise report a clean audit over nothing at all,
# which is this project's most-repeated failure and has now happened at least
# nine times. The closing line is **positive** — *"examined N files across 5
# classes, M violations"* — rather than an enumeration of ways to be empty.
#
# **Class 5 is a RATCHET, not a zero (#3689).** Its population is `docs/` files
# quoting an absolute machine path; they are dated rung records, they stay as
# written, and the enforced property is that the count MAY NOT GROW. It was an
# ADVISORY printed on every run until 2026-08-27, and in that state it went
# 16 -> 18 inside a single wave with nobody reading it — which is `#3679`'s own
# sentence ("a rule with no enforcement is a paragraph") landing on this
# script's own output. Raising `DOCS_ABS_CEILING` is a normal edit; doing it
# silently is the thing that is now impossible.
#
#   exit 0  every class clean
#   exit 1  at least one violation — the list is on stdout
#   exit 2  the audit could not run (not a git repo, or a class examined zero
#           files, or a tracked path contains a newline)
#   exit 3  --self-test found a class whose detector does not fire
#
# Usage:
#   scripts/tracked_artifact_audit.sh              audit the index
#   scripts/tracked_artifact_audit.sh --self-test  plant one violation per class
#                                                  in a THROWAWAY repo and require
#                                                  the audit to go red on each

set -eu

REPO="$(cd "$(dirname "$0")/.." && pwd)"

# The two absolute-path patterns, ASSEMBLED AT RUNTIME. Written as literals,
# this file's own text would contain `/home/<a-user>/` and class 2 scans the
# content of every tracked file under `scripts/` — see the note in self_test().
_h="/home"
ABS_FWD="$_h/[a-z][a-z0-9_-]*/"
ABS_BS='\\home\\[a-z][a-z0-9_-]*\\'

# ---- the allowlist, PRINTED EVERY RUN ---------------------------------------
# One line per entry, each with its reason AND the number of files it actually
# suppressed on this run. `board_audit.sh`'s convention: the filter is a LIST
# that is printed, never a silent regex, so an entry that stops being justified
# shows up as a suppressed line rather than vanishing. The suppression COUNT is
# #3675's addition — a prefix entry that silently grows from 10 files to 400 is
# a regex wearing a list's clothes.
allow_reason() {
    case "$1" in
        crates/c2-harness/src/provenance.rs)
            echo "class 2 — a DOC COMMENT quoting a historical worktree path as \
evidence (:477, the #3048/w-fork record): documentary, not a resolved path" ;;
        work/w-bss2/prov.py)
            echo "class 3 — SYNTHETIC non-existent user paths used as INPUTS to \
that file's assertions about the provenance path-relativiser. Not this box's \
paths. Rewriting a test's input changes what the test asserts (#3676)" ;;
        docs/perf/perf_scale.png)
            echo "class 4 — the README's throughput figure, regenerated by \
scripts/plot_perf.py from docs/perf/perf_scale.csv. A documentation image is \
not a build artifact and CLAUDE.md names neither it nor its class" ;;
        crates/c2-harness/tests/corpus_sample/)
            echo "class 4 ONLY — a PREFIX covering 16 tracked files, TEN of \
which contain a NUL: 4-to-60-byte synthetic COFF/IL fixtures the corpus tests \
read as bytes. Deliberately tracked test data, not captured IL (no _CL_ name, \
no .obj/.il extension, written by the tests' own fixtures, not the toolchain). \
The other six are text and are exempt from nothing else — the entry grants \
class 4 and no other class" ;;
        *) echo "" ;;
    esac
}
# Space-separated, and **scoped to the class that justified the entry**. A
# trailing `/` makes an entry a PREFIX; anything else is an exact path.
#
# The scoping is #3675's, and it is not tidiness. A flat list exempts its files
# from EVERY class, so `crates/c2-harness/tests/corpus_sample/` — allowlisted
# because ten of its files are deliberately binary — would also have exempted
# all sixteen of them from the absolute-path classes, which nothing justifies.
# An allowlist entry carries the reason for ONE class and now grants exactly
# that class. Class 1 has no allowlist at all: a forbidden NAME is forbidden.
ALLOW_C2="crates/c2-harness/src/provenance.rs"
ALLOW_C3="work/w-bss2/prov.py"
ALLOW_C4="docs/perf/perf_scale.png crates/c2-harness/tests/corpus_sample/"

# ---- class 5's ceiling ------------------------------------------------------
#
# The count of tracked files under `docs/` that carry an absolute machine path.
# NOT a target of zero and not an allowlist: the population is dated rung
# records quoting the worktree a measurement ran in, and this repo's standing
# rule is that dated records stay as written. What the ceiling forbids is
# GROWTH — see the RATCHET block in `audit()` for why an advisory was not
# enough.
#
# Measured on `f3f8d5eeb` (2026-08-27), the tree this ceiling was set on:
# **18**. It read 16 at `41ca1ee9a` and grew by two during wave 15 while the
# advisory printed the number on every run.
#
# Raising this is a normal, expected edit. Raise it in the same commit as the
# file that needs it and name the file in the commit message.
DOCS_ABS_CEILING=18
ALLOWLIST="$ALLOW_C2 $ALLOW_C3 $ALLOW_C4"

# NOTE THE UNDERSCORED LOOP VARIABLE, AND IT IS NOT STYLE EITHER. POSIX sh has
# no `local`, so a function that loops over `a` CLOBBERS an `a` in its caller.
# Written as `for a in $1`, this helper silently rewrote the allowlist printer's
# own loop variable and the report came out naming ONE entry four times with the
# class column reading `?`. Caught because the printed list disagreed with
# itself; a report that is only ever read for its final number would not have
# shown it.
allowed() {   # $1 = class list, $2 = path; 0 if allowlisted for that class
    for _al in $1; do
        case "$_al" in
            */) case "$2" in "$_al"*) return 0 ;; esac ;;
            *)  [ "$2" = "$_al" ] && return 0 ;;
        esac
    done
    return 1
}

# THE NUL TEST, AND THE ONLY ONE THIS FILE USES. See trap 2 above for the
# measured table, and `work/w-gen2/scrub.sh --self-test` for it watched firing
# in both directions.
has_nul() {   # 0 = the file contains a NUL byte
    [ "$(tr -d '\000' < "$1" | wc -c)" -ne "$(wc -c < "$1")" ]
}

audit() {
    root="$1"
    rc=0
    viol=0

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

    # Everything below treats the newline-separated list as safe. Assert that
    # rather than assume it: one path with a newline in it would silently split
    # into two nonexistent ones and class 4's aggregate would read short.
    nz="$(git -C "$root" ls-files -z | tr -dc '\000' | wc -c | tr -d ' ')"
    if [ "$denom" -ne "$nz" ]; then
        echo "ERROR: $denom lines vs $nz NUL-terminated paths — a tracked path" >&2
        echo "  contains a newline. Refusing to audit a list that mis-splits." >&2
        return 2
    fi

    # ---- class 1: forbidden file NAMES ---------------------------------------
    # CLAUDE.md § Commits, "Never commit": captured/generated IL (_CL_*, *.il),
    # build artifacts (*.obj, *.o, /target), plus the gitignore's own binary
    # classes. Anchored per path SEGMENT so `foo.object` and `libo/` do not hit.
    # Kept alongside class 4 because a name in this list is forbidden even when
    # the file is empty — and because it names the class in the report.
    name_re='(^|/)(_CL_[^/]*|[^/]*\.(obj|o|il|profraw|profdata|pyc))$|^(target|corpus|compilers|paint)/'
    hits="$(printf '%s\n' "$files" | grep -E "$name_re" || true)"
    n="$(printf '%s' "$hits" | grep -c . || true)"
    echo "class 1  forbidden artifact NAMES      — examined $denom, violations $n"
    if [ "$n" -gt 0 ]; then
        printf '%s\n' "$hits" | sed 's/^/    VIOLATION /'
        rc=1; viol=$((viol + n))
    fi

    # ---- class 2: absolute machine paths in the CODE surfaces ----------------
    # CLAUDE.md: "absolute machine paths (/home/<user>/… — use C2RS_* env /
    # relative-to-repo defaults; toolchain location is env-driven by design)".
    # `git grep` rather than a per-file loop: one process, and `-I` skips the
    # binaries class 4 owns.
    code_denom="$(git -C "$root" ls-files -- crates scripts fixtures c2host c1host \
        | grep -c . || true)"
    if [ "$code_denom" -eq 0 ]; then
        echo "class 2  absolute paths, CODE surfaces — examined 0"
        echo "ERROR: class 2 examined ZERO files — its pathspec matched nothing." >&2
        echo "  A class that graded nothing is not a class that passed (#3470)." >&2
        return 2
    fi
    apaths=""
    for f in $(git -C "$root" grep -I -l -E -e "$ABS_FWD" -e "$ABS_BS" \
               -- crates scripts fixtures c2host c1host 2>/dev/null || true); do
        allowed "$ALLOW_C2" "$f" || apaths="$apaths$f
"
    done
    n2="$(printf '%s' "$apaths" | grep -c . || true)"
    echo "class 2  absolute paths, CODE surfaces — examined $code_denom, violations $n2"
    if [ "$n2" -gt 0 ]; then
        printf '%s' "$apaths" | sed 's/^/    VIOLATION /'
        rc=1; viol=$((viol + n2))
    fi

    # ---- class 3: absolute machine paths on the work/ EVIDENCE SHELF ---------
    # Decision 18's second carve-out. This is the population `#3615` found to be
    # outside the audit entirely, and it is scanned in BOTH spellings.
    shelf_denom="$(git -C "$root" ls-files -- work | grep -c . || true)"
    if [ "$shelf_denom" -eq 0 ]; then
        echo "class 3  absolute paths, work/ SHELF   — examined 0"
        echo "ERROR: class 3 examined ZERO files under work/. The shelf is the" >&2
        echo "  population this class exists for; zero means the pathspec is" >&2
        echo "  wrong, not that the shelf is clean (#3470, #1002)." >&2
        return 2
    fi
    spaths=""
    for f in $(git -C "$root" grep -I -l -E -e "$ABS_FWD" -e "$ABS_BS" \
               -- work 2>/dev/null || true); do
        allowed "$ALLOW_C3" "$f" || spaths="$spaths$f
"
    done
    n3="$(printf '%s' "$spaths" | grep -c . || true)"
    echo "class 3  absolute paths, work/ SHELF   — examined $shelf_denom, violations $n3"
    if [ "$n3" -gt 0 ]; then
        printf '%s' "$spaths" | sed 's/^/    VIOLATION /'
        rc=1; viol=$((viol + n3))
    fi

    # ---- class 4: BINARY CONTENT, repo-wide, by exact byte count -------------
    # Decision 18's first carve-out, and the one `c2rs.base` walked past.
    # Aggregate first: `cat` the whole tree once, with and without NULs. Equal
    # totals means no tracked file holds a NUL ANYWHERE — no window, no
    # heuristic. Only a difference pays for the per-file walk that names them.
    allow_nul=0
    allow_files=0
    for f in $files; do
        if allowed "$ALLOW_C4" "$f" && [ -f "$root/$f" ] && has_nul "$root/$f"; then
            fb="$(wc -c < "$root/$f" | tr -d ' ')"
            fs="$(tr -d '\000' < "$root/$f" | wc -c | tr -d ' ')"
            allow_nul=$((allow_nul + fb - fs))
            allow_files=$((allow_files + 1))
        fi
    done
    raw="$(git -C "$root" ls-files -z | (cd "$root" && xargs -0 cat 2>/dev/null) \
        | wc -c | tr -d ' ')"
    stripped="$(git -C "$root" ls-files -z | (cd "$root" && xargs -0 cat 2>/dev/null) \
        | tr -d '\000' | wc -c | tr -d ' ')"
    total_nul=$((raw - stripped))
    bad_nul=$((total_nul - allow_nul))
    bhits=""
    n4=0
    if [ "$bad_nul" -ne 0 ]; then
        for f in $files; do           # only now pay for naming them
            [ -f "$root/$f" ] || continue
            allowed "$ALLOW_C4" "$f" && continue
            if has_nul "$root/$f"; then
                bhits="$bhits$f
"
                n4=$((n4 + 1))
            fi
        done
    fi
    echo "class 4  BINARY content (a NUL byte)   — examined $denom, violations $n4"
    echo "         read $raw bytes; $total_nul NUL byte(s) total, $allow_nul of them in $allow_files allowlisted file(s)"
    if [ "$raw" -eq 0 ]; then
        echo "ERROR: class 4 read ZERO bytes. It examined nothing." >&2
        return 2
    fi
    if [ "$n4" -gt 0 ]; then
        printf '%s' "$bhits" | sed 's/^/    VIOLATION /'
        rc=1; viol=$((viol + n4))
    elif [ "$bad_nul" -ne 0 ]; then
        echo "ERROR: the aggregate says $bad_nul unaccounted NUL byte(s) but the" >&2
        echo "  per-file walk named none. The two disagree; trust neither." >&2
        return 2
    fi

    # ---- the allowlist, printed whether or not it suppressed anything --------
    echo "allowlisted (printed every run, never a silent filter):"
    for ent in $ALLOWLIST; do
        if allowed "$ALLOW_C2" "$ent"; then cls="class 2 only"
        elif allowed "$ALLOW_C3" "$ent"; then cls="class 3 only"
        elif allowed "$ALLOW_C4" "$ent"; then cls="class 4 only"
        else cls="NO CLASS — this entry grants nothing and should be deleted"
        fi
        sup=0
        for f in $files; do
            case "$ent" in
                */) case "$f" in "$ent"*) sup=$((sup + 1)) ;; esac ;;
                *)  [ "$f" = "$ent" ] && sup=$((sup + 1)) ;;
            esac
        done
        echo "    $ent — grants $cls — matches $sup tracked file(s) this run"
        echo "        $(allow_reason "$ent")"
    done

    # ---- ADVISORY: the wide number #3156 asked for --------------------------
    wide="$(git -C "$root" ls-files -c --ignored --exclude-standard | grep -c . || true)"
    echo "ADVISORY — tracked files matching .gitignore entire: $wide"
    echo "  #3156 prescribed gating on THIS number. It is not gated on, and"
    echo "  owner decision 18 is why: work/ is a tracked evidence SHELF and"
    echo "  essentially all of it is deliberate lane evidence. Printed so the"
    echo "  wide version is re-proposed with its price visible."

    # ---- ADVISORY: docs/, which no lane's fence has covered ------------------
    docs_denom="$(git -C "$root" ls-files -- docs | grep -c . || true)"
    dpaths="$(git -C "$root" grep -I -l -E -e "$ABS_FWD" -e "$ABS_BS" \
        -- docs 2>/dev/null | grep -c . || true)"
    echo "RATCHET — absolute machine paths under docs/: $dpaths of $docs_denom examined (ceiling $DOCS_ABS_CEILING)"
    echo "  NOT gated at ZERO: docs/ is outside the two carve-outs decision 18"
    echo "  names, and the files are dated rung records that quote the worktree"
    echo "  a measurement ran in. Those stay as written."
    echo "  GATED AT A CEILING instead, board #3689. Printing a number every"
    echo "  run was supposed to stop it going quiet and it did not: this read"
    echo "  16 at 41ca1ee9a and 18 four commits later, inside one wave, with"
    echo "  the advisory visible on every run and nobody reading it. An"
    echo "  unenforced number is #3679's own sentence — a rule with no"
    echo "  enforcement is a paragraph — aimed at this script's own output."
    echo "  To ADD a file that quotes a machine path, raise DOCS_ABS_CEILING in"
    echo "  this script in the same commit and say which file and why. That is"
    echo "  one line, and it makes the growth a decision instead of a drift."
    if [ "$dpaths" -gt "$DOCS_ABS_CEILING" ]; then
        echo "VIOLATION class 5: docs/ absolute-path files rose to $dpaths, above the ceiling of $DOCS_ABS_CEILING." >&2
        git -C "$root" grep -I -l -E -e "$ABS_FWD" -e "$ABS_BS" -- docs 2>/dev/null \
            | sed 's/^/    /' >&2
        viol=$((viol + 1))
        rc=1
    elif [ "$dpaths" -lt "$DOCS_ABS_CEILING" ]; then
        echo "  NOTE: $dpaths is BELOW the ceiling. Lower DOCS_ABS_CEILING to"
        echo "  $dpaths so the slack cannot be spent silently — a ceiling with"
        echo "  headroom is an advisory again."
    fi

    # ---- the positive summary ------------------------------------------------
    # 5 classes since #3689, and the count is spelled out rather than left at
    # "4" because the summary line is the one line most readers read: a class
    # that is enforced but absent from the headline is enforcement nobody knows
    # they have. Class 5's denominator is `docs/`, not `$denom`, and it is
    # printed on its own RATCHET line above with its ceiling beside it.
    echo "SUMMARY: examined $denom tracked files across 5 classes; $viol violation(s)."
    return "$rc"
}

# ---- --self-test: every class watched going RED on a planted violation -------
#
# A guard nobody has seen fail is a guard nobody has tested. This plants one
# violation per class in a THROWAWAY repository — never in this one, so a killed
# run cannot leave a plant behind — and requires the audit to go red on each and
# green again with the plant removed. Exit codes are read from `$?` DIRECTLY and
# never through a pipe: `$?` after a `tee` is `tee`'s, and a lane's evidence
# capture once recorded `EXIT=0` on a broken run for exactly that reason.
self_test() {
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/c2rs-artifact-selftest-XXXXXX")"
    trap 'rm -rf "$tmp"' EXIT INT TERM
    git -C "$tmp" init -q
    git -C "$tmp" config user.email a@b.c
    git -C "$tmp" config user.name t
    mkdir -p "$tmp/crates" "$tmp/scripts" "$tmp/work/lane" "$tmp/docs"
    printf 'ok\n' > "$tmp/crates/keep.rs"
    printf 'note\n' > "$tmp/docs/keep.md"
    # The shelf must be NON-EMPTY in the control, because class 3 fails a run
    # that examined zero files under work/ — which is the point of the class.
    printf 'a lane transcript with no machine path in it\n' > "$tmp/work/lane/ev.txt"
    printf '/work\n' > "$tmp/.gitignore"
    git -C "$tmp" add -A >/dev/null
    git -C "$tmp" add -f work/lane/ev.txt >/dev/null
    git -C "$tmp" commit -qm base

    audit "$tmp" >/dev/null 2>&1 && st=0 || st=$?
    if [ "$st" -ne 0 ]; then
        echo "SELF-TEST FAIL: the clean control is not green (exit $st)." >&2
        audit "$tmp" >&2 || true
        return 3
    fi
    echo "  control (clean repo, non-empty work/)          GREEN"

    fails=0
    # THE ABSOLUTE-PATH PLANTS ARE ASSEMBLED AT RUNTIME, AND THAT IS NOT STYLE.
    #
    # Written as literals, this script's own text would contain
    # `/home/<a-user>/…` — and class 2 scans the CONTENT of every tracked file
    # under scripts/. The guard would then flag ITSELF the moment it was
    # committed, which is exactly what happened on the first attempt: the audit
    # was run and passed while the file was still untracked, so `git ls-files`
    # did not list it, and it went red one commit later at 9,618 files instead
    # of 9,616. **A guard validated against a population that does not yet
    # contain the guard is not validated.** Assembling the string from two
    # halves keeps the file honest under its own rule — self-exemption was the
    # alternative and it is the wrong instinct: a rule its enforcer is exempt
    # from is a rule with one guaranteed blind spot.
    home_seg="/home"
    abs_plant="main=$home_seg/someuser/code/thing"
    # The wibo/DOS spelling of the same thing, which nothing detected until
    # #3675: 26 tracked files carried ONLY this form.
    bs_plant="TITLE  z:\\${home_seg#/}\\someuser\\code\\thing\\a.cpp"
    # path:content — one plant per class this guard CLAIMS to cover. The last
    # two are `c2rs.base`'s exact shape: a binary with NO forbidden name at all,
    # which a name-defined class 1 cannot see, planted on BOTH sides of the
    # crates//work/ boundary so class 4's repo-wide scope is exercised.
    for plant in \
        "crates/a.obj:binary" \
        "crates/a.o:binary" \
        "crates/_CL_abc123:capture" \
        "crates/a.il:il" \
        "crates/a.profraw:cov" \
        "target/release/c2rs:tree" \
        "scripts/hard.sh:@ABS@" \
        "work/lane/run.log:@ABS@" \
        "work/lane/listing.cod:@BS@" \
        "work/lane/nameless:@NUL@" \
        "crates/nameless.dat:@NUL@"
    do
        p="${plant%%:*}"; c="${plant#*:}"
        mkdir -p "$tmp/$(dirname "$p")"
        case "$c" in
            "@ABS@") printf '%s\n' "$abs_plant" > "$tmp/$p" ;;
            "@BS@")  printf '%s\n' "$bs_plant"  > "$tmp/$p" ;;
            "@NUL@") printf 'ELF\000\000\000not a forbidden name\n' > "$tmp/$p" ;;
            *)       printf '%s\n' "$c" > "$tmp/$p" ;;
        esac
        git -C "$tmp" add -f "$p" >/dev/null
        audit "$tmp" >/dev/null 2>&1 && st=0 || st=$?
        if [ "$st" -eq 0 ]; then
            echo "  PLANT $p  *** STAYED GREEN ***" >&2
            fails=$((fails + 1))
        else
            echo "  plant $p -> RED (exit $st)"
        fi
        git -C "$tmp" rm -q --cached "$p" >/dev/null
        rm -f "$tmp/$p"
        audit "$tmp" >/dev/null 2>&1 && st=0 || st=$?
        if [ "$st" -ne 0 ]; then
            echo "  PLANT $p removed but audit STILL RED — not reversible" >&2
            fails=$((fails + 1))
        fi
    done

    # The allowlist must SUPPRESS, and be watched suppressing — an allowlist
    # that does not actually exempt its entry is an allowlist hiding a red, and
    # one that exempts more than its entry is a regex wearing a list's clothes.
    mkdir -p "$tmp/work/w-bss2"
    printf 'p = "%s/u/a/b/c"\n' "$home_seg" > "$tmp/work/w-bss2/prov.py"
    git -C "$tmp" add -f work/w-bss2/prov.py >/dev/null
    audit "$tmp" >/dev/null 2>&1 && st=0 || st=$?
    if [ "$st" -eq 0 ]; then
        echo "  allowlisted work/w-bss2/prov.py -> green (suppression works)"
    else
        echo "  ALLOWLIST DID NOT SUPPRESS work/w-bss2/prov.py (exit $st)" >&2
        fails=$((fails + 1))
    fi
    # ... and the SAME content one directory over must still go red, or the
    # entry is exempting a class rather than a file.
    printf 'p = "%s/u/a/b/c"\n' "$home_seg" > "$tmp/work/lane/copy.py"
    git -C "$tmp" add -f work/lane/copy.py >/dev/null
    audit "$tmp" >/dev/null 2>&1 && st=0 || st=$?
    if [ "$st" -eq 1 ]; then
        echo "  same content at work/lane/copy.py -> RED (entry is a FILE, not a class)"
    else
        echo "  ALLOWLIST OVER-SUPPRESSED: work/lane/copy.py exit $st, expected 1" >&2
        fails=$((fails + 1))
    fi
    git -C "$tmp" rm -q --cached work/w-bss2/prov.py work/lane/copy.py >/dev/null
    rm -f "$tmp/work/w-bss2/prov.py" "$tmp/work/lane/copy.py"

    # ---- class 5: the RATCHET, watched in BOTH directions (#3689) ------------
    #
    # A ceiling has two ways to be useless and only one of them is the obvious
    # one. It can fail to fire when the count EXCEEDS it — and it can fire on a
    # count that is merely nonzero, which would make every dated rung record a
    # violation and get the class switched off within a day. Both are checked,
    # and the second is the one that decides whether this class can live in a
    # repo that has 18 legitimate such files.
    #
    # `DOCS_ABS_CEILING` is lowered around the plant rather than the plant being
    # sized to 18 files: the ceiling is the subject under test, so driving it
    # directly is the honest probe, and 18 plants would test nothing extra.
    _saved_ceiling="$DOCS_ABS_CEILING"
    mkdir -p "$tmp/docs"
    printf '%s\n' "$abs_plant" > "$tmp/docs/rung.md"
    git -C "$tmp" add -f docs/rung.md >/dev/null

    DOCS_ABS_CEILING=1
    audit "$tmp" >/dev/null 2>&1 && st=0 || st=$?
    if [ "$st" -eq 0 ]; then
        echo "  docs/ abspath AT the ceiling (1 of 1) -> green (a ratchet is not a zero)"
    else
        echo "  RATCHET FIRED AT ITS OWN CEILING: exit $st, expected 0. A class" >&2
        echo "    that reds on 18 legitimate dated records will be switched off." >&2
        fails=$((fails + 1))
    fi

    DOCS_ABS_CEILING=0
    audit "$tmp" >/dev/null 2>&1 && st=0 || st=$?
    if [ "$st" -eq 1 ]; then
        echo "  docs/ abspath ABOVE the ceiling (1 of 0) -> RED (growth is caught)"
    else
        echo "  RATCHET DID NOT FIRE ABOVE ITS CEILING: exit $st, expected 1" >&2
        fails=$((fails + 1))
    fi

    git -C "$tmp" rm -q --cached docs/rung.md >/dev/null
    rm -f "$tmp/docs/rung.md"
    audit "$tmp" >/dev/null 2>&1 && st=0 || st=$?
    if [ "$st" -ne 0 ]; then
        echo "  RATCHET plant removed but audit STILL RED — not reversible" >&2
        fails=$((fails + 1))
    fi
    DOCS_ABS_CEILING="$_saved_ceiling"

    # The denominator guards: an empty index must FAIL, and so must a repo with
    # nothing under work/ — class 3 grading zero files is not class 3 passing.
    empty="$tmp/empty"
    mkdir -p "$empty" && git -C "$empty" init -q
    audit "$empty" >/dev/null 2>&1 && st=0 || st=$?
    if [ "$st" -eq 2 ]; then
        echo "  empty index -> exit 2 (an audit over 0 files is not a pass)"
    else
        echo "  EMPTY INDEX reported exit $st, expected 2" >&2
        fails=$((fails + 1))
    fi

    noshelf="$tmp/noshelf"
    mkdir -p "$noshelf/crates" && git -C "$noshelf" init -q
    git -C "$noshelf" config user.email a@b.c
    git -C "$noshelf" config user.name t
    printf 'ok\n' > "$noshelf/crates/keep.rs"
    git -C "$noshelf" add -A >/dev/null
    git -C "$noshelf" commit -qm base
    audit "$noshelf" >/dev/null 2>&1 && st=0 || st=$?
    if [ "$st" -eq 2 ]; then
        echo "  repo with an EMPTY work/ -> exit 2 (class 3 graded nothing)"
    else
        echo "  EMPTY SHELF reported exit $st, expected 2" >&2
        fails=$((fails + 1))
    fi

    if [ "$fails" -gt 0 ]; then
        echo "SELF-TEST FAIL: $fails detector(s) that do not behave." >&2
        return 3
    fi
    echo "SELF-TEST PASS: 11 planted violations red and each reversible,"
    echo "  allowlist suppresses its file and not its class, control green,"
    echo "  empty index and empty shelf both refused, and the class-5 ratchet"
    echo "  green AT its ceiling and red ABOVE it."
    return 0
}

case "${1:-}" in
    --self-test) self_test ;;
    "") audit "$REPO" ;;
    *) echo "usage: $0 [--self-test]" >&2; exit 2 ;;
esac
