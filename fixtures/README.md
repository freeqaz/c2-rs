# Fixtures

These are the **inputs** to the differential harness. They are **C++ only**.

## Why only `.cpp`?

The whole point of c2-rs is that IL and obj are *never hand-maintained*. Both
are generated at test time by the reference toolchain (`cl.exe` + `c2.dll`
under wibo):

- The **IL bundle** (`_CL_*{ex,gl,sy,in,db}`) is captured on demand via the
  `/Bd /d2nop` early-abort trick (see the crate `c2-reference`).
- The **`.obj`** is produced by a normal `/Ox /GS- /c` compile.

Committing captured IL or obj would create a second source of truth that can
silently drift from the toolchain. So `.gitignore` excludes `_CL_*`, `*.obj`,
and `*.il`; only the `.cpp` under `cpp/` is tracked.

## Contents

`cpp/` — self-contained (include-free) translation units:

| File | Origin | Notes |
|------|--------|-------|
| `il_bool_materialization.cpp` | dc3-decomp il-fixtures corpus | signed/unsigned comparison → boolean materialization |
| `il_call_return.cpp` | dc3-decomp il-fixtures corpus | call / return / virtual-call shapes |
| `add3.cpp` | written here | tiny freestanding int functions |

Include-free is deliberate: no `e:\` include roots means no `WIBO_PATH_MAP` /
`WIBO_COMPUTER_NAME` string-hash determinism knobs are needed — the capture is
reproducible with a bare toolchain.

## Adding a fixture

Drop a self-contained `.cpp` into `cpp/`. `c2rs bench` picks up every
`cpp/*.cpp` automatically. Do not add headers or include paths without also
wiring the include/path-map handling in `c2-reference` — the current capture
path is include-free by design.
