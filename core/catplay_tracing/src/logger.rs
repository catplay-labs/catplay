use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::io::{self, Write};
use std::{
    fmt::Write as FmtWrite,
    fs::{File, OpenOptions},
    path::Path,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use crate::lowprio_queue::LowPriorityWorker;

pub struct AsyncLogger {
    shared: Arc<LoggerShared>,
}

pub trait LogSink: Send + Sync {
    fn write(&self, value: &str);
    fn flush(&self);
}

pub struct StderrLogSink;

impl LogSink for StderrLogSink {
    fn write(&self, value: &str) {
        let mut stderr = io::stderr().lock();
        let _ = writeln!(stderr, "{}", value);
    }

    fn flush(&self) {
        let _ = io::stderr().lock().flush();
    }
}

pub struct FileAppendLogSink {
    file: Mutex<File>,
}

impl FileAppendLogSink {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { file: Mutex::new(file) })
    }
}

impl LogSink for FileAppendLogSink {
    fn write(&self, value: &str) {
        let Ok(mut file) = self.file.lock() else {
            return;
        };
        let _ = writeln!(file, "{}", value);
    }

    fn flush(&self) {
        let Ok(mut file) = self.file.lock() else {
            return;
        };
        let _ = file.flush();
    }
}

pub struct NoopLogSink;

impl LogSink for NoopLogSink {
    fn write(&self, _value: &str) {}

    fn flush(&self) {}
}

struct TimestampedRecord<'a> {
    record: &'a Record<'a>,
    created_at: Instant,
}

struct OwnedLogRecord {
    created_at: Instant,
    level: Level,
    target: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
    message: String,
}

struct LoggerShared {
    level_filter: LevelFilter,
    start: Instant,
    timestamp_style: TimestampStyle,
    sink: Arc<dyn LogSink>,
    is_async: AtomicBool,
    log_worker: LowPriorityWorker<()>,
    format_buffer_pool: LogBufferPool,
}

#[derive(Clone, Copy)]
pub enum TimestampStyle {
    ElapsedSecondsMillis,
}

const FORMAT_BUFFER_TARGET_CAPACITY: usize = 4 * 1024;
const FORMAT_BUFFER_POOL_MAX_LEN: usize = 64;

static FORMAT_QUEUE: OnceLock<LowPriorityWorker<()>> = OnceLock::new();
static ACTIVE_LOGGER: OnceLock<Arc<LoggerShared>> = OnceLock::new();

fn format_queue() -> &'static LowPriorityWorker<()> {
    FORMAT_QUEUE.get_or_init(|| LowPriorityWorker::new("catplay-log-fmt", ()))
}

/// Performs log flush when dropped; useful in main().
pub struct AsyncLoggerBarrier(());

impl Drop for AsyncLoggerBarrier {
    fn drop(&mut self) {
        AsyncLogger::flush();
    }
}

impl AsyncLogger {
    pub fn new(level_filter: LevelFilter) -> Self {
        Self::new_with_sink(level_filter, Arc::new(StderrLogSink))
    }

    pub fn new_with_file_sink(level_filter: LevelFilter, path: impl AsRef<Path>) -> io::Result<Self> {
        let sink = Arc::new(FileAppendLogSink::new(path)?);
        Ok(Self::new_with_sink(level_filter, sink))
    }

    pub fn new_with_noop_sink() -> Self {
        Self::new_with_sink(log::LevelFilter::Off, Arc::new(NoopLogSink))
    }

    pub fn new_with_sink(level_filter: LevelFilter, sink: Arc<dyn LogSink>) -> Self {
        Self {
            shared: Arc::new(LoggerShared {
                level_filter,
                start: Instant::now(),
                timestamp_style: TimestampStyle::ElapsedSecondsMillis,
                sink,
                is_async: AtomicBool::new(false),
                log_worker: LowPriorityWorker::new("catplay/logger", ()),
                format_buffer_pool: LogBufferPool::new(),
            }),
        }
    }

    pub fn from_env_secondary() -> Self {
        Self::from_env_secondary_with_sink(Arc::new(StderrLogSink))
    }

    pub fn from_env_secondary_with_sink(sink: Arc<dyn LogSink>) -> Self {
        let level_filter = Self::level_filter_from_env();
        Self::new_with_sink(level_filter, sink)
    }

    pub fn from_env_secondary_with_file_sink(path: impl AsRef<Path>) -> io::Result<Self> {
        let sink = Arc::new(FileAppendLogSink::new(path)?);
        Ok(Self::from_env_secondary_with_sink(sink))
    }

    pub fn set_async(is_async: bool) {
        if let Some(logger) = ACTIVE_LOGGER.get() {
            logger.is_async.store(is_async, Ordering::Release);
        }
    }

    pub fn set_async_for_self(&self, is_async: bool) {
        self.shared.is_async.store(is_async, Ordering::Release);
    }

    pub fn init_from_env() -> Result<AsyncLoggerBarrier, SetLoggerError> {
        Self::init_from_env_with_sink(Arc::new(StderrLogSink))
    }

    pub fn init_from_env_with_sink(sink: Arc<dyn LogSink>) -> Result<AsyncLoggerBarrier, SetLoggerError> {
        let level_filter = Self::level_filter_from_env();
        let logger = Self::new_with_sink(level_filter, sink);
        let shared = logger.shared.clone();
        log::set_max_level(level_filter);
        log::set_boxed_logger(Box::new(logger))?;
        let _ = ACTIVE_LOGGER.set(shared);
        Ok(Self::create_barrier())
    }

    pub fn create_barrier() -> AsyncLoggerBarrier {
        AsyncLoggerBarrier(())
    }

    pub fn flush() {
        if let Some(logger) = ACTIVE_LOGGER.get() {
            logger.flush();
        }
    }

    pub fn flush_self(&self) {
        self.shared.flush();
    }

    fn level_filter_from_env() -> LevelFilter {
        std::env::var("RUST_LOG")
            .ok()
            .and_then(|value| Self::parse_rust_log(&value))
            .unwrap_or(LevelFilter::Debug)
    }

    fn parse_rust_log(value: &str) -> Option<LevelFilter> {
        value
            .split(',')
            .find_map(|directive| Self::parse_directive_level(directive.trim()))
    }

    fn parse_directive_level(directive: &str) -> Option<LevelFilter> {
        if directive.is_empty() {
            return None;
        }

        let level_str = directive
            .split_once('=')
            .map(|(_, level)| level.trim())
            .unwrap_or(directive)
            .to_ascii_lowercase();

        match level_str.as_str() {
            "off" => Some(LevelFilter::Off),
            "error" => Some(LevelFilter::Error),
            "warn" | "warning" => Some(LevelFilter::Warn),
            "info" => Some(LevelFilter::Info),
            "debug" => Some(LevelFilter::Debug),
            "trace" => Some(LevelFilter::Trace),
            _ => None,
        }
    }

    fn ansi_reset() -> &'static str {
        "\x1b[0m"
    }

    fn level_style(level: Level) -> &'static str {
        match level {
            Level::Trace => "\x1b[35m",
            Level::Debug => "\x1b[34m",
            Level::Info => "\x1b[32m",
            Level::Warn => "\x1b[33m",
            Level::Error => "\x1b[1;31m",
        }
    }
}

pub fn enqueue_async_log<F>(
    created_at: Instant,
    level: Level,
    target: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
    format_task: F,
) where
    F: FnOnce() -> String + Send + 'static,
{
    let metadata = Metadata::builder().level(level).target(target).build();
    if !log::logger().enabled(&metadata) {
        return;
    }

    let Some(logger) = ACTIVE_LOGGER.get().cloned() else {
        let message = format_task();
        log_owned_message_via_log_crate(OwnedLogRecord {
            created_at,
            level,
            target,
            module_path,
            file,
            line,
            message,
        });
        return;
    };

    if !logger.is_async() {
        let message = format_task();
        logger.log_owned_message(OwnedLogRecord {
            created_at,
            level,
            target,
            module_path,
            file,
            line,
            message,
        });
        return;
    }

    format_queue().enqueue_cb(move |_| {
        let message = format_task();
        logger.log_owned_message(OwnedLogRecord {
            created_at,
            level,
            target,
            module_path,
            file,
            line,
            message,
        });
    });
}

#[allow(clippy::too_many_arguments)]
pub fn enqueue_async_log_for_logger(
    logger: &AsyncLogger,
    created_at: Instant,
    level: Level,
    target: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
    format_task: impl FnOnce() -> String + Send + 'static,
) {
    let metadata = Metadata::builder().level(level).target(target).build();
    if !logger.shared.enabled(&metadata) {
        return;
    }

    let shared = logger.shared.clone();
    if !shared.is_async() {
        let message = format_task();
        shared.log_owned_message(OwnedLogRecord {
            created_at,
            level,
            target,
            module_path,
            file,
            line,
            message,
        });
        return;
    }

    format_queue().enqueue_cb(move |_| {
        let message = format_task();
        shared.log_owned_message(OwnedLogRecord {
            created_at,
            level,
            target,
            module_path,
            file,
            line,
            message,
        });
    });
}

fn log_owned_message_via_log_crate(record: OwnedLogRecord) {
    let args = format_args!("{}", record.message);
    let log_record = Record::builder()
        .args(args)
        .level(record.level)
        .target(record.target)
        .module_path_static(Some(record.module_path))
        .file_static(Some(record.file))
        .line(Some(record.line))
        .build();

    log::logger().log(&log_record);
}

impl Log for AsyncLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.shared.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        self.shared.log_timestamped(&TimestampedRecord {
            record,
            created_at: Instant::now(),
        });
    }

    fn flush(&self) {
        format_queue().barrier();
        self.shared.flush();
    }
}

#[doc(hidden)]
pub fn macro_now_instant() -> Instant {
    Instant::now()
}

#[doc(hidden)]
pub fn macro_format_owned(args: std::fmt::Arguments<'_>) -> String {
    args.to_string()
}

impl LoggerShared {
    fn is_async(&self) -> bool {
        self.is_async.load(Ordering::Acquire)
    }

    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    fn log_owned_message(&self, record: OwnedLogRecord) {
        let args = format_args!("{}", record.message);
        let log_record = Record::builder()
            .args(args)
            .level(record.level)
            .target(record.target)
            .module_path_static(Some(record.module_path))
            .file_static(Some(record.file))
            .line(Some(record.line))
            .build();

        self.log_timestamped(&TimestampedRecord {
            record: &log_record,
            created_at: record.created_at,
        });
    }

    fn log_timestamped(&self, record: &TimestampedRecord<'_>) {
        if !self.enabled(record.record.metadata()) {
            return;
        }

        let mut formatted = self.format_buffer_pool.take();
        self.format_record_into(record, &mut formatted);

        if self.is_async() {
            let sink = self.sink.clone();
            let buffer_pool = self.format_buffer_pool.clone();
            self.log_worker.enqueue_cb(move |_| {
                sink.write(&formatted);
                buffer_pool.put(formatted);
            });
        } else {
            self.sink.write(&formatted);
            self.format_buffer_pool.put(formatted);
        }
    }

    fn flush(&self) {
        if let Some(q) = FORMAT_QUEUE.get() {
            q.barrier();
        }

        self.log_worker.barrier();
        self.sink.flush();
    }

    fn format_record_into(&self, record: &TimestampedRecord<'_>, out: &mut String) {
        out.clear();
        let delta = record.created_at.saturating_duration_since(self.start);
        let style = AsyncLogger::level_style(record.record.level());
        out.push_str(AsyncLogger::ansi_reset());
        out.push_str(style);
        out.push('[');
        self.format_timestamp_into(delta, out);
        let _ = write!(out, " {:<5} {}] ", record.record.level(), record.record.target());
        let _ = write!(out, "{}", record.record.args());
        out.push_str(AsyncLogger::ansi_reset());
    }

    fn format_timestamp_into(&self, delta: std::time::Duration, out: &mut String) {
        match self.timestamp_style {
            TimestampStyle::ElapsedSecondsMillis => {
                let _ = write!(out, "{:>7}.{:03}s", delta.as_secs(), delta.subsec_millis());
            }
        }
    }
}

#[derive(Clone, Default)]
struct LogBufferPool {
    inner: Arc<Mutex<Vec<String>>>,
}

impl LogBufferPool {
    fn new() -> Self {
        Self::default()
    }

    fn take(&self) -> String {
        if let Ok(mut guard) = self.inner.lock()
            && let Some(mut buffer) = guard.pop()
        {
            buffer.clear();
            return buffer;
        }

        String::with_capacity(FORMAT_BUFFER_TARGET_CAPACITY)
    }

    fn put(&self, mut buffer: String) {
        if buffer.capacity() > FORMAT_BUFFER_TARGET_CAPACITY {
            return;
        }

        buffer.clear();
        let Ok(mut guard) = self.inner.lock() else {
            return;
        };

        if guard.len() < FORMAT_BUFFER_POOL_MAX_LEN {
            guard.push(buffer);
        }
    }
}

pub fn setup_prod_logger() {
    let _ = AsyncLogger::init_from_env();
}

pub fn setup_test_logger(trace: bool) {
    if std::env::var_os("RUST_LOG").is_none() {
        unsafe {
            std::env::set_var("RUST_LOG", if trace { "trace" } else { "debug" });
        }
    }

    let _ = AsyncLogger::init_from_env();
    log::warn!("Logger is now adjusted for tests");
}

#[test]
fn test() {
    AsyncLogger::init_from_env().unwrap();
    AsyncLogger::set_async(true);

    log::info!("test");
    log::debug!("test");
    log::error!("test");

    let a = vec![1; 1];
    crate::aerror!("atest1 {:?}", a);
    crate::aerror!("atest2 {}", 2);
    let _ = AsyncLogger::create_barrier();
}

#[test]
fn file_append_string_sink_writes_and_flushes_to_random_tmp_path() {
    let suffix = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let path = std::path::PathBuf::from(format!("/tmp/test.log.{}", suffix));

    let sink = FileAppendLogSink::new(&path).unwrap();
    sink.write("line-one");
    sink.write("line-two");
    sink.flush();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("line-one"));
    assert!(content.contains("line-two"));

    let _ = std::fs::remove_file(path);
}

#[test]
fn secondary_logger_writes_via_t_macros_to_file_sink() {
    let suffix = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let path = std::path::PathBuf::from(format!("/tmp/secondary.log.{}", suffix));

    let logger = AsyncLogger::new_with_file_sink(LevelFilter::Trace, &path).unwrap();
    logger.set_async_for_self(true);

    crate::tinfo!(&logger, "secondary-info {}", 7);
    crate::terror!(&logger, target: "secondary::target", "secondary-error {}", 9);
    logger.flush_self();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("secondary-info 7"));
    assert!(content.contains("secondary-error 9"));
    assert!(content.contains("secondary::target"));

    let _ = std::fs::remove_file(path);
}
