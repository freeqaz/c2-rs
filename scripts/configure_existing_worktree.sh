#!/bin/bash
#
# configure_existing_worktree.sh — make an EXISTING worktree buildable + gradeable.
#
# Mirrors the sibling `../rb3-xenon/scripts/configure_existing_worktree.sh`. Use
# this for worktrees created by something other than `scripts/setup_worktree.sh` —
# in particular Claude Code's own worktree isolation (`EnterWorktree`, or an agent
# launched with `isolation: "worktree"`), which does a plain `git worktree add` and
# therefore produces a tree with **no toolchain and a cold build cache**.
#
# Usage:
#   scripts/configure_existing_worktree.sh [worktree-path] [--cold-cache]
#
# Idempotent — safe to run repeatedly on the same worktree.
#
# WHY THIS MATTERS MORE HERE THAN IT LOOKS
# ----------------------------------------------------------------------------
# `compilers/` is gitignored (MS binaries, never committed), and when it is absent
# the harness and every integration test **degrade cleanly** to `SKIP: toolchain
# absent` rather than failing — a deliberate design choice, see CLAUDE.md. In a
# worktree that means the differential silently grades nothing: `cargo test` is
# green, `c2rs diff` says SKIP, and a change that mis-emits looks exactly like a
# change that is byte-exact. So the first thing this script does after linking is
# assert the toolchain actually resolves, and it fails loudly if it does not.

set -euo pipefail

MAIN_REPO="$(cd "$(dirname "$0")/.." && pwd)"

WARM_CACHE=1
POSITIONAL=()
for arg in "$@"; do
    case "$arg" in
        --cold-cache) WARM_CACHE=0 ;;
        *) POSITIONAL+=("$arg") ;;
    esac
done
WORKTREE_PATH="${POSITIONAL[0]:-$(pwd)}"

if [ ! -e "$WORKTREE_PATH/.git" ]; then
    echo "ERROR: $WORKTREE_PATH is not a git worktree or repository" >&2
    exit 1
fi
WORKTREE_PATH="$(cd "$WORKTREE_PATH" && pwd)"

if [ "$WORKTREE_PATH" = "$MAIN_REPO" ]; then
    echo "==> $WORKTREE_PATH is the main repo; nothing to configure."
    exit 0
fi

reflink_dir_besteffort() {
    local src="$1" dst="$2" i
    mkdir -p "$(dirname "$dst")"
    rm -rf "$dst"
    for i in 1 2 3 4; do
        if cp -a --reflink=auto "$src" "$dst" 2>/dev/null; then return 0; fi
        sleep "$i"
    done
    [ -d "$dst" ] && return 0
    return 1
}

echo "==> Configuring worktree at $WORKTREE_PATH"

# ---- compilers/ : symlink (read-only toolchain) ------------------------------
if [ -d "$MAIN_REPO/compilers" ]; then
    echo "    compilers/  (symlink — read-only MS toolchain)"
    rm -rf "$WORKTREE_PATH/compilers"
    ln -s "$MAIN_REPO/compilers" "$WORKTREE_PATH/compilers"
else
    echo "ERROR: $MAIN_REPO/compilers is missing — run scripts/fetch_compilers.sh." >&2
    echo "  Refusing to configure a worktree whose differential would SKIP silently." >&2
    exit 1
fi

# ---- target/ : reflink copy (cargo WRITES here) ------------------------------
# Never a symlink: two cargo builds sharing one target dir corrupt each other, and
# with several agents running that is the failure that costs a day.
if [ "$WARM_CACHE" -eq 1 ] && [ -d "$MAIN_REPO/target" ]; then
    echo "    target/  (reflink copy — private build dir + WARM cargo cache)"
    reflink_dir_besteffort "$MAIN_REPO/target" "$WORKTREE_PATH/target" \
        || echo "    WARN: target/ copy failed; the worktree will build from cold." >&2
else
    echo "    target/  (skipped — cold cache)"
fi

# ---- work/dc3-workload/ : reflink copy (a gap scan writes beside it) --------
if [ -d "$MAIN_REPO/work/dc3-workload" ]; then
    echo "    work/dc3-workload/  (reflink copy — workload manifest + scans)"
    mkdir -p "$WORKTREE_PATH/work"
    reflink_dir_besteffort "$MAIN_REPO/work/dc3-workload" \
        "$WORKTREE_PATH/work/dc3-workload" \
        || echo "    WARN: workload copy failed; regenerate with scripts/gen_dc3_workload.sh." >&2
fi

# ---- assert the toolchain actually resolves ---------------------------------
# The whole point. A worktree that SKIPs grades every change as passing, so this is
# a hard gate rather than a hint.
echo "==> Verifying the toolchain resolves (not SKIP)"
cd "$WORKTREE_PATH"
if [ ! -x "$WORKTREE_PATH/target/release/c2rs" ]; then
    echo "    building the harness (once) to run the check"
    cargo build --release -p c2-harness >/dev/null 2>&1 || {
        echo "ERROR: cargo build failed in the worktree." >&2
        exit 1
    }
fi
verdict="$("$WORKTREE_PATH/target/release/c2rs" census fixtures/cpp/w5_chain.cpp 2>&1 | head -n1)"
case "$verdict" in
    *"4/4 functions in class"*)
        echo "    OK: $verdict" ;;
    "")
        echo "ERROR: the differential SKIPPED — no toolchain resolved in this worktree." >&2
        echo "  A worktree in this state reports every change as passing. Fix before use:" >&2
        echo "    compilers/ -> $MAIN_REPO/compilers (symlinked above)" >&2
        echo "    wibo       -> C2RS_WIBO, a sibling ../wibo build, or PATH" >&2
        exit 1 ;;
    *)
        echo "ERROR: unexpected verdict from the fixture census:" >&2
        echo "  $verdict" >&2
        exit 1 ;;
esac

cat <<EOF

==> Ready: $WORKTREE_PATH

    cd $WORKTREE_PATH
    cargo test --workspace                 # unit + integration
    ./target/release/c2rs bench            # every fixture, the correctness gate
    C2RS_JOBS=16 scripts/mode_lane.sh /O1  # 0 mismatch or the change is wrong

    A \`mismatch\` anywhere is an alarm, not a gap (CLAUDE.md, docs/GAPS.md §6).
EOF
