#!/usr/bin/env python3
"""Fill the rung's GATE_COUNTS placeholder from the gate's own artifact, which is
asserted NUL-free first (board #1135: a green gate that cannot be read is
indistinguishable from a gate that never ran)."""
import pathlib

raw = pathlib.Path('work/w-f23/gate_final.txt').read_bytes()
assert raw.count(b'\x00') == 0, 'gate artifact contains NUL bytes — re-run cleanly'
t = raw.decode(errors='strict')
i = t.index('\nlanes:  ')
j = t.index('\nlogs:', i)
block = t[i + 1:j]
assert 'GATE: PASS' in t, 'no PASS verdict in the gate artifact'

extra = """
Re-run at the FINAL tip. An earlier run at tree `82130a9c` gave the identical
counts, and what landed after it is `docs/`, `work/` and **100 lines inside
`#[cfg(test)] mod tests`** — checked by walking the unified diff's new-file line
numbers against the `#[cfg(test)]` marker (added lines 1758-1857, marker at
1636, **0 lines removed**), not by eye.

**`graded: 4860` against the brief's baseline of 4,770 is master's own number,
not this lane's.** 4,860 = 270 fixtures x 18 lanes; 4,770 = 265 x 18. The five
came from `w-align` (`d04a7e40`, *"five w-align cells, three that convert and two
that must keep refusing"*), which landed between `w-gen`'s rung and this lane's
base. `git diff ceca69b4..HEAD -- fixtures/` is **empty** — this lane adds no
fixture, and its `Fixtures:` line says why.
"""

p = pathlib.Path('docs/rungs/2026-08-08-w-f23.md')
s = p.read_text()
assert 'GATE_COUNTS' in s, 'placeholder already filled'
s = s.replace('GATE_COUNTS', block.rstrip())
s = s.replace('```\n\n**`cargo test --workspace --release`**',
              '```\n' + extra + '\n**`cargo test --workspace --release`**', 1)
s = s.replace('**36 targets, 1,122 passed, 0 failed, 1\nignored** (baseline **1,120 / 36 / 0**; **+2** is F2\'s two tests, and F3\'s third\nlands with it)',
              '**36 targets, 1,123 passed, 0 failed, 1\nignored** (baseline **1,120 / 36 / 0**; **+3** is this lane\'s two F2 tests and one\nF3 test)')
p.write_text(s)
print('filled')
