# usage: dfm_timers.py PE  -> every TTimer in binary DFM (TPF0) resources: form, name, Interval, Enabled, OnTimer
import sys,struct
d=open(sys.argv[1],'rb').read()
def ss(p): n=d[p]; return d[p+1:p+1+n].decode('latin1'),p+1+n
def val(p):
    t=d[p];p+=1
    if t==0: return None,p
    if t==2: return struct.unpack_from('<b',d,p)[0],p+1
    if t==3: return struct.unpack_from('<h',d,p)[0],p+2
    if t==4: return struct.unpack_from('<i',d,p)[0],p+4
    if t in(6,7): return ss(p)
    if t==8: return False,p
    if t==9: return True,p
    if t==0x12: n=struct.unpack_from('<i',d,p)[0]; return d[p+4:p+4+2*n].decode('utf-16le','replace'),p+4+2*n
    if t==0x0b:
        out=[]
        while d[p]: s,p=ss(p); out.append(s)
        return out,p+1
    if t==5: return 'ext',p+10
    if t==0x0a: n=struct.unpack_from('<i',d,p)[0]; return 'bin',p+4+n
    if t==0x14: n=struct.unpack_from('<i',d,p)[0]; return d[p+4:p+4+n].decode('utf8','replace'),p+4+n
    raise ValueError(t)
forms=[i for i in range(len(d)) if d.startswith(b'TPF0',i)]
for i in range(len(d)):
    n=d[i]
    if 5<n<32 and d[i+1:i+2]==b'T' and d[i+1:i+1+n].endswith(b'Timer') and d[i+1:i+1+n].isalnum() and 0<d[i+1+n]<64 and d[i-1]==0:
        form=max([f for f in forms if f<i],default=-1)
        fname=ss(form+4)[0] if form>=0 else '?'
        cls=d[i+1:i+1+n].decode(); name,p=ss(i+1+n); props={}
        try:
            while d[p]:
                k,p=ss(p); v,p=val(p); props[k]=v
        except Exception as e: props['err']=str(e)
        print(f'off=0x{i:x} form={fname} {cls} {name} Interval={props.get("Interval",1000)} Enabled={props.get("Enabled",True)} OnTimer={props.get("OnTimer")}')
