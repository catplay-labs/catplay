#[cfg(feature = "std")]
use crate::logger::AsyncLogger;
#[cfg(feature = "std")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "std")]
static SESSION_TRACER_ENABLED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "std")]
pub struct SessionTracer {
    logger: AsyncLogger,
    log_path: std::path::PathBuf,
}

#[cfg(feature = "std")]
impl SessionTracer {
    pub fn enable_globally() {
        Self::set_globally_enabled(true);
    }

    pub fn disable_globally() {
        Self::set_globally_enabled(false);
    }

    pub fn set_globally_enabled(is_enabled: bool) {
        SESSION_TRACER_ENABLED.store(is_enabled, Ordering::Release);
    }

    pub fn new(name: impl AsRef<str>) -> Self {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let log_path = std::path::PathBuf::from(format!("/tmp/{}.{}.log", name.as_ref(), suffix));
        let logger = if SESSION_TRACER_ENABLED.load(Ordering::Acquire) {
            match AsyncLogger::new_with_file_sink(log::LevelFilter::Trace, &log_path) {
                Ok(logger) => logger,
                Err(err) => {
                    use crate::logger::AsyncLogger;

                    log::log!(
                        log::Level::Error,
                        "failed to create session tracer file at '{}': {}",
                        log_path.display(),
                        err
                    );
                    AsyncLogger::new_with_noop_sink()
                }
            }
        } else {
            AsyncLogger::new_with_noop_sink()
        };
        logger.set_async_for_self(true);
        Self { logger, log_path }
    }

    pub fn logger(&self) -> &AsyncLogger {
        &self.logger
    }

    pub fn log_path(&self) -> &std::path::Path {
        &self.log_path
    }
}

#[cfg(feature = "std")]
impl Drop for SessionTracer {
    fn drop(&mut self) {
        self.logger.flush_self();
    }
}

#[cfg(not(feature = "std"))]
pub struct SessionTracer;

#[cfg(not(feature = "std"))]
impl SessionTracer {
    pub fn enable_globally() {}

    pub fn disable_globally() {}

    pub fn set_globally_enabled(_is_enabled: bool) {}

    pub fn new(_path: impl AsRef<str>) -> Self {
        Self
    }
}

#[cfg(feature = "std")]
#[test]
fn session_tracer_flushes_on_drop_and_st_macros_write() {
    let path: std::path::PathBuf;

    SessionTracer::enable_globally();

    {
        let tracer = SessionTracer::new("session-tracer");
        path = tracer.log_path().to_path_buf();
        crate::stinfo!(tracer, "session-info {}", 11);
        crate::sterror!(tracer, target: "session::target", "session-error {}", 22);
    }

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("session-info 11"));
    assert!(content.contains("session-error 22"));
    assert!(content.contains("session::target"));

    let _ = std::fs::remove_file(path);
    SessionTracer::disable_globally();
}
