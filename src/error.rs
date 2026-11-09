use crate::ffi;
use std::fmt;

/// The category of an [`Error`], corresponding to RocksDB's `Status::Code`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    NotFound,
    Corruption,
    NotSupported,
    InvalidArgument,
    IOError,
    MergeInProgress,
    Incomplete,
    ShutdownInProgress,
    TimedOut,
    Aborted,
    Busy,
    Expired,
    TryAgain,
    CompactionTooLarge,
    ColumnFamilyDropped,
    Unknown,
}

/// An error returned by RocksDB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Returns the error message, as formatted by RocksDB.
    pub fn into_string(self) -> String {
        self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl From<ffi::Status> for Error {
    fn from(status: ffi::Status) -> Self {
        let kind = match status.code {
            ffi::StatusCode::NotFound => ErrorKind::NotFound,
            ffi::StatusCode::Corruption => ErrorKind::Corruption,
            ffi::StatusCode::NotSupported => ErrorKind::NotSupported,
            ffi::StatusCode::InvalidArgument => ErrorKind::InvalidArgument,
            ffi::StatusCode::IOError => ErrorKind::IOError,
            ffi::StatusCode::MergeInProgress => ErrorKind::MergeInProgress,
            ffi::StatusCode::Incomplete => ErrorKind::Incomplete,
            ffi::StatusCode::ShutdownInProgress => ErrorKind::ShutdownInProgress,
            ffi::StatusCode::TimedOut => ErrorKind::TimedOut,
            ffi::StatusCode::Aborted => ErrorKind::Aborted,
            ffi::StatusCode::Busy => ErrorKind::Busy,
            ffi::StatusCode::Expired => ErrorKind::Expired,
            ffi::StatusCode::TryAgain => ErrorKind::TryAgain,
            ffi::StatusCode::CompactionTooLarge => ErrorKind::CompactionTooLarge,
            ffi::StatusCode::ColumnFamilyDropped => ErrorKind::ColumnFamilyDropped,
            // cxx enums can hold any value of their underlying type.
            _ => ErrorKind::Unknown,
        };
        Self::new(kind, status.message)
    }
}

/// Runs an FFI call that reports errors through a `Status` out parameter.
pub(crate) fn ffi_try<T>(f: impl FnOnce(&mut ffi::Status) -> T) -> Result<T, Error> {
    let mut status = ffi::Status::ok();
    let value = f(&mut status);
    if status.code == ffi::StatusCode::Ok {
        Ok(value)
    } else {
        Err(status.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_try_ok() {
        assert_eq!(ffi_try(|_| 42), Ok(42));
    }

    #[test]
    fn ffi_try_error() {
        let err = ffi_try(|status| {
            status.code = ffi::StatusCode::Incomplete;
            status.message = "Result incomplete: too many skips".into();
        })
        .unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Incomplete);
        assert_eq!(err.to_string(), "Result incomplete: too many skips");
        assert_eq!(err.into_string(), "Result incomplete: too many skips");
    }

    #[test]
    fn unknown_status_code() {
        let status = ffi::Status {
            code: ffi::StatusCode { repr: u8::MAX },
            message: String::new(),
        };
        assert_eq!(Error::from(status).kind(), ErrorKind::Unknown);
    }
}
