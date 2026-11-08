# GPU power table (Mac power fix): stock source, offsets, validation

Research notes for the builder of the Mac power fix
([design](../superpowers/specs/2026-11-08-mac-power-fix-design.md), open question "Source of the
stock table"). Built: `src/gpu_table.rs` (parse, validate, patch; unit tests on synthetic buffers
only), written by `install` in `src/mac.rs`; checked locally against the Mac's VFCT dump (table
found at `0xCB98`, 1674 bytes; the patched table matches the MorePowerTool registry table at 494/510
and differs from it only in MorePowerTool's OverDrive edits, offsets 194-255). Confirmation on the
Mac (HWiNFO PPT limit 32 W) still owed. Researched 2026-11-08 on the target Mac (read-only) plus the public
Linux amdgpu headers (`smu_v11_0_pptable.h`, `smu11_driver_if_navi10.h`, `atomfirmware.h`).
Everything here is in our own words; no vendor bytes are in this repo or meant to be shipped.

Goal: write `PP_PhmSoftPowerPlayTable` (REG_BINARY) under
`HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000`, built
at run time from the card's stock PowerPlay table with only the power limit changed to 32 W.

## Where the stock table comes from: the ACPI VFCT table

Apple's firmware hands the GPU's video BIOS to the OS in an ACPI table called **VFCT** ("video
BIOS content table"; Linux amdgpu reads it as one of its VBIOS sources). Windows exposes every
ACPI table to user mode through `GetSystemFirmwareTable` in kernel32, so no kernel driver, no ADL
and no file on disk is needed.

- Call `GetSystemFirmwareTable(provider, table_id, buf, size)` with provider `'ACPI'`
  (`0x41435049`) and table id `'VFCT'` as a little-endian u32 of the four signature bytes
  (`0x54434656`). Call once with a null buffer to get the size, then again to fill it.
  `EnumSystemFirmwareTables('ACPI', ...)` lists the signatures if you want to check presence first.
- The table is written by the firmware at boot, so it is the stock image even when a soft table
  (MorePowerTool or ours) is in the registry. It cannot be changed from Windows.
- Layout (all little-endian):
  - bytes 0..36: the standard ACPI header (signature `VFCT`, u32 total length at 4, and a checksum:
    all bytes of the table sum to 0 mod 256).
  - offset 52: u32 offset of the first image entry (76 on the Mac). Offset 56: u32 offset of a
    second "lib" image (0 on the Mac).
  - each image entry: a 28-byte header (u32 PCI bus, u32 device, u32 function, u16 vendor id,
    u16 device id, u16 subsystem vendor, u16 subsystem id, u32 revision, u32 image length) followed
    by the image bytes. Entries follow one another; walk them until one has vendor `0x1002` and
    device `0x7340`. On the Mac there is one entry (bus 3, device 0, function 0, subsystem fields
    0, image length 62976) followed by 28 zero bytes.

### Finding the PowerPlay table inside the VBIOS image

1. The image starts with the PCI option-ROM signature `55 AA`; byte 2 times 512 is the image
   length (62976 here). All bytes of that length sum to 0 mod 256 (ROM checksum).
2. The u16 at offset `0x18` points to the PCI data structure (`PCIR`): vendor id at +4, device id
   at +6. Check `0x1002` / `0x7340`.
3. The u16 at offset `0x48` points to the ATOM ROM header. Its bytes +4..+8 read `ATOM`; +24 and
   +26 hold the subsystem vendor and subsystem id (`0x106B`, `0x020F` on the Mac); the u16 at +32
   (`0x20`) is the offset of the master data table.
4. The master data table is a 4-byte common header (u16 size, u8 format, u8 content) followed by
   u16 offsets, one per data table, 0 = absent. The PowerPlay table is entry **15** (the 16th u16,
   named `powerplayinfo` in `atomfirmware.h`), i.e. the u16 at master + 4 + 30.
5. At that offset sits the PowerPlay table, starting with its own common header whose first u16 is
   its total size. Copy exactly that many bytes: that is the stock soft table, byte for byte the
   format the driver accepts in `PP_PhmSoftPowerPlayTable` (a MorePowerTool registry table and the
   VBIOS table differ only in the edited fields, see "Verified on the Mac").

The VBIOS part number (`113-D32206U1-019`) appears as a NUL-terminated ASCII string inside the
image; the same string is the registry's `HardwareInformation.BiosString` (UTF-16) under the class
key, which is a cheap gate check before reading anything.

### Other sources checked (and why not)

- **Registry:** the only PowerPlay data under the class key is `PP_PhmSoftPowerPlayTable` itself
  (absent on a fresh install, MorePowerTool's on the user's Mac) and `PP_PhmSoftWTTable` (the
  WattMan table, no power limit). No VBIOS copy is stored there.
- **AMD ADL (`atiadlxx.dll`, driver 25.10.45.11 on the Mac):** no export returns the PowerPlay
  table. The relevant ones are `ADL2_Adapter_VideoBiosInfo_Get` (part number/version strings only)
  and `ADL2_Overdrive8_Init_Setting(X2)_Get` (OverDrive ranges only). Not usable as a source.
- **PCI expansion ROM / legacy shadow:** needs a kernel driver to map; out of scope. GPU-Z reads
  the ROM through its own driver.
- **WMI:** `Win32_VideoController` only has strings.
- **ACPI ATRM:** the GPU's SSDT has no ATRM method (mbp16-power `docs/reference/target-dump.md`),
  and user mode can't evaluate ACPI methods anyway.

Fallback: none needed on this model, since the gate already requires the exact MacBookPro16,1 /
5500M / VBIOS. If VFCT is missing, or any validation step below fails, skip the GPU table with one
plain line and run the rest of the boost. Never build from the registry value: on a machine that
already has a soft table it is not stock.

## What to patch

Navi 14 uses the SMU 11 PowerPlay layout (`smu_11_0_powerplay_table`, packed, little-endian). Byte
offsets from the start of the table:

| Offset | Size | Field | Stock (Mac) |
|---|---|---|---|
| 0 | u16 | structure size (= table length) | 1674 |
| 2 | u8 | format revision | 12 |
| 3 | u8 | content revision | 0 |
| 4 | u8 | table revision | 1 |
| 5 | u16 | table_size: offset of the embedded SMU `PPTable_t` | 482 |
| 182 | | OverDrive table (revision u8, 3 reserved, u32 feature count, u32 setting count, u8 cap[32] at 194, u32 max[32] at 226, u32 min[32] at 354) | revision 128, caps all 0 |
| 482 | u32 | `PPTable_t` Version | 8 |
| **494** | u16 | `SocketPowerLimitAc[0]` (PPT0, watts) | **50** |
| 496..501 | u16 x3 | `SocketPowerLimitAc[1..3]` | 0 |
| **510** | u16 | `SocketPowerLimitDc[0]` (PPT0, watts) | **50** |
| 512..517 | u16 x3 | `SocketPowerLimitDc[1..3]` | 0 |
| 590 | u16 | `MaxVoltageGfx` (mV x 4) | 3800 = 950 mV |

MorePowerTool's "Power Limit GPU (W)" writes the same value to both `SocketPowerLimitAc[0]` (494)
and `SocketPowerLimitDc[0]` (510). Our patch is exactly those two u16 values set to 32. Nothing
else changes:

- There is no checksum or CRC in the PowerPlay table, and the size fields don't change. The VBIOS
  checksum is irrelevant because we never write the VBIOS.
- The OverDrive power percentage (OD max/min setting 9, offsets 262 / 390, both 20 %) is the
  Adrenalin slider range, not the limit; MorePowerTool leaves it alone and so do we.
- Leave the OverDrive caps at stock (all 0). MorePowerTool also switched on OD caps 0-3 and 8 and
  lowered the OD clock/voltage maxima; those are its edits, not part of the power limit.

## Validation before writing (all must pass, else skip)

1. Gate (already in the design): Apple / MacBookPro16,1 / `PCI\VEN_1002&DEV_7340&SUBSYS_020F106B`
   / `HardwareInformation.BiosString` = `113-D32206U1-019`.
2. VFCT: signature `VFCT`, length field equals the returned size, bytes sum to 0 mod 256, image
   entry inside the table with vendor `0x1002` / device `0x7340` and an image length that fits.
3. Image: starts `55 AA`, byte 2 x 512 <= image length, ROM bytes sum to 0 mod 256, `PCIR`
   vendor/device `0x1002`/`0x7340`, `ATOM` signature, ATOM header subsystem `0x106B`/`0x020F`,
   part number string `113-D32206U1-019` present.
4. PowerPlay table: offset non-zero and offset + size inside the image; size exactly 1674, format
   revision 12, content revision 0, table revision 1, `table_size` 482, `PPTable_t` Version 8.
   Pin these exact values: they are what was verified on this VBIOS; a different value means a
   different layout and offsets 494/510 can't be trusted.
5. Field sanity: `SocketPowerLimitAc[0]` and `SocketPowerLimitDc[0]` both 50 (stock), entries 1..3
   all 0. The new value is a named constant (32, fallback 30), must be below stock and at least
   ~20 W.
6. After patching: the patched buffer differs from the stock one in exactly offsets 494 and 510
   (low bytes; the high bytes stay 0).
7. After writing: read the value back and compare byte for byte. For `status`: the table is ours
   if the registry value equals stock-with-32-W rebuilt from VFCT (a cheap exact compare), and it
   is "someone else's table" otherwise.

## Verified on the Mac (2026-11-08, read-only)

- `EnumSystemFirmwareTables('ACPI')` lists APIC DMAR ECDT FACP HPET MCFG SBST SSDT **VFCT**.
  `GetSystemFirmwareTable('ACPI','VFCT')` returned 63108 bytes from an elevated session (OEM
  `APPLE`, ACPI checksum good, one image entry at offset 76 for 1002:7340 at bus 3).
- The VFCT image (62976 bytes) is byte-identical to the first 62976 bytes of `Desktop\Navi
  14.rom` (the rest of that 512 KiB file is zero). ROM checksum, `PCIR` 1002/7340, ATOM header
  subsystem 106B/020F and the string `113-D32206U1-019` all check out. The PowerPlay table is at
  image offset `0xCB98` (data table index 15), 1674 bytes, revisions 12/0/1, `table_size` 482.
- The live registry table (MorePowerTool, 32 W) equals the VFCT table except: OD caps 0-3 and 8
  set to 1, OD max clocks 1665 -> 1550 MHz, OD max curve voltages 1050 -> 950 mV, and
  `SocketPowerLimitAc[0]`/`Dc[0]` 50 -> 32. So MorePowerTool's power limit is exactly offsets 494
  and 510, and the registry format is the raw VBIOS table.
- `Desktop\5500m-unmodded.mpt` and `5500m-modded.mpt` are 1930 bytes: a 256-byte MorePowerTool
  file header followed by the 1674-byte table. The modded one sets the power limit to 100 W plus the
  OD edits above (it is not the table in the registry).
- **The "unmodded" .mpt is not pure stock:** it differs from the VFCT table at offset 590,
  `MaxVoltageGfx` 1050 mV, where the firmware's table has **950 mV**. The current registry table
  also has 950 there. So the "950 mV undervolt" of the measured runs matches stock
  `MaxVoltageGfx`; MorePowerTool only lowered the OverDrive voltage maxima. Use VFCT, not the .mpt,
  as the reference.
- ADL exports listed from the Mac's `atiadlxx.dll` (copied off, inspected locally): no table getter.

## Inferred, not yet verified

- `GetSystemFirmwareTable` for ACPI should work without elevation; only an elevated session was
  tried (gamebooster runs elevated anyway).
- That the Boot Camp driver honours a soft table whose OverDrive caps are stock (all 0). Every
  measured 32 W run had MorePowerTool's OD caps switched on. First-build check: after the GPU
  restart, HWiNFO's GPU PPT limit reads 32 W.
- That the stock-voltage run owed in the design draws about the same as the measured runs: with
  `MaxVoltageGfx` already 950 mV at stock, the only voltage difference left is the OD curve maxima,
  which matter only if someone uses Adrenalin's tuning.
- Other 5500M VBIOS versions: the gate blocks them; their offsets were not checked.

## Open risks

- A future Boot Camp firmware that drops VFCT would turn the fix off (safe failure, one line).
- An AMD driver update could change how the soft table is read; the size/revision checks catch a
  VBIOS change, not a driver change.
- The registry write is the only change; `uninstall` restores the previous value or deletes it, as
  in the design.
