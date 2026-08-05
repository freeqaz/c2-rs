# tools/vmx — VMX128, decoded correctly and checkably

Lane `w-vmx`, 2026-08-05. Full specification: **`docs/VMX128_DECODE.md`**.

**Not a gate, and never will be.** The sole judge of the port is the real
`c2.dll` under wibo plus a byte-exact obj compare. This directory is diagnostic
capability, in the same category as `tools/llvm/` and `scripts/plot_perf.py`:
Python tooling **outside** the std-only Rust workspace. Nothing under `crates/`
is touched, and nothing here is derived from disassembling `c2.dll` — reading
the compiler's own published `/FAcs` listing is black-box observation, so no
`docs/whitebox/DISCLOSURE.md` row is implied.

## The problem

`llvm-mc -triple=powerpc` decodes VMX128 into **plausible legal modern PowerPC
with no diagnostic**. `18 00 07 10` is `vrlimi128`; LLVM prints
`lxvp 0, 1808(0)` and exits 0. `10 23 20 c3` is `lvx128 vr1,r3,r4`; LLVM prints
`vucmprlb 1, 3, 4` and exits 0. "The disassembler did not complain" is worth
nothing here.

## The fix: a positive check with a printed count

`cl /FAcs` makes the Microsoft back end narrate the bytes it just emitted —
`00000\t102320c3\t lvx128 vr1,r3,r4`. That is the oracle: it cannot be wrong
about what `c2` emitted, because it is `c2` reporting what it emitted. Every
decode here is graded against it.

```
  in scope (primary opcode 4/5/6)    6701
  decoded (matched a table row)      6701
  VERIFIED (mnemonic + operands)     6701
  MISMATCH (wrong mnemonic)          0
  UNRECOGNIZED (no table row)        0
  distinct VMX128 mnemonics seen     66 of 77
```

**A run that verifies zero instructions exits non-zero.** `--selftest` perturbs
the decode on *our* side only and requires one MISMATCH per instruction, which
is the check that the instrument can still fail. (Corrupting the input is not a
control — both sides read the same corrupted input and agree; `tools/llvm/`
records that this was tried there first and correctly reported 0 diffs.)

## Files

| file | what it does | deps |
|---|---|---|
| `vmx128_isa.py` | **generated** tables: 77 VMX128 rows + 178 other opcode-4/5/6 encodings, from powerpc-rs `isa.yaml` at a pinned commit and sha256 | stdlib |
| `gen_isa_table.py` | regenerates the above | PyYAML + network; `SKIP`s cleanly without either |
| `vmx128.py` | the split-register bit-field machinery and the operand printer | stdlib |
| `codparse.py` | reads an MSVC `/FAcs` `.cod` — **the oracle side** | stdlib |
| `llvmmc.py` | asks `llvm-mc`, reconstructing the per-word answer exactly from its stdout/stderr split | stdlib |
| **`vmxcheck.py`** | **the deliverable** — grade decodes against the listing, count everything, fail loudly | stdlib |
| `collide.py` | the collision table: exact (computed) + measured (llvm-mc) | stdlib |
| `vmxscan.py` | prevalence census over an object tree | stdlib + `tools/coffdump.py` |
| `genprobe.py` | emit a TU that makes `cl` produce the VMX128 opcode space | stdlib |
| `listing.sh` | run the real `cl.exe` with `/FAcs` | wibo + `compilers/` |
| `build_objs.sh` | compile all 878 workload TUs, one obj per directory | as above |

## Use

```sh
tools/vmx/build_objs.sh                        # ~30 s, ~102 MB
tools/vmx/vmxscan.py work/w-vmx/objs           # how much VMX128 is in there

tools/vmx/listing.sh src/system/synth_xbox/FFT.cpp work/w-vmx/lst/fft
tools/vmx/vmxcheck.py --llvm work/w-vmx/lst/*/*.cod
tools/vmx/vmxcheck.py --selftest work/w-vmx/lst/*/*.cod

tools/vmx/vmx128.py 102320c3 18000710          # decode words on the command line
tools/vmx/collide.py                           # the collision table
```

## Two traps, both hit here on the first run

* **`work/capture-cache` has 2,081,021 entries.** Never glob it, never `find`
  or `du` from the repo root; two kernel OOM kills came from exactly that.
  `vmxscan.py` walks an explicit one-level directory and nothing else.
* **A decoder with the wrong CPU profile lies the same way LLVM does.** On its
  first run `vmx128.py` printed `ps_sel fr10,fr10,fr11,fr9` for the real
  workload word `0x114a4aee`, which `cl`'s listing calls
  `vmaddfp vr10,vr10,vr11,vr9` — because `isa.yaml` also carries the Gekko
  PairedSingles extension in the same primary opcode 4. Hence
  `vmx128.XENON_PROFILE`. The oracle caught it in the first minute; nothing
  else would have.

## The answer, if you only want the number

**3 of 878 TUs (0.344 %), 7 functions, 113 instruction words of 3,866,911
(0.0029 %).** `docs/VMX128_DECODE.md` §7 recommends **not** building VMX128
codegen and says what would change that.
