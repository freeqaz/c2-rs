#!/bin/sh
# Standalone driver for the `resolve_corpus` arms, so they can be developed and
# run without touching the graded tree. The block that lands in
# `gate.sh --selftest` is a copy of the CASES, driving the same real function.
set -u
here="$(cd "$(dirname "$0")" && pwd)"
. "$here/../../scripts/corpus_dir.sh"

st="$here/corpus-st"; rm -rf "$st"; mkdir -p "$st"
fails=0; cases=0
t_case() {
    cases=$((cases + 1))
    if [ "$2" -eq 0 ]; then printf '  ok    %-38s %s\n' "$1" "${3:-}"
    else printf '  FAIL  %-38s %s\n' "$1" "${3:-}"; fails=$((fails + 1)); fi
}

# A FAKE TREE: `resolve_corpus` needs `scripts/sweep_gen.py` + `scripts/sweep.d`
# and nothing else, so the arms run with no toolchain and no python fragments
# from the real corpus.
mk_tree() {  # <dir> <marker>
    mkdir -p "$1/scripts/sweep.d"
    cat > "$1/scripts/sweep_gen.py" <<'PY'
import os, sys
out, frag = sys.argv[1], sys.argv[2]
n = 0
for f in sorted(os.listdir(frag)):
    body = open(os.path.join(frag, f)).read()
    for i in range(3):
        open(os.path.join(out, "%s-%04d.cpp" % (f[:-3], i)), "w").write(body + "// %d\n" % i)
        n += 1
print(n)
PY
    printf 'int f%s(int a){return a+1;}\n' "$2" > "$1/scripts/sweep.d/10-x.py"
}

gen_private() {  # <tree> <privdir>
    rm -rf "$2"; mkdir -p "$2"
    python3 "$1/scripts/sweep_gen.py" "$2" "$1/scripts/sweep.d" >/dev/null
}

# ---- 1. two trees at DIFFERENT PATHS with identical generators must produce
#         the SAME digest. This is the entire mechanism; if it fails, nothing
#         is ever shared and the lane bought nothing.
mk_tree "$st/wtA" A
mk_tree "$st/wtB" A
_da=$(corpus_digest "$st/wtA"); _db=$(corpus_digest "$st/wtB")
[ -n "$_da" ] && [ "$_da" = "$_db" ] && _r=0 || _r=1
t_case corpus-digest-is-path-independent "$_r" "wtA=$_da wtB=$_db"

# ---- 2. …and a different generator input must produce a DIFFERENT digest, so
#         two lanes with different `sweep.d` can never share a directory. This
#         is board #3249's hazard, closed by construction.
mk_tree "$st/wtC" C
_dc=$(corpus_digest "$st/wtC")
[ -n "$_dc" ] && [ "$_dc" != "$_da" ] && _r=0 || _r=1
t_case corpus-digest-separates-corpora "$_r" "wtC=$_dc"

# ---- 3. the first run PUBLISHES and the second ADOPTS.
export C2RS_CORPUS_ROOT="$st/root"
gen_private "$st/wtA" "$st/privA"
resolve_corpus "$st/wtA" "$st/privA" > "$st/outA.txt" 2>&1
[ "$C2RS_CORPUS_KIND" = shared ] && [ "$C2RS_CORPUS_DIR" = "$st/root/gen-$_da" ] && _r=0 || _r=1
t_case corpus-first-run-publishes "$_r" "kind=$C2RS_CORPUS_KIND"

gen_private "$st/wtB" "$st/privB"
resolve_corpus "$st/wtB" "$st/privB" > "$st/outB.txt" 2>&1
[ "$C2RS_CORPUS_KIND" = shared ] && [ "$C2RS_CORPUS_DIR" = "$st/root/gen-$_da" ] && _r=0 || _r=1
t_case corpus-second-worktree-adopts "$_r" "a DIFFERENT tree resolved to the SAME directory"

# ---- 4. A SHORT shared generation must be REFUSED, not graded. This is the arm
#         that matters most: the failure this repo keeps recording is an absence
#         read as a success, and a corpus missing cases is exactly that.
_one=$(find "$st/root/gen-$_da" -maxdepth 1 -name '*.cpp' | head -1)
mv "$_one" "$st/stashed.cpp"
resolve_corpus "$st/wtA" "$st/privA" > "$st/outShort.txt" 2>&1
[ "$C2RS_CORPUS_KIND" = private ] && grep -q 'REFUSED the shared generation' "$st/outShort.txt" && _r=0 || _r=1
t_case corpus-short-generation-refused "$_r" "one case removed -> kind=$C2RS_CORPUS_KIND"
mv "$st/stashed.cpp" "$_one"

# ---- 5. An EXTRA case is refused too. A superset is not a subset problem and a
#         count-shaped check would pass it if one were also removed.
printf 'int extra(){return 0;}\n' > "$st/root/gen-$_da/zz-extra-9999.cpp"
resolve_corpus "$st/wtA" "$st/privA" > "$st/outExtra.txt" 2>&1
[ "$C2RS_CORPUS_KIND" = private ] && _r=0 || _r=1
t_case corpus-extra-case-refused "$_r" "kind=$C2RS_CORPUS_KIND"
rm -f "$st/root/gen-$_da/zz-extra-9999.cpp"

# ---- 6. A case whose CONTENT differs by one byte is refused. Same count, same
#         names — the only thing a name/count check cannot see, and the only
#         thing that would silently change what the gate grades.
_one=$(find "$st/root/gen-$_da" -maxdepth 1 -name '*.cpp' | head -1)
cp "$_one" "$st/orig.cpp"
printf '// tampered\n' >> "$_one"
resolve_corpus "$st/wtA" "$st/privA" > "$st/outTamper.txt" 2>&1
[ "$C2RS_CORPUS_KIND" = private ] && _r=0 || _r=1
t_case corpus-tampered-case-refused "$_r" "same names, same count, one byte -> kind=$C2RS_CORPUS_KIND"
cp "$st/orig.cpp" "$_one"

# ---- 7. …and it RECOVERS: once the tampering is undone the very next run
#         adopts again. A check that latches would be a check nobody can clear.
resolve_corpus "$st/wtA" "$st/privA" > "$st/outBack.txt" 2>&1
[ "$C2RS_CORPUS_KIND" = shared ] && _r=0 || _r=1
t_case corpus-recovers-after-repair "$_r" "kind=$C2RS_CORPUS_KIND"

# ---- 8. the off switch, which is the cold control every A/B in the rung used.
C2RS_NO_SHARED_CORPUS=1 resolve_corpus "$st/wtA" "$st/privA" > "$st/outOff.txt" 2>&1
grep -q 'C2RS_NO_SHARED_CORPUS=1' "$st/outOff.txt" && _r=0 || _r=1
t_case corpus-off-switch-stays-private "$_r" ""

# ---- 9. an unwritable root degrades to private, never to a failure. Every
#         error path in this helper must leave the pre-existing behaviour.
C2RS_CORPUS_ROOT=/proc/nonexistent/corpus resolve_corpus "$st/wtA" "$st/privA" > "$st/outRO.txt" 2>&1
_rc=$?
[ "$_rc" -eq 0 ] && [ "$C2RS_CORPUS_KIND" = private ] && _r=0 || _r=1
t_case corpus-unwritable-root-degrades "$_r" "rc=$_rc kind=$C2RS_CORPUS_KIND"

# ---- 10. a tree whose generator cannot run at all degrades to private.
mkdir -p "$st/wtBad/scripts/sweep.d"
printf 'import sys; sys.exit(3)\n' > "$st/wtBad/scripts/sweep_gen.py"
printf 'x\n' > "$st/wtBad/scripts/sweep.d/10-x.py"
gen_private "$st/wtA" "$st/privBad"
resolve_corpus "$st/wtBad" "$st/privBad" > "$st/outBad.txt" 2>&1
_rc=$?
[ "$_rc" -eq 0 ] && [ "$C2RS_CORPUS_KIND" = private ] && _r=0 || _r=1
t_case corpus-broken-generator-degrades "$_r" "rc=$_rc kind=$C2RS_CORPUS_KIND"

# ---- 11. TWO PUBLISHERS AT ONCE: the loser must discard, never clobber, and
#          both must end up on the same directory. This is the race the design
#          replaces a lock with, so it is driven rather than argued.
rm -rf "$st/root2"; export C2RS_CORPUS_ROOT="$st/root2"
gen_private "$st/wtA" "$st/privR1"; gen_private "$st/wtB" "$st/privR2"
( resolve_corpus "$st/wtA" "$st/privR1" > "$st/outR1.txt" 2>&1; echo "$C2RS_CORPUS_KIND $C2RS_CORPUS_DIR" > "$st/r1" ) &
( resolve_corpus "$st/wtB" "$st/privR2" > "$st/outR2.txt" 2>&1; echo "$C2RS_CORPUS_KIND $C2RS_CORPUS_DIR" > "$st/r2" ) &
wait
_k1=$(cut -d' ' -f1 < "$st/r1"); _k2=$(cut -d' ' -f1 < "$st/r2")
_p1=$(cut -d' ' -f2 < "$st/r1"); _p2=$(cut -d' ' -f2 < "$st/r2")
_leftover=$(find "$st/root2" -maxdepth 1 -name '.tmp-*' | wc -l)
_gens=$(find "$st/root2" -maxdepth 1 -type d -name 'gen-*' | wc -l)
[ "$_k1" = shared ] && [ "$_k2" = shared ] && [ "$_p1" = "$_p2" ] \
    && [ "$_leftover" -eq 0 ] && [ "$_gens" -eq 1 ] && _r=0 || _r=1
t_case corpus-concurrent-publish-converges "$_r" \
    "kinds=$_k1/$_k2 same-dir=$([ "$_p1" = "$_p2" ] && echo yes || echo no) generations=$_gens tmp-left=$_leftover"

# ---- 12. an ALREADY-PUBLISHED generation is never rewritten. Immutability is
#          the property that removes the lock, so it is asserted, not assumed.
_mt_before=$(find "$st/root2" -maxdepth 1 -type d -name 'gen-*' -printf '%T@\n')
resolve_corpus "$st/wtA" "$st/privR1" > /dev/null 2>&1
_mt_after=$(find "$st/root2" -maxdepth 1 -type d -name 'gen-*' -printf '%T@\n')
[ "$_mt_before" = "$_mt_after" ] && _r=0 || _r=1
t_case corpus-published-generation-immutable "$_r" "mtime unchanged across a second resolve"

unset C2RS_CORPUS_ROOT
echo
echo "resolve_corpus: $cases cases, $fails failed"
[ "$fails" -eq 0 ]
