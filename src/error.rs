use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),

    #[error("Invalid request format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    IoError(String),
}

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Timeout")]
    Timeout,
}

impl From<reqwest::Error> for HttpError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            HttpError::Timeout
        } else {
            HttpError::RequestFailed(err.to_string())
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err.into())
    }
}
