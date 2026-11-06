# usage: refs.py EXE needle...   needle = 0xFILEOFF (string) or 0xVA (prefix v)
import sys,struct,re
d=open(sys.argv[1],'rb').read()
TEXT_OFF,TEXT_VA,TEXT_SZ=0x400,0x401000,0x618b60
def o2v(o): return o-0x400+0x401000
def v2o(v): return v-0x401000+0x400
def func_start(o):
    # scan back for push ebp; mov ebp,esp at 4-aligned address
    p=o
    while p>o-0x4000:
        p-=1
        if d[p:p+3]==b'\x55\x8b\xec' and (o2v(p)%4==0): return p
    return None
for a in sys.argv[2:]:
    v=int(a[1:],16) if a.startswith('v') else o2v(int(a,16))
    pat=struct.pack('<I',v)
    hits=[m.start() for m in re.finditer(re.escape(pat),d[TEXT_OFF:TEXT_OFF+TEXT_SZ])]
    out=[]
    for h in hits:
        h+=TEXT_OFF; fs=func_start(h)
        out.append(f"ref@{o2v(h):08x} fn~{o2v(fs) if fs else 0:08x}")
    print(f"{a} va={v:08x}: "+'; '.join(out[:12])+(f' (+{len(out)-12})' if len(out)>12 else ''))
