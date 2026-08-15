#!/bin/sh
# scan_mutants.sh — the mutation controls that are graded on REAL IL rather than
# on a pinned segment: each one is patched in, the 878-TU workload is rescanned,
# and the published key it should move is read back.
#
# `mutate.sh`'s ten are unit-scale and prove the clauses are exercised. These
# three prove the CORPUS-SCALE keys are load-bearing — a key nobody can make
# move is a key nobody is measuring. Same rule as mutate.sh: the tree must be
# clean, because revert is `git checkout --`.
set -e
cd "$(dirname "$0")/../.."

IL=crates/c2-il/src/func
HAR=crates/c2-harness/src/gap
D=work/w-stmt5

revert() { git checkout -- "$IL" "$HAR" 2>/dev/null || true; }
trap revert EXIT

if [ -n "$(git status --porcelain -- "$IL" "$HAR")" ]; then
  echo "REFUSING: dirty tree; revert is 'git checkout --'. Commit first." >&2
  exit 2
fi

patch() { python3 -c "
import sys
p, old, new = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert old in s, 'MUTANT DID NOT APPLY: ' + p
open(p, 'w').write(s.replace(old, new, 1))
" "$1" "$2" "$3"; }

# grade <id> <key> <baseline value> <predicted>
grade() {
  id="$1"; key="$2"; base="$3"; want="$4"
  cargo build --release -p c2-harness >/dev/null 2>&1
  ./"$D"/scan.sh "mut_$id" >/dev/null 2>&1
  got="$(awk -v k="$key" '$1==k {print $2}' "$D/mut_$id.keys")"
  if [ "$got" = "$base" ]; then colour=GREEN; else colour=RED; fi
  if [ "$colour" = "$want" ]; then v="AS REGISTERED"; else v="*** OFF PREREG ***"; fi
  printf '%-4s %-38s base %-10s got %-10s %-5s  %s\n' "$id" "$key" "$base" "${got:-ABSENT}" "$colour" "$v"
  revert
}

s1() { # the series stops counting undecoded bodies: the partition breaks
  patch "$HAR/scan.rs" \
    '                                *res.emit
                                    .entry(format!(
                                        "emit-cflow-shape|{}",
                                        cflow_series_bucket(&f.cflow)
                                    ))
                                    .or_insert(0) += 1;' \
    '                                if f.cflow.starts_with("cflow-") {
                                    *res.emit
                                        .entry(format!(
                                            "emit-cflow-shape|{}",
                                            cflow_series_bucket(&f.cflow)
                                        ))
                                        .or_insert(0) += 1;
                                }'
  grade S1 emit-cflow-shape-accounted 113612 RED
}

s2() { # the fallthrough name loses its `admits()` gate — the bug this lane hit
  patch "$IL/census.rs" \
    'if v.admits() && body::shapes::step5::CfgAdmit::has_fallthrough_epilogue(&scan) {' \
    'if body::shapes::step5::CfgAdmit::has_fallthrough_epilogue(&scan) {'
  grade S2 step5-refuse-unmodeled-operand-BLOCKED 1495664 RED
}

s3() { # the liveness guard back to the `||` it shipped as for one commit
  patch "$IL/body/shapes/step5.rs" \
    'defs == 0 && refs == 0' \
    'defs == 0 || refs == 0'
  grade S3 step5-consistency-alarms 0 RED
}

s4() { # REGISTERED GREEN: a comment-only edit inside the predicate
  patch "$IL/body/shapes/step5.rs" \
    'let t: &LabelTable = &scan.labels;' \
    '// S4: a comment, and nothing else
        let t: &LabelTable = &scan.labels;'
  grade S4 step5-accounted 2410886 GREEN
}

s5() { # the boundary rows back in `fn_cflow`, where they doubled a published key
  patch "$HAR/scan.rs" \
    '*res.fn_cfg_admit
                .entry(format!(
                    "{}|{}",' \
    '*res.fn_cflow
                .entry(format!(
                    "cflow-{}|{}",'
  grade S5 cflow-residue-inclass-offclass 517425 RED
}

case "${1:-all}" in
  all) s1; s2; s3; s4; s5 ;;
  *) "s$1" ;;
esac
