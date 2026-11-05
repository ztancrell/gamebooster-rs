# usage: gdis.py PE startVA endVA -> intel disassembly annotated with strings/imports (generic PE32/PE32+)
# or:   gdis.py PE --refs "text"  -> string VAs and absolute 32-bit references to them (PE32 only;
#       x64 code uses RIP-relative addressing, so grep objdump output for '# 0x<VA>' instead)
import sys,re,subprocess,struct
exe=sys.argv[1]; d=open(exe,'rb').read()
P=subprocess.run(['objdump','-p',exe],capture_output=True,text=True).stdout
base=int(re.search(r'ImageBase\s+([0-9a-f]+)',P).group(1),16)
secs=[]
for l in subprocess.run(['objdump','-h',exe],capture_output=True,text=True).stdout.splitlines():
    m=re.match(r'\s*\d+\s+(\S+)\s+([0-9a-f]+)\s+([0-9a-f]+)\s+[0-9a-f]+\s+([0-9a-f]+)',l)
    if m: secs.append((int(m.group(3),16),int(m.group(2),16),int(m.group(4),16)))
def v2o(v):
    for va,sz,off in secs:
        if va<=v<va+sz: return v-va+off
imp={}; dll=None
for l in P.splitlines():
    m=re.match(r'\s*DLL Name: (\S+)',l)
    if m: dll=m.group(1)
    m=re.match(r'\s+([0-9a-f]{8,16})\s+<none>\s+[0-9a-f]+\s+(\S+)',l)
    if m: imp[int(m.group(1),16)+base]=m.group(2)
def strat(v):
    o=v2o(v)
    if o is None or o+2>len(d): return None
    t='';p=o
    while p<o+400 and p+2<=len(d):
        c=struct.unpack('<H',d[p:p+2])[0]
        if c==0: break
        if c<0x20 or c>0x7e: t=None;break
        t+=chr(c);p+=2
    if t and len(t)>=3: return 'U"'+t+'"'
    t=b'';p=o
    while p<o+300 and p<len(d) and d[p]!=0:
        if d[p]<0x20 or d[p]>0x7e: return None
        t+=bytes([d[p]]);p+=1
    if len(t)>=4: return 'A"'+t.decode()+'"'
if sys.argv[2]=='--refs':
    s=sys.argv[3]; hits=set()
    for enc in (s.encode(),s.encode('utf-16le')):
        for m in re.finditer(re.escape(enc),d):
            o=m.start()
            for va,sz,off in secs:
                if off<=o<off+sz: hits.add(o-off+va)
    for h in sorted(hits):
        for m in re.finditer(re.escape(struct.pack('<I',h)),d): 
            o=m.start()
            for va,sz,off in secs:
                if off<=o<off+sz: print(hex(h),'ref@',hex(o-off+va))
        print(hex(h),'string')
    sys.exit()
s=int(sys.argv[2],16);e=int(sys.argv[3],16)
out=subprocess.run(['objdump','-d','-M','intel','--no-show-raw-insn',f'--start-address={s:#x}',f'--stop-address={e:#x}',exe],capture_output=True,text=True).stdout
for l in out.splitlines()[6:]:
    n=[]
    for h in re.findall(r'0x([0-9a-f]{5,16})\b',l):
        v=int(h,16)
        if v in imp: n.append(imp[v])
        x=strat(v)
        if x: n.append(x)
    print(l+('   ; '+' '.join(n) if n else ''))
