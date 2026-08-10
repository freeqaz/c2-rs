// w-wordwrap2 — `__declspec(thread)`, which lands in `.tls$` and must be
// REFUSED.
//
// The record is the reason this clause exists rather than a hypothetical:
// `GlDataObject::flags`' own doc shows a thread-local's `.gl` data record
// byte-identical to an ordinary uninitialized object's in EVERY field the reader
// had before lane `w-sect` found the bit —
//
//   char b1;                     ... 82 01 00 02 01 01 00 . 00 .
//   __declspec(thread) int t1;   ... 86 01 00 02 01 04 00 . 10 .
//
// — so a reader that stops at the attribute reports a `.tls$` object as a `.bss`
// one and emits the wrong SECTION NAME the moment a `.bss` writer has a caller.
// This lane gave that writer its first function-bearing caller, so the clause is
// graded rather than left as a comment.
//
// **The thread-local is NOT stored to from an in-class body, on purpose.** A
// `t = x;` leaf is itself out of class, so a fixture that wrote one would refuse
// at `shape-token-*` and grade none of this clause. Here the only function is an
// ordinary `global_store_leaf` into the ordinary `.bss` object, so the TU
// reaches the reader with a shared `.bss` to place AND a `.tls$` it cannot, and
// the refusal is this clause's.

__declspec(thread) unsigned int t_option;
unsigned int g_option;

void SetG(unsigned int x) { g_option = x; }
