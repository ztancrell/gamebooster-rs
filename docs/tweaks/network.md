# Network: ping and fastest DNS

gamebooster does not copy Smart Game Booster's network accelerator (vendor relay servers plus a
packet driver). Instead it has two small, honest tools. Code: `src/network.rs`.

## Ping

`gamebooster ping` measures how far away each region is and prints the regions fastest first.
It changes nothing and needs no administrator rights (it does read the config file, for
`ping_hosts`).

**What it measures.** For each host, it resolves the name once, then times 5 TCP connects to port
443 and reports the median in ms. A TCP connect is one network round trip (SYN, SYN-ACK), so this is
close to what `ping` would show, without needing ICMP (which many cloud hosts drop).

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
Latency to cloud regions (game servers are hosted near these), median of 5 TCP connects:
      19 ms  US East (N. Virginia)      dynamodb.us-east-1.amazonaws.com
      32 ms  Canada (Central)           dynamodb.ca-central-1.amazonaws.com
      63 ms  US West (Oregon)           dynamodb.us-west-2.amazonaws.com
  ...
     failed  Some region                some.host (connection timed out)
This is an approximation, not BF6's servers. BF6 picks its server region in the game's own settings.
```

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
