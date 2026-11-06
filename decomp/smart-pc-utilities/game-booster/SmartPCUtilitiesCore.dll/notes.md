# SmartPCUtilitiesCore.dll

The vendor's shared helper library (also used by its other products, e.g. "CryptoBuster", named in
the exception lists). ConfuserEx-protected; names only.

| Type | What the names show |
|---|---|
| `OS.PowerHelper` | Power plans: `PowerEnumerate`, `PowerReadFriendlyName`, `PowerGetActiveScheme`, `PowerSetActiveScheme` (powrprof). Game Mode switches to the imported plan and back. |
| `OS.ServiceControllerEx` | Services via SCM (`OpenService`, `QueryServiceStatusEx`) plus **`SetStartMode`** (`ServiceStartMode`): it can change start types, not only stop services. |
| `Processes.ProcessHelper`, `ProcessInfo`, `PrivilegeHelper` | Priority (`PriorityClass`, `PriorityUp`/`PriorityDown`), `NtQueryInformationProcess`, `ReadProcessMemory` (reads other processes' command lines), privilege enabling. Memory: `SetProcessWorkingSetSize` (working-set trim) and `NtSetSystemInformation` (standby-list purge, the `ClearStandbyMemory` option). |
| `Security.WinDefender` | Defender state (`IsRealTimeProtectionEnabled`, `IsTamperProtectionEnabled`); the "Tamper Protection" manual page explains the user must turn Tamper Protection off before the app can change Defender. Changes go through hosted PowerShell (`Scripting.PowerShell.PowerShellHelper`, `System.Management.Automation`). |
| `OS.SystemRestoreHelper` | Creates a System Restore point (`CreateSystemRestorePoint`, `IsSystemRestoreEnabled`) before tweaks. |
| `WindowsRegistry.RegistryHelper`, `RegistryMonitor` | Registry read/write with permission changes (`RegistrySecurity`, `RegistryAccessRule`); a change monitor. |
| `Security.WinTrust.SignVerify`, `EncryptionHelper` | `WinVerifyTrustEx` file-signature checks, XML signature verification, AES/RSA/HMAC-SHA256, MD5/SHA-1/SHA-256 helpers. |
| `ApplicationServices.Promotions.PromotionManager` | Fetches promotions from the vendor (`FetchPromotionsV1Async`): own products and **third-party** offers (`PromotionType`), skippable. |
| `Hardware.DiskHelper`, `SSDHelper`, `DisplayHelper` | SSD detection (skip defrag on SSD), display mode changes. |
| `Connectivity.ConnectivityHelper`, `WebClientEx` | HTTP (timeouts, connectivity check). |
