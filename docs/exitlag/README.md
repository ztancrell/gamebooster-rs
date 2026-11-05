# ExitLag: what it really does

ExitLag (exitlag.com, by Skowsand Tecnologia Ltda, Brazil) is a popular paid "ping reducer" for
online games. This page comes from reading the official installer, version **5.24.0**, downloaded
from ExitLag's CDN on 2026-11-05 (static reverse engineering). **Nothing was run, no account was
made, and no ExitLag server or relay was contacted.** Technical notes and hashes:
[../../decomp/exitlag/README.md](../../decomp/exitlag/README.md).

## Short version

- **What you install:** a Qt 6 desktop app plus **two kernel packet drivers** (renamed commercial
  SDKs: a NetFilter SDK WFP driver and a Windows Packet Filter NDIS driver), both
  Microsoft-attestation-signed, and an **always-on service** (Intel's PresentMon, for the FPS
  overlay). The app adds itself to **autostart by default** on first run.
- **How it works:** it is a **game-traffic relay**, not a VPN. For games in its catalogue, the
  driver hands the game's connections to the app, which sends them through ExitLag's relay servers
  ("bridges", one or two in series) to an exit server near the game's data centre, which talks to
  the game server. "Multipath" sends **the same packets over several relays at once** (and
  optionally over two internet connections) and keeps whichever copy arrives first.
- **No encryption layer** was found for the relayed game packets (only HTTPS to ExitLag's web
  service and RSA for the login password). No kill switch, and none is needed: it is not for
  privacy.
- **The ping it shows before you play ("Estimated ping") is a model:** your measured ping to a relay
  plus ping numbers **reported by ExitLag's own servers** for the rest of the path, often to the
  game's *data centre* rather than the game server itself. The "latency improvement" compares that
  with a direct ping that may use a different method (e.g. ICMP). The number to trust is the
  game's own scoreboard ping, with and without ExitLag.
- **It also ships a "PC Boost" kit:** Ultimate power plan, core parking, process priority, timer
  resolution, SuperFetch/Search off, NTFS last access off, performance counters off, EEE off,
  Game Bar off, a RAM cleaner, GPU power limit to max and clocks locked high, Unreal Engine `.ini`
  edits, a DNS changer and a bandwidth limiter for other apps. All are toggles in the app; the
  uninstaller does not undo them.
- **Subscription:** login required, a short free trial (3 days per reviews), auto-renewing plans,
  account tied to a hardware ID, trial countdown pop-ups and promo banners.
- **Telemetry:** UI and feature events, per-connection quality reports, your pings to their
  network, hardware ID, WMI hardware queries, CPU/GPU stats, discovered games, surveys, crash
  dumps.

## Can a relay lower your ping?

A relay can only help when the **normal internet route is worse than the relay route**. Distance sets
a floor: light in fibre covers about 100 km per millisecond of round trip, and every relay hop adds
its own processing and a detour.

**Real cases where it helps:**
- **Bad ISP peering or a route detour.** Your ISP hands traffic to a congested or far-away transit
  link (e.g. US East to US East via Chicago, or South America to Miami and back). The relay's
  provider has a direct path, so "you to relay to game" can beat "you to game".
- **Packet loss on one path.** Redundant multipath genuinely hides loss on a single bad link (Wi-Fi
  drops, a congested peering point), because a second copy usually arrives. It costs double upload
  bandwidth or more and can't fix loss in your own Wi-Fi when both copies go through it (unless you
  use "Multi Internet" over a second connection).
- **Games whose own region choice is poor**, or regions far away (playing on another continent).

**Where it cannot help:**
- **The route is already close to direct.** If you reach the game's region in about the time light
  needs to get there, there is nothing to shorten; a relay adds a hop and usually 1–5 ms or more.
- **Problems at home**: Wi-Fi interference, bufferbloat on your router when someone uploads, a slow
  PC. A relay doesn't touch them.
- **Server-side lag**: tick rate, server load, netcode. Those show up as "lag" but not as network
  latency.

**What `gamebooster ping` told us:** on the target Mac, US East (N. Virginia) answered in about
**21 ms**. That is already a near-direct path for an East Coast player; BF6 servers in that region sit
within a few milliseconds of that. A relay could at best shave a few ms if the ISP route has a
detour, and more likely adds some. ExitLag would mainly be worth trying if you see **packet loss**
or **spikes** that a loss/route test (below) pins on the ISP path, not on Wi-Fi or the PC.

## Feature verdicts

Verdicts as in [the SGB overview](../sgb/README.md): **useful**, **placebo**, **harmful**,
**risky** (can break things, cost security or privacy, or trip anti-cheat), **bloat**.

| Feature | What it does | Verdict |
|---|---|---|
| Relay routing (bridges + exit server) | Sends the game's traffic through ExitLag's network | **useful** only with a bad ISP route or a far region; otherwise placebo or slightly worse |
| Multipath (several relays at once) | Same packets over 2+ relays, first copy wins | **useful** against loss on a single path; costs upload bandwidth |
| Multi Internet | Copies over two connections (e.g. Ethernet + phone hotspot) | **useful** for bad Wi-Fi/loss if you have a second link; burns mobile data |
| Estimated / boosted ping | Relay ping + server-reported ping to the data centre | **misleading** as an absolute number; compare in-game ping instead |
| Scanner: ping, MTR, speed test | Measures your connection without ExitLag | **useful** (the honest part) |
| DNS optimizer | Picks a faster DNS server and sets it on the adapter; reset goes to Windows default | placebo for in-game ping (DNS is only used at connect); reset can lose a previous static DNS |
| DNS redirection inside the tunnel | Server-pushed per-game DNS rules; fake "no such name" for IPv6 | risky (silent behaviour change), mostly harmless |
| TunnelKick | Closes the game's open TCP connections so they reconnect through ExitLag | needed by its design; can drop you from a lobby |
| Traffic priority (bandwidth limits for other apps) | Caps other programs' bandwidth | **useful** if others on the PC saturate the line; a router SQM/QoS does it better |
| Ultimate power plan | Own copy of Ultimate Performance | placebo on desktops; **harmful** on laptops (heat) |
| Core parking | Edits the core-parking power setting | placebo on Windows 10/11 |
| Game priority / max priority | Raises the game's process priority, restored afterwards | placebo; "max" (realtime-like) can starve input/audio |
| Input lag reducer | Higher timer resolution | placebo on Windows 11 (games set their own) |
| Keyboard response | Keyboard repeat/delay | placebo for games (raw input) |
| SuperFetch / Windows Search off | Stops and disables services | placebo; slower search and app starts |
| NTFS last access off | `fsutil` setting | placebo |
| Performance counters off | Disables Windows perf counters | **harmful** (breaks monitoring tools, including ExitLag's own overlay) |
| Energy Efficient Ethernet off | Adapter setting; restarts the adapter | useful rarely (only with EEE-related stutter); otherwise placebo |
| Game Bar / DVR off | Off | placebo unless recording in background |
| RAM cleaner | Empties all working sets, flushes standby/modified lists | placebo to **harmful** (causes page faults and stutter as everything reloads) |
| GPU performance profile | Power limit to max, core clocks locked high (NVIDIA/AMD) | **harmful** on laptops (heat → throttling); placebo when GPU-bound |
| Unreal Engine booster | Edits the game's `.ini` from a server preset, with backup | risky (game updates, anti-cheat on some titles); irrelevant to BF6 (Frostbite) |
| Overlay (FPS, ping) | Separate window, PresentMon data; no injection | fine; but the PresentMon **service** runs at every boot |
| Autostart on by default | Logon scheduled task | **bloat** / dark pattern |

## For a 2019 MacBook Pro under Boot Camp playing BF6

- **Network:** at ~21 ms to US East, the route is already near-direct. Expect little or no gain in
  ping. Try it (on the trial) only if you see packet loss or spikes, and judge by BF6's own
  scoreboard ping and loss counters, not by ExitLag's estimate. On Wi-Fi, a cable or a better
  channel will do more than any relay.
- **PC Boost items to avoid on this Mac:** GPU profile (max power limit on a heat-limited Radeon Pro
  5300M/5500M, and Apple's old Boot Camp driver may not expose Overdrive anyway), Ultimate power
  plan and core parking (more heat, worse throttling), RAM cleaner, performance counters off,
  "max game priority". EEE off restarts the adapter; harmless but pointless on Wi-Fi.
- **Anti-cheat (EA Javelin):** ExitLag installs two kernel packet drivers that sit under every
  connection, and it redirects the game's traffic through third-party servers. Neither is a cheat,
  and both drivers are properly signed, but EA support's standard advice for Javelin errors is to
  turn off VPN and proxy software first, and EA has not published a statement on ExitLag. Its own
  "can I get banned" page only covers other publishers. Risk: **moderate-low for bans, real for
  connection errors**; if BF6 fails to connect or Javelin complains, uninstalling ExitLag (which
  removes both drivers) is the first step.
- **Bottom line:** not worth paying for on this setup unless a loss/route test shows an ISP problem.

## Compared with SGB's accelerator and gamebooster-rs

| | ExitLag 5.24 | SGB 6.0 accelerator | gamebooster-rs |
|---|---|---|---|
| Packet capture | Two kernel drivers (NetFilter SDK WFP, WinpkFilter NDIS), installed at setup | Rebranded WinDivert driver | **None** (no driver, ever) |
| Tunnel | Own relay protocol over TCP/UDP, 1–2 bridges + exit server, redundant multipath, retransmission | KCP over UDP with ChaCha20 (iTop VPN core), single node | None (decided not to copy a relay accelerator) |
| Encryption | None found for game packets | ChaCha20 | n/a |
| Node choice | Sum of measured + server-reported legs, live switch to spare routes | Geo distance, server load, TCP/UDP probe | n/a |
| Ping shown | Model (relay ping + server-reported legs) | Ping to the node only | Real median TCP connect to cloud regions, labelled as an approximation |
| System changes | DNS, power plan, services, GPU clocks, NTFS, perf counters... (toggles) | DNS to 127.0.0.1, BFE/firewall services forced on (not restored) | Only reversible tweaks, undone by `restore` |
| Diagnostics | Ping test, MTR, speed test | No | `ping` (latency to regions) |

ExitLag's routing core is more sophisticated than SGB's (multi-relay, redundant copies, per-flow
re-routing), and its installer cleans its drivers up on uninstall. But it shares the same
fundamental limit: it can only help when your ISP's route is the problem.

## Worth adopting (reversible, driver-free)

1. **Loss and jitter in `gamebooster ping`.** Today it reports a median TCP connect time. Add the
   spread (min/max or p90) and how many of the connects failed or timed out, so a lossy or spiky
   path shows up. Still no admin, no driver.
2. **A route check (`gamebooster ping --route <host>`).** A traceroute/MTR-style probe: send probes
   with increasing TTL (or run Windows' own `pathping`/`tracert` and summarise), and report per-hop
   latency and loss, plus the first hop (the router) separately. That answers the real question
   ExitLag sells against: "is my problem Wi-Fi/home, my ISP, or the far end?". If the first hop
   already loses packets, no relay helps.
3. **Gateway ping during a session**, like ExitLag's ping monitor: occasional pings to the default
   gateway to flag Wi-Fi trouble. Read-only.
4. Documentation: explain in `docs/tweaks/network.md` when a relay can help (bad peering, detours,
   loss) and when it can't (near-direct route), using the ~21 ms US East result as the example.

Not worth adopting: relays, packet drivers, multipath (all need vendor servers or a driver, which
DESIGN.md rules out), the RAM cleaner, GPU clock locks, perf-counter and service toggles.

## Sources

- Installer `https://cdn.exitlag.com/SetupExitLag-5.24.0-x64.exe` (link pattern from the archived
  [exitlag.com/download](https://web.archive.org/web/20250911092654/https://www.exitlag.com/download)
  page; the live site blocks non-browser requests).
- ExitLag's own pages, via search results: [What is ExitLag](https://www.exitlag.com/blog/what-is-exitlag/),
  [How ExitLag works](https://www.exitlag.com/blog/how-exitlag-works/),
  [Can I get banned for using ExitLag?](https://www.exitlag.com/blog/can-i-get-banned-for-using-exitlag/).
- EA forum advice on Javelin errors and VPN/proxy software:
  [example thread](https://forums.ea.com/discussions/battlefield-6-technical-issues-en/game-crash-ea-javelin-anticheat-exittype-95/13482903).
- Trial length: [Windows Report guide](https://windowsreport.com/how-to-use-exitlag/).
