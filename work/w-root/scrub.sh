#!/bin/bash
# scrub.sh — strip absolute machine paths from a log before it is committed.
#
#   work/w-inread/scrub.sh <file>...
#
# `CLAUDE.md`: no `/home/<user>/…` in anything committed. Cargo lines, the gap
# scan's provenance block and every `z:\…` source path in a `--list` run carry
# them, so this rewrites them to `<REPO>` / `<HOME>` in place. It is idempotent.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MAIN="$(cd "$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir)/.." && pwd)"
HOME_DIR="${HOME:-/home/$(id -un)}"
# The `z:\` spellings first: they are the same path with backslashes.
ZROOT="z:$(printf '%s' "$ROOT" | tr '/' '\\')"
ZMAIN="z:$(printf '%s' "$MAIN" | tr '/' '\\')"
ZHOME="z:$(printf '%s' "$HOME_DIR" | tr '/' '\\')"
for f in "$@"; do
    python3 - "$f" "$ROOT" "$MAIN" "$HOME_DIR" "$ZROOT" "$ZMAIN" "$ZHOME" <<'PY'
import sys
path, root, main, home, zroot, zmain, zhome = sys.argv[1:8]
t = open(path, encoding="utf-8", errors="surrogateescape").read()
for a, b in ((zroot, "z:<WORKTREE>"), (zmain, "z:<REPO>"), (zhome, "z:<HOME>"),
             (root, "<WORKTREE>"), (main, "<REPO>"), (home, "<HOME>")):
    t = t.replace(a, b)
open(path, "w", encoding="utf-8", errors="surrogateescape").write(t)
PY
    printf 'scrubbed %s (%s absolute paths left)\n' "$f" "$(grep -c "$HOME_DIR" "$f" || true)"
done
