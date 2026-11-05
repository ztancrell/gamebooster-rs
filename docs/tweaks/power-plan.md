# Power plan

**What it is.** gamebooster copies Windows' *High performance* plan into its own plan called
`gamebooster` (fixed GUID `a2921d59-9c60-439e-b61a-21c441a0df53`), changes the settings below for
both plugged in (AC) and battery (DC), keeps your current display brightness, and activates it.
Restore reactivates the plan you had before and deletes `gamebooster`. If High performance is
missing (some laptops hide it), Balanced is copied instead and the same values are written.

**Windows default.** Boot Camp installs run *Balanced*. The minimum processor state is 5 %, so
idle cores park and clock down; PCIe links to the GPU and SSD drop into power-saving states
(ASPM); and on battery the cooling policy is *passive* (slow the CPU before spinning up the fans).

**Why it helps.** A game's CPU and GPU work comes in bursts, one per frame. On Balanced, each
burst first has to wake cores and raise clocks, and PCIe links have to leave their power-saving
state. That delay shows up as frame-time spikes. Minimum processor state 100 % and ASPM off
remove it. Active cooling lets the fans react before the CPU throttles, which matters on iMacs and
MacBook Pros where Windows' fan behaviour is conservative. On Macs with AMD Radeon graphics, the
*ATI PowerPlay: maximize performance* setting keeps the GPU from downclocking between frames.
The cost: more heat, more fan noise, shorter battery life. That's why restore switches back.

## Exact settings

| Subgroup | Setting | Value |
|---|---|---|
| Processor `54533251-82be-4824-96c1-47b60b740d00` | Minimum processor state `893dee8e-2bef-41e0-89c6-b55d0929964c` | 100 |
| Processor | Maximum processor state `bc5038f7-23e0-4960-96da-33abaf5935ec` | 100 |
| Processor | System cooling policy `94d3a615-a899-4ac5-ae2b-e4d8f634367f` | 1 (active) |
| PCI Express `501a4d13-42af-4429-9fd1-a8218c268e20` | Link state power management `ee12f906-d277-404b-b6da-e5fa1a576df5` | 0 (off) |
| ATI graphics `f693fb01-e858-4f00-b20f-f30e12ac06d6` | ATI PowerPlay `191f65b5-d45c-4a4f-8aae-1ab8bfd980e6` | 1 (maximize performance; AMD only) |
| Hard disk `0012ee47-9041-4b5d-9b77-535fba8b1442` | Turn off after `6738e2c4-e8a5-4a42-b16a-e040e769756e` | 0 (never) |
| Wireless adapter `19cbb8fa-5279-450e-9fac-8a3d5fedd0c1` | Power saving mode `12bbebe6-58d6-4636-95bb-3217ef867c1a` | 0 (maximum performance) |
| Sleep `238c9fa8-0aad-41ed-83f4-97be242c8f20` | Sleep after `29f6c1db-86da-48c5-9fdb-f2b67b1f44da` | 0 (never) |
| Sleep | Hibernate after `9d7815a6-7ee4-497e-8888-515a05f02364` | 0 (never) |
| Buttons `4f971e89-eebd-4455-a8de-9e59040e7347` | Lid close action `5ca83367-6e45-459f-a27b-476b1d01c936` | 0 (do nothing) |
| None `fea3413e-7e05-4911-9a71-700331f1c294` | Require password on wake `0e796bdb-100d-47d6-a2d5-f7d2daa51f51` | 0 |
| Display `7516b95f-f776-4464-8c53-06167f40cc99` | Brightness `aded5e82-b909-4619-9949-f5d71dac0bcb` | your current AC / DC values |
| Display | Turn off after `3c0bc021-c8a8-4e07-a973-6b14cbcb2b7e` | 0 (never) |
| Display | Dim after `17aaa29b-8b43-4b94-aafe-35f64daaf1ee` | 0 (never) |
| Display | Adaptive brightness `fbd9aa66-9553-4097-ba44-ed6e9d65eab8` | 0 (off) |
| Display | unidentified hidden setting `90959d22-d6a1-49b9-af93-bce885ad335b` | 0 |
| Multimedia `9596fb26-9850-41fd-ac3e-f7c3c00afd4b` | When sharing media `03680956-93bc-4294-bba6-4e0f09bb717f` | 1 (prevent idle sleep) |
| Multimedia | When playing video `34c7b99f-9a6d-4b3c-8dc7-b6693b78cef4` | 0 (optimize quality) |

A setting your machine doesn't have (the ATI one on Intel or NVIDIA graphics, for example) is
printed as `skipped (not on this machine)`; that is expected.

## Try and undo by hand

In an administrator Command Prompt (`%P%` is our plan GUID):

```bat
powercfg /getactivescheme
rem note the GUID it prints: that's your plan to come back to

set P=a2921d59-9c60-439e-b61a-21c441a0df53
powercfg /duplicatescheme 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c %P%
powercfg /changename %P% "gamebooster"
powercfg /setacvalueindex %P% SUB_PROCESSOR PROCTHROTTLEMIN 100
powercfg /setdcvalueindex %P% SUB_PROCESSOR PROCTHROTTLEMIN 100
powercfg /setacvalueindex %P% SUB_PROCESSOR SYSCOOLPOL 1
powercfg /setacvalueindex %P% SUB_PCIEXPRESS ASPM 0
powercfg /setacvalueindex %P% f693fb01-e858-4f00-b20f-f30e12ac06d6 191f65b5-d45c-4a4f-8aae-1ab8bfd980e6 1
powercfg /setactive %P%
```

Undo: `powercfg /setactive <the GUID you noted>` then `powercfg /delete %P%`. If you lost the
GUID, Balanced is `381b4222-f694-41f0-9685-ff5bb260df2e`.

To see a hidden setting in Control Panel > Power Options > Advanced settings:
`powercfg /attributes SUB_PROCESSOR PROCTHROTTLEMIN -ATTRIB_HIDE`. `powercfg /qh %P%` lists every
setting of the plan, hidden ones included.

## Measure

- `powercfg /getactivescheme` shows `gamebooster` while boosted.
- HWiNFO: CPU clocks stay high between frames instead of dropping; the "core parking" or
  per-core usage graphs stop showing idle cores.
- CapFrameX in BF6 (see [README](README.md#how-to-measure-any-tweak)): expect better 1 % lows and
  fewer spikes. If temperatures hit the limit (about 100 °C on Mac Intel CPUs), clocks will drop
  anyway: a cooling pad or raising the Mac's back helps more than any setting.
