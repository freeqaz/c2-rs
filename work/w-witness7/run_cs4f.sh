#!/usr/bin/env bash
cd "$(dirname "$0")/../.." || exit 1
set -u
python3 work/w-witness7/patch.py apply M-CS4F || exit 1
L=work/w-witness7/logs/M-CS4F.suite.log
{ C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast; echo "EXIT=$?"; } > $L 2>&1
export C2RS_REQUIRE_TOOLCHAIN=1
MAIN_REPO="$(cd "$(git rev-parse --git-common-dir)/.." && pwd)"
export C2RS_DC3="${C2RS_DC3:-$MAIN_REPO/../dc3-decomp}"
scripts/gate.sh --jobs 16 --require-graded > work/w-witness7/logs/M-CS4F.gate.log 2>&1
echo "gate exit=$?"
./target/release/c2rs gap --list work/dc3-workload/files.txt \
    --flags-file work/dc3-workload/flags.txt --cwd "$C2RS_DC3" --jobs 16 \
    > work/w-witness7/logs/M-CS4F.scan.log 2>&1
echo "scan exit=$?"
echo "--- markers, grepped from raw log TEXT (never from an exit code) ---"
grep -ac 'w-witness7 CS4-fallback' work/w-witness7/logs/M-CS4F.suite.log \
   work/w-witness7/logs/M-CS4F.gate.log work/w-witness7/logs/M-CS4F.scan.log
python3 work/w-witness7/patch.py revert
echo "=== CS4F DONE"
