# w-mixed — PREREG ADDENDUM 1

Written **after** P0's ladder and **before** GRID M's generator existed, before
any GRID M source was written and before any GRID M cell was compiled. Two
corrections to `PREREG.md`, both forced by files already committed on master and
neither by anything this lane compiled.

---

## A1.1 §2.1's MECHANISM IS REFUTED, by a disassembly committed six days ago

`PREREG.md` §2.1 claims the `2base` `+1` is *not* a kind bonus but a real extra
**read** of the producer's register — that in the bind spelling the address
register also serves as the store BASE. **That is false**, and
`work/w-spell/gridS_dis.txt` says so:

```text
  == S-self-1base-r2k3            == S-self-2base-r2k3
  li   10, 7                      li   10, 7
  addi 11, 3, 96                  addi 11, 3, 96
  stw  10, 32(3)                  stw  10, 32(3)
  stw  11, 96(3)                  stw  10, 36(3)
  stw  10, 36(3)                  stw  10, 40(3)
  stw  10, 40(3)                  stw  11, 96(3)
  stw  11, 100(3)                 stw  11, 100(3)
```

The bind is folded to `r3` plus a displacement in **both** spellings. `r11` is
read exactly `ru` times in both. The two cells differ in their store
**SCHEDULE** and in nothing else the register file can see — which is board
#1128 (*a bind IS a second base symbol*) showing up where it was already known
to, in `order`, and **not** in `alloc`.

**Consequence, registered before the grade:** `b` loses its mechanism and
reverts to what the record already knew it was — a **free parameter fitted on
three cells** (`H2-self-2base-r2k4`, `H2-self-2base-r3k5`, `X-A-r3k5`). It is
now a claim that the allocator reads the same *symbol structure* the scheduler
does, with no account of why. That makes **P3 the more likely outcome, and
P1 more likely still.** This is written down here rather than discovered in §8.

## A1.2 THE DOMAIN MUST SAY **PREFIX**, and the record already refutes the wider reading

`PREREG.md` §2 says the domain is *"an interior address"*. Read literally that
includes a **cross** address — the value points at a sub-object that is not a
prefix of the one being stored into — and on that reading **H-MIX is already
wrong on the record**, at six cells, every one of them at `(1,1)`:

```text
  w-ilx   V3-cross-in2-r1k1  V4-cross-tail-r1k1  V5-otherptr-r1k1
          V8-bindcross-r1k1                          (holdout_grade.out)
  w-spell S-cross-1base-r1k1  S-cross-2base-r1k1     (fit.out / gridS_dis.txt)
```

all `obj = const`, where `cu <= ru+1+b` says `prod`.

So the domain is tightened, **before the freeze**, to KEY ILX's own SELF
condition, which is also `xboxheap`'s:

> the stored address is a **prefix** of the address stored into — the value is
> `&X` and every store of it writes into `X` or into a sub-object of `X`.

This is syntactic and the reader can compute it from `BoundAddr { base, off }`
against the store's own base and offset. `xboxheap` satisfies it: the value is
`this+8` and the two stores of it are at `this+8` and `this+12`.

**This is a tightening of a domain, made after re-reading committed tables and
before compiling anything** — and it is exactly the move #912 warns about. So
GRID M carries the `cross` and `otherobj` cells anyway, **declared
out-of-domain controls at freeze time**, and their job is to show the domain
boundary is real rather than drawn around a failure. If a `cross` control comes
back agreeing with the in-domain rule, the tightening was unnecessary and this
addendum is a MISS.

## A1.3 P0 is settled and P8 is added

P0 graded **HIT** on a compiled ladder (`work/w-mixed/p0/probe.txt`): with the
mixed-kind clause lifted, `k_target` — `w-carrier`'s own copy of `xboxheap`'s
ctor — moves to `store-run-bind-call-tail-mr-slot:eof`, and with that lifted too
it moves again, to `store-run-bind-no-emitter-carrier:eof`. It never converts.

| **P8** | added: **`xboxheap` prices at ≥ 3 named reader keys, not 1**, and there is a fourth refusal below them in the emitter (`leaf/store.rs:274`, `value_bound`) that no reader lift can reach | the ladder + a read of the emitter |
