# Power plan

**What it is.** gamebooster copies Windows' *High performance* plan into its own plan called
`gamebooster` (fixed GUID `a2921d59-9c60-439e-b61a-21c441a0df53`), changes the settings below for
both plugged in (AC) and battery (DC), keeps your current display brightness and lid / power
button / sleep button actions, and activates it.
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
| Processor `54533251-82be-4824-96c1-47b60b740d00` | Minimum processor state `893dee8e-2bef-41e0-89c6-b55d0929964c` | `min_processor_state` from the config, default 100 ([low-power combo](#optional-low-minimum-state)) |
| Processor | Maximum processor state `bc5038f7-23e0-4960-96da-33abaf5935ec` | `max_processor_state` from the config, default 100 ([Turbo Boost off](#optional-turbo-boost-off) below 100) |
| Processor | Maximum processor frequency `75b0ae3f-bce0-45a7-8c89-c9611c25e100` | `max_cpu_mhz` from the config, MHz; only written when set ([frequency cap](#optional-frequency-cap)), otherwise High performance's 0 (no limit) |
| Processor | System cooling policy `94d3a615-a899-4ac5-ae2b-e4d8f634367f` | 1 (active) |
| PCI Express `501a4d13-42af-4429-9fd1-a8218c268e20` | Link state power management `ee12f906-d277-404b-b6da-e5fa1a576df5` | 0 (off) |
| ATI graphics `f693fb01-e858-4f00-b20f-f30e12ac06d6` | ATI PowerPlay `191f65b5-d45c-4a4f-8aae-1ab8bfd980e6` | 1 (maximize performance; AMD only) |
| Hard disk `0012ee47-9041-4b5d-9b77-535fba8b1442` | Turn off after `6738e2c4-e8a5-4a42-b16a-e040e769756e` | 0 (never) |
| Wireless adapter `19cbb8fa-5279-450e-9fac-8a3d5fedd0c1` | Power saving mode `12bbebe6-58d6-4636-95bb-3217ef867c1a` | 0 (maximum performance) |
| Sleep `238c9fa8-0aad-41ed-83f4-97be242c8f20` | Sleep after `29f6c1db-86da-48c5-9fdb-f2b67b1f44da` | 0 (never) |
| Sleep | Hibernate after `9d7815a6-7ee4-497e-8888-515a05f02364` | 0 (never) |
| Buttons `4f971e89-eebd-4455-a8de-9e59040e7347` | Lid close action `5ca83367-6e45-459f-a27b-476b1d01c936` | your current AC / DC values |
| Buttons | Power button action `7648efa3-dd9c-4e3e-b566-50f929386280` | your current AC / DC values |
| Buttons | Sleep button action `96996bc0-ad50-47ec-923b-6f41874dd9eb` | your current AC / DC values |
| None `fea3413e-7e05-4911-9a71-700331f1c294` | Require password on wake `0e796bdb-100d-47d6-a2d5-f7d2daa51f51` | 0 |
| Display `7516b95f-f776-4464-8c53-06167f40cc99` | Brightness `aded5e82-b909-4619-9949-f5d71dac0bcb` | your current AC / DC values |
| Display | Turn off after `3c0bc021-c8a8-4e07-a973-6b14cbcb2b7e` | 0 (never) |
| Display | Dim after `17aaa29b-8b43-4b94-aafe-35f64daaf1ee` | 0 (never) |
| Display | Adaptive brightness `fbd9aa66-9553-4097-ba44-ed6e9d65eab8` | 0 (off) |
| Display | unidentified hidden setting `90959d22-d6a1-49b9-af93-bce885ad335b` | 0 |
| Multimedia `9596fb26-9850-41fd-ac3e-f7c3c00afd4b` | When sharing media `03680956-93bc-4294-bba6-4e0f09bb717f` | 1 (prevent idle sleep) |
| Multimedia | When playing video `34c7b99f-9a6d-4b3c-8dc7-b6693b78cef4` | 0 (optimize quality) |

Superseded: the lid close action used to be forced to 0 (do nothing), as SGB does. Since
2026-11-05 the lid, power button and sleep button actions are copied from the plan you had
(`KEEP` in `src/power.rs`), so closing the lid or pressing a button does what it always does.

A setting your machine doesn't have (the ATI one on Intel or NVIDIA graphics, for example) is
printed as `skipped (not on this machine)`; that is expected. The same goes for the carried-over
settings: if the plan you had lacks the display brightness or a lid / button action, `boost` and
`--dry-run` print e.g. `skipped (not on this machine) keep current lid close action`. On Boot Camp
Macs all three lid / button actions show as skipped (Windows there only has "Start menu power
button" under Buttons).

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

## Optional: Turbo Boost off

**What it is.** Turbo Boost lets an Intel CPU run above its base clock (the target Mac's i7-9750H: 2.6 GHz base,
up to 4.5 GHz turbo; the i9-9880H: 2.3 / 4.8 GHz) while power and temperature allow. Windows
asks for turbo only when the *maximum processor state* is 100 %. At 99 % or lower it caps the
request at the base clock, so turbo is never used. That is why `max_processor_state = 99` turns
Turbo Boost off: one percent less on paper, but no turbo at all in practice. (On AMD CPUs the
effect is similar for Precision Boost, but this project is tested on Intel.)

**Default.** `max_processor_state = 100` (turbo on), the same as High performance and Balanced.
gamebooster writes the value for both AC and DC; `--dry-run` and `boost` print it and say
"Turbo Boost off" when it is below 100. Values outside 5 to 100 are rejected when the config loads.

**Trade-off.** Turbo gives the highest peak clocks, but this i9 reaches about 100 °C within
minutes, then throttles hard and unevenly, and clocks swing between turbo and far below base. With
turbo off the CPU runs cooler and at a steady base clock, the fans are quieter and the GPU (which
shares the cooling) keeps more headroom. Average FPS may drop a little in CPU-bound scenes; the
1 % lows often improve. Only measuring tells which wins on your machine and game.

**Try and undo by hand** (administrator Command Prompt, works on whatever plan is active):

```bat
powercfg /setacvalueindex SCHEME_CURRENT SUB_PROCESSOR PROCTHROTTLEMAX 99
powercfg /setdcvalueindex SCHEME_CURRENT SUB_PROCESSOR PROCTHROTTLEMAX 99
powercfg /setactive SCHEME_CURRENT
```

`/setactive` re-applies the plan so the change takes effect. Undo: the same three lines with
`100`. Check with `powercfg /query SCHEME_CURRENT SUB_PROCESSOR PROCTHROTTLEMAX` (0x63 = 99,
0x64 = 100).

**Measure.**

- HWiNFO sensors: with turbo off, core clocks sit at or below base instead of jumping to 4+ GHz;
  CPU package temperature should drop and the "Thermal Throttling" / "PROCHOT" flags should stay
  "No" during a session.
- CapFrameX in the same BF6 scene, 3 runs at 100 and 3 at 99 (change the config value, then
  `gamebooster restore` and `gamebooster boost`): compare the 1 % lows and the frame-time graph,
  not only the average FPS.

## Optional: low minimum state

**What it is.** `min_processor_state` (0 to 100, never above `max_processor_state`) replaces the
100 % minimum above. Unset keeps 100 %, as SGB does. A low minimum lets idle cores clock down and
park again, which cuts heat and power draw between frames.

**When to use it.** If `gamebooster status` warns that the CPU looks locked at its minimum clock
(about 800 MHz under load, usually BD PROCHOT on a Boot Camp MacBook Pro, see
[prochot.md](prochot.md)), the low-power combo draws the least power while boosted:

```toml
min_processor_state = 5
max_processor_state = 99   # Turbo Boost off
```

**Trade-off.** Clocks ramp up per burst again, so frame-time spikes can come back; this is for a
machine that overheats or hits PROCHOT, not for one that runs fine at 100 %.

**Try and undo by hand:** as for Turbo Boost above, with `PROCTHROTTLEMIN 5` (undo: `100`); check
with `powercfg /query SCHEME_CURRENT SUB_PROCESSOR PROCTHROTTLEMIN` (0x05 = 5).

## Optional: frequency cap

**What it is.** `max_cpu_mhz` writes the *Maximum processor frequency* setting (Processor subgroup,
`75b0ae3f-bce0-45a7-8c89-c9611c25e100`, in MHz, 0 = no limit) into the gamebooster plan, AC and DC.
The setting is hidden in Control Panel but writable with `powercfg`. Unlike
`max_processor_state = 99`, Turbo Boost stays on: the CPU may run above its base clock, but not
above the cap. It is the driver-free version of the fixed-multiplier workaround from
[prochot.md](prochot.md): users with a 2019 16" MacBook Pro under Boot Camp who held the CPU at
about 2.6 to 3.0 GHz stayed stable instead of boosting, overheating the VRM and crashing to
800 MHz ([research](prochot-research.md)).

**Default.** Unset (or `0`): not written, so the plan keeps High performance's 0 (no limit).
Values outside 800 to 6000 are rejected when the config loads. `--dry-run` and `boost` print
`maximum processor frequency 2800 MHz`; `bench` records it in its config snapshot (0 = no cap).
A machine without the setting prints `skipped (not on this machine)`. Restore needs nothing
extra: it deletes the whole gamebooster plan.

**With `max_processor_state`.** They don't conflict, both apply and the lower limit wins:
`max_processor_state = 99` turns turbo off (CPU at or below base, 2.6 GHz on the i7-9750H, 2.3 GHz on the i9-9880H), so a
cap above base then changes nothing. To test the cap, use it with `max_processor_state = 100`:

```toml
max_processor_state = 100   # Turbo allowed...
max_cpu_mhz = 2800          # ...up to 2.8 GHz; try 2600 to 3000
```

**Try and undo by hand** (administrator Command Prompt, on whatever plan is active):

```bat
powercfg /setacvalueindex SCHEME_CURRENT SUB_PROCESSOR 75b0ae3f-bce0-45a7-8c89-c9611c25e100 2800
powercfg /setdcvalueindex SCHEME_CURRENT SUB_PROCESSOR 75b0ae3f-bce0-45a7-8c89-c9611c25e100 2800
powercfg /setactive SCHEME_CURRENT
```

Undo: the same lines with `0`. Check with
`powercfg /qh SCHEME_CURRENT SUB_PROCESSOR 75b0ae3f-bce0-45a7-8c89-c9611c25e100`
(`/qh`, not `/query`: this is a hidden setting and `/query` leaves it out)
(0x00000af0 = 2800, 0x00000a28 = 2600, 0x00000bb8 = 3000).

**Measure.** Under ThrottleStop's TS Bench (or a game), the clock in Task Manager or ThrottleStop
should sit at or just under the cap, never 4+ GHz, and *Limits* should not show `PROCHOT` /
`EXT PROCHOT`. Then compare CapFrameX runs in the same BF6 scene against `max_processor_state = 99`.
Measured on the target Mac (2026-11-05): written correctly (checked with `/qh`), but no effect
observable: under TS Bench the CPU never got past ~2.29 GHz at ~22 W package power, below the
cap, with or without gamebooster's plan. See
[hardware-test.md](../hardware-test.md#results-on-the-target-mac).
