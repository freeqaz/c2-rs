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
i686-w64-mingw32-gcc -static -static-libgcc -O2 -o <cache>/c2host.exe c2host/c2host.c
```

`i686-w64-mingw32-gcc` (mingw-w64, x86) must be on `PATH`.

## What is / isn't tracked

The `.c` source **is** tracked. The built `.exe` is **never** committed — it is a
machine-specific binary regenerated on demand into the gitignored cache dir.

[wibo]: https://github.com/decompals/wibo
