// W-OSFINFO — the range-and-flag guarded two-level table lookup whose two
// failure statements are TAIL-MERGED with its success statement. The class
// `src/xdk/LIBCMT/osfinfo.cpp`'s `_free_osfhnd` is the workload instance of.
//
// Compile at the workload's own profile (`/O1 /Oi /EHsc /GR`). At `/Ox` this
// file is deliberately 0/1 in class: the mode gate lives in the PARSER
// (board #1638) and `scripts/mode_lane.sh` compiles every fixture at both.
//
// What this fixture is FOR, beyond the class itself — three things no earlier
// fixture in this seam can grade:
//
//  * **A REFLO in a `lwz` DISPLACEMENT.** `nhandle` is read by VALUE, so
//    nothing takes its address and there is no `addi` low half anywhere in the
//    body. Every other relocated global this port emits arrives through an
//    `addi rD,r11,0`.
//  * **A REFHI in a register that is NOT the scratch.** `pioinfo`'s high half
//    lands in r10 while r11 is busy, which `data_refs_of`'s old
//    `addis r11,0,0`-keyed walk could not SEE, let alone pair.
//  * **A zero-displacement load that is NOT a relocation**, twelve words below
//    one that is, off the same base register. `e->hnd` at `+0x50` and the
//    `nhandle` REFLO at `+0x18` are the same instruction form; only the walk's
//    open-pair state separates them.
//
// It is also the second graded cell for board #1720's merged symbol-order rule,
// and the first with TWO callees: the externals run `callee · callee · data ·
// data` here where `undname.cpp`'s run `data · callee · data`. Two loops would
// get THIS obj right and that one wrong, which is why one cell could never have
// established the rule.

// sizeof(ioinfo) MUST be 72 and must NOT be a power of two: the `+ (fh & 31)`
// below is pointer arithmetic, so c2 scales it by the struct's size, and it
// picks `mulli` for a non-power-of-two and `slwi` for a power of two. With one
// witness of each form the chooser is not fitted, so the reader refuses a
// power-of-two element size outright.
struct ioinfo {
    long hnd;      // offset 0 — pinned: the success store and the error store
                   // are ONE word, and that is only legal at zero
    char osfile;   // offset 4 — a BYTE field, so a `lbz` and not a `lwz`
    char pad[67];  // 4 + 1 + 67 = 72
};

extern int nhandle;
extern ioinfo *pioinfo[];

extern "C" int *c2rs_errno_a();
extern "C" int *c2rs_errno_b();

// The formal is UNSIGNED and the signed reading is a cast, which is what the
// workload's own IL says: `_free_osfhnd`'s `fh` loads as `86 42 75` and the
// range guard carries a `2C` to `86 41 74`. That conversion is not decoration —
// it is what makes c2 emit `cmpwi` for the first guard and `cmplw` for the
// second, on the same register four words apart. Written `int fh` this file is
// a DIFFERENT body: no `2C`, one compare form, and out of class.
int wosf_free_handle(unsigned fh)
{
    if ((int) fh >= 0 && fh < (unsigned) nhandle) {
        int i = (int) fh >> 5;
        // POINTER arithmetic, not a byte cast: the `* 72` is c2's own scaling
        // and appears in the IL as `33 <T> 48 · 04 · 28 00 00`. Written with an
        // explicit `(char *)` this is a DIFFERENT body — the table element
        // carries a `2C` conversion the class has no word for.
        ioinfo *e = &pioinfo[i][fh & 31];
        // ONE `if` with `&&`, not two nested ones. The IL says so: after the
        // flag guard's `38 <L>` the stream goes straight into the handle
        // guard's `B9` with no `53` between them, which is the short-circuit
        // shape and not a nested block. Written nested, this file has two extra
        // scope opens and is out of class.
        if ((e->osfile & 1) != 0 && e->hnd != -1) {
            e->hnd = -1;
            return 0;
        }
    }
    *c2rs_errno_a() = 9;
    *c2rs_errno_b() = 0;
    return -1;
}
