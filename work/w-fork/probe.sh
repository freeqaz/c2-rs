#!/bin/sh
# lane w-fork — per-TU-state probe driver.
#
# Does c2.dll capture per-TU state (cwd, TMP/TEMP, MSC_CMD_FLAGS, MSC_IDE_FLAGS,
# _CL_, INCLUDE/LIB/LIBPATH) at LoadLibrary/DllMain time?  If it does, a
# fork-server that forks AFTER LoadLibrary cannot vary that state per
# compilation and the fork point has to move earlier.
#
# Four modes, identical c2 argv and identical -Fo path in every one (the path
# string is embedded in the obj, so it must not vary):
#
#   early   real state before LoadLibrary, untouched after      -> baseline
#   late    DECOY before LoadLibrary, real state after          -> fork-server
#   never   DECOY before LoadLibrary, never corrected           -> power control
#   reverse real before LoadLibrary, DECOY after                -> power control
#
# early vs late byte-identical  => nothing that matters is baked at load.
# never / reverse must NOT both silently succeed-and-match, or the probe has no
# power over these variables and proves nothing.
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
CORPUS=${1:-$ROOT/work/w-fork/probe-corpus}
WIBO=${C2RS_WIBO:-$ROOT/../wibo/build/release/wibo}
C2DLL=$ROOT/compilers/X360/16.00.11886.00/c2.dll
PROBE=$ROOT/work/w-fork/c2probe.exe
DECOY=$ROOT/work/w-fork/decoy
mkdir -p "$DECOY"

n=0; eq_late=0; ne_late=0
never_obj=0; never_fail=0; never_eq=0
rev_obj=0; rev_fail=0; rev_eq=0
base_fail=0

for case in "$CORPUS"/*; do
  [ -f "$case/argv.txt" ] || continue
  # shellcheck disable=SC2046
  set -- $(cat "$case/argv.txt")
  rm -f "$case"/probe_*.obj
  ok=1
  for mode in early late never reverse; do
    rm -f "$case/out.obj"
    (cd "$ROOT/work/w-fork" && "$WIBO" "$PROBE" "$mode" "$DECOY" "$case" \
       "$C2DLL" "$C2DLL" "$@") >/dev/null 2>&1 || true
    if [ -s "$case/out.obj" ]; then
      mv "$case/out.obj" "$case/probe_$mode.obj"
    else
      case "$mode" in
        early|late) ok=0 ;;
        never) never_fail=$((never_fail+1)) ;;
        reverse) rev_fail=$((rev_fail+1)) ;;
      esac
    fi
  done
  if [ "$ok" = 0 ]; then base_fail=$((base_fail+1)); continue; fi
  n=$((n+1))
  [ -f "$case/probe_never.obj" ] && never_obj=$((never_obj+1))
  [ -f "$case/probe_reverse.obj" ] && rev_obj=$((rev_obj+1))
  v=$(python3 - "$case" <<'PY'
import sys, os
c = sys.argv[1]
def norm(p):
    b = bytearray(open(p, 'rb').read())
    b[4:8] = b'\0\0\0\0'          # COFF TimeDateStamp
    return bytes(b)
e = norm(os.path.join(c, 'probe_early.obj'))
l = norm(os.path.join(c, 'probe_late.obj'))
out = ['LATE_EQ' if e == l else 'LATE_NE']
for tag, f in (('NEVER', 'probe_never.obj'), ('REV', 'probe_reverse.obj')):
    p = os.path.join(c, f)
    out.append('%s_%s' % (tag, 'EQ' if os.path.exists(p) and norm(p) == e else 'NE'))
print(' '.join(out))
PY
)
  echo "$v" > "$case/probe_verdict.txt"
  case "$v" in *LATE_EQ*) eq_late=$((eq_late+1));; *) ne_late=$((ne_late+1));; esac
  case "$v" in *NEVER_EQ*) never_eq=$((never_eq+1));; esac
  case "$v" in *\ REV_EQ*) rev_eq=$((rev_eq+1));; esac
done

echo "cases with a usable early+late pair : $n   (early or late produced no obj: $base_fail)"
echo "  early == late   (fork-after-LoadLibrary is state-safe) : $eq_late / $n   [differ: $ne_late]"
echo "POWER CONTROLS — the probe must be able to see a difference:"
echo "  never  : produced an obj $never_obj, aborted $never_fail, byte-equal to early $never_eq"
echo "  reverse: produced an obj $rev_obj, aborted $rev_fail, byte-equal to early $rev_eq"
if [ "$n" -eq 0 ]; then
  echo "PROBE PRODUCED NOTHING — this is a FAILURE, not a pass"; exit 1
fi
if [ "$never_fail" -eq 0 ] && [ "$never_eq" -eq "$n" ] && [ "$rev_fail" -eq 0 ] && [ "$rev_eq" -eq "$n" ]; then
  echo "PROBE HAS NO POWER over these variables — 'early == late' proves nothing"; exit 2
fi
