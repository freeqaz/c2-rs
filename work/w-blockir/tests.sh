#!/bin/sh
# tests.sh — run the whole workspace and count, with #2262's correction applied.
#
# `cargo test --workspace` stops at the first failing TARGET, so a red total is a
# TRUNCATION and its `N passed` is not a test count. `--no-fail-fast` is
# mandatory, and the TARGET count is printed beside the test count so a short run
# cannot be read as a green one.
#
# Usage: tests.sh <out-stem>
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$here/../.." && pwd)"
stem="$1"
cd "$repo_root"
cargo test --workspace --release --no-fail-fast > "$here/$stem.raw" 2>&1 || true
awk '
/^test result:/ {
    for (i = 1; i <= NF; i++) {
        if ($i == "passed;")   p += $(i-1);
        if ($i == "failed;")   f += $(i-1);
        if ($i == "ignored;")  g += $(i-1);
    }
    t++;
}
END { printf "%s passed, %s failed, %s ignored, %s targets\n", p+0, f+0, g+0, t+0 }
' "$here/$stem.raw" | tee "$here/$stem.txt"
