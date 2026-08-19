# `P_SYMBOL` — the `.gl` symbol record → COFF symbol record

> **Reference page.** Provenance marks are mandatory and mean exactly this:
> **`[R]`** read from the disassembly and *not* obj-checked — a hypothesis;
> **`[O]`** confirmed against a real obj or a `/FAsc` listing, with the witness
> named; **`[I]`** an interpretive step on top of an `[R]` or `[O]`.
> `[R]` means *"the instructions were read correctly"*, **never** *"this is what
> c2 does"* — those come apart, and `C2_MAP_METHOD.md` §7 is the priced example.
> Navigation only: nothing on this page may enter `crates/` without a
> [`DISCLOSURE.md`](../DISCLOSURE.md) row naming the address.
>
> Addresses are absolute VAs in `c2.dll`
> `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.
> Index: [`ADDR.tsv`](ADDR.tsv) · whole-image index: [`FUNCS.tsv`](FUNCS.tsv) ·
> front door: [`README.md`](README.md)

**Why this page exists.** `CEILING.md` §6.1 **phase 5** ("weak externals at
scale", `alias-weak-needed-tus` **675 of 871**) and **phase 6** ("COMDAT
synthesis", §2.3's **450**) had *no* whitebox page and *no* findings document —
searching `docs/whitebox/` for `weak.?extern` at base `e82c9ede6` returned
exactly one hit, and it was `DISCLOSURE.md` mentioning the word in passing.
Both phases are decided in **one 724-byte function**, `0x10b28a9b`, and this
page is that function.

**Coverage: 27 addresses in `FUN_10b28a9b` and its four callees.** Not covered:
the string-table encoder (`P_COFF.md` §2), the relocation writer, and *where the
symbol-record flag words are set* — this page reads them, it does not find their
authors.

---

## 1. The record's field map, as this function reads it

Offsets are into the `.gl` symbol object c2 keeps in memory. Every row is
`[R]` — the reading of an instruction — unless marked otherwise. **A field's
meaning here is what *this* function assumes about it, which is not the same as
what the field is.** `+0x4c` is the standing proof of that; see §5.

| offset | width | what this function does with it | site |
|---|---|---|---|
| `+0x18` | 4 | the COFF `Value` for most paths | `0x10b28cfa`, `0x10b28c04` |
| `+0x20` | 4 | a second flag word: `0x80000` forces `Value = 4`; `0x20000000` routes to the aux-bearing writer | `0x10b28d1d`, `0x10b28c11` |
| `+0x24` | 4 | **the assigned COFF symbol index**, `0` = not yet assigned | `0x10b28cbe`, `0x10b28ce6` |
| `+0x30` | 1 | **the record KIND**: `4` extern/alias, `1` data, `3` handled at `0x10b28b4c`; anything else writes nothing | `0x10b28ba3`, `0x10b28ca8` |
| `+0x31` | 1 | a sub-kind; `{0x54,0x55,0x56}` is what makes a kind-3 symbol a *function* | `0x10b2823b` |
| `+0x32` | 1 | per-symbol emitted flags. **bit 0** = "already written" (idempotence); **bit 2** = "an alias names me" | `0x10b28aa3`, `0x10b28c92` |
| `+0x37` | 4 | the flag word. **bits 5..8** storage kind; **bit 9** `0x200` dllexport; **bit 10** `0x400` function type; **bits 21..23** linkage; **bit 22** `0x400000` the ALIAS bit | `0x10b28be6`, `0x10b28ba8`, `0x10b28b02` |
| `+0x3c` | 4 | one further alias hop, taken only for a kind-1 alias onto a kind-4 target | `0x10b28cb4` |
| `+0x3f` | 4 | **a second weak-alias token**, on the storage-kind-2 route | `0x10b28c7d` |
| `+0x4c` | 4 | tested as `& 0x20` (**the emit bit**) *and* loaded whole as the alias token. **See §5** | `0x10b28ae4`, `0x10b28b09` |

---

## 2. Phase 5 — the weak external, instruction by instruction

`alias-weak-needed-tus` is **675 of 871**, and `CEILING.md` §6.1 records that
**"no factor in §10.19 represents it"**. This is the code.

Two disjoint routes reach one join point at `0x10b28c88`:

| route | condition | site |
|---|---|---|
| **alias** | `[sym+0x30] == 4` **and** `[sym+0x37] & 0x400000` (the ALIAS bit `W-ALIAS-1` adopted). The token is `[sym+0x4c]`, loaded **whole** | `0x10b28b02` → `0x10b28b09` → `0x10b28b0c` |
| **storage-kind 2** | `[sym+0x30] == 1` **and** `(([sym+0x37]>>5)&0xF) == 2` **and** `[sym+0x3f] != 0`. A **different token field** | `0x10b28be6` → `0x10b28c7d` → `0x10b28c82` |

> **The second route is the load-bearing part of this section.** It is invisible
> to any obj grid built from aliases, because it is not keyed on the alias bit
> and it reads a different field. A lane that black-boxes weak externals by
> mutating tag-`0x10` records measures route one and cannot know route two
> exists. `[R]`.

Then, in order:

| # | what | site | mark |
|---|---|---|---|
| 1 | resolve the token → the target symbol (`FUN_10b9860d`, `p2symtab.c`, 31 callers) | `0x10b28c8b` | `[R]` |
| 2 | `or [target+0x32], 4` — "an alias names me" | `0x10b28c92` | `[R]` |
| 3 | **if `[target+0x37] & 0x200000`, call the emit-marker `FUN_10b276e4`** | `0x10b28c96` → `0x10b28ca3` | `[R]` |
| 4 | one further hop when a kind-1 alias names a kind-4 target: `target = [target+0x3c]` | `0x10b28ca8`–`0x10b28cb4` | `[R]` |
| 5 | **recurse into this same function on the target** | `0x10b28cb9` | `[R]` |
| 6 | if `[target+0x24] == 0`, create the DEFAULT symbol: `StorageClass = 2` (EXTERNAL), `Value = 0`, `SectionNumber = 0`; latch its index | `0x10b28cbe`, `0x10b28cd2`, `0x10b28ce1`, `0x10b28ce6` | `[R]` |
| 7 | emit the alias: `StorageClass = 0x69` (`IMAGE_SYM_CLASS_WEAK_EXTERNAL`), `Value = [sym+0x18]`, `Type` from `0x10b2823b`, aux `TagIndex = [target+0x24]`, aux `Characteristics = 2` | `0x10b28cfd`, `0x10b28cfa`, `0x10b28cec`, `0x10b28cea`, call at `0x10b28d0c` | `[R]` |
| 8 | `FUN_10b2af4f` (43 bytes) writes the 18-byte aux as `{TagIndex, Characteristics}` — the COFF-spec layout | `0x10b2af4f` | `[R]` |

### 2.1 What is `[O]`, and the instrument that already made it so

**Three of the eight rows above were already obj-confirmed, at full workload
scale, before this lane read a single byte** — and the confirming instrument is
in `crates/`, not in `docs/whitebox/`:

| claim | witness | scale |
|---|---|---|
| the class is `0x69` | `ObjImage::weak_externals`, `crates/c2-obj/src/lib.rs` | every obj the scan reads |
| aux `Characteristics == 2` | gap key **`alias-weak-not-search-library`**, a **KNOWN ANSWER 0** | the 878-TU workload |
| aux `TagIndex` names the default | gap key **`alias-weak-default-disagree`**, a **KNOWN ANSWER 0** | the 878-TU workload |
| the symbol-table shape `#535 EXTERNAL ??_G…` / `#536 WEAK_EXTERNAL ??_E… → default #535, Characteristics = 2` | `w-phase7`, quoted verbatim in `ObjImage::weak_externals`'s doc comment | one probe (`HamUser.cpp`) |

**So the disassembly's marginal product on phase 5 is not the constants.** It is:

1. **the ORDERING RULE** (step 5 + step 6): the default symbol precedes the weak
   record *because the emitter recurses into the target first*, and the default
   is minted on demand. The obj shows `#535` then `#536` and **cannot
   distinguish a rule from an adjacency** — `[I]` on `[R]`;
2. **the SECOND ROUTE** (`[sym+0x3f]`, storage-kind 2), which no alias-keyed
   grid can reach;
3. **the GUARD** — which `.gl` symbols take the path at all.

This is priced in `rungs/2026-08-19-c2map3.md` §4, and it is the lane's answer
to the strategic question.

---

## 3. Phase 6 — where a symbol gets a section, and where it does not

`CEILING.md` §6.1 phase 6 is *"TUs carrying an emitted symbol with no `.gl` body
record"* — **450** of them, *"no binding repair reaches them and no phase in the
plan builds this"*. The decision is a four-way `dec`-chain on the storage-kind
field at **`0x10b28be6`**, `(([sym+0x37] >> 5) & 0xF)`:

| value | target | `StorageClass` | `SectionNumber` | `Value` | mark |
|---:|---|---|---|---|---|
| **1** | `0x10b28d1d` | `2` EXTERNAL | `FUN_10b287b8([sym+0xc])` — the per-section finalize that ORs `IMAGE_SCN_LNK_COMDAT` (`P_COFF.md` §2) | `4` when `[sym+0x20] & 0x80000`, else `[sym+0x18]` | `[R]` |
| **2** | `0x10b28c7d` | — | — | — | **route to §2's weak path when `[sym+0x3f] != 0`**; otherwise `0x10b28d16`, an undefined external `[R]` |
| **3** | `0x10b28c6e` | — | — | `[sym+0x1c]` per the decompiled body, `SectionNumber = 0` | `[R]`, **and not disassembled instruction-by-instruction by this lane** |
| **else** | `0x10b28c00` | `3` STATIC | `FUN_10b287b8([sym+0xc])` | `[sym+0x18]` | `[R]` |

And **two ways a `.gl` symbol produces no COFF record at all**:

* `[sym+0x32] & 1` — already written (`0x10b28aa3`) `[R]`;
* `(([sym+0x37] >> 0x15) & 7) ∈ {1, 3}` — a **linkage class that is suppressed
  outright**, `0x10b28bb4` / `0x10b28bbd` `[R]`. Note bit 21 of the same word is
  separately the emit-marker guard read at `0x10b28c96`; **the 3-bit field and
  the single bit are read by different sites and this page does not claim they
  are the same thing.**

> **What this does NOT settle.** Phase 6's 450 TUs are defined by the *absence*
> of a `.gl` body record, and nothing above proves that the storage-kind-3 arm
> is what serves them. The honest statement is: **the branch that decides
> "section or no section" is `0x10b28be6`, and a lane opening phase 6 starts
> there instead of building a probe grid.** Which arm the 450 take is a
> measurement this lane did not run.

---

## 4. `0x10b2823b` — the COFF `Type` word, 38 bytes, previously unlabelled

```
Type = 0x20   iff  ([sym+0x30] == 3  and  [sym+0x31] ∈ {0x54,0x55,0x56})
              or   ([sym+0x30] == 4  and  [sym+0x37] & 0x400)
     = 0      otherwise
```

`0x20` is `IMAGE_SYM_DTYPE_FUNCTION << 4` — the only nonzero `Type` c2 writes
`[R]`. Three call sites: `0x10b28b4e`, `0x10b28cf1`, `0x10b28d44`. The function
had **no row anywhere in the record** before this page (`FUNCS.tsv` `cover =
none`).

---

## 5. ⛔ `[sym+0x4c]` is read two incompatible ways in this one function, and this page does not resolve it

* `0x10b28ae4` — `test BYTE PTR [esi+0x4c], 0x20`, on a **kind-4** symbol,
  gating the `/EXPORT:` `.drectve` entry. That is `C2_MAP.md` §3E's **emit
  bit**.
* `0x10b28b09` — `mov ecx, DWORD PTR [esi+0x4c]`, on a **kind-4** symbol,
  loading the word **whole** as the alias token — which is exactly what
  **`W-ALIAS-1`** adopted into `crates/c2-il/src/func/glalias.rs`: *"on a
  tag-0x10 record that word is a **symbol token**, not a flag word — which is
  the whole finding"*.

Both sites are in the same arm of the same function, 37 bytes apart. Under the
token reading, `token & 0x20` is meaningless. Two readings survive and this lane
**measured neither**:

1. `[sym+0x37] & 0x200` (dllexport) and `& 0x400000` (alias) never co-occur in
   practice, so the `/EXPORT:` test is unreachable for aliases — an
   **empirical** claim, and the workload can decide it;
2. c2 tests bit 5 of an alias token, which is a latent defect the port would
   have to reproduce.

**Registered as an open question, not as a finding.** `W-ALIAS-1`'s own
disclosure row is unaffected either way — it is about the tag-`0x10` *record*,
and this is about a *consumer*. The one-line probe that decides it: count
workload `.gl` symbols with both bits set.

---

## 6. Entries

`size` and the caller/callee counts are Ghidra's, from the flat export.

| addr | size | callers | callees | TU | what |
|---|---:|---:|---:|---|---|
| `0x10b28a9b` | 724 | 2 | 12 | `coff.c` gap | **the symbol emitter.** Kind dispatch, idempotence, `/EXPORT:`, storage class, section number, the weak-external pair `[R]` |
| `0x10b28aa3` | — | — | — | — | `[sym+0x32] & 1`, the once-only guard; set at `0x10b28ab3` **before** any emission, which is what terminates the recursion at `0x10b28cb9` `[R]` |
| `0x10b28ad8` | — | — | — | — | kind-4 `/EXPORT:` gate: `+0x37 & 0x200` **and** `+0x4c & 0x20` `[R]`. Literals `0x10b01b38` `"/EXPORT:"`, `0x10b01b44` `",DATA"` — **tier 1**, plain `strings` output |
| `0x10b28ae4` | — | — | — | — | `test [esi+0x4c],0x20` — the emit bit, re-read by the `.drectve` writer. **See §5** `[R]` |
| `0x10b28b02` | — | — | — | — | `test eax,0x400000` — **the ALIAS bit's consumer**, the site `W-ALIAS-1` implies and does not name `[R]` |
| `0x10b28b11` | — | — | — | — | `and eax,0x1e0` / `cmp eax,0x80`: a non-alias kind-4 is written only at storage kind 4, as an undefined external `[R]` |
| `0x10b28ba3` | — | — | — | — | `cmp cl,1` — the kind-1 (data) arm `[R]` |
| `0x10b28ba8` | — | — | — | — | `([sym+0x37]>>0x15)&7`; **1 and 3 suppress the symbol entirely** `[R]` |
| `0x10b28bc3` | — | — | — | — | kind-1 `/EXPORT:` gate → `"/EXPORT:<name>,DATA"` `[R]` |
| `0x10b28be6` | — | — | — | — | **`([sym+0x37]>>5)&0xF`, the storage-kind field — phase 6's decision site** `[R]` |
| `0x10b28c00` | — | — | — | — | the default arm: `StorageClass = 3` (STATIC) `[R]` |
| `0x10b28c7d` | — | — | — | — | storage-kind 2: `[sym+0x3f] != 0` → the weak path. **The second route** `[R]` |
| `0x10b28c88` | — | — | — | — | the weak-external join `[R]` |
| `0x10b28c8b` | — | — | — | — | `call 0x10b9860d`, resolve token → target `[R]` |
| `0x10b28c92` | — | — | — | — | `or [target+0x32],4` `[R]` |
| `0x10b28c96` | — | — | — | — | `test [target+0x37],0x200000`, the emit-marker guard `[R]` |
| `0x10b28ca3` | — | — | — | — | **`call 0x10b276e4`** — the site `W-ALIAS-2` names as *"named and NOT decoded"*. Decoded: a guarded emit-marker call **from the COFF symbol writer** `[R]` |
| `0x10b28ca8` | — | — | — | — | the kind-1→kind-4 hop through `[target+0x3c]` `[R]` |
| `0x10b28cb9` | — | — | — | — | **`call 0x10b28a9b`** — recurse on the target. The ordering rule `[R]`/`[I]` |
| `0x10b28cbe` | — | — | — | — | `cmp [target+0x24],0` — mint the default on demand `[R]` |
| `0x10b28cd2` | — | — | — | — | `push 2` — the default's `StorageClass` `[R]` |
| `0x10b28cea` | — | — | — | — | `push 2` — aux `Characteristics`. **`[O]`**: `alias-weak-not-search-library`, KNOWN ANSWER **0** over 878 TUs |
| `0x10b28cec` | — | — | — | — | `push [target+0x24]` — aux `TagIndex`. **`[O]`**: `alias-weak-default-disagree`, KNOWN ANSWER **0** |
| `0x10b28cfd` | — | — | — | — | `push 0x69` — `IMAGE_SYM_CLASS_WEAK_EXTERNAL`. **`[O]`** via `ObjImage::weak_externals` |
| `0x10b28d1d` | — | — | — | — | storage-kind 1: `Value = 4` when `[sym+0x20]&0x80000`, else `[sym+0x18]`; section via `FUN_10b287b8` `[R]` |
| `0x10b2823b` | 38 | 2 | 0 | `coff.c` gap | **the COFF `Type` word**; §4 `[R]` |
| `0x10b2a757` | 164 | 4 | 4 | `coffemit.c` anchor | append a symbol, no aux `[R]` |
| `0x10b2a8da` | 92 | 2 | 3 | `coffemit.c` anchor | append a symbol **and hand back its aux slot** `[R]` |
| `0x10b2af4f` | 43 | 1 | 1 | `coffemit.c` anchor | the 18-byte weak aux: `{TagIndex, Characteristics}` `[R]` |
| `0x10c2e218` | 4 | — | — | data | the UNDEFINED `SectionNumber` constant; `data.tsv` gives its value as `0x0`; five readers, all in `0x10b28a9b` and `0x10b2a757` `[R]` |
