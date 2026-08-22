#!/bin/sh
# doc_cite_audit.sh — does every `FOO.md` / `docs/FOO.md` / `FOO.md:NNN`
# citation in the docs actually resolve?
#
# WHY THIS EXISTS. `file.md:NNN` and `docs/FOO.md` citations are load-bearing
# across this tree — in docs, in `docs/BOARD.md` rows, in rung docs, in
# `crates/` comments and in merge messages. `scripts/board_audit.sh` answers a
# different question (which `#N` does ROADMAP cite that BOARD has no row for)
# and **cannot see cross-doc citation breakage at all**. That gap has been
# named three times — board **#3367**, **#3368** and **#3370**, the last of
# which put it this way: *"a citation to a decision is a citation to a
# document, and the document's own amendments are part of the citation."*
# This script closes the mechanical half of it: the target exists, and the
# cited line is inside the file. It does NOT and cannot check that the cited
# line still says what the citer thinks it says — see LIMITS below.
#
# ABSENCE IS NOT SUCCESS. Following `board_audit.sh`'s rule and ROADMAP
# §9.18.8: a regex that matched nothing and a tree with no broken citations
# look identical from the outside. So this script prints the number of
# citations it CHECKED on every run, and **exits non-zero if that number is
# zero**, exactly as it does for a real finding.
#
# THE SUPPRESSION CLASSES ARE PRINTED, NEVER SILENT. Several classes of token
# are legitimately unresolvable and are counted and named on every run rather
# than filtered out of sight:
#
#   1. `work/...`   — lane scratch, gitignored by design (`work/W42/ESTIMATE.md`).
#   2. `../...`     — sibling repos (`../dc3-decomp`, `../objdiff`), not in tree.
#   3. URLs         — anything to the right of an `http://` / `https://`.
#   4. `tmp/`, `~`  — session-local paths the docs quote inside commands.
#   5. non-`.md` unresolved — `crates/.../foo.rs` spellings that are
#      illustrative rather than real. Counted, not reported: the docs quote
#      plenty of these on purpose and reporting them would drown the signal.
#
# LIMITS, stated so nobody reads a green run as more than it is:
#   * A resolving citation can still be STALE. `FOO.md:251` staying in range
#     after 400 lines are inserted above it is the #3370 failure mode and this
#     script is blind to it.
#   * Section citations (`§6`) and anchors are not checked.
#   * A non-`.md` target is line-range-checked only when it resolves.
#
# Usage:
#   scripts/doc_cite_audit.sh                 # audit docs/ + README.md + CLAUDE.md
#   scripts/doc_cite_audit.sh --scan docs/whitebox
#   scripts/doc_cite_audit.sh --all           # every .md in the repo
#   scripts/doc_cite_audit.sh --bare          # also list the bare-name misses
#   scripts/doc_cite_audit.sh --self-test     # POSITIVE CONTROL, see below
#
# THE POSITIVE CONTROL. `--self-test` builds a throwaway tree containing one
# deliberately broken path citation and one deliberately out-of-range line
# citation, runs this same engine over it, and asserts it goes RED with both
# findings and stays quiet on the suppressed classes. A detector that has never
# been watched failing is not evidence. Run it before quoting a green audit.
#
# Exit: 0 = citations checked and all resolved. 1 = findings, or nothing checked.

set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
root="$repo_root"
scan=""
self_test=0
show_bare=0

while [ $# -gt 0 ]; do
    case "$1" in
        --scan)      scan="${scan} $2"; shift 2 ;;
        --all)       scan="${scan} ALLMD"; shift ;;
        --root)      root="$2"; shift 2 ;;
        --self-test) self_test=1; shift ;;
        --bare)      show_bare=1; shift ;;
        -h|--help)   sed -n '2,55p' "$0"; exit 0 ;;
        *)           echo "doc_cite_audit.sh: unknown argument: $1" >&2; exit 2 ;;
    esac
done

# ---------------------------------------------------------------- self-test
if [ "$self_test" -eq 1 ]; then
    ctl="$(mktemp -d)"
    trap 'rm -rf "$ctl"' EXIT
    mkdir -p "$ctl/docs/sub"
    printf 'one\ntwo\nthree\n' > "$ctl/docs/REAL.md"
    printf 'x\n' > "$ctl/docs/sub/NESTED.md"
    cat > "$ctl/docs/CONTROL.md" <<'CTL'
# control
Good path citation: `docs/REAL.md` and a nested one `docs/sub/NESTED.md`.
Good line citation: `REAL.md:2`.
PLANTED DEFECT 1 — target does not exist: `docs/NO_SUCH_DOC.md`.
PLANTED DEFECT 2 — line out of range: `REAL.md:999`.
Suppressed, must not be findings: `work/lane/SCRATCH.md`, `../sibling/X.md`,
`~/notes/HOME.md`, and https://example.com/URLDOC.md .
CTL
    echo "=== POSITIVE CONTROL — auditor run on a tree with 2 planted defects ==="
    set +e
    out="$("$0" --root "$ctl" --scan docs 2>&1)"
    rc=$?
    set -e
    printf '%s\n' "$out"
    echo "=== control exit code: $rc (expected: non-zero) ==="
    ok=1
    [ "$rc" -ne 0 ] || { echo "CONTROL FAILED: exit was 0, expected non-zero"; ok=0; }
    printf '%s\n' "$out" | grep -q 'NO_SUCH_DOC.md'  || { echo "CONTROL FAILED: missing-target defect not reported"; ok=0; }
    printf '%s\n' "$out" | grep -q 'REAL.md:999'     || { echo "CONTROL FAILED: out-of-range defect not reported"; ok=0; }
    printf '%s\n' "$out" | grep -q 'findings: 2'     || { echo "CONTROL FAILED: expected exactly 2 findings"; ok=0; }
    if printf '%s\n' "$out" | grep -q 'SCRATCH.md\|URLDOC.md\|HOME.md'; then
        echo "CONTROL FAILED: a suppressed class was reported as a finding"; ok=0
    fi
    if [ "$ok" -eq 1 ]; then
        echo
        echo "SELF-TEST PASS — red on both planted defects, quiet on every suppressed class."
        exit 0
    fi
    echo "SELF-TEST FAIL"
    exit 1
fi

[ -n "$scan" ] || scan="docs README.md CLAUDE.md"

cd "$root"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# ------------------------------------------------------- the file/line index
# Everything on disk, so a gitignored-but-present target still resolves.
find . -name .git -prune -o -name target -prune -o -type f -print \
    | sed 's|^\./||' | sort > "$tmp/files"

find . -name .git -prune -o -name target -prune -o -type d -print \
    | sed 's|^\./||' | sed 's|^$|.|' | sort > "$tmp/dirs"

grep -E '\.(md|rs|sh|py|txt|toml|cpp|h)$' "$tmp/files" > "$tmp/countable" || true
: > "$tmp/lines"
if [ -s "$tmp/countable" ]; then
    # One `wc` per batch, not one per file: the per-file loop cost ~15 s here.
    tr '\n' '\0' < "$tmp/countable" | xargs -0 wc -l 2>/dev/null \
        | sed 's/^ *//' | grep -v ' total$' > "$tmp/lines" || true
fi

# ------------------------------------------------------------- the scan set
: > "$tmp/scanset"
for s in $scan; do
    if [ "$s" = "ALLMD" ]; then
        grep '\.md$' "$tmp/files" >> "$tmp/scanset"
    elif [ -d "$s" ]; then
        find "$s" -type f -name '*.md' | sed 's|^\./||' >> "$tmp/scanset"
    elif [ -f "$s" ]; then
        printf '%s\n' "$s" >> "$tmp/scanset"
    fi
done
sort -u "$tmp/scanset" -o "$tmp/scanset"

nscan=$(wc -l < "$tmp/scanset" | tr -d ' ')
if [ "$nscan" -eq 0 ]; then
    echo "doc_cite_audit: nothing to scan (--scan '$scan')" >&2
    exit 1
fi

# ------------------------------------------------------------------- engine
# shellcheck disable=SC2046
awk -v idxfiles="$tmp/files" -v idxlines="$tmp/lines" -v idxdirs="$tmp/dirs" \
    -v nfiles="$nscan" -v showbare="$show_bare" '
function dirname(p,   i) {
    i = length(p)
    while (i > 0 && substr(p, i, 1) != "/") i--
    if (i == 0) return ""
    return substr(p, 1, i)          # keeps the trailing slash
}
function basename(p,   i) {
    i = length(p)
    while (i > 0 && substr(p, i, 1) != "/") i--
    return substr(p, i + 1)
}
function norm(p,   parts, n, i, out, k, r) {
    gsub(/\/\.\//, "/", p)
    sub(/^\.\//, "", p)
    n = split(p, parts, "/")
    k = 0
    for (i = 1; i <= n; i++) {
        if (parts[i] == "." || parts[i] == "") continue
        if (parts[i] == ".." && k > 0 && out[k] != "..") { k--; continue }
        out[++k] = parts[i]
    }
    r = ""
    for (i = 1; i <= k; i++) r = (i == 1 ? out[i] : r "/" out[i])
    return r
}
# Every base a relative citation could plausibly be written against: the citing
# directory and each of its ancestors, the repo root, and `docs/`. The ancestor
# walk is what makes `ref/README.md` — written inside `docs/whitebox/ref/` and
# meaning its own sibling — resolve instead of reading as a missing target.
function bases(from,   d, n) {
    nb = 0
    d = dirname(from)
    while (d != "") { BASE[++nb] = d; sub(/[^\/]*\/$/, "", d) }
    BASE[++nb] = ""
    BASE[++nb] = "docs/"
    return nb
}
function resolve(tok, from,   c, i, n) {
    AMBIG = 0
    n = bases(from)
    if (tok ~ /\//) {
        for (i = 1; i <= n; i++) { c = norm(BASE[i] tok); if (c in FILES) return c }
        return ""
    }
    # A BARE name that the tree holds more than once is inherently ambiguous:
    # whichever copy the search order lands on is a GUESS, and a guess must
    # never be range-checked. `mod.rs` exists 13 times here; guessing one and
    # then declaring `mod.rs:1856` out of range because the guess is a 16-line
    # file is a fabricated finding, and the first run produced 40 of them.
    # (`README.md:252` from `docs/whitebox/` is the same trap pointing at a
    # file outside the repo entirely.) Existence still counts; the range does not.
    if (BYNAME_N[tok] > 1) AMBIG = 1
    for (i = 1; i <= n; i++) { c = norm(BASE[i] tok); if (c in FILES) return c }
    # Last resort: anywhere in the tree. `P_DAG.md` is cited bare from six
    # directories and lives in `docs/whitebox/ref/`.
    if (tok in BYNAME) return BYNAME[tok]
    return ""
}
# Can we even judge this relative path? Its first segment has to name a real
# directory under one of the bases. `axes1/RESULTS.md` names a lane-scratch
# folder that was never in the tree; reporting it would be noise, so it is
# counted as unrooted instead.
function rootable(tok, from,   seg, i, n, c) {
    seg = tok; sub(/\/.*$/, "", seg)
    n = bases(from)
    for (i = 1; i <= n; i++) { c = norm(BASE[i] seg); if (c in DIRS) return 1 }
    return 0
}
BEGIN {
    while ((getline l < idxfiles) > 0) {
        FILES[l] = 1
        b = basename(l)
        BYNAME_N[b]++
        if (!(b in BYNAME)) BYNAME[b] = l
    }
    while ((getline l < idxdirs) > 0) DIRS[l] = 1
    while ((getline l < idxlines) > 0) { split(l, a, " "); LINES[a[2]] = a[1] }
    checked = 0; ranged = 0; findings = 0
    sup_work = 0; sup_sib = 0; sup_url = 0; sup_tmp = 0
    sup_other = 0; sup_bare = 0; sup_unrooted = 0; sup_ambig = 0
}
{
    urlpos = match($0, /https?:\/\//) ? RSTART : 0
    rest = $0; off = 0
    # The leading `.` / `~` alternatives are load-bearing: without them a
    # `../sibling/X.md` token matches from `sibling/` and is reported as a
    # missing target instead of being suppressed. The self-test caught exactly
    # that, which is the whole reason the self-test exists.
    while (match(rest, /[.~A-Za-z0-9_][A-Za-z0-9_.~\/-]*\.(md|rs|sh|py|toml|txt|cpp|h)(:[0-9]+)?/)) {
        tok    = substr(rest, RSTART, RLENGTH)
        abspos = off + RSTART
        off   += RSTART + RLENGTH - 1
        rest   = substr(rest, RSTART + RLENGTH)

        if (urlpos && abspos > urlpos) { sup_url++; continue }

        ln = 0
        if (match(tok, /:[0-9]+$/)) { ln = substr(tok, RSTART + 1) + 0; tok = substr(tok, 1, RSTART - 1) }

        if (tok ~ /^work\//)     { sup_work++; continue }
        if (tok ~ /^\.\.\//)     { sup_sib++;  continue }
        if (tok ~ /(^|\/)tmp\//) { sup_tmp++;  continue }
        if (tok ~ /^~/)          { sup_tmp++;  continue }

        target = resolve(tok, FILENAME)
        if (target == "") {
            if (tok !~ /\.md$/)        { sup_other++; continue }
            if (tok !~ /\//)           {
                # A bare basename that resolves nowhere is usually lane scratch
                # quoted by its short name (`ESTIMATE.md`, `MAGNITUDE.md`) or a
                # glob fragment (`WB_*_FINDINGS.md` → `_FINDINGS.md`). Counted,
                # and listed only under --bare, so it can never drown the class
                # that is actually load-bearing.
                sup_bare++
                if (showbare) printf "%s:%d: bare-unresolved     %s\n", FILENAME, FNR, tok
                continue
            }
            if (!rootable(tok, FILENAME)) { sup_unrooted++; continue }
            printf "%s:%d: MISSING TARGET      %s\n", FILENAME, FNR, tok
            findings++
            continue
        }
        if (tok ~ /\.md$/) checked++
        if (ln > 0) {
            if (AMBIG) { sup_ambig++; continue }
            ranged++
            if (!(target in LINES)) continue
            if (ln > LINES[target] + 0) {
                printf "%s:%d: LINE OUT OF RANGE   %s:%d  (%s has %d lines)\n",
                    FILENAME, FNR, tok, ln, target, LINES[target]
                findings++
            }
        }
    }
}
END {
    printf "\n"
    printf "files scanned:                       %d\n", nfiles
    printf "docs cited and resolved:             %d\n", checked
    printf "line-number citations range-checked: %d\n", ranged
    printf "suppressed (named classes):\n"
    printf "  work/ scratch          %6d\n", sup_work
    printf "  ../ sibling repo       %6d\n", sup_sib
    printf "  inside a URL           %6d\n", sup_url
    printf "  tmp/ or ~ session path %6d\n", sup_tmp
    printf "  non-.md unresolved     %6d\n", sup_other
    printf "  unrooted rel. path     %6d   (first segment names no directory in the tree)\n", sup_unrooted
    printf "  bare name unresolved   %6d   (rerun with --bare to list)\n", sup_bare
    printf "  ambiguous basename     %6d   (name exists >1x in tree; range NOT checked)\n", sup_ambig
    printf "findings: %d\n", findings
    if (checked == 0) {
        printf "\nNOTHING CHECKED — a matcher that matched nothing reads exactly like a clean tree. Treated as a failure.\n"
        exit 1
    }
    exit (findings > 0)
}
' $(cat "$tmp/scanset")
