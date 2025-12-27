use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Format,
    Io,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Format => write!(f, "FORMAT_ERROR"),
            Self::Io => write!(f, "IO_ERROR"),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    msg: String,
}

impl Error {
    pub fn new(kind: ErrorKind, args: fmt::Arguments<'_>) -> Self {
        let mut msg = String::new();
        // Prefix with the error type name to mirror the C++ implementation.
        let _ = fmt::write(&mut msg, format_args!("{kind}: "));
        let _ = fmt::write(&mut msg, args);
        Self { kind, msg }
    }

    pub fn format_error(args: fmt::Arguments<'_>) -> Self {
        Self::new(ErrorKind::Format, args)
    }

    pub fn io_error(args: fmt::Arguments<'_>) -> Self {
        Self::new(ErrorKind::Io, args)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::io_error(format_args!("{}", e))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Level {
    pub name: &'static str,
    pub value: u8,
}

impl Level {
    pub fn trace() -> Self {
        Self {
            name: "TRACE",
            value: 0,
        }
    }

    pub fn debug() -> Self {
        Self {
            name: "DEBUG",
            value: 10,
        }
    }

    pub fn info() -> Self {
        Self {
            name: "INFO",
            value: 20,
        }
    }

    pub fn warn() -> Self {
        Self {
            name: "WARN",
            value: 30,
        }
    }

    pub fn error() -> Self {
        Self {
            name: "ERROR",
            value: 40,
        }
    }

    pub fn critical() -> Self {
        Self {
            name: "CRITICAL",
            value: 50,
        }
    }
}

impl PartialOrd for Level {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Level {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

pub struct Context<'a> {
    pub level: Level,
    pub location: &'static std::panic::Location<'static>,
    pub time: chrono::DateTime<chrono::Utc>,
    pub message: fmt::Arguments<'a>,
}

pub trait Emitter: Send + Sync {
    fn emit(&self, v: String) -> Result<(), Error>;
}

pub trait Formatter: Send + Sync {
    fn fmt(&self, ctx: &Context<'_>) -> Result<String, Error>;
}

pub trait Filter: Send + Sync {
    fn allow(&self, ctx: &Context<'_>) -> bool;
}
