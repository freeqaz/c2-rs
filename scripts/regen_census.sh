#!/bin/sh
# Regenerate the workload censuses that docs/OBJ_DATA_BSS_SHAPE.md grades against.
#
#   work/w-bss/census/sections.jsonl   the allocator's OUTPUT  (committed, expensive)
#   work/w-bss2/glcensus.jsonl         the allocator's INPUT   (untracked, cheap)
#
# The two are NOT independent and cannot be built in either order: glcensus.py
# reads sections.jsonl, so the cheap file sits downstream of the expensive one.
#
# Cost, measured: sections.jsonl needs the real toolchain AND the sibling
# dc3-decomp source tree, and re-materializes ~102 MB of objs (871 x ~143 KB) to
# produce 12.5 MB of census. glcensus.jsonl is front-end only, ~2 min, and writes
# no obj and no IL. That asymmetry is why one is committed and the other is not.
#
# Usage:
#   scripts/regen_census.sh                 both, then delete the objs
#   scripts/regen_census.sh --sections      sections.jsonl only
#   scripts/regen_census.sh --gl            glcensus.jsonl only (needs sections.jsonl)
#   scripts/regen_census.sh --keep-objs     leave work/w-bss/census/objs in place
#   scripts/regen_census.sh --jobs N        parallel compiles (default: nproc-2, max 16)
#   scripts/regen_census.sh --timeout SECS  overall deadline (default 7200)
#   scripts/regen_census.sh --allow-dirty   record a census against a DIRTY corpus
#   scripts/regen_census.sh --allow-move    record one against a corpus that MOVED
#
# Provenance
# ----------
# Both censuses write a `<file>.prov` sidecar naming the dc3 commit, the dirty
# flag, the corpus directory, the flags hash and (for glcensus) the
# sections.jsonl hash they joined against.  `work/w-bss2/grade.py` REFUSES to
# grade a pair whose stamps disagree.
#
# The corpus HEAD is snapshotted BEFORE the compiles and re-checked AFTER them:
# `../dc3-decomp` is a live repo that other agents merge into, and lane w-repro
# spent an entire lane finding out that it took 40+ commits during a 30-minute
# measurement window.  A sections census takes tens of minutes; being straddled
# by a merge is the normal case, not the unlucky one, and it is an ERROR here.
#
# Env:
#   C2RS_DC3_SRC   dc3-decomp source tree. Defaults to ../dc3-decomp if present.
#
# Degrades cleanly: with no toolchain it prints "SKIP: toolchain absent" and
# exits 0, per the project rule that nothing panics when the compilers are gone.
set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

DO_SECTIONS=1
DO_GL=1
KEEP_OBJS=0
JOBS=""
TIMEOUT=7200
ALLOW_DIRTY=0
ALLOW_MOVE=0

while [ $# -gt 0 ]; do
  case "$1" in
    --sections)  DO_GL=0 ;;
    --gl)        DO_SECTIONS=0 ;;
    --keep-objs) KEEP_OBJS=1 ;;
    --allow-dirty) ALLOW_DIRTY=1 ;;
    --allow-move)  ALLOW_MOVE=1 ;;
    --jobs)      shift; JOBS="${1:?--jobs needs a number}" ;;
    --timeout)   shift; TIMEOUT="${1:?--timeout needs seconds}" ;;
    -h|--help)   sed -n '2,30p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *)           echo "regen_census: unknown option '$1'" >&2; exit 2 ;;
  esac
  shift
done

if [ -z "$JOBS" ]; then
  n=$(nproc 2>/dev/null || echo 4)
  JOBS=$((n - 2)); [ "$JOBS" -lt 1 ] && JOBS=1
  [ "$JOBS" -gt 16 ] && JOBS=16
fi

CENSUS_DIR="$ROOT/work/w-bss/census"
OBJS="$CENSUS_DIR/objs"
SECTIONS="$CENSUS_DIR/sections.jsonl"
GLCENSUS="$ROOT/work/w-bss2/glcensus.jsonl"
FILES="$ROOT/work/dc3-workload/files.txt"

# ---------------------------------------------------------------- prerequisites

: "${C2RS_DC3_SRC:=$ROOT/../dc3-decomp}"
export C2RS_DC3_SRC

if [ ! -d "$C2RS_DC3_SRC" ]; then
  echo "SKIP: toolchain absent (no dc3 source tree at $C2RS_DC3_SRC;"
  echo "      set C2RS_DC3_SRC to override)"
  exit 0
fi
if [ ! -f "$FILES" ]; then
  echo "SKIP: toolchain absent (no workload list at $FILES)" >&2
  exit 0
fi

echo "regen_census: root      $ROOT"
echo "regen_census: dc3 src   $C2RS_DC3_SRC"
echo "regen_census: jobs      $JOBS"
echo "regen_census: deadline  ${TIMEOUT}s"

# ------------------------------------------------------------------ provenance
# Snapshot the corpus BEFORE anything compiles. census.py re-checks it after and
# refuses to stamp a census whose corpus moved underneath it; passing the
# snapshot down is what makes the stamp cover the compile phase rather than only
# the aggregation (census.py records which, as `begin_scope`).
PROV_BEGIN="$CENSUS_DIR/.prov-begin.json"
mkdir -p "$CENSUS_DIR"
python3 - "$C2RS_DC3_SRC" "$PROV_BEGIN" <<'PY'
import json, sys, os
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(sys.argv[2])),
                                "..", "..", "w-bss2"))
import prov
b = prov.begin(sys.argv[1])
prov.begin_write(sys.argv[2], b)
print("regen_census: corpus    %s%s"
      % (b["head"] or "UNVERSIONED", "  *** DIRTY ***" if b["dirty"] else ""))
PY
export C2RS_PROV_BEGIN="$PROV_BEGIN"
[ "$ALLOW_DIRTY" = 1 ] && export C2RS_PROV_ALLOW_DIRTY=1
[ "$ALLOW_MOVE" = 1 ] && export C2RS_PROV_ALLOW_MOVE=1

START=$(date +%s)
deadline_hit() {
  now=$(date +%s)
  [ $((now - START)) -ge "$TIMEOUT" ]
}

# ------------------------------------------------------------------- sections

if [ "$DO_SECTIONS" = 1 ]; then
  echo "regen_census: building c2rs (release)"
  cargo build --release -p c2-harness >/dev/null

  # one.sh needs the binary it shells out to.
  if [ ! -x "$ROOT/target/release/c2rs" ]; then
    echo "SKIP: toolchain absent (target/release/c2rs did not build)" >&2
    exit 0
  fi

  total=$(wc -l < "$FILES" | tr -d ' ')
  echo "regen_census: compiling $total TUs -> $OBJS (~102 MB, deleted afterwards"
  echo "              unless --keep-objs)"
  mkdir -p "$OBJS"

  # Bounded fan-out WITHOUT xargs -P racing the deadline check: run in batches of
  # $JOBS and wait on the PIDs we launched. Never pgrep -- a pattern matching our
  # own argv would spin forever (see CLAUDE.md).
  done_n=0
  pids=""
  while IFS= read -r f; do
    [ -z "$f" ] && continue
    if deadline_hit; then
      echo "TIMEOUT after ${TIMEOUT}s at $done_n/$total compiles -- partial objs left in $OBJS" >&2
      exit 3
    fi
    sh "$CENSUS_DIR/one.sh" "$f" &
    pids="$pids $!"
    done_n=$((done_n + 1))
    if [ $((done_n % JOBS)) -eq 0 ]; then
      for p in $pids; do wait "$p" || true; done
      pids=""
      printf '\rregen_census: %d/%d compiled' "$done_n" "$total"
    fi
  done < "$FILES"
  for p in $pids; do wait "$p" || true; done
  printf '\rregen_census: %d/%d compiled\n' "$done_n" "$total"

  # one.sh leaves a .err beside any TU that produced no obj. Seven TUs are
  # EXPECTED to fail -- they die in c1xx before c2 is reached, so 871 is the
  # terminal denominator, not an instrument limit (OBJ_DATA_BSS_SHAPE.md 11).
  nobj=$(find "$OBJS" -maxdepth 1 -name '*.obj' | wc -l | tr -d ' ')
  nerr=$(find "$OBJS" -maxdepth 1 -name '*.err' | wc -l | tr -d ' ')
  echo "regen_census: $nobj objs, $nerr failures (7 expected: C2084/C2512/C1189/C1083)"
  if [ "$nerr" -gt 7 ]; then
    echo "regen_census: WARNING -- $nerr failures exceeds the expected 7." >&2
    echo "              That is a finding, not noise. Inspect $OBJS/*.err before" >&2
    echo "              trusting any number derived from this census." >&2
  fi

  echo "regen_census: aggregating -> sections.jsonl"
  ( cd "$CENSUS_DIR" && python3 census.py )
  [ -s "$SECTIONS" ] || { echo "regen_census: census.py produced no output" >&2; exit 1; }
  echo "regen_census: sections.jsonl  $(wc -l < "$SECTIONS" | tr -d ' ') records, $(du -h "$SECTIONS" | cut -f1)"

  if [ "$KEEP_OBJS" = 0 ]; then
    echo "regen_census: deleting $OBJS (pass --keep-objs to retain)"
    rm -rf "$OBJS"
  fi
fi

# ------------------------------------------------------------------- glcensus

if [ "$DO_GL" = 1 ]; then
  if [ ! -s "$SECTIONS" ]; then
    echo "regen_census: cannot build glcensus.jsonl -- it reads $SECTIONS," >&2
    echo "              which does not exist. Run without --gl first." >&2
    exit 1
  fi
  echo "regen_census: capturing .gl for the workload (front-end only, ~2 min)"
  ( cd "$ROOT/work/w-bss2" && python3 glcensus.py glcensus.jsonl "$JOBS" )
  [ -s "$GLCENSUS" ] || { echo "regen_census: glcensus.py produced no output" >&2; exit 1; }
  echo "regen_census: glcensus.jsonl  $(wc -l < "$GLCENSUS" | tr -d ' ') records, $(du -h "$GLCENSUS" | cut -f1)"
fi

rm -f "$PROV_BEGIN"

# ---------------------------------------------------------------- the stamps
# Print what every downstream number is now pinned to. A census whose sidecar
# cannot be read is a census nothing may be graded against, so this is a check,
# not decoration.
for f in "$SECTIONS" "$GLCENSUS"; do
  [ -s "$f" ] || continue
  printf 'regen_census: %-14s ' "$(basename "$f")"
  python3 "$ROOT/work/w-bss2/prov.py" "$f" || exit 1
done

ELAPSED=$(( $(date +%s) - START ))
echo "regen_census: done in ${ELAPSED}s"
