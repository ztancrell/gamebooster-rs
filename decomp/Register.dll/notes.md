# Register.dll notes (licensing)

Static only. Analysed copy: `sgb-extract/carved/pe_030060cd_1492168.bin` (1,492,168 bytes; sha256
prefix `2e84aab2d8e73f73`). Method: exports via `objdump -p`, `strings`, Ghidra (REA) for one
function. High-level only: no key algorithm was studied or recorded, on purpose.

## Exports
`ActiveApp`, `ActiveTrial`, `CheckLicenseLocatin`, `CheckTrialInstalled`, `ClientCheckKey`,
`ExtendOfTimes`, `ExtendTrialDays`, `GetSurplusDays`, `ValidateThreadLicense`.

## Model (high level)
- **Online.** Activation and periodic re-validation POST `code=…&cpu=…&disk=…[&ck=1]` to
  `https://smgblicense.itopupdate.com/check.php`. Results: valid / `Invalid` / `Expried` / `OverSeat`
  (too many PCs on one code). Backup hosts come from the same S3/GCS "AntiBan" list as the stats.
- **Machine fingerprint**: CPU info + the serial number of `\\.\PHYSICALDRIVE0` (WMI
  `Win32_PhysicalMedia.SerialNumber`, with SMART/IOCTL code as fallback). Sent with the code.
- **Local cache**: `License\license.dat` / `App_.dat` / `appscription.dat` with fields `FingerPrint`,
  `LicenseType`, `ExpDate`, `Expried`, `OverSeat`, `LastValidate`, `LastServerDate`,
  `LastCheckTime`, `LeftDays`; log `Register.log`.
- **Trial tracking hidden in the file system**: trial/extension counters (`Extend`, `ExtendCount`,
  `LocalIOTFlag`) are written to `desktop.ini` `[.ShellClassInfo]` sections of folders named with
  one of ~100 hard-coded CLSIDs (e.g. `{E0224FF9-7AE3-4F9E-991A-2F004F7E3952}`), so uninstalling does
  not reset the trial.
- **Rewrites the hosts file** (FUN 0x522b40, called from 0x527d14 and 0x5286bc in a loop over the
  license server host list, remove flag = 1): reads `%SystemRoot%\System32\drivers\etc\hosts`, and
  for every non-comment line that mentions a license host, deletes the line, clears the file's
  read-only/hidden attributes, saves it, and runs `cmd.exe /k ipconfig /flushdns &Exit`. This is an
  anti-piracy measure (undoing a hosts-file block of the license server), done silently to a system file.
- `CheckIsRUEnv`: separate behaviour for Russia (not traced).

## Free vs paid (from SgbMain UI strings)
Three tiers: Free, VIP (legacy/one-off "Expiration date: Never (V6)"), and SVIP (time-limited
subscription). Paid tiers unlock "Max Mode" GPU overclock level 2, and above all the network
accelerator (SgbAccelerate.dll VPN-style tunnel to vendor servers); free users get a timed balance
("Balance: %s h %1s min", "Your game will disconnect when time runs out"). Local Boost (services,
processes, RAM, power plan) is not gated by these strings.
