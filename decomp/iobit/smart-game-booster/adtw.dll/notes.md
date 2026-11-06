# adtw.dll

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_0d35f472_569400.bin`. Delphi, version info "Smart Game Booster adtw".

Exports `HasDivert`, `RemoveDivert`. Imports include `EnumDeviceDrivers`/`GetDeviceDriverBaseName*`.
Strings: `goodbyedpi.exe`, `winws.exe` (zapret), `WinDivert32.sys`, `WinDivert64.sys`, and
`cmd.exe /c sc stop|sc delete GoodbyeDPI | WinDivert | WinDivert14 | zapret`.

**Purpose**: detects other WinDivert-based tools (GoodbyeDPI, zapret, a foreign WinDivert driver) that
would clash with the network accelerator's own WinDivert, and `RemoveDivert` **stops and deletes their
services**. SgbMain loads it (`adtw.dll`, `HasDivert`, `RemoveDivert`). Whether the user is asked
first was not traced.
