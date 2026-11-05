"""Dump MSI database tables as text, from streams already unpacked by `7z x file.msi`.

Usage: python3 -I msitables.py STREAMDIR [Table ...]
7z decodes the stream names; table streams are named '!<Table>'. Format: string pool
(!_StringPool / !_StringData), column definitions in !_Columns, each table stored column-major.
"""
import os, struct, sys


def load_strings(d):
    pool = open(os.path.join(d, '!_StringPool'), 'rb').read()
    data = open(os.path.join(d, '!_StringData'), 'rb').read()
    cp, = struct.unpack_from('<I', pool, 0)
    wide = bool(cp & 0x80000000)
    out, pos, i = [''], 0, 4
    while i + 4 <= len(pool):
        ln, rc = struct.unpack_from('<HH', pool, i); i += 4
        if ln == 0 and rc:  # long string: next entry holds the length
            ln = struct.unpack_from("<I", pool, i)[0]
            i += 4
        out.append(data[pos:pos + ln].decode('utf-8', 'replace')); pos += ln
    return out, (3 if wide else 2)


def cell_width(t, sref):
    if t & 0x0800:
        return sref  # string / binary (stored as string index)
    return 2 if (t & 0xff) == 2 else 4


def read_table(d, cols, name, strs, sref):
    p = os.path.join(d, '!' + name)
    raw = open(p, 'rb').read() if os.path.exists(p) else b''
    defs = sorted(cols.get(name, []))
    rowsize = sum(cell_width(t, sref) for _, _, t in defs)
    n = len(raw) // rowsize if rowsize else 0
    table, off = [[None] * len(defs) for _ in range(n)], 0
    for ci, (_, cname, t) in enumerate(defs):
        w = cell_width(t, sref)
        for r in range(n):
            v = int.from_bytes(raw[off:off + w], 'little'); off += w
            if t & 0x0800:
                table[r][ci] = strs[v] if v < len(strs) else '?%d' % v
            elif v == 0:
                table[r][ci] = None
            else:
                table[r][ci] = v - (0x8000 if w == 2 else 0x80000000)
    return [c for _, c, _ in defs], table


def columns(d, strs, sref):
    raw = open(os.path.join(d, '!_Columns'), 'rb').read()
    n = len(raw) // (sref * 2 + 4)
    o = 0
    tbl = [int.from_bytes(raw[o + sref * r:o + sref * r + sref], 'little') for r in range(n)]; o += sref * n
    num = [struct.unpack_from('<H', raw, o + 2 * r)[0] - 0x8000 for r in range(n)]; o += 2 * n
    nam = [int.from_bytes(raw[o + sref * r:o + sref * r + sref], 'little') for r in range(n)]; o += sref * n
    typ = [struct.unpack_from('<H', raw, o + 2 * r)[0] - 0x8000 for r in range(n)]
    cols = {}
    for t, nu, na, ty in zip(tbl, num, nam, typ):
        cols.setdefault(strs[t], []).append((nu, strs[na], ty))
    return cols


if __name__ == '__main__':
    assert cell_width(0x0d48, 2) == 2 and cell_width(0x1502, 2) == 2 and cell_width(0x0104, 2) == 4
    d = sys.argv[1]
    strs, sref = load_strings(d)
    cols = columns(d, strs, sref)
    for name in (sys.argv[2:] or sorted(cols)):
        hdr, rows = read_table(d, cols, name, strs, sref)
        print('## %s (%d rows): %s' % (name, len(rows), ' | '.join(hdr)))
        for r in rows:
            print('  ' + ' | '.join('' if v is None else str(v) for v in r))
