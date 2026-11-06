# Carve PE files out of an Inno Setup 6.x solid LZMA chunk (setup-1) without running anything.
# usage: python -I inno_carve.py SETUP.EXE OUTDIR
import sys, lzma, struct, os, re
src, out = sys.argv[1], sys.argv[2]
d = open(src, 'rb').read()
t = d.find(b'rDlPtS'); off1 = struct.unpack('<I', d[t+36:t+40])[0]
assert d[off1:off1+4] == b'zlb\x1a'
p = d[off1+4:off1+9]; props = p[0]
lc, lp, pb = props % 9, (props // 9) % 5, props // 45
dic = struct.unpack('<I', p[1:5])[0]
dec = lzma.LZMADecompressor(lzma.FORMAT_RAW, filters=[{'id': lzma.FILTER_LZMA1, 'lc': lc, 'lp': lp, 'pb': pb, 'dict_size': dic}])
raw = dec.decompress(d[off1+9:])
os.makedirs(out, exist_ok=True)
open(os.path.join(out, 'setup-1.solid.bin'), 'wb').write(raw)
print('decompressed', len(raw))

def unfilter(b):  # inverse of Inno TransformCallInstructions, 64 KiB blocks
    b = bytearray(b)
    for bs in range(0, len(b), 0x10000):
        size = min(0x10000, len(b) - bs)
        if size < 5: continue
        i = 0
        while i < size - 4:
            if b[bs+i] in (0xE8, 0xE9):
                i += 1
                j = bs + i
                if b[j+3] in (0, 0xFF):
                    addr = (bs + i + 4) & 0xFFFFFF
                    rel = (b[j] | b[j+1] << 8 | b[j+2] << 16) - addr
                    if rel & 0x800000: b[j+3] ^= 0xFF
                    b[j:j+3] = (rel & 0xFFFFFF).to_bytes(3, 'little')
                i += 4
            else:
                i += 1
    return bytes(b)

for m in re.finditer(rb'MZ', raw):
    o = m.start()
    if o + 0x40 > len(raw): continue
    pe = struct.unpack('<I', raw[o+0x3c:o+0x40])[0]
    if pe > 0x1000 or raw[o+pe:o+pe+4] != b'PE\0\0': continue
    nsec = struct.unpack('<H', raw[o+pe+6:o+pe+8])[0]; opt = struct.unpack('<H', raw[o+pe+20:o+pe+22])[0]
    magic = struct.unpack('<H', raw[o+pe+24:o+pe+26])[0]
    ddo = o + pe + 24 + (96 if magic == 0x10b else 112)
    cva, csz = struct.unpack('<II', raw[ddo+4*8:ddo+4*8+8])  # security dir = file offset
    end = 0; sh = o + pe + 24 + opt
    for k in range(nsec):
        rs, rp = struct.unpack('<II', raw[sh+40*k+16:sh+40*k+24]); end = max(end, rp + rs)
    end = max(end, cva + csz)
    print(f'PE @{o:#x} size {end}')
    open(os.path.join(out, f'pe_{o:08x}_{end}.bin'), 'wb').write(unfilter(raw[o:o+end]))
