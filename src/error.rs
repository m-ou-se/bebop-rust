use std::fmt;
use std::str::Utf8Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    UnexpectedEnd,
    InvalidUtf8,
    UnknownEnumValue,
    UnknownUnionTag,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedEnd => f.write_str("unexpected end of message"),
            Error::InvalidUtf8 => f.write_str("invalid UTF-8 in string"),
            Error::UnknownEnumValue => f.write_str("unknown enum value"),
            Error::UnknownUnionTag => f.write_str("unknown union tag"),
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Error {
        Error::InvalidUtf8
    }
}
