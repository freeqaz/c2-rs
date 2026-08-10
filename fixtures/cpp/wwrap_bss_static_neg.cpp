// w-wordwrap2 — the INTERNAL-LINKAGE object, which is OUT of the shared-`.bss`
// class and must be REFUSED.
//
// Rule S1' has three slots for a non-COMDAT `.bss` and the LINKAGE picks which:
//
//   A  a STATIC object first reached from a `.data` initializer   -> before .XBLD$W(C2)
//   B  an EAGER EXTERNAL object                                   -> BETWEEN the watermarks
//   C  a STATIC first reached from a FUNCTION BODY                -> AFTER the code groups
//
// This TU is slot `C`, and `work/w-wordwrap2/probe/p5.obj` is c2's answer:
//
//   .drectve  .debug$S  .XBLD$W  .XBLD$W  .text(S1)  .bss  .text(R)
//
// — the section sits BETWEEN the two `.text` COMDATs, immediately after its own
// first referrer, and its symbol group sits there too. Board **#1179** measures
// that slot at **109 of 871** workload objs, so it is the common case and not a
// corner; nothing places it, and emitting slot `B` here would be a wrong section
// ORDER on an obj whose section COUNT is right — board #259's direction.
//
// **Both bodies are deliberately IN CLASS.** The first draft of this file paired
// the store with `unsigned int Get() { return s; }`, and that body is out of
// class, so the TU never reached the writer and the cell graded NONE of the
// clause it exists for — `vocab-gap` at `body-out-of-class`, an over-fenced cell
// wearing a green refusal. Two `global_store_leaf` bodies put the TU in front of
// the reader clause that actually refuses it.
//
// 0 of the functions in this file may be emitted into an obj.

static unsigned int s_a;
static unsigned int s_b;

void SetA(unsigned int x) { s_a = x; }
void SetB(unsigned int x) { s_b = x; }
