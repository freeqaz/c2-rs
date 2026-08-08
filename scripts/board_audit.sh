#!/bin/sh
# board_audit.sh — is every `#N` that ROADMAP.md leans on actually ON the board,
# and does every anchor ON the board still point at something real?
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
#      merge"). They collide numerically with board #5, #14 and #15 (and with
#      the cited-but-rowless #9, #10, #12) and must never be absorbed into the
#      board series.
#   3. MULTI-NUMBER ROWS. `| 46, 48 | Provenance and loader reporting |` is one
#      row covering two numbers. A row matcher that reads only the first number
#      reports #48 as missing, which is how this script's first run produced two
#      of its six "gaps".
#
# The filter for (1) and (2) is a LIST, printed every run, so a number that
# becomes a real item later shows up as a suppressed line rather than vanishing.
#
# THREE MORE CHECKS, ADDED 2026-08-02 (lane w-boardfix)
# --------------------------------------------------------------------------
# BOARD.md's `R:NNNN` anchors were raw line numbers into an 8,600-line file that
# grows MID-FILE (§9.21 landed before §10 and shifted every line after it, and
# nothing noticed). All anchors are now section numbers (`R:§9.20.5`), which
# survive insertion, and three checks keep the class from rotting again:
#
#   4. SECTION ANCHORS RESOLVE. Every `R:§x` / `EH:§x` / `LC:§x` / `SEAMS:§x`
#      in BOARD.md must open a real heading in its target file. A count and the
#      unresolved list are printed; an extractor that matches nothing prints
#      "0 distinct", which is visible, not silent.
#   5. NO RAW LINE-NUMBER ANCHORS. Any `R:<digits>` (or EH:/LC:/SEAMS:) is
#      listed as drift waiting to happen. `rungs/<lane>:<line>` refs are NOT
#      flagged: rung records are frozen one-shot files that do not grow.
#   6. ROWS BEHIND THE PROSE. If a ROADMAP *heading* names `#N` (a section
#      ABOUT the item — "10.13 #152 re-priced…") in a section that no row of
#      #N cites, the row missed a re-measurement. This is the mechanized form
#      of the 2026-08-02 staleness defect (#152's probe TUs moved by §10.11,
#      row not updated). Deliberately scoped to HEADINGS: the same check over
#      every prose citation of #N flags 27 of 50 rows on day one (summary
#      tables restate half the board), which is an ignored klaxon, not a
#      check. The heading-scoped form flagged 5 rows when written, of which 3
#      were genuinely stale — that ratio is the evidence for the scope.
#
# MUTATION-TESTED, because a working audit and a broken one read identically.
# Re-run these if you touch an extractor — an audit whose output does not move
# under a broken input is measuring nothing:
#   - Deleting the `| 149 |` row from a copy of BOARD.md makes this print
#     `#149 cited 9x` under CITED BUT NOT ON THE BOARD.
#   - Rewriting `R:§9.20.5` as `R:§9.99.9` in a copy of BOARD.md makes check 4
#     print `R:§9.99.9 — no heading '9.99.9' in ROADMAP.md` (verified
#     2026-08-02).
#   - Rewriting that same anchor as `R:9205` (the old raw `R:` + digits form,
#     like the pre-conversion `R:7734`) moves check 5 from 0 to 1 with the
#     offending BOARD.md line listed (verified 2026-08-02).
#   - Deleting `R:§9.20.5, R:§10.13` from the #152<sub>w-emitset</sub> row's
#     defined-cell makes check 6 print `#152 — heading-named in §10.13, no row
#     cites it` (verified 2026-08-02).
#
# Usage:  scripts/board_audit.sh [--check]
#   --check   known-answer self-test; needs no toolchain and no network.

set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ROADMAP="$repo/docs/ROADMAP.md"
BOARD="$repo/docs/BOARD.md"
EHDOC="$repo/docs/EH_RECORDS.md"
LCDOC="$repo/docs/LABEL_COUNTER.md"
SEAMSDOC="$repo/docs/ARCHITECTURE_SEAMS.md"

# Numbers ROADMAP.md cites that are NOT board items. Each needs a reason, and
# the reason is what a reader checks when a number moves onto the board.
suppressed() {
    cat <<'EOF'
1	ranking — "the #1 census blocker"; 15 citations, none an item
2	ranking — "#2 blocker at 141,800"
3	ranking — "the #3 row is behind EH too"
4	ranking — "the row that was #4 by bodies" (R:§8.6)
9	GAPS.md §6 mis-emit instance series (cited as "§6 #9", R:§6l)
10	GAPS.md §6 mis-emit instance series (cited as "§6 #10", R:§6l)
12	GAPS.md §6 mis-emit instance series, not the board series
1668	endpoint of an explicitly-unminted range declaration ("#1668–#1679 are free")
1679	endpoint of an explicitly-unminted range declaration ("#1668–#1679 are free")
EOF
}

extract_cited() {
    # Every `#N` in prose. Deliberately broad: narrowing the pattern to
    # `board #N` would miss the majority, since ROADMAP cites items bare.
    grep -o '#[0-9]\{1,4\}\b' "$1" | tr -d '#' | sort -n | uniq -c \
        | while read -r n num; do printf '%s %s\n' "$num" "$n"; done
}

extract_rows() {
    # A board row's number(s). Handles `| 149 |`, `| **158** |`,
    # `| 151<sub>w-vgl</sub> |`, `| **151**<sub>w-vgl</sub> **→ realise it** |`
    # and the multi-number `| 46, 48 |` — the last of which is why this is not
    # a one-number match.
    sed -n 's/^| *\*\{0,2\}\([0-9, ]\{1,12\}\).*$/\1/p' "$1" \
        | tr ',' '\n' | tr -d ' ' | grep '^[0-9]\{1,4\}$' | sort -n -u
}

# --- check 4-6 helpers -------------------------------------------------------

anchor_file() { # prefix -> the file it anchors into
    case "$1" in
        R)     printf '%s' "$ROADMAP" ;;
        EH)    printf '%s' "$EHDOC" ;;
        LC)    printf '%s' "$LCDOC" ;;
        SEAMS) printf '%s' "$SEAMSDOC" ;;
    esac
}

extract_section_anchors() {
    # Every `PFX:§token` in $1, as "PFX token", deduped. A trailing `.` is
    # sentence punctuation, not part of the section number.
    grep -o '\(R\|EH\|LC\|SEAMS\):§[0-9][0-9A-Za-z.]*' "$1" \
        | sed 's/\.$//; s/:§/ /' | sort -u
}

heading_exists() { # $1=file $2=section token -> true iff a heading opens with it
    esc=$(printf '%s' "$2" | sed 's/\./\\./g')
    grep -q "^#\{1,4\} $esc\([ .]\|\$\)" "$1"
}

extract_raw_anchors() {
    # `PFX:` followed directly by digits — the pre-2026-08-02 line-number form.
    # Prints board-file line numbers so the offender is one jump away.
    grep -no '\(R\|EH\|LC\|SEAMS\):[0-9]\{1,6\}' "$1" || true
}

section_map() { # numbered headings of $1 as "line:token" ("6c." -> "6c")
    grep -n '^#\{1,4\} [0-9]' "$1" | sed 's/:#\{1,4\} /:/; s/ .*//; s/\.$//'
}

last_naming_section() {
    # $1=roadmap $2=N -> the newest numbered section whose HEADING names #N
    l=$(grep -n "^#\{1,4\} .*#$2\b" "$1" | tail -1 | cut -d: -f1)
    [ -n "$l" ] || return 0
    section_map "$1" | awk -F: -v l="$l" '$1<=l{s=$2} END{if (s!="") print s}'
}

row_sections() { # $1=board $2=N -> every §token any row of #N cites
    grep "^| *\*\{0,2\}$2\b\|, $2\b" "$1" \
        | grep -o '§[0-9][0-9A-Za-z.]*' | tr -d '§' | sed 's/\.$//' | sort -u
}

stale_rows() { # $1=board $2=roadmap -> "N sec" for every row behind the prose
    for n in $(extract_rows "$1"); do
        sec=$(last_naming_section "$2" "$n")
        [ -n "$sec" ] || continue
        cov=no
        for tok in $(row_sections "$1" "$n"); do
            # covered by the exact section, an ancestor, a descendant, or a
            # lettered sibling (§9.17.3 covers 9.17.3a)
            case "$sec" in "$tok"|"$tok".*|"$tok"[a-z]) cov=yes ;; esac
            case "$tok" in "$sec".*|"$sec"[a-z]) cov=yes ;; esac
        done
        [ "$cov" = yes ] || printf '%s %s\n' "$n" "$sec"
    done
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

    # -- check 4: section anchors resolve --------------------------------
    anchors=$(extract_section_anchors "$BOARD")
    n_anchors=$(printf '%s\n' "$anchors" | grep -c '^[A-Z]' || true)
    printf '\nsection anchors in BOARD.md: %s distinct (R:/EH:/LC:/SEAMS: -> §)\n' "$n_anchors"
    unres=$(printf '%s\n' "$anchors" | while read -r pfx tok; do
        [ -n "$pfx" ] || continue
        heading_exists "$(anchor_file "$pfx")" "$tok" \
            || printf '  %s:§%s — no heading %s in %s\n' \
                   "$pfx" "$tok" "$tok" "$(basename "$(anchor_file "$pfx")")"
    done)
    unresolved=$(printf '%s' "$unres" | grep -c '§' || true)
    printf 'UNRESOLVED SECTION ANCHORS: %s\n' "$unresolved"
    [ -n "$unres" ] && printf '%s\n' "$unres"
    [ "$unresolved" -eq 0 ] && printf '  (none — every §-anchor opens a real heading in its file)\n'

    # -- check 5: no raw line-number anchors -----------------------------
    raw=$(extract_raw_anchors "$BOARD")
    n_raw=$(printf '%s' "$raw" | grep -c ':' || true)
    printf '\nRAW LINE-NUMBER ANCHORS (drift-prone; convert to §): %s\n' "$n_raw"
    [ -n "$raw" ] && printf '%s\n' "$raw" | sed 's/^/  BOARD.md:/'
    [ "$n_raw" -eq 0 ] && printf '  (none — rungs/<lane>:<line> refs into frozen rung files are exempt by design)\n'

    # -- check 6: rows behind the prose ----------------------------------
    stale=$(stale_rows "$BOARD" "$ROADMAP")
    n_stale=$(printf '%s' "$stale" | grep -c '^[0-9]' || true)
    printf '\nROWS BEHIND THE PROSE (a ROADMAP heading names #N in a section no row of #N cites): %s\n' "$n_stale"
    printf '%s\n' "$stale" | while read -r n sec; do
        [ -n "$n" ] || continue
        printf '  #%s — heading-named in §%s, no row cites it:\n' "$n" "$sec"
        grep -n "^#\{1,4\} .*#$n\b" "$ROADMAP" | tail -1 | sed 's/^/      R:/'
    done
    [ "$n_stale" -eq 0 ] && printf '  (none — every heading-level re-measurement is cited by its row)\n'

    # -- check 7: duplicate row numbers ----------------------------------
    # Two sessions racing on the namespace minted #976-#985 twice (w-bytes and
    # w-inread, 2026-08-08), and six re-gates ran green over the duplicates —
    # nothing looked. A row number is an identity: every citation of #N in a
    # rung, a doc or a commit message resolves through it, so a duplicate makes
    # every one of those citations ambiguous forever. Printed as COUNT + LIST
    # like every other check; the count is the output, not an "OK".
    dups=$(grep -oE '^\| \*\*[0-9]+\*\*' "$BOARD" | grep -oE '[0-9]+' | sort -n | uniq -d)
    n_dups=$(printf '%s' "$dups" | grep -c '^[0-9]' || true)
    printf '\nDUPLICATE ROW NUMBERS (two rows claim one identity): %s\n' "$n_dups"
    [ -n "$dups" ] && printf '%s\n' "$dups" | while read -r n; do
        [ -n "$n" ] || continue
        grep -n "^| \*\*$n\*\*" "$BOARD" | sed 's/\(.\{100\}\).*/\1/;s/^/  BOARD.md:/'
    done
    [ "$n_dups" -eq 0 ] && printf '  (none — every row number names exactly one row)\n'
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
    t 'suppression list size' "$(suppressed | wc -l | tr -d ' ')" '9'

    # -- checks 4-6, on hand-built fixtures, never the live files --------
    cat > "$tmp/rm2.md" <<'EOF'
# 1. Intro
## 6c. The census, closed
## 9.20 The binding
## 9.20.5 The ladder, re-priced
## 10.13 #77 re-priced against 4,591
EOF
    printf '## 9.20.55 only the wider section exists here\n' > "$tmp/rm3.md"
    cat > "$tmp/b2.md" <<'EOF'
| # | item | worth | defined | notes |
|---|---|---|---|---|
| 77 | an item | x | R:§9.20.5. | also R:§6c and EH:§7.5 and R:§99.99 |
| 78 | raw refs | x | R:7734 | legend `R:§x` and rungs/w-adjust:52 and LC:4532 |
EOF

    got=$(extract_section_anchors "$tmp/b2.md" | tr '\n' '/' | tr ' ' ':' | sed 's,/$,,')
    t 'anchor extraction (dedupe, prefix split, trailing-dot strip)' \
        "$got" 'EH:7.5/R:6c/R:9.20.5/R:99.99'

    got=$(heading_exists "$tmp/rm2.md" '9.20.5' && echo yes || echo no)
    t 'anchor resolves to its heading' "$got" 'yes'
    got=$(heading_exists "$tmp/rm2.md" '6c' && echo yes || echo no)
    t 'lettered section with trailing dot resolves' "$got" 'yes'
    got=$(heading_exists "$tmp/rm3.md" '9.20.5' && echo yes || echo no)
    t '9.20.5 must NOT resolve against a lone 9.20.55' "$got" 'no'
    got=$(heading_exists "$tmp/rm2.md" '99.99' && echo yes || echo no)
    t 'missing section does not resolve' "$got" 'no'

    got=$(extract_raw_anchors "$tmp/b2.md" | tr '\n' ' ' | sed 's/ $//')
    t 'raw line-number anchors found; rungs/ and legend exempt' \
        "$got" '4:R:7734 4:LC:4532'

    got=$(stale_rows "$tmp/b2.md" "$tmp/rm2.md" | tr '\n' ' ' | sed 's/ $//')
    t 'row behind the prose is flagged (#77 vs heading 10.13)' "$got" '77 10.13'
    sed 's/R:§9.20.5./R:§10.13/' "$tmp/b2.md" > "$tmp/b3.md"
    got=$(stale_rows "$tmp/b3.md" "$tmp/rm2.md" | tr '\n' ' ' | sed 's/ $//')
    t 'row citing the naming section is not flagged' "$got" ''

    if [ "$fails" -eq 0 ]; then
        printf 'BOARD AUDIT CHECK: PASS — 11 known-answer tests\n'; return 0
    fi
    printf 'BOARD AUDIT CHECK: FAIL — %s\n' "$fails"; return 1
}

case "${1:-}" in
    --check) check ;;
    '') audit ;;
    *) printf 'usage: %s [--check]\n' "$0" >&2; exit 2 ;;
esac
