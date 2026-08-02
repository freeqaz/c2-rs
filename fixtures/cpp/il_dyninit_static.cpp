// **The dynamic-initializer thunk** — board #158, the Phase-7 entry point that
// needs no emit-set model.
//
// Two lines, and they reproduce `src/system/synth/tomcrypt/TomCryptLicense.cpp`
// and `src/system/zlib/ZlibLicense.cpp` exactly: those two workload TUs have
// **byte-identical** `.ex` files (2,839 B each) whose only function is this
// shape, and c2 emits one `.text` COMDAT for it.
//
// c2's own listing (`c2rs listing`), six instructions and a tail branch:
//
//   ??__EsL@@YAXXZ PROC NEAR          ; `dynamic initializer for 'sL'', COMDAT
//     lis   r11,??_C@_03FIKCJHKP@abc?$AA@
//     lis   r10,sL
//     addi  r4,r11,??_C@_03FIKCJHKP@abc?$AA@
//     addi  r3,r10,sL
//     li    r5,0
//     b     ??0L@@QAA@PBDH@Z
//   .bss     sL DB 01H DUP (?)
//   .CRT$XCU sL$initializer$ DD ??__EsL@@YAXXZ
//
// **Why it is the entry point.** §10.11: the symbol is not synthesized out of
// nothing. `.ex` carries a real `4F 1F` function start (with `OPT_WORD_OX` here,
// `OPT_WORD_O1` on the workload), and `.gl` carries one framed record whose
// body-start offset **is** that start, binding `??__EsL@@YAXXZ` at distance 19.
// The binding already works today, with no model. What is missing is a body
// decode and an obj shape.
//
// **Where the decode dies, at the byte.** After `46` (`ExToken::Formals`) a
// source function carries `4C 4F 11` (`ExToken::Lo`) and this one carries
// `4C 53`:
//
//   w_add          … 4f 01 03 53 53 26 0a 0a 46 2d 09 0a  4c 4f 11  53 b9 09 0a …
//   ??__EsL        … 4f 01 02 53 53 26 ed 09    46        4c 53     26 e6 09 …
//
// `try_ex_token` (`codec.rs:1094`) knows `4C 4F 11` and `4C 4B`
// (`VoidCallEnd`) and nothing else beginning `4C`, so `4C 53` returns `None`.
//
// **What that byte is NOT.** Not "compiler-generated": a virtual destructor's
// `??_G` deleting-destructor thunk is compiler-generated too and carries
// `4C 4F 11` like any source function (measured on a three-body probe, three
// starts and three `4C 4F 11`). Not "no locals": `w_add` has none either. The
// discriminator is **unidentified**, and this fixture exists so it can be
// identified against a two-line source instead of a 2,839-byte capture.
//
// **The obj shape is the other half, and it is larger than the decode.** This
// TU's obj carries `.rdata` (the string COMDAT), `.bss` (the object), and
// `.CRT$XCU` (the initializer pointer) beside `.text` — the port emits a fixed
// four-section shell. `NotImplemented` today, and expected to stay that way
// until both halves land.

struct L { L(const char* s, int r); };
static L sL("abc", 0);
