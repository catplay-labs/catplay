//! Minimal Bluetooth diagnostics without BlueZ tools: `info` prints controller
//! settings via the mgmt socket; `monitor HOST:PORT` streams HCI monitor frames.
use std::{io::Write, net::TcpStream};
unsafe extern "C" {
    fn socket(d: i32, t: i32, p: i32) -> i32;
    fn bind(fd: i32, addr: *const u8, len: u32) -> i32;
    fn read(fd: i32, buf: *mut u8, n: usize) -> isize;
    fn write(fd: i32, buf: *const u8, n: usize) -> isize;
}
fn hci_socket(channel: u16) -> i32 {
    // AF_BLUETOOTH=31, SOCK_RAW=3 | SOCK_CLOEXEC, BTPROTO_HCI=1
    let fd = unsafe { socket(31, 3 | 0x80000, 1) };
    assert!(fd >= 0, "socket: {}", std::io::Error::last_os_error());
    // struct sockaddr_hci { u16 family; u16 dev; u16 channel; }
    let mut sa = [0u8; 6];
    sa[0..2].copy_from_slice(&31u16.to_ne_bytes());
    sa[2..4].copy_from_slice(&0xffffu16.to_ne_bytes()); // HCI_DEV_NONE
    sa[4..6].copy_from_slice(&channel.to_ne_bytes());
    assert!(unsafe { bind(fd, sa.as_ptr(), 6) } == 0, "bind: {}", std::io::Error::last_os_error());
    fd
}
const SETTINGS: [&str; 18] = ["powered", "connectable", "fast-connectable", "discoverable", "bondable",
    "link-security", "ssp", "br/edr", "hs", "le", "advertising", "secure-conn", "debug-keys", "privacy",
    "configuration", "static-addr", "phy-configuration", "wideband-speech"];
fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("info") => {
            let fd = hci_socket(3); // HCI_CHANNEL_CONTROL
            // Read Controller Information: opcode 0x0004, index 0, len 0
            let cmd = [0x04u8, 0x00, 0x00, 0x00, 0x00, 0x00];
            unsafe { write(fd, cmd.as_ptr(), cmd.len()) };
            let mut buf = [0u8; 512];
            loop {
                let n = unsafe { read(fd, buf.as_mut_ptr(), buf.len()) };
                if n < 6 { continue; }
                let event = u16::from_le_bytes([buf[0], buf[1]]);
                // Command Complete (0x0001) for opcode 0x0004
                if event == 0x0001 && u16::from_le_bytes([buf[6], buf[7]]) == 0x0004 {
                    let status = buf[8];
                    let p = &buf[9..];
                    let addr: Vec<String> = p[0..6].iter().rev().map(|b| format!("{b:02X}")).collect();
                    let supported = u32::from_le_bytes([p[9], p[10], p[11], p[12]]);
                    let current = u32::from_le_bytes([p[13], p[14], p[15], p[16]]);
                    let names = |m: u32| (0..18).filter(|i| m & (1 << i) != 0).map(|i| SETTINGS[i]).collect::<Vec<_>>().join(" ");
                    println!("status={status} addr={} version={}", addr.join(":"), p[6]);
                    println!("supported: {}", names(supported));
                    println!("current:   {}", names(current));
                    break;
                }
            }
        }
        Some("monitor") => {
            let fd = hci_socket(2); // HCI_CHANNEL_MONITOR
            let mut out = TcpStream::connect(&args[2]).expect("connect");
            let mut buf = [0u8; 4096];
            loop {
                let n = unsafe { read(fd, buf.as_mut_ptr(), buf.len()) };
                if n <= 0 { continue; }
                let n = n as usize;
                let up = std::fs::read_to_string("/proc/uptime").unwrap_or_default();
                let up = up.split_whitespace().next().unwrap_or("?");
                // monitor header: opcode u16, index u16, len u16
                let hex: String = buf[..n].iter().map(|b| format!("{b:02x}")).collect();
                if writeln!(out, "B {up} {hex}").is_err() { return; }
            }
        }
        Some(cmd @ ("sc" | "power")) => {
            // Set Powered 0x0005, Set Secure Connections 0x002D: index 0, one-byte value
            let opcode: u16 = if cmd == "power" { 0x0005 } else { 0x002d };
            let value = if args.get(2).map(String::as_str) == Some("on") { 1u8 } else { 0u8 };
            let fd = hci_socket(3);
            let o = opcode.to_le_bytes();
            let packet = [o[0], o[1], 0x00, 0x00, 0x01, 0x00, value];
            unsafe { write(fd, packet.as_ptr(), packet.len()) };
            let mut buf = [0u8; 64];
            loop {
                let n = unsafe { read(fd, buf.as_mut_ptr(), buf.len()) };
                if n < 9 { continue; }
                let event = u16::from_le_bytes([buf[0], buf[1]]);
                if (event == 0x0001 || event == 0x0002) && u16::from_le_bytes([buf[6], buf[7]]) == opcode {
                    println!("{cmd}={value} status={}", buf[8]);
                    break;
                }
            }
        }
        _ => eprintln!("usage: bt-diag info | monitor HOST:PORT | sc on|off | power on|off"),
    }
}
