# Sourceable helper: build `c2rs` and PIN it for one gate run.
#
#     . "$repo_root/scripts/harness_bin.sh"
#     pin_harness "$repo_root" "$run_dir"
#     c2rs="$C2RS_PINNED"
#
# One locator for a fact three gate components need — `expr_sweep.sh`,
# `mode_lane.sh` and `cross_sweep.sh` — because it has now been got wrong in all
# three at once, which is what a rule with three implementations does.
#
# TWO FAILURES, ONE FIX
# ---------------------
# **1. A stale binary.** All three scripts guarded the build with
# `if [ ! -x "$c2rs" ]`: build only when the binary is ABSENT, otherwise run
# whatever is on disk. So a gate could grade the current tree's *cases* with a
# previous tree's *code*. Not hypothetical — a sweep on the cflow-decode merge
# reported 47 mismatches against a binary five hours behind HEAD, and the rebuild
# that resolved it took 3.43 s. The false-MISMATCH direction costs an
# investigation; **the false-GREEN direction is the hazard**: land a regression,
# run the gate, and it passes because it graded the old binary, with nothing in
# the log to tell the two apart. `cargo build` is a no-op when current, so the
# guard was never buying anything.
#
# **2. A binary republished under a running gate.** A gate that executes
# `target/release/c2rs` directly depends on a file the rest of the tree may
# rewrite at any moment — another lane, another agent, a `cargo test`.
#
# Be careful about what is established here. One sweep did die mid-run (exit 144,
# no `checked=` line, 6,225 cases discarded) with a `cargo` build alongside it,
# **and that death could not be reproduced on demand**: 400-case runs against
# `target/release/c2rs` survived three forced relinks and a full
# `cargo test --workspace`. So the mechanism is an unproven hypothesis, not a
# measured fact, and it should not be cited as one. (Some of those deaths were
# later traced to something else entirely — a `pkill -f expr_sweep` whose pattern
# matched its own command line.)
#
# The fix does not rest on that hypothesis. It rests on a structural property:
# a run that holds its own copy is unaffected by anything the tree does to
# `target/`, whether or not cargo's publication is atomic. Same shape as the
# `c2host.exe` in-place link this repo fixed today, one layer up.
#
# Both go away together: **build unconditionally, then copy the binary into the
# run's own directory and run THAT copy.** The copy is published by `rename`, so
# it is atomic; once a run holds it, nothing in the tree can change it. A stale
# binary becomes impossible, a concurrent `cargo build` becomes harmless, and two
# gate runs can proceed at once without fighting over one file.
#
# The identity line is not decoration. It names the binary a run actually
# executed — source path, build time, content hash, tree HEAD and whether the
# tree was dirty — so "which code produced this number" is answerable from the
# log rather than reconstructed afterwards from a mismatch count. Two runs quoting
# the same `sha` graded the same code; two quoting different ones did not,
# whatever their commit messages say.

# pin_harness <repo-root> <run-dir> — sets C2RS_PINNED. Returns non-zero (and
# under the callers' `set -e`, aborts) rather than falling back to a binary it
# cannot vouch for: a gate that grades with an unknown binary is worse than a
# gate that does not run.
pin_harness() {
    _pb_root="$1"
    _pb_run="$2"

    if [ -n "${C2RS_BIN:-}" ]; then
        # An explicit override is the caller taking responsibility for identity;
        # say so, so the log never implies this run pinned anything.
        C2RS_PINNED="$C2RS_BIN"
        echo "harness: $C2RS_PINNED  (C2RS_BIN override — NOT built or pinned by this run)"
        return 0
    fi

    echo "building the harness (no-op if current)"
    if ! (cd "$_pb_root" && cargo build --release -p c2-harness); then
        echo "FATAL: cargo build failed — refusing to grade with whatever binary \
happens to be on disk" >&2
        return 1
    fi
    _pb_src="$_pb_root/target/release/c2rs"
    if [ ! -x "$_pb_src" ]; then
        echo "FATAL: no $_pb_src after a successful build" >&2
        return 1
    fi

    mkdir -p "$_pb_run"
    _pb_dst="$_pb_run/c2rs"
    _pb_tmp="$_pb_run/.c2rs.$$.tmp"
    rm -f "$_pb_tmp"
    if ! cp "$_pb_src" "$_pb_tmp"; then
        rm -f "$_pb_tmp"
        echo "FATAL: could not copy the harness into $_pb_run" >&2
        return 1
    fi
    chmod +x "$_pb_tmp"
    # rename, not a second cp: publication is atomic, so a reader sees the whole
    # binary or the previous one, never a partial.
    if ! mv -f "$_pb_tmp" "$_pb_dst"; then
        rm -f "$_pb_tmp"
        echo "FATAL: could not publish the pinned harness at $_pb_dst" >&2
        return 1
    fi

    _pb_sha=$(sha256sum "$_pb_dst" 2>/dev/null | cut -c1-12)
    _pb_head=$(cd "$_pb_root" && git rev-parse --short HEAD 2>/dev/null || echo '?')
    _pb_dirty=''
    (cd "$_pb_root" && git diff --quiet HEAD 2>/dev/null) || _pb_dirty='-dirty'
    echo "harness: $_pb_dst"
    echo "  pinned from $_pb_src  built $(date -r "$_pb_src" '+%F %T')"
    echo "  sha ${_pb_sha:-?}  tree ${_pb_head}${_pb_dirty}"
    C2RS_PINNED="$_pb_dst"
}
