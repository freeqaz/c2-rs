#!/bin/sh
# w-joint2 counterfactual — transiently widen PORT_WRITER_SECTIONS, re-run the
# REAL `c2rs gap`, revert, and assert the tree is clean.
#
# REGISTERED BEFORE RUNNING (prereg R3/R4):
#   + .rdata$r            : factor-c 590, b-and-c 315, a-and-b-and-c 27,
#                           a-and-b-and-c-and-d-or-e 8, match 8
#   + all three           : factor-c 871, b-and-c 338, a-and-b-and-c 27,
#                           a-and-b-and-c-and-d-or-e 8, match 8
# 590/315 are w-rdata's (writer edit) and w-reach's (key reconstruction)
# published figures, so they are a KNOWN-ANSWER CONTROL on this instrument.
# 27 and 8 are this lane's predictions and are NOT controls.
set -e
cd "$(dirname "$0")/../.."
F=crates/c2-core/src/coff/function.rs
OUT=work/w-joint2
: "${C2RS_COMPILERS:?set it}"; : "${C2RS_WIBO:?set it}"
LIST=${C2RS_LANEROOT:?set it to the main repo}/work/dc3-workload/files.txt
FLAGS=$C2RS_LANEROOT/work/dc3-workload/flags.txt
CWD=${C2RS_DC3:?set it to the dc3 tree}

run() { # $1 = label
  cargo build --release -p c2-harness --bin c2rs 2>&1 | grep -E '^error' && exit 1
  ./target/release/c2rs gap --list "$LIST" --flags-file "$FLAGS" --cwd "$CWD" \
      --jobs 6 --factors-tsv "$OUT/factors_$1.tsv" > "$OUT/gap_$1.txt"
  echo "== $1 =="
  grep -E 'gap-metric (match|factor-[abcde]|b-and-c|a-and-b-and-c|a-and-b-and-c-and-d-or-e|writer-sections|frontier) ' "$OUT/gap_$1.txt"
}

trap 'git checkout -- "$F"; echo "-- reverted --"' EXIT

# ---- C1: + .rdata$r ---------------------------------------------------------
python3 - "$F" <<'PY'
import sys,re
p=sys.argv[1]; s=open(p).read()
s=s.replace('pub const PORT_WRITER_SECTIONS: [&str; 10] = [',
            'pub const PORT_WRITER_SECTIONS: [&str; 11] = [\n    ".rdata$r",',1)
open(p,'w').write(s)
PY
run c1_rdatar

# ---- C2: + all three ladder names -------------------------------------------
git checkout -- "$F"
python3 - "$F" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
s=s.replace('pub const PORT_WRITER_SECTIONS: [&str; 10] = [',
            'pub const PORT_WRITER_SECTIONS: [&str; 13] = [\n    ".rdata$r",\n    ".text$yd",\n    ".xdata$x",',1)
open(p,'w').write(s)
PY
run c2_all3

# ---- revert and assert ------------------------------------------------------
git checkout -- "$F"
trap - EXIT
if [ -n "$(git status --porcelain crates/)" ]; then
  echo "FAIL: crates/ is dirty after revert"; git status --porcelain crates/; exit 1
fi
echo "REVERTED CLEAN: git status --porcelain crates/ is empty"
cargo build --release -p c2-harness --bin c2rs 2>&1 | tail -1
