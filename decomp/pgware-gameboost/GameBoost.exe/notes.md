# GameBoost.exe

Delphi VCL, PE32, 4,139,520 bytes, sha256 `746c100aaf1b752ec2d1501d40baa61a37f16afe92890adb31b3e62ff15cb5de`,
unsigned. Internal version 3.10.21.2024. Forms (`RCDATA`): `TGBMainForm`, `TGBSettingsForm` (the
progress dialog), `TGBAboutForm`; UI text in `RCDATA/ENGLISH`. Read with `objdump` + `decomp/gdis.py`.

Relevant imports: `RegSetValueExW`/`RegCreateKeyExW`, `ShellExecuteW`, `ExitWindowsEx`, `Sleep`.
**No** `SetPriorityClass`, affinity, power (`powrprof`), service-control, timer-resolution or GPU
APIs: everything it does is registry writes plus a reboot.

## The window

Four pickers (from the `TGBMainForm` DFM) and two buttons:

| Picker | Field | Items (index order) | Used by Optimize? |
|---|---|---|---|
| Operating System | `[ebx+0x460]` | Windows 11, 10, 8, 7, Vista, XP, Server 2012, 2008, 2003, 2000 | Yes: picks which "OS" tweaks apply (`0x6cf734` maps to an internal code) |
| Processor | `[ebx+0x464]` | Intel, AMD, VIA | **No**: only saved to `HKLM\Software\PGWARE\GameBoost\ProcessorType` |
| Internet Type | `[ebx+0x498]` | Cable/DSL/Wi-Fi, FiOS/Wi-Fi, Dial-up 56k, Satellite, phone tethering, LTE/4G/3G…, LAN, ISDN, T1/T3/OC3+ | Only "not Dial-up and not ISDN" vs the rest: the TCP block is skipped for those two; every other choice gives identical values |
| Speed Setting | trackbar `[ebx+0x470]+0x2d0` | Boost / **Maximum Boost** (trial: upsell box, opens the store) | Only `MaxConnectionsPerServer` / `MaxConnectionsPer1_0Server`: 4 (Boost) or 8 (Maximum) |

Nothing is detected automatically. The progress dialog's "Finding modem information", "Finding 3D
graphics device", "Finding hard drives" etc. are a fake progress loop: the
loop does random 1–3 million message-pump iterations plus three
500 ms sleeps per step, 33 steps on Optimize and 29 on Restore, so tens of seconds of animation.

## Optimize (`GoButton`, handler around 0x6ca6e0..0x6cb620)

1. Trial + "Maximum Boost": upsell message box; Yes opens `http://www.pgware.com/store/checkout/?product=gameboost`.
2. Saves the four picker values to `HKLM\Software\PGWARE\GameBoost\`.
3. Fake progress (above).
4. If `Restore` = 0 (first time): backs up 9 values into `HKLM\Software\PGWARE\GameBoost\` (value
   names reused). Sets `Restore` = 1.
5. Writes the values below. 6. "Restart Windows now?" (Yes → `ExitWindowsEx`).

Full machine-generated list: `python -I ../tweaks.py GameBoost.exe`. "Win10/11" column: applied
when Windows 10 or 11 is picked with a broadband internet type. "Backup": original saved and put back
by Restore; "delete": Restore deletes the value instead (original not saved).

| Value | Data written | Win10/11 | Restore | Windows 10/11 default / effect |
|---|---|---|---|---|
| HKCU `Control Panel\Desktop\WindowMetrics\MinAnimate` | `"1"` | yes | backup | `1` is the default (animations on); no change |
| HKLM `…\Memory Management\DisablePagingExecutive` | 0 | yes | backup | default 0; no change |
| HKLM `…\Memory Management\LargeSystemCache` | 0 | yes | backup | default 0 on client; no change |
| HKLM `…\PriorityControl\Win32PrioritySeparation` | **1** | yes | backup | default 2 (foreground quantum 3×). 1 = 2×: the foreground game gets *less* boost |
| HKLM `…\CrashControl\AutoReboot` | 1 | yes | backup | default 1; no change |
| HKCU `…\Internet Settings\MaxConnectionsPerServer`, `MaxConnectionsPer1_0Server` | 4 or 8 | yes | backup | WinINet only (IE-era HTTP connections per server); no effect on games |
| HKLM `Tcpip\Parameters\KeepAliveTime` / `KeepAliveInterval` | 7200000 / 1000 | yes | backup | both are the documented defaults; no change |
| HKCU `Control Panel\Desktop\SmoothScroll` | binary `00 00 00` | yes | delete | turns list smooth-scrolling off; cosmetic |
| HKCU `Internet Explorer\Main\Use StyleSheets` | `"Yes"` | yes | delete | IE only; no effect |
| HKCU `Explorer\AutoComplete\Append Completion` | `"Yes"` | yes | delete | inline autocomplete in Run/Explorer; not performance |
| HKLM `Control\FileSystem\ContigFileAllocSize` | 0x600 | yes | delete | NT4/2000-era FAT/NTFS hint; no known effect on Win10/11 |
| HKLM `Policies\Explorer\LinkResolveIgnoreLinkInfo` / `NoResolveSearch` | 0 / 1 | yes | delete | machine policy: broken shortcuts no longer search for their target; not performance |
| HKLM `…\Memory Management\IoPageLockLimit` | 0x1000 (4 KB) | yes | delete | ignored since Windows Server 2003; no effect |
| HKLM `…\CrashControl\SendAlert` | 1 | yes | delete | admin alert on crash; no effect |
| HKLM `Windows NT\CurrentVersion\Windows\NoPopUpsOnBoot` | 1 | yes | delete | legacy; no effect |
| HKLM `Windows NT\CurrentVersion\Windows\Error Mode` | 1 | yes | delete | **typo**: the real value is `ErrorMode` (no space); writes junk |
| HKLM `Services\Messenger\Start` | 4 | yes | delete | Messenger service doesn't exist since Vista: creates a stray service key with only `Start` |
| HKLM `Control\Session Manager\EnablePrefetcher` | 1 | yes (all OS but Server 2003) | delete | **wrong key**: Windows reads `EnablePrefetcher` under `…\Memory Management\PrefetchParameters`; this stray value does nothing |
| HKLM `Tcpip\Parameters\DefaultTTL` | 64 | yes | delete | default 128; harmless, no speed effect |
| HKLM `Tcpip\Parameters\EnablePMTUBHDetect` / `EnablePMTUDiscovery` / `SackOpts` | 0 / 1 / 1 | yes | delete | all defaults; no change |
| HKLM `Tcpip\Parameters\Tcp1323Opts`, `GlobalMaxTcpWindowSize`, `TcpWindowSize` | 3, 256960, 256960 | yes | delete | XP-era; ignored since Vista (receive-window auto-tuning) |
| HKLM `Tcpip\Parameters\TcpMaxDupAcks` | 2 | yes | delete | default 2; no change |
| HKLM `Policies\Microsoft\Windows\Psched\NonBestEffortLimit` | **50** | yes | delete | QoS reservable bandwidth policy; only matters for apps that make QoS reservations. Not the popular "0 %" myth: it sets 50 % |
| HKLM `Policies\Microsoft\Windows\Psched\MaxOutstandingSends` | 65535 | yes | delete | Psched policy; no gaming effect |
| HKLM `Session Manager\AutoChkTimeOut` | 0 | no (XP, Server 2003/2008/2012 only) | delete | skips the chkdsk countdown at boot |
| HKLM `Dfrg\BootOptimizeFunction\Enable` | `"Y"` | yes (Vista and later) | delete | `"Y"` is the default; Restore then *deletes* it |
| HKLM `Services\P3\Parameters\HackFlags` | 1 | yes (Vista and later) | delete | Pentium III processor driver (XP-era); creates a stray key |

## Restore (`RestoreButton`, around 0x6ccd0c..0x6cd9c0)

Confirmation box, then: the 9 backed-up values are written back (with Windows defaults as fallback
if the backup is missing) and the backup copies deleted; **every other value is deleted**, not
restored. Consequences: a value the user (or an admin policy) had set before is lost (e.g. any
existing `Psched` or `Policies\Explorer` policy, `Dfrg\BootOptimizeFunction\Enable`), and the stray
`Messenger` / `P3` keys stay (only the `Start` / `HackFlags` values go). Which values Restore deletes
depends on the OS and internet type picked *at Restore time*, not at Optimize time. Asks to reboot.

## Startup (around 0x6ca2c0 and 0x6dca80)

- Every launch of the main window starts `GameBoost.exe /online` (licence check, see
  [GameBoostRegister notes](../GameBoostRegister.exe/notes.md)).
- If more than 45 days since `UpdatedDate`, starts `GameBoost.exe /checkupdate`, which GETs
  `http://www.pgware.com/downloads/update/?product=gameboost&version=3.10.21.2024` (User-Agent
  `Mozilla/4.0 (compatible; PGWARE)`) and expects `PGWARE|0/1/2`; an update offer launches
  `GameBoostUpdate.exe`.

## Not present (despite the product page)

"CPU prioritization of full-screen applications … more cores", "software based over-clocking of
memory, CPU and internet": no code for any of it. The only CPU-scheduling change is
`Win32PrioritySeparation` = 1, which slightly *reduces* foreground priority.
