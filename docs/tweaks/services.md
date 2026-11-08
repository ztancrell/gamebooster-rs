# Stop services

**What it is.** Stops the services listed in `services = [...]` for the session. For each listed
service that is running, the running services that depend on it are stopped first, then the
service itself, each with a 30 s timeout. The list is a strict allow-list: if a running dependent
isn't listed too, the service is skipped (with a message) rather than touching an unlisted one.
Only services that were running are recorded; restore starts exactly those again, in reverse
order. The startup type is never changed, so a reboot also brings them back. A service that has
been disabled since the boost (by you or Windows) is left stopped with a note, and doesn't count as
a failed restore.

Superseded: earlier versions also stopped running dependents that weren't listed.

**Windows default.** Many services run all the time: search indexing (`WSearch`), prefetching
(`SysMain`), telemetry (`DiagTrack`), the print spooler (`Spooler`), plus third-party updaters
(Adobe, Google, Apple Software Update) and Boot Camp's own helpers.

**Why it helps.** Each service wakes up now and then to do work: index a new file, check for
updates, upload telemetry. On a 4-core Mac that work competes with the game's threads and causes
the occasional long frame. The gain depends entirely on what runs on your machine.

**Exact setting.** `services = ["WSearch", "SysMain"]` in `gamebooster.toml`: service *key* names,
not the display names. Nothing is stopped by default. Don't list audio (`Audiosrv`), networking,
security software or Apple/Boot Camp keyboard and trackpad services.

## Suggested list (from SGB, pruned)

`gamebooster.example.toml` carries a commented-out `services = [...]` block (64 services): Smart
Game Booster's `[Services_WIN10]` list, which Windows 11 also gets, ticked and unticked entries
alike (decoded in [decomp/iobit/smart-game-booster/SgbData/README.md](../../decomp/iobit/smart-game-booster/SgbData/README.md)). It is opt-in: the
default stays `services = []`. Each line says what the service is; `risky:` marks ones that can
break something during the session (an update in progress, text input, Bluetooth, security
features), `SGB default: off` marks ones SGB listed but didn't tick. Listed services that aren't
installed or aren't running are simply skipped.

Changed from SGB's list:

- `Hyper-V Data Exchange Service` is a display name; the key `vmickvpexchange` is used instead.
- Removed, gone from current Windows 10 (22H2) and 11: `UI0Detect` (Interactive Services
  Detection, removed in 1803), `hkmsvc` and `napagent` (Network Access Protection, removed with
  Windows 10), `Mcx2Svc`, `ehRecvr` and `ehSched` (Media Center), `AllUserInstallAgent` (Windows 8
  app installer, replaced by AppXSvc), `WSService` (Windows 8 Store service).
- Removed as security-relevant: `BDESVC` (BitLocker Drive Encryption Service).

See which of them exist and run on your machine (PowerShell):

```powershell
Get-Service -Name TrustedInstaller,TabletInputService,StiSvc,WMPNetworkSvc,AxInstSV,ALG,AppMgmt,PeerDistSvc,CertPropSvc,WdiServiceHost,WdiSystemHost,EFS,Fax,KtmRm,lltdsvc,swprv,NetTcpPortSharing,Netlogon,pla,PNRPAutoReg,wercplsupport,PcaSvc,QWAVE,SessionEnv,UmRdpService,RpcLocator,RemoteRegistry,RemoteAccess,SCPolicySvc,SNMPTRAP,StorSvc,WebClient,WcsPlugInService,WerSvc,Wecsvc,FontCache,WinRM,W32Time,DeviceInstall,DsmSvc,fhsvc,vmickvpexchange,vmicshutdown,vmicheartbeat,vmicrdv,vmictimesync,vmicvss,NcaSvc,PrintNotify,svsvc,WiaRpc,SystemEventsBroker,TimeBroker,bthserv,hidserv,DiagTrack,MapsBroker,DPS,TrkWks,SharedAccess,MSiSCSI,CscService,SSDPSRV,Themes -ErrorAction SilentlyContinue | Format-Table Name,Status,StartType
```

Names missing from the output aren't installed. Only the `Running` ones would be stopped.

## Try and undo by hand

```bat
rem STATE: RUNNING or STOPPED
sc query WSearch
rem services that depend on it
sc enumdepend WSearch
rem asks before stopping dependents too
net stop WSearch
rem undo
net start WSearch
```

Or `services.msc`: find the service, right-click > Stop / Start. The "Service name" in its
Properties dialog is the key name to put in the config.

## Measure

- `gamebooster boost --dry-run` lists the exact services (with dependents) that would stop, and
  names any it skips because of an unlisted dependent.
- `sc query <name>` shows STOPPED while boosted.
- Task Manager > Services tab shows the PID; check that process's CPU use in the Details tab
  before you decide a service is worth stopping.
- CapFrameX runs with and without the service stopped ([how](README.md#how-to-measure-any-tweak)).
