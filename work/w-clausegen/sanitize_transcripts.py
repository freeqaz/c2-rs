#!/usr/bin/env python3
"""Relativise absolute machine paths out of this lane's committed transcripts.

`CLAUDE.md`'s never-commit list includes absolute machine paths; class 3 of
`scripts/tracked_artifact_audit.sh` is the enforcement, and it went RED on
`work/w-clausegen/gate.out` and `work/w-clausegen/cargo_test.out`.

# The trap this is written to avoid, which is not hypothetical

The coordinator hit it at a previous merge: **a sed that substitutes only the
prefix you thought of reports clean over the wrong population.** In these two
files there are THREE distinct roots, not one:

  1. the worktree               .../\u200bc2-rs/.claude/worktrees/agent-<id>
  2. the SHARED main checkout   .../\u200bc2-rs            <- lines 38/42/50/54
  3. anything else under /home/<user>/

Substituting (1) alone leaves every `work/corpus/gen-*` line from the shared
tree intact, and those are the lines that carry the sweep's REFUSED-generation
message. So this replaces **longest root first**, then sweeps whatever is left
with a generic pattern.

# And it does not trust its own pattern

The substitution is verified with **the audit's own two regexes**, lifted
verbatim from `scripts/tracked_artifact_audit.sh` (`ABS_FWD`, `ABS_BS`) — both
spellings, because the reference side runs under wibo and writes
`z:\\home\\<user>\\...` into oracle logs, and 26 tracked files once carried
*only* that form. Checking with the pattern you substituted on is how a
sanitizer reports success over the half it did not consider.

Idempotent. Exits non-zero if anything survives.
"""
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TARGETS = ['work/w-clausegen/gate.out', 'work/w-clausegen/cargo_test.out']

# Lifted VERBATIM from scripts/tracked_artifact_audit.sh lines 150-151.
ABS_FWD = re.compile(r'/home/[a-z][a-z0-9_-]*/')
ABS_BS = re.compile(r'\\home\\[a-z][a-z0-9_-]*\\')

# The shared checkout is this worktree's parent repo. Derived, never hardcoded:
# a hardcoded root is the same defect one level up.
WORKTREE = REPO
MAIN = os.path.realpath(os.path.join(REPO, '..', '..', '..')) \
    if '/.claude/worktrees/' in REPO else REPO


def sanitize(text):
    # LONGEST ROOT FIRST. The worktree path CONTAINS the main path as a prefix,
    # so main-first would rewrite the worktree lines into `<REPO>/.claude/...`
    # and leave a correct-looking but wrong attribution.
    for root, tag in sorted([(WORKTREE, '<WORKTREE>'), (MAIN, '<REPO>')],
                            key=lambda kv: -len(kv[0])):
        text = text.replace(root, tag)
        text = text.replace(root.replace('/', '\\'), tag)
    # Whatever is left: any other user path, in either spelling.
    text = ABS_FWD.sub('<HOME>/', text)
    text = ABS_BS.sub(r'<HOME>\\', text)
    return text


def main():
    print(f"worktree root : {WORKTREE}")
    print(f"main checkout : {MAIN}")
    print(f"(distinct roots: {'YES' if WORKTREE != MAIN else 'NO'})\n")
    bad = 0
    for rel in TARGETS:
        p = os.path.join(REPO, rel)
        before = open(p, encoding='utf-8', errors='replace').read()
        n_fwd = len(ABS_FWD.findall(before))
        n_bs = len(ABS_BS.findall(before))
        after = sanitize(before)
        if after != before:
            open(p, 'w', encoding='utf-8').write(after)
        # VERIFY WITH THE AUDIT'S REGEXES, not with the substitution above.
        r_fwd = len(ABS_FWD.findall(after))
        r_bs = len(ABS_BS.findall(after))
        lines = sum(1 for l in before.splitlines()
                    if ABS_FWD.search(l) or ABS_BS.search(l))
        print(f"{rel}")
        print(f"  matches before : {n_fwd} forward + {n_bs} backslash, "
              f"over {lines} line(s)")
        print(f"  matches after  : {r_fwd} forward + {r_bs} backslash")
        if r_fwd or r_bs:
            print("  RESIDUE:")
            for i, l in enumerate(after.splitlines(), 1):
                if ABS_FWD.search(l) or ABS_BS.search(l):
                    print(f"    {i}: {l}")
            bad += 1
    print(f"\nSANITIZE: {'RED' if bad else 'GREEN'} "
          f"({bad} file(s) still carrying an absolute path)")
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
