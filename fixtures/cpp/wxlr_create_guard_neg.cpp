// W-XLR — the NEGATIVE cells. Ten near-misses of `wxlr_create_guard.cpp`'s
// class, each of which must be **0/1 in class** at the workload's own profile
// (`/O1 /Oi /EHsc /GR`), and each for its OWN reason.
//
// Read per cell with an applied-and-reverted probe patch
// (`work/w-xlr/decline_probe.md`), never off `c2rs census`'s fall-through
// blocker: board **#1704** — the census reports one key for the whole file, so
// a `_neg` fixture that is only *counted* proves that ten functions declined and
// nothing about whether they declined for ten different reasons. That method is
// w-cfgclass §6.2's and this is the sixth lane to pay it.
//
// The clause each cell is FOR is named beside it. Where the walk stops one
// production *before* the clause the comment names, the comment says so rather
// than being reworded — an honest record of where a cell's evidence actually
// lands is worth more than a tidy table.

class CGuardClient;
class CGuardTransport;
enum GuardTransportId { GUARD_TRANSPORT_DEFAULT = 0 };

extern "C" CGuardClient *wxlr_create_client(unsigned int *size);
extern "C" CGuardTransport *wxlr_create_transport(CGuardClient *client,
                                                  GuardTransportId id,
                                                  unsigned int size);
extern "C" CGuardTransport *wxlr_create_transport2(CGuardClient *client,
                                                   unsigned int size);
extern "C" CGuardClient *wxlr_create_client2(unsigned int *a, unsigned int *b);
extern "C" void wxlr_note(long v);

#define WXLR_E_OUTOFMEMORY       0x8007000EL
#define WXLR_E_INVALID_OPERATION 0x800710DDL
#define WXLR_E_FAIL              0x80004005L

// ---------------------------------------------------------------------------
// n1 — the stack object is SIGNED.
//
// `xlrc-stack-object-is-not-an-unsigned-word`. The one cell in this file that
// guards a **live wrong emit** rather than a shape the emitter has no words
// for: the relational opcode is sign-agnostic, so `int size` and
// `unsigned size` produce the identical `22` byte and differ only in the
// operand TYPE — and c2 emits `cmpwi cr6,r11,4` here where the emitter has an
// unconditional `cmplwi`. One wrong word, in an obj that links.
// ---------------------------------------------------------------------------
extern "C" long wxlr_n1(GuardTransportId id, unsigned int *outSize,
                        CGuardClient **outClient, CGuardTransport **outTransport)
{
    int size = 4;
    long result = 0;
    CGuardClient *client = wxlr_create_client((unsigned int *)&size);
    if (client == 0) {
        if (size < 4) {
            result = WXLR_E_OUTOFMEMORY;
        } else {
            result = WXLR_E_INVALID_OPERATION;
        }
    } else {
        CGuardTransport *t = wxlr_create_transport(client, id, (unsigned int)size);
        if (t == 0) {
            result = WXLR_E_FAIL;
        } else {
            *outSize = (unsigned int)size;
            *outClient = client;
            *outTransport = t;
        }
    }
    return result;
}

// ---------------------------------------------------------------------------
// n2 — TWO address-taken locals.
//
// `xlrc-not-exactly-one-address-taken-local`. The frame reserves four bytes for
// exactly one stack object; a second one is four more bytes, a different `stwu`
// immediate and a different displacement on every access. `.sy`'s
// `addr_locals` is the positive channel that says so, and the clause requires
// it to be a one-element list, not merely to CONTAIN the token.
// ---------------------------------------------------------------------------
extern "C" long wxlr_n2(GuardTransportId id, unsigned int *outSize,
                        CGuardClient **outClient, CGuardTransport **outTransport)
{
    unsigned int size = 4;
    unsigned int other = 7;
    long result = 0;
    CGuardClient *client = wxlr_create_client2(&size, &other);
    if (client == 0) {
        if (size < 4) {
            result = WXLR_E_OUTOFMEMORY;
        } else {
            result = WXLR_E_INVALID_OPERATION;
        }
    } else {
        CGuardTransport *t = wxlr_create_transport(client, id, size);
        if (t == 0) {
            result = WXLR_E_FAIL;
        } else {
            *outSize = size;
            *outClient = client;
            *outTransport = t;
        }
    }
    return result;
}

// ---------------------------------------------------------------------------
// n3 — the two arm constants do NOT share a high half.
//
// `xlrc-arm-constants-do-not-share-a-lis`. c2 cannot hoist one `lis` above the
// branch, so it emits a `lis`+`ori` pair in each arm: FIVE words where this
// class writes four, and every displacement after the hoist is wrong.
// ---------------------------------------------------------------------------
extern "C" long wxlr_n3(GuardTransportId id, unsigned int *outSize,
                        CGuardClient **outClient, CGuardTransport **outTransport)
{
    unsigned int size = 4;
    long result = 0;
    CGuardClient *client = wxlr_create_client(&size);
    if (client == 0) {
        if (size < 4) {
            result = WXLR_E_OUTOFMEMORY;
        } else {
            result = 0x800410DDL;
        }
    } else {
        CGuardTransport *t = wxlr_create_transport(client, id, size);
        if (t == 0) {
            result = WXLR_E_FAIL;
        } else {
            *outSize = size;
            *outClient = client;
            *outTransport = t;
        }
    }
    return result;
}

// ---------------------------------------------------------------------------
// n4 — a status constant whose LOW half is zero.
//
// `xlrc-status-constant-is-not-lis-plus-ori`. c2 emits a single `lis` and no
// `ori` for it, which is a shorter body this class has no witness of. Refused
// rather than guessed — the mirror of the zero-HIGH-half case, where c2 emits a
// single `li`.
// ---------------------------------------------------------------------------
extern "C" long wxlr_n4(GuardTransportId id, unsigned int *outSize,
                        CGuardClient **outClient, CGuardTransport **outTransport)
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
        CGuardTransport *t = wxlr_create_transport(client, id, size);
        if (t == 0) {
            result = 0x80000000L;
        } else {
            *outSize = size;
            *outClient = client;
            *outTransport = t;
        }
    }
    return result;
}

// ---------------------------------------------------------------------------
// n5 — the status accumulator is initialized to something other than zero.
//
// `xlrc-result-not-initialized-to-zero`. The emitter's `li r26,0` is not a
// field; making it one would be a fifth immediate this class has never been
// graded on, and the reader refuses rather than the emitter inventing it.
// ---------------------------------------------------------------------------
extern "C" long wxlr_n5(GuardTransportId id, unsigned int *outSize,
                        CGuardClient **outClient, CGuardTransport **outTransport)
{
    unsigned int size = 4;
    long result = 1;
    CGuardClient *client = wxlr_create_client(&size);
    if (client == 0) {
        if (size < 4) {
            result = WXLR_E_OUTOFMEMORY;
        } else {
            result = WXLR_E_INVALID_OPERATION;
        }
    } else {
        CGuardTransport *t = wxlr_create_transport(client, id, size);
        if (t == 0) {
            result = WXLR_E_FAIL;
        } else {
            *outSize = size;
            *outClient = client;
            *outTransport = t;
        }
    }
    return result;
}

// ---------------------------------------------------------------------------
// n6 — THREE formals.
//
// `xlrc-not-four-formals-free-fn`. Four formals park in r30–r27 by four `mr`
// words the emitter does not vary; three is a different prologue, a different
// `saved_gprs` and therefore a different frame size AND a different helper
// width (`__savegprlr_27`, not `_26`).
// ---------------------------------------------------------------------------
extern "C" long wxlr_n6(GuardTransportId id, unsigned int *outSize,
                        CGuardClient **outClient)
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
        CGuardTransport *t = wxlr_create_transport(client, id, size);
        if (t == 0) {
            result = WXLR_E_FAIL;
        } else {
            *outSize = size;
            *outClient = client;
        }
    }
    return result;
}

// ---------------------------------------------------------------------------
// n7 — the outer test is INVERTED.
//
// `xlrc-outer-test-relation`. `!= 0` is opcode `20` where the class requires
// `1F`, and the two arms swap: the record-form `mr.`'s branch sense, the block
// order and all three intra-section displacements change together.
// ---------------------------------------------------------------------------
extern "C" long wxlr_n7(GuardTransportId id, unsigned int *outSize,
                        CGuardClient **outClient, CGuardTransport **outTransport)
{
    unsigned int size = 4;
    long result = 0;
    CGuardClient *client = wxlr_create_client(&size);
    if (client != 0) {
        CGuardTransport *t = wxlr_create_transport(client, id, size);
        if (t == 0) {
            result = WXLR_E_FAIL;
        } else {
            *outSize = size;
            *outClient = client;
            *outTransport = t;
        }
    } else {
        if (size < 4) {
            result = WXLR_E_OUTOFMEMORY;
        } else {
            result = WXLR_E_INVALID_OPERATION;
        }
    }
    return result;
}

// ---------------------------------------------------------------------------
// n8 — an extra statement in the success arm.
//
// `xlrc-ok-close-8`. The three stores are the whole arm; a fourth statement is
// a fourth block-plan slot and at minimum one more `bl` with its own
// relocation. The clause that catches it is the scope close, not a statement
// count — which is the honest place for it: the walk arrives at `54 08`
// expecting the arm to be over and finds a `4F 01` line marker and a `26`.
// ---------------------------------------------------------------------------
extern "C" long wxlr_n8(GuardTransportId id, unsigned int *outSize,
                        CGuardClient **outClient, CGuardTransport **outTransport)
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
        CGuardTransport *t = wxlr_create_transport(client, id, size);
        if (t == 0) {
            result = WXLR_E_FAIL;
        } else {
            *outSize = size;
            *outClient = client;
            *outTransport = t;
            wxlr_note(result);
        }
    }
    return result;
}

// ---------------------------------------------------------------------------
// n9 — the middle guard is `<=`, not `<`.
//
// `xlrc-inner-test-relation`. Opcode `21` where the class requires `22`, and
// c2 lowers `<=` on an unsigned operand differently from `<` — this class has
// one `cmplwi` + one `bf cr6.LT` and no word for the other sense.
// ---------------------------------------------------------------------------
extern "C" long wxlr_n9(GuardTransportId id, unsigned int *outSize,
                        CGuardClient **outClient, CGuardTransport **outTransport)
{
    unsigned int size = 4;
    long result = 0;
    CGuardClient *client = wxlr_create_client(&size);
    if (client == 0) {
        if (size <= 4) {
            result = WXLR_E_OUTOFMEMORY;
        } else {
            result = WXLR_E_INVALID_OPERATION;
        }
    } else {
        CGuardTransport *t = wxlr_create_transport(client, id, size);
        if (t == 0) {
            result = WXLR_E_FAIL;
        } else {
            *outSize = size;
            *outClient = client;
            *outTransport = t;
        }
    }
    return result;
}

// ---------------------------------------------------------------------------
// n10 — the attach call takes TWO arguments, not three.
//
// `xlrc-attach-arg2-is-not-the-first-formal`. The three-argument setup is three
// pinned words (`mr r4,r30` / `lwz r5,80(r1)` / `mr r3,r31`); with two
// arguments the first formal is never read, `saved_gprs` drops to five and the
// helper width changes with it.
// ---------------------------------------------------------------------------
extern "C" long wxlr_n10(GuardTransportId id, unsigned int *outSize,
                         CGuardClient **outClient, CGuardTransport **outTransport)
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
        CGuardTransport *t = wxlr_create_transport2(client, size);
        if (t == 0) {
            result = WXLR_E_FAIL;
        } else {
            *outSize = size;
            *outClient = client;
            *outTransport = t;
        }
    }
    (void)id;
    return result;
}
