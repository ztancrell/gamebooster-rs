//! The Mac power fix's GPU part: a soft PowerPlay table for the Radeon Pro 5500M with only the
//! power limit changed to [`GPU_WATTS`], built at run time from the card's **stock** table (no
//! other edits; the exe embeds no vendor table bytes). See
//! docs/superpowers/specs/2026-11-08-mac-power-fix-design.md and docs/tweaks/gpu-power-table.md.
//!
//! The stock table comes from the ACPI `VFCT` table (the VBIOS image the firmware hands the OS),
//! read with `GetSystemFirmwareTable`: VFCT -> image entry 1002:7340 -> ATOM ROM header -> master
//! data table entry 15 (`powerplayinfo`). The patch is `SocketPowerLimitAc[0]` and
//! `SocketPowerLimitDc[0]` (u16 at 494 and 510) from 50 to [`GPU_WATTS`]. Every layout value is
//! pinned to what was verified on this VBIOS; any mismatch is an error and nothing is written. The
//! Mac gate, the backup in the install record, the registry write and read-back, the GPU restart
//! and `uninstall` live in `crate::mac`.

/// GPU power limit (PPT) the table sets. Fallback 30 W if the confirmation run crosses the SMC budget.
pub const GPU_WATTS: u32 = 32;
const _: () = assert!(GPU_WATTS >= 20 && GPU_WATTS < STOCK_WATTS as u32);

/// The registry value (REG_BINARY) under the GPU's display class key that the AMD driver reads at start.
pub const VALUE: &str = "PP_PhmSoftPowerPlayTable";

const STOCK_WATTS: u16 = 50;
const TABLE_LEN: usize = 1674;
/// `SocketPowerLimitAc[0]` and `SocketPowerLimitDc[0]`, each followed by three zero entries.
const LIMITS: [usize; 2] = [494, 510];

fn u16_at(b: &[u8], off: usize) -> Result<u16, String> {
    b.get(off..off + 2).map(|s| u16::from_le_bytes([s[0], s[1]])).ok_or_else(|| format!("offset {off} is outside the data"))
}
fn u32_at(b: &[u8], off: usize) -> Result<u32, String> {
    b.get(off..off + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]])).ok_or_else(|| format!("offset {off} is outside the data"))
}
fn sums_to_zero(b: &[u8]) -> bool {
    b.iter().fold(0u8, |a, &x| a.wrapping_add(x)) == 0
}
fn check(ok: bool, why: &str) -> Result<(), String> {
    if ok { Ok(()) } else { Err(why.into()) }
}

/// The 5500M's VBIOS image inside the ACPI VFCT table, validated.
fn image(vfct: &[u8]) -> Result<&[u8], String> {
    check(vfct.starts_with(b"VFCT"), "VFCT: wrong signature")?;
    check(u32_at(vfct, 4)? as usize == vfct.len(), "VFCT: length field doesn't match the table size")?;
    check(sums_to_zero(vfct), "VFCT: bad checksum")?;
    let mut off = u32_at(vfct, 52)? as usize;
    loop {
        let (vendor, device, len) = (u16_at(vfct, off + 12)?, u16_at(vfct, off + 14)?, u32_at(vfct, off + 24)? as usize);
        let img = vfct.get(off + 28..off + 28 + len).ok_or("VFCT: image runs past the table")?;
        if (vendor, device) == (0x1002, 0x7340) {
            return Ok(img);
        }
        check(len != 0, "VFCT: no image for the 5500M (1002:7340)")?;
        off += 28 + len;
    }
}

/// The stock PowerPlay table inside the VBIOS image, validated.
fn stock(img: &[u8]) -> Result<&[u8], String> {
    check(img.starts_with(&[0x55, 0xAA]), "VBIOS: no option-ROM signature")?;
    let rom = img.get(..*img.get(2).ok_or("VBIOS: too short")? as usize * 512).ok_or("VBIOS: ROM length past the image")?;
    check(sums_to_zero(rom), "VBIOS: bad ROM checksum")?;
    let pcir = u16_at(rom, 0x18)? as usize;
    check(rom.get(pcir..pcir + 4) == Some(b"PCIR"), "VBIOS: no PCIR structure")?;
    check((u16_at(rom, pcir + 4)?, u16_at(rom, pcir + 6)?) == (0x1002, 0x7340), "VBIOS: PCIR is not 1002:7340")?;
    let atom = u16_at(rom, 0x48)? as usize;
    check(rom.get(atom + 4..atom + 8) == Some(b"ATOM"), "VBIOS: no ATOM ROM header")?;
    check((u16_at(rom, atom + 24)?, u16_at(rom, atom + 26)?) == (0x106B, 0x020F), "VBIOS: ATOM subsystem is not Apple 106B:020F")?;
    let part = crate::mac::VBIOS.as_bytes();
    check(rom.windows(part.len()).any(|w| w == part), "VBIOS: part number not found")?;
    let pp = u16_at(rom, u16_at(rom, atom + 32)? as usize + 4 + 2 * 15)? as usize;
    check(pp != 0, "VBIOS: no PowerPlay table")?;
    let t = rom.get(pp..pp + u16_at(rom, pp)? as usize).ok_or("PowerPlay table runs past the ROM")?;
    check(t.len() == TABLE_LEN, "PowerPlay table: unexpected size")?;
    check(t[2..5] == [12, 0, 1], "PowerPlay table: unexpected revisions")?;
    check(u16_at(t, 5)? == 482 && u32_at(t, 482)? == 8, "PowerPlay table: unexpected SMU table offset or version")?;
    for at in LIMITS {
        check(u16_at(t, at)? == STOCK_WATTS && t[at + 2..at + 8] == [0; 6], "PowerPlay table: power limit is not stock 50 W")?;
    }
    Ok(t)
}

/// Stock table from `vfct` with the power limit set to [`GPU_WATTS`].
pub fn patch(vfct: &[u8]) -> Result<Vec<u8>, String> {
    let stock = stock(image(vfct)?)?;
    let mut t = stock.to_vec();
    for at in LIMITS {
        t[at..at + 2].copy_from_slice(&(GPU_WATTS as u16).to_le_bytes());
    }
    let changed: Vec<usize> = (0..t.len()).filter(|&i| t[i] != stock[i]).collect();
    check(changed == LIMITS, "patched table differs from stock outside the power limit")?;
    Ok(t)
}

/// The patched table to write as [`VALUE`], from this machine's ACPI VFCT table.
#[cfg(windows)]
pub fn build() -> Result<Vec<u8>, String> {
    use windows_sys::Win32::System::SystemInformation::GetSystemFirmwareTable;
    const ACPI: u32 = u32::from_be_bytes(*b"ACPI");
    const VFCT: u32 = u32::from_le_bytes(*b"VFCT");
    let size = unsafe { GetSystemFirmwareTable(ACPI, VFCT, std::ptr::null_mut(), 0) };
    check(size != 0, "no ACPI VFCT table (can't read the stock GPU table)")?;
    let mut buf = vec![0u8; size as usize];
    let got = unsafe { GetSystemFirmwareTable(ACPI, VFCT, buf.as_mut_ptr(), size) };
    check(got == size, "can't read the ACPI VFCT table")?;
    patch(&buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put16(b: &mut [u8], at: usize, v: u16) {
        b[at..at + 2].copy_from_slice(&v.to_le_bytes());
    }
    fn put32(b: &mut [u8], at: usize, v: u32) {
        b[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
    fn fix_sum(b: &mut [u8], at: usize) {
        b[at] = 0;
        b[at] = 0u8.wrapping_sub(b.iter().fold(0u8, |a, &x| a.wrapping_add(x)));
    }

    /// A synthetic VFCT with only the fields the parser checks. ROM: 5 x 512 bytes, PCIR at
    /// 0x40, ATOM header at 0x80, part number at 0x100, master table at 0x140, table at 0x200.
    /// `edit` changes the table before the checksums are fixed.
    fn vfct(edit: impl Fn(&mut Vec<u8>)) -> Vec<u8> {
        let mut rom = vec![0u8; 2560];
        rom[..3].copy_from_slice(&[0x55, 0xAA, 5]);
        put16(&mut rom, 0x18, 0x40);
        rom[0x40..0x44].copy_from_slice(b"PCIR");
        put16(&mut rom, 0x44, 0x1002);
        put16(&mut rom, 0x46, 0x7340);
        put16(&mut rom, 0x48, 0x80);
        rom[0x84..0x88].copy_from_slice(b"ATOM");
        put16(&mut rom, 0x80 + 24, 0x106B);
        put16(&mut rom, 0x80 + 26, 0x020F);
        put16(&mut rom, 0x80 + 32, 0x140);
        rom[0x100..0x110].copy_from_slice(crate::mac::VBIOS.as_bytes());
        put16(&mut rom, 0x140 + 4 + 30, 0x200);
        let mut t = vec![0xA5u8; TABLE_LEN];
        put16(&mut t, 0, TABLE_LEN as u16);
        t[2..5].copy_from_slice(&[12, 0, 1]);
        put16(&mut t, 5, 482);
        put32(&mut t, 482, 8);
        for at in LIMITS {
            put16(&mut t, at, 50);
            t[at + 2..at + 8].fill(0);
        }
        edit(&mut t);
        rom[0x200..0x200 + TABLE_LEN].copy_from_slice(&t);
        let mut v = vec![0u8; 76];
        v[..4].copy_from_slice(b"VFCT");
        put32(&mut v, 52, 76);
        let mut entry = vec![0u8; 28];
        put16(&mut entry, 12, 0x1002);
        put16(&mut entry, 14, 0x7340);
        put32(&mut entry, 24, rom.len() as u32);
        v.extend(entry);
        v.extend(rom);
        v.extend([0u8; 28]);
        let n = v.len() as u32;
        put32(&mut v, 4, n);
        refix(&mut v);
        v
    }

    /// Fix the ROM checksum, then the VFCT one, so a test hits the check it means to.
    fn refix(v: &mut Vec<u8>) {
        fix_sum(&mut v[ROM..ROM + 2560], 0x9FF);
        fix_sum(v, 9);
    }
    const ROM: usize = 76 + 28;

    #[test]
    fn patch_touches_only_the_power_limit() {
        let v = vfct(|_| {});
        let t = patch(&v).unwrap();
        let stock = &v[ROM + 0x200..][..TABLE_LEN];
        let diff: Vec<usize> = (0..TABLE_LEN).filter(|&i| t[i] != stock[i]).collect();
        assert_eq!(diff, LIMITS);
        assert_eq!((u16_at(&t, 494).unwrap(), u16_at(&t, 510).unwrap()), (32, 32));
    }

    #[test]
    fn every_check_rejects() {
        let bad = |what: &str, edit: &dyn Fn(&mut Vec<u8>)| {
            let mut v = vfct(|_| {});
            edit(&mut v);
            refix(&mut v);
            assert!(patch(&v).is_err(), "{what} accepted");
        };
        bad("signature", &|v| v[0] = b'X');
        bad("VFCT length", &|v| v.push(0));
        bad("device", &|v| put16(v, 76 + 14, 0x7341));
        bad("image length", &|v| put32(v, 76 + 24, 1 << 20));
        bad("entry offset", &|v| put32(v, 52, 1 << 20));
        bad("ROM signature", &|v| v[ROM] = 0);
        bad("PCIR device", &|v| put16(v, ROM + 0x46, 0x7341));
        bad("ATOM signature", &|v| v[ROM + 0x84] = b'X');
        bad("ATOM subsystem", &|v| put16(v, ROM + 0x80 + 26, 0x0210));
        bad("part number", &|v| v[ROM + 0x100] = b'X');
        bad("table offset past the ROM", &|v| {
            put16(v, ROM + 0x140 + 34, 0x9F0);
            put16(v, ROM + 0x9F0, TABLE_LEN as u16);
        });
        bad("no table", &|v| put16(v, ROM + 0x140 + 34, 0));
        let mut v = vfct(|_| {});
        v[ROM + 5] ^= 1;
        fix_sum(&mut v, 9);
        assert!(patch(&v).is_err(), "ROM checksum accepted");
        v[9] ^= 1;
        assert!(patch(&vfct(|_| {})).is_ok() && patch(&v).is_err(), "VFCT checksum accepted");

        let bad_table = |what: &str, edit: fn(&mut Vec<u8>)| assert!(patch(&vfct(edit)).is_err(), "{what} accepted");
        bad_table("size", |t| put16(t, 0, 1675));
        bad_table("format revision", |t| t[2] = 11);
        bad_table("table revision", |t| t[4] = 2);
        bad_table("SMU offset", |t| put16(t, 5, 480));
        bad_table("SMU version", |t| put32(t, 482, 9));
        bad_table("AC limit not 50", |t| put16(t, 494, 32));
        bad_table("DC limit not 50", |t| put16(t, 510, 45));
        bad_table("AC[1] not 0", |t| put16(t, 496, 1));
    }
}
