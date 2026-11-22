//! Push the app log and kernel messages to a TCP listener as they appear,
//! so the last lines before a hard hang reach another machine.
use std::{fs::{File, OpenOptions}, io::{Read, Seek, SeekFrom, Write}, net::TcpStream, os::unix::fs::OpenOptionsExt};
unsafe extern "C" { fn poll(fds: *mut u8, count: usize, timeout: i32) -> i32; }
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut out = TcpStream::connect(&args[1]).expect("connect");
    out.set_nodelay(true).ok();
    let mut kmsg = OpenOptions::new().read(true).custom_flags(0x800).open("/dev/kmsg").expect("kmsg");
    kmsg.seek(SeekFrom::End(0)).ok();
    let mut app = File::open("/tmp/catplay.log").expect("app log");
    app.seek(SeekFrom::End(0)).ok();
    let mut buf = [0u8; 8192];
    loop {
        let mut any = false;
        loop {
            match kmsg.read(&mut buf) {
                Ok(n) if n > 0 => { any = true; let _ = out.write_all(b"K "); if out.write_all(&buf[..n]).is_err() { return; } }
                Err(e) if e.raw_os_error() == Some(32) => continue,
                _ => break,
            }
        }
        loop {
            match app.read(&mut buf) {
                Ok(n) if n > 0 => { any = true; if out.write_all(&buf[..n]).is_err() { return; } }
                _ => break,
            }
        }
        if !any { unsafe { poll(std::ptr::null_mut(), 0, 5); } }
    }
}
