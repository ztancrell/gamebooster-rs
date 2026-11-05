# Network: ping and fastest DNS

gamebooster does not copy Smart Game Booster's network accelerator (vendor relay servers plus a
packet driver). Instead it has small, honest tools: `ping`, `ping --route` and fastest DNS. Code:
`src/network.rs`.

## Ping

`gamebooster ping` measures how far away each region is and prints the regions fastest first.
It changes nothing and needs no administrator rights (it does read the config file, for
`ping_hosts`).

**What it measures.** For each host (all hosts in parallel), it resolves the name once, then times
`ping_samples` (default 10) TCP connects to port 443, one after another, each with a 2 s timeout.
A TCP connect is one network round trip (SYN, SYN-ACK), so this is close to what `ping` would show,
without needing ICMP (which many cloud hosts drop). Per host it prints:

- **median**: the typical round trip, in ms.
- **p90**: 9 in 10 connects were this fast or faster. A p90 far above the median means spikes.
- **jitter**: the average change between one connect and the next (the mean absolute difference
  of consecutive samples, as in RFC 3550), in ms. In a game, jitter feels like rubber-banding even
  when the median is fine.
- **loss**: connects that failed or timed out, out of the number sent (`1/10`).

A line ends in `<- unstable` when it has any loss or jitter above 10 ms. One unstable far region
(Asia from the US, say) is normal; loss or jitter on *every* region, including the nearest, points
at your own connection: run `ping --route` to see where.

**What the hosts are.** EA does not publish per-region pingable endpoints for Battlefield 6 (as of
November 2026 a search finds none; an
[EA forum request](https://forums.ea.com/discussions/battlefield-6-technical-issues-en/severe-packet-loss-40-in-battlefield-6---need-server-ips-for-isp/13174133)
for server IPs went unanswered). So the defaults are public AWS regional endpoints, labelled in the
output as "cloud regions (game servers are hosted near these)". Every default host is taken from
AWS's own list, [Amazon DynamoDB endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/ddb.html):

| Host | Region |
|---|---|
| `dynamodb.us-east-1.amazonaws.com` | US East (N. Virginia) |
| `dynamodb.us-west-1.amazonaws.com` | US West (N. California) |
| `dynamodb.us-west-2.amazonaws.com` | US West (Oregon) |
| `dynamodb.ca-central-1.amazonaws.com` | Canada (Central) |
| `dynamodb.sa-east-1.amazonaws.com` | South America (Sao Paulo) |
| `dynamodb.eu-west-1.amazonaws.com` | Europe (Ireland) |
| `dynamodb.eu-west-2.amazonaws.com` | Europe (London) |
| `dynamodb.eu-central-1.amazonaws.com` | Europe (Frankfurt) |
| `dynamodb.me-central-1.amazonaws.com` | Middle East (UAE) |
| `dynamodb.ap-northeast-1.amazonaws.com` | Asia Pacific (Tokyo) |
| `dynamodb.ap-southeast-1.amazonaws.com` | Asia Pacific (Singapore) |
| `dynamodb.ap-southeast-2.amazonaws.com` | Asia Pacific (Sydney) |

(`dynamodb.me-south-1`, Bahrain, is also on that list but didn't accept connections when this was
written, so it's left out.) Replace the list with `ping_hosts = [...]` in the config.

**Limits.** This is an approximation. Third-party guides say BF6 may run on AWS, but EA hasn't
confirmed where its servers are, and a game server in a region can be in a different data center
from these endpoints. In-game ping also includes the game's own server tick and processing. Use the
result to see which regions are near you, then choose BF6's server region in the game's own
settings; gamebooster can't change it.

Example output:

```
Latency to cloud regions (game servers are hosted near these), 10 TCP connects each:
     median   p90 jitter  loss
      19 ms    20    0.5  0/10 US East (N. Virginia)      dynamodb.us-east-1.amazonaws.com
      32 ms    33    0.4  0/10 Canada (Central)           dynamodb.ca-central-1.amazonaws.com
      61 ms    61    0.3  0/10 US West (Oregon)           dynamodb.us-west-2.amazonaws.com
  ...
     197 ms   230   22.9  0/10 Asia Pacific (Tokyo)       dynamodb.ap-northeast-1.amazonaws.com  <- unstable
     failed  Some region                some.host (failed to lookup address information ...)
p90: 9 in 10 connects were this fast or faster. Jitter: average change between connects (ms).
Loss: failed or timed-out connects. "<- unstable" = any loss or jitter above 10 ms.
This is an approximation, not BF6's servers. BF6 picks its server region in the game's own settings.
```

## Route check

`gamebooster ping --route [host]` answers "is my problem my Wi-Fi, my ISP, or the far end?", like
`pathping` or `mtr`. Without a host it first does a quick `ping` (3 connects per region) and uses
the fastest region. Changes nothing, no administrator rights (Windows' ICMP helper,
`IcmpSendEcho2`, needs none).

**What it does.** It sends ICMP echoes to the host's IPv4 address with TTL 1, 2, 3 ... up to 30;
each router on the way answers "TTL expired", which reveals the hops (two tries per hop, 1 s each;
it stops at the destination or after 4 silent hops in a row). Then it pings every hop that answered
`ping_samples` times (200 ms apart, all hops in parallel) and prints each hop's median, p90 and
loss, with its reverse-DNS name when one comes back within about half a second. Hop 1 is labelled
`your router`, with `(Wi-Fi)` or `(Ethernet)` for the adapter that carries the route.

Example (shape, addresses made up):

```
Route to dynamodb.us-east-1.amazonaws.com (3.218.182.10), 10 ICMP echoes per hop:
  hop    median   p90  loss  address
    1      2 ms     4  0/10  192.168.1.1  your router (Wi-Fi)
    2     11 ms    14  0/10  100.64.0.1
    3     12 ms    15  3/10  68.86.1.1 be-301-ar01.example.comcast.net
    4         *                    no answer
    5     19 ms    21  0/10  52.93.1.1
The destination didn't answer ICMP: many cloud endpoints drop it. Judge by the last hops that answer.
Loss only at some intermediate hops, not through to the end: usually routers rate-limiting ICMP replies, not real loss.
Loss or latency that starts at a hop and persists to the end points at that hop: hop 1 = your Wi-Fi/LAN,
the next few = your ISP, far hops = the remote network. See docs/tweaks/network.md.
```

**How to read it.**

- **Loss at one hop but not at later ones** is almost always that router rate-limiting ICMP
  replies to itself (routers answer pings slowly and last), not loss of your game traffic. Only loss
  that **starts at a hop and continues on every hop after it, to the end**, is real.
- **Where it starts is who to blame.** Hop 1 (your router): Wi-Fi or the home network; try a cable,
  another channel, or moving closer. The next few hops: your ISP. Far hops: the network the game
  server is in, or the ISP's upstream. The tool prints this for persisting loss (`Loss starts at hop
  N and persists to the end: ...`). Latency works the same way: a jump that starts at a hop and
  stays to the end belongs to that hop; a single slow hop in the middle is again usually ICMP being
  handled slowly.
- **No answer from the destination.** Many cloud endpoints (AWS included) drop ICMP, so the last
  hops are often `*`. The tool says so; judge by the last hops that do answer.
- **`*` hops in the middle** don't answer ICMP at all; they are skipped, not counted as loss.

**When a relay (ExitLag and similar) can help.** A relay sends your game traffic through the
vendor's servers on a different path. It can only help when the problem starts in your ISP's
network or its peering on the way to the game (loss or a latency jump from an ISP hop onward that
persists to the end) and a different path avoids it. It can't help with problems at hop 1 (your
Wi-Fi or home network: the relayed traffic goes through the same router), and it adds little when
the route is already near-direct (the ~19 to 21 ms US East result above leaves almost nothing to
win). See [the ExitLag research](../exitlag/README.md).

## Fastest DNS

Off by default: `fastest_dns = true` in the config turns it on. Boost step 6, before game priority.

**What it does.** During boost planning (also in `--dry-run`), it finds the adapter Windows uses to
reach the internet (`GetBestInterface` to 8.8.8.8), reads its current IPv4 DNS server from
`HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\{adapter GUID}` (`NameServer`
if set statically, else `DhcpNameServer`), and times 5 lookups of `www.ea.com` (a raw UDP DNS query)
against it and against Cloudflare `1.1.1.1`, Google `8.8.8.8` and Quad9 `9.9.9.9`. It prints all the
medians. If the fastest public resolver is more than 10 ms **and** more than 20 % faster than the
current server, boost sets it as the adapter's static IPv4 DNS server with
`SetInterfaceDnsSettings` (Windows 10 2004 or later) and runs `ipconfig /flushdns`. Otherwise it
keeps the current DNS. If the current server doesn't answer, it changes nothing.

**Restore.** The state file records whether the old setting was a static list (and which) or DHCP.
Restore writes back the static list, or clears the static setting so DHCP applies again, and
flushes the cache.

**What it improves, and what not.** Only name lookups: the time before a connection starts
(launcher, matchmaking and web requests, loading pages). It does **not** lower in-game ping: once
connected, game traffic goes straight to the server's IP address, DNS isn't involved. Mostly the
lookups measured are cached at the resolver, so the timing is roughly your round trip to it; a
router's DNS can look fast while being slow for uncached names. A public resolver can also send you
to a different CDN location than your ISP's resolver would.

**IPv6** DNS settings are not touched. If your network has IPv6, Windows may still use the IPv6 DNS
servers it has for some lookups.

**Undo by hand** (from an administrator terminal; the adapter name is printed during boost and by
`gamebooster status`, or see `netsh interface show interface`):

```bat
netsh interface ip set dns "<adapter>" dhcp
ipconfig /flushdns
```

If you had a static DNS server before, use `netsh interface ip set dns "<adapter>" static <server>`
instead of `dhcp`.

**Measure.** `gamebooster boost --dry-run` prints the timings without changing anything. Run it a
few times at different times of day; if the winner changes each time, the difference doesn't
matter.
