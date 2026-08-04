# tools/llvm — a second implementation that can read our objs

`crates/c2-obj`, `scripts/gt_dump.py` and `tools/coffdump.py` are three COFF
readers, and all three are ours. Until this lane, every disagreement about what
an obj contains was settled by code written by the project under test.

LLVM can read these files. It refuses them out of the box for one reason only:
`identify_magic()` has no case for `IMAGE_FILE_MACHINE_POWERPCBE = 0x01F2`.
Get past that and `llvm-readobj` decodes the header, the section table, the
symbol table with aux records, and every relocation; `llvm-objdump -d`
disassembles big-endian PowerPC per COMDAT with symbol names; `--codeview`
decodes `.debug$S` and `.debug$T` in full.

**This is diagnostic capability, not a gate.** Nothing here is or ever becomes
the correctness judge — that is the real `c2` under wibo plus a byte-exact obj
compare, and it stays that way. Nothing here is linked into the workspace
either; LLVM is a subprocess, in the same category as `scripts/plot_perf.py`.

## Two ways to get there; you probably want the first

### 1. Stock distro LLVM, no build at all

Any `llvm-readobj` new enough to have a COFF dumper works, on a **scratch copy
of the obj whose machine word is rewritten `0x01F2` → `0x01F0`**. Every script
here does that automatically and writes the copy under
`$C2RS_LLVM_SCRATCH` (default `work/w-llvm/scratch/`).

Verified on this box with Arch `llvm 22.1.8` from `PATH`: identical decode to
the patched build on all 155 objs and 5,478,967 field comparisons.

The scratch copy is a diagnostic convenience. It is never an input to the port
and never compared as bytes; the machine word is the only byte that differs and
the cross-check knows it.

Cost: 0. Limits: relocation types print as `Unknown (16)` rather than
`IMAGE_REL_PPC_REFHI` (the raw number is still shown), `llvm-objdump` needs
`--triple=powerpc-unknown-unknown` spelled out, and the format line reads
`COFF-<unknown arch>`.

### 2. The patched build

`ppcbe.patch` — 89 added lines across 5 files, nothing modified — against
upstream tag **`llvmorg-22.1.8`**, commit
**`ca7933e47d3a3451d81e72ac174dcb5aa28b59d1`**. Verified to apply cleanly to a
pristine checkout of that tag.

```sh
tools/llvm/build.sh                 # clone + patch + build, ~4 minutes
export C2RS_LLVM_BIN=$HOME/build/llvm-w-llvm/build-ppcbe/bin
```

Measured here (AMD Ryzen 9 7950X, gcc 16.1.1, ninja 1.13.2, `-j8`, one link job):

| step | cost |
|---|---|
| `git clone --depth 1 --branch llvmorg-22.1.8` | 2.6 GB working tree + 291 MB `.git` |
| `cmake` configure (`LLVM_TARGETS_TO_BUILD=PowerPC`, Release, no tests) | ~30 s |
| `ninja llvm-readobj llvm-objdump llvm-mc` | **190 s wall**, 870 targets |
| build tree | **177 MB** |
| binaries | `llvm-readobj` 11 MB, `llvm-objdump` 11 MB, `llvm-mc` 3.6 MB |

What the patch buys, and it is honest to call it small: the objs are accepted
unmodified (no scratch copy), `Arch: powerpc` so `llvm-objdump` needs no
`--triple`, and relocation types get names. **It buys no decoded field that
route 1 does not already decode.** If you are picking this up to answer a
question about an obj, use route 1 and skip the build.

> The relocation *names* the patch adds are transcribed from the MS PE/COFF
> specification revision 6.0 §5.2.6 and cross-checked against
> `gimli-rs/object`'s `src/pe.rs`. LLVM has never had a PPC COFF relocation
> table. So the names are **not** independent evidence — a check that leans on
> them is checking two written sources against each other. Everything under the
> name (record decode, offsets, symbol indices) is LLVM's own code and is
> independent.

## Environment

| variable | meaning | default |
|---|---|---|
| `C2RS_LLVM_BIN` | directory holding `llvm-readobj` etc. | — |
| `C2RS_LLVM_PREFIX` | same, one level up (`$PREFIX/bin`) | — |
| `C2RS_LLVM_SCRATCH` | where machine-patched scratch copies go | `work/w-llvm/scratch` |
| `C2RS_LLVM_SRC` | where `build.sh` clones LLVM | `$HOME/build/llvm-w-llvm` |
| `C2RS_LLVM_TAG` / `C2RS_LLVM_JOBS` | build tag / compile jobs | `llvmorg-22.1.8` / `8` |

Resolution order is `C2RS_LLVM_BIN` → `C2RS_LLVM_PREFIX/bin` → `PATH`.
**With no LLVM anywhere, every script prints `SKIP: llvm-readobj absent …` and
exits 0.** That is the project rule for anything touching an external
toolchain, and it is why none of this can appear in a gate as a silent pass.

## The scripts

| file | what it does |
|---|---|
| `llvmpath.py` | locate LLVM; decide whether a scratch copy is needed; `SKIP` cleanly |
| `readobj_parse.py` | parse `llvm-readobj`'s indented text (there is no COFF JSON writer) |
| `xcheck.py` | **the deliverable** — decode objs with LLVM *and* with all three of our readers and report every disagreement |
| `longname_probe.py` | build a synthetic obj with a `/NNN` long section name and see which readers resolve it |
| `c2objdump/` | 60-line out-of-workspace Rust bin that prints what `crates/c2-obj` sees |
| `ppcbe.patch`, `build.sh` | the patch and its build recipe |

```sh
# the cross-check, over whatever objs you have
tools/llvm/xcheck.py work/w-llvm/objs/*.obj
tools/llvm/xcheck.py --tsv diffs.tsv work/w-llvm/objs/*.obj

# prove the cross-check can still fail (see below)
tools/llvm/xcheck.py --selftest work/w-llvm/objs/*.obj
```

Reference objs come from `work/w-frame/refobj.sh <src.cpp> <out.obj>`, which
compiles a real dc3 TU at the workload's own flags. Do not point any of this at
`work/capture-cache`.

## `--selftest`, and why corrupting the obj is not a control

`xcheck.py` reports "compared N objs / M field instances / K disagreements".
A run that graded nothing must not look like a run that agreed — this repo has
14 recorded instances of absence read as success — so the comparison count is
printed, and a run with zero comparisons exits non-zero.

The obvious control is wrong. Mutating a byte in the obj **is not detected**,
and should not be: both readers read the same mutated file and agree about it.
That was tried here first and it reported 0 diffs, exactly as it must.

`--selftest` instead perturbs **one field on our side only** (`sec.rawsize`,
after decode) and requires the comparison to report exactly one disagreement per
section and no others. That is the check that the instrument can still fail.

## Known limits, measured

* **Relocation type names**: stock LLVM prints `Unknown (16)`; the raw number is
  always right. The patch names them, from the sources noted above.
* **CodeView register names come out x86.** Confirmed on a `/Z7` capture:
  `S_REGISTER { Seg: BL (0x4), Name: this }` — register 4 holding `this`, which
  on this ABI is `r3`, printed with the x86 table because LLVM carries no PPC
  CodeView register enum. The *number* is right; the name is not.
* **LLVM cannot write PPC COFF** (no `PPCWinCOFFObjectWriter`, no relocation
  writer). Read-only, permanently.
* **`llvm-mc` and VMX128** — see `docs/ABI_EDGES.md` and
  `docs/rungs/_2026-08-04-w-llvm.md`. Opcode-6 and opcode-4 VMX128 words decode
  as *plausible legal* Power10/AltiVec instructions with **no diagnostic**.
  Scalar/FP is exact.
