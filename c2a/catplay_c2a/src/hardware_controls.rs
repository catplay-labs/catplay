use crate::{
    HardwareConfig,
    hardware_logic::{Button, IndicatorState, led_on, rgb_spi_frame},
};

/// Hardware I/O must never block the single-threaded audio/RTSP event loop.
pub(crate) struct HardwareWorker {
    state: std::sync::Arc<std::sync::atomic::AtomicU8>,
    action: std::sync::Arc<std::sync::atomic::AtomicBool>,
    stop: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}

impl HardwareWorker {
    pub(crate) fn new(cfg: HardwareConfig) -> Self {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, AtomicU8, Ordering},
        };
        let mut worker = Self {
            state: Arc::new(AtomicU8::new(0)),
            action: Arc::new(AtomicBool::new(false)),
            stop: None,
        };
        if !cfg.enabled {
            return worker;
        }
        let stop = Arc::new(AtomicBool::new(false));
        // Keep GPIO sampling independent even when the SPI driver times out.
        let configurations = [
            (
                "catplay-button",
                HardwareConfig {
                    led_path: None,
                    led_spi_device: None,
                    ..cfg.clone()
                },
            ),
            (
                "catplay-led",
                HardwareConfig {
                    button_gpiochip: None,
                    button_line: None,
                    ..cfg
                },
            ),
        ];
        for (name, cfg) in configurations {
            if cfg.button_gpiochip.is_none() && cfg.led_path.is_none() && cfg.led_spi_device.is_none() {
                continue;
            }
            let thread_stop = stop.clone();
            let state = worker.state.clone();
            let action = worker.action.clone();
            match std::thread::Builder::new()
                .name(name.into())
                .stack_size(128 * 1024)
                .spawn(move || {
                    let mut controls = HardwareControls::new(cfg);
                    while !thread_stop.load(Ordering::Relaxed) {
                        let indicator = match state.load(Ordering::Relaxed) {
                            1 => IndicatorState::Connected,
                            2 => IndicatorState::Switching,
                            3 => IndicatorState::Error,
                            _ => IndicatorState::Waiting,
                        };
                        if controls.update(indicator) {
                            action.store(true, Ordering::Release);
                        }
                        std::thread::sleep(Duration::from_millis(25));
                    }
                }) {
                Ok(_) => worker.stop = Some(stop.clone()),
                Err(err) => warn!("Optional hardware worker unavailable: {err}"),
            }
        }
        worker
    }
    pub(crate) fn enabled(&self) -> bool {
        self.stop.is_some()
    }
    pub(crate) fn update(&self, state: IndicatorState) -> bool {
        use std::sync::atomic::Ordering;
        self.state.store(
            match state {
                IndicatorState::Waiting => 0,
                IndicatorState::Connected => 1,
                IndicatorState::Switching => 2,
                IndicatorState::Error => 3,
            },
            Ordering::Relaxed,
        );
        self.action.swap(false, Ordering::AcqRel)
    }
}

impl Drop for HardwareWorker {
    fn drop(&mut self) {
        if let Some(stop) = &self.stop {
            stop.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        // The worker releases its GPIO/LED on exit; never join a blocked SPI call
        // from the RTSP event loop. No new worker is spawned by an I/O retry.
    }
}
use log::warn;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

/// Optional peripherals: errors disable/retry the peripheral, never CarPlay.
pub(crate) struct HardwareControls {
    cfg: HardwareConfig,
    button: Option<GpioButton>,
    key: Button,
    led: Option<Led>,
    last_level: Option<bool>,
    retry_button: Instant,
    retry_led: Instant,
    epoch: Instant,
}

impl HardwareControls {
    pub(crate) fn new(cfg: HardwareConfig) -> Self {
        let now = Instant::now();
        Self {
            cfg,
            button: None,
            key: Button::new(now),
            led: None,
            last_level: None,
            retry_button: now,
            retry_led: now,
            epoch: now,
        }
    }

    #[allow(dead_code)] // Also compiled into the standalone diagnostic example.
    pub(crate) fn button_ready(&self) -> bool {
        self.button.is_some()
    }

    #[allow(dead_code)] // Diagnostic example.
    pub(crate) fn led_ready(&self) -> bool {
        self.led.is_some()
    }

    pub(crate) fn update(&mut self, state: IndicatorState) -> bool {
        if !self.cfg.enabled {
            return false;
        }
        let now = Instant::now();
        if self.button.is_none() && now >= self.retry_button {
            self.retry_button = now + Duration::from_secs(5);
            if let (Some(chip), Some(line)) = (&self.cfg.button_gpiochip, self.cfg.button_line) {
                match GpioButton::open(
                    chip,
                    line,
                    self.cfg.button_active_low,
                    self.cfg.button_bias_pull_up,
                    self.cfg.button_chip_label.as_deref(),
                ) {
                    Ok(button) => {
                        self.button = Some(button);
                        self.key = Button::new(now);
                        log::info!("Device-switch button ready");
                    }
                    Err(err) => warn!("Cannot open device-switch button: {err}"),
                }
            }
        }
        let mut action = false;
        if let Some(button) = &self.button {
            match button.pressed() {
                Ok(pressed) => action = self.key.sample(pressed, now),
                Err(err) => {
                    warn!("Device-switch button unavailable: {err}");
                    self.button = None;
                }
            }
        }
        if self.led.is_none() && now >= self.retry_led {
            self.retry_led = now + Duration::from_secs(5);
            match Led::open(&self.cfg) {
                Ok(led) => {
                    if led.is_some() {
                        log::info!("Status LED ready");
                    }
                    self.led = led;
                    self.last_level = None;
                }
                Err(err) => warn!("Cannot open status LED: {err}"),
            }
        }
        let on = led_on(state, now.duration_since(self.epoch));
        if let Some(led) = &mut self.led
            && self.last_level != Some(on)
        {
            match led.set(on) {
                Ok(()) => self.last_level = Some(on),
                Err(err) => {
                    warn!("Status LED unavailable: {err}");
                    self.led = None;
                }
            }
        }
        action
    }
}

impl Drop for HardwareControls {
    fn drop(&mut self) {
        if let Some(led) = &mut self.led {
            let _ = led.set(false);
        }
    }
}

enum Led {
    Sysfs(PathBuf, u32),
    Spi(File),
}

impl Led {
    fn open(cfg: &HardwareConfig) -> io::Result<Option<Self>> {
        match (&cfg.led_path, &cfg.led_spi_device) {
            (Some(_), Some(_)) => Err(io::Error::other("configure only one LED backend")),
            (Some(path), None) => {
                let path = PathBuf::from(path);
                let max: u32 = fs::read_to_string(path.join("max_brightness"))?
                    .trim()
                    .parse()
                    .map_err(io::Error::other)?;
                // Do not fight the kernel's timer/heartbeat trigger.
                if path.join("trigger").exists() {
                    fs::write(path.join("trigger"), "none")?;
                }
                OpenOptions::new()
                    .write(true)
                    .open(path.join("brightness"))?;
                Ok(Some(Self::Sysfs(path, max)))
            }
            (None, Some(path)) => Ok(Some(Self::Spi(open_spi(path)?))),
            (None, None) => Ok(None),
        }
    }
    fn set(&mut self, on: bool) -> io::Result<()> {
        match self {
            Self::Sysfs(path, max) => fs::write(path.join("brightness"), if on { *max } else { 0 }.to_string()),
            Self::Spi(file) => file.write_all(&rgb_spi_frame(on)),
        }
    }
}

#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd, FromRawFd};

#[cfg(target_os = "linux")]
fn open_spi(path: &str) -> io::Result<File> {
    let file = OpenOptions::new().write(true).open(path)?;
    let mode: u8 = 0; // SPI mode 0; no physical CS is connected to the LED.
    let bits: u8 = 8;
    let speed: u32 = 2_400_000;
    for (request, ptr) in [
        (0x40016b01, &mode as *const u8 as *const libc::c_void),
        (0x40016b03, &bits as *const u8 as *const libc::c_void),
        (0x40046b04, &speed as *const u32 as *const libc::c_void),
    ] {
        if unsafe { libc::ioctl(file.as_raw_fd(), request as _, ptr) } < 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(file)
}

#[cfg(not(target_os = "linux"))]
fn open_spi(_: &str) -> io::Result<File> {
    Err(io::Error::other("SPI LED requires Linux"))
}

struct GpioButton(File);

// Linux GPIO character-device ABI v2; do not use legacy global GPIO numbers.
#[cfg(target_os = "linux")]
impl GpioButton {
    fn open(path: &str, line: u32, active_low: bool, pull_up: bool, expected_label: Option<&str>) -> io::Result<Self> {
        #[repr(C)]
        struct Request {
            offsets: [u32; 64],
            consumer: [u8; 32],
            flags: u64,
            num_attrs: u32,
            config_padding: [u32; 5],
            attrs: [u64; 30],
            num_lines: u32,
            event_buffer_size: u32,
            padding: [u32; 5],
            fd: i32,
        }
        const {
            assert!(std::mem::size_of::<Request>() == 592);
        }
        let chip = File::open(path)?;
        #[repr(C)]
        struct ChipInfo {
            name: [u8; 32],
            label: [u8; 32],
            lines: u32,
        }
        let mut info: ChipInfo = unsafe { std::mem::zeroed() };
        if unsafe { libc::ioctl(chip.as_raw_fd(), 0x8044b401u32 as _, &mut info) } < 0 {
            return Err(io::Error::last_os_error());
        }
        let label = std::str::from_utf8(info.label.split(|b| *b == 0).next().unwrap_or_default()).unwrap_or("");
        if line >= info.lines || expected_label.is_some_and(|expected| expected != label) {
            return Err(io::Error::other(format!(
                "GPIO chip label={label}, lines={}; check board mapping for line {line}",
                info.lines
            )));
        }
        let mut request: Request = unsafe { std::mem::zeroed() };
        request.offsets[0] = line;
        request.consumer[..14].copy_from_slice(b"catplay-switch");
        request.flags = (1 << 2) | if active_low { 1 << 1 } else { 0 } | if pull_up { 1 << 8 } else { 0 };
        request.num_lines = 1;
        if unsafe { libc::ioctl(chip.as_raw_fd(), 0xc250b407u32 as _, &mut request) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(unsafe { File::from_raw_fd(request.fd) }))
    }
    fn pressed(&self) -> io::Result<bool> {
        #[repr(C)]
        struct Values {
            bits: u64,
            mask: u64,
        }
        let mut values = Values { bits: 0, mask: 1 };
        if unsafe { libc::ioctl(self.0.as_raw_fd(), 0xc010b40eu32 as _, &mut values) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(values.bits & 1 != 0)
    }
}

#[cfg(not(target_os = "linux"))]
impl GpioButton {
    fn open(_: &str, _: u32, _: bool, _: bool, _: Option<&str>) -> io::Result<Self> {
        Err(io::Error::other("GPIO requires Linux"))
    }
    fn pressed(&self) -> io::Result<bool> {
        Err(io::Error::other("GPIO requires Linux"))
    }
}
