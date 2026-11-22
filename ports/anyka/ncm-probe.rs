//! Step-by-step traffic probe for the car's USB NCM link. Each step is
//! recorded to flash before and after it runs, so a hang names its step.
use std::{fs::OpenOptions, io::{Read, Write}, net::{Ipv6Addr, SocketAddrV6, TcpStream, UdpSocket},
          time::Duration};

unsafe extern "C" {
    fn if_nametoindex(name: *const u8) -> u32;
    fn socket(domain: i32, kind: i32, protocol: i32) -> i32;
    fn sendto(fd: i32, buf: *const u8, len: usize, flags: i32, addr: *const u8, addrlen: u32) -> isize;
    fn recv(fd: i32, buf: *mut u8, len: usize, flags: i32) -> isize;
    fn setsockopt(fd: i32, level: i32, name: i32, value: *const u8, len: u32) -> i32;
    fn close(fd: i32) -> i32;
    fn poll(fds: *mut u8, count: usize, timeout: i32) -> i32;
}

fn uptime() -> String {
    std::fs::read_to_string("/proc/uptime").unwrap_or_default().split_whitespace().next().unwrap_or("?").into()
}

fn record(line: &str) {
    let line = format!("uptime={} {line}\n", uptime());
    print!("{line}");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open("/usr/data/catplay/probe.log") {
        let _ = f.write_all(line.as_bytes());
        let _ = f.sync_data();
    }
}

fn ping(addr: Ipv6Addr, scope: u32, size: usize) -> Result<String, String> {
    let fd = unsafe { socket(10, 3, 58) }; // AF_INET6, SOCK_RAW, IPPROTO_ICMPV6
    if fd < 0 { return Err(std::io::Error::last_os_error().to_string()); }
    let timeout = [2i32, 0, 0, 0]; // struct timeval { 2 s }
    unsafe { setsockopt(fd, 1, 20, timeout.as_ptr() as _, 8) }; // SOL_SOCKET, SO_RCVTIMEO
    let mut packet = vec![0u8; 8 + size];
    packet[0] = 128; // echo request; the kernel fills in the checksum
    packet[4] = 0x12; packet[5] = 0x34; packet[7] = 1;
    let mut sa = [0u8; 28]; // sockaddr_in6
    sa[0..2].copy_from_slice(&10u16.to_ne_bytes());
    sa[8..24].copy_from_slice(&addr.octets());
    sa[24..28].copy_from_slice(&scope.to_ne_bytes());
    let sent = unsafe { sendto(fd, packet.as_ptr(), packet.len(), 0, sa.as_ptr(), 28) };
    if sent < 0 { let e = std::io::Error::last_os_error(); unsafe { close(fd) }; return Err(e.to_string()); }
    let mut buf = [0u8; 2048];
    let result = loop {
        let n = unsafe { recv(fd, buf.as_mut_ptr(), buf.len(), 0) };
        if n < 0 { break Err(std::io::Error::last_os_error().to_string()); }
        if n >= 8 && buf[0] == 129 { break Ok(format!("echo reply {n} bytes")); }
    };
    unsafe { close(fd) };
    result
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: ncm-probe IFACE ADDR STEP...\n  ping:SIZE  udp:PORT  tcp:PORT  tcpsend:PORT:BYTES  sleep:SECONDS");
        std::process::exit(2);
    }
    let iface = format!("{}\0", args[1]);
    let scope = unsafe { if_nametoindex(iface.as_ptr()) };
    let addr: Ipv6Addr = args[2].parse().expect("IPv6 address");
    record(&format!("PROBE iface={} scope={scope} addr={addr}", args[1]));
    for step in &args[3..] {
        let parts: Vec<&str> = step.split(':').collect();
        record(&format!("BEGIN {step}"));
        let target = |port: &str| SocketAddrV6::new(addr, port.parse().expect("port"), 0, scope);
        let result: Result<String, String> = match parts[0] {
            "ping" => ping(addr, scope, parts.get(1).unwrap_or(&"56").parse().expect("size")),
            "udp" => UdpSocket::bind("[::]:0").and_then(|s| s.send_to(&[0x55; 32], target(parts[1])))
                .map(|n| format!("sent {n}")).map_err(|e| e.to_string()),
            "tcp" | "tcpsend" => (|| {
                let mut s = TcpStream::connect(target(parts[1]))
                    .map_err(|e| format!("connect: {e}"))?;
                let mut note = String::from("connected");
                if parts[0] == "tcpsend" {
                    let bytes: usize = parts[2].parse().expect("bytes");
                    s.write_all(&vec![b'A'; bytes]).map_err(|e| format!("write: {e}"))?;
                    note += &format!(", wrote {bytes}");
                    s.set_read_timeout(Some(Duration::from_secs(2))).ok();
                    let mut buf = [0u8; 4096];
                    note += &match s.read(&mut buf) { Ok(n) => format!(", read {n}"), Err(e) => format!(", read: {e}") };
                }
                Ok(note)
            })(),
            "tcphold" => (|| {
                let s = TcpStream::connect(target(parts[1])).map_err(|e| format!("connect: {e}"))?;
                record(&format!("HOLDING tcp socket for {}s", parts[2]));
                unsafe { poll(std::ptr::null_mut(), 0, parts[2].parse::<i32>().expect("seconds") * 1000) };
                drop(s);
                Ok("held and closed".into())
            })(),
            "info" => (|| {
                let mut s = TcpStream::connect(target(parts[1])).map_err(|e| format!("connect: {e}"))?;
                let body: &[u8] = b"\x62\x70\x6c\x69\x73\x74\x30\x30\xd0\x08\x00\x00\x00\x00\x00\x00\x01\x01\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x09";
                let head = format!("GET /info RTSP/1.0\r\nCSeq: 1\r\nUser-Agent: AirPlay/566.25.21\r\nContent-Type: application/x-apple-binary-plist\r\nContent-Length: {}\r\n\r\n", body.len());
                s.write_all(&[head.as_bytes(), body].concat()).map_err(|e| format!("write: {e}"))?;
                s.set_read_timeout(Some(Duration::from_secs(3))).ok();
                let mut total = 0usize;
                let mut buf = [0u8; 4096];
                loop {
                    match s.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => { total += n; record(&format!("RX {n} (total {total})")); }
                        Err(_) => break,
                    }
                }
                Ok(format!("received {total} bytes"))
            })(),
            "sleep" => { unsafe { poll(std::ptr::null_mut(), 0, parts[1].parse::<i32>().expect("seconds") * 1000) }; Ok("slept".into()) }
            other => Err(format!("unknown step {other}")),
        };
        record(&format!("END {step} {result:?}"));
    }
}
