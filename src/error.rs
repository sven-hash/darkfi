use rusqlite;
use std::fmt;

use crate::net::error::NetError;
use crate::state;
use crate::vm::ZKVMError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
//#[derive(Debug, Copy, Clone)]

// need to be able to copy the errors into theads
// net error has clone and copy attribute 
// copy vs clone
//struct Error;

pub enum Error {
    Foo,
    CommitsDontAdd,
    InvalidCredential,
    TransactionPedersenCheckFailed,
    TokenAlreadySpent,
    InputTokenVerifyFailed,
    RangeproofPedersenMatchFailed,
    ProofsFailed,
    MissingProofs,
    Io(std::io::ErrorKind),
    /// VarInt was encoded in a non-minimal way
    NonMinimalVarInt,
    /// Parsing error
    ParseFailed(&'static str),
    ParseIntError,
    AsyncChannelError,
    MalformedPacket,
    AddrParseError,
    BadVariableRefType,
    BadOperationType,
    BadConstraintType,
    InvalidParamName,
    MissingParams,
    VMError,
    BadContract,
    Groth16Error,
    RusqliteError,
    OperationFailed,
    ConnectFailed,
    ConnectTimeout,
    ChannelStopped,
    ChannelTimeout,
    ServiceStopped,
    Utf8Error,
    NoteDecryptionFailed,
    ServicesError(&'static str),
    ZMQError,
    VerifyFailed,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Foo => f.write_str("foo"),
            Error::CommitsDontAdd => f.write_str("Commits don't add up properly"),
            Error::InvalidCredential => f.write_str("Credential is invalid"),
            Error::TransactionPedersenCheckFailed => {
                f.write_str("Transaction pedersens for input and output don't sum up")
            }
            Error::TokenAlreadySpent => f.write_str("This input token is already spent"),
            Error::InputTokenVerifyFailed => f.write_str("Input token verify of credential failed"),
            Error::RangeproofPedersenMatchFailed => {
                f.write_str("Rangeproof pedersen check for match failed")
            }
            Error::ProofsFailed => f.write_str("Proof validation failed"),
            Error::MissingProofs => f.write_str("Missing proofs"),
            Error::Io(ref err) => write!(f, "io error:{:?}", err),
            Error::NonMinimalVarInt => f.write_str("non-minimal varint"),
            Error::ParseFailed(ref err) => write!(f, "parse failed: {}", err),
            Error::ParseIntError => f.write_str("Parse int error"),
            Error::AsyncChannelError => f.write_str("Async_channel error"),
            Error::MalformedPacket => f.write_str("Malformed packet"),
            Error::AddrParseError => f.write_str("Unable to parse address"),
            Error::BadVariableRefType => f.write_str("Bad variable ref type byte"),
            Error::BadOperationType => f.write_str("Bad operation type byte"),
            Error::BadConstraintType => f.write_str("Bad constraint type byte"),
            Error::InvalidParamName => f.write_str("Invalid param name"),
            Error::MissingParams => f.write_str("Missing params"),
            Error::VMError => f.write_str("VM error"),
            Error::BadContract => f.write_str("Contract is poorly defined"),
            Error::Groth16Error => f.write_str("Groth16 error"),
            Error::RusqliteError => f.write_str("Rusqlite error"),
            Error::OperationFailed => f.write_str("Operation failed"),
            Error::ConnectFailed => f.write_str("Connection failed"),
            Error::ConnectTimeout => f.write_str("Connection timed out"),
            Error::ChannelStopped => f.write_str("Channel stopped"),
            Error::ChannelTimeout => f.write_str("Channel timed out"),
            Error::ServiceStopped => f.write_str("Service stopped"),
            Error::Utf8Error => f.write_str("Malformed UTF8"),
            Error::NoteDecryptionFailed => f.write_str("Unable to decrypt mint note"),
            Error::ServicesError(ref err) => write!(f, "Services error: {}", err),
            Error::ZMQError => f.write_str("ZMQ error"),
            Error::VerifyFailed => f.write_str("Verify failed"),
        }
    }
}

// TODO: Match statement to parse external errors into strings.
impl From<zeromq::ZmqError> for Error {
    fn from(err: zeromq::ZmqError) -> Error {
        Error::ZMQError
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err.kind())
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Error {
        Error::RusqliteError
    }
}

impl From<ZKVMError> for Error {
    fn from(err: ZKVMError) -> Error {
        Error::VMError
    }
}

impl From<bellman::SynthesisError> for Error {
    fn from(err: bellman::SynthesisError) -> Error {
        Error::Groth16Error
    }
}

impl<T> From<async_channel::SendError<T>> for Error {
    fn from(err: async_channel::SendError<T>) -> Error {
        Error::AsyncChannelError
    }
}

impl From<async_channel::RecvError> for Error {
    fn from(err: async_channel::RecvError) -> Error {
        Error::AsyncChannelError
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(_err: std::net::AddrParseError) -> Error {
        Error::AddrParseError
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_err: std::num::ParseIntError) -> Error {
        Error::ParseIntError
    }
}

impl From<NetError> for Error {
    fn from(err: NetError) -> Error {
        match err {
            NetError::OperationFailed => Error::OperationFailed,
            NetError::ConnectFailed => Error::ConnectFailed,
            NetError::ConnectTimeout => Error::ConnectTimeout,
            NetError::ChannelStopped => Error::ChannelStopped,
            NetError::ChannelTimeout => Error::ChannelTimeout,
            NetError::ServiceStopped => Error::ServiceStopped,
        }
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(_err: std::string::FromUtf8Error) -> Error {
        Error::Utf8Error
    }
}

impl From<state::VerifyFailed> for Error {
    fn from(err: state::VerifyFailed) -> Error {
        Error::VerifyFailed
    }
}

