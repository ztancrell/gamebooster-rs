# usage: python3 -I winsvc_lists.py WinSvc.ini  -> markdown lists of the default Boost services
# WinSvc.ini is plain text (UTF-8/ASCII INI); no decryption needed. 1 = ticked (stopped by
# default), 0 = listed but unticked. Keys repeat in some sections; first value wins, order kept.
import sys

def parse(text):
    secs, cur = {}, None
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith(';'):
            continue
        if line.startswith('[') and line.endswith(']'):
            cur = secs.setdefault(line[1:-1], {})
        elif cur is not None and '=' in line:
            k, v = line.rsplit('=', 1)
            cur.setdefault(k.strip(), v.strip())
    return secs

def demo():
    s = parse("[Services_X]\nA=1\nB=0\nA=0\n[Services_3rd]")
    assert s == {'Services_X': {'A': '1', 'B': '0'}, 'Services_3rd': {}}

if __name__ == '__main__':
    demo()
    for sec, kv in parse(open(sys.argv[1], encoding='utf-8', errors='replace').read()).items():
        on = [k for k, v in kv.items() if v == '1']
        off = [k for k, v in kv.items() if v != '1']
        print(f"### [{sec}] ({len(on)} ticked, {len(off)} unticked)\n")
        print("Ticked (=1, selected for Boost by default): " + (', '.join(f'`{k}`' for k in on) or 'none') + "\n")
        print("Unticked (=0, listed only): " + (', '.join(f'`{k}`' for k in off) or 'none') + "\n")
