use std::fmt::Display;

use crate::ParseError;

#[derive(Debug, Clone)]
pub struct ArgKey {
    pub value: String,
}

impl ArgKey {
    pub fn is_arg_key(k: &str) -> bool {
        (k.starts_with("--") && k.len() > 2) || (k.starts_with("-") && k.len() == 2)
    }

    pub fn make(k: &str) -> Result<Self, ParseError> {
        match Self::is_arg_key(k) {
            true => Ok(Self::make_unchecked(k)),
            false => Err(ParseError::not_argument_key(k)),
        }
    }

    fn make_unchecked(k: &str) -> Self {
        Self { value: k.into() }
    }

    pub fn parse_arg(k: &str) -> Result<(Self, Option<&str>), ParseError> {
        if !Self::is_arg_key(k) {
            return Err(ParseError::not_argument_key(k));
        }
        match k.find("=") {
            None => Ok((ArgKey::make_unchecked(k), None)),
            Some(eq_pos) => {
                let (pre_eq, post_eq) = k.split_at(eq_pos);
                Ok((ArgKey::make_unchecked(pre_eq), Some(post_eq)))
            }
        }
    }
}

impl From<ArgKey> for String {
    fn from(k: ArgKey) -> Self {
        k.value
    }
}

impl PartialEq<ArgKey> for str {
    fn eq(&self, other: &ArgKey) -> bool {
        other.value == self
    }
}

impl PartialEq<ArgKey> for ArgKey {
    fn eq(&self, other: &ArgKey) -> bool {
        other.value == self.value
    }
}

impl Display for ArgKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
