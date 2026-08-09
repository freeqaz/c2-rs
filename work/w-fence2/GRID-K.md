# GRID-K — the `.gl` DEFINED-record linkage byte, and the INLINE bit beside it

Cell: `work/w-fence2/probe/k1.cpp`, captured at `/nologo /c /GR /O1 /Oi /EHsc
/GS-` (`work/w-fence2/probe/flags_o1.txt`); IL in
`work/w-fence2/probe/il-k1/_CL_68a212f5.gl`. Readers: `work/w-fence2/glread.py`,
`work/w-fence2/glctx.py`.

## 0. Why the cell exists

`gl.rs::linkage_needs_a_directive` reads the linkage byte at a fixed
`name_nul + 3` and its own doc records the hole this grid closes:

> *"the **defined-record** value set could not be separated from them without
> replicating this function's own framing."*

Fifteen defined records here, every one with a framed body-start offset, one per
linkage form.

## 1. The measured table

Bytes are `gl[name_nul + 1 ..]` — `<tag> <kind> <linkage> <retsize> <flags>`.

| source form | name in `.gl` | tag kind | **+3 linkage** | +4 retsize | **+5 flags** |
|---|---|---|:--:|:--:|:--:|
| `extern "C" void f()` | `k_ext_a` | `82 07` | **05** | 00 | **00** |
| `extern "C" static void f()` | `k_stat_a` | `82 07` | **03** | 00 | 00 |
| `extern "C" __forceinline void f()` | `k_cfi_a` | `82 07` | **05** | 00 | **20** |
| `extern "C" static __forceinline void f()` | `k_sfi_a` | `82 07` | **03** | 00 | 00 |
| `__declspec(dllexport)` | `k_exp_a` | `82 07` | **09** | 00 | 00 |
| C++ free function | `?k_cpp_ext@@YAHH@Z` | `86 01` | **05** | 04 | **00** |
| C++ `static` free function | `?k_cpp_stat@@YAHH@Z` | `86 01` | **03** | 04 | 00 |
| C++ `inline` | `?k_cpp_inline@@YAHH@Z` | `86 01` | **05** | 04 | **20** |
| C++ `__forceinline` | `?k_cpp_fi@@YAHH@Z` | `86 01` | **05** | 04 | **20** |
| member defined IN-CLASS (implicitly inline) | `?m_in@KS@@QAAHH@Z` | `86 01` | **05** | 04 | **20** |
| member defined OUT-of-class | `?m_out@KS@@QAAHH@Z` | `86 01` | **05** | 04 | **00** |

## 2. Two findings, and the second is the one that makes the fence safe

> ### **K1 — `05` IS "defined, EXTERNAL linkage", ON DEFINED RECORDS, AND `03` IS `static`.** `gl.rs`'s three-value reading holds where it had never been separated. `05` / `03` / `09` are the only values these fifteen records take.

> ### **K2 — THE LINKAGE BYTE DOES NOT SEE `__forceinline`, AND THE BYTE AFTER IT DOES.** `k_ext_a` and `k_cfi_a` differ in **exactly one byte of the whole record** — `00` against `20` at `name_nul + 5`. Same for `?k_cpp_ext` against `?k_cpp_fi`. The bit is set for `inline`, for `__forceinline` and for an implicitly-inline member, and clear for a plain external, an out-of-line member, a `static`, and — because the concept does not apply to internal linkage — for `static __forceinline`.

**Why K2 is load-bearing.** `WB_INLINE_FINDINGS` **F4** measured
`__forceinline` inlining a **980-byte** callee at `/O1` *and* `/O2`: it bypasses
every size test there is. A decline rule keyed on size is therefore **wrong** on
a `__forceinline` callee, and `w-inlfence`'s decline **D9** said a
`__forceinline` cell *"would grade nothing"* because the coarse fence refused it
along with everything else. Under a narrowed fence it grades a **wrong obj** —
and this byte is what stops it.

## 3. What the fence reads, and how it fails

    linkage(name) == 0x05           defined, external, not `static`, not dllexport
    AND gl[name_nul + 4] < 0x80     the return-type size is the one-byte form
    AND gl[name_nul + 5] == 0x00    NOT inline / __forceinline / implicitly inline

**Fail-closed on all three.** The `+4 < 0x80` guard is not decoration: `+4` is
the return type's size (`00` void, `04` int, `14` for `gl.rs`'s 20-byte
aggregate), and a return type large enough to escape the one-byte form would
shift `+5` — so an unreadable width refuses rather than reading the wrong byte.
Requiring `+5 == 0x00` exactly, rather than testing bit `0x20`, refuses every
value this grid did not see.

**The subject passes:** `vsnprnc.cpp`'s `.gl` gives `_vsprintf_s_l` and
`vsprintf_s` both `86 01 05 04 00` — linkage `05`, retsize `04`, flags `00`.
