//! Network: the `ping` command and the optional fastest-DNS boost step.
//!
//! **`ping`.** Times TCP connects (port 443, median of [`SAMPLES`]) to public cloud region
//! endpoints and prints them fastest first. EA doesn't publish per-region pingable endpoints for
//! BF6, so these are AWS regional DynamoDB endpoints: an approximation of "how far is each region",
//! not the game server itself. Changes nothing. BF6 picks its region in its own settings.
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

/// Median TCP connect time to `host:443`. The name is resolved once, before timing.
pub fn tcp_ms(host: &str) -> Result<f64, String> {
    let addr = (host, 443).to_socket_addrs().map_err(|e| e.to_string())?.next().ok_or("no address")?;
    timed(|_| TcpStream::connect_timeout(&addr, TIMEOUT).map(drop).map_err(|e| e.to_string()))
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
pub fn ping(hosts: &[String]) {
    println!("Latency to cloud regions (game servers are hosted near these), median of {SAMPLES} TCP connects:");
    let mut res: Vec<(&String, Result<f64, String>)> =
        std::thread::scope(|s| hosts.iter().map(|h| (h, s.spawn(|| tcp_ms(h)))).collect::<Vec<_>>().into_iter().map(|(h, t)| (h, t.join().unwrap())).collect());
    res.sort_by(|a, b| match (&a.1, &b.1) {
        (Ok(x), Ok(y)) => x.total_cmp(y),
        (a, b) => b.is_ok().cmp(&a.is_ok()),
    });
    for (host, r) in res {
        let label = DEFAULT_PING_HOSTS.iter().find(|(h, _)| h == host).map_or("", |(_, l)| l);
        match r {
            Ok(ms) => println!("  {ms:6.0} ms  {label:26} {host}"),
            Err(e) => println!("  {:>9}  {label:26} {host} ({e})", "failed"),
        }
    }
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
        ping(&default_ping_hosts());
    }

    #[test]
    fn guid_roundtrip() {
        let s = "{4D36E972-E325-11CE-BFC1-08002BE10318}";
        assert_eq!(guid_string(parse_guid(s).unwrap()), s);
    }
}
