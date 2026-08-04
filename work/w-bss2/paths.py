#!/usr/bin/env python3
"""Lane w-bss2: toolchain and tree resolution, env-driven with repo-relative
defaults — the project's rule (`CLAUDE.md`: nothing absolute lives in source).

This file lives at `<lane-root>/work/w-bss2/`, so the lane root is two levels up
and the MAIN repo is the `.claude/worktrees/<lane>` parent when we are in a
worktree.  Everything is overridable:

  C2RS_LANE_ROOT   the worktree (defaults to two levels up from this file)
  C2RS_MAIN_ROOT   the main repo, which owns target/ and work/dc3-workload/
  C2RS_COMPILERS   the compilers/ directory
  C2RS_CL_EXE      cl.exe
  C2RS_WIBO        the wibo binary
  C2RS_DC3_SRC     the dc3-decomp source tree the workload compiles from
"""
import os

HERE = os.path.dirname(os.path.abspath(__file__))
LANE = os.environ.get("C2RS_LANE_ROOT") or os.path.abspath(
    os.path.join(HERE, "..", ".."))


def _main_root():
    if os.environ.get("C2RS_MAIN_ROOT"):
        return os.environ["C2RS_MAIN_ROOT"]
    # `<main>/.claude/worktrees/<lane>` -> `<main>`; otherwise we ARE the main repo
    p = os.path.abspath(os.path.join(LANE, "..", "..", ".."))
    if os.path.basename(os.path.dirname(LANE)) == "worktrees":
        return p
    return LANE


MAIN = _main_root()
COMPILERS = os.environ.get("C2RS_COMPILERS") or os.path.join(MAIN, "compilers")
CL_EXE = os.environ.get("C2RS_CL_EXE") or os.path.join(
    COMPILERS, "X360", "16.00.11886.00", "cl.exe")
WIBO = (os.environ.get("C2RS_WIBO")
        or os.path.join(os.path.dirname(MAIN), "wibo", "build", "wibo"))
DC3 = os.environ.get("C2RS_DC3_SRC") or os.path.join(
    os.path.dirname(MAIN), "dc3-decomp")
C2RS = os.path.join(MAIN, "target", "release", "c2rs")
WORKLOAD = os.path.join(MAIN, "work", "dc3-workload")
SECTIONS = os.path.join(MAIN, "work", "w-bss", "census", "sections.jsonl")


def flags():
    """The workload's own flag set, verbatim — including /GR and the /I list."""
    return open(os.path.join(WORKLOAD, "flags.txt")).read().split()


def probe_flags():
    """The same set with the project's `/I <dir>` pairs dropped, for standalone
    probe sources.  `/I` and its directory are two separate tokens in
    flags.txt, so both have to go."""
    out, skip = [], False
    for f in flags():
        if skip:
            skip = False
            continue
        if f == "/I":
            skip = True
            continue
        if f.startswith("/I") and len(f) > 2:
            continue
        out.append(f)
    return out
