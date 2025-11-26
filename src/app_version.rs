use std::cmp;
use std::fmt;

use crate::ParseError;

#[derive(Default, cmp::PartialEq, Copy, Debug, Clone)]
pub struct AppVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl AppVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl cmp::PartialOrd for AppVersion {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        if self.major > other.major
            || self.major == other.major && self.minor > other.minor
            || self.major == other.major && self.minor == other.minor && self.patch > other.patch
        {
            Some(cmp::Ordering::Greater)
        } else if self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
        {
            Some(cmp::Ordering::Equal)
        } else {
            Some(cmp::Ordering::Less)
        }
    }
}
impl fmt::Display for AppVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl TryFrom<&str> for AppVersion {
    type Error = ParseError;
    fn try_from(v: &str) -> Result<AppVersion, ParseError> {
        let mut split_it = v.split('.');
        let major_s = split_it.next();
        if major_s.is_none() {
            return Err(ParseError::invalid_value(format_args!("{v}")));
        }
        let minor_s = split_it.next();
        if minor_s.is_none() {
            return Err(ParseError::invalid_value(format_args!("{v}")));
        }
        let patch_s = split_it.next();
        if patch_s.is_none() {
            return Err(ParseError::invalid_value(format_args!("{v}")));
        }
        match major_s.unwrap().parse::<u32>() {
            Ok(major) => match minor_s.unwrap().parse::<u32>() {
                Ok(minor) => match patch_s.unwrap().parse::<u32>() {
                    Ok(patch) => Ok(AppVersion {
                        major,
                        minor,
                        patch,
                    }),
                    Err(_) => Err(ParseError::invalid_value(format_args!("{v}"))),
                },
                Err(_) => Err(ParseError::invalid_value(format_args!("{v}"))),
            },
            Err(_) => Err(ParseError::invalid_value(format_args!("{v}"))),
        }
    }
}
