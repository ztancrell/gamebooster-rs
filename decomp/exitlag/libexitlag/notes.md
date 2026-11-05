# libexitlag.dll 3.27.1 (routing core)

"API for proxying connections through ExitLag". C++20 (Boost.Asio coroutines), OpenSSL 3, Protobuf.
3,886,776 bytes, sha256 `f5e1de65de6945f60d083b7a56127d30dbd1618c2664f03bf87229140d1afcce`. The
DLL exports its C++ classes with full decorated names, so the class and method names below are
ExitLag's own. Evidence is from exports, imports and log/format strings; no function was
decompiled.

## Vocabulary (from class, protobuf and log names)

- **BS / bridge server**: ExitLag relay node the client connects to.
- **FS / final server**: ExitLag exit node near the game's data centre; it opens the real
  connection to the game server.
- **DS**: the game ("destination") server. **DC**: a data centre (ExitLag's map of where game
  servers live). **LS**: the local proxy inside `ExitLag.exe`.
- Protobuf package `ExitLag_WebService_Protocol_L2`: `BridgeServer` (with `DualBridgeRoute`),
  `FinalServer` (with `PingToDS`), `DataCenter`, `IpGroup`, `App` (ports, port ranges, regions,
  `PortRangeDisallowedForExpensiveBridges`), `RemotePings2/3` (`FromBridge`, `FromFinal`,
  `FromDataCenter`, `FromIpGroup`), `UserPingsReport`, `UserToDatacenterPingsReport`,
  `AppConnReport.Route`, `CompareHardwareIDRequest`, `UpdateRequest/Response`, `RemoteConfig`,
  `CommunityServer`, `AuthRequest` (email, Google, Apple, Discord).

## Route: client to bridge(s) to final server to game

- One `BridgePool` per proxied connection (logged with the game PID and destination). Up to **two
  bridges in series** are supported ("at most 2 BS in a route", "Better dual-bridge route: [You] x
  ms [bridge] y ms [bridge] z ms [FS]").
- The pool keeps **several bridge sockets in parallel** (per-app settings `tcp_bridges` and
  `udp_bridges`; option `multibridge_routes`; UI "Multipath"). Each outbound packet gets a sequence
  number and is sent to the bridges (`send_to_bridges`); the receive side drops duplicates
  ("Received duplicate packet ... from BS", "out-of-order duplicate"), requests retransmission of
  gaps (`send_retransmission_request`, ranges) and confirms delivery (`send_confirmation`,
  "Received confirmation ... (RTT: x ms)"). That is **redundant multipath**: the same game packet
  travels over several relay paths and the first copy wins, which hides loss on any single path at
  the cost of 2x or more upstream bandwidth.
- Bridge sockets can be TCP or UDP, IPv4 or IPv6, and can be bound to a specific local network
  interface (`set_multi_internet_interfaces`): "Multi Internet" sends the copies over two
  connections at once (e.g. Ethernet and a phone hotspot).
- Per-socket RTT and jitter windows (`has_stable_rtt`, `has_high_jitter`); `optimize_routes_if_
  beneficial` and `check_pending_route_optimization` switch a live flow to a spare bridge when it
  is better by a decaying threshold; on failure it looks for a spare route ("No spare route found,
  current route will be reused").
- Keep-alives, TCP FIN/RST forwarding, "SDR ping" detection (Valve Steam Datagram Relay probe
  packets are recognised and truncated), `set_high_udp_ports`, and
  `set_qualitative_socket_qos` (the client sets QoS on its own sockets via `SIO_SET_QOS`).
- **Bridge tiers**: apps carry `excluded_bridge_tiers` (exclude completely, or "if not better") and
  ports/port ranges "disallowed for expensive bridges": premium/expensive relays are only used for
  some traffic.

## Encryption

The only crypto calls are RSA-OAEP/SHA-256 public-key encryption (used for the login password,
"encrypted_password") and TLS via OpenSSL for HTTPS and WebSocket traffic to the web service. **No
symmetric cipher is imported or implemented visibly for the bridge data path**, so relayed game
packets appear to travel in ExitLag's own framing without an encryption layer. (Game traffic is
usually encrypted by the game itself.) ExitLag is therefore a game-traffic relay, not a VPN, and
offers no privacy property.

## Pings and route choice

- `NetworkMap` holds pings from the client to every bridge (measured locally by `BridgePinger`
  over UDP, with an ICMP pinger and a dual-stack variant; background schedule of N bridges every S
  seconds, "high-potential" bridges first) and **remote pings fetched from ExitLag's servers**:
  bridge to bridge, bridge to final server, final server to data centre or IP group
  (`get_ping_from_client_to_bridge`, `get_ping_from_bridge_to_bridge`,
  `get_ping_from_bridge_to_final`, `update_remote_pings`, gzipped `REMOTE_PINGS`).
- `get_best_initial_routes` / `get_best_fs` pick the route with the lowest **sum** of those legs.
  Option `use_datacenter_ip_to_measure_ping` means the last leg is measured to a data-centre
  address, not to the exact game server.
- Reports sent back: `UserPingsReport` and `UserToDatacenterPingsReport` (your pings to their
  network), and per connection min/avg/max proxied RTT plus a "direct RTT" (`send_direct_rtt`).
- Web service: list of endpoints set by the client, pinged and sorted ("Ping servers over
  HTTPS..."), server events over a WebSocket.

## Licensing, at a high level

Login is required to start (`get_authentication_token`, `login_again`); the account is bound to a
hardware ID (`get_user_hwid`, `CompareHardwareIDRequest`); the user's ASN, continent and IP
geolocation are fetched. No attempt was made to look at or around the checks.
