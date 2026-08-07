// **W-ORDER3 / board #174 — the control that kills two rivals at once, and the
// reason slot `A` is about LINKAGE and not about the relocation. Byte-exact.**
//
// `wa16_bss_static_reloc.cpp` is this file with one `static` added, and it puts
// `.bss` **before** both `.XBLD$W` watermarks. Board #1152 filed the open
// question precisely: *"nothing here separates 'internal linkage moves it' from
// 'a `.data` relocation into `.bss` moves it', because every surviving cell has
// both"*. This cell has the relocation and **not** the internal linkage:
//
//     .drectve .debug$S .XBLD$W(C2) .bss .XBLD$W(C1) .data
//
// `.bss` stays in Rule S1's middle slot. So both of these are refuted:
//
//   * **the reloc rival** — "a `.data` relocation targeting an object in the
//     `.bss` moves the section". This obj has exactly that relocation.
//   * **the address-taken rival** — "the `.bss` object having its address taken
//     moves the section". `&g` is taken here too.
//
// The complementary half is `worder3_bss_slot_after_text.cpp`, where a static is
// kept alive with **no** relocation anywhere and the section moves to a third
// slot entirely. Between the two, the trigger is linkage plus the identity of
// the first contributor, and neither rival survives.
//
// The workload agrees out of sample: of 247 real non-COMDAT `.bss` sections,
// every one of the 138 in this slot contains an external symbol, and **0 of 25**
// purely-static sections are here.

struct A{int a;};
A g;
A* p = &g;
