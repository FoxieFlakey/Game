use std::{backtrace::{Backtrace, BacktraceStatus}, error::Error, fmt::{Display, Write}, panic::Location};

pub mod sig_safe;

mod idented_writer;
pub use idented_writer::IdentedWriter;

#[derive(Debug)]
pub struct ErrorWithContext<TheError: Error + ?Sized + 'static, Cause: Error + ?Sized + 'static = dyn Error + 'static> {
    pub the_error: Box<TheError>,
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
pub struct StringError {
    pub message: String
}

impl Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl From<String> for StringError {
    fn from(value: String) -> Self {
        Self { message: value }
    }
}

impl ErrorWithContext<StringError, dyn Error + 'static> {
    #[track_caller]
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            the_error: StringError::from(message.into()).into(),
            caused_by: None,
            by: Location::caller(),
            stacktrace: Backtrace::capture(),
            suppressed: Vec::new()
        }
    }
}

impl<TheError: Error + ?Sized + 'static, Cause: Error + 'static> ErrorWithContext<TheError, Cause> {
    #[track_caller]
    pub fn wrap<S: Into<String>>(self, message: S) -> ErrorWithContext<StringError, Self> {
        ErrorWithContext::with_cause_impl(StringError::from(message.into()).into(), Box::new(self), Location::caller())
    }
}

impl<TheError: Error + ?Sized + 'static> ErrorWithContext<TheError, dyn Error + 'static> {
    #[track_caller]
    pub fn wrap<S: Into<String>>(self, message: S) -> ErrorWithContext<StringError, Self> {
        ErrorWithContext::with_cause_impl(StringError::from(message.into()).into(), Box::new(self), Location::caller())
    }
}

impl<Cause: Error + ?Sized + 'static> ErrorWithContext<StringError, Cause> {
    #[track_caller]
    pub fn with_message<S: Into<String>>(message: S, cause: Box<Cause>) -> Self {
        Self::with_cause_impl(StringError::from(message.into()).into(), cause, Location::caller())
    }
}

impl<TheError: Error + ?Sized + 'static, Cause: Error + 'static> ErrorWithContext<TheError, Cause> {
    #[track_caller]
    pub fn add_suppressed<E: Into<Box<dyn Error + 'static>>>(mut self, error: E) -> Self {
        self.suppressed.push(SuppressionInfo {
            by: Location::caller(),
            error: error.into()
        });
        self
    }
    
    #[track_caller]
    pub fn with_cause(err: Box<TheError>, cause: Box<Cause>) -> Self {
        Self::with_cause_impl(err, cause, Location::caller())
    }
}

impl<TheError: Error + ?Sized + 'static, Cause: Error + ?Sized + 'static> ErrorWithContext<TheError, Cause> {
    fn with_cause_impl(err: Box<TheError>, cause: Box<Cause>, by: &'static Location<'static>) -> Self {
        Self {
            the_error: err,
            caused_by: Some(cause),
            by,
            stacktrace: Backtrace::capture(),
            suppressed: Vec::new()
        }
    }
}

impl<TheError: Error + 'static, Cause: Error + ?Sized + 'static> ErrorWithContext<TheError, Cause> {
	pub fn map<NewError, F>(self, mapper: F) -> ErrorWithContext<NewError, Cause>
		where NewError: Error + 'static,
			F: FnOnce(TheError) -> NewError
	{
		ErrorWithContext {
			the_error: Box::new(mapper(*self.the_error)),
			by: self.by,
			caused_by: self.caused_by,
			stacktrace: self.stacktrace,
			suppressed: self.suppressed
		}
	}
}

impl<TheError: Error + ?Sized + 'static, Cause: Error + ?Sized + 'static> Display for ErrorWithContext<TheError, Cause> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.the_error)?;
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

impl<ThisError: Error + ?Sized + 'static> Error for ErrorWithContext<ThisError, dyn Error + 'static> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.caused_by.as_ref().map(|x| &**x)
    }
}

impl<ThisError: Error + ?Sized + 'static, Cause: Error + 'static> Error for ErrorWithContext<ThisError, Cause> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.caused_by.as_ref().map(|x| x as &(dyn Error + 'static))
    }
}

impl ErrorWithContext<dyn Error + 'static, dyn Error + 'static> {
    #[track_caller]
    pub fn new_err<TheError: Error + 'static>(value: TheError) -> ErrorWithContext<dyn Error + 'static, dyn Error + 'static> {
        ErrorWithContext {
            the_error: Box::new(value) as Box<dyn Error + 'static>,
            by: Location::caller(),
            caused_by: None,
            stacktrace: Backtrace::capture(),
            suppressed: Vec::new()
        }
    }
}

impl<TheError: Error + 'static, Cause: Error + 'static> From<ErrorWithContext<TheError, Cause>> for ErrorWithContext<dyn Error + 'static, dyn Error + 'static> {
    fn from(value: ErrorWithContext<TheError, Cause>) -> ErrorWithContext<dyn Error + 'static, dyn Error + 'static> {
        ErrorWithContext {
            the_error: value.the_error as Box<dyn Error + 'static>,
            by: value.by,
            caused_by: value.caused_by.map(|x| x as Box<dyn Error + 'static>),
            stacktrace: value.stacktrace,
            suppressed: value.suppressed
        }
    }
}

impl<TheError: Error + 'static> From<ErrorWithContext<TheError>> for ErrorWithContext<dyn Error + 'static> {
    fn from(value: ErrorWithContext<TheError>) -> ErrorWithContext<dyn Error + 'static, dyn Error + 'static> {
        ErrorWithContext {
            the_error: value.the_error as Box<dyn Error + 'static>,
            by: value.by,
            caused_by: value.caused_by,
            stacktrace: value.stacktrace,
            suppressed: value.suppressed
        }
    }
}

#[track_caller]
pub fn add_err_context<TheError>(err: TheError) -> ErrorWithContext<TheError>
	where TheError: Error + 'static
{
	ErrorWithContext {
		the_error: err.into(),
		by: Location::caller(),
		caused_by: None,
		stacktrace: Backtrace::capture(),
		suppressed: Vec::new()
	}
}


