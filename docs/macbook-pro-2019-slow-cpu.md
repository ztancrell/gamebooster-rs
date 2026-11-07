# MacBook Pro 2019 stuck slow in Windows (Boot Camp)

For a 2019 MacBook Pro with an Intel i9-9880H, i7-9750H or a similar Coffee Lake chip running Windows 10
or 11 under Boot Camp, when the CPU stays slow under load. Found and fixed on a real machine on
2026-11-05; the full notes are in [hardware-test.md](hardware-test.md#cpu-clock-and-power-under-load-2026-11-05).

## Does this match your Mac?

- Under full load (a game, ThrottleStop's TS Bench) the clock stays around 2.2 to 2.3 GHz
  (multiplier about 22), on battery and plugged in.
- Package power sits around 22 W (HWiNFO, "CPU Package Power"), well below the normal ~45 W.
- The CPU stays cool (about 60 °C) and the fans stay quiet.
- HWiNFO's "Performance Limit Reasons" shows nothing active. An SMC reset doesn't help.

If instead the clock drops to about **800 MHz**, that's a different problem: see
[Stuck at 800 MHz](#stuck-at-800-mhz-instead) below.

## The two causes

1. **"Maximum processor state" below 100 % in a Windows power plan.** Any value below 100 turns
   Turbo Boost off and caps the CPU below its base clock. On the tested Mac it was 80 %, on
   **several** plans (WinUtil's "Ultimate Performance" and "Balanced"), probably set by a tweak
   tool. So check every plan, not only the active one.
2. **ThrottleStop's "Disable Turbo" box was ticked.**

Either one alone is enough to hold the CPU down.

## Step 1: check the power plans

Open PowerShell as administrator (Start, type `powershell`, right-click, *Run as administrator*).

1. List the plans and their GUIDs (the long IDs). The active one has a `*`:

   ```
   powercfg /list
   ```

   Copy GUIDs from this output; don't guess them. (Balanced is always
   `381b4222-f694-41f0-9685-ff5bb260df2e`.)

2. Check the active plan's maximum processor state:

   ```
   powercfg /qh scheme_current sub_processor PROCTHROTTLEMAX
   ```

   Use `/qh`, not `/q`: it also shows hidden settings. Look at the last two lines,
   *Current AC Power Setting Index* and *Current DC Power Setting Index*. Both should be
   `0x00000064` (= 100). `0x00000050` (= 80) is the problem.

3. Check the other plans by putting their GUID in place of `scheme_current`:

   ```
   powercfg /qh 381b4222-f694-41f0-9685-ff5bb260df2e sub_processor PROCTHROTTLEMAX
   ```

## Step 2: set it back to 100 %

For the active plan:

```
powercfg /setacvalueindex scheme_current sub_processor PROCTHROTTLEMAX 100
powercfg /setdcvalueindex scheme_current sub_processor PROCTHROTTLEMAX 100
powercfg /setactive scheme_current
```

Repeat for every plan that showed less than 100: either switch to it first
(`powercfg /setactive <GUID>`) and run the three lines again, or replace `scheme_current` with
its GUID in the first two lines.

Without commands: press Win+R, type `powercfg.cpl`, Enter, then *Change plan settings* next to a
plan, *Change advanced power settings*, *Processor power management*, *Maximum processor state*:
set *On battery* and *Plugged in* to 100 %. Do it for each plan.

gamebooster users: `gamebooster status` warns when the active plan caps Maximum processor state
below 100 %, and prints the fix.

## Step 3: untick "Disable Turbo" in ThrottleStop

If you use ThrottleStop: in its main window, untick **Disable Turbo**, then click **Save**.
Leave **BD PROCHOT** ticked: that's a safety feature, not the problem.

## What you should see afterwards

Run ThrottleStop's TS Bench (or a game) and watch the clock:

- About 4.0 GHz on all cores under full load (the tested i9-9880H held 4050 MHz, measured with 2
  cores disabled by `numproc 12`, see [hardware-test.md](hardware-test.md)). Higher at light load
  (single-core maximum 4.8 GHz on the i9-9880H, 4.5 GHz on the i7-9750H).
- Package power up to about 85 W.
- Temperatures mostly 54 to 70 °C, with brief peaks to 100 °C.
- ThrottleStop's PROCHOT indicator may flicker red briefly at those peaks. That's normal thermal
  protection. Only a solid, constant red means trouble.

## Optional: full speed only when needed

"Ultimate Performance" keeps the CPU at full speed all the time (Minimum processor state 100 %),
which costs battery and heat. For speed on demand, use **Balanced** with:

- Maximum processor state 100 % (Step 2), and
- Minimum processor state 5 %:

  ```
  powercfg /qh scheme_current sub_processor PROCTHROTTLEMIN
  ```

  Both index lines should read `0x00000005`. To set it:
  `powercfg /setacvalueindex scheme_current sub_processor PROCTHROTTLEMIN 5`, the same with
  `/setdcvalueindex`, then `powercfg /setactive scheme_current`.

More on these settings: [power plan](tweaks/power-plan.md).

## Stuck at 800 MHz instead

A different, well-known problem on these Macs: the clock drops to about 800 MHz under load with
Turbo on. That's BD PROCHOT, signalled by the Mac's power controller, not by the settings above.
A mitigation to try, with no driver: cap the maximum frequency, for example at 3000 MHz, through
the power plan's *Maximum processor frequency* setting (`PROCFREQMAX`), or with gamebooster's
`max_cpu_mhz = 3000`. It may not help on every machine. The full story and other fixes:
[Stuck at 800 MHz (BD PROCHOT)](tweaks/prochot.md).
