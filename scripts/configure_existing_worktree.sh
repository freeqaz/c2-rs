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

# ---- MAIN_REPO : ASK GIT, do not do path arithmetic on $0 ---------------------
#
# This was `MAIN_REPO="$(cd "$(dirname "$0")/.." && pwd)"` — the directory one
# level above the script — and it is wrong for the case this script exists to
# serve. Board **#3500** (`w-3475` §10.1):
#
#   Every worktree contains its own copy of `scripts/`. So for a worktree in a
#   SIBLING directory, invoked through its own copy, `dirname $0/..` IS THE
#   WORKTREE. The `$WORKTREE_PATH = $MAIN_REPO` test below then fires, the script
#   prints "is the main repo; nothing to configure" and EXITS 0 — without linking
#   `compilers/`, without copying the workload, and without running the toolchain
#   assertion that is the entire point of the file. **Skipped for four of the
#   five worktrees live on this box on 2026-08-24.**
#
# It is `#3516`'s defect class exactly: it reports success while doing something
# other than what it says. And it fails SILENTLY and in the DANGEROUS direction —
# an unconfigured worktree has no `compilers/`, so the differential degrades to
# `SKIP: toolchain absent` and grades every change as passing.
#
# `--git-common-dir` is the main repo's `.git` **from any linked worktree and
# from the main repo itself**, which is precisely the question being asked.
# `--path-format=absolute` needs git 2.31+; the `$0` arithmetic survives only as
# the fallback for a tree that is not a git checkout at all.
if MAIN_GIT_DIR="$(git -C "$(dirname "$0")" rev-parse --path-format=absolute \
                      --git-common-dir 2>/dev/null)" && [ -n "$MAIN_GIT_DIR" ]; then
    MAIN_REPO="$(cd "$MAIN_GIT_DIR/.." && pwd)"
else
    echo "    WARN: git could not name the main repo; falling back to \$0 arithmetic," >&2
    echo "          which is wrong for a sibling-directory worktree (board #3500)." >&2
    MAIN_REPO="$(cd "$(dirname "$0")/.." && pwd)"
fi

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

# ---- base-ref sanity --------------------------------------------------------
# Claude Code's own worktree isolation branches from `origin/<default>` by default
# (the `worktree.baseRef: fresh` setting), which here is ~30 commits behind local
# `master` and can predate whole subcommands. An agent lost time to exactly that.
# Report it rather than silently rebasing someone's branch.
if git -C "$MAIN_REPO" rev-parse --verify --quiet master >/dev/null; then
    if ! git -C "$WORKTREE_PATH" merge-base --is-ancestor master HEAD 2>/dev/null; then
        behind="$(git -C "$WORKTREE_PATH" rev-list --count HEAD..master 2>/dev/null || echo '?')"
        echo "    WARN: this worktree is $behind commits behind local 'master'." >&2
        echo "          It was probably branched from origin/master. If you have no" >&2
        echo "          commits yet:  git reset --hard master" >&2
    fi
fi

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

# ---- wibo : a sibling symlink NEXT TO the worktrees --------------------------
# `Toolchain::locate` looks for `<repo_root>/../wibo/build/release/wibo`, and
# `repo_root` is `CARGO_MANIFEST_DIR/../..` — the WORKTREE, not the main repo. So
# from `<main>/.claude/worktrees/<name>` the sibling lookup resolves to
# `<main>/.claude/worktrees/wibo`, which does not exist, and every in-worktree build
# reverts to `SKIP: toolchain absent`.
#
# This is not hypothetical and it is the nastiest form of the SKIP trap, because it
# passes the check below and then breaks: reflinking the main repo's `target/` puts
# a WORKING binary in the worktree, so the first verification succeeds — and the
# agent's first `cargo build` replaces it with one that silently finds no toolchain.
# An agent reported exactly that sequence.
#
# One symlink beside the worktrees fixes it for all of them at once, and it is the
# same sibling-resolution the design already relies on. `C2RS_WIBO` still overrides.
WT_PARENT="$(dirname "$WORKTREE_PATH")"
if [ ! -e "$WT_PARENT/wibo" ]; then
    MAIN_WIBO=""
    for cand in "${C2RS_WIBO:-}" "$MAIN_REPO/../wibo/build/release/wibo" \
                "$MAIN_REPO/../wibo/build/wibo" "$(command -v wibo 2>/dev/null || true)"; do
        [ -n "$cand" ] && [ -x "$cand" ] || continue
        # Walk up from the binary to the directory that contains `build/`.
        MAIN_WIBO="$(cd "$(dirname "$cand")/.." && pwd)"
        [ -d "$MAIN_WIBO/release" ] && MAIN_WIBO="$(cd "$MAIN_WIBO/.." && pwd)"
        break
    done
    if [ -n "$MAIN_WIBO" ] && [ -d "$MAIN_WIBO" ]; then
        echo "    ../wibo  (symlink beside the worktrees — the sibling lookup)"
        ln -s "$MAIN_WIBO" "$WT_PARENT/wibo"
    else
        echo "    WARN: no wibo found to link; set C2RS_WIBO for every command." >&2
    fi
fi

# ---- assert the toolchain actually resolves ---------------------------------
# The whole point. A worktree that SKIPs grades every change as passing, so this is
# a hard gate rather than a hint.
echo "==> Verifying the toolchain resolves (not SKIP)"
cd "$WORKTREE_PATH"
# ALWAYS rebuild before checking. Checking the reflinked binary from the main repo
# verifies the wrong thing — it is the in-worktree build whose toolchain resolution
# is in question, and validating the copy is how the failure above stayed hidden.
echo "    building the harness in-tree (the binary under test must be this one)"
cargo build --release -p c2-harness >/dev/null 2>&1 || {
    echo "ERROR: cargo build failed in the worktree." >&2
    exit 1
}
# Capture the WHOLE census, then select the line — never `| head -n1`.
#
# That pipeline broke this script twice over and the failure was SILENT, which is
# the part worth the comment (lane w-brfalse, board #443):
#
#  1. `head -n1` closes the pipe after one line, `c2rs` keeps writing, and Rust's
#     `println!` PANICS on `EPIPE` — `failed printing to stdout: Broken pipe`,
#     **exit 101**. With `set -euo pipefail` at the top of this file, the command
#     substitution's 101 kills the script *before* the `case` below ever runs.
#  2. The panic message went to stderr, `2>&1` merged stderr into the pipe, and
#     the pipe was already closed — so the script died on exit 101 having printed
#     **no error at all**, from a worktree whose toolchain was perfectly fine.
#  3. Independently, `census` prints a `profile:` banner first now, so line 1 was
#     never the verdict line any more even when the pipeline survived.
#
# Both failures are fail-CLOSED, which is the safe direction, but neither is
# diagnosable from the output. `grep` over a captured string has no pipe to break
# and does not care which line the verdict landed on.
census_out="$("$WORKTREE_PATH/target/release/c2rs" census fixtures/cpp/w5_chain.cpp 2>&1)" || {
    echo "ERROR: c2rs census exited non-zero in the worktree:" >&2
    echo "$census_out" >&2
    exit 1
}
verdict="$(printf '%s\n' "$census_out" | grep -m1 'functions in class' || true)"
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
    ./target/release/c2rs bench            # the ORACLE SELF-TEST — determinism and
                                           # capture stability. It exercises the
                                           # REFERENCE and never calls
                                           # PortC2::build: a counting probe read
                                           # 0 hits at build against a clean
                                           # 'summary: 391 pass, 0 fail', vs
                                           # 73,083 hits under gate.sh (#3516).
                                           # This line used to say "every fixture,
                                           # the correctness gate". It is NOT the
                                           # correctness gate and never was; the
                                           # next line is. CLAUDE.md § Layout has
                                           # always had this right.
    scripts/gate.sh --jobs 4               # THE CORRECTNESS GATE. EVERY mode lane; 0 mismatch or the
                                           # change is wrong. Quote the counts it
                                           # prints — a run that graded 0 is a
                                           # failure, not a pass. This used to
                                           # name one lane, and a lane nobody
                                           # enumerates is a lane that does not
                                           # run (docs/GAPS.md §7).

    A \`mismatch\` anywhere is an alarm, not a gap (CLAUDE.md, docs/GAPS.md §6).
EOF
