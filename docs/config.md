# Config

`gamebooster init` writes [gamebooster.example.toml](../gamebooster.example.toml) (built into the exe)
as `gamebooster.toml` next to `gamebooster.exe`; `gamebooster install` (or a double-click) does the
same in `C:\gamebooster-rs` (see [Install](install.md#student-install-double-click)). Keys:

| Key | Default | Meaning |
|---|---|---|
| `power_plan` | `true` | Use the gamebooster power plan. |
| `max_processor_state` | `100` | Maximum processor state (%) in that plan, 5 to 100. `99` turns Intel Turbo Boost off (less heat and throttling, lower peak clocks); see [power plan](tweaks/power-plan.md#optional-turbo-boost-off). |
| `min_processor_state` | unset (= `100`) | Minimum processor state (%) in that plan, 0 to 100, not above `max_processor_state`. `5` with `max_processor_state = 99` is the low-power combo for a Mac stuck at 800 MHz; see [power plan](tweaks/power-plan.md#optional-low-minimum-state). |
| `max_cpu_mhz` | unset (no cap) | Maximum processor frequency (MHz) in that plan, 800 to 6000; `0` or unset = no cap. Turbo stays on up to the cap; with `max_processor_state` below 100 too, the lower limit wins. Try 2600 to 3000 on a Mac that hits BD PROCHOT. A running ThrottleStop overrides it (use its own Turbo / clock settings instead); without ThrottleStop it works (verified on the target Mac 2026-11-07); see [power plan](tweaks/power-plan.md#optional-frequency-cap). `install` on a 16" MacBook Pro 2019 with the 5500M sets `1700` (fallback `1600` if BF6 still hits the power budget); like every boost setting it applies only while boosted, and `auto` boosts only while a `[[game]]` runs (BF6 in the example), so the CPU is at full speed otherwise. |
| `game_mode` | `true` | Make sure Game Mode is on. |
| `accessibility_shortcuts_off` | `true` | Turn off the Sticky / Filter / Toggle Keys hotkeys while boosted; see [accessibility shortcuts](tweaks/accessibility-shortcuts.md). |
| `game_power_throttling_off` | `true` | Opt each running `[[game]]` out of Windows power throttling (EcoQoS) while boosted; restore hands it back to Windows. See [game priority](tweaks/priority.md#power-throttling-off). |
| `ram_trim` | `false` | Trim other processes' RAM once. |
| `game_priority_method` | `"process"` | How game priority is set: `"process"` (on the running game) or `"registry"` (IFEO `PerfOptions\CpuPriorityClass`, applied by Windows at the next game start; for anti-cheats that block the process way; no `realtime`). See [game priority](tweaks/priority.md#registry-method). |
| `lower_others` | `false` | Lower your other Normal-priority processes during the boost; see [game priority](tweaks/priority.md#lower-other-processes). |
| `lower_others_io` | `true` | With `lower_others`: also set each lowered process's disk (I/O) priority to Low (only ones at Normal); restore puts it back. Processes that refuse (anti-cheat protected) are printed and skipped. Try by feel; see [game priority](tweaks/priority.md#io-priority). |
| `lower_others_mode` | `"once"` | `"once"`: boost lowers them all. `"dynamic"`: boost lowers nothing; `boost --watch` (or `auto`) restrains only busy ones, ProBalance-style (Below normal only). See [Watch mode](watch.md). |
| `lower_others_class` | `"below_normal"` | `"below_normal"` or `"idle"`. Idle can starve voice chat and streams. Once mode only; dynamic mode always uses Below normal. |
| `lower_others_exclude` | `[]` | Extra image names to leave alone (case-insensitive), on top of the built-in list. |
| `lower_others_exclude_publishers` | `[]` | Extra publishers to leave alone: case-insensitive substrings of the exe's CompanyName (file Properties > Details), on top of the built-in list. |
| `priority_separation` | `true` | Set `Win32PrioritySeparation` to `0x2A` while boosted (restore puts back the old value or deletes it); `false` leaves it alone. Try by feel; see [game priority](tweaks/priority.md#win32priorityseparation). |
| `fastest_dns` | `false` | On boost, time DNS lookups against the current server and 1.1.1.1, 8.8.8.8, 9.9.9.9; switch the active adapter's IPv4 DNS to a public one only if it is more than 10 ms and 20 % faster. See [network](tweaks/network.md#fastest-dns). |
| `auto_restore_delay_secs` | `30` | `gamebooster auto`: seconds to wait after the last game exits before restoring; a game starting within that time keeps the boost. `0` = restore at once. |
| `ping_hosts` | 12 AWS regional endpoints | Hosts `gamebooster ping` measures (TCP port 443). |
| `ping_samples` | `10` | Probes per host for `ping` and per hop for `ping --route` (1 to 100). |
| `services` | `[]` | Service key names to stop (`sc query` names, e.g. `"WSearch"`). The example file has a commented-out suggested list from SGB, pruned; see [services](tweaks/services.md#suggested-list-from-sgb-pruned). |
| `close` | `[]` | Process image names to close (e.g. `"OneDrive.exe"`). |
| `[[game]]` `exe`, `priority` | example: `bf6.exe`, `high` | Games to raise; priority is `realtime`, `high`, `above_normal`, `normal`, `below_normal` or `low`. |

Unknown keys and typos are errors. Protected processes (Windows core, `explorer.exe`,
`taskmgr.exe`, Steam, the EA app, Epic, Defender, Symantec, gamebooster itself) are refused in
`close` and as games. The BF6 executable name `bf6.exe` is the expected one but **unverified**:
check Task Manager > Details while the game runs.
