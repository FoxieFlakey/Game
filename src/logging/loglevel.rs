use std::fmt::Display;

pub enum LogLevel {
    Fatal,
    Alert,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fatal => write!(f, "Fatal"),
            Self::Alert => write!(f, "Alert"),
            Self::Error => write!(f, "Error"),
            Self::Warning => write!(f, "Warning"),
            Self::Info => write!(f, "Info"),
            Self::Debug => write!(f, "Debug"),
            Self::Trace => write!(f, "Trace"),
        }
    }
}
