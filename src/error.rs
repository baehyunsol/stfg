#[derive(Debug)]
pub enum Error {
    CliError {
        message: String,
        span: Option<ragit_cli::RenderedSpan>,
    },
    FileError(ragit_fs::FileError),
    RusqliteError(rusqlite::Error),
    Base64DecodeError(base64::DecodeError),
    EdgeCase(String),
    CorruptedDataFile(String),
}

impl From<ragit_cli::Error> for Error {
    fn from(e: ragit_cli::Error) -> Self {
        Error::CliError {
            message: e.kind.render(),
            span: e.span,
        }
    }
}

impl From<ragit_fs::FileError> for Error {
    fn from(e: ragit_fs::FileError) -> Error {
        Error::FileError(e)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Error {
        Error::RusqliteError(e)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Error {
        Error::Base64DecodeError(e)
    }
}
