use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Once, OnceLock};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tui_logger::TuiTracingSubscriberLayer;

use crate::config;

mod events;
mod settings;

pub use self::events::*;
pub use self::settings::*;

static INIT: Once = Once::new();
static GUARD: OnceLock<WorkerGuard> = OnceLock::new();
static STDOUT_BUFFER: OnceLock<Arc<Mutex<Vec<u8>>>> = OnceLock::new();
static STDOUT_LOGGING_ENABLED: OnceLock<bool> = OnceLock::new();
static LOGGER_SETTINGS: OnceLock<LoggerSettings> = OnceLock::new();

#[derive(Clone)]
struct BufferedMakeWriter {
    inner: Arc<Mutex<Vec<u8>>>,
}

struct BufferedWriter {
    inner: Arc<Mutex<Vec<u8>>>,
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for BufferedMakeWriter {
    type Writer = BufferedWriter;

    fn make_writer(&'a self) -> Self::Writer {
        BufferedWriter {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Write for BufferedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = self.inner.lock().expect("stdout log buffer poisoned");
        guard.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn log_dir() -> PathBuf {
    config::spoon_home_dir().join("logs")
}

fn log_path() -> PathBuf {
    log_dir().join("spoon.log")
}

pub fn init(settings: LoggerSettings) {
    INIT.call_once(|| {
        let _ = STDOUT_LOGGING_ENABLED.set(settings.stdout_enabled);
        let _ = LOGGER_SETTINGS.set(settings);
        let dir = log_dir();
        if fs::create_dir_all(&dir).is_err() {
            return;
        }

        let file_appender = tracing_appender::rolling::never(&dir, "spoon.log");
        let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
        let _ = GUARD.set(guard);
        let _ = tui_logger::init_logger(settings.tui_level);
        tui_logger::set_default_level(settings.tui_level);

        let stdout_buffer = STDOUT_BUFFER
            .get_or_init(|| Arc::new(Mutex::new(Vec::new())))
            .clone();

        let stdout_layer = fmt::layer()
            .with_ansi(true)
            .with_target(false)
            .with_writer(BufferedMakeWriter {
                inner: stdout_buffer,
            })
            .with_filter(settings.stdout_level);
        let file_layer = fmt::layer()
            .with_ansi(false)
            .with_target(false)
            .with_writer(file_writer)
            .with_filter(settings.file_level);

        let _ = tracing_subscriber::registry()
            .with(TuiTracingSubscriberLayer)
            .with(stdout_layer)
            .with(file_layer)
            .try_init();

        session_start(&log_path());
    });
}

pub fn flush_buffered_stdout() {
    if !STDOUT_LOGGING_ENABLED.get().copied().unwrap_or(false) {
        return;
    }
    let Some(buffer) = STDOUT_BUFFER.get() else {
        return;
    };
    let mut guard = buffer.lock().expect("stdout log buffer poisoned");
    if guard.is_empty() {
        return;
    }
    let text = String::from_utf8_lossy(&guard).to_string();
    guard.clear();
    drop(guard);
    print!("{text}");
}
