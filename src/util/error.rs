// Custom error handling I made
// because ErrorWithContext becomed ugly .w.
//
// Without the 'cause' it is wrapped inside
// the error itself not managed by Error struct

use std::backtrace::{Backtrace, BacktraceStatus};
use std::error::Error;
use std::fmt::{self, Display, Write};
use std::panic::Location;

use crate::util::IdentedWriter;

#[derive(Debug)]
pub struct CustomError<T: ?Sized + 'static> {
    // Context of what attempting to be done
    context: Option<String>,
    stacktrace: Backtrace,

    // Location where error is constructed
    // which is almost all of time where
    // its occured
    by: &'static Location<'static>,

    // Errors that occured while handling
    // the primary error.
    suppressed: Vec<Box<CustomError<Box<dyn Error + Send>>>>,
    err: T,
}

impl<T: Display + ?Sized> Display for CustomError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.err, f)
    }
}

impl<T: Error + ?Sized> Error for CustomError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.err.source()
    }
}

pub trait CustomErrorExt: Error + Sized {
    fn context<S: Into<String>>(self, msg: S) -> CustomError<Self>;
    fn into_custom_err(self) -> CustomError<Self>;
}

impl<T: Error + Sized> CustomErrorExt for T {
    #[track_caller]
    fn context<S: Into<String>>(self, msg: S) -> CustomError<Self> {
        CustomError {
            by: Location::caller(),
            context: Some(msg.into()),
            err: self,
            stacktrace: Backtrace::capture(),
            suppressed: Vec::new(),
        }
    }

    #[track_caller]
    fn into_custom_err(self) -> CustomError<Self> {
        CustomError {
            by: Location::caller(),
            context: None,
            err: self,
            stacktrace: Backtrace::capture(),
            suppressed: Vec::new(),
        }
    }
}

impl<'a, E: Sized + Error + 'a> CustomError<E> {
    pub fn convert<NewE: Error + From<E>>(self) -> CustomError<NewE> {
        CustomError {
            by: self.by,
            context: self.context,
            err: self.err.into(),
            stacktrace: self.stacktrace,
            suppressed: self.suppressed,
        }
    }

    pub fn get_err(&self) -> &E {
        &self.err
    }

    pub fn into_boxed(self) -> CustomError<Box<dyn Error + 'a>> {
        CustomError {
            err: Box::new(self.err),
            by: self.by,
            context: self.context,
            stacktrace: self.stacktrace,
            suppressed: self.suppressed,
        }
    }
}

impl<E: Sized + Error> From<E> for CustomError<E> {
    #[track_caller]
    fn from(value: E) -> Self {
        Self {
            by: Location::caller(),
            context: None,
            err: value,
            stacktrace: Backtrace::capture(),
            suppressed: Vec::new(),
        }
    }
}

pub struct Printable<'a>(pub &'a CustomError<Box<dyn Error>>);

impl Display for Printable<'_> {
    fn fmt(&self, writer: &mut fmt::Formatter) -> fmt::Result {
        writeln!(writer, "Error occured: {}", &self.0.err)?;
        if let Some(context) = &self.0.context {
            writeln!(writer, "Context: {}", context)?;
        } else {
            writeln!(writer, "No context provided")?;
        }

        if matches!(self.0.stacktrace.status(), BacktraceStatus::Captured) {
            writeln!(writer, "Stack trace: ")?;
            writeln!(writer, "{}", self.0.stacktrace)?;
        } else {
            writeln!(writer, "Stack trace unavailable")?;
        }

        if self.0.suppressed.len() > 0 {
            writeln!(writer, "Suppressed errors:")?;
            for (idx, e) in self.0.suppressed.iter().enumerate() {
                let mut writer = IdentedWriter::new(1, "  ", writer, false);
                writeln!(&mut writer, "[{idx}] Suppressed error: {}", &e.err)?;
            }
        }

        Ok(())
    }
}

impl<E: Sized + Error> From<CustomError<E>> for Box<CustomError<dyn Error>> {
    fn from(value: CustomError<E>) -> Self {
        Box::new(CustomError { ..value })
    }
}
