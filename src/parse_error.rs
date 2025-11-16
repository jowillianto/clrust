use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
    InvalidValue,
    DuplicateArgument,
    NoValueGiven,
    NotRequiredArgument,
    NotArgumentKey,
    TooManyValueGiven,
    NotPositional,
}

#[derive(Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub msg: String,
    pub key: Option<String>,
}

impl ParseError {
    pub fn invalid_value(msg: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::InvalidValue,
            msg: msg.into(),
            key: None,
        }
    }

    pub fn duplicate_argument(msg: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::DuplicateArgument,
            msg: msg.into(),
            key: None,
        }
    }

    pub fn no_value_given(msg: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::NoValueGiven,
            msg: msg.into(),
            key: None,
        }
    }

    pub fn not_required_argument(msg: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::NotRequiredArgument,
            msg: msg.into(),
            key: None,
        }
    }

    pub fn not_argument_key(msg: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::NotArgumentKey,
            msg: msg.into(),
            key: None,
        }
    }

    pub fn too_many_value_given(msg: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::TooManyValueGiven,
            msg: msg.into(),
            key: None,
        }
    }

    pub fn not_positional(msg: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::NotPositional,
            msg: msg.into(),
            key: None,
        }
    }
    pub fn key(mut self, k: impl Into<String>) -> Self {
        self.key = Some(k.into());
        self
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.key {
            None => write!(f, "{:?}: {}", self.kind, self.msg),
            Some(k) => write!(f, "{:?}: {} - {}", self.kind, k, self.msg),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
