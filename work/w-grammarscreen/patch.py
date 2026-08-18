#!/usr/bin/env python3
"""w-grammarscreen — apply / revert the screen. Refuses to start on a dirty tree.

    patch.py apply     install `grammarprobe.rs` + `#[track_caller]` + the probe
                       call in `blk`, `blk_type` and `Block::refuse`
    patch.py revert    `git checkout` the three touched paths and delete the module
    patch.py verify    assert `git diff -- crates fixtures scripts` is EMPTY

Every edit asserts a UNIQUE textual match before it applies, so a stale locator
is a refusal rather than a wrong edit (`w-deadsites` §2's rule).
"""
import os
import shutil
import subprocess
import sys

ROOT = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", ".."))
LANE = os.path.join(ROOT, "work", "w-grammarscreen")
MOD = os.path.join(ROOT, "crates", "c2-il", "src", "func", "body", "mod.rs")
LIB = os.path.join(ROOT, "crates", "c2-il", "src", "lib.rs")
PROBE = os.path.join(ROOT, "crates", "c2-il", "src", "grammarprobe.rs")

CALL = "    crate::grammarprobe::hit(::std::panic::Location::caller());\n"

EDITS = [
    # (file, unique anchor, replacement)
    (
        MOD,
        'pub(crate) fn blk(seg: &[u8], p: usize, ctx: &\'static str) -> Block {\n',
        '#[track_caller]\npub(crate) fn blk(seg: &[u8], p: usize, ctx: &\'static str) -> Block {\n' + CALL,
    ),
    (
        MOD,
        "pub(crate) fn blk_type(seg: &[u8], p: usize, report_at: usize, ctx: &'static str) -> Block {\n",
        "#[track_caller]\npub(crate) fn blk_type(seg: &[u8], p: usize, report_at: usize, ctx: &'static str) -> Block {\n" + CALL,
    ),
    (
        MOD,
        "    pub(crate) fn refuse(seg: &[u8], off: usize, ctx: &'static str) -> Block {\n",
        "    #[track_caller]\n    pub(crate) fn refuse(seg: &[u8], off: usize, ctx: &'static str) -> Block {\n    " + CALL,
    ),
    (LIB, "pub mod codec;\n", "mod grammarprobe;\npub mod codec;\n"),
]

TOUCHED = ["crates/c2-il/src/func/body/mod.rs", "crates/c2-il/src/lib.rs"]


def dirty():
    out = subprocess.run(
        ["git", "-C", ROOT, "diff", "--name-only", "--", "crates", "fixtures", "scripts"],
        capture_output=True, text=True).stdout.split()
    return out


def apply():
    d = dirty()
    if d:
        sys.exit("REFUSED: tree already dirty under crates/fixtures/scripts: %s" % d)
    if os.path.exists(PROBE):
        sys.exit("REFUSED: %s already present" % PROBE)
    shutil.copyfile(os.path.join(LANE, "grammarprobe.rs"), PROBE)
    for path, anchor, repl in EDITS:
        text = open(path).read()
        n = text.count(anchor)
        if n != 1:
            sys.exit("REFUSED: anchor matched %d times in %s: %r" % (n, path, anchor[:60]))
        open(path, "w").write(text.replace(anchor, repl))
    print("applied: probe module + %d edits" % len(EDITS))


def revert():
    subprocess.run(["git", "-C", ROOT, "checkout", "--"] + TOUCHED, check=True)
    if os.path.exists(PROBE):
        os.remove(PROBE)
    verify()


def verify():
    d = subprocess.run(
        ["git", "-C", ROOT, "status", "--porcelain", "--", "crates", "fixtures", "scripts"],
        capture_output=True, text=True).stdout.strip()
    if d:
        sys.exit("NOT CLEAN:\n" + d)
    print("clean: crates/ fixtures/ scripts/ byte-identical to HEAD")


if __name__ == "__main__":
    {"apply": apply, "revert": revert, "verify": verify}[sys.argv[1]]()
