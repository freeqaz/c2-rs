#!/bin/sh
# w-vocabgap — the mutation controls.  Colours were frozen in
# work/w-vocabgap/PREREG.md §4 BEFORE any of them ran, greens included.
#
# A zero-delta is a measurement only if the guard producing it can be made to
# fail.  The one that matters is M6: an analyzer whose S(t) came back empty
# would report "845 of 845 key-covered by the empty set" -- the most optimistic
# answer available and the one that looks most like success.  w-loo's
# zero-reach guard is the precedent: without it its mutant printed 52 margins
# of 0 and read as a clean null.
#
# Every mutant is an env hook inside sets.py or a mutated INPUT, never an
# uncommitted patch to crates/.  This lane's `crates/` delta is required-zero
# and mutate.sh cannot break it, which is deliberate: w-stmt5's runner reverted
# with `git checkout --` and its FIRST revert deleted the tests that would have
# reddened two mutants (that lane's §6 note 1).
set -u
d=work/w-vocabgap
base="$d/base.out"
pass=0; fail=0

say() { printf '%-6s %-44s predicted %-8s got %-8s %s\n' "$1" "$2" "$3" "$4" "$5"; }
grade() { # id desc predicted actual
  if [ "$3" = "$4" ]; then pass=$((pass+1)); say "$1" "$2" "$3" "$4" "OK"; \
  else fail=$((fail+1)); say "$1" "$2" "$3" "$4" "** OFF PREREG **"; fi
}

# --- a mutant that CHANGES a published number is RED; one that leaves the
#     whole report byte-identical is GREEN; one that exits non-zero is REFUSE.
run_mut() {  # name -> prints RED / GREEN / REFUSE
  out=$(C2RS_VG_MUTANT="$1" python3 $d/sets.py "${2:-base}" 2>&1)
  rc=$?
  if [ $rc -ne 0 ]; then echo REFUSE; return; fi
  if [ "$out" = "$(cat "$base")" ]; then echo GREEN; else echo RED; fi
}

[ -f "$base" ] || { echo "no $base -- run: python3 $d/sets.py base > $base"; exit 2; }

echo "== w-vocabgap mutation controls =="

grade M1 "coverage S(t)<=G  ->  S(t) intersect G nonempty" RED    "$(run_mut intersect)"
grade M2 "scores all 878 rows, not the 845 vocab-gap"      RED    "$(run_mut allrows)"
grade M3 "S(t) as a mass-weighted multiset, not a set"     RED    "$(run_mut mass)"
grade M5 "the totality guard: sum != fnbyte-refused-parse" REFUSE "$(run_mut totality)"

# --- M4, the GREEN control: a comment-only edit inside the analyzer.
cp $d/sets.py $d/sets.py.m4bak
printf '\n# M4: a comment-only edit, registered GREEN in PREREG.md §4.\n' >> $d/sets.py
m4=$(python3 $d/sets.py base 2>&1)
mv $d/sets.py.m4bak $d/sets.py
if [ "$m4" = "$(cat "$base")" ]; then g=GREEN; else g=RED; fi
grade M4 "a comment-only edit inside the analyzer"         GREEN  "$g"

# --- M6, THE ONE THAT MATTERS: every vocab-gap row's emit_blockers emptied.
#     Must REFUSE.  It must NOT print "845 of 845 covered by the empty set".
python3 - <<'PY'
import json
o=open('work/w-vocabgap/m6.jsonl','w')
for l in open('work/w-vocabgap/base.jsonl'):
    r=json.loads(l)
    if not r.get('record') and r.get('class')=='vocab-gap':
        r['emit_blockers']={}
    o.write(json.dumps(r)+'\n')
PY
cp $d/base.log $d/m6.log; cp $d/base.tsv $d/m6.tsv
grade M6 "every vocab-gap row's emit_blockers emptied"     REFUSE "$(run_mut '' m6)"

# --- M7a/M7b: the graded-TU floor, and the proof that it is load-bearing.
head -n 401 $d/base.jsonl > $d/m7.jsonl
cp $d/base.log $d/m7.log; cp $d/base.tsv $d/m7.tsv
grade M7a "a truncated (400-TU) stream, floor ON"          REFUSE "$(run_mut '' m7)"
# With the floor removed the truncated stream must still be caught -- by the
# TOTALITY guard, one layer down.  Two independent guards, not one.
grade M7b "the same stream with the floor REMOVED"         REFUSE "$(run_mut nofloor m7)"

rm -f $d/m6.jsonl $d/m6.log $d/m6.tsv $d/m7.jsonl $d/m7.log $d/m7.tsv
echo
echo "mutants: $((pass+fail)) run, $pass as registered, $fail off prereg"
[ $fail -eq 0 ] || exit 1
