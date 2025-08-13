use crate::error::ParseError;

#[derive(Clone, Debug)]
pub struct ArgKey {
    value: String,
}

impl ArgKey {
    pub fn is_arg_key(value: &str) -> bool {
        return value.starts_with('-');
    }
    pub fn new(value: &str) -> Result<ArgKey, ParseError> {
        if Self::is_arg_key(value) {
            return Ok(ArgKey {
                value: String::from(value),
            });
        } else {
            return Err(ParseError::InvalidKey);
        }
    }
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        return Self {
            value: value.into(),
        };
    }
    pub fn value(&self) -> &String {
        return &self.value;
    }
    pub fn from_cmd(arg: &str) -> Result<(ArgKey, Option<String>), ParseError> {
        return match Self::is_arg_key(arg) {
            true => match arg.find('=') {
                Some(eq_pos) => unsafe {
                    let key = String::from(arg.get_unchecked(..eq_pos));
                    let value = String::from(arg.get_unchecked(eq_pos + 1..));
                    return Ok((Self { value: key }, Some(value)));
                },
                None => Ok((
                    Self {
                        value: String::from(arg),
                    },
                    None,
                )),
            },
            false => Err(ParseError::InvalidKey),
        };
    }
}

impl From<ArgKey> for String {
    fn from(value: ArgKey) -> Self {
        return value.value;
    }
}

impl<T: PartialEq<String>> PartialEq<T> for ArgKey {
    fn eq(&self, other: &T) -> bool {
        return other == &self.value;
    }
}
impl PartialEq<ArgKey> for str {
    fn eq(&self, other: &ArgKey) -> bool {
        return &other.value == self;
    }
}
impl Eq for ArgKey {}
