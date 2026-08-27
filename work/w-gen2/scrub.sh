#!/bin/sh
# scrub.sh — replace this box's absolute paths in this lane's committed evidence.
#
# The patterns are DERIVED, never listed: a previous lane's scrub script hard-coded
# five of this box's own paths as its search patterns, which is the same defect one
# level out. Two rules, in this order:
#
#   1. the repo/worktree root, whatever it is, becomes `<repo>`;
#   2. any remaining `/home/<user>` becomes `/home/<user>`, matched as
#      `/home/[a-z][a-z0-9_-]*` and not as a literal.
#
# BOARD #1135: never rewrite a file another process still holds open. A scrub once
# raced a backgrounded `gate.sh` that still held its `>` descriptor and punched a
# NUL hole into a PASSING gate's log — `grep` returned nothing and a waiter
# reported TIMEOUT; the mirror case, on a FAILING gate, makes `grep -q FAIL` read
# clean.
#
# The guard is **per file and by open descriptor**, not "is any gate alive". The
# coarse form is both too weak (a writer that is not a gate slips through) and too
# strong (a PEER LANE's gate, in another worktree, writing a file this script will
# never open, blocks it). Peer lanes run concurrently here by design and their
# logs must be left alone — which is the other half of #1135.
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

has_nul() {
    # 0 = the file contains a NUL byte.
    #
    # THE LINE THIS REPLACES WAS NOT BROKEN, AND SAYING SO IS THE POINT.
    # It read `LC_ALL=C grep -q -a -P '\x00' "$f"`, and that **works**. The
    # discriminator is `-a`, which board #1236 / #3544 / #3673 never name.
    # Measured here 2026-08-27 on `printf 'a\000b\n'` vs `printf 'ab\n'`,
    # both implementations on this box (0 = fired, 1 = silent):
    #
    #                            NUL file   text file
    #   ugrep 7.5.0  -qP           1          1      <- SILENT. broken.
    #   GNU 3.12     -qP           1          1      <- SILENT. broken.
    #   ugrep 7.5.0  -q -a -P      0          1      <- correct
    #   GNU 3.12     -q -a -P      0          1      <- correct
    #   byte count                 0          1      <- correct
    #
    # So `tracked_artifact_audit.sh`'s standing comment — *"grep cannot test
    # for a NUL byte"* — is **false**; it can, with `-a`. What is true is that
    # the spelling everyone reaches for **omits `-a`**, and its failure is
    # silent: the coordinator's 2026-08-26 scan used `grep -P '\x00'` and
    # reported a clean tree over a 3.8 MB ELF.
    #
    # The other standing spelling is worse than silent — it is anti-correlated:
    #   GNU   grep -c $'\0'  -> 2 on the NUL file, 1 on a PLAIN TEXT file
    #   ugrep grep -c $'\0'  -> exit 1 / empty on the NUL file, 1 on plain text
    # because `$'\0'` is the EMPTY string in argv (the shell truncates at the
    # NUL), so it matches every line and counts lines.
    #
    # The byte count is kept anyway, for one reason: it is correct without
    # depending on a flag whose absence fails quietly. `--self-test` below
    # watches it fire in both directions, and watches the flagless grep form
    # stay silent, so nobody reinstates that one.
    [ "$(tr -d '\000' < "$1" | wc -c)" -ne "$(wc -c < "$1")" ]
}

self_test() {
    # A detector nobody has watched fire is not a detector (#1175, #3544).
    d="$(mktemp -d "${TMPDIR:-/tmp}/w-gen2-nul-XXXXXX")"
    trap 'rm -rf "$d"' EXIT INT TERM
    printf 'a\000b\n' > "$d/nul"
    printf 'ab\n'     > "$d/txt"
    rc=0
    if has_nul "$d/nul"; then echo "  NUL probe  -> RED   (correct)"
    else echo "  NUL PROBE STAYED GREEN — the detector does not fire" >&2; rc=1; fi
    if has_nul "$d/txt"; then
        echo "  TEXT PROBE WENT RED — the detector fires on clean input" >&2; rc=1
    else echo "  text probe -> green (correct)"; fi
    # The flagless grep form, watched STAYING SILENT on a file that does
    # contain a NUL. This is the control that matters: it is the spelling the
    # 2026-08-26 scan used, and it is why a 3.8 MB ELF read as a clean tree.
    if LC_ALL=C grep -q -P '\x00' "$d/nul" 2>/dev/null; then
        echo "  UNEXPECTED: grep -qP (no -a) fired on the NUL probe here." >&2
        echo "  It is silent on ugrep 7.5.0 and GNU 3.12 as measured; re-read" >&2
        echo "  the table in has_nul() before trusting either." >&2
        rc=1
    else
        echo "  control: grep -qP hex-NUL WITHOUT -a -> silent on the NUL probe"
        echo "           (this is the broken reflex, watched not firing)"
    fi
    # and the form that DOES work with grep, so the record is not a caricature
    if LC_ALL=C grep -q -a -P '\x00' "$d/nul" 2>/dev/null; then
        echo "  control: grep -q -a -P hex-NUL -> RED (the -a form works)"
    else
        echo "  NOTE: grep -q -a -P did NOT fire here; that contradicts the" >&2
        echo "  measured table in has_nul(). Trust the byte count." >&2
    fi
    [ "$rc" -eq 0 ] && echo "NUL SELF-TEST PASS: fires on NUL, silent on text."
    return "$rc"
}

[ "${1:-}" = "--self-test" ] && { self_test; exit $?; }

holders() {
    # PIDs with the given absolute path open, via /proc's fd symlinks. Bounded
    # depth; /proc only, never the repo tree.
    find /proc -mindepth 3 -maxdepth 3 -path '/proc/[0-9]*/fd/*' -lname "$1" \
        2>/dev/null | cut -d/ -f3 | sort -u
}

for f in "$@"; do
    [ -f "$f" ] || { echo "no such file: $f" >&2; exit 1; }
    abs="$(cd "$(dirname "$f")" && pwd)/$(basename "$f")"
    h="$(holders "$abs")"
    if [ -n "$h" ]; then
        echo "REFUSING: $f is held open by PID(s): $h — scrub after they exit" >&2
        echo "  (board #1135: rewriting a log a writer still holds punches a NUL" >&2
        echo "   hole into it and makes every later grep of it meaningless)" >&2
        exit 2
    fi
    tmp="$f.scrub.$$"
    sed -e "s|$root|<repo>|g" \
        -e 's|/home/[a-z][a-z0-9_-]*|/home/<user>|g' "$f" > "$tmp"
    mv "$tmp" "$f"
    if has_nul "$f"; then
        echo "FATAL: $f is not NUL-free after the rewrite (board #1135)." >&2
        exit 3
    fi
done
echo "scrubbed $# file(s); all NUL-free, none held open"
