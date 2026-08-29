#!/bin/sh
# w-inlclause controls (#3336). Every check this lane quotes a GREEN from is
# watched RED first, on a planted defect chosen to exercise ONE rule.
set -u
cd "$(dirname "$0")/../.." || exit 1
R=work/w-inlclause/read_state.py
echo "=== read_state.py: one plant per rule ==============================="
for p in \
  "C7=read=R3          | GRAMMAR: R3 with a live readcite and a real blocker" \
  "C7=readcite=nope.md#0x1 | CITE: the cited path does not exist" \
  "C7=readcite=docs/whitebox/ref/P_INLINE.md#0xdeadbeef | CITE: the path exists and the anchor does not" \
  "C14=read=R2         | GRAMMAR: state R-derived may not carry read R2" \
  "C7=blocker=because  | VALUE: a free-text blocker" \
  "C7=blocker=none     | GRAMMAR: R1 + absent + no blocker is an unadopted derivable clause" \
  "C21=blocker=none    | GRAMMAR: an unexercisable row must carry n-a" \
  ; do
  plant=$(echo "$p" | cut -d'|' -f1 | sed 's/ *$//')
  why=$(echo "$p" | cut -d'|' -f2-)
  out=$(python3 "$R" --plant "$plant" 2>&1)
  v=$(echo "$out" | tail -1 | sed 's/.*READ-STATE: \([A-Z]*\).*/\1/')
  f=$(echo "$out" | grep -c '  FAIL ')
  printf '  %-56s %s (%s FAIL line(s)) --%s\n' "$plant" "$v" "$f" "$why"
done
echo "  CONTROL: the unplanted table must be GREEN"
printf '    unplanted: %s\n' "$(python3 "$R" 2>&1 | tail -1)"

echo
echo "=== read_scan.py: a planted address that pins nothing ==============="
out=$(python3 work/w-inlclause/read_scan.py --plant C7=deadbeef 2>&1)
echo "$out" | grep -E 'C7|PIN-SCAN'

echo
echo "=== check_table.py: w-inlmetric's own control, re-watched on this tip"
python3 work/w-inlmetric/check_table.py --plant C16=10b5c06b 2>&1 | grep -E 'C16|CONFORMANCE-CHECK'
python3 work/w-inlmetric/check_table.py 2>&1 | tail -1

echo
echo "=== the adopted clause: mutate its constant, the domain must move ==="
S=crates/c2-core/src/splice.rs
cp "$S" /tmp/w-inlclause-splice.bak
for m in "INLINE_MAXLEVEL_UNBOUNDED: i64 = 255|INLINE_MAXLEVEL_UNBOUNDED: i64 = 256" \
         "max_level: 2, ..BUDGET_C2|max_level: 3, ..BUDGET_C2"; do
  a=$(echo "$m" | cut -d'|' -f1); b=$(echo "$m" | cut -d'|' -f2)
  perl -0pi -e "s/\Q$a\E/$b/" "$S"
  r=$(cargo test -p c2-core --lib surface::tests::the_decision_surface_domain_matches 2>&1)
  n=$(echo "$r" | grep -o 'MOVED — [0-9]* line' | grep -o '[0-9]*')
  if echo "$r" | grep -q 'test result: ok'; then
    printf '  %-46s GREEN -- NOT COVERED, this is a false coverage claim (#3746)\n' "$a"
  else
    printf '  %-46s RED -- %s domain line(s) moved\n' "$a" "${n:-?}"
  fi
  # `cp`/`mv` preserves the BACKUP's older mtime, so cargo relinks the MUTATED
  # binary for the next check and the control lies in the safe direction
  # (#3767, w-inlbudget SS5.4). `touch` is what makes the restore real.
  cp /tmp/w-inlclause-splice.bak "$S"; touch "$S"
done
echo "  CONTROL: the restored tree must be GREEN"
cargo test -p c2-core --lib surface::tests::the_decision_surface_domain_matches 2>&1 | grep 'test result'
# NOT `git diff`: this lane edits splice.rs, so a diff against HEAD is expected
# and would make the restore check unable to fail. Compare against the BACKUP.
cmp -s /tmp/w-inlclause-splice.bak "$S" && echo "  restored tree: byte-identical to the pre-mutation backup" || echo "  RESTORE FAILED"
