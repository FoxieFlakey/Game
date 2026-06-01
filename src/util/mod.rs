use std::{backtrace::{Backtrace, BacktraceStatus}, error::Error, fmt::{Display, Write}, panic::Location};

mod idented_writer;
pub use idented_writer::IdentedWriter;

#[derive(Debug)]
pub struct ErrorWithContext<Cause: Error + ?Sized + 'static> {
    pub context: String,
    pub by: &'static Location<'static>,
    pub caused_by: Option<Box<Cause>>,
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

#[derive(Debug)]
pub struct EmptyError {}

impl Display for EmptyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<EmptyError is no error>")
    }
}

impl Error for EmptyError {
}

impl ErrorWithContext<EmptyError> {
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
}

impl<Cause: Error + 'static> ErrorWithContext<Cause> {
    #[track_caller]
    pub fn wrap<S: Into<String>>(self, message: S) -> ErrorWithContext<Self> {
        ErrorWithContext::with_cause_impl(message, self.into(), Location::caller())
    }
}

impl<Cause: Error + ?Sized + 'static> ErrorWithContext<Cause> {
    #[track_caller]
    pub fn add_suppressed<E: Into<Box<dyn Error + 'static>>>(mut self, error: E) -> Self {
        self.suppressed.push(SuppressionInfo {
            by: Location::caller(),
            error: error.into()
        });
        self
    }

    #[track_caller]
    pub fn with_cause<S: Into<String>>(message: S, error: Box<Cause>) -> Self {
        Self::with_cause_impl(message, error, Location::caller())
    }

    fn with_cause_impl<S: Into<String>>(message: S, error: Box<Cause>, by: &'static Location<'static>) -> Self {
        Self {
            context: message.into(),
            caused_by: Some(error),
            by: Location::caller(),
            stacktrace: Backtrace::capture(),
            suppressed: Vec::new()
        }
    }
}

impl<Cause: Error + ?Sized + 'static> Display for ErrorWithContext<Cause> {
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

impl Error for ErrorWithContext<dyn Error + 'static> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.caused_by.as_ref().map(|x| &**x)
    }
}

impl<Cause: Error + 'static> Error for ErrorWithContext<Cause> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.caused_by.as_ref().map(|x| x as &(dyn Error + 'static))
    }
}

impl<Cause: Error + 'static> From<ErrorWithContext<Cause>> for ErrorWithContext<dyn Error + 'static> {
    fn from(value: ErrorWithContext<Cause>) -> ErrorWithContext<dyn Error + 'static> {
        ErrorWithContext {
            context: value.context,
            by: value.by,
            caused_by: value.caused_by.map(|x| x as Box<dyn Error + 'static>),
            stacktrace: value.stacktrace,
            suppressed: value.suppressed
        }
    }
}



