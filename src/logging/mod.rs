// There log crates but feeling like making my own in the meantime

use std::{borrow::Cow, fmt::Arguments, sync::LazyLock, thread, time::Instant};

use chrono::Local;

mod loglevel;
pub use loglevel::LogLevel;

#[macro_export]
macro_rules! fatal {
  ($logger:ident, $($arg:tt)*) => {
    $logger.print_log($crate::logging::LogLevel::Fatal, format_args!($($arg)*))
  };

  ($($arg:tt)*) => {
    $crate::logging::LOGGER_DEFAULT.print_log($crate::logging::LogLevel::Fatal, format_args!($($arg)*))
  };
}

#[macro_export]
macro_rules! alert {
  ($logger:ident, $($arg:tt)*) => {
    $logger.print_log($crate::logging::LogLevel::Alert, format_args!($($arg)*))
  };

  ($($arg:tt)*) => {
    $crate::logging::LOGGER_DEFAULT.print_log($crate::logging::LogLevel::Alert, format_args!($($arg)*))
  };
}

#[macro_export]
macro_rules! error {
  ($logger:ident, $($arg:tt)*) => {
    $logger.print_log($crate::logging::LogLevel::Error, format_args!($($arg)*))
  };

  ($($arg:tt)*) => {
    $crate::logging::LOGGER_DEFAULT.print_log($crate::logging::LogLevel::Error, format_args!($($arg)*))
  };
}

#[macro_export]
macro_rules! warn {
  ($logger:ident, $($arg:tt)*) => {
    $logger.print_log($crate::logging::LogLevel::Warning, format_args!($($arg)*))
  };
  ($($arg:tt)*) => {
    $crate::logging::LOGGER_DEFAULT.print_log($crate::logging::LogLevel::Warning, format_args!($($arg)*))
  };
}

#[macro_export]
macro_rules! info {
  ($logger:ident, $($arg:tt)*) => {
    $logger.print_log($crate::logging::LogLevel::Info, format_args!($($arg)*))
  };

  ($($arg:tt)*) => {
    $crate::logging::LOGGER_DEFAULT.print_log($crate::logging::LogLevel::Info, format_args!($($arg)*))
  };
}

#[macro_export]
macro_rules! debug {
  ($logger:ident, $($arg:tt)*) => {
    $logger.print_log($crate::logging::LogLevel::Debug, format_args!($($arg)*))
  };

  ($($arg:tt)*) => {
    $crate::logging::LOGGER_DEFAULT.print_log($crate::logging::LogLevel::Debug, format_args!($($arg)*))
  };
}

#[macro_export]
macro_rules! trace {
  ($logger:ident, $($arg:tt)*) => {
    $logger.print_log($crate::logging::LogLevel::Trace, format_args!($($arg)*))
  };

  ($($arg:tt)*) => {
    $crate::logging::LOGGER_DEFAULT.print_log($crate::logging::LogLevel::Trace, format_args!($($arg)*))
  };
}

pub struct Logger {
    name: Cow<'static, str>,
}

impl Logger {
    #[allow(unused)]
    pub fn new(name: String) -> Self {
        Self {
            name: Cow::Owned(name),
        }
    }

    pub const fn new_str(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
        }
    }

    pub fn print_log(&self, level: LogLevel, args: Arguments<'_>) {
        let secs = (Instant::now() - *STARTUP_TIME).as_secs_f32();
        println!(
            "[{secs:#14.6}] [{}/{level}] [{}] {args}",
            thread::current().name().unwrap_or("??"),
            self.name
        );
    }
}

pub static STARTUP_TIME: LazyLock<Instant> = LazyLock::new(|| Instant::now());
pub static LOGGER: Logger = Logger::new_str("Logging");
#[allow(unused)]
pub static LOGGER_DEFAULT: Logger = Logger::new_str("Main");

static LOGGER_FROM_LOG_CRATE: Logger = Logger::new_str("FromLogCrate");

pub fn init() {
    LazyLock::force(&STARTUP_TIME);
    let time_started = Local::now();
    info!(LOGGER, "Started at {}", time_started.to_rfc3339());

    struct Receiver;
    impl log::Log for Receiver {
        fn enabled(&self, _metadata: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            // wgpu_hal is noisy in maximum level
            // its downgraded to info
            static INFO_LEVEL_MOD_PREFIX: [&str; 3] = ["wgpu_hal", "wgpu_core", "naga"];

            let mod_name = record.module_path().unwrap_or("<unknown module>");
            for entry in INFO_LEVEL_MOD_PREFIX {
                // Was forced to be info level
                if mod_name.starts_with(entry) && record.level() > log::Level::Info {
                    return;
                }
            }

            match record.level() {
                log::Level::Error => error!(
                    LOGGER_FROM_LOG_CRATE,
                    "[mod {}] {}",
                    mod_name,
                    record.args()
                ),
                log::Level::Warn => warn!(
                    LOGGER_FROM_LOG_CRATE,
                    "[mod {}] {}",
                    mod_name,
                    record.args()
                ),
                log::Level::Info => info!(
                    LOGGER_FROM_LOG_CRATE,
                    "[mod {}] {}",
                    mod_name,
                    record.args()
                ),
                log::Level::Debug => debug!(
                    LOGGER_FROM_LOG_CRATE,
                    "[mod {}] {}",
                    mod_name,
                    record.args()
                ),
                log::Level::Trace => trace!(
                    LOGGER_FROM_LOG_CRATE,
                    "[mod {}] {}",
                    mod_name,
                    record.args()
                ),
            }
        }

        fn flush(&self) {}
    }

    log::set_max_level(log::LevelFilter::Trace);
    if let Err(e) = log::set_logger(&Receiver) {
        error!("Cannot set logger for log crate! Logs from other crates wont be available");
        error!("Reason: {e}");
    }

    let std_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(loc) = info.location() {
            fatal!(
                "Panic occured at {}:{}:{}",
                loc.file(),
                loc.line(),
                loc.column()
            );
        } else {
            fatal!("Panic occured at unknown location");
        }

        if let Some(message) = info.payload_as_str() {
            fatal!("Message: {}", message);
        } else {
            fatal!("No message was given");
        }

        info!(
            "Now continue calling to std's panic hook (won't be visible in log, its directly printed to stdout/stderr)"
        );
        std_panic(info);
    }));
}
