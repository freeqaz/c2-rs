#!/usr/bin/env python3
"""Scrub absolute machine paths out of this lane's committed artifacts.

Board #1135: NEVER rewrite a file a background job still holds open — a scrub
that raced a backgrounded `gate.sh` punched a 122-byte NUL hole into a PASSING
gate's log, `grep` returned nothing, and a waiter reported TIMEOUT. Every writer
here has exited (each was a foreground or notified background task), and the
script asserts each file is NUL-free BEFORE and AFTER, so a raced rewrite is
caught rather than committed.
"""
import pathlib, sys

REPL = [
    ('/home/free/code/milohax/c2-rs/.claude/worktrees/agent-a68a36f09e4b45463', '<worktree>'),
    ('/home/free/code/milohax/dc3-decomp', '<dc3>'),
    ('/home/free/code/milohax/c2-rs', '<c2-rs>'),
    ('/home/free/code/milohax', '<milohax>'),
    ('/home/free', '<home>'),
]

files = [f for f in pathlib.Path('work/w-f23/staged.txt').read_text().split()]
changed = 0
for f in files:
    p = pathlib.Path(f)
    if not p.exists():
        continue
    raw = p.read_bytes()
    if b'\x00' in raw:
        sys.exit(f'REFUSING: {f} contains NUL bytes before the scrub — re-run it cleanly')
    t = raw.decode()
    if '/home/' not in t:
        continue
    for a, b in REPL:
        t = t.replace(a, b)
    assert '/home/' not in t, f'{f} still carries an absolute path'
    p.write_text(t)
    out = p.read_bytes()
    assert b'\x00' not in out, f'{f} acquired NUL bytes during the scrub'
    changed += 1
print(f'scrubbed {changed} file(s); all asserted NUL-free before and after')
