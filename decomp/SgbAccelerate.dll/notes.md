# SgbAccelerate.dll (+ WinDivert.dll, itvwd*.sys)

`SgbAccelerate.dll` in the `-db` extraction and carved `pe_0c86fb8f_485944.bin` are byte-identical
(SHA-256 314991ae...), so `functions/*.c` here are trustworthy for call targets. Decompile is
partial on purpose (out of scope). This note is an architecture overview from strings, exports,
imports and two functions (`0x10018be0` startProxy, `0x10019490` socket-layer thread). Nothing was
executed and no vendor server was contacted. VA = file offset + 0x10000c00 for `.rdata`.

## Pieces

- **SgbAccelerate.dll**: C++ client, built from `E:\DelphiWork\SMGB\itop_Server\vpnclient\` (the
  **iTop VPN** client core, same company). Uses libhv (event loop, all `_h*` exports), KCP (`ikcp_*`,
  "input ack: sn=%lu rtt=%ld rto=%ld"), ChaCha20 (`chacha20`), WinDivert. One non-libhv export:
  `_vpnclient_setDefaultPortAbuse@8`. Main class `CClientP2S`, net layer `vpnenv`.
- **WinDivert.dll**: WinDivert user library, loads driver `itvwd` (`itvwd.dll` export name,
  EventLog key `...\EventLog\System\WinDivert`).
- **nDrivers/itvwd64.sys**, `Update/rZyWyKka.dat` (ZipCrypto zip holding `itvwd64.sys`, 73,288 B)
  and `Update/d5mMpla8.dat` (`itvwd_arm64.sys`, 88,120 B): rebranded WinDivert kernel driver.
  `itvwd64.sys` is signed by "Microsoft Windows Hardware Compatibility Publisher" (cert 2022-06-07 ..
  2023-06-01), i.e. Microsoft attestation signing; it loads on stock 64-bit Windows.
- **SgbMain.exe** (Delphi) holds the control plane: units `uTopVPN*`, `uVpnClient`,
  `TServerListManager`; log line `InitVPNClient2 load SgbAccelerate.dll ok.`

## Control plane (SgbMain.exe strings)

- API base `https://api.gameboosta.com/` (hard-coded fallback IPs `3.229.56.169|54.165.29.74`),
  endpoints `info/init`, `info/ipinfo`, `config/auto-location`, `config/ip-config`,
  `boost/get-token`, `boost/server-load`, `boost/connectivity`, `boost/post-connect`,
  `boost/bind-ips`, `boost/gmlist`, `boost/gmres`, `base-info/stat-ip`; also
  `https://ascstats.iobit.com/base-info.php` (country).
- Login: email/password, Google, activation code, or an iTop VPN account
  (`lbl_code_bound_account`); device and account limits (`network_error_max_connect_`,
  "maximum limit of 3 registered accounts on this device"); heartbeat (`Hearttime`,
  `Heartfrequency`, `DoHeartBeatFailed`).
- Server list: JSON `group_list` / `server_group_list` / `server_list` with `country_code`,
  `target_game`, `is_p2p`, `is_ipv6`, `is_nat`, `cadd_isvip`, `port_info`, `security_level`,
  `algorithm` (`chacha20`); cached as `GBseverList.sgdt` / `Database/SpnnDef.sgdt`, with a backup URL.
- Picking a server (`GetAvailableHostList`, `GetAvailableHost2..5`): candidates by game
  (`GetGameArea`, "Smart Acceleration"), country or city, then ranked "by conn", "by distance"
  (`CalcHostDistance la:` = great-circle distance from the geo-IP location), "by random", plus
  server load (`boost/server-load`, `DoServerLoadCheck set ping`) and a ping pass
  (`GetAvailableHost2 before/after ping count`). Free users get a separate pool
  (`GetAvailableHost2 add FreeTrial count`).
- Ping measurement (in the DLL): `getServerDelay[TCP]` = time to TCP-connect to the proxy port;
  `getServerDelay[UDP]` = UDP probe with `loss_rate:`. Shown as Latency / Loss Rate per node.
- Connect: sets the adapter DNS to 127.0.0.1 and backs up the old one (`[Connect]DoDNSClear
  SetCustomDNS to 127.0.0.1 bakDNS:`), restores on disconnect; `netsh interface ipv4/ipv6 set
  interface` with `ASCNETSHBak.dat`; may block IPv6 (`IsBlockIPv6Package`); flushes DNS. On
  disconnect: `cmd.exe /c sc stop itvwd`.
- Services: if BFE is disabled, `SYSTEM\CurrentControlSet\Services\BFE` `Start` is fixed and
  `/c net.exe start BFE` runs ("Connection failed as the BFE service is disabled"). The
  ChangeServiceConfigW calls at 0x81c667 (SharedAccess/MpsSvc) and 0x9c63b2 (BFE) set
  SERVICE_AUTO_START (see `../SgbMain.exe/notes.md`). WinDivert sits on WFP, which needs BFE.
  These are **persistent** changes; nothing found puts the old start type back.
- Conflict checks: detects other VPNs / firewalls / GoodbyeDPI and offers to remove them
  (`lbl_notcompatible_googdpi`, `connect_failed_wfp`).

## Data plane (SgbAccelerate.dll)

`CClientP2S::init` builds a WinDivert filter from these templates (`%s` = exclusions such as
` and ip.DstAddr != <server> and ip.SrcAddr != <server>`, `%d` = local proxy port):

```
(ip and tcp and outbound and ( (ip.DstAddr != 127.0.0.1 and ip.SrcAddr != 127.0.0.1 %s)
   or (ip.SrcAddr == 127.0.0.1 and tcp.SrcPort==%d)))
or (ip and udp and outbound and (ip.DstAddr != 127.0.0.1 or udp.SrcPort==%d or udp.DstPort==53) %s)
```
plus an IPv6 twin. So the driver captures **all outbound TCP and UDP** (IPv4 and v6) except
loopback and the tunnel server itself. `startProxy` (0x10018be0) opens it at NETWORK layer with
priority -30000, and a second handle at SOCKET layer (layer 3, flags 5 = SNIFF|RECV_ONLY) using
`remoteAddr != 127.0.0.1 and localAddr != 127.0.0.1 and localPort!=%d and remotePort!=%d`.
The socket-layer thread (0x10019490, `transAppFilter`) maps flows to processes
(`OpenProcess` + `QueryFullProcessImageNameA`, `appfilter address/appnotify`) so only the chosen
game's flows go into the tunnel; others are reinjected unchanged ("origin").

Captured game packets go to a local libhv TCP/UDP listener (`listen tcp4/udp4 on %s:%d`) and out
to the node over **KCP on UDP**, ChaCha20-encrypted, with TCP and UDP session tokens
(`setTcpToken`, `setUdpToken`; server errors `token err`, `server maxcount err`). Threads:
transpacket, app_filter, checkheartbeat (`canDialServer`, unexpected-disconnect callback),
monitorspeed.

Abuse filter (`vpnenv_firewall`), configured from the server: drops BitTorrent (`BitTorrent
protocol`, `9:get_peers`, `9:info_hash20:`), eD2k, SMTP, HTTP, HTTPS and listed DNS names
(`dnshack` with regex), per-port rate limits (`updateAbuseConfig`, droprate), optional
`FIREWALL_FLAG_DISABLE_UDP`. This is the "DNS/HTTP/HTTPS/P2P/BitTorrent filters" in spec section 6:
it stops the tunnel being used as a free general VPN. **Kill switch**: `setKillSwitch open/close`.

## Free tier

Free accounts get limited "Free Boost Time" hours (`GiftTimeDes` "Get %d-Hour Free Boost Time",
`lblRunningLow` "Your free boost time is running low") and a separate free server pool; after that
SVIP is required ("Network acceleration is available while your SVIP subscription is active").
Region blocks exist (`network_error_region_restriction`).

## Full Ghidra pass (2026-11-05): what the data plane does in detail

A complete Ghidra 12.1.2 headless decompile of the carved copy (2,163 functions) was read for the
points below and then deleted, as agreed: only these prose findings are kept. Addresses are VAs in
this DLL. Confidence is high unless stated.

**Login and tokens.** The DLL does no login. SgbMain obtains the session tokens from
`boost/get-token` and hands them over through the setter at 0x1001a6c0 (`setTcpToken`/`setUdpToken`,
export ordinal 10094): two 64-bit values stored in the tunnel object. If the tunnel is currently in
the "token err" state, setting a new token clears that state so the next connect attempt retries
with it. The node's first reply is decrypted (64-byte header) and checked in the KCP receive path at
0x100103b0 (siblings 0x10010d80, 0x10011560): status -11 means "token err" and status -12 "server
maxcount err" (too many clients on that node); either one fires a callback into SgbMain, which shows
the error and stops.

**Node list and failover inside the DLL.** `CClientP2S::init` (0x10018430) takes up to 16
host:port pairs (anything beyond 16 is ignored), a rotation interval in seconds, a DNS server
string, a network type (IPv4/IPv6) and a flag word. Each address is parsed with `inet_pton`; one bad
address aborts init with an error code in the 0x1101000x range, which SgbMain reads back. The flag
word turns into tunnel options (TCP, UDP, IPv6, and a mode where any outbound TCP from the local proxy
port is captured). The node choice itself (by game, region, distance, load, ping) is SgbMain's; the
DLL only fails over between the nodes it was given.

**Ping.** `getServerDelay` (0x10019a60, exported through 0x1001b400): TCP latency is a non-blocking
connect to the node's port timed with `select`; UDP is a datagram probe with a receive timeout that
also counts lost replies and logs `loss_rate`. These are the Latency and Loss Rate columns.

**Heartbeat and failure handling.** A thread (0x100196e0) wakes every 0.5 s and, about every 10 s,
tests whether the current node still accepts a TCP connection (0x10019e30, which uses the
non-blocking connect at 0x100180b0). When the rotation interval has passed it starts from a random
node instead. It tries the remaining nodes forward and then backward, and remembers the first one
that answers as the active node. After 3 failed checks in a row (5 when the network type is 1) it
calls SgbMain's "unexpected disconnect" callback. SgbMain then shows the error and tears the
connection down; the DLL does not reconnect on its own. While the tunnel reports itself paused, the
thread sleeps 100 s between checks.

**Tunnel and encryption.** KCP over UDP via libhv, with flow control that holds back sending once
the queue passes 90% of the KCP window (0x100103b0). The cipher is chosen by name in 0x1000f980:
`none`, `xor` (one key byte), `rc4` (8-byte key) or `chacha20`; the server list's `algorithm` field
says `chacha20`. For ChaCha20 the routine copies **8 bytes** of key material into the 32-byte key
(the rest stays zero) and zeroes the 12-byte nonce. Medium confidence that the nonce really stays
zero: no per-packet nonce update was found, but one could sit in the call sites 0x10015670,
0x1000fdc0 and 0x1000fe80. Either way, the effective key is 64 bits, so the tunnel is obfuscation
rather than strong encryption.

**WinDivert filters.** Beyond the templates quoted above: init inserts ` and ip.DstAddr != <node>
and ip.SrcAddr != <node>` for **every** node in the list, so tunnel traffic itself is never captured.
Flag 8 switches to the variant where any outbound TCP from the proxy port is captured, not only
loopback. Flags 0x22 add the IPv6 twin, and the IPv4 and IPv6 halves are joined as `(%s) or (%s)`.
The separate socket-layer filter for mapping flows to processes is built at the same time. On
`stopProxy` (0x100190b0) both handles are closed and the app filter list is cleared.

**Kill switch.** `setKillSwitch` (0x1001a9c0, ordinal 10096) only stores a byte in the client object
(the same byte init sets from one of its arguments). No code in the DLL that reads that byte was
found. So the kill switch is either enforced by SgbMain or reaches the tunnel through a path this
pass did not find. Medium confidence: the search was by object offset.

**DNS.** The DLL gets a DNS server address at init (logged as `dnsserver:`) and, through the
firewall (0x10013f40, 0x10013150), parses DNS queries inside captured UDP. It drops names on the
server-supplied `dnshack` list (plain or regex match). The resolver change itself (DNS set to
127.0.0.1 with a backup, restored on disconnect) is done by SgbMain, not here. No DNS restore code
exists in the DLL, so a crash leaves whatever SgbMain set.

**Other failure handling.** All exported calls write an error code and a Win32 error into the
object for SgbMain to read. Init refuses while the proxy is running. `startProxy` fails cleanly when
the WinDivert handle can't be opened (driver missing, or BFE stopped). The threads time out and log
`time out` when asked to stop (0x1001a040). The abuse rules (`updateAbuseConfig`, 0x1001aac0)
refuse to change while the proxy is running.
