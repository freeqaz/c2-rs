import io
p='docs/BOARD.md'
s=open(p).read()
tail = """> **`#2267`–`#2279` are minted by nobody and are FREE.** Lane `w-main` was
> allocated `#2260`–`#2279` and used seven (`#2260`–`#2266`). The unused
> thirteen are recorded as explicitly unminted rather than left to be inferred
> from a gap.
"""
assert s.count(tail)==1, s.count(tail)
rows = tail + open('work/w-blockir/rows.md').read()
s=s.replace(tail, rows)
open(p,'w').write(s)
print("ok")
