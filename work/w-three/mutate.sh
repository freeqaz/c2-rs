#!/bin/sh
# w-three — the mutation controls. Colours frozen in work/w-three/PREREG.md §3
# BEFORE any of them ran.
#
# Exits NON-ZERO if any mutant comes back off its registered colour. `w-bind16`
# had 3 of 4 come back green against registered red and read its first RED off a
# stale INDEX.md fired by its own uncommitted doc — the FLATTERING direction.
# The point of freezing the colour is that the flattering direction is the one
# you cannot see.
set -u
d=work/w-three
S=$d/base.jsonl
tmp=$(mktemp -d "${TMPDIR:-/tmp}/w3mut.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
bad=0
say() { printf '%-4s %-58s registered %-8s got %-8s %s\n' "$1" "$2" "$3" "$4" "$5"; }
chk() { # id desc registered got
  if [ "$3" != "$4" ]; then say "$1" "$2" "$3" "$4" "*** OFF PREREG ***"; bad=$((bad+1));
  else say "$1" "$2" "$3" "$4" "ok"; fi
}

# ---- M1  a COMMENT-ONLY edit to the probe. Registered GREEN.
sed '1a # a comment that changes nothing' $d/price.py > "$tmp/m1.py"
python3 $d/price.py "$S" > "$tmp/a.out" 2>&1
python3 "$tmp/m1.py" "$S" > "$tmp/b.out" 2>&1
if cmp -s "$tmp/a.out" "$tmp/b.out"; then g=GREEN; else g=RED; fi
chk M1 "comment-only edit to price.py" GREEN "$g"

# ---- M2  THE POSITIVE CONTROL. A `match` TU must render DIFFERENTLY.
#          Registered FIRES.
if grep -q "gate stops at    None" "$tmp/a.out" && grep -q "gate stops at    gl-stop-26-introduced" "$tmp/a.out"; then g=FIRES; else g=SILENT; fi
chk M2 "match TU renders a DIFFERENT gate profile from the three" FIRES "$g"

# ---- M2b the factor-E match (src/Main.cpp) must render differently again.
if grep -q "ratio is NOT COMPUTED here" "$tmp/a.out"; then g=FIRES; else g=SILENT; fi
chk M2b "factor-E match refuses the ratio instead of printing 0" FIRES "$g"

# ---- M3  the FRONTIER must be a THIRD profile. Registered FIRES.
if grep -q "gate stops at    body-out-of-class" "$tmp/a.out" \
   && [ "$(grep -c 'distinct (class, gate_cause, gate_causes) profiles' "$tmp/a.out")" = 1 ] \
   && grep -q "profiles over 8 TUs: 4" "$tmp/a.out"; then g=FIRES; else g=SILENT; fi
chk M3 "frontier TU is a third/fourth distinct profile (4 over 8)" FIRES "$g"

# ---- M4  a TRUNCATED stream. Registered REFUSE.
head -400 "$S" > "$tmp/trunc.jsonl"
python3 $d/price.py "$tmp/trunc.jsonl" >/dev/null 2>"$tmp/e4"
if [ $? -ne 0 ] && grep -q "^REFUSE" "$tmp/e4"; then g=REFUSE; else g=REPORTED; fi
chk M4 "truncated scan (400 rows)" REFUSE "$g"
python3 $d/series.py "$tmp/trunc.jsonl" >/dev/null 2>"$tmp/e4b"
grep -q "^REFUSE" "$tmp/e4b" && g=REFUSE || g=REPORTED
chk M4b "truncated scan, series.py" REFUSE "$g"

# ---- M5  a TU that is not in the stream. Registered REFUSE.
python3 $d/price.py "$S" --tu src/does/not/Exist.cpp >/dev/null 2>"$tmp/e5"
grep -q "^REFUSE" "$tmp/e5" && g=REFUSE || g=REPORTED
chk M5 "a requested TU absent from the stream" REFUSE "$g"

# ---- M6  a required emit key deleted from EVERY row. Registered REFUSE.
python3 - "$S" > "$tmp/nokey.jsonl" <<'PY'
import json,sys
for l in open(sys.argv[1]):
    r=json.loads(l)
    if r.get("src"): r.get("emit",{}).pop("emit-emitted",None)
    print(json.dumps(r))
PY
python3 $d/price.py "$tmp/nokey.jsonl" >/dev/null 2>"$tmp/e6"
grep -q "^REFUSE" "$tmp/e6" && g=REFUSE || g=REPORTED
chk M6 "emit-emitted deleted from every row" REFUSE "$g"

# ---- M7  `emit_blockers = {}` is NEVER-ASKED, not nothing-blocks. FIRES iff a
#          `vocab-gap` TU with a NON-EMPTY map is exhibited beside the three.
if grep -q "the M7 witness that empty != nothing-blocks" "$tmp/a.out" \
   && grep -q "EMPTY on 5 of 8, NON-EMPTY on 3" "$tmp/a.out"; then g=FIRES; else g=SILENT; fi
chk M7 "empty blocker map is falsifiable against a same-class witness" FIRES "$g"

# ---- M8  the objdump REFUSES a non-COFF input rather than reporting 0 sections.
head -c 8 $d/objs/vec.obj > "$tmp/short.obj"
python3 $d/objdump.py "$tmp/short.obj" >/dev/null 2>"$tmp/e8"
grep -q "^REFUSE" "$tmp/e8" && g=REFUSE || g=REPORTED
chk M8 "objdump on a truncated obj" REFUSE "$g"

echo
if [ $bad -eq 0 ]; then echo "MUTANTS: 9 run, 9 as registered, 0 off prereg"; exit 0
else echo "MUTANTS: $bad OFF PREREG"; exit 1; fi
