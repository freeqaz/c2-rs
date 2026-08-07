#!/bin/sh
# Which of w-gate's new selftest assertions has actually been SEEN to fail?
#
# A mutation suite that reddens something is not evidence that it reddened the
# thing you built. This walks the assertion list and prints, per assertion, how
# many mutations put it in the RED column. A zero here is an assertion nobody has
# ever seen fail — which on this project is the same as not having it.
set -eu
repo="$(cd "$(dirname "$0")/../.." && pwd)"
log="$repo/work/w-gate/mutations.txt"

miss=0
total=0
while IFS= read -r a; do
    [ -n "$a" ] || continue
    case "$a" in '#'*) continue ;; esac
    n=$(grep -c "RED: .*$a" "$log" || true)
    total=$((total + 1))
    [ "$n" -eq 0 ] && miss=$((miss + 1))
    printf '%4s  %s\n' "$n" "$a"
done <<'LIST'
require-graded-all-skip-fails
demand-off-all-skip-still-exits-0
require-graded-green-run-passes
demand-off-green-run-passes
require-graded-sampled-satisfies
hint-names-every-override
hint-names-the-one-command-that-fixes-it
hint-does-not-name-c2rs-dc3
hint-overrides-still-read-by-the-resolver
hint-reads-the-version-dir-from-source
hint-marks-absent-paths-missing
hint-does-not-claim-they-exist
hint-declines-to-blame-a-present-path
require-graded-refuses-reap-only
require-graded-refuses-before-reaping
require-graded-env-leaves-list-alone
require-graded-says-when-it-does-not-apply
the demand turns an all-skip run RED
fails on a COUNT of graded units
the lanes-that-graded count is printed beside the sum
never also says SKIPPED
nor PASS
the resolution hint rides along
without the demand an all-skip run is UNCHANGED
a run that graded 4 corpora passes WITH the demand set
the demand is silent when the count it wants is positive
a SAMPLED run graded something
does not moonlight as a completeness check
a partial skip still fails as a PARTIAL SKIP
the demand did not duplicate a check that already existed
a mismatch under the demand still raises the mismatch alarm
and is never relabelled as a nothing-graded run
LIST

echo
echo "$total assertions checked; $miss never seen red."
[ "$miss" -eq 0 ]
