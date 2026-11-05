# List every registry write/delete in GameBoost.exe's Optimize (apply) and Restore handlers.
# usage: python -I tweaks.py GameBoost.exe   (sha256 746c100a...5de, internal version 3.10.21.2024)
# Pattern (Delphi register call): push value; push 0; mov ecx,name; mov edx,key; mov eax,hive; call helper.
import sys, re, subprocess, struct
exe = sys.argv[1]; d = open(exe, 'rb').read()
HELP = {0x638320: 'SetDWORD', 0x638228: 'SetString', 0x638408: 'SetBinary', 0x6385f8: 'DeleteValue',
        0x638050: 'GetDWORD', 0x637f20: 'GetString', 0x638510: 'SetBool', 0x63813c: 'GetBool'}
HIVE = {0x80000001: 'HKCU', 0x80000002: 'HKLM'}
RANGES = {'apply': (0x6cafc8, 0x6cb57e), 'restore': (0x6cd465, 0x6cd938)}

def ustr(va):  # Delphi UnicodeString literal at VA (.text: VA 0x401000 = file 0x400)
    o = va - 0x401000 + 0x400; s = ''
    while d[o:o+2] != b'\0\0': s += chr(struct.unpack('<H', d[o:o+2])[0]); o += 2
    return s

def calls(start, stop):
    asm = subprocess.run(['objdump', '-d', '-M', 'intel', f'--start-address={start}', f'--stop-address={stop}', exe],
                         capture_output=True, text=True).stdout
    pushes, regs, out = [], {}, []
    for l in asm.splitlines():
        m = re.search(r'\t(push|mov|call)\s+(.*)$', l)
        if not m: continue
        op, arg = m.group(1), m.group(2).strip()
        if op == 'push': pushes.append(arg)
        elif op == 'mov' and re.match(r'e[acd]x,0x[0-9a-f]+$', arg): regs[arg[:3]] = int(arg[4:], 16)
        elif op == 'mov' and arg[:3] in ('eax', 'ecx', 'edx'): regs.pop(arg[:3], None)
        elif op == 'call':
            t = int(arg.split()[0], 16) if arg.startswith('0x') else None
            if t in HELP and 'ecx' in regs:
                key = ustr(regs['edx']) if 'edx' in regs else '<GameBoost backup key>'
                val = pushes[-2] if len(pushes) >= 2 and HELP[t] != 'DeleteValue' else ''
                out.append((HELP[t], HIVE.get(regs.get('eax'), '?'), key, ustr(regs['ecx']), val))
            pushes, regs = [], {}
    return out

if __name__ == '__main__':
    res = {k: calls(*v) for k, v in RANGES.items()}
    for k, rows in res.items():
        print(f'== {k}')
        for r in rows: print('  %-11s %-4s %-70s %-26s %s' % r)
    # self-check against the hand-read disassembly (decomp/pgware-gameboost/GameBoost.exe/notes.md)
    a = {(r[3], r[4]) for r in res['apply'] if r[0].startswith('Set')}
    for want in [('Win32PrioritySeparation', '0x1'), ('TcpWindowSize', '0x3ebc0'), ('NonBestEffortLimit', '0x32'),
                 ('DefaultTTL', '0x40'), ('IoPageLockLimit', '0x1000'), ('EnablePrefetcher', '0x1')]:
        assert want in a, want
    assert sum(r[0] == 'DeleteValue' for r in res['restore']) >= 30
    print('self-check ok')
