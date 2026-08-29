# PREREG ADDENDUM 1 — lane `w-globarms`

**Tier discipline, stated first because it is the point of the file.** The
scores below are reported **by tier and never pooled** (`w-globobj` §4's rule).
Two tiers exist in this lane and their provenance is different:

| tier | what was frozen, and when |
|---|---|
| **PREREG** | `work/w-globarms/PREREG.md`, commit **`a0e5b58a3`**, written and committed **before `c2.dll` was opened and before any cell was compiled**. 12 obj predictions (§3 O1–O12), 6 kind-enum predictions (§2 K1–K6), 4 read predictions (§6 R-A…R-D), 12 arm classifications (§4), a ceiling, and G1. |
| **ADDENDUM 1** | this file. Its cells (`arm2_grid.cpp`) were designed **after** the read, and their predictions were written into the grid file's own header **before** the grid was compiled — but **the grid was compiled before this file was committed.** That is a weaker provenance than PREREG's and it is named rather than smoothed over. The commit that carries the grid (`e3835448f`) contains the predictions in its header; the diff shows they were not edited after the dumps existed. |

---

## 1. THE READ — what the image says, and it is more than the prereg asked for

`FUN_10b550e5`'s twelve arms are a decision over `sym+0x04`, and the prereg's
§2 asked where that byte comes from. It comes from **one function**:

> **`FUN_10bd2913` (`0x10bd2913`) is c2's front-end → back-end symbol map.** It
> is memoised on `gl+0x10` (early-out at `0x10bd2917`, cache write at
> `0x10bd299c`), it writes the globregs kind at **`0x10bd2a1d`**, it sets
> `[sym+0x08] = sym` at `0x10bd2a20` — which is why gate A's **A3** leader test
> passes for everything it makes — and it writes back `[sym+0x00] = gl` at
> `0x10bd299f`, which is the pointer **A10** dereferences to reach `+0x37`.
> It has **19 callers**.

The kind is computed from the `.gl` record's own kind byte `[gl+0x30]`
(`P_SYMBOL.md` §1: **1 data, 3 function, 4 extern/alias**) through a four-step
`dec`-chain at `0x10bd2926`, and for `[gl+0x30] == 1` through an **8-entry jump
table at `0x10bd2a9f`** indexed by the **3-bit linkage field**
`([gl+0x37] >> 0x15) & 7` — *the same field* `P_SYMBOL.md` §3 reads at
`0x10b28bb4`, where **linkage 1 and 3 are the classes that produce no COFF
record at all**.

```
  [gl+0x30] == 1  -> the linkage table            [gl+0x30] == 3 -> kind 0xb
  [gl+0x30] == 2  -> kind 4                       [gl+0x30] == 4 -> kind 0xa
                                                  else           -> kind 0xa
  linkage 0 -> a NULL table slot: unreachable by invariant
  linkage 1 -> kind 4        linkage 3 -> kind 5
  linkage 2 -> kind 8 when ([gl+0x37] & 0x1e0) == 0x80 else kind 7   (= linkage 6)
  linkage 4 -> storage-kind switch: 1,2 -> 7; 4 -> 8; else -> 9      (= linkage 7)
  linkage 5 -> ((gl+0x20) >> 4) & 2 | 5, i.e. kind 5 or kind 7
```

**So gate A's A6 arm (kinds 4, 5) is exactly the two linkage classes that get
no COFF record — the ordinary autos — and every symbol that does get a COFF
record arrives at A8 or A9 as kind 7, 8 or 9.** That is the sentence the
prereg's K2/K5 were groping for, and it is read, not fitted.

Two further reads the prereg did not ask for:

* **`FUN_10bd2492` (`0x10bd2492`) segregates the symbol arena into FIVE
  sub-pools by kind**, each with its own free list and its own current chunk,
  all drawing 32-slot `0x60`-stride chunks from the one appended chain
  `FUN_10bd2343` maintains. `{3,6}`→`symtab+0x2c/+0x24`;
  `{0,1,2,4,5,0xd}`→`+0x30/+0x18`; `{7,8}`→`+0x34/+0x1c`;
  `{9,0xa,0xb}`→`+0x38/+0x28`; everything else→`+0x20` with no free list.
* **`FUN_10bd3225` is the symbol-TABLE constructor, not "one symbol
  allocation"** as `P_GLOBREGS` §2 has it. Its last act (`0x10bd339c`) mints a
  single record and stamps it **kind `0x10`**, parking it at `symtab+0x3c`.
  **A1 skips that one sentinel.**

## 2. ADDENDUM-1 PREDICTIONS — A6's internal test, the one separator that does not leave an arm

A6 admits kinds 4 and 5 and joins them to the `DAT_10c2e3e8` set **only when
`sym+0x05 & 2` is set**; A8's kinds 7 and 8 join it always. So A6 is the only
arm carrying a per-symbol branch that C++ source can move **without changing
the kind**. `FUN_10bd2db7` (`0x10bd2db7`) is the only setter of that bit and it
walks the leader's `+0x0c` chain, so the flag is a property of a symbol
**group**.

| # | cell | prediction | p |
|---|---|---|---:|
| **A1.1** | `gb_pair_yescape` — two `int` locals, `&y` escapes | **`x` in a register, `y` in the frame** — the flag is per-symbol | 0.75 |
| **A1.2** | `gb_pair_xescape` — the mirror | the map **swaps** | 0.75 |
| **A1.3** | `gb_pair_none` | both promoted | 0.90 |
| **A1.4** | `gb_addr_local` — `&x` taken but never escaping | **PROMOTED** — the bit is not "address taken" | 0.55 |
| **A1.5** | `gb_addr_escape` | MEMORY | 0.90 |
| **A1.6** | `gb_fnaddr2` — a function's address used for two calls | the pointer promoted; no candidate for the kind-`0xb` function symbol | 0.80 |

## 3. WHAT THE ADDENDUM DOES NOT LICENSE

It does **not** license reading A8's MEMORY verdicts as gate A's doing.
`ga_extern` / `ga_fstatic` / `ga_lstatic` are MEMORY, but a symbol with external
linkage must be observable to another TU across an opaque call **for language
reasons**, so their verdict is **confounded** and this lane says so in the
findings rather than banking three cells. A6's internal pair is not confounded:
same type, same TU, same profile, same kind, and only the escape moves.
