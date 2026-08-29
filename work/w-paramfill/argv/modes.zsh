#!/usr/bin/env zsh
# w-paramfill: does ANY mode this project compiles pass -Fl / -ltcg / -optref
# to c2?  Those three are the switches GATE 1 (DAT_10c462c4) and its two
# in-model companions read.  `cl /Bd` prints each pass's own command line.
#
# INSTRUMENT NOTE, inherited from w-inlswitch SS8.1: zsh does NOT word-split an
# unquoted parameter expansion, so a multi-flag mode written as "$m" reaches
# cl.exe as ONE argument and only its first flag is parsed.  Use ${=m}.
set -u
root=${1:?repo root}
wibo=${C2RS_WIBO:-$root/../../../../wibo/build/release/wibo}
cl=$root/compilers/X360/16.00.11886.00/cl.exe
[[ -x $wibo && -f $cl ]] || { print "SKIP: toolchain absent (wibo=$wibo cl=$cl)"; exit 0 }

d=$(mktemp -d); trap "rm -rf $d" EXIT
cat > $d/t.cpp <<'EOF'
static int add3(int a, int b, int c) { return a + b + c; }
int use(int x) { return add3(x, x + 1, x + 2); }
EOF

modes=()
while read -r slug rest; do
  [[ -z $slug || $slug == \#* ]] && continue
  modes+=("$rest")
done < $root/scripts/lanes.txt
# controls + the two modes that could plausibly carry a module list
modes+=("/Os" "/Ot" "/Ox /Ob0" "/GL" "/GL /O2" "/O2 /GL /Gy" "/FAsc" "/O2 /FAsc" "/Ox /GL /EHsc")

print "# c2's own command line per cl mode.  Lane w-paramfill, 2026-08-29."
print "# cl 16.00.11886.00 under wibo; every row is 'cl /Bd <mode> /GS- /c t.cpp'."
print "# The IL path is elided as <IL>.  zsh word-split is explicit (\${=m})."
print ""
n=0
for m in $modes; do
  out=$(cd $d && "$wibo" "$cl" /Bd ${=m} /GS- /c "z:$d/t.cpp" 2>&1)
  line=$(print -r -- "$out" | grep -a 'c2\.dll' | head -1 \
         | sed -E 's#[^ ]*[\\/]c2\.dll#c2.dll#; s#-il [^ ]*#-il <IL>#; s#-f [^ ]*#-f <SRC>#; s#z:/[^ ]*#<PATH>#g')
  [[ -z $line ]] && line='(no c2 pass -- cl did not run the back end)'
  printf '%-24s %s\n' "cl $m" "$line"
  n=$((n+1))
done > $d/rows.txt
cat $d/rows.txt
print ""
print "# --- THE ANSWER, by grep over the $n rows above ---"
for tok in ' -Fl' ' -ltcg' ' -optref' ' -GL' ' -FAsc' ' -Fa' ' -Fs' ' -FA'; do
  c=$(grep -c -- "$tok" $d/rows.txt)
  printf "#   %-10s : %d hit(s) over %d mode rows\n" "'$tok'" "$c" "$n"
done
