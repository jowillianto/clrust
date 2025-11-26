use std::{
    error::Error,
    fmt::{self, Display},
};

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
    fn from_args(kind: ParseErrorKind, args: fmt::Arguments<'_>) -> Self {
        Self {
            kind,
            msg: fmt::format(args),
            key: None,
        }
    }

    pub fn invalid_value(args: fmt::Arguments<'_>) -> Self {
        Self::from_args(ParseErrorKind::InvalidValue, args)
    }

    pub fn duplicate_argument(args: fmt::Arguments<'_>) -> Self {
        Self::from_args(ParseErrorKind::DuplicateArgument, args)
    }

    pub fn no_value_given(args: fmt::Arguments<'_>) -> Self {
        Self::from_args(ParseErrorKind::NoValueGiven, args)
    }

    pub fn not_required_argument(args: fmt::Arguments<'_>) -> Self {
        Self::from_args(ParseErrorKind::NotRequiredArgument, args)
    }

    pub fn not_argument_key(args: fmt::Arguments<'_>) -> Self {
        Self::from_args(ParseErrorKind::NotArgumentKey, args)
    }

    pub fn too_many_value_given(args: fmt::Arguments<'_>) -> Self {
        Self::from_args(ParseErrorKind::TooManyValueGiven, args)
    }

    pub fn not_positional(args: fmt::Arguments<'_>) -> Self {
        Self::from_args(ParseErrorKind::NotPositional, args)
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

impl Error for ParseError {}
