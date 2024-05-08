use crate::driver::DriverError;


#[derive(Debug)]
pub enum AppError {
    Display(DriverError),
    Io(std::io::Error),
    Utf8(std::string::FromUtf8Error),
    Regex(regex::Error),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AppError {}

impl From<DriverError> for AppError {
    fn from(value: DriverError) -> Self {
        Self::Display(value)
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl From<regex::Error> for AppError {
    fn from(value: regex::Error) -> Self {
        Self::Regex(value)
    }
}
