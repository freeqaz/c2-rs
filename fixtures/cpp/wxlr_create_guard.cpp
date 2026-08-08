// W-XLR — the two-stage create/attach guard whose four failure paths converge
// on one returned status. The class `src/xdk/xlrc/xlrcimpl.cpp`'s
// `CXLrcImpl_CreateClientWithTransport` is the workload instance of, and the
// FIRST function this port emits whose frame goes through `__savegprlr_N`.
//
// Compile at the workload's own profile (`/O1 /Oi /EHsc /GR`). At `/Ox` this
// file is deliberately 0/1 in class: the mode gate lives in the PARSER
// (board #1638) and `scripts/mode_lane.sh` compiles every fixture at both.
//
// What this fixture is FOR, beyond the class itself — four things no earlier
// fixture in the tree can grade:
//
//  * **A prologue that CALLS.** Six live callee-saved GPRs put this body in
//    Class C, so the prologue is `mflr r12` / `bl __savegprlr_26` / `stwu` and
//    the epilogue is `addi r1,r1,144` / `b __restgprlr_26` with **no `blr`**.
//    Every framed obj the port emitted before this one has three prologue words
//    and one REL24 per IL-named callee; this one has four relocations for two.
//  * **Two undefined externals whose symbol records sit AFTER the `$T` label.**
//    `docs/CODEGEN_FRAMED_CALLS.md` §2.3a's group, and the first cell in the
//    tree that can tell it from the callee region.
//  * **An ADDRESS-TAKEN four-byte local.** `size` is written to `80(r1)` once
//    and re-read three times, because each callee may have written it. Every
//    local the port had admitted before this one is register-resident and gets
//    folded into the expression that reads it.
//  * **A `lis` HOISTED ABOVE the branch that chooses which `ori` follows it.**
//    `E_OUTOFMEMORY` and `E_INVALID_OPERATION` share a high half, so c2 emits
//    one `lis` before the `cmplwi` and one `ori` in each arm. A per-statement
//    lowering writes five words where c2 writes four and every displacement
//    after the hoist is wrong.
//
// THREE things that had to be written exactly right, each read off the
// workload's own IL rather than guessed, and each of which type-checks and
// means the same thing to a C++ programmer when written the other way:
//
//  1. **The initial value and the compared bound are two literals**, not one
//     named constant used twice. c2 emits `li r11,4` and `cmplwi cr6,r11,4` and
//     the recognizer carries them as two fields; a shared `const` reads the
//     same and is the same body, but the fixture should exercise both fields.
//  2. **`size` must be `unsigned`, and the compare must therefore be
//     `cmplwi`.** Written `int size`, the middle guard is a *signed* `cmpwi`
//     and the class has no word for it.
//  3. **The three success stores are through the formals' POINTER VALUES.**
//     `*outSize = size` — not `outSize[0] = size` with an index, which is the
//     same program and a different token stream.

// Opaque handle types: only their pointers ever appear, so no definition is
// needed and none is given — a definition would add a `.debug$S` record and
// change nothing else.
class CGuardClient;
class CGuardTransport;

// A four-byte scalar first formal. The workload's is an `enum`; anything the
// front end passes in one GPR and never touches produces the same `mr r30,r3`.
enum GuardTransportId { GUARD_TRANSPORT_DEFAULT = 0 };

extern "C" CGuardClient *wxlr_create_client(unsigned int *size);
extern "C" CGuardTransport *wxlr_create_transport(CGuardClient *client,
                                                  GuardTransportId id,
                                                  unsigned int size);

// The two arm constants MUST share a high half — that is what makes the `lis`
// hoistable, and the recognizer refuses the body outright when they do not.
// These are the workload's own values.
#define WXLR_E_OUTOFMEMORY       0x8007000EL
#define WXLR_E_INVALID_OPERATION 0x800710DDL
#define WXLR_E_FAIL              0x80004005L

extern "C" long wxlr_create_client_with_transport(GuardTransportId id,
                                                  unsigned int *outSize,
                                                  CGuardClient **outClient,
                                                  CGuardTransport **outTransport)
{
    unsigned int size = 4;
    long result = 0;

    CGuardClient *client = wxlr_create_client(&size);
    if (client == 0) {
        if (size < 4) {
            result = WXLR_E_OUTOFMEMORY;
        } else {
            result = WXLR_E_INVALID_OPERATION;
        }
    } else {
        CGuardTransport *transport = wxlr_create_transport(client, id, size);
        if (transport == 0) {
            result = WXLR_E_FAIL;
        } else {
            *outSize = size;
            *outClient = client;
            *outTransport = transport;
        }
    }

    return result;
}
