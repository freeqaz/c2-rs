#!/bin/sh
# mutate.sh <id>|all — w-stmt5's mutation controls.
#
# Each mutant is a one-hunk source patch applied with `python3 -c`, graded by
# the tests it should redden, then reverted with `git checkout --`. Colours were
# registered in work/w-stmt5/PREREG.md §5 BEFORE any of them ran, M7 GREEN
# included.
#
# A mutant that does not apply is a FAILURE of the control, not a pass: the
# patch text is asserted against the file, so a drifted source aborts loudly
# instead of silently grading nothing.
#
# **REVERT IS `git checkout --`, SO THE TREE MUST BE CLEAN FIRST**, and this
# script refuses to run otherwise. Two of these mutants read GREEN on their
# first run for exactly this reason: the tests that would have reddened them
# were still uncommitted, and the FIRST revert deleted them, so every later
# mutant was graded by a suite two tests short. That is a control silently
# grading nothing, which is the failure this file is supposed to catch, caught
# on itself.
set -e
cd "$(dirname "$0")/../.."

IL=crates/c2-il/src/func/body/shapes
HAR=crates/c2-harness/src/gap

revert() { git checkout -- "$IL" "$HAR" 2>/dev/null || true; }
trap revert EXIT

if [ -n "$(git status --porcelain -- "$IL" "$HAR")" ]; then
  echo "REFUSING: $IL / $HAR are dirty. This script reverts with 'git checkout --'," >&2
  echo "so uncommitted work here would be DELETED and every mutant after the first" >&2
  echo "would be graded against a source tree it did not write. Commit first." >&2
  exit 2
fi

patch() { python3 -c "
import sys
p, old, new = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert old in s, 'MUTANT DID NOT APPLY: ' + p
open(p, 'w').write(s.replace(old, new, 1))
" "$1" "$2" "$3"; }

grade() { # grade <id> <predicted> <test filter> [package]
  id="$1"; want="$2"; filt="$3"; pkg="${4:-c2-il}"
  if cargo test --release -p "$pkg" "$filt" >/dev/null 2>&1; then got=GREEN; else got=RED; fi
  if [ "$got" = "$want" ]; then verdict="AS REGISTERED"; else verdict="*** OFF PREREG ***"; fi
  printf '%-4s predicted %-5s  got %-5s  %s\n' "$id" "$want" "$got" "$verdict"
  revert
}

m1() { # the series bucketer shards the undecoded bodies by their cf-* key
  patch "$HAR/classify.rs" \
    'return "undecoded|-".to_string();' \
    'return format!("{class}|-");'
  grade M1 RED the_series_bucketer c2-harness
}

m2() { # ends_with -> contains, the residue split
  patch "$HAR/classify.rs" \
    'match rest.strip_suffix("+expr-modeled") {
        Some(shape) => format!("{shape}|modeled"),
        None => format!("{rest}|expr"),
    }' \
    'match rest.find("+expr-modeled") {
        Some(i) => format!("{}|modeled", &rest[..i]),
        None => format!("{rest}|expr"),
    }'
  grade M2 RED the_series_bucketer c2-harness
}

m3() { # the back-edge clause is dropped -- a loop is admitted
  patch "$IL/step5.rs" \
    'if t.back_edges() > 0 {
            return CfgVerdict::BackEdge;
        }' \
    '// M3'
  grade M3 RED step5
}

m4() { # the shape clause is dropped -- a switch is scored off a partial map
  patch "$IL/step5.rs" \
    'if cf.shape == CfShape::Switch {
            return CfgVerdict::Switch;
        }' \
    '// M4'
  grade M4 RED step5
}

m5() { # the residue clause is dropped
  patch "$IL/step5.rs" \
    'if cf.residue != CfResidue::Modeled {
            return CfgVerdict::UnmodeledOperand;
        }' \
    '// M5'
  grade M5 RED step5
}

m6() { # the decoded-first clause is dropped -- a partial walk is scored
  patch "$IL/step5.rs" \
    'let Ok(cf) = &scan.body else {
            return CfgVerdict::Undecoded;
        };' \
    'let cf = match &scan.body {
            Ok(c) => c,
            // The prefix the walk got to, scored as if it were the body.
            Err(_) => &super::control_flow::CfBody {
                shape: CfShape::Straight,
                residue: CfResidue::Modeled,
            },
        };'
  grade M6 RED step5
}

m7() { # REGISTERED GREEN: swap the two clauses that are NOT load-bearing
  patch "$IL/step5.rs" \
    'if t.back_edges() > 0 {
            return CfgVerdict::BackEdge;
        }
        if cf.residue != CfResidue::Modeled {
            return CfgVerdict::UnmodeledOperand;
        }' \
    'if cf.residue != CfResidue::Modeled {
            return CfgVerdict::UnmodeledOperand;
        }
        if t.back_edges() > 0 {
            return CfgVerdict::BackEdge;
        }'
  # The ADMITTED SET is unchanged -- that is the registered green.
  grade M7 GREEN every_admitted_body_satisfies
}

m7b() { # the same swap, graded by the VERDICT test rather than the admit set
  patch "$IL/step5.rs" \
    'if t.back_edges() > 0 {
            return CfgVerdict::BackEdge;
        }
        if cf.residue != CfResidue::Modeled {
            return CfgVerdict::UnmodeledOperand;
        }' \
    'if cf.residue != CfResidue::Modeled {
            return CfgVerdict::UnmodeledOperand;
        }
        if t.back_edges() > 0 {
            return CfgVerdict::BackEdge;
        }'
  grade M7b GREEN the_back_edge_clause_refuses_a_real_while_loop
}

m8() { # the ORDER that IS load-bearing: unresolved after back_edges
  patch "$IL/step5.rs" \
    'if t.unresolved() > 0 {
            return CfgVerdict::UnresolvedTarget;
        }
        if t.back_edges() > 0 {
            return CfgVerdict::BackEdge;
        }' \
    'if t.back_edges() > 0 {
            return CfgVerdict::BackEdge;
        }
        if t.unresolved() > 0 {
            return CfgVerdict::UnresolvedTarget;
        }'
  grade M8 RED unresolved_outranks_back_edge
}

m9() { # the liveness guard back to the || it was written with
  patch "$IL/step5.rs" \
    'defs == 0 && refs == 0' \
    'defs == 0 || refs == 0'
  grade M9 RED a_body_can_define_its_epilogue_label
}

case "${1:-all}" in
  all) m1; m2; m3; m4; m5; m6; m7; m7b; m8; m9 ;;
  *) "m$1" ;;
esac
