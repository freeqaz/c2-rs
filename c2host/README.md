# c2host

A tiny x86 Windows host stub that runs the real MSVC PPC backend `c2.dll`
standalone under [wibo], used by the **P0.1 replay** path in `c2-reference`.

## What it does

`c2.dll` exposes the backend entry point as the **stdcall-decorated** export
`_InvokeCompilerPass@12` (3 args; also reachable as `InvokeCompilerPass`, or
ordinal 3). `cl.exe` normally invokes it after the C++ front-end (`c1xx.dll`)
writes the `_CL_*` IL bundle. `c2host` skips the front-end entirely: it

1. `LoadLibraryA`s `c2.dll` (by full host path — wibo resolves it and c2's
   imports, `msobj*/mspdb*/msdis*/pgodb100/msvcr100`, without any special CWD),
2. `GetProcAddress`es `_InvokeCompilerPass@12` (fallbacks: `InvokeCompilerPass`,
   then ordinal 3),
3. calls it as `int __stdcall fn(int argc, char **argv, int unk = 0)` with the
   reconstructed c2 argv (`-il <bundle base> … -Fo <obj>`).

Feeding an unmodified captured bundle back through this stub reproduces the
pipeline `.obj` **byte-for-byte** (COFF `TimeDateStamp` included — wibo pins it),
which is what makes the differential harness's reference side real.

argv contract (as passed to `c2host` under wibo):

```
wibo c2host.exe <c2.dll for LoadLibrary> <c2.dll as argv[0]> <c2 argv…>
```

The first arg is the `LoadLibraryA` target; the second becomes the backend's
`argv[0]`; the rest are the c2 backend flags.

## Build

Built **on demand** into a gitignored cache (default `target/c2host/c2host.exe`,
overridable via `C2RS_C2HOST`) by `Toolchain::ensure_c2host()`. The exact
command:

```sh
i686-w64-mingw32-gcc -static -static-libgcc -O2 -o <cache>/c2host.exe \
    c2host/c2host.c c2host/stagetap.c
```

`i686-w64-mingw32-gcc` (mingw-w64, x86) must be on `PATH`.

**Both sources, and the list is `Toolchain::c2host_sources()`.** The stub grew a
second translation unit when the stage tap landed; `ensure_c2host` rebuilds when
**any** of them is newer than the cached `.exe`, because a single-source
staleness check keeps serving an instrument built from a previous revision's
tap. If you add a third source, add it there — this section is documentation and
the function is the definition.

## What is / isn't tracked

The `.c` source **is** tracked. The built `.exe` is **never** committed — it is a
machine-specific binary regenerated on demand into the gitignored cache dir.

[wibo]: https://github.com/decompals/wibo

## The stage tap (`stagetap.c` / `stagetap.h`)

`c2host.exe` is linked from **two** sources now: `c2host.c` and `stagetap.c`.
The second installs **call-site detours** at real `c2.dll`'s own per-function
phase boundaries, so a divergence can be localized to a *pass* instead of
costing a whole-object byte-archaeology session. Read `stagetap.h` for the
contract; the short version:

* **Inert unless asked.** With `C2RS_STAGE_TAPS` unset, `tap_arm()` returns
  immediately having written nothing. Not one byte of `c2.dll` is touched and
  the process is exactly what it was before this file existed. The Rust seam
  (`c2-reference::stage`) *removes* the variable for a disarmed run rather
  than merely not setting it, so an ambient value cannot silently arm a
  control.
* **Fail-closed arming.** A site is patched only if the bytes there are still
  `e8 <rel32>` **and** the decoded original target equals the recorded target
  plus the *measured* load slide. Otherwise it prints `REFUSE` and leaves the
  image alone. Never patch a guess.
* **The slide is measured from an export, not from the `HMODULE`.** Under wibo
  `LoadLibraryA` returns `HMODULE 0x00000018` — an opaque token, not a base.
  `GetProcAddress("_InvokeCompilerPass@12")` minus its static VA `0x10bebffd`
  gives the slide, and `VirtualQuery`'s `AllocationBase` is printed beside it
  as a second derivation.
* **No `FlushInstructionCache`.** wibo does not implement it (it aborts the
  process). x86 has a coherent instruction cache, the bytes are written before
  c2 has executed them, and there is one thread.
* **No I/O inside a c2 frame.** Events are accumulated in static storage and
  printed from `main` after `InvokeCompilerPass` returns.

Usage:

```sh
C2RS_STAGE_TAPS=all      # or a comma list: sched1,color,region
```

`C2RS_STAGE_TAPS` names sites from `stagetap.c`'s table; the Rust side keeps a
copy in `c2-reference::stage::STAGE_SITES` and a test asserts the two lists are
one list.

**The tap is never a gate.** The obj byte compare against real `c2.dll` remains
the sole judge of the port; nothing is admitted on snapshot equality alone.
