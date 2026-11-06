# ExitLag.exe (client, version 5.24.0)

Qt 6 / QML client, 117,380,792 bytes, sha256
`9baa794c84c5a90f1b2c84130877ba8ae5a8b1867ece00866dd3b03d0eccc106`, signed Skowsand Tecnologia
Ltda. Uses `libexitlag.dll` for routing, `nfapi.dll` / `ndisapi.dll` for packet capture,
`PresentMonAPI2.dll` for frame times, WebView2 for embedded web pages, Crashpad for crash dumps.
Evidence: component log tags (`[Proxy]`, `[DNS]`, ...), UI strings, analytics event names and import
call sites. No function was decompiled.

## Traffic capture and redirection (`[Proxy]`, `[winpkf_thread]`)

- Per app: ExitLag only proxies processes of games in its server-side app catalogue (ports, port
  ranges, protocols and regions per app), plus optionally the game's launcher (`proxify_launcher`).
  Installed games are found by scanning uninstall registry keys, Steam and Epic libraries.
- WFP mode (NetFilter SDK) redirects the chosen process's TCP and UDP flows to the local proxy
  (LS); NDIS mode (WinpkFilter) rewrites raw packets and re-injects them. Both feed the
  `libexitlag` bridge pool (see `../libexitlag/notes.md`).
- **TunnelKick**: when routing starts, existing TCP connections of the game process are closed so
  the game reconnects through the tunnel.
- **DNS inside the tunnel** (`[DnsOverride]`, per-app "DNS redirection rule"): server-pushed rules
  redirect the game's DNS queries to another resolver (off in some countries, off when the DNS
  optimizer is used), and it can answer IPv6 lookups with a fake "No such name" to force IPv4.
- Special handling for Overwatch voice chat (SIP/RTP sessions are followed), a hard-coded EA
  process list (FC 24/26, EA app, `eaanticheat.gameservice*.exe`, purpose not confirmed; most likely
  special-casing or excluding them from redirection), and Windows Firewall allow rules for itself
  (`netsh advfirewall firewall add rule ... action=allow`).
- No kill switch exists: if all bridges fail the flow is reused or dropped; there is nothing that
  blocks traffic outside the tunnel (it is not a privacy VPN, so none is needed).

## Ping shown to the user

- **"Estimated ping"** (also "Boosted ping" computed for installed games before you route them)
  comes from `OptimizedPingParams`: the best sum over allowed final servers of the client's measured
  ping to a bridge plus ExitLag's **server-reported** bridge-to-final and final-to-data-centre
  pings. It is a model, not a measurement to the game server.
- **"Latency improvement"** (`[QualityMeasurement]`) is a direct ping to the game server (or its
  data centre, ICMP or game port depending on the app's "direct ping method"; "DS is not within a
  DC, skipping latency calculation") minus the optimized value. Both values are reported to ExitLag
  (`ConnectionReport`).
- The overlay can show "game ping only" or the route ping.

## Diagnostics (`[Scanner]`, `[PingTest]`, `[MTR]`, `[PingMonitor]`)

A connection scanner with a ping test (ICMP/UDP/TCP, count, interval, timeout, per-probe RTT and
TTL), an **MTR test** (traceroute with per-hop loss/latency), a speed test, and a gateway ping
monitor ("The gateway is not replying to pings"). These measure without ExitLag and are the honest
part of the product.

## "PC Boost" / FPS features (all toggles, off until enabled)

| Feature | What it changes | Undo |
|---|---|---|
| Ultimate power plan | Creates and activates "Ultimate Performance (ExitLag)"; also writes AC/DC wake-timer settings | Toggle off restores saved plan |
| Core parking | Edits the core-parking power setting (`0cc5b647-...` under `Control\Power\PowerSettings`) | Toggle |
| Game priority / max game priority | `SetPriorityClass` on the game ("Priority of process ... was reverted to original value") | Automatic |
| Input lag reducer | Raises timer resolution, `SetProcessInformation` | Automatic |
| Keyboard response | Keyboard repeat/delay values | Toggle |
| SuperFetch / Windows Search | Stops and disables `SysMain` / `WSearch` | Toggle |
| NTFS last access | `fsutil` last-access update off | Toggle |
| Windows performance counters off | Registry switch that disables perf counters (breaks ExitLag's own overlay, which warns about it) | Toggle |
| Energy Efficient Ethernet | EEE off on physical adapters, **adapter restarted** (brief disconnect) | Toggle |
| Game Bar / Game DVR | Off | Toggle |
| GPU performance profile | NVIDIA via NVML, AMD via ADL Overdrive: **power limit to max, core clocks locked high** | "Profile Reset" |
| Unreal Engine config | Edits the game's UE `.ini` from a server preset, keeps a backup, refuses to apply twice, skips titles where "engine tweaks are blocked" | Revert from backup |
| RAM cleaner | Empties every process's working set, flushes the modified list, combines pages; manual, periodic, or on game connect; allow-list | n/a (transient) |
| Traffic priority (TrafficShaper) | Per-app bandwidth limits for other programs (NetFilter); disabled when Dr.Web is present | Toggle |
| DNS optimizer | Tests DNS servers and sets the adapter's DNS (`SetInterfaceDnsSettings`); "reset" returns to the Windows default, not to a previous static setting | Reset button |
| Overlay | FPS/CPU/GPU/RAM/ping overlay window fed by PresentMon via `ExitLagPmService`; a separate top-level window, no injection | Toggle |

Import check: `WriteProcessMemory` is called only inside Crashpad's
`DumpAndCrashTargetProcess` (library code); no injection into games was found.

## Autostart, updates, telemetry

- **Autostart is enabled by default on first run** ("Enabling autostart (first run)") as a
  Task Scheduler task with a logon trigger that runs `exitlag_autostart.exe`.
- Updates: `[AppUpdater]` asks the web service (protobuf `UpdateRequest`), downloads and launches
  the new installer. The client imports no `WinVerifyTrust`, so no Authenticode check of the
  downloaded installer was found in the client; the updater also logs that it ignores an SSL error
  "probably caused by invalid wildcard handling" after logging the certificate's subject and issuer
  (exact condition not confirmed statically). The installer itself is signed.
- Telemetry: two event pipelines (`[EventService]`, `[MetricEdge]`) carrying UI and feature events
  (e.g. every PC-boost toggle, DNS optimizer paywall hits, multi-internet use), per-connection
  reports, your pings to their network, hardware ID, WMI queries (computer system, video RAM, disk),
  CPU/GPU telemetry, game discovery results, CSAT/NPS surveys, remote banners and inbox pop-ups,
  Crashpad crash dumps.
- Web service hosts seen: `wscdn01..04.exitlag.net`, `cdn-banners.exitlag.com` (+ a bunny.net
  mirror), `www.exitlag.com` (pricing, client area), a Cloudflare R2 bucket for marketing images.

## Monetisation patterns

Login required (email, Google, Apple, Discord); trial countdown modals at 72/48/24/0 h, "YOUR
TRIAL HAS EXPIRED", in-app banners with promo codes (Black Friday), pricing links with UTM tags,
paywall on the DNS optimizer, affiliate links; the account is tied to a hardware ID.
