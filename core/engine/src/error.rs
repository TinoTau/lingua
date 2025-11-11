use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct EngineError {
    message: Cow<'static, str>,
}

impl EngineError {
    pub fn new<T>(message: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for EngineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for EngineError {}

pub type EngineResult<T> = Result<T, EngineError>;
