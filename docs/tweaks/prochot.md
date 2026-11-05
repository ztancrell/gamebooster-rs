# Stuck at 800 MHz (BD PROCHOT)

**Symptom.** The game drops to a slideshow and Task Manager shows the CPU at about 0.8 GHz even
under load. `gamebooster status` prints `CPU clock under load: 800 MHz` (or similar) and a warning.
Idle CPUs sit at 800 MHz normally; the problem is staying there while busy. `status` keeps one
thread busy for about a second while it measures, so a reading below 1000 MHz is the lock.

**Cause.** BD PROCHOT (bidirectional processor hot): something outside the CPU, on a MacBook Pro
usually the charger or battery circuitry or the voltage regulators (VRM), asserts the PROCHOT line,
and the CPU drops to its minimum clock until it is released. Under Boot Camp, Windows has no Apple
power management to keep that in check.

**Check.** In [ThrottleStop](https://www.techpowerup.com/download/techpowerup-throttlestop/), open
*Limits*: a red or highlighted `PROCHOT` / `EXT PROCHOT` entry while the clock is stuck confirms it.
The main window's `BD PROCHOT` box shows whether the CPU obeys the external signal.

## Fixes, in order

1. **Test on battery.** Unplug the charger and run `gamebooster status` again. If the clock is fine
   on battery, the charger or charging path is the trigger.
2. **Right-hand USB-C ports and the 96 W charger.** Charging from the left ports heats the side the
   CPU sits on; use a right-hand port and Apple's 96 W charger (a weaker one makes things worse).
3. **SMC reset.** Shut down, hold the power button 10 seconds (T2 Macs: see Apple's SMC reset
   steps), start again.
4. **macOS update.** SMC and T2 firmware only update from macOS: boot macOS and install updates.
5. **Low-power settings.** In `gamebooster.toml`: `min_processor_state = 5` and
   `max_processor_state = 99` (Turbo Boost off), then `gamebooster restore` and `gamebooster boost`.
   See [power plan](power-plan.md#optional-low-minimum-state).
   Or cap the clock instead of turning Turbo off: `max_cpu_mhz = 2800` (try 2600 to 3000) with
   `max_processor_state = 100`. This is the no-driver version of the "fixed multiplier" workaround
   (ThrottleStop holding about 2.6 to 3.0 GHz) that kept users stable; see
   [power plan](power-plan.md#optional-frequency-cap).
6. **Macs Fan Control.** Raise the fan curve so the machine stays cooler before the lock kicks in.

**Last resort: untick `BD PROCHOT` in ThrottleStop.** The CPU then ignores the external signal.
Its own temperature protection (TjMax, about 100 °C) still throttles and shuts it down when needed,
but you lose the early warning from the VRM and charging circuitry. Watch temperatures in HWiNFO,
and turn it back on if anything else misbehaves. gamebooster does not do this itself: it needs a
kernel driver, which this project never uses (WinRing0 is blocked by Defender, and test-signing
mode breaks BF6's anti-cheat).

Evidence, ranked workarounds and what gamebooster could automate: [prochot-research.md](prochot-research.md).
