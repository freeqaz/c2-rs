# WB-F `wb-eh` — PREREG

> **PROVENANCE — DISASSEMBLY-DERIVED.** See
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 for the exact bytes
> (`sha256 c80981…6258`, verified in this lane before this file was written) and
> [`DISCLOSURE.md`](DISCLOSURE.md) for what adoption costs. Nothing in this lane
> is adopted into `crates/`.

Registered **before the first grep of `~/ghidra-projects/export/c2/` by this
lane** and **before the first `cl.exe` this lane runs**, per board #770's
standing rule (streak ~10 optimistic / 2 pessimistic / 1 hit). Every item carries
its **registered direction**, so a miss is scoreable rather than narratable.

Lane `wb-eh` / branch `wt-wb-eh`, branched at master **`9ed20248`**.
Board range **#1860–#1879**. Scored in
[`WB_EH_FINDINGS.md`](WB_EH_FINDINGS.md) §PREREG.

## What this lane already knows before it starts (declared, so nothing below is
## re-sold as a discovery)

These are the prior-art facts this lane read first. **A confirmation of one of
them is a confirmation, not a finding**, and is scored as such:

* `docs/EH_RECORDS.md` and `docs/EH_CRITICAL_PATH.md` — black-box: the 8-byte
  `{__CxxFrameHandler, __ehfuncinfo$…}` `.text` prefix, the function symbol at
  `Value = 8`, **two `.pdata` COMDATs** per EH function (funclet's emitted
  first), the unwind word `flags | (len_words << 8) | prolog_words`, the EH
  record set in **plain `.rdata`, `Selection = 5` associative**, the type
  descriptor `??_R0` in `.data` `Selection = 2`, `.xdata$x` = throw-side
  (`_TI`/`_CTA`/`_CT`).
* w-eh5's retraction: **`.rdata$r` is RTTI, not EH.** Factor C's EH contribution
  is **zero**. Not re-tread here.
* w-one / board **#1354**, **#1469**: `expr-convert-no-value-0x2C` is the port's
  own class-stack running out, its "cannot be reached by a well-formed stream"
  comment is refuted **4,973 times over 829 of 878 TUs**, and the terminal
  `Main.cpp` reaches once it is lifted is **`0x5C`**, filed as the "EH LIVE-STATE
  marker", 309,804 bodies.
* `WB_READER_FINDINGS.md` §3: the 192-entry operand-class table `DAT_10b25e48`
  at `0x10b3d626`. `0x2C` is class **`05`** = `TYPE` + **one raw `GetByte`**;
  `0x5C` is class **`13`** = `TYPE` + `i32c`. §5.4 leaves `0x2C`'s raw byte
  **unconfirmed** ("designed and not run") with two witnesses whose payload is
  `0x00`, where a raw byte and a varint agree.

---

## P1 — where the machinery is

| # | prediction | registered direction if wrong |
|---|---|---|
| P1.1 | The EH symbol prefixes are **minted by c2**, not carried in the IL: at least **five** of `__ehfuncinfo$` `__unwindtable$` `__tryblocktable$` `__catchsym$` `__catch$` `__unwind$` `__ehhandler$` appear as literal strings in `c2.dll` with at least one xref each | optimistic |
| P1.2 | `__CxxFrameHandler` appears as a literal string in `c2.dll` (c2 mints the external, the IL never names it) | optimistic |
| P1.3 | There is a **single name-minting helper** (prefix + mangled function name, sprintf/strcat-shaped) shared by ≥ 3 of the `$`-prefixes | optimistic |
| P1.4 | The `.pdata` writer is **one** function that both EH and non-EH functions reach, and the EH-ness enters as a flag rather than as a separate emitter | optimistic |
| P1.5 | I will find the constant **`0x80000000`** (or a `bts`/`or` of bit 31) in the `.pdata` word computation, distinct from the `<< 8` length field | optimistic |

## P2 — the table formats

| # | prediction |
|---|---|
| P2.1 | The `.pdata` record is exactly **two u32**: `BeginAddress` (relocated `ADDR32`, addend = offset from the function symbol whose `Value` is 8) and the packed unwind word. **No third word, no separate `.xdata` for the main body.** Registered against the rival "PPC XDK uses a pointer-to-xdata form" |
| P2.2 | The unwind word's layout is `bit31 | (len_words << 8) | prolog_words`, i.e. `flags` occupy bits 31..? and prolog_words bits 7..0, confirming `EH_CRITICAL_PATH.md` §2 from the binary. **A confirmation, not a finding** |
| P2.3 | **Bit 31 = "this region is preceded by the 8-byte handler prefix"** (EH_CRITICAL_PATH's four-record reading). Rivals registered *now*: (R-a) bit 31 = "function has a language handler / EH state at all"; (R-b) bit 31 = "region has a prologue"; (R-c) bit 31 = "region saves LR". I register **the prefix reading as the one I expect to survive**, and I register that **four records is a thin basis** — if the binary says (R-a) or (R-b) I will say so |
| P2.4 | `__ehfuncinfo$` is a fixed-layout record whose **first u32 is a magic** (the MSVC `EHmagicNumber`, `0x19930520`/`0x19930521`/`0x19930522` family). I predict **`0x19930522`** appears as an immediate in `c2.dll`. Registered direction: optimistic — if no magic appears the record is built from a template blob |
| P2.5 | The funclet symbols `__catch$NNNN` / `__unwind$NNNN` take their number from **the same label counter** `docs/LABEL_COUNTER.md` describes (`$M`/`$T`), not from a private EH counter. Registered direction: optimistic |
| P2.6 | All EH relocations are **`ADDR32` (IMAGE_REL_PPC_ADDR32 = 0x0006)** — no `ADDR32NB`, no `SECREL`. Registered direction: optimistic |

## P3 — Main.cpp's stuck rung (the deliverable that matters)

| # | prediction | direction |
|---|---|---|
| P3.1 | The stuck rung is **not an EH construct at all**. `expr-convert-no-value-0x2C` is a **port-side artifact**: c2's reader for `0x2C` reads `TYPE` + one raw byte and **never consults a stack of source classes**, so the port's "no value on the class stack" is a fact about `cstack`, not about the stream. I register this as the NAME | pessimistic (I am registering that the lane's headline is a *deflation* of the rung, which is the direction that makes the lane look small) |
| P3.2 | The construct `0x2C` denotes is a **conversion/cast whose source is the preceding expression node**, and the raw byte is a **conversion-kind selector** (not a symbol id, not a length). Registered rivals: (R-d) the byte is a `varU` low byte after all (i.e. the port is right); (R-e) the byte is a rounding/precision mode | — |
| P3.3 | Once `0x2C` is lifted, `Main.cpp`'s next first-blocker is **`0x5C`**, the EH state marker — a *confirmation* of #1354, and it is what makes this an EH row at all. I additionally predict `0x5C`'s c2-side meaning is **"set the current EH state to N"**, i.e. it writes the ip-to-state (`$T…`) array, with its `TYPE` operand being the type of the object whose lifetime the state covers | optimistic on the `$T` link |
| P3.4 | The honest price of `Main.cpp` to conversion at this tip is **≥ 12 named refusals** and this lane will deliver a **priced decline**, not a route to a conversion. Registered direction: **pessimistic** — if the true price is under 12 I have over-priced it | pessimistic |
| P3.5 | The port's `0x2C` width rule and c2's **disagree only above payload `0x80`**, and I predict the workload contains **at least one** `0x2C` site whose payload byte is ≥ `0x80` (which would make WB-A §5.4's latent desync live). Registered direction: optimistic — if zero sites exist the disagreement stays latent and unprovable black-box | optimistic |

## P4 — the black-box confirmations this lane will run

Outcome vocabulary: `IDENT` (obj byte-identical with `TimeDateStamp` zeroed),
`DIFF` (an obj, bytes differ), `NOOBJ` (c2 refuses). **WB-A §5.3(6) is already
known and is registered here: a desynchronised operand stream does NOT make c2
ICE**, so no cell below predicts `NOOBJ` on a width argument.

| cell | what | predicted |
|---|---|---|
| E1 | minimal `try { g(a); } catch(int e) {…}` at the workload flags | c2 emits: `.text` with the 8-byte prefix, function symbol `Value = 8`, **2** `.pdata`, EH `.rdata` (Sel 5) of **96** bytes, `??_R0H@8` in `.data`. Port verdict: **refuses** (census reports a blocker; **no** `mismatch`) |
| E2 | the same function with the `catch` removed but a destructible local kept (unwind funclet only) | 2 `.pdata`, and the **unwind funclet's record has bit 31 CLEAR** while the main record has it set (EH_CRITICAL_PATH's four-record rule, re-run on a fifth and sixth record) |
| E3 | **the discriminating cell for P2.3**: a function with a `catch` **and** a destructible local (two funclets, one with a prefix and one without) | catch funclet bit31 **SET**, unwind funclet bit31 **CLEAR**, main **SET**. Rival (R-a) "has a language handler" predicts **all three SET**; rival (R-b) "has a prologue" predicts the unwind funclet SET too (it has a real prologue word). **This cell separates all three** |
| E4 | a **leaf, frameless, no-EH** function in the same TU as an EH function | its `.pdata` bit 31 **CLEAR** — the null control that keeps E3 from being a constant |
| E5 | `cl /FAsc` on E3's source | the listing names the funclet labels and the `.rdata` record layout by symbol, and the funclet `.pdata` is listed **before** the body's |

**Asserted minimum of discriminating cells: 2.** If E3 does not separate at least
two of the three rivals, P2.3 is reported as **not established** rather than
"consistent with".

## D — decline clauses, registered before the first probe

| # | clause |
|---|---|
| **D1** | If the EH machinery is reachable only through a code path the objs do not take (method doc §7's `.bss` failure mode), the reading is filed **navigation-only** and no DISCLOSURE row is drafted for it |
| **D2** | If E3 refutes the bit-31 prefix rule, `EH_CRITICAL_PATH.md` §2's rule is **retracted in this lane's findings** and the surviving rival is named. It is not hedged |
| **D3** | If the `0x2C` reading turns out to matter for `Main.cpp` in a way P3.1 denies — i.e. c2 *does* consult prior nodes for the convert's source — P3.1 is a **MISS** and is reported as the lane's headline error |
| **D4** | This lane adopts **nothing** into `crates/`. Any scratch instrument patch is reverted before the gate and never committed |
| **D5** | If the priced route to `Main.cpp` comes out **cheaper** than P3.4's ≥ 12, the lane says so and hands the code lane a route rather than a decline — a pessimistic miss is still a miss |
| **D6** | No number from `docs/EH_RECORDS.md` or `docs/EH_CRITICAL_PATH.md` is re-reported as this lane's measurement. Anything re-measured is labelled **re-measured** with both values |

## Not taken, declared now

* The **throw side** (`.xdata$x`, `_TI`/`_CTA`/`_CT`) is out of scope: 67 objs,
  entirely STLport, and `Main.cpp` does not need it.
* **RTTI** (`.rdata$r`, `??_R1`–`??_R4`) is out of scope — w-eh5 settled it.
* Shipping anything into `crates/` is WB-F's explicit non-goal.
