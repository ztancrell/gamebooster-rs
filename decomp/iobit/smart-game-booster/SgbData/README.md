# SgbData: Smart Game Booster default Boost lists

Static analysis only; no vendor binary was run.

## Result

The default service lists are **not encrypted**. The installer drops a plain-text INI,
`{userappdata}\PCGameBoost\Smart Game Booster\WinSvc.ini` (4,213 bytes, SHA-256
`9790e9f4c1484750de96e310985730d65594120b688375cf8dcf394ef5790968`; Inno entry with
`MinVersion` NT 6.1, flag `ignoreversion`). On first start `SgbInit.exe` copies it over
`UserWinSvc.ini` in the same folder and deletes the original (function VA 0x6575e0:
`CopyFileW("…\WinSvc.ini", "…\UserWinSvc.ini", FALSE)`, then `DeleteFileW("…\WinSvc.ini")`; on
failure it only logs `"WinSvc.ini :" + GetLastError` via OutputDebugString). SgbMain's service
boost then reads `UserWinSvc.ini` (VA 0x7885d0) with the section chosen by VA 0x78754c.

The INI's own comment: services with value 1 are ticked (selected), 0 are only listed, unticked.
There is **no `[Services_2000]` section** (Windows 2000 gets an empty list). `[Services_3rd]` is
empty (third-party services are added by the user). Some keys repeat in a section (`StorSvc` in
WIN8/WIN10, `DiagTrack`, `MapsBroker` in WIN10); the first value is used below.
`Hyper-V Data Exchange Service` is a display name, not a service key (the key is `vmickvpexchange`).
Many entries are legacy services that no longer exist on Windows 10/11 (`UI0Detect`, `hkmsvc`,
`napagent`, `Mcx2Svc`, `ehRecvr`, `ehSched`, `WSService`, `AllUserInstallAgent`, …).

Regenerate the lists below with `python3 -I winsvc_lists.py <path to WinSvc.ini>`.

## Default process lists: not in the installer

`SgbInit.exe` does the same copy for `ProcWL.txt` → `UserProcWL.txt`, `ProcBL.txt` →
`UserProcBL.txt`, `AppSafe.txt` → `UserAppSafe.txt` and `3rdSvcWL.txt` → `User3rdSvcWL.txt`
(same function), but the installer ships **none** of those four source files. So a fresh install
has no default process kill/keep list from a data file; the process list comes from the user's
choices (and possibly a download). SgbMain also opens these `User*` files in Notepad for editing
(VA 0x7d1be4, 0x95d260).

## The encrypted `.dbd` files (not decoded, not Boost)

Superseded: the `.dbd` files below are now decoded ([`dbd/README.md`](dbd/README.md)); `Def.dbd` is the
malware scanner's signature set and the security/AV-detection data and every other data file in the
install are described in [`scanner/README.md`](scanner/README.md).

`Database/*.dbd`, `Database/DriverCleanIgnore.dat` and `Update/ExpSlist.dat` all start with the
same 24 bytes (`0008dc3345ab51295f6c147912d70f4b ea5ec653b2d8b74e`), then diverge; entropy is
8.0 bits/byte and body lengths are not multiples of 8 or 16 (stream cipher or similar over
compressed data). SgbMain treats each as a password-protected container of named entries
(`LoadCYRIDataBase` opens `CYRI.dbd` with password-like literal `SMGB5_CYRI` and extracts
`ScoreCFG.ini` and `CYRI.db`; `Opt.dbd` uses `asc6_2012_6`, an Advanced SystemCare 6 leftover, and
holds `Main.ini`). None of them is a Boost list: `CYRI`/`Def` = game database, `Opt` = system
optimizer, `PriTemp` = junk-file/privacy scanner templates, `SysSpeed` = speed-up items. The
container reader (VA 0x75793c–0x758c00) was not identified further. `*.sgdt` are also high-entropy
(not analysed). `Update/*.dat` are ZipCrypto archives of the WinDivert drivers; `Ngulocal.zip` is
an encrypted `Ngulocal.json`.

## Method

1. innoextract 1.9 fails on this installer (Inno Setup 6.4.2). binary-refinery's `xtinno`
   (installed with `uv tool install binary-refinery`) extracts it:
   `emit setup.exe | xtinno [| dump {path} ]`, into a fresh directory.
2. Search all extracted files for the INI section and list-file names; `WinSvc.ini` was found as
   plain text, `SgbInit.exe` (not protected) holds the copy code.
3. Disassembly with `../SgbMain.exe/dis.py` and `refs.py` (the same `.text` layout applies to
   `SgbInit.exe`).

## Default service lists (WinSvc.ini)

### [Services_XP] (38 ticked, 8 unticked)

Ticked (=1, selected for Boost by default): `stisvc`, `Alerter`, `ALG`, `AppMgmt`, `ClipSrv`, `COMSysApp`, `MSDTC`, `helpsvc`, `HTTPFilter`, `ImapiService`, `CiSvc`, `dmserver`, `dmadmin`, `Messenger`, `SwPrv`, `LmHosts`, `mnmsrvc`, `NetDDE`, `NetDDEdsdm`, `Netlogon`, `Nla`, `xmlprov`, `NtLmSsp`, `WmdmPmSN`, `RSVP`, `RDSessMgr`, `RpcLocator`, `RemoteRegistry`, `RemoteAccess`, `seclogon`, `ShellHWDetection`, `SENS`, `TlntSvr`, `UPS`, `vds`, `WebClient`, `Wmi`, `WmiApSrv`

Unticked (=0, listed only): `Browser`, `HidServ`, `LanmanServer`, `Schedule`, `Themes`, `TrkWks`, `SSDPSRV`, `W32Time`

### [Services_VISTA] (39 ticked, 12 unticked)

Ticked (=1, selected for Boost by default): `TrustedInstaller`, `stisvc`, `AeLookupSvc`, `ALG`, `AppMgmt`, `CertPropSvc`, `DFSR`, `WdiServiceHost`, `WdiSystemHost`, `MSDTC`, `Fax`, `hkmsvc`, `IKEEXT`, `UI0Detect`, `KtmRm`, `lltdsvc`, `NetTcpPortSharing`, `Netlogon`, `napagent`, `WPCSvc`, `PNRPAutoReg`, `wercplsupport`, `QWAVE`, `RemoteRegistry`, `RemoteAccess`, `SstpSvc`, `SCPolicySvc`, `SNMPTRAP`, `TapiSrv`, `UmRdpService`, `vds`, `WebClient`, `idsvc`, `WerSvc`, `ehRecvr`, `ehSched`, `ehstart`, `WinRM`, `W32Time`

Unticked (=0, listed only): `Browser`, `UxSms`, `hidserv`, `EMDMgmt`, `LanmanServer`, `TabletInputService`, `Themes`, `DPS`, `TrkWks`, `SharedAccess`, `MSiSCSI`, `CscService`

### [Services_WIN7] (48 ticked, 12 unticked)

Ticked (=1, selected for Boost by default): `TrustedInstaller`, `AeLookupSvc`, `TabletInputService`, `StiSvc`, `WMPNetworkSvc`, `AxInstSV`, `ALG`, `AppMgmt`, `PeerDistSvc`, `CertPropSvc`, `WdiServiceHost`, `WdiSystemHost`, `EFS`, `Fax`, `hkmsvc`, `UI0Detect`, `KtmRm`, `lltdsvc`, `Mcx2Svc`, `swprv`, `NetTcpPortSharing`, `Netlogon`, `napagent`, `pla`, `PNRPAutoReg`, `wercplsupport`, `PcaSvc`, `QWAVE`, `SessionEnv`, `UmRdpService`, `RpcLocator`, `RemoteRegistry`, `RemoteAccess`, `SCPolicySvc`, `SNMPTRAP`, `sppuinotify`, `StorSvc`, `TBS`, `WebClient`, `idsvc`, `WcsPlugInService`, `WerSvc`, `Wecsvc`, `FontCache`, `ehRecvr`, `ehSched`, `WinRM`, `W32Time`

Unticked (=0, listed only): `bthserv`, `Browser`, `UxSms`, `hidserv`, `Themes`, `BDESVC`, `DPS`, `TrkWks`, `SharedAccess`, `MSiSCSI`, `CscService`, `SSDPSRV`

### [Services_WIN8] (62 ticked, 10 unticked)

Ticked (=1, selected for Boost by default): `TrustedInstaller`, `AeLookupSvc`, `TabletInputService`, `stisvc`, `WMPNetworkSvc`, `AxInstSV`, `ALG`, `AppMgmt`, `PeerDistSvc`, `CertPropSvc`, `WdiServiceHost`, `WdiSystemHost`, `EFS`, `Fax`, `hkmsvc`, `UI0Detect`, `KtmRm`, `lltdsvc`, `swprv`, `NetTcpPortSharing`, `Netlogon`, `napagent`, `pla`, `PNRPAutoReg`, `wercplsupport`, `PcaSvc`, `QWAVE`, `SessionEnv`, `UmRdpService`, `RpcLocator`, `RemoteRegistry`, `RemoteAccess`, `SCPolicySvc`, `SNMPTRAP`, `StorSvc`, `WebClient`, `WcsPlugInService`, `WerSvc`, `Wecsvc`, `FontCache`, `WinRM`, `W32Time`, `DeviceInstall`, `DsmSvc`, `fhsvc`, `Hyper-V Data Exchange Service`, `vmicshutdown`, `vmicheartbeat`, `vmicrdv`, `vmictimesync`, `vmicvss`, `Mcx2Svc`, `NcaSvc`, `PrintNotify`, `svsvc`, `WiaRpc`, `SystemEventsBroker`, `TimeBroker`, `AllUserInstallAgent`, `ehRecvr`, `ehSched`, `WSService`

Unticked (=0, listed only): `bthserv`, `hidserv`, `BDESVC`, `DPS`, `TrkWks`, `SharedAccess`, `MSiSCSI`, `CscService`, `SSDPSRV`, `Themes`

### [Services_WIN10] (61 ticked, 12 unticked)

Ticked (=1, selected for Boost by default): `TrustedInstaller`, `TabletInputService`, `StiSvc`, `WMPNetworkSvc`, `AxInstSV`, `ALG`, `AppMgmt`, `PeerDistSvc`, `CertPropSvc`, `WdiServiceHost`, `WdiSystemHost`, `EFS`, `Fax`, `hkmsvc`, `UI0Detect`, `KtmRm`, `lltdsvc`, `swprv`, `NetTcpPortSharing`, `Netlogon`, `napagent`, `pla`, `PNRPAutoReg`, `wercplsupport`, `PcaSvc`, `QWAVE`, `SessionEnv`, `UmRdpService`, `RpcLocator`, `RemoteRegistry`, `RemoteAccess`, `SCPolicySvc`, `SNMPTRAP`, `StorSvc`, `WebClient`, `WcsPlugInService`, `WerSvc`, `Wecsvc`, `FontCache`, `WinRM`, `W32Time`, `DeviceInstall`, `DsmSvc`, `fhsvc`, `Hyper-V Data Exchange Service`, `vmicshutdown`, `vmicheartbeat`, `vmicrdv`, `vmictimesync`, `vmicvss`, `Mcx2Svc`, `NcaSvc`, `PrintNotify`, `svsvc`, `WiaRpc`, `SystemEventsBroker`, `TimeBroker`, `AllUserInstallAgent`, `ehRecvr`, `ehSched`, `WSService`

Unticked (=0, listed only): `bthserv`, `hidserv`, `DiagTrack`, `MapsBroker`, `BDESVC`, `DPS`, `TrkWks`, `SharedAccess`, `MSiSCSI`, `CscService`, `SSDPSRV`, `Themes`

### [Services_3rd] (0 ticked, 0 unticked)

Ticked (=1, selected for Boost by default): none

Unticked (=0, listed only): none

