# Drivers: ndextlag.sys (NDIS) and nfextlag.sys (WFP)

ExitLag does not write its own packet driver. It ships **two licensed commercial packet-filter
SDKs**, renamed with an "extlag" suffix, and lets the user pick one as the "Packets redirection
method" in the client: "Beta - WFP" (NetFilter SDK) or "Legacy - NDIS" (Windows Packet Filter).
Both are installed during setup (see `../installer/notes.md`).

## nfextlag.sys: NetFilter SDK WFP callout

- File version 1.7.1.6, description "NetFilter SDK WFP Driver - ext8"; INF class `WFPCALLOUTS`,
  provider string "exitlag.com", DriverVer 02/27/2023 1.0.0.1. 126,896 bytes (x64), sha256
  `1ec0364cdfb04fab4bbdbfc57854b739f906c165a336c670b26953775edf2c47`.
- Signing: embedded signature by Skowsand Tecnologia Ltda (DigiCert EV chain) plus a catalog
  (`nfextlag.cat`, OS attribute `2:10.0`) countersigned by **Microsoft Windows Hardware Compatibility
  Publisher**. That is Microsoft attestation signing, so it loads on stock 64-bit Windows 10/11
  without test-signing.
- NetFilter SDK (by the vendor of the same name) works at the WFP connect/ALE layers: it can
  redirect a chosen process's TCP connections and UDP flows to a local proxy, per process. User
  side: `nfapi.dll` 1.5.2.4 imported by `ExitLag.exe`.
- In the client this is the default and the only mode that supports "Multi Internet" ("Cannot
  enable multi-internet without WFP") and the DNS optimizer's interception.

## ndextlag.sys: Windows Packet Filter (WinpkFilter) NDIS LWF

- File version 3.4.0.1, company "Skowsand Tecnologia LTDA", description "NDIS Filter driver"; INF
  class NetService, provider "NT Kernel Resources" (the WinpkFilter vendor). 59,696 bytes (x64),
  sha256 `70013fcd32f4745347cbd45c1cc911c0d6939048727f1c8dbcc1da36edf20fa9`. Device
  `\\.\NDEXTLAG`. User side: `ndisapi.dll` 3.4.0.1.
- Signing: embedded and catalog signatures by **WHIMSTERS FINANCIAL SOLUTIONS LTD** (a UK company,
  GlobalSign EV code signing, certificate valid 2022-11-22 to 2023-11-23), countersigned by
  Microsoft Windows Hardware Compatibility Publisher (attestation). So it loads on stock Windows,
  but the submitting company is neither ExitLag nor NT Kernel Resources. Why is not visible from the
  files; it could be a signing service or an earlier licensee.
- An NDIS lightweight filter sees **every packet on every adapter**, below the TCP/IP stack. The
  client's `winpkf_thread` reads packets for the selected game, rewrites them towards the local
  proxy, and re-injects reassembled and "restored" frames to the stack ("Injected restored frame
  (fragmented UDP) to MSTCP"). It needs a reboot after first install ("Please restart the system to
  use the NDIS driver", `exitlag_reboot_required.exe`).
- The client also uses it for a "firewall detector": it sends a test frame with a randomly
  generated MAC address through each adapter to see whether a firewall blocks ExitLag, and falls
  back to NFSDK if so.

## Comparison with SGB

SGB ships a rebranded **WinDivert** (`itvwd64.sys`, `../../../iobit/smart-game-booster/SgbAccelerate.dll/notes.md`), attestation
signed. ExitLag ships two commercial equivalents. All three are signed, legitimate packet-capture
drivers; none is malicious in itself, but each is a kernel component that sees and rewrites game
traffic.
