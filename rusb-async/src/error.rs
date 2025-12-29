use std::{fmt, result};

/// A result of a function that may return a `Error`.
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// No transfers pending
    NoTransfersPending,

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
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        match self {
            Error::NoTransfersPending => fmt.write_str("No transfers pending"),
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
