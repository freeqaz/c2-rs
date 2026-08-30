#!/bin/sh
# Sanitize a gate transcript for the work/ evidence shelf.
#
# `scripts/tracked_artifact_audit.sh` class 3 refuses absolute machine paths on
# the shelf, and two of four lanes tripped it last wave. SANITISE, never delete
# — the transcript is the evidence. Verified afterwards with the audit's OWN
# regexes (`ABS_FWD` / `ABS_BS`), not with the pattern substituted on: a check
# that greps for what you just replaced cannot fail.
#
# Usage: sanitize.sh <raw> <out>
set -eu
raw="$1"; out="$2"
root=$(cd "$(dirname "$0")/../.." && pwd)
shared=$(printf '%s' "$root" | sed 's|/\.claude/worktrees/.*$||')

sed -e "s|$root|<WORKTREE>|g" -e "s|$shared|<REPO>|g" \
    -e 's|/home/[a-z][a-z0-9_-]*/|<PATH>/|g' \
    -e 's|\\home\\[a-z][a-z0-9_-]*\\|<PATH>\\|g' "$raw" > "$out"

# The audit's own regexes, re-derived from the audit rather than retyped.
fwd=$(sed -n 's/^ABS_FWD="\$_h\(.*\)"$/\1/p' "$root/scripts/tracked_artifact_audit.sh")
bs=$(sed -n "s/^ABS_BS='\(.*\)'\$/\1/p" "$root/scripts/tracked_artifact_audit.sh")
h=$(sed -n 's/^_h=.*//p' "$root/scripts/tracked_artifact_audit.sh")
hits=$(grep -c -E -e "/home$fwd" -e "$bs" "$out" || true)
echo "sanitised $raw -> $out"
echo "  audit class-3 regexes /home$fwd and $bs : $hits hit(s) (must be 0)"
[ "$hits" -eq 0 ]
