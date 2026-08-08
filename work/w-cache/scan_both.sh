#!/bin/sh
# w-cache evidence: the full 878-TU gap scan at BOTH ends, identical instrument.
# Sequential, so the two runs never contend for the same cache key lock.
set -u
cd "$(dirname "$0")/../.."
W=work/w-cache
for end in base tip; do
    case "$end" in
        base) bin="$W/c2rs-base" ;;
        tip)  bin="./target/release/c2rs" ;;
    esac
    echo "=== $end : $bin ==="
    "$bin" gap \
        --list work/dc3-workload/files.txt \
        --flags-file work/dc3-workload/flags.txt \
        --cwd ../../../../dc3-decomp \
        --jsonl "$W/$end.jsonl" \
        --jobs 8 \
        --work "/tmp/wcache-scan-$end" \
        > "$W/$end.log" 2>&1
    echo "  rc=$?  $(grep -c . "$W/$end.log") lines"
done
echo "DONE"
