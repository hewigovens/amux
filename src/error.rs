use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct AmuxError(String);

impl AmuxError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for AmuxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for AmuxError {}

pub type DynError = Box<dyn Error + Send + Sync + 'static>;

pub type Result<T> = std::result::Result<T, DynError>;

pub fn fail(msg: impl Into<String>) -> DynError {
    Box::new(AmuxError::new(msg))
}

pub fn bail<T>(msg: impl Into<String>) -> Result<T> {
    Err(fail(msg))
}

pub fn with_context(err: impl Into<DynError>, msg: impl Into<String>) -> DynError {
    let err = err.into();
    fail(format!("{}: {}", msg.into(), err))
}
