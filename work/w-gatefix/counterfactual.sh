#!/bin/sh
# w-gatefix — THE COUNTERFACTUAL, RUN AGAINST `master`'s OWN `gate.sh`.
#
# `gate.sh --check`'s arms C1/C2/C3 show that the SUBJECTS were defective at the
# merge base. This shows the thing a reader actually wants: **master's
# `hatch_red_run`, driven exactly as the gate drives it, eats the edit** — and
# the tip's does not. Both halves against a throwaway `git init` tree, so it can
# be re-run at any time without risking a real worktree.
#
#   sh work/w-gatefix/counterfactual.sh [<base-rev>]
#
# Exit 0 iff master ate the edit AND the tip preserved it. Either half behaving
# the other way is the failure: a base that preserves means the arms below are
# asserting nothing, and a tip that eats means the fix did not take.
set -eu

BASE="${1:-0d0a74d2}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
scratch="${TMPDIR:-/tmp}/w-gatefix-cf.$$"
trap 'rm -rf "$scratch"' EXIT INT TERM
mkdir -p "$scratch"

git -C "$root" show "$BASE:scripts/gate.sh"            > "$scratch/gate.base.sh"
git -C "$root" show "$BASE:work/w-hatch/hatch_red.py"  > "$scratch/hatch_red.base.py"

# One synthetic checkout, built the way the gate's own arms build theirs.
mktree() {   # <dir> <hatchred>
    rm -rf "$1"; mkdir -p "$1/crates" "$1/work/w-hatch" "$1/work/w-front3"
    printf 'pub fn probe() -> u32 { 1 }\n' > "$1/crates/probe.rs"
    cp "$2" "$1/work/w-hatch/hatch_red.py"
    cp "$root/work/w-front3/hatch.py" "$1/work/w-front3/hatch.py" 2>/dev/null || true
    git -C "$1" init -q
    git -C "$1" add crates/probe.rs work/w-hatch/hatch_red.py >/dev/null 2>&1
    [ -f "$1/work/w-front3/hatch.py" ] && git -C "$1" add work/w-front3/hatch.py >/dev/null 2>&1
    git -C "$1" -c user.name=cf -c user.email=cf@localhost commit -q -m init
    printf '\n// A FOREIGN UNCOMMITTED EDIT. IT MUST SURVIVE.\n' >> "$1/crates/probe.rs"
}

# Drive ONE `gate.sh`'s `hatch_red_run` against ONE synthetic tree, by extracting
# the function and its helpers from that file rather than reimplementing them.
# `sed -n '/^name()/,/^}/p'` is the same extraction the gate's own A4 arm uses.
drive() {   # <gate.sh> <tree> -> prints the tuple
    {
        echo 'set -u'
        echo "repo_root=\"$2\""
        echo 'allow_dirty=0'
        echo "GRADED_DIRS='crates fixtures scripts'"
        sed -n '/^graded_tree_hash()/,/^}/p'  "$1"
        sed -n '/^crates_dirty()/,/^}/p'      "$1"
        sed -n '/^hatch_red_verdict()/,/^}/p' "$1"
        sed -n '/^hatch_red_run()/,/^}/p'     "$1"
        echo 'hatch_red_run "$repo_root/../hr.log" 2>/dev/null || true'
    } > "$scratch/drv.sh"
    sh "$scratch/drv.sh" 2>/dev/null || true
}

survived() { grep -q 'IT MUST SURVIVE' "$1/crates/probe.rs" 2>/dev/null; }

echo "=============================================================================="
echo "COUNTERFACTUAL — master's gate.sh ($BASE) vs this tip's, same synthetic tree"
echo "=============================================================================="
echo

rc=0

mktree "$scratch/base" "$scratch/hatch_red.base.py"
tb=$(drive "$scratch/gate.base.sh" "$scratch/base")
if survived "$scratch/base"; then
    echo "  BASE  $BASE : edit SURVIVED  <-- UNEXPECTED. The arms assert nothing."
    echo "        tuple: $tb"
    rc=1
else
    echo "  BASE  $BASE : edit EATEN     <-- as required; the defect is real here"
    echo "        tuple: $tb"
    echo "        (note the word: it is not DIRTY-TREE, because the --list probe"
    echo "         cleaned the tree before the interlock two lines below it looked)"
fi
echo

mktree "$scratch/tip" "$root/work/w-hatch/hatch_red.py"
tt=$(drive "$root/scripts/gate.sh" "$scratch/tip")
if survived "$scratch/tip"; then
    echo "  TIP         : edit SURVIVED  <-- as required; the row refused first"
    echo "        tuple: $tt"
else
    echo "  TIP         : edit EATEN     <-- THE FIX DID NOT TAKE"
    echo "        tuple: $tt"
    rc=1
fi
echo
if [ "$rc" -eq 0 ]; then
    echo "COUNTERFACTUAL: PASS — the base eats, the tip refuses."
else
    echo "COUNTERFACTUAL: FAIL — see above."
fi
exit "$rc"
