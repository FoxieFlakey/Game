use std::{backtrace::{Backtrace, BacktraceStatus}, error::Error, fmt::{Display, Write}, panic::Location};

mod idented_writer;
pub use idented_writer::IdentedWriter;

#[derive(Debug)]
pub struct ErrorWithContext {
    pub context: String,
    pub by: &'static Location<'static>,
    pub caused_by: Option<Box<dyn Error + 'static>>,
    pub stacktrace: Backtrace,

    // Additional error that was suppressed
    // such as case like there an error occur
    // and that error suppressed because
    // another occured when cleaning up.
    pub suppressed: Vec<SuppressionInfo>
}

#[derive(Debug)]
pub struct SuppressionInfo {
    pub by: &'static Location<'static>,
    pub error: Box<dyn Error + 'static>
}

impl ErrorWithContext {
    #[track_caller]
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            context: message.into(),
            caused_by: None,
            by: Location::caller(),
            stacktrace: Backtrace::capture(),
            suppressed: Vec::new()
        }
    }

    #[track_caller]
    pub fn add_suppressed<E: Into<Box<dyn Error + 'static>>>(mut self, error: E) -> Self {
        self.suppressed.push(SuppressionInfo {
            by: Location::caller(),
            error: error.into()
        });
        self
    }

    #[track_caller]
    pub fn with_cause<S: Into<String>, E: Into<Box<dyn Error + 'static>>>(message: S, error: E) -> Self {
        Self {
            context: message.into(),
            caused_by: Some(error.into()),
            by: Location::caller(),
            stacktrace: Backtrace::capture(),
            suppressed: Vec::new()
        }
    }
}

impl Display for ErrorWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.context)?;
        writeln!(f, "Error happened at: {}", self.by)?;
        match self.stacktrace.status() {
            BacktraceStatus::Disabled => writeln!(f, "Stacktrace disabled")?,
            BacktraceStatus::Unsupported => writeln!(f, "Stacktrace unsupported")?,
            BacktraceStatus::Captured => {
                writeln!(f, "Stacktrace:")?;    
                let mut idented = IdentedWriter::new(1, "  ", f, false);
                writeln!(&mut idented, "{}", self.stacktrace)?;
            },
            _ => writeln!(f, "Stacktrace status is unknown")?
        }

        if let Some(e) = self.caused_by.as_ref() {
            write!(f, "\nCaused by: ")?;
            let mut idented = IdentedWriter::new(1, "  ", f, true);
            write!(&mut idented, "{e}")?;
        }

        if self.suppressed.len() > 0 {
            writeln!(f, "\nSuppressed:")?;
            let mut idented = IdentedWriter::new(1, "  ", f, false);
            for (i, e) in self.suppressed.iter().enumerate() {
                write!(&mut idented, "[{i}] Suppressed by {}: {}", e.by, e.error)?;

                if i < self.suppressed.len() - 1 {
                    // Not the last item, emit newlines
                    writeln!(&mut idented, "\n")?;
                }
            }
        }

        Ok(())
    }
}

impl Error for ErrorWithContext {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.caused_by.as_deref()
    }
}



