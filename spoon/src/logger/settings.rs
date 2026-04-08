use tracing_subscriber::filter::LevelFilter as TraceLevelFilter;
use tui_logger::LevelFilter as TuiLevelFilter;

#[derive(Debug, Clone, Copy)]
pub struct LoggerSettings {
    pub stdout_enabled: bool,
    pub stdout_level: TraceLevelFilter,
    pub file_level: TraceLevelFilter,
    pub tui_level: TuiLevelFilter,
}

impl LoggerSettings {
    pub const fn standard(verbose: bool) -> Self {
        Self {
            stdout_enabled: verbose,
            stdout_level: if verbose {
                TraceLevelFilter::INFO
            } else {
                TraceLevelFilter::OFF
            },
            file_level: TraceLevelFilter::TRACE,
            tui_level: TuiLevelFilter::Trace,
        }
    }
}
