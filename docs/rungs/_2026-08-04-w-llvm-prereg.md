# w-llvm — prediction, registered before measuring

    Lane:      w-llvm, 2026-08-04, worktree `wt-w-llvm` off master `c303ad0`
    Status:    PREREG. Scored in `_2026-08-04-w-llvm.md` §1.

Written before the first `llvm-readobj` invocation of the lane and copied here
verbatim from `work/w-llvm/PREDICTION.txt` (which is under the gitignored
`/work`). It was **written** before measuring and **committed** afterwards, in
the same session — the ordering is stated rather than proven, and the scoring
in §1 quotes it unmodified.

```
w-llvm — predictions registered 2026-08-04, BEFORE any measurement.

P1. The whole LLVM source build is avoidable. A 2-byte edit to a *scratch copy of
    the obj* (machine 0x01F2 -> 0x01F0 at file offset 0..2) will make the STOCK
    system llvm-readobj (Arch llvm 22.1.8) parse the file, because the machine
    word is consulted only by identify_magic, by reloc-type-name lookup, and by
    disassembler triple selection -- not by any structural field decode.
    Confidence ~80%. If true, PRIOR_ART's "patch a scratch copy" is best read as
    patching the obj, and the brief's instruction to patch llvm/lib/BinaryFormat/
    Magic.cpp is a more expensive route to the same place.

P2. --codeview decodes .debug$S (S_OBJNAME, S_COMPILE2) as PRIOR_ART claims.
    Confidence ~75%. CodeView records are machine-independent LE.

P3. Relocation type names print as something useless (Unknown / a wrong x86 or
    ARM name). Confidence ~85%.

P4. THE ONE THAT MATTERS. Disagreement count between llvm-readobj and our own
    decoder on *field values* will be LOW -- I predict 0-2 genuine value
    disagreements across the whole sample. Both sides read little-endian structs
    at fixed offsets; the interesting divergence will be in *interpretation*
    (reloc type names, aux-record union selection, COMDAT selection, string
    table / long name handling), not in the numbers. If I am wrong and LLVM
    disagrees on a structural field, that is the single most valuable result
    this lane can produce and it goes at the top of the report.

DECLINE CONDITION (stated in advance). I decline the "build LLVM from source"
half of the lane if P1 holds AND a same-day field-by-field compare shows LLVM
decoding no COFF field that crates/c2-obj does not already decode. In that case
the honest deliverable is: the diagnostic capability is available today for
zero build cost, the source patch buys only cosmetics (no scratch copy, real
reloc names), and I say so instead of spending hours on cmake.

I will NOT decline on "the comparison found no disagreements" -- a wide compare
that agrees is a real result, provided N objs and M fields are both stated and
both large.
```
