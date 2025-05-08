use std::{fmt, result};

/// A result of a function that may return a `Error`.
pub type Result<T> = result::Result<T, Error>;

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub enum Error {
    /// Transfer allocation failed.
    TransferAlloc,

    /// Poll timed out
    PollTimeout,

    /// Transfer is stalled
    Stall,

    /// Device was disconnected
    Disconnected,

    /// Device sent more data than expected
    Overflow,

    /// Other Error
    Other(&'static str),

    /// Error code on other failure
    Errno(&'static str, i32),

    /// Transfer was cancelled
    Cancelled,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> result::Result<(), fmt::Error> {
        match self {
            Error::TransferAlloc => fmt.write_str("Transfer allocation failed"),
            Error::PollTimeout => fmt.write_str("Poll timed out"),
            Error::Stall => fmt.write_str("Transfer is stalled"),
            Error::Disconnected => fmt.write_str("Device was disconnected"),
            Error::Overflow => fmt.write_str("Device sent more data than expected"),
            Error::Other(s) => write!(fmt, "Other Error: {s}"),
            Error::Errno(s, n) => write!(fmt, "{s} ERRNO: {n}"),
            Error::Cancelled => fmt.write_str("Transfer was cancelled"),
        }
    }
}

impl std::error::Error for Error {}
