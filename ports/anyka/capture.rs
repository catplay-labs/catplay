//! Bounded, opt-in diagnostics on DATA for tests away from the development host.
use std::{fs::{self, File, OpenOptions}, io::{self, Read, Seek, SeekFrom, Write},
          os::unix::fs::OpenOptionsExt, path::{Path, PathBuf}};

unsafe extern "C" {
    fn poll(fds: *mut u8, count: usize, timeout: i32) -> i32;
    fn mlockall(flags: i32) -> i32;
    fn ioctl(fd: i32, request: u32, arg: *mut i32) -> i32;
}

/// Feed the hardware watchdog from memory-locked code that never touches flash.
/// A CPU hang then reboots the dongle; stuck flash I/O alone does not.
fn start_watchdog(timeout: i32) -> io::Result<()> {
    use std::os::fd::AsRawFd;
    let mut file = OpenOptions::new().write(true).open("/dev/watchdog")?;
    let mut value = timeout;
    // WDIOC_SETTIMEOUT
    if unsafe { ioctl(file.as_raw_fd(), 0xC0045706, &mut value) } < 0 {
        return Err(io::Error::last_os_error());
    }
    std::thread::spawn(move || loop {
        let _ = file.write_all(b"k");
        unsafe { poll(std::ptr::null_mut(), 0, 1000); }
    });
    Ok(())
}

struct Log { path: PathBuf, file: File, size: usize, limit: usize }
impl Log {
    fn new(path: PathBuf, limit: usize) -> io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let size = file.metadata()?.len() as usize;
        Ok(Self { path, file, size, limit })
    }
    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        let data = &data[data.len().saturating_sub(self.limit)..];
        if self.size + data.len() > self.limit {
            self.file.sync_data()?;
            fs::rename(&self.path, self.path.with_extension("previous.log"))?;
            self.file = OpenOptions::new().create(true).write(true).truncate(true).open(&self.path)?;
            self.size = 0;
        }
        self.file.write_all(data)?;
        self.size += data.len();
        Ok(())
    }
    fn sync(&self) -> io::Result<()> { self.file.sync_data() }
}

fn uptime() -> f64 {
    fs::read_to_string("/proc/uptime").ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok()).unwrap_or(0.)
}

fn status() -> String {
    let mut processes = Vec::new();
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if !entry.file_name().to_string_lossy().bytes().all(|b| b.is_ascii_digit()) { continue; }
            if let Ok(cmd) = fs::read(entry.path().join("cmdline")) {
                if cmd.windows(b"/opt/catplay/catplay_c2a".len())
                    .any(|s| s == b"/opt/catplay/catplay_c2a") {
                    processes.push(entry.file_name().to_string_lossy().into_owned());
                }
            }
        }
    }
    processes.sort();
    let phone = fs::read_to_string("/sys/class/iphone/default/status").unwrap_or_default();
    let udc = fs::read_to_string("/sys/class/udc/20200000.usb/state").unwrap_or_default();
    format!("app_pids={} iphone={} udc={} host={} device={}\n",
        processes.join(","), phone.trim(), udc.trim(),
        Path::new("/sys/module/ak_hcd").exists(), Path::new("/sys/module/ak_udc").exists())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let option = |name: &str, fallback: &str| args.windows(2)
        .find(|a| a[0] == name).map(|a| a[1].clone()).unwrap_or_else(|| fallback.into());
    let directory = PathBuf::from(option("--directory", "/usr/data/catplay/capture"));
    let app_path = PathBuf::from(option("--app-log", "/tmp/catplay.log"));
    let kernel_path = option("--kernel-log", "/dev/kmsg");
    let limit: usize = option("--limit", "65536").parse().expect("log limit");
    let duration: f64 = option("--duration", "1200").parse().expect("capture duration");
    assert!(limit >= 1024 && duration > 0.);
    fs::create_dir_all(&directory)?;
    let mut kernel = Log::new(directory.join("kernel.log"), limit)?;
    let mut app = Log::new(directory.join("application.log"), limit)?;
    let mut events = Log::new(directory.join("events.log"), limit)?;
    let boot = fs::read_to_string("/proc/sys/kernel/random/boot_id")?;
    let start = uptime();
    let header = format!("\nCAPTURE_START boot={} uptime={start:.2}\n", boot.trim());
    kernel.write(header.as_bytes())?; app.write(header.as_bytes())?; events.write(header.as_bytes())?;
    kernel.sync()?; app.sync()?; events.sync()?;
    // MCL_CURRENT | MCL_FUTURE keeps the watchdog thread runnable under I/O stalls.
    let locked = unsafe { mlockall(3) } == 0;
    let watchdog = if option("--watchdog", "0") == "1" {
        start_watchdog(option("--watchdog-timeout", "10").parse().expect("watchdog timeout"))
            .map(|()| "armed".to_string()).unwrap_or_else(|e| format!("failed: {e}"))
    } else { "off".into() };
    events.write(format!("WATCHDOG {watchdog} mlockall={locked}\n").as_bytes())?;
    events.sync()?;
    let mut input = OpenOptions::new().read(true).custom_flags(0x800).open(kernel_path).ok();
    if input.is_none() { events.write(b"Kernel capture unavailable\n")?; }
    // Status reads touch driver sysfs attributes that can block while a USB
    // probe holds a device lock. Keep them off the thread that saves logs.
    let events = std::sync::Arc::new(std::sync::Mutex::new(events));
    let status_events = events.clone();
    std::thread::spawn(move || {
        let mut last_status = String::new();
        let mut last_heartbeat = start - 30.;
        while uptime() - start < duration {
            let now = uptime();
            let current = status();
            // Sample memory pressure every second so a stall can be told apart
            // from a hard hang: look for MemAvailable falling and pgmajfault rising.
            if current != last_status || now - last_heartbeat >= 1. {
                let memory = fs::read_to_string("/proc/meminfo").unwrap_or_default();
                let field = |name: &str| memory.lines().find(|s| s.starts_with(name))
                    .and_then(|s| s.split_whitespace().nth(1)).unwrap_or("?").to_string();
                let vmstat = fs::read_to_string("/proc/vmstat").unwrap_or_default();
                let faults = vmstat.lines().find(|s| s.starts_with("pgmajfault "))
                    .and_then(|s| s.split_whitespace().nth(1)).unwrap_or("?").to_string();
                let load = fs::read_to_string("/proc/loadavg").unwrap_or_default();
                let available = format!("avail={} free={} cached={} majflt={} load={}",
                    field("MemAvailable:"), field("MemFree:"), field("Cached:"), faults,
                    load.split_whitespace().take(4).collect::<Vec<_>>().join(","));
                let mut events = status_events.lock().unwrap();
                let _ = events.write(format!("uptime={now:.2} {available} {current}").as_bytes());
                let _ = events.sync();
                last_status = current; last_heartbeat = now;
            }
            unsafe { poll(std::ptr::null_mut(), 0, 250); }
        }
    });
    let mut offset = 0;
    let mut last_sync = start;
    let mut last_heartbeat = start;
    let mut buffer = [0u8; 8192];
    while uptime() - start < duration {
        let mut wrote_kernel = false;
        if let Some(input) = input.as_mut() {
            for _ in 0..64 {
                match input.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => { kernel.write(&buffer[..n])?; wrote_kernel = true; }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(e) if e.raw_os_error() == Some(32) => continue, // kmsg overrun
                    Err(e) => {
                        events.lock().unwrap().write(format!("kernel_read_error={e}\n").as_bytes())?;
                        break;
                    }
                }
            }
        }
        if let Ok(mut file) = File::open(&app_path) {
            if file.metadata()?.len() < offset { offset = 0; app.write(b"\nAPPLICATION_LOG_RESET\n")?; }
            file.seek(SeekFrom::Start(offset))?;
            for _ in 0..64 {
                let n = file.read(&mut buffer)?;
                if n == 0 { break; }
                app.write(&buffer[..n])?;
                offset += n as u64;
            }
        }
        let now = uptime();
        // A heartbeat in the kernel log shows the logging thread outlived a stuck probe.
        if now - last_heartbeat >= 10. {
            kernel.write(format!("CAPTURE_ALIVE uptime={now:.2}\n").as_bytes())?;
            last_heartbeat = now;
        }
        // Sync kernel messages immediately so the last lines before a hang survive.
        if wrote_kernel || now - last_sync >= 1. {
            kernel.sync()?; app.sync()?; last_sync = now;
        }
        // Avoid Instant/clock_gettime64 on the vendor kernel.
        unsafe { poll(std::ptr::null_mut(), 0, 100); }
    }
    let mut events = events.lock().unwrap();
    events.write(format!("CAPTURE_STOP uptime={:.2} duration_complete\n", uptime()).as_bytes())?;
    kernel.sync()?; app.sync()?; events.sync()?;
    Ok(())
}
