//! Network: the `ping` command and the optional fastest-DNS boost step.
//!
//! **`ping`.** Times `ping_samples` TCP connects (port 443) to public cloud region endpoints, in
//! parallel per host, and prints median, p90, jitter and loss per region, fastest first. EA doesn't publish per-region pingable endpoints for
//! BF6, so these are AWS regional DynamoDB endpoints: an approximation of "how far is each region",
//! not the game server itself. Changes nothing. BF6 picks its region in its own settings.
//!
//! **`ping --route [host]`.** mtr/pathping-style path check (Windows only, no admin): finds the hops
//! with ICMP echoes of increasing TTL (`IcmpSendEcho2`), then pings each hop `ping_samples` times
//! for median and loss, and says where persisting loss starts (router, ISP, far network).
//!
//! **Fastest DNS** (`fastest_dns = true`). Times DNS lookups (UDP, median of [`SAMPLES`]) against
//! the active adapter's current IPv4 DNS server and Cloudflare, Google and Quad9. If a public one is
//! faster by more than 10 ms *and* 20 %, sets it as the adapter's static IPv4 DNS server with
//! `SetInterfaceDnsSettings` and flushes the DNS cache. The state file records whether the old
//! setting was a static list or DHCP; restore puts that back. Speeds up name lookups (connecting,
//! loading web pages), never in-game ping. IPv6 DNS is left alone.
//!
//! **Undo by hand.** `netsh interface ip set dns "<adapter>" dhcp` (or `static <old server>`),
//! then `ipconfig /flushdns`. Details: docs/tweaks/network.md.

use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, TcpStream, ToSocketAddrs, UdpSocket};
use std::time::{Duration, Instant};

pub const SAMPLES: usize = 5;
const TIMEOUT: Duration = Duration::from_secs(2);

/// (host, label). Source: AWS, "Amazon DynamoDB endpoints and quotas",
/// https://docs.aws.amazon.com/general/latest/gr/ddb.html
pub const DEFAULT_PING_HOSTS: &[(&str, &str)] = &[
    ("dynamodb.us-east-1.amazonaws.com", "US East (N. Virginia)"),
    ("dynamodb.us-west-1.amazonaws.com", "US West (N. California)"),
    ("dynamodb.us-west-2.amazonaws.com", "US West (Oregon)"),
    ("dynamodb.ca-central-1.amazonaws.com", "Canada (Central)"),
    ("dynamodb.sa-east-1.amazonaws.com", "South America (Sao Paulo)"),
    ("dynamodb.eu-west-1.amazonaws.com", "Europe (Ireland)"),
    ("dynamodb.eu-west-2.amazonaws.com", "Europe (London)"),
    ("dynamodb.eu-central-1.amazonaws.com", "Europe (Frankfurt)"),
    ("dynamodb.me-central-1.amazonaws.com", "Middle East (UAE)"),
    ("dynamodb.ap-northeast-1.amazonaws.com", "Asia Pacific (Tokyo)"),
    ("dynamodb.ap-southeast-1.amazonaws.com", "Asia Pacific (Singapore)"),
    ("dynamodb.ap-southeast-2.amazonaws.com", "Asia Pacific (Sydney)"),
];

pub fn default_ping_hosts() -> Vec<String> {
    DEFAULT_PING_HOSTS.iter().map(|(h, _)| h.to_string()).collect()
}

/// Public resolvers fastest-DNS compares against (address, name).
pub const PUBLIC_DNS: &[(&str, &str)] = &[("1.1.1.1", "Cloudflare"), ("8.8.8.8", "Google"), ("9.9.9.9", "Quad9")];

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DnsState {
    /// Adapter GUID, e.g. "{4D36E972-E325-11CE-BFC1-08002BE10318}".
    pub adapter_guid: String,
    /// Friendly name (as in `netsh`), for messages and the manual undo.
    pub adapter: String,
    /// The static IPv4 DNS list before the boost ("1.2.3.4,5.6.7.8"), or None = from DHCP.
    pub previous_static: Option<String>,
    /// The server boost set.
    pub server: String,
}

pub fn median(mut v: Vec<f64>) -> Option<f64> {
    v.sort_by(|a, b| a.total_cmp(b));
    let n = v.len();
    match n {
        0 => None,
        _ if n % 2 == 1 => Some(v[n / 2]),
        _ => Some((v[n / 2 - 1] + v[n / 2]) / 2.0),
    }
}

/// Index of the public resolver to switch to: the fastest one, only if it beats the current
/// server by more than 10 ms and more than 20 %.
pub fn pick_dns(current_ms: f64, public_ms: &[Option<f64>]) -> Option<usize> {
    let (i, best) = public_ms.iter().enumerate().filter_map(|(i, m)| m.map(|m| (i, m))).min_by(|a, b| a.1.total_cmp(&b.1))?;
    (current_ms - best > 10.0 && best < current_ms * 0.8).then_some(i)
}

/// Median time of the successful runs of `f` (each returns Err on failure), or the last error.
fn timed(mut f: impl FnMut(usize) -> Result<(), String>) -> Result<f64, String> {
    let (mut ok, mut err) = (vec![], String::new());
    for i in 0..SAMPLES {
        let t = Instant::now();
        match f(i) {
            Ok(()) => ok.push(t.elapsed().as_secs_f64() * 1000.0),
            Err(e) => err = e,
        }
    }
    median(ok).ok_or(err)
}

/// Summary of N probes.
#[derive(Debug, PartialEq)]
pub struct Stats {
    pub median: f64,
    pub p90: f64,
    /// Mean absolute difference between consecutive successful samples (RFC 3550-style), ms.
    pub jitter: f64,
    pub lost: usize,
    pub sent: usize,
}

impl Stats {
    /// Worth a mark in the output: any loss, or jitter above 10 ms.
    pub fn flagged(&self) -> bool {
        self.lost > 0 || self.jitter > 10.0
    }
}

/// Nearest-rank percentile (p in 0..=100).
pub fn percentile(v: &[f64], p: f64) -> Option<f64> {
    let mut v = v.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let rank = ((p / 100.0 * v.len() as f64).ceil() as usize).clamp(1, v.len().max(1));
    v.get(rank - 1).copied()
}

pub fn jitter(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    v.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>() / (v.len() - 1) as f64
}

/// Stats of samples in send order (None = lost), or None if all were lost.
pub fn stats(samples: &[Option<f64>]) -> Option<Stats> {
    let ok: Vec<f64> = samples.iter().flatten().copied().collect();
    Some(Stats { p90: percentile(&ok, 90.0)?, jitter: jitter(&ok), median: median(ok.clone())?, lost: samples.len() - ok.len(), sent: samples.len() })
}

/// Lost count per hop (None = hop never answered, skipped). Returns the index of the hop where loss
/// starts and persists through the last answering hop, or None if the last answering hop has none.
pub fn loss_from(lost: &[Option<usize>]) -> Option<usize> {
    let mut start = None;
    for (i, l) in lost.iter().enumerate() {
        match l {
            Some(0) => start = None,
            Some(_) => _ = start.get_or_insert(i),
            None => {}
        }
    }
    start
}

/// Whose network hop number `hop` (1-based) usually is.
pub fn hop_owner(hop: usize) -> &'static str {
    match hop {
        1 => "your router: Wi-Fi or home network",
        2..=4 => "the first hops after your router: your ISP",
        _ => "further along the path: the ISP's upstream or the remote network",
    }
}

/// `n` TCP connect times to `host:443` in ms, None = failed or timed out. The name is resolved once.
pub fn tcp_samples(host: &str, n: usize) -> Result<Vec<Option<f64>>, String> {
    let addr = (host, 443).to_socket_addrs().map_err(|e| e.to_string())?.next().ok_or("no address")?;
    Ok((0..n)
        .map(|_| {
            let t = Instant::now();
            TcpStream::connect_timeout(&addr, TIMEOUT).ok().map(|_| t.elapsed().as_secs_f64() * 1000.0)
        })
        .collect())
}

/// Every host measured in parallel, fastest median first; failures last.
fn measure_all(hosts: &[String], n: usize) -> Vec<(&String, Result<Option<Stats>, String>)> {
    let mut res: Vec<_> = std::thread::scope(|s| {
        let threads: Vec<_> = hosts.iter().map(|h| (h, s.spawn(move || tcp_samples(h, n).map(|v| stats(&v))))).collect();
        threads.into_iter().map(|(h, t)| (h, t.join().unwrap())).collect()
    });
    let key = |r: &Result<Option<Stats>, String>| r.as_ref().ok().and_then(Option::as_ref).map_or(f64::INFINITY, |s| s.median);
    res.sort_by(|a, b| key(&a.1).total_cmp(&key(&b.1)));
    res
}

fn label(host: &str) -> &'static str {
    DEFAULT_PING_HOSTS.iter().find(|(h, _)| *h == host).map_or("", |(_, l)| l)
}

/// A DNS query for an A record (recursion desired).
fn dns_query(id: u16, name: &str) -> Vec<u8> {
    let mut q = id.to_be_bytes().to_vec();
    q.extend([1, 0, 0, 1, 0, 0, 0, 0, 0, 0]); // flags RD, 1 question
    for label in name.split('.') {
        q.push(label.len() as u8);
        q.extend(label.as_bytes());
    }
    q.extend([0, 0, 1, 0, 1]); // root, type A, class IN
    q
}

/// Median time for `server` to answer a lookup of www.ea.com.
pub fn dns_ms(server: Ipv4Addr) -> Result<f64, String> {
    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    sock.set_read_timeout(Some(TIMEOUT)).map_err(|e| e.to_string())?;
    sock.connect((server, 53)).map_err(|e| e.to_string())?;
    timed(|i| {
        let id = 0x6b00 + i as u16;
        sock.send(&dns_query(id, "www.ea.com")).map_err(|e| e.to_string())?;
        let mut buf = [0u8; 512];
        loop {
            let n = sock.recv(&mut buf).map_err(|e| e.to_string())?;
            if n >= 2 && buf[..2] == id.to_be_bytes() {
                return Ok(());
            }
        }
    })
}

/// `gamebooster ping`: measure every host in parallel, print fastest first.
pub fn ping(hosts: &[String], samples: usize) {
    println!("Latency to cloud regions (game servers are hosted near these), {samples} TCP connects each:");
    println!("  {:>9} {:>5} {:>6} {:>5}", "median", "p90", "jitter", "loss");
    for (host, r) in measure_all(hosts, samples) {
        let label = label(host);
        match r {
            Ok(Some(s)) => {
                let mark = if s.flagged() { "  <- unstable" } else { "" };
                println!("  {:6.0} ms {:5.0} {:6.1} {:>2}/{:<2} {label:26} {host}{mark}", s.median, s.p90, s.jitter, s.lost, s.sent);
            }
            Ok(None) => println!("  {:>9} {:>12} {samples:>2}/{samples:<2} {label:26} {host} (no connect succeeded)", "failed", ""),
            Err(e) => println!("  {:>9}  {label:26} {host} ({e})", "failed"),
        }
    }
    println!("p90: 9 in 10 connects were this fast or faster. Jitter: average change between connects (ms).");
    println!("Loss: failed or timed-out connects. \"<- unstable\" = any loss or jitter above 10 ms.");
    println!("This is an approximation, not BF6's servers. BF6 picks its server region in the game's own settings.");
}

/// "{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}" for a GUID as u128.
pub fn guid_string(g: u128) -> String {
    let h = format!("{g:032X}");
    format!("{{{}-{}-{}-{}-{}}}", &h[..8], &h[8..12], &h[12..16], &h[16..20], &h[20..])
}

pub fn parse_guid(s: &str) -> Option<u128> {
    u128::from_str_radix(&s.trim_matches(|c| c == '{' || c == '}').replace('-', ""), 16).ok()
}

#[cfg(windows)]
pub use sys::*;

#[cfg(windows)]
mod sys {
    use super::*;
    use crate::{from_wide, wide};
    use windows_sys::core::GUID;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::NetworkManagement::IpHelper::*;
    use windows_sys::Win32::NetworkManagement::Ndis::NET_LUID_LH;
    use windows_sys::Win32::System::Registry::*;

    const IFACES: &str = r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\";

    /// GUID and friendly name of the adapter Windows uses to reach the internet.
    fn active_adapter() -> Result<(GUID, String), String> {
        unsafe {
            let mut idx = 0;
            let e = GetBestInterface(u32::from_ne_bytes([8, 8, 8, 8]), &mut idx);
            if e != 0 {
                return Err(format!("no route to the internet (error {e})"));
            }
            let mut luid: NET_LUID_LH = std::mem::zeroed();
            let mut guid: GUID = std::mem::zeroed();
            let mut alias = [0u16; 257];
            if ConvertInterfaceIndexToLuid(idx, &mut luid) != ERROR_SUCCESS
                || ConvertInterfaceLuidToGuid(&luid, &mut guid) != ERROR_SUCCESS
                || ConvertInterfaceLuidToAlias(&luid, alias.as_mut_ptr(), alias.len()) != ERROR_SUCCESS
            {
                return Err(format!("can't identify network interface {idx}"));
            }
            Ok((guid, from_wide(&alias)))
        }
    }

    fn to_u128(g: &GUID) -> u128 {
        (g.data1 as u128) << 96 | (g.data2 as u128) << 80 | (g.data3 as u128) << 64 | u64::from_be_bytes(g.data4) as u128
    }

    /// A REG_SZ under the adapter's Tcpip key, "" if absent.
    fn reg_sz(guid: &str, value: &str) -> String {
        let (key, value) = (wide(&format!("{IFACES}{guid}")), wide(value));
        let mut buf = [0u16; 1024];
        let mut size = (buf.len() * 2) as u32;
        let e = unsafe {
            RegGetValueW(HKEY_LOCAL_MACHINE, key.as_ptr(), value.as_ptr(), RRF_RT_REG_SZ, std::ptr::null_mut(), buf.as_mut_ptr() as *mut _, &mut size)
        };
        if e == ERROR_SUCCESS { from_wide(&buf) } else { String::new() }
    }

    /// Read-only: time the current and public resolvers, print the timings, and return what boost
    /// will change (None = keep the current DNS).
    pub fn plan() -> Result<Option<DnsState>, String> {
        let (guid, adapter) = active_adapter()?;
        let adapter_guid = guid_string(to_u128(&guid));
        let stat = reg_sz(&adapter_guid, "NameServer");
        let current = if stat.is_empty() { reg_sz(&adapter_guid, "DhcpNameServer") } else { stat.clone() };
        let server: Ipv4Addr = current
            .split([',', ' '])
            .find_map(|s| s.parse().ok())
            .ok_or_else(|| format!("no IPv4 DNS server found for {adapter}"))?;
        let cur_ms = dns_ms(server).map_err(|e| format!("current DNS {server} doesn't answer: {e}"))?;
        let src = if stat.is_empty() { "DHCP" } else { "static" };
        println!("  DNS lookup, median of {SAMPLES} ({adapter}): current {server} ({src}) {cur_ms:.0} ms");
        let public: Vec<Option<f64>> = PUBLIC_DNS
            .iter()
            .map(|(ip, name)| {
                let r = dns_ms(ip.parse().unwrap());
                match &r {
                    Ok(ms) => println!("    {name} {ip}: {ms:.0} ms"),
                    Err(e) => println!("    {name} {ip}: failed ({e})"),
                }
                r.ok()
            })
            .collect();
        Ok(pick_dns(cur_ms, &public).map(|i| DnsState {
            adapter_guid,
            adapter,
            previous_static: (!stat.is_empty()).then_some(stat),
            server: PUBLIC_DNS[i].0.into(),
        }))
    }

    /// Set the adapter's static IPv4 DNS list; "" goes back to DHCP. Then flush the DNS cache.
    fn set(guid: &str, servers: &str) -> Result<(), String> {
        let g = GUID::from_u128(parse_guid(guid).ok_or_else(|| format!("bad adapter GUID {guid}"))?);
        let mut ns = wide(servers);
        let settings = DNS_INTERFACE_SETTINGS {
            Version: DNS_INTERFACE_SETTINGS_VERSION1,
            Flags: DNS_SETTING_NAMESERVER as u64,
            NameServer: ns.as_mut_ptr(),
            ..Default::default()
        };
        let e = unsafe { SetInterfaceDnsSettings(g, &settings) };
        if e != ERROR_SUCCESS {
            return Err(format!("SetInterfaceDnsSettings failed (error {e})"));
        }
        let _ = std::process::Command::new("ipconfig").arg("/flushdns").output();
        Ok(())
    }

    pub fn boost(s: &DnsState) -> Result<(), String> {
        set(&s.adapter_guid, &s.server)?;
        println!("  {}: DNS {} (cache flushed)", s.adapter, s.server);
        Ok(())
    }

    pub fn restore(s: &DnsState) -> Result<(), String> {
        set(&s.adapter_guid, s.previous_static.as_deref().unwrap_or(""))?;
        println!("  {}: DNS {} (cache flushed)", s.adapter, s.previous_static.as_deref().unwrap_or("from DHCP"));
        Ok(())
    }

    /// An ICMP handle (`IcmpCreateFile`), closed on drop.
    struct Icmp(windows_sys::Win32::Foundation::HANDLE);

    impl Icmp {
        fn new() -> Result<Icmp, String> {
            let h = unsafe { IcmpCreateFile() };
            if h == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
                return Err(format!("IcmpCreateFile failed (error {})", unsafe { windows_sys::Win32::Foundation::GetLastError() }));
            }
            Ok(Icmp(h))
        }

        /// One echo to `ip` with `ttl`: the replying address, whether it's the target itself (not a
        /// TTL-expired router), and the round trip in ms. None = no reply within `timeout_ms`.
        fn echo(&self, ip: Ipv4Addr, ttl: u8, timeout_ms: u32) -> Option<(Ipv4Addr, bool, f64)> {
            let data = *b"gamebooster ping";
            let opts = IP_OPTION_INFORMATION { Ttl: ttl, ..Default::default() };
            let mut buf = [0u64; 64]; // >= sizeof(ICMP_ECHO_REPLY) + data + 8, 8-byte aligned
            let t = Instant::now();
            unsafe {
                IcmpSendEcho2(self.0, std::ptr::null_mut(), None, std::ptr::null(), u32::from_ne_bytes(ip.octets()), data.as_ptr() as _, data.len() as u16, &opts,
                    buf.as_mut_ptr() as _, std::mem::size_of_val(&buf) as u32, timeout_ms);
            }
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            // Read the reply whatever the return value: TTL-expired replies come back as a "failure".
            let r = unsafe { &*(buf.as_ptr() as *const ICMP_ECHO_REPLY) };
            let from = Ipv4Addr::from(r.Address.to_ne_bytes());
            match r.Status {
                IP_SUCCESS | IP_TTL_EXPIRED_TRANSIT if !from.is_unspecified() => Some((from, r.Status == IP_SUCCESS, ms)),
                _ => None,
            }
        }
    }

    impl Drop for Icmp {
        fn drop(&mut self) {
            unsafe { IcmpCloseHandle(self.0) };
        }
    }

    /// Reverse DNS name, None if there is none (or WinSock isn't up).
    fn reverse_dns(ip: Ipv4Addr) -> Option<String> {
        use windows_sys::Win32::Networking::WinSock::*;
        let sa = SOCKADDR_IN { sin_family: AF_INET, sin_port: 0, sin_addr: IN_ADDR { S_un: IN_ADDR_0 { S_addr: u32::from_ne_bytes(ip.octets()) } }, sin_zero: [0; 8] };
        let mut buf = [0u8; 1025];
        let e = unsafe {
            getnameinfo(&sa as *const _ as *const SOCKADDR, std::mem::size_of::<SOCKADDR_IN>() as i32, buf.as_mut_ptr(), buf.len() as u32, std::ptr::null_mut(), 0, NI_NAMEREQD as i32)
        };
        (e == 0).then(|| std::ffi::CStr::from_bytes_until_nul(&buf).ok().map(|c| c.to_string_lossy().into_owned())).flatten()
    }

    /// "Wi-Fi" / "Ethernet" for the adapter that routes to `ip`, None if unknown.
    fn link_kind(ip: Ipv4Addr) -> Option<&'static str> {
        unsafe {
            let mut row: MIB_IF_ROW2 = std::mem::zeroed();
            if GetBestInterface(u32::from_ne_bytes(ip.octets()), &mut row.InterfaceIndex) != 0 || GetIfEntry2(&mut row) != ERROR_SUCCESS {
                return None;
            }
            match row.Type {
                IF_TYPE_IEEE80211 => Some("Wi-Fi"),
                IF_TYPE_ETHERNET_CSMACD => Some("Ethernet"),
                _ => None,
            }
        }
    }

    /// `gamebooster ping --route [host]`: per-hop latency and loss to `host`, or to the fastest of
    /// `hosts` from a quick ping run.
    pub fn route(hosts: &[String], host: Option<&str>, samples: usize) -> Result<(), String> {
        unsafe {
            let mut d = std::mem::zeroed();
            windows_sys::Win32::Networking::WinSock::WSAStartup(0x202, &mut d); // for getnameinfo
        }
        let host = match host {
            Some(h) => h.to_string(),
            None => {
                println!("Finding the fastest region (3 TCP connects each)...");
                let first = measure_all(hosts, 3).into_iter().find(|(_, r)| matches!(r, Ok(Some(_))));
                first.map(|(h, _)| h.clone()).ok_or("no ping_hosts host answered; give one: gamebooster ping --route <host>")?
            }
        };
        let ip = (host.as_str(), 0)
            .to_socket_addrs()
            .map_err(|e| format!("{host}: {e}"))?
            .find_map(|a| match a.ip() {
                std::net::IpAddr::V4(v) => Some(v),
                _ => None,
            })
            .ok_or_else(|| format!("{host} has no IPv4 address"))?;
        println!("Route to {host} ({ip}), {samples} ICMP echoes per hop:");
        let icmp = Icmp::new()?;
        // Discover hops: up to 2 tries per TTL; stop at the target or after 4 silent hops in a row.
        let (mut hops, mut reached) = (Vec::<Option<Ipv4Addr>>::new(), false);
        for ttl in 1..=30u8 {
            let r = icmp.echo(ip, ttl, 1000).or_else(|| icmp.echo(ip, ttl, 1000));
            hops.push(r.map(|(a, _, _)| a));
            if r.is_some_and(|(_, done, _)| done) {
                reached = true;
                break;
            }
            if hops.len() >= 4 && hops[hops.len() - 4..].iter().all(Option::is_none) {
                break;
            }
        }
        while hops.last() == Some(&None) {
            hops.pop();
        }
        // Measure each answering hop in parallel (direct echoes, 200 ms apart), names alongside.
        let measured: Vec<Option<(Ipv4Addr, Vec<Option<f64>>, Option<String>)>> = std::thread::scope(|s| {
            let threads: Vec<_> = hops
                .iter()
                .map(|h| {
                    h.map(|a| {
                        s.spawn(move || {
                            let (tx, rx) = std::sync::mpsc::channel();
                            std::thread::spawn(move || tx.send(reverse_dns(a)));
                            let v = Icmp::new().map_or(vec![None; samples], |icmp| {
                                (0..samples)
                                    .map(|i| {
                                        if i > 0 {
                                            std::thread::sleep(Duration::from_millis(200));
                                        }
                                        icmp.echo(a, 64, 1000).filter(|r| r.1).map(|r| r.2)
                                    })
                                    .collect()
                            });
                            (a, v, rx.recv_timeout(Duration::from_millis(500)).ok().flatten())
                        })
                    })
                })
                .collect();
            threads.into_iter().map(|t| t.map(|t| t.join().unwrap())).collect()
        });
        println!("  {:>3} {:>9} {:>5} {:>5}  address", "hop", "median", "p90", "loss");
        let mut lost = vec![];
        for (i, m) in measured.iter().enumerate() {
            let Some((a, v, name)) = m else {
                println!("  {:>3} {:>9}                    no answer", i + 1, "*");
                lost.push(None);
                continue;
            };
            let st = stats(v);
            lost.push(Some(st.as_ref().map_or(samples, |s| s.lost)));
            let name = name.as_deref().map_or(String::new(), |n| format!(" {n}"));
            let mut note = String::new();
            if i == 0 {
                note = format!("  your router{}", link_kind(ip).map_or(String::new(), |k| format!(" ({k})")));
            }
            if reached && i + 1 == measured.len() {
                note += "  (destination)";
            }
            match st {
                Some(s) => println!("  {:>3} {:6.0} ms {:5.0} {:>2}/{:<2}  {a}{name}{note}", i + 1, s.median, s.p90, s.lost, s.sent),
                None => println!("  {:>3} {:>9} {:>5} {samples:>2}/{samples:<2}  {a}{name}{note}", i + 1, "lost", ""),
            }
        }
        if !reached {
            println!("The destination didn't answer ICMP: many cloud endpoints drop it. Judge by the last hops that answer.");
        }
        match loss_from(&lost) {
            Some(i) if reached => println!("Loss starts at hop {} and persists to the end: {}.", i + 1, hop_owner(i + 1)),
            _ if lost.iter().flatten().any(|&l| l > 0) => {
                println!("Loss only at some intermediate hops, not through to the end: usually routers rate-limiting ICMP replies, not real loss.")
            }
            _ => println!("No loss on the path."),
        }
        println!("Loss or latency that starts at a hop and persists to the end points at that hop: hop 1 = your Wi-Fi/LAN,");
        println!("the next few = your ISP, far hops = the remote network. See docs/tweaks/network.md.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_odd_even_empty() {
        assert_eq!(median(vec![30.0, 10.0, 20.0]), Some(20.0));
        assert_eq!(median(vec![40.0, 10.0, 20.0, 30.0]), Some(25.0));
        assert_eq!(median(vec![]), None);
    }

    #[test]
    fn pick_dns_needs_10ms_and_20_percent() {
        assert_eq!(pick_dns(50.0, &[Some(30.0), Some(20.0), None]), Some(1));
        assert_eq!(pick_dns(50.0, &[Some(41.0)]), None); // 9 ms
        assert_eq!(pick_dns(100.0, &[Some(85.0)]), None); // 15 ms but 15 %
        assert_eq!(pick_dns(10.0, &[None, None]), None);
    }

    #[test]
    fn dns_query_packet() {
        let q = dns_query(0x1234, "www.ea.com");
        assert_eq!(&q[..4], &[0x12, 0x34, 1, 0]);
        assert_eq!(&q[12..], b"\x03www\x02ea\x03com\x00\x00\x01\x00\x01");
    }

    /// Live check against the real network: `cargo test live_network -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn live_network() {
        println!("1.1.1.1 dns: {:?}", dns_ms("1.1.1.1".parse().unwrap()));
        ping(&default_ping_hosts(), 10);
    }

    #[test]
    fn stats_jitter_loss_percentiles() {
        assert_eq!(jitter(&[10.0, 20.0, 15.0]), 7.5);
        assert_eq!(jitter(&[10.0]), 0.0);
        assert_eq!(percentile(&[5.0, 1.0, 4.0, 2.0, 3.0, 6.0, 7.0, 8.0, 9.0, 10.0], 90.0), Some(9.0));
        assert_eq!(percentile(&[3.0], 90.0), Some(3.0));
        assert_eq!(percentile(&[], 90.0), None);
        let s = stats(&[Some(10.0), None, Some(30.0), Some(20.0)]).unwrap();
        assert_eq!(s, Stats { median: 20.0, p90: 30.0, jitter: 15.0, lost: 1, sent: 4 });
        assert!(s.flagged());
        assert!(!stats(&[Some(10.0), Some(12.0)]).unwrap().flagged());
        assert!(stats(&[Some(10.0), Some(25.0)]).unwrap().flagged()); // jitter 15 ms
        assert_eq!(stats(&[None, None]), None);
    }

    #[test]
    fn persisting_loss() {
        assert_eq!(loss_from(&[Some(0), Some(2), Some(1), Some(3)]), Some(1));
        assert_eq!(loss_from(&[Some(0), Some(5), Some(0)]), None); // rate-limiting at hop 2 only
        assert_eq!(loss_from(&[Some(1), None, Some(2)]), Some(0)); // silent hops don't break it
        assert_eq!(loss_from(&[Some(0), Some(0)]), None);
        assert_eq!(loss_from(&[]), None);
        assert!(hop_owner(1).contains("router") && hop_owner(3).contains("ISP") && hop_owner(9).contains("remote"));
    }

    #[test]
    fn guid_roundtrip() {
        let s = "{4D36E972-E325-11CE-BFC1-08002BE10318}";
        assert_eq!(guid_string(parse_guid(s).unwrap()), s);
    }
}
