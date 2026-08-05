#!/bin/bash
#
# arms.sh — regenerate w-brfalse's five-arm counterfactual (board #440).
#
# ONE binary, environment variables apart. That is the whole design and it is not
# a detail: the first pass of this lane took four arms on one binary and the
# fifth on a rebuild, which is a cross-binary comparison and exactly the confound
# the two-scan method exists to avoid. If you edit `crates/` between arms, every
# number below is void — rebuild once, then run all five.
#
# Read-only measurement tooling. It runs the shipped `c2rs` and nothing else.
#
# USAGE
#   work/w-brfalse/arms.sh [outdir]
#
# Needs the toolchain (`C2RS_COMPILERS` / `C2RS_WIBO`) and the workload
# (`work/dc3-workload`). Prints a positive count for every arm; a run that
# produced no records is a failure, not a pass.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OUT="${1:-$ROOT/work/w-brfalse}"
DC3="${C2RS_DC3:-$ROOT/../dc3-decomp}"
BIN="$ROOT/target/release/c2rs"

[ -x "$BIN" ] || { echo "no $BIN — cargo build --release -p c2-harness first" >&2; exit 1; }
[ -d "$DC3" ] || { echo "no dc3 tree at $DC3 — set C2RS_DC3" >&2; exit 1; }
mkdir -p "$OUT"

echo "binary: $(md5sum "$BIN")"

arm() {
    local name="$1"; shift
    env "$@" "$BIN" gap \
        --list "$ROOT/work/dc3-workload/files.txt" \
        --flags-file "$ROOT/work/dc3-workload/flags.txt" \
        --cwd "$DC3" --jobs "${C2RS_JOBS:-8}" \
        --jsonl "$OUT/v2-$name.jsonl" > "$OUT/v2gap-$name.txt" 2>&1
    # A positive check with a printed count, never a status: `capture-fail 7` is
    # the discriminator that says the run really reached the workload. A bad
    # --cwd gives `capture-fail 878 / match 0` and otherwise looks ordinary.
    printf '%-5s %s\n' "$name" \
        "$(grep -E 'gap-metric (match|capture-fail|frontier) ' "$OUT/v2gap-$name.txt" | tr -s ' ' | tr '\n' ' ')"
}

arm off
arm rel  C2RS_SINK_REL=expr
arm b1   C2RS_SINK_REL=expr C2RS_SINK_BRANCH=expr
arm b2   C2RS_SINK_REL=expr C2RS_SINK_BRANCH=cflow
arm b4   C2RS_SINK_REL=expr C2RS_SINK_BRANCH=stmt

echo
python3 - "$OUT" <<'PY'
import json, sys
from collections import Counter
out = sys.argv[1]
def load(a):
    c = Counter()
    n = 0
    for line in open(f"{out}/v2-{a}.jsonl"):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        n += 1
        for k, v in (r.get("emit_blockers") or {}).items():
            c[k] += v
    return c, n
hdr = f"{'arm':<5}{'TUs':>5}{'sites':>8}{'brfalse':>9}{'brtrue':>8}{'jump':>7}{'label':>7}{'0x53':>7}{'relPois':>9}{'brPois':>8}"
print(hdr)
for a in ["off", "rel", "b1", "b2", "b4"]:
    c, n = load(a)
    assert n, f"arm {a} produced ZERO TU records — refusing to report an empty measurement"
    rp = sum(v for k, v in c.items() if k.startswith("expr-rel-sink-poison"))
    bp = sum(v for k, v in c.items() if k.startswith("expr-branch-sink-poison"))
    print(f"{a:<5}{n:>5}{sum(c.values()):>8}{c.get('expr-brfalse',0):>9}"
          f"{c.get('expr-brtrue',0):>8}{c.get('expr-jump',0):>7}{c.get('expr-label',0):>7}"
          f"{c.get('expr-op-0x53',0):>7}{rp:>9}{bp:>8}")
print("\n  `sites` MUST be identical across every arm: the sinks rename keys, they")
print("  do not convert functions. A moving total means the sink stopped poisoning.")
PY

echo
for a in b1 b2 b4; do
    echo "==== REL -> $a ===="
    python3 "$ROOT/work/w-cmp/substitute.py" "$OUT/v2-rel.jsonl" "$OUT/v2-$a.jsonl" \
        | grep -E "closed mass|ladder-credited|MEASURED"
done
