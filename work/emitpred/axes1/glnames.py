import sys,re
d=open(sys.argv[1],'rb').read()
# split on 0x00 and 0x26, keep printable tokens that look like symbol names
toks=re.split(rb'[\x00\x26]', d)
out=set()
for t in toks:
    s=t.decode('latin1')
    m=re.search(r'[?A-Za-z_][A-Za-z0-9_?@$.]{2,}$', s)
    if m: out.add(m.group(0))
for x in sorted(out): print(x)
