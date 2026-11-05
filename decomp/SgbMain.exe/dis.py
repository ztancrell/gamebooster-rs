# usage: dis.py EXE startVA endVA  -> annotated intel disassembly
import sys,re,subprocess,struct
exe=sys.argv[1]; s=int(sys.argv[2],16); e=int(sys.argv[3],16)
d=open(exe,'rb').read()
imp={}
for l in subprocess.run(['objdump','-p',exe],capture_output=True,text=True).stdout.splitlines():
    m=re.match(r'\s+([0-9a-f]{8})\s+<none>\s+[0-9a-f]+\s+(\S+)',l)
    if m: imp[int(m.group(1),16)+0x400000]=m.group(2)
def v2o(v):
    if 0x401000<=v<0xa1a000+0x4a94: return v-0x401000+0x400 if v<0xa1a000 else v-0xa1a000+0x619000
    if 0xa1f000<=v<0xa58000: return v-0xa1f000+0x61dc00
    return None
def thunk(v):
    o=v2o(v)
    if o and d[o:o+2]==b'\xff\x25':
        t=struct.unpack('<I',d[o+2:o+6])[0]; return imp.get(t)
def strat(v):
    o=v2o(v)
    if not o: return None
    # utf16
    t=''; p=o
    while p<o+400:
        c=struct.unpack('<H',d[p:p+2])[0]
        if c==0: break
        if c<0x20 and c not in (9,10,13): t=None; break
        t+=chr(c); p+=2
    if t and len(t)>=2 and sum(ch.isprintable() for ch in t)==len(t) and p<o+400: return 'U"'+t+'"'
    t=b''; p=o
    while p<o+200 and d[p]!=0:
        if d[p]<0x20 or d[p]>0x7e: return None
        t+=bytes([d[p]]); p+=1
    if len(t)>=3: return 'A"'+t.decode()+'"'
out=subprocess.run(['objdump','-d','-M','intel','--no-show-raw-insn',f'--start-address={s:#x}',f'--stop-address={e:#x}',exe],capture_output=True,text=True).stdout
for l in out.splitlines()[6:]:
    notes=[]
    for h in re.findall(r'0x([0-9a-f]{5,8})\b',l):
        v=int(h,16)
        if v in imp: notes.append(imp[v])
        n=thunk(v)
        if n: notes.append('->'+n)
        if 'call' not in l and 'j' != l.split()[1][:1] if len(l.split())>1 else False:
            st=strat(v)
            if st: notes.append(st)
    m=re.search(r'call\s+([0-9a-f]+)',l)
    if m:
        n=thunk(int(m.group(1),16))
        if n: notes.append('->'+n)
    print(l.rstrip()+('    ; '+' | '.join(notes) if notes else ''))
