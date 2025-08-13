use core::fmt;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    ValueRequired,
    TooManyOrTooLittleValue,
    NotPositional,
    EndOfArgumentFound,
    InvalidValue,
    InvalidKey,
    NotEnd,
}

pub struct ArgParseError {
    pos: String,
    err: ParseError,
}
impl ArgParseError {
    pub fn new(pos: impl Into<String>, err: ParseError) -> Self {
        return Self {
            pos: pos.into(),
            err: err,
        };
    }
    pub fn or_else(pos: impl Into<String>) -> impl FnOnce(ParseError) -> Self {
        return |e| ArgParseError::new(pos, e);
    }
}
impl fmt::Display for ArgParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}: {}", self.pos, self.err);
    }
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{:?}", self);
    }
}
