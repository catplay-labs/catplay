//! Opt-in, bounded progress recording independent of the async executor.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    sync::{
        OnceLock,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

macro_rules! stages {
    ($($name:ident),+ $(,)?) => {
        #[derive(Clone, Copy)]
        #[repr(u32)]
        pub enum Stage { $($name),+ }
        fn stage_name(value: u32) -> &'static str {
            match value { $(v if v == Stage::$name as u32 => stringify!($name),)+ _ => "Unknown" }
        }
    };
}

stages! {
    Unknown, ParentRender, SwitchRequested, ParentRadioRestart, ParentTx,
    ParentAfterTx, ParentAdmission, ParentRx, ParentAfterRx, ParentPendingLock,
    ParentManagerNew, ParentReturn, SleepLock, SleepSelect, SleepReady,
    ClientRender, ClientUsb, ClientAfterUsb, ClientInvitePoll, ClientAfterInvite,
    ClientAnnounce, ClientAfterAnnounce, ClientReconnect, ClientReturn,
    ClientClose, ClientClosed, WirelessRender, WirelessCache, WirelessSwitch,
    BluetoothRead, BluetoothReadDone, ReceiverBorrow, ReceiverBorrowDone,
}

const LIMIT: usize = 32 * 1024;
const LIFETIME_MS: u32 = 180_000;
const STALL_MS: u32 = 3_000;

struct State {
    start: Instant,
    enabled: AtomicBool,
    stage: AtomicU32,
    stage_at: AtomicU32,
    stages: AtomicU32,
    loops: AtomicU32,
    ticks: AtomicU32,
    arms: AtomicU32,
}
static STATE: OnceLock<State> = OnceLock::new();

#[derive(Clone, Copy, Default)]
struct Sample {
    now: u32,
    stage: u32,
    stage_at: u32,
    stages: u32,
    loops: u32,
    ticks: u32,
    arms: u32,
}

impl State {
    fn sample(&self) -> Sample {
        Sample {
            now: self.start.elapsed().as_millis().min(u32::MAX as u128) as u32,
            stage: self.stage.load(Ordering::Acquire),
            stage_at: self.stage_at.load(Ordering::Acquire),
            stages: self.stages.load(Ordering::Acquire),
            loops: self.loops.load(Ordering::Acquire),
            ticks: self.ticks.load(Ordering::Acquire),
            arms: self.arms.load(Ordering::Acquire),
        }
    }
}

pub fn enabled() -> bool {
    STATE
        .get()
        .is_some_and(|s| s.enabled.load(Ordering::Relaxed))
}

pub fn start_from_env() -> bool {
    if STATE.get().is_some() {
        return enabled();
    }
    let Some(path) = std::env::var_os("CATPLAY_HANDOVER_TRACE").filter(|p| !p.is_empty()) else {
        return false;
    };
    let state = State {
        start: Instant::now(),
        enabled: AtomicBool::new(true),
        stage: AtomicU32::new(0),
        stage_at: AtomicU32::new(0),
        stages: AtomicU32::new(0),
        loops: AtomicU32::new(0),
        ticks: AtomicU32::new(0),
        arms: AtomicU32::new(0),
    };
    if STATE.set(state).is_err() {
        return enabled();
    }
    let state = STATE.get().unwrap();
    if thread::Builder::new()
        .name("handover-trace".into())
        .stack_size(128 * 1024)
        .spawn(move || {
            if monitor(PathBuf::from(path), state).is_err() {
                state.enabled.store(false, Ordering::Release);
            }
        })
        .is_err()
    {
        state.enabled.store(false, Ordering::Release);
    }
    enabled()
}

#[inline]
pub fn mark(stage: Stage) {
    if let Some(s) = STATE.get().filter(|s| s.enabled.load(Ordering::Relaxed)) {
        s.stage.store(stage as u32, Ordering::Release);
        s.stage_at
            .store(s.start.elapsed().as_millis().min(u32::MAX as u128) as u32, Ordering::Release);
        s.stages.fetch_add(1, Ordering::Release);
    }
}
pub fn parent_tick() {
    if let Some(s) = STATE.get().filter(|s| s.enabled.load(Ordering::Relaxed)) {
        s.loops.fetch_add(1, Ordering::Release);
    }
    mark(Stage::ParentRender);
}
pub fn runtime_tick() {
    if let Some(s) = STATE.get().filter(|s| s.enabled.load(Ordering::Relaxed)) {
        s.ticks.fetch_add(1, Ordering::Release);
    }
}
pub fn arm() {
    if let Some(s) = STATE.get().filter(|s| s.enabled.load(Ordering::Relaxed)) {
        s.arms.fetch_add(1, Ordering::Release);
    }
    mark(Stage::SwitchRequested);
}
pub fn stop() {
    if let Some(s) = STATE.get() {
        s.enabled.store(false, Ordering::Release);
    }
}

#[derive(Default)]
struct Progress {
    last_loops: u32,
    last_ticks: u32,
    loop_at: u32,
    tick_at: u32,
}
impl Progress {
    fn observe(&mut self, s: Sample) -> Option<&'static str> {
        if s.loops != self.last_loops {
            self.last_loops = s.loops;
            self.loop_at = s.now;
        }
        if s.ticks != self.last_ticks {
            self.last_ticks = s.ticks;
            self.tick_at = s.now;
        }
        if s.arms == 0 || s.now.saturating_sub(self.loop_at) < STALL_MS {
            return None;
        }
        Some(if s.now.saturating_sub(self.tick_at) >= STALL_MS {
            "runtime_stalled"
        } else {
            "parent_stalled_runtime_alive"
        })
    }
}

fn safe_proc(path: &str, default: &str) -> String {
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| {
            s.len() <= 64
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c))
        })
        .unwrap_or_else(|| default.into())
}
fn waiters() -> String {
    let Ok(tasks) = fs::read_dir("/proc/self/task") else {
        return "[]".into();
    };
    let mut entries = Vec::new();
    for task in tasks.flatten().take(16) {
        let Some(tid) = task
            .file_name()
            .to_str()
            .and_then(|n| n.parse::<u32>().ok())
        else {
            continue;
        };
        let path = format!("/proc/self/task/{tid}/wchan");
        entries.push(format!("\"{tid}:{}\"", safe_proc(&path, "unknown")));
    }
    format!("[{}]", entries.join(","))
}
fn record(file: &mut File, bytes: &mut usize, kind: &str, s: Sample, stalled: bool) -> io::Result<bool> {
    let boot = safe_proc("/proc/sys/kernel/random/boot_id", "unknown");
    let line = format!(
        "{{\"kind\":\"{kind}\",\"pid\":{},\"boot_id\":\"{boot}\",\"ms\":{},\"stage\":\"{}\",\"stage_at_ms\":{},\"stage_seq\":{},\"loop_seq\":{},\"runtime_ticks\":{},\"arm_seq\":{},\"waiters\":{}}}\n",
        std::process::id(),
        s.now,
        stage_name(s.stage),
        s.stage_at,
        s.stages,
        s.loops,
        s.ticks,
        s.arms,
        if stalled { waiters() } else { "[]".into() },
    );
    if *bytes + line.len() > LIMIT {
        return Ok(false);
    }
    file.write_all(line.as_bytes())?;
    file.sync_data()?;
    *bytes += line.len();
    Ok(true)
}

fn monitor(path: PathBuf, state: &'static State) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    let mut bytes = file.metadata()?.len() as usize;
    if !record(&mut file, &mut bytes, "boot", state.sample(), false)? {
        stop();
        return Ok(());
    }
    let mut progress = Progress::default();
    let mut ready = false;
    let mut last_write = 0;
    let mut last_arm = 0;
    let mut last_stall = None;
    while enabled() {
        thread::sleep(Duration::from_millis(500));
        let s = state.sample();
        if s.now >= LIFETIME_MS {
            break;
        }
        let stalled = progress.observe(s);
        let kind = if !ready && s.now >= 1_000 {
            ready = true;
            Some("ready")
        } else if s.arms != last_arm {
            last_arm = s.arms;
            Some("armed")
        } else if stalled != last_stall {
            Some(stalled.unwrap_or("resumed"))
        } else if s.arms > 0 && s.now.saturating_sub(last_write) >= 2_000 {
            Some(stalled.unwrap_or("progress"))
        } else {
            None
        };
        last_stall = stalled;
        if let Some(kind) = kind {
            if !record(&mut file, &mut bytes, kind, s, stalled.is_some())? {
                break;
            }
            last_write = s.now;
        }
    }
    let _ = record(&mut file, &mut bytes, "finished", state.sample(), false);
    stop();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unarmed_idle_is_not_a_stall() {
        assert_eq!(
            Progress::default().observe(Sample {
                now: 20_000,
                ..Sample::default()
            }),
            None
        );
    }
    #[test]
    fn stalled_parent_with_live_runtime_is_distinguished() {
        let mut p = Progress::default();
        p.observe(Sample {
            loops: 1,
            ticks: 1,
            arms: 1,
            ..Sample::default()
        });
        assert_eq!(
            p.observe(Sample {
                now: 4_000,
                loops: 1,
                ticks: 5,
                arms: 1,
                ..Sample::default()
            }),
            Some("parent_stalled_runtime_alive")
        );
        assert_eq!(
            p.observe(Sample {
                now: 4_100,
                loops: 2,
                ticks: 5,
                arms: 1,
                ..Sample::default()
            }),
            None
        );
    }
    #[test]
    fn frozen_runtime_is_detected() {
        let mut p = Progress::default();
        p.observe(Sample {
            loops: 1,
            ticks: 1,
            arms: 1,
            ..Sample::default()
        });
        assert_eq!(
            p.observe(Sample {
                now: 4_000,
                loops: 1,
                ticks: 1,
                arms: 1,
                ..Sample::default()
            }),
            Some("runtime_stalled")
        );
    }
}
