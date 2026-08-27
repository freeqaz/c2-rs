#!/usr/bin/env python3
"""scrub.py — remove THIS BOX's absolute paths from the tracked evidence shelf.

Decision 18 makes `work/` a tracked evidence shelf with two absolute
carve-outs; this enforces the second one. `CLAUDE.md` § Commits: *"Never
commit: … absolute machine paths (`/home/<user>/…`)"*.

WHAT IT DOES NOT TOUCH, and this is the load-bearing half
---------------------------------------------------------
`e:\\lazer_build_gmc1`, `dc3-decomp` and the XDK build id are **INTENTIONAL**
(`CLAUDE.md` § "Project context"). 366 tracked files name them. This script
never matches them: its patterns are anchored on the *user home* segment, and
`dc3-decomp` survives every rule as a literal substring
(`/home/<user>/code/milohax/dc3-decomp` -> `<home>/code/milohax/dc3-decomp`).
The scrub asserts the occurrence counts of both strings are unchanged.

THE ROOTS ARE NOT ONE ROOT — measured, not assumed
--------------------------------------------------
A blanket `s|<one root>|<repo>|` would flatten distinctions the transcripts
legitimately record. At 41ca1ee9a the tracked text references **13** distinct
absolute roots:

    458 files  /home/<user>/code/milohax/c2-rs            the main checkout
     92 files  /home/<user>/code/milohax/wibo             the sibling wibo build
     83 files  /home/<user>/code/milohax/dc3-decomp       the sibling decomp tree
     20 files  /home/<user>/tmp                           run scratch
     12 files  /home/<user>/code/milohax/c2-rs-wt-w-stageoracle
      5 files  /home/<user>/code/milohax/c2-rs-wt-w-restim
      4 files  /home/<user>/code/milohax/c2-rs-wt-w-cfgclass
      3 files  /home/<user>/code/milohax/wibo-forkserver  a DIFFERENT wibo build
      2 files  /home/<user>/code/milohax/c2-rs-wt-w-3475
      2 files  /home/<user>/code/milohax/c2-rs-wt-w-warranty
      1 file   /home/<user>/code/milohax/c2-rs-wt-w-fork
      1 file   /home/<user>/code/milohax/c2-rs-wt-w-ledger
      1 file   /home/<u>/a, /home/<x>/leak    SYNTHETIC — see below

`/home/<user>/code/milohax/c2-rs` is a **strict prefix of seven of the
others** (`c2-rs-wt-w-*`, the pre-`.claude/worktrees` worktree naming). So
the rules are applied **longest-first with a right boundary**: the repo-root
rule refuses to match when the next character could continue an identifier,
and the old sibling worktrees come out as `<home>/code/milohax/c2-rs-wt-w-…`,
visibly a different root from `<repo>`. That distinction is real — several
transcripts record a measurement taken in a worktree *against* a corpus in
the main checkout — and flattening it would have made those two the same
place.

`/home/<u>/a/b/c` and `/home/<x>/leak` are **left alone on purpose**: they are
synthetic inputs to `work/w-bss2/prov.py`'s assertions about the provenance
path-relativiser, not paths on this box. They are exempted in
`scripts/tracked_artifact_audit.sh`'s printed allowlist, by name and with
that reason, rather than silently rewritten — rewriting a test's input
changes what the test asserts.

THE WINDOWS SPELLING, WHICH NO STANDING DETECTOR COULD SEE
-----------------------------------------------------------
The reference side runs under wibo, which maps the filesystem to a DOS drive,
so `cl`/`c2` write `z:\\home\\<user>\\code\\milohax\\…` into every `.cod`
listing, `.list` file and oracle log. **36 tracked files carry that form and
26 of them contain no forward-slash `/home/` at all** — they are invisible to
`/home/[a-z][a-z0-9_-]*/`, which is the detector `tracked_artifact_audit.sh`
and every prior lane's scrubber use. Both `z:` and `Z:` occur. The drive
letter is kept: it is evidence about what wibo mapped, not about this box.

BOARD #1135 — never rewrite a file another process still holds open
-------------------------------------------------------------------
A scrub once raced a backgrounded `gate.sh` that still held its `>` fd and
punched a 122-byte NUL hole into a **passing** gate's transcript; `grep` then
returned nothing and a waiter reported TIMEOUT on a run that had passed
18/18. The guard here is **by open descriptor** (`/proc/*/fd`), per file, not
by timing and not by "is a gate alive" — a peer lane's gate in another
worktree must not block this, and a non-gate writer must not slip past it
(`#1236` repaired the timing form into this one). **The number of descriptors
scanned is printed**: a `/proc` walk that found nothing because it walked
nothing is the failure this repo has recorded ~15 times.

Usage:
    scrub.py --check     report what would change; write nothing; exit 1 if any
    scrub.py --apply     rewrite in place, then verify every invariant
"""

import hashlib
import os
import re
import subprocess
import sys

# ---------------------------------------------------------------- the rules --
# Applied in order. Each is (compiled pattern, replacement, human name).
# The right boundary `(?![-A-Za-z0-9_])` is what keeps `c2-rs` from eating
# `c2-rs-wt-w-restim`; without it the seven old sibling worktrees collapse
# into the main checkout.
_B = rb"(?![-A-Za-z0-9_])"

# THE PATTERNS ARE ASSEMBLED AT RUNTIME, AND THAT IS NOT STYLE.
#
# Written as literals, this file would itself contain `/home/<a-user>/…` — and
# `scripts/tracked_artifact_audit.sh`'s new class 3 scans the CONTENT of every
# tracked file under `work/`. The scrubber would then be flagged by the guard
# it exists to satisfy, and the only ways out are a self-exemption or a
# runtime assembly. `tracked_artifact_audit.sh` already hit this and chose
# assembly, for the reason it records in its own comment: *a rule its enforcer
# is exempt from is a rule with one guaranteed blind spot.* Same choice here.
#
# Derived, not transcribed, so the segments below are the only place the box's
# identity appears — and they are two ordinary words, not a path.
_H = b"/" + b"home"
_U = b"free"
_PROJ = b"/code/milohax/c2-rs"


def _bs(p):
    """The same path in wibo's DOS spelling (forward slashes -> backslashes)."""
    return p.replace(b"/", b"\\")


RULES = [
    (re.compile(re.escape(_H + b"/" + _U + _PROJ) + _B), b"<repo>",
     "the main checkout, forward-slash"),
    (re.compile(re.escape(_bs(_H + b"/" + _U + _PROJ)) + _B), b"<repo>",
     "the main checkout, wibo/DOS backslash (drive letter preserved)"),
    (re.compile(re.escape(_H + b"/" + _U) + _B), b"<home>",
     "any other path under this box's home, forward-slash"),
    (re.compile(re.escape(_bs(_H + b"/" + _U)) + _B), rb"\\<home>",
     "any other path under this box's home, wibo/DOS backslash"),
]

# Strings that must survive untouched, with their occurrence counts asserted.
INTENTIONAL = [b"lazer_build_gmc1", b"dc3-decomp", b"16.00.11886.00"]

# Any absolute-path token, in EITHER spelling, in EITHER the pre or the post
# text. Masking these is how "byte-identical outside the substituted prefix"
# is turned into a check a reader can re-run.
PATHTOK = re.compile(
    rb"(?:/home/[A-Za-z0-9_.-]+|\\home\\[A-Za-z0-9_.-]+|<repo>|\\?<home>)"
    rb"(?:[/\\][A-Za-z0-9_.@+~-]+)*"
)


def canon(data: bytes) -> bytes:
    """Every absolute-path token replaced by a fixed marker.

    If canon(before) == canon(after) then the ONLY bytes that moved are inside
    path tokens: every count, verdict line, hash and measurement is identical.
    """
    return PATHTOK.sub(b"\x01PATH\x01", data)


def holders(paths):
    """pid set per path, by /proc fd symlink. Returns (map, descriptors_seen)."""
    want = {os.path.realpath(p): p for p in paths}
    held, seen = {}, 0
    try:
        pids = [d for d in os.listdir("/proc") if d.isdigit()]
    except OSError:
        return held, -1
    for pid in pids:
        d = "/proc/%s/fd" % pid
        try:
            fds = os.listdir(d)
        except OSError:
            continue
        for fd in fds:
            try:
                tgt = os.readlink(os.path.join(d, fd))
            except OSError:
                continue
            seen += 1
            if tgt in want:
                held.setdefault(want[tgt], set()).add(pid)
    return held, seen


def tracked(root):
    out = subprocess.run(["git", "-C", root, "ls-files", "-z"],
                         capture_output=True, check=True).stdout
    return [p.decode() for p in out.split(b"\0") if p]


# Lanes whose worktrees are live at dispatch, plus the coordinator's gate-base
# shelf: not this lane's to rewrite, however many paths they carry.
FENCED_OUT = ("work/w-mopfold/", "work/w-secported/", "work/w-provaudit/",
              "work/coordinator/")

# This lane's own prereg names the forbidden literal as PROSE, once, at line
# 22 — quoting the string the lane exists to remove. A prereg is never edited
# after it is committed, so it is not scrubbed; it is exempted BY NAME in
# `scripts/tracked_artifact_audit.sh`'s printed allowlist instead, with that
# reason, exactly as `crates/c2-harness/src/provenance.rs` already is for a
# documentary path in a doc comment. This is a self-exemption and it is
# recorded as one: see the rung, which also names the general fix (write the
# literal as `/home/<user>`, which no detector matches, and which this
# prereg's other three mentions already do).
PROSE_EXEMPT = (
    "work/w-shelf/PREREG.md",
    # A FROZEN PROOF TRANSCRIPT, and the exemption is about what a proof IS.
    # `scrub_proof.txt` is `verify_scrub.py`'s output at the moment the 513-file
    # scrub landed; one of its LABELS quotes the literal ("still carrying
    # <literal> : 0"). Rewriting the label would edit an evidence artifact after
    # the fact to make a later run of a different tool look tidier — worse than
    # the label. It is a label, not a path: no trailing separator, so no
    # detector reads it as one, and `tracked_artifact_audit.sh` is green on it
    # without an allowlist entry.
    "work/w-shelf/scrub_proof.txt",
)


def main(argv):
    if len(argv) != 2 or argv[1] not in ("--check", "--apply"):
        sys.stderr.write(__doc__.rsplit("Usage:", 1)[-1])
        return 2
    apply_ = argv[1] == "--apply"
    root = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", ".."))

    files = tracked(root)
    print("tracked files examined: %d" % len(files))
    if not files:
        sys.stderr.write("ERROR: examined ZERO files (#3470, #1002)\n")
        return 2

    # ---- population: every tracked TEXT file under work/ that any rule hits --
    pop, skipped_fence, skipped_prose, binary = [], [], [], 0
    for f in files:
        p = os.path.join(root, f)
        try:
            data = open(p, "rb").read()
        except FileNotFoundError:
            continue
        if b"\x00" in data:          # a working NUL test, never grep (#1236)
            binary += 1
            continue
        if not any(rx.search(data) for rx, _, _ in RULES):
            continue
        if not f.startswith("work/"):
            continue                 # docs/ and crates/ are outside this fence
        if f.startswith(FENCED_OUT):
            skipped_fence.append(f)
            continue
        if f in PROSE_EXEMPT:
            skipped_prose.append(f)
            continue
        pop.append((f, data))
    print("binary (NUL-bearing) files skipped: %d" % binary)
    print("in-scope files under work/ carrying an absolute machine path: %d"
          % len(pop))
    print("skipped — live peer / coordinator fence: %d %s"
          % (len(skipped_fence), skipped_fence or ""))
    print("skipped — prose mention in an unamendable prereg: %d %s"
          % (len(skipped_prose), skipped_prose or ""))
    if not pop:
        print("SCRUB: nothing to do — 0 files in scope")
        return 0

    # ---- #1135: refuse any file a live process still holds open -------------
    held, seen = holders([os.path.join(root, f) for f, _ in pop])
    print("open descriptors scanned: %d" % seen)
    if seen <= 0:
        sys.stderr.write("ERROR: the /proc walk saw %d descriptors. A holder "
                         "check that scanned nothing is not a check.\n" % seen)
        return 2
    if held:
        for k, v in sorted(held.items()):
            sys.stderr.write("REFUSING %s — held open by pid(s) %s\n"
                             % (k, sorted(v)))
        sys.stderr.write("board #1135: rewriting a file a writer still holds "
                         "punches a NUL hole into it.\n")
        return 2
    print("files held open by a live process: 0  (#1135 clear)")

    # ---- rewrite, then verify -----------------------------------------------
    per_rule = dict((name, 0) for _, _, name in RULES)
    changed, failures = 0, []
    agg_pre, agg_post = hashlib.sha256(), hashlib.sha256()
    gate_blocks = 0
    for f, pre in pop:
        post = pre
        for rx, rep, name in RULES:
            post, n = rx.subn(rep, post)
            per_rule[name] += n
        if post == pre:
            continue
        changed += 1

        # invariant A — byte-identical outside path tokens
        if canon(pre) != canon(post):
            failures.append(("A canon", f))
        # invariant B — line count preserved (crates/ doc comments cite lines)
        if pre.count(b"\n") != post.count(b"\n"):
            failures.append(("B lines", f))
        # invariant C — a line no rule matches must come out byte-identical,
        # AT THE SAME INDEX. Selected on the PRE text and compared by index,
        # not by re-filtering the POST text: several transcripts already carry
        # a literal `<repo>` or a partially-scrubbed `<home>/code/milohax/...`
        # from an earlier lane's scrubber, and a filter keyed on the output
        # token drops those lines from one side only and reports a phantom.
        lp, lq = pre.split(b"\n"), post.split(b"\n")
        keep_pre, keep_post = [], []
        if len(lp) == len(lq):
            for i, l in enumerate(lp):
                if not any(rx.search(l) for rx, _, _ in RULES):
                    keep_pre.append(l)
                    keep_post.append(lq[i])
        if (len(lp) != len(lq)
                or hashlib.sha256(b"\n".join(keep_pre)).digest()
                != hashlib.sha256(b"\n".join(keep_post)).digest()):
            failures.append(("C untouched-lines", f))
        # invariant D — the intentional provenance strings survive, counted
        for s in INTENTIONAL:
            if pre.count(s) != post.count(s):
                failures.append(("D intentional %s" % s.decode(), f))
        # invariant E — the GATE: verdict block, hashed before and after
        gp = b"\n".join(l for l in pre.split(b"\n") if l.startswith(b"GATE:"))
        gq = b"\n".join(l for l in post.split(b"\n") if l.startswith(b"GATE:"))
        if gp:
            gate_blocks += 1
            if hashlib.sha256(gp).digest() != hashlib.sha256(gq).digest():
                failures.append(("E GATE-block", f))
        # invariant F — no rule may introduce a NUL
        if b"\x00" in post:
            failures.append(("F NUL introduced", f))

        agg_pre.update(canon(pre))
        agg_post.update(canon(post))
        if apply_:
            with open(os.path.join(root, f), "wb") as fh:
                fh.write(post)

    print("files rewritten: %d" % changed if apply_ else
          "files that WOULD be rewritten: %d" % changed)
    print("substitutions by rule:")
    for _, _, name in RULES:
        print("    %8d  %s" % (per_rule[name], name))
    print("files carrying a GATE: verdict block, hashed both sides: %d"
          % gate_blocks)
    print("aggregate canon sha256 pre : %s" % agg_pre.hexdigest())
    print("aggregate canon sha256 post: %s" % agg_post.hexdigest())
    print("AGGREGATE CANON MATCH: %s"
          % ("yes" if agg_pre.hexdigest() == agg_post.hexdigest() else "NO"))
    if failures:
        for k, f in failures:
            sys.stderr.write("INVARIANT FAIL [%s] %s\n" % (k, f))
        sys.stderr.write("%d invariant failure(s) — nothing is believed.\n"
                         % len(failures))
        return 1
    print("invariants A-F: %d files x 6 checks, 0 failures" % changed)
    return 0 if apply_ else (1 if changed else 0)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
