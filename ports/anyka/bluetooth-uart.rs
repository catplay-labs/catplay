//! Attach the stock radio UART to Linux's H4 line discipline for bring-up.
//! Requires bluetooth.ko and hci_uart.ko; the process must keep the UART open.
use std::{ffi::c_int, fs::OpenOptions, io::{Read, Write}, os::fd::AsRawFd};

#[repr(C)]
struct PollFd { fd: c_int, events: i16, revents: i16 }

unsafe extern "C" {
    fn tcgetattr(fd: c_int, output: *mut u32) -> c_int;
    fn tcsetattr(fd: c_int, action: c_int, input: *const u32) -> c_int;
    fn cfmakeraw(termios: *mut u32);
    fn cfsetispeed(termios: *mut u32, speed: u32) -> c_int;
    fn cfsetospeed(termios: *mut u32, speed: u32) -> c_int;
    fn ioctl(fd: c_int, request: usize, arg: usize) -> c_int;
    fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int;
    fn close(fd: c_int) -> c_int;
    fn poll(fds: *mut PollFd, count: usize, timeout: c_int) -> c_int;
}

fn uart_read(uart: &mut std::fs::File, bytes: &mut [u8]) {
    for byte in bytes {
        let mut p = PollFd { fd: uart.as_raw_fd(), events: 1, revents: 0 };
        assert!(unsafe { poll(&mut p, 1, 2000) } > 0, "HCI UART response timed out");
        uart.read_exact(std::slice::from_mut(byte)).expect("HCI UART read");
    }
}

fn set_address(uart: &mut std::fs::File, address: &str) {
    let octets: Vec<u8> = address.trim().split(':')
        .map(|s| u8::from_str_radix(s, 16).expect("Bluetooth address octet")).collect();
    assert_eq!(octets.len(), 6, "Bluetooth address length");
    // AIC's volatile Write BD_ADDR command, before Linux registers the HCI device.
    let mut command = vec![1, 0x70, 0xfc, 6];
    command.extend(octets.iter().rev());
    uart.write_all(&command).expect("HCI address command");
    loop {
        let mut kind = [0];
        uart_read(uart, &mut kind);
        if kind[0] != 4 { continue; }
        let mut header = [0; 2];
        uart_read(uart, &mut header);
        let mut body = vec![0; header[1] as usize];
        uart_read(uart, &mut body);
        if header[0] == 0x0e && body.len() >= 4 && body[1..3] == [0x70, 0xfc] {
            assert_eq!(body[3], 0, "AIC Write BD_ADDR status");
            println!("Configured Bluetooth address {}", address.trim());
            return;
        }
    }
}

fn check(result: c_int, operation: &str) -> c_int {
    assert!(result >= 0, "{operation}: {}", std::io::Error::last_os_error());
    result
}

fn main() {
    let mut uart = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/ttySAK1")
        .expect("radio UART");
    let fd = uart.as_raw_fd();
    // Opaque, aligned storage larger than the ARM musl termios structure.
    let mut termios = [0u32; 32];
    check(unsafe { tcgetattr(fd, termios.as_mut_ptr()) }, "tcgetattr");
    unsafe { cfmakeraw(termios.as_mut_ptr()) };
    // Linux ARM B115200. Cold boot has no vendor daemon to set this speed.
    check(unsafe { cfsetispeed(termios.as_mut_ptr(), 0x1002) }, "input baud");
    check(unsafe { cfsetospeed(termios.as_mut_ptr(), 0x1002) }, "output baud");
    check(unsafe { tcsetattr(fd, 0, termios.as_ptr()) }, "tcsetattr");
    let args: Vec<String> = std::env::args().collect();
    let address = if let Some(option) = args.windows(2).find(|pair| pair[0] == "--address") {
        option[1].clone()
    } else {
        let path = "/usr/data/catplay/bluetooth-address";
        std::fs::read_to_string(path).unwrap_or_else(|_| {
            let wifi = std::fs::read_to_string("/sys/class/net/wlan0/address")
                .expect("Wi-Fi address for Bluetooth identity");
            let mut octets: Vec<u8> = wifi.trim().split(':')
                .map(|s| u8::from_str_radix(s, 16).unwrap()).collect();
            assert_eq!(octets.len(), 6);
            // A stable locally administered address for unprovisioned boards.
            octets[0] = (octets[0] | 2) & 0xfe;
            let address = octets.iter().map(|n| format!("{n:02X}"))
                .collect::<Vec<_>>().join(":");
            std::fs::create_dir_all("/usr/data/catplay").expect("Bluetooth state directory");
            std::fs::write(path, format!("{address}\n")).expect("Persist Bluetooth address");
            address
        })
    };
    set_address(&mut uart, &address);
    let discipline: c_int = 15; // N_HCI
    check(unsafe { ioctl(fd, 0x5423, &discipline as *const c_int as usize) }, "TIOCSETD");
    check(unsafe { ioctl(fd, 0x400455c8, 0) }, "HCIUARTSETPROTO H4");
    let id = check(unsafe { ioctl(fd, 0x800455ca, 0) }, "HCIUARTGETDEVICE");
    println!("Attached /dev/ttySAK1 as hci{id} at 115200 baud");
    if std::env::args().any(|arg| arg == "--up") {
        let sock = check(unsafe { socket(31, 3, 1) }, "HCI socket");
        let result = unsafe { ioctl(sock, 0x400448c9, id as usize) };
        if result < 0 {
            eprintln!("HCIDEVUP: {}", std::io::Error::last_os_error());
        } else {
            println!("hci{id} is up");
        }
        unsafe { close(sock) };
    }
    loop {
        std::thread::park();
    }
}
