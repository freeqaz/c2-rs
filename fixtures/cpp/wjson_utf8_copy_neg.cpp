// W-JSON — the NEGATIVE cells. Ten near-misses of `wjson_utf8_copy.cpp`'s
// class, each of which must be **0/1 in class** at the workload's own profile
// (`/O1 /Oi /EHsc /GR`), and each for its OWN reason.
//
// Read per cell with an applied-and-reverted probe patch
// (`work/w-json/decline_probe.md`), never off `c2rs census`'s fall-through
// blocker: board **#1704** — the census reports one key for the whole file, so a
// `_neg` fixture that is only *counted* proves that ten functions declined and
// nothing about whether they declined for ten different reasons. That method is
// w-cfgclass §6.2's and this is the SEVENTH lane to pay it.
//
// Every cell differs from `wjson_utf8_copy.cpp` in **exactly one way**, braces
// included — w-xlr #1789's lesson, where four cells written with an unbraced
// inner `if` all stopped at the same scope-pair production and three of them
// never reached the fact they were written for, while the file still read
// `0/10`.
//
// The clause each cell is FOR is named beside it, and the key it actually
// reaches is recorded in `work/w-json/neg_clauses.txt`. Where the walk stops in
// a different block from the one the comment expects, the comment says so
// rather than being reworded.

#define WJSON_E_INVALIDARG   0x80070057L
#define WJSON_E_INSUFFICIENT 0x803F0005L

// ---- n1: `hr` is SIGNED --------------------------------------------------
// The live signedness fence, board #1788's direction. `long hr` and
// `unsigned long hr` produce the IDENTICAL relational bytes — the opcodes are
// sign-agnostic — and differ only in the operand TYPE. c2 emits `cmpw` where
// this class's emitter has an unconditional `cmplw`, in four places. Caught by
// `TY_TAG_KIND` slot 0, which is pinned `(0x86, 0x42)`.
class WJsonN1 {
public:
    long GetBuffer(unsigned short *pBuffer, unsigned long *pSize);
private:
    unsigned short *mBuffer;
    unsigned long mBufferSize;
};
long WJsonN1::GetBuffer(unsigned short *pBuffer, unsigned long *pSize) {
    long hr = 0;
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

// ---- n2: the ASCII bound is 0x3F, not 0x7F -------------------------------
// A PINNED UTF-8 constant. It lands in a `cmplwi` immediate, so it looks like a
// free field — and it is not, because it is one of eight constants that are one
// program: the `clrlwi` width beside it still masks seven bits. Block
// `json-loop-head-and-one-byte-arm`.
class WJsonN2 {
public:
    long GetBuffer(unsigned short *pBuffer, unsigned long *pSize);
private:
    unsigned short *mBuffer;
    unsigned long mBufferSize;
};
long WJsonN2::GetBuffer(unsigned short *pBuffer, unsigned long *pSize) {
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
                if (wc <= 0x3F) {
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

// ---- n3: the two-byte lead is 0xD0, not 0xC0 -----------------------------
// The same argument one block later, and the cell that separates the two-byte
// arm's key from the one-byte arm's. Block `json-two-byte-arm`.
class WJsonN3 {
public:
    long GetBuffer(unsigned short *pBuffer, unsigned long *pSize);
private:
    unsigned short *mBuffer;
    unsigned long mBufferSize;
};
long WJsonN3::GetBuffer(unsigned short *pBuffer, unsigned long *pSize) {
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
                            *pBuffer = (unsigned char)(0xD0 | ((wc >> 6) & 0x1F));
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

// ---- n4: the three-byte arm stores through `*pBuffer`, not `*(pBuffer+1)` -
// The positive fixture's correction 1, undone. It is the same UTF-8 output and a
// completely different address computation: c2 folds the two bumps into one
// `addi` chain in the reference and cannot here. Block `json-three-byte-arm`.
class WJsonN4 {
public:
    long GetBuffer(unsigned short *pBuffer, unsigned long *pSize);
private:
    unsigned short *mBuffer;
    unsigned long mBufferSize;
};
long WJsonN4::GetBuffer(unsigned short *pBuffer, unsigned long *pSize) {
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
                            *pBuffer = (unsigned char)(0x80 | ((wc >> 6) & 0x3F));
                            pBuffer++;
                            *pBuffer = (unsigned char)((wc & 0x3F) | 0x80);
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

// ---- n5: the capacity test is `<=`, not `<` -------------------------------
// A DIFFERENT relational opcode byte in the one-byte arm's guard: `21` where the
// class has `22`. The nearest thing in this file to a one-byte change, and the
// cell that shows the template pins operators and not only operands.
class WJsonN5 {
public:
    long GetBuffer(unsigned short *pBuffer, unsigned long *pSize);
private:
    unsigned short *mBuffer;
    unsigned long mBufferSize;
};
long WJsonN5::GetBuffer(unsigned short *pBuffer, unsigned long *pSize) {
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
                    if (outputSize <= *pSize) {
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

// ---- n6: a `while`, not a `do`/`while` ------------------------------------
// The loop's own shape. Semantically identical here — the body is guarded by
// `mBufferSize > 0` either way — and a completely different block plan: the test
// moves to the top and the back edge becomes a `3A` at the bottom.
class WJsonN6 {
public:
    long GetBuffer(unsigned short *pBuffer, unsigned long *pSize);
private:
    unsigned short *mBuffer;
    unsigned long mBufferSize;
};
long WJsonN6::GetBuffer(unsigned short *pBuffer, unsigned long *pSize) {
    unsigned long hr = 0;
    if (!pSize || (!pBuffer && *pSize != 0)) {
        hr = WJSON_E_INVALIDARG;
    } else {
        unsigned long outputSize = 0;
        unsigned long index = 0;
        if (mBufferSize > 0) {
            int offset = 0;
            while (index < mBufferSize) {
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
            }
        }
        if (outputSize >= *pSize) {
            hr = WJSON_E_INSUFFICIENT;
        }
        *pSize = outputSize + 1;
    }
    return hr;
}

// ---- n7: the bad-argument status has a ZERO LOW HALF ----------------------
// `0x80070000` is one `lis` where the class has `lis`+`ori`, so it is a shorter
// body. This is a POST-match clause: the template accepts the stream and
// `is_two_word_constant` refuses the value.
class WJsonN7 {
public:
    long GetBuffer(unsigned short *pBuffer, unsigned long *pSize);
private:
    unsigned short *mBuffer;
    unsigned long mBufferSize;
};
long WJsonN7::GetBuffer(unsigned short *pBuffer, unsigned long *pSize) {
    unsigned long hr = 0;
    if (!pSize || (!pBuffer && *pSize != 0)) {
        hr = 0x80070000L;
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

// ---- n8: a THIRD formal ---------------------------------------------------
// The formal count is pinned because it decides which registers the guards read:
// a third one arrives in r6, which this body uses for the byte offset. Refused
// before a body byte is read, by `parse_params`/`parse_formals`.
class WJsonN8 {
public:
    long GetBuffer(unsigned short *pBuffer, unsigned long *pSize, unsigned long extra);
private:
    unsigned short *mBuffer;
    unsigned long mBufferSize;
};
long WJsonN8::GetBuffer(unsigned short *pBuffer, unsigned long *pSize, unsigned long extra) {
    unsigned long hr = 0;
    if (!pSize || (!pBuffer && *pSize != 0)) {
        hr = WJSON_E_INVALIDARG;
    } else {
        unsigned long outputSize = extra;
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

// ---- n9: the wide arms re-read `*pSize` instead of `maxSize` --------------
// The positive fixture's correction 3, undone. `maxSize` is one `lwz` hoisted to
// the top of the wide block; re-reading emits a second one in each arm, and the
// register that held it is free for something else.
class WJsonN9 {
public:
    long GetBuffer(unsigned short *pBuffer, unsigned long *pSize);
private:
    unsigned short *mBuffer;
    unsigned long mBufferSize;
};
long WJsonN9::GetBuffer(unsigned short *pBuffer, unsigned long *pSize) {
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
                        if (outputSize < *pSize) {
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

// ---- n10: the trailing `*pSize = outputSize + 1` becomes `+ 2` ------------
// The last block's only literal, and the cell that separates the tail's key from
// every block above it. It lands in an `addi` immediate and is STILL pinned: the
// class has one witness of one value and #1767's rule is about what an emitter
// can vary, not about what an immediate field can hold.
class WJsonN10 {
public:
    long GetBuffer(unsigned short *pBuffer, unsigned long *pSize);
private:
    unsigned short *mBuffer;
    unsigned long mBufferSize;
};
long WJsonN10::GetBuffer(unsigned short *pBuffer, unsigned long *pSize) {
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
        *pSize = outputSize + 2;
    }
    return hr;
}
