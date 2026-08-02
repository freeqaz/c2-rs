#!/bin/sh
# board_audit.sh — is every `#N` that ROADMAP.md leans on actually ON the board?
#
# `docs/BOARD.md` is hand-maintained: unlike `docs/rungs/INDEX.md` there is no
# header block to generate it from, so its only protection against drift is that
# somebody notices. This is that somebody, mechanized.
#
# It answers ONE question — which numbers does `ROADMAP.md` cite that
# `BOARD.md` has no row for — and it answers it by printing a COUNT and a LIST,
# never a status. A board audit that printed "OK" would be the project's
# thirteenth absence-read-as-success (ROADMAP §9.18.8): a regex that matched
# nothing and a board that covered everything look identical from the outside.
# So the exit code is advisory and the numbers are the output.
#
# THREE CLASSES OF FALSE POSITIVE, NAMED RATHER THAN SILENTLY FILTERED
# --------------------------------------------------------------------------
# A bare `#N` in ROADMAP.md is not always a board item, and the three ways it is
# not are the reason this script exists at all rather than a one-line grep:
#
#   1. RANKINGS. "the #1 census blocker", "#2 blocker at 141,800", "the #3 row
#      is behind EH too" — ordinals over a histogram, not items. #1 alone is
#      cited 15 times this way. Any future item numbered 1, 2 or 3 would be
#      unreadable in this document; that is a fact about the prose, not a gap.
#   2. `GAPS.md` §6's SEPARATE SERIES. Its mis-emit instances #1–#15 are a defect
#      taxonomy, and ROADMAP cites them ("instance #12 was found by the previous
#      merge"). They collide numerically with board #5, #12, #14 and #15 and
#      must never be absorbed into the board series.
#   3. MULTI-NUMBER ROWS. `| 46, 48 | Provenance and loader reporting |` is one
#      row covering two numbers. A row matcher that reads only the first number
#      reports #48 as missing, which is how this script's first run produced two
#      of its six "gaps".
#
# The filter for (1) and (2) is a LIST, printed every run, so a number that
# becomes a real item later shows up as a suppressed line rather than vanishing.
#
# MUTATION-TESTED, because a working audit and a broken one read identically.
# Deleting the `| 149 |` row from a copy of BOARD.md makes this print
# `#149 cited 9x` under CITED BUT NOT ON THE BOARD. Re-run that if you touch
# either extractor: an audit whose output does not move when a row is deleted is
# measuring nothing, and would be exactly the failure it was written against.
#
# Usage:  scripts/board_audit.sh [--check]
#   --check   known-answer self-test; needs no toolchain and no network.

set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ROADMAP="$repo/docs/ROADMAP.md"
BOARD="$repo/docs/BOARD.md"

# Numbers ROADMAP.md cites that are NOT board items. Each needs a reason, and
# the reason is what a reader checks when a number moves onto the board.
suppressed() {
    cat <<'EOF'
1	ranking — "the #1 census blocker"; 15 citations, none an item
2	ranking — "#2 blocker at 141,800"
3	ranking — "the #3 row is behind EH too"
4	ranking — "the row that was #4 by bodies" (R:3999)
9	GAPS.md §6 mis-emit instance series (cited as "§6 #9", R:2685)
10	GAPS.md §6 mis-emit instance series (cited as "§6 #10", R:2775)
12	GAPS.md §6 mis-emit instance series, not the board series
EOF
}

extract_cited() {
    # Every `#N` in prose. Deliberately broad: narrowing the pattern to
    # `board #N` would miss the majority, since ROADMAP cites items bare.
    grep -o '#[0-9]\{1,3\}\b' "$1" | tr -d '#' | sort -n | uniq -c \
        | while read -r n num; do printf '%s %s\n' "$num" "$n"; done
}

extract_rows() {
    # A board row's number(s). Handles `| 149 |`, `| **158** |`,
    # `| 151<sub>w-vgl</sub> |`, `| **151**<sub>w-vgl</sub> **→ realise it** |`
    # and the multi-number `| 46, 48 |` — the last of which is why this is not
    # a one-number match.
    sed -n 's/^| *\*\{0,2\}\([0-9, ]\{1,12\}\).*$/\1/p' "$1" \
        | tr ',' '\n' | tr -d ' ' | grep '^[0-9]\{1,3\}$' | sort -n -u
}

audit() {
    cited=$(extract_cited "$ROADMAP")
    rows=$(extract_rows "$BOARD")
    supp=$(suppressed | cut -f1)

    n_rows=$(printf '%s\n' "$rows" | grep -c '^[0-9]')
    n_cited=$(printf '%s\n' "$cited" | grep -c '^[0-9]')
    printf 'board rows            : %s distinct numbers\n' "$n_rows"
    printf 'ROADMAP citations     : %s distinct numbers\n' "$n_cited"

    printf '\nsuppressed (not board items, with the reason):\n'
    suppressed | while IFS='	' read -r num why; do
        c=$(printf '%s\n' "$cited" | awk -v n="$num" '$1==n {print $2}')
        printf '  #%-4s cited %-4s %s\n' "$num" "${c:-0}x" "$why"
    done

    missing=$(printf '%s\n' "$cited" | awk '{print $1}' \
        | grep -vxF "$(printf '%s\n' "$rows")" 2>/dev/null || true)
    missing=$(for m in $missing; do
        printf '%s\n' "$supp" | grep -qx "$m" || printf '%s\n' "$m"
    done)

    n_missing=$(printf '%s' "$missing" | grep -c '^[0-9]' || true)
    printf '\nCITED BUT NOT ON THE BOARD: %s\n' "$n_missing"
    for m in $missing; do
        c=$(printf '%s\n' "$cited" | awk -v n="$m" '$1==n {print $2}')
        printf '  #%-4s cited %sx\n' "$m" "$c"
        grep -n "#$m\b" "$ROADMAP" | head -2 | sed 's/^/      /'
    done
    [ "$n_missing" -eq 0 ] && printf '  (none — every cited number has a row or a named reason)\n'
    return 0
}

check() {
    fails=0
    t() { # t <label> <got> <want>
        if [ "$2" = "$3" ]; then printf '  ok   %s\n' "$1"
        else printf '  FAIL %s: got %s, want %s\n' "$1" "$2" "$3"; fails=$((fails+1)); fi
    }
    tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT

    # The multi-number row is the case that produced two false gaps on the
    # first run. Pinned by construction, not by the live file.
    cat > "$tmp/b.md" <<'EOF'
| # | item | worth | defined | notes |
|---|---|---|---|---|
| 46, 48 | Provenance and loader reporting | x | y | z |
| **158** | A bold row | x | y | z |
| 151<sub>w-vgl</sub> | A suffixed row | x | y | z |
| **151**<sub>w-vgl</sub> **→ realise it** | A bold suffixed row | x | y | z |
| 149 | A plain row | x | y | z |
EOF
    got=$(extract_rows "$tmp/b.md" | tr '\n' ' ' | sed 's/ $//')
    t 'row extraction (multi-number, bold, <sub>)' "$got" '46 48 149 151 158'

    printf '#158 and #158 again, then #46 and #48 and #1\n' > "$tmp/r.md"
    got=$(extract_cited "$tmp/r.md" | tr '\n' ' ' | sed 's/ $//')
    t 'citation counting' "$got" '1 1 46 1 48 1 158 2'

    # A suppression list that silently swallowed a real item would defeat the
    # whole script, so its size is pinned too.
    t 'suppression list size' "$(suppressed | wc -l | tr -d ' ')" '7'

    if [ "$fails" -eq 0 ]; then
        printf 'BOARD AUDIT CHECK: PASS — 3 known-answer tests\n'; return 0
    fi
    printf 'BOARD AUDIT CHECK: FAIL — %s\n' "$fails"; return 1
}

case "${1:-}" in
    --check) check ;;
    '') audit ;;
    *) printf 'usage: %s [--check]\n' "$0" >&2; exit 2 ;;
esac
