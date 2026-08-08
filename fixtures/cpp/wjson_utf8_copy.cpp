// W-JSON — the UTF-16 → UTF-8 copy loop. The class
// `src/xdk/xjson/jsonwriter.cpp`'s `JsonWriter::GetBuffer` is the workload
// instance of, and the FIRST body this port emits that contains a BACK EDGE.
//
// Compile at the workload's own profile (`/O1 /Oi /EHsc /GR`). At `/Ox` this
// file is deliberately 0/1 in class: the mode gate lives in the PARSER
// (board #1638) and `scripts/mode_lane.sh` compiles every fixture at both.
//
// What this fixture is FOR, beyond the class itself — three things no earlier
// fixture in the tree can grade:
//
//  * **A FRAMELESS `__savegprlr_N` frame.** Four live callee-saved GPRs put
//    this body in Class C, and it makes no call, has no addressed local and
//    needs no outgoing-parameter area — so c2 allocates **no frame at all**.
//    The prologue is TWO words (`mflr r12` / `bl __savegprlr_28`, no `stwu`)
//    and the epilogue is ONE (`b __restgprlr_28`, no `addi r1,r1,F`, no `blr`).
//    W-XLR's Class C prologue is three words and its epilogue two; this is a
//    different shape and it is refused by its own predicate.
//  * **A BACK EDGE.** Every framed body the port emitted before this one is
//    acyclic. The `do`/`while`'s test is the function's only backward branch
//    and its displacement is the only negative one in the obj.
//  * **A LABEL LEAD OF FOUR.** `docs/LABEL_COUNTER.md` §1.1's surcharge table
//    charges this function `+2` — a first-introduced `__savegprlr_28` pair and
//    nothing else — and the reference obj forces **4**. Both counterfactuals
//    were built and scanned red. This fixture is the second witness of that
//    number, in a TU whose `.gl` seed is not the workload's.
//
// THREE things that had to be written exactly right, each read off the
// workload's own IL rather than guessed, and each of which type-checks and
// means something slightly different when written the other way:
//
//  1. **The three-byte arm writes through `*(pBuffer + 1)`, not `*pBuffer`.**
//     The source bumps `pBuffer` and then stores one element PAST it, twice, so
//     the emitted stores land at element 0, element 2 and element 3 — and the
//     pointer chain c2 builds is `r10 = r4 + 2` then `r11 = r10 + 2`, not
//     `r11 = r4 + 4`. Written the obvious way it is a different body.
//  2. **`hr` is `unsigned long` and the return type is `long`.** The function
//     returns `hr` through one `2C` conversion; making `hr` a `long` deletes
//     that conversion and merges two TYPE slots the recognizer requires
//     distinct.
//  3. **`maxSize` is a local read once, not `*pSize` read three times.** The
//     two wide arms compare against `maxSize`; re-reading `*pSize` in each emits
//     a second `lwz` this class has no word for.

class WJsonWriter {
public:
    long GetBuffer(unsigned short *pBuffer, unsigned long *pSize);

private:
    unsigned short *mBuffer;   // OFF_BUFFER = 0
    unsigned long mBufferSize; // OFF_SIZE   = 4
};

#define WJSON_E_INVALIDARG   0x80070057L
#define WJSON_E_INSUFFICIENT 0x803F0005L

long WJsonWriter::GetBuffer(unsigned short *pBuffer, unsigned long *pSize) {
    unsigned long hr = 0;

    if (!pSize || (!pBuffer && *pSize != 0)) {
        hr = WJSON_E_INVALIDARG;
    } else {
        unsigned long outputSize = 0;
        unsigned long index = 0;

        if (mBufferSize > 0) {
            int offset = 0;
            do {
                index++;
                unsigned short wc = *(unsigned short *)((char *)mBuffer + offset);
                offset += 2;

                if (wc <= 0x7F) {
                    outputSize++;
                    if (outputSize < *pSize) {
                        *pBuffer = (unsigned char)(wc & 0x7F);
                        pBuffer++;
                        *pBuffer = 0;
                    }
                } else {
                    unsigned long maxSize = *pSize;
                    if (wc <= 0x7FF) {
                        outputSize += 2;
                        if (outputSize < maxSize) {
                            *pBuffer = (unsigned char)(0xC0 | ((wc >> 6) & 0x1F));
                            pBuffer++;
                            *pBuffer = (unsigned char)(0x80 | (wc & 0x3F));
                            pBuffer++;
                            *pBuffer = 0;
                        }
                    } else {
                        outputSize += 3;
                        if (outputSize < maxSize) {
                            *pBuffer = (unsigned char)(0xE0 | ((wc >> 12) & 0x0F));
                            pBuffer++;
                            *(pBuffer + 1) = (unsigned char)(0x80 | ((wc >> 6) & 0x3F));
                            pBuffer++;
                            *(pBuffer + 1) = (unsigned char)((wc & 0x3F) | 0x80);
                            pBuffer++;
                            *pBuffer = 0;
                        }
                    }
                }
            } while (index < mBufferSize);
        }

        if (outputSize >= *pSize) {
            hr = WJSON_E_INSUFFICIENT;
        }

        *pSize = outputSize + 1;
    }

    return hr;
}
