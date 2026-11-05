# BD PROCHOT on a 2019 MacBook Pro: research notes

Research done 2026-11-05 for the "stuck at 800 MHz" lock under Boot Camp. The user-facing fix list
is [prochot.md](prochot.md); this page collects the evidence behind it, ranks the workarounds and
says what gamebooster could do on its own without a kernel driver. Almost all evidence is user
forum reports from 2019 to 2024; nothing here comes from Apple or Intel engineering, and no
source documents a permanent Windows-side fix for this model.

## What the target machine shows

ThrottleStop's *Limits* window shows `PROCHOT` red during TS Bench, while a one-second
single-thread load on Balanced gave about 1.7 GHz in Task Manager. So the lock is triggered by
sustained load rather than being stuck all the time, which points at a real trigger (heat near
the VRM or charging circuitry, or the charger running out of headroom) more than at a dead sensor.
1.7 GHz on one thread is also well below what a 9th-gen H-series chip should do, so something is
already limiting it before the full 800 MHz lock.

Superseded (2026-11-05): the "`PROCHOT` red" reading came from ThrottleStop's main-window label
`PROCHOT 100°C` (right of the *Limits* button), which shows TjMax and is not a throttling
indicator. The real *Limits* window stayed empty during TS Bench, so the 800 MHz lock has not been
observed on this Mac; the CPU instead sits at ~2.29 GHz and ~22 W package power. See
[hardware-test.md](../hardware-test.md#results-on-the-target-mac).

## Root causes reported for this model

**Load-triggered PROCHOT tied to the dGPU and VRM.** On 16-inch models with the Radeon Pro
5300M/5500M, Boot Camp users report BD PROCHOT going red after minutes of gaming, the CPU
falling to 800 MHz, and the GPU dropping at the same time; several think the GPU or VRM side is
the trigger and that PROCHOT is the symptom
([bootcampdrivers thread 699](https://bootcampdrivers.freeforums.net/thread/699/macbook-16inch-throttle-technical-details),
6 users; [thread 702](https://bootcampdrivers.freeforums.net/thread/702/solved-thermal-throttling-2019-5500m),
where one user saw 800 MHz with CPU temperatures under 70 °C). MacRumors users with external
monitors describe the same 0.8 GHz drop and attribute it to overheating VRM chips
([MacRumors "16-inch is hot & noisy with an external monitor"](https://forums.macrumors.com/threads/2019-16-is-hot-noisy-with-an-external-monitor.2211747/page-202)).

**dGPU power with an external monitor.** With an external display the Radeon idles at about
18 to 20 W on the 5300M/5500M, and one poster says the draw jumps to 20 W in Boot Camp as well
([AMD community](https://community.amd.com:443/t5/drivers-software/macbook-pro-16-quot-is-hot-noisy-with-an-external-monitor/m-p/328779),
[Apple Community](https://discussions.apple.com/thread/251329522),
[MacRumors](https://spy.macrumors.com/threads/2019-16-is-hot-noisy-with-an-external-monitor.2211747/page-120)).
That heat and power comes out of the same budget as the CPU. Under Boot Camp the Intel iGPU is
hidden and Windows runs only on the dGPU
([MacRumors](https://forums.macrumors.com/threads/windows-10-on-macbook-pro-16.2214033),
[MacRumors 2010](https://www.macrumors.com/2010/04/16/new-15-inch-and-17-inch-macbook-pro-restricted-to-discrete-graphics-in-windows)),
so "force the iGPU" is not an option on Windows.

**Charger headroom.** The 16-inch ships with a 96 W adapter (the 15-inch with 87 W)
([MacRumors](https://www.macrumors.com/2019/11/13/16-inch-macbook-pro-96w-usb-c-power-adapter/)).
Users measure the machine drawing about 110 to 120 W while gaming in Boot Camp, so the battery
drains even on the 96 W charger
([MacRumors](https://forums.macrumors.com/threads/16-i9-gaming-in-bootcamp-drains-battery-plugged-in.2280496));
Apple's 96 W adapter tops out around 92 W and 85/87 W chargers or docks drain faster
([MacRumors](https://forums.macrumors.com/threads/96w-apple-charger-delivers-only-92w-draining-battery.2223498),
[MacRumors](https://forums.macrumors.com/threads/mbp-16-compatible-docks-with-96w-100w-power.2211247/page-9)).
Docks, hubs and old USB-C cables can stop the full wattage reaching the Mac
([Apple Community](https://discussions.apple.com/thread/252945033),
[Apple Community](https://discussions.apple.com/thread/251318567)). No source links a weak
charger directly to the 800 MHz lock; it is a plausible trigger, not a proven one.

**Left-side charging ports.** On USB-C MacBook Pros, charging from the left ports heats the
"Thunderbolt Left Proximity" sensor and macOS throttles a few minutes later; moving the charger to
the right side stops it
([Tom's Guide](https://www.tomsguide.com/news/macbook-pro-charging-bug-kernel_task),
[pxlnv / mjtsai](https://pxlnv.com/linklog/macbook-pro-thunderbolt-charging/)). Those reports are
from macOS (kernel_task); nobody has measured the same thing under Windows, but the sensor and
SMC are the same hardware.

**Windows gets less power management than macOS.** Notebookcheck measured the 2019 15-inch
8 to 10 % slower under Boot Camp and suspected macOS-only undervolting or power tuning
([Notebookcheck](https://www.notebookcheck.net/Apple-may-be-factory-undervolting-2019-MacBook-Pro-15-to-help-with-CPU-throttling.423784.0.html)).
A forum user makes the same argument (macOS: cooling plus undervolt; Windows: stock voltage)
([thread 702](https://bootcampdrivers.freeforums.net/thread/702/solved-thermal-throttling-2019-5500m)).

**Firmware.** Apple fixed the 2018 MacBook Pro throttling with a macOS supplemental update that
corrected a missing key in the firmware for thermal management
([Tom's Guide](https://www.tomsguide.com/us/macbook-pro-throttling-bug-fix,news-27668.html)).
Firmware (SMC/T2) only updates from macOS. One 2018 15-inch Boot Camp user with PROCHOT-like
throttling that ThrottleStop's BD PROCHOT box did not cure reports that a clean macOS reinstall
(not from Time Machine) made it go away in Windows too
([Apple Community](https://discussions.apple.com/thread/250954082), 1 report).

**Dust and old paste.** One iMac owner stuck at 800 MHz in Windows fixed it by cleaning the vents
([MacRumors](https://forums.macrumors.com/threads/bootcamp-on-mbp-processor-stuck-at-base-frequency.2184389/));
one 16-inch owner fixed GPU/CPU throttling with new paste, thermal pads and a CPU cap
([thread 702](https://bootcampdrivers.freeforums.net/thread/702/solved-thermal-throttling-2019-5500m)).

**Battery health.** No source for this model ties a worn battery to PROCHOT. Not ruled out, not
supported.

**Boot Camp driver versions.** Users tried several AMD driver versions with mixed results on GPU
throttling, nothing decisive for the CPU lock
([thread 699](https://bootcampdrivers.freeforums.net/thread/699/macbook-16inch-throttle-technical-details)).
No source shows Apple's Boot Camp drivers include Intel DPTF (Dynamic Platform and Thermal
Framework); search found DPTF only in PC vendors' packages (e.g.
[Lenovo](https://support.lenovo.com/gb/en/downloads/DS502017)). There are no reports of
installing DPTF on a Mac, and it would need ACPI devices Apple's firmware may not expose:
don't try it.

## Workarounds, ranked by evidence and safety

1. **Charge from a right-side port, with Apple's 96 W charger plugged straight into the Mac,
   no dock or hub, original cable.** Safe and free. Evidence: strong for the left-port heat
   effect on macOS (widely reproduced, see above), moderate for charger headroom (several
   MacRumors measurements), none measured under Windows for the 800 MHz lock specifically.
2. **Test on battery and without an external monitor.** Safe diagnostic. Separates "charger /
   charging circuit" from "dGPU/VRM heat". No direct reports for this model, but it isolates the
   two best-supported triggers above.
3. **Cap the load so the trigger never fires.** Users who held a fixed CPU multiplier
   (26 to 30, about 2.6 to 3 GHz) kept PROCHOT away and got stable performance
   ([thread 699](https://bootcampdrivers.freeforums.net/thread/699/macbook-16inch-throttle-technical-details),
   2 users; [thread 702](https://bootcampdrivers.freeforums.net/thread/702/solved-thermal-throttling-2019-5500m),
   1 user). Disabling Turbo outright did *not* stop it for one user (CPU still fell to
   1.3 to 1.5 GHz). The driver-free version of this is the power plan's *Maximum processor
   frequency* (`75b0ae3f-bce0-45a7-8c89-c9611c25e100`, in MHz) or *Maximum processor state*
   below 100 % (turns Turbo off; what our `max_processor_state = 99` does). Nobody reports these
   for this model, but they act on the same thing the multiplier cap does. Safe.
4. **Limit the dGPU's power.** One user found a 42 W GPU power limit (written with
   MorePowerTool) "stabilizes" the machine without any hardware mod
   ([thread 702](https://bootcampdrivers.freeforums.net/thread/702/solved-thermal-throttling-2019-5500m));
   a guide combines it with raised minimum GPU clocks
   ([thread 887](https://bootcampdrivers.freeforums.net/thread/887/guide-throttling-mbp-5500m-5600m),
   1 user, black screens if overdone, no MorePowerTool support for the 5300M). MorePowerTool
   writes a soft PowerPlay table to the registry, so no extra driver, but a bad table means a
   black screen and Safe Mode. A frame cap (AMD Chill / in-game limit) is the safe version of
   the same idea and helped one user in some games
   ([thread 699](https://bootcampdrivers.freeforums.net/thread/699/macbook-16inch-throttle-technical-details)).
5. **SMC reset (T2 steps) and macOS/firmware updates.** Safe. Apple's T2 notebook procedure:
   shut down, hold left Control + left Option + right Shift for 7 s, add the power button for
   another 7 s, release, wait, power on
   ([Apple](https://support.apple.com/en-us/102605)). Evidence weak: helped once in thread 699;
   a clean macOS reinstall helped one 2018 user; the 2018 fix shows firmware can matter.
6. **Cooling: Macs Fan Control, raise the back, clean vents; re-paste as a repair.** Safe apart
   from opening the case. Evidence: 1 report each for dust and paste (above).
7. **ThrottleStop, untick BD PROCHOT** (clears bit 0 of MSR 0x1FC,
   [fralapo README](https://github.com/fralapo/Disable-BD-PROCHOT-on-LINUX)). The baseline
   everybody uses; it worked on a MacBook Air with a bad sensor
   ([larry1301](https://larry1301.wordpress.com/2015/12/14/macbook-cpu-throttling-motherboard-thermal-sensor-and-bd-prochot-msr/))
   and is part of the 16-inch guide in thread 887. But on Macs where the trigger is real load it
   often does little: a 2018 15-inch user saw no effect
   ([Apple Community](https://discussions.apple.com/thread/250954082)) and an external-monitor
   user reported "little effect" ([MacRumors](https://forums.macrumors.com/threads/2019-16-is-hot-noisy-with-an-external-monitor.2211747/page-202)),
   because the machine is also hitting power limits. It resets on every boot and after sleep
   (S3 sets the bit again; [DisablePROCHOT README](https://github.com/arter97/DisablePROCHOT)),
   so people start ThrottleStop from Task Scheduler. QuickCPU is no substitute on a 16-inch: it
   kept the CPU at 800 MHz with PROCHOT disabled
   ([thread 887](https://bootcampdrivers.freeforums.net/thread/887/guide-throttling-mbp-5500m-5600m))
   or pinned it at base clock
   ([egpu.io](https://egpu.io/forums/pc-setup/download-quickcpu-instead-of-throttlestop/)).
   Risks below.
8. **Not available or not worth it:**
   - *Intel XTU:* no source shows a BD PROCHOT control in XTU; even in Intel's own forum the
     user who cleared it did so in ThrottleStop
     ([Intel community](https://community.intel.com/t5/Processors/Disabling-Thermal-Throttling/m-p/1238985)).
     Intel support says XTU targets unlocked K/X
     CPUs and laptop support was dropped around 2020-21
     ([Intel community](https://community.intel.com/t5/Processors/Intel-Extreme-Tuning-Ultility/m-p/1436349),
     [HP community](https://h30434.www3.hp.com/t5/Notebook-Software-and-How-To-Questions/Why-is-this-program-not-available-on-my-laptop-Intel-Extreme/m-p/8504601)).
     ThrottleStop's author says no other software he knows of shows or disables BD PROCHOT
     ([forum mirror](https://www.nbrchive.net/xfa/windows-os-and-software.15/After%20using%20ThrottleStop,%20post%20restart%20is%20locked%20to%20800%20MHz%20(0,8%20GHz))).
   - *Undervolting:* voltage controls are greyed out on recent MacBook Pros, part of the
     post-Plundervolt lockdown; the only unlock is a firmware ROM patch tested on 2014 models
     ([egpu.io](https://egpu.io/forums/laptop-computing/undervolting-MacBook-pro-15-2017/),
     [ultrabookreview](https://ultrabookreview.com/37095-dells-disabling-undervolting-on-their-laptops-heres-how-to-re-enable-it),
     [UndervoltPatchMacBook](https://github.noahele.workers.dev/IsmaelHG/UndervoltPatchMacBook)).
   - *Pre-boot EFI tool* ([DisablePROCHOT](https://github.com/arter97/DisablePROCHOT)): clears
     the bit before Windows with no Windows driver, but needs rEFInd or Clover. The T2 at Full
     or Medium Security only boots Apple- or Microsoft-signed code
     ([Apple](https://support.apple.com/102522)), and BF6 refuses to start without Secure Boot
     ([TechSpot](https://www.techspot.com/news/108925-battlefield-6-anti-cheat-system-requires-secure-boot.html)).
     Dead end for this user.
   - *Processor performance boost mode / EPP / increase-decrease policy and registry PPM
     values:* these change how eagerly Windows asks for boost, they can't override a hardware
     limit ([Digital Citizen](https://www.digitalcitizen.life/hidden-windows-11-setting-lets-you-fine-tune-cpu-boost-behavior/)).
     No report of them helping with PROCHOT. Boost mode "Disabled" equals Turbo off, already
     covered by item 3.
   - *Windows power slider / Energy saver:* lowers performance across the board; no reports for
     this problem.

## Risks

**Disabling BD PROCHOT.** The CPU keeps its own TjMax throttle and shutdown, so it protects
itself ([ThrottleStop author via forum mirror](https://www.nbrchive.net/xfa/windows-os-and-software.15/After%20using%20ThrottleStop,%20post%20restart%20is%20locked%20to%20800%20MHz%20(0,8%20GHz)),
[Tom's Hardware](https://forums.tomshardware.com/threads/bd-prochot-throttle-is-always-on-even-though-temps-are-low.3695109/)).
What you lose is the brake for everything *else*: VRM, charging circuit, GPU. "Disabling BD
PROCHOT removes one layer of thermal defense ... you now have no hardware-level brake"
([fralapo README](https://github.com/fralapo/Disable-BD-PROCHOT-on-LINUX)). On this machine the
evidence says the signal is real (it fires under sustained load, together with GPU throttling),
which is exactly the case where ignoring it is least safe. One 16-inch user going this way had the
laptop put itself to sleep at 85 °C
([thread 887](https://bootcampdrivers.freeforums.net/thread/887/guide-throttling-mbp-5500m-5600m)).

**ThrottleStop's driver and anti-cheat.** ThrottleStop.sys has a public CVE (CVE-2025-7771:
IOCTLs giving arbitrary physical memory read/write), is used by ransomware to kill antivirus
([NVD](https://nvd.nist.gov/vuln/detail/CVE-2025-7771),
[Tenable](https://www.tenable.com/cve/CVE-2025-7771),
[LOLDrivers](https://www.loldrivers.io/drivers/6e0786f5-2168-40a8-a068-e261c4eb10e7/)), and one
case study says versions up to 9.6 were added to Microsoft's vulnerable driver blocklist in 2025
([KernelSight](https://splintersfury.github.io/KernelSight/case-studies/CVE-2025-7771)); not
confirmed against Microsoft's own list. That list is on by default since Windows 11 22H2 and is
enforced whenever Memory Integrity (HVCI) is on
([Microsoft](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/microsoft-recommended-driver-block-rules)).
No source says EA Javelin blocks ThrottleStop today. But Javelin blocks drivers EA considers
vulnerable (EVGA Precision/FlowControl was blocked for exactly that reason,
[EA forum](https://forums.ea.com/discussions/fc-26-technical-issues-en/ea-javelin-anticheat-blocks-evga-graphic-card/13598969);
other monitoring drivers trip error 119,
[Steam](https://steamcommunity.com/app/2807960/discussions/0/694246174784256997/)), its denylist
is not exhaustive ([EA help](https://help.ea.com/en/articles/platforms/pc-ea-anticheat/)), and
EA says it keeps moving previously allowed tools onto the prohibited list
([EA Season 4 update](https://www.ea.com/games/battlefield/battlefield-6/news/battlefield-6-anticheat-update-season-4)).
Riot Vanguard already blocks ThrottleStop's older RwDrv.sys
([Tom's Hardware](https://forums.tomshardware.com/threads/unable-to-turn-on-disable-and-lock-turbo-power-limits-with-throttlestop.3612268/)).
Practical rule: a block shows up as an error at launch, not a ban, as far as reports go, but
treat any MSR-capable driver (ThrottleStop, RW-Everything, old WinRing0 tools) as something that
can stop BF6 starting after any Javelin update. HWiNFO is listed by a third-party guide as a
common error-119 trigger ([Tenorshare](https://4ddig.tenorshare.com/windows-fix/ea-anticheat-encountered-an-error.html)),
unconfirmed by EA.

## What gamebooster could automate (no driver of our own)

- **Better lock detection.** `status` already reads PDH `Processor Frequency` and `% Processor
  Performance`. The same `Processor Information` counter set has `% Performance Limit` and
  `Performance Limit Flags` ("reasons why the processor performance was limited")
  ([wutils](https://wutils.com/wmi/root/cimv2/win32_perfrawdata_counters_processorinformation/),
  [windows_exporter](https://github.com/prometheus-community/windows_exporter/issues/1313)).
  The bit meanings aren't documented publicly (the driver-side reasons are in
  [PEP_PPM_PERF_CONSTRAINTS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/pep_x/ns-pep_x-_pep_ppm_query_perf_constraints)),
  so print them raw next to the clock and learn from this Mac. Running the load for 10 to 30 s
  instead of 1 s would catch the load-triggered lock that the 1 s test misses here. PROCHOT
  itself (MSR 0x19C/0x1FC) can't be read without a driver.
- **Charger check.** `Windows.Devices.Power.Battery.AggregateBattery.GetReport()` gives
  `ChargeRateInMilliwatts`, `DesignCapacityInMilliwattHours` and
  `FullChargeCapacityInMilliwattHours`
  ([Microsoft](https://learn.microsoft.com/en-us/uwp/api/windows.devices.power.batteryreport)).
  With `GetSystemPowerStatus` saying AC is online, a negative charge rate under load means the
  charger can't keep up: tell the user to check the 96 W charger, the cable, the right-side port
  and no dock. Full-charge / design capacity gives battery wear for free. Windows exposes no
  adapter wattage or port side.
- **Power-plan cap as a "Mac safe mode".** Already possible: `max_processor_state = 99`. Adding
  *Maximum processor frequency* (`PROCFREQMAX`, `75b0ae3f-bce0-45a7-8c89-c9611c25e100`, e.g.
  2800 MHz) would be the driver-free equivalent of the ThrottleStop multiplier cap that worked
  for users; it is an ordinary powercfg setting we can write and restore like the others. Needs a
  test on this Mac (does the cap hold, does PROCHOT stay away).
  Superseded: built as `max_cpu_mhz`, see [power-plan.md](power-plan.md#optional-frequency-cap).
- **Frame cap.** Recommend an in-game or AMD Chill cap in the docs; cheaper than any tweak.
- **Not automatable:** clearing BD PROCHOT, undervolting, GPU soft PowerPlay tables (risk of a
  black screen, not something a booster should write), SMC reset, firmware updates.

## What to try next on your Mac

1. Charger in a right-hand port, Apple 96 W brick, its own cable, no hub; external monitor
   unplugged. Run TS Bench for 10 minutes and watch *Limits*.
2. Same test on battery (at least 50 % charge). If PROCHOT stays off, it's the charging side.
3. In Task Manager, watch whether the battery percentage falls while plugged in during a game.
4. Set `max_processor_state = 99` in `gamebooster.toml`, `gamebooster restore`,
   `gamebooster boost`, and repeat step 1. Then try an in-game frame cap (e.g. 60).
5. Do the T2 SMC reset above; boot macOS once and install every update.
6. Install Macs Fan Control, set a more aggressive curve, raise the back of the Mac.
7. Only then, and only for a test session without BF6 running, untick BD PROCHOT in
   ThrottleStop with HWiNFO open; if the clock still falls, the limit is power, not PROCHOT, and
   leaving it unticked buys nothing.
