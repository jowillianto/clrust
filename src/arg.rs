use crate::{ArgKey, ParseError, ParsedArg, tui};

pub trait ArgValidator {
    fn id(&self) -> Option<String> {
        None
    }
    fn validate(&self, _v: Option<&str>) -> Result<(), ParseError> {
        Ok(())
    }
    fn post_validate(&self, _k: Option<&ArgKey>, _args: &ParsedArg) -> Result<(), ParseError> {
        Ok(())
    }
    fn help(&self) -> Option<tui::DomNode> {
        None
    }
}

#[derive(Debug, Default, Clone)]
pub struct ArgOptionValidator {
    options: Vec<(String, Option<String>)>,
}

impl ArgOptionValidator {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn option(
        mut self,
        value: impl Into<String> + PartialEq<String>,
        help: impl Into<Option<String>>,
    ) -> ArgOptionValidator {
        let help = help.into();
        if let Some(option) = self.options.iter_mut().find(|(v, _)| value == *v) {
            option.1 = help;
        } else {
            self.options.push((value.into(), help))
        }
        self
    }
    pub fn iter(&self) -> impl Iterator<Item = &(String, Option<String>)> {
        self.options.iter()
    }
    pub fn len(&self) -> usize {
        self.options.len()
    }
    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }
}

impl ArgValidator for ArgOptionValidator {
    fn id(&self) -> Option<String> {
        Some(String::from("Option"))
    }
    fn help(&self) -> Option<tui::DomNode> {
        if self.is_empty() {
            return None;
        }
        let mut layout = tui::Layout::default();
        for (v, h) in self.iter() {
            if let Some(h) = h {
                layout = layout.append_child(tui::Paragraph(format!("- {}: {}", v, h)));
            } else {
                layout = layout.append_child(tui::Paragraph(format!("- {}: <no-help>", v)));
            }
        }
        Some(tui::VStack(layout))
    }
    fn validate(&self, v: Option<&str>) -> Result<(), ParseError> {
        match v {
            None => Err(ParseError::no_value_given("")),
            Some(v) => match self.iter().find(|(k, _)| k == v) {
                None => Err(ParseError::invalid_value(format!(
                    "{} is not a valid option",
                    v
                ))),
                Some(_) => Ok(()),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ArgCountValidator {
    min_size: u64,
    max_size: u64,
}

impl ArgCountValidator {
    pub fn range(min_size: u64, max_size: u64) -> Self {
        Self { min_size, max_size }
    }

    pub fn at_least(min_size: u64) -> Self {
        Self::range(min_size, u64::MAX)
    }

    pub fn at_most(max_size: u64) -> Self {
        Self::range(0, max_size)
    }

    pub fn equal_to(value: u64) -> Self {
        Self::range(value, value)
    }

    pub fn one() -> Self {
        Self::equal_to(1)
    }
}

impl ArgValidator for ArgCountValidator {
    fn id(&self) -> Option<String> {
        Some(String::from("ArgCountValidator"))
    }

    fn help(&self) -> Option<tui::DomNode> {
        let desc = if self.min_size == self.max_size && self.min_size != 1 {
            format!("Arg Count: ={}", self.min_size)
        } else if self.min_size == 0 && self.max_size == 1 {
            "Optional".into()
        } else if self.min_size == 1 && self.max_size == 1 {
            "Required".into()
        } else if self.min_size == 1 && self.max_size == u64::MAX {
            format!("Arg Count: >= {}", self.min_size)
        } else {
            format!("Arg Count: {} <= n <= {}", self.min_size, self.max_size)
        };

        Some(tui::Paragraph(desc))
    }

    fn post_validate(&self, key: Option<&ArgKey>, args: &ParsedArg) -> Result<(), ParseError> {
        let count = key.map(|k| args.count(k) as u64).unwrap_or(1);
        if count < self.min_size || count > self.max_size {
            Err(ParseError::too_many_value_given(format!(
                "{} not in {} <= x <= {}",
                count, self.min_size, self.max_size
            )))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ArgEmptyValidator {
    allow_empty: bool,
}

impl ArgEmptyValidator {
    pub const fn new(allow_empty: bool) -> Self {
        Self { allow_empty }
    }

    pub const fn allow() -> Self {
        Self::new(true)
    }

    pub const fn require_value() -> Self {
        Self::new(false)
    }
}

impl ArgValidator for ArgEmptyValidator {
    fn id(&self) -> Option<String> {
        Some(String::from("ArgEmptyValidator"))
    }

    fn help(&self) -> Option<tui::DomNode> {
        if self.allow_empty {
            Some(tui::Paragraph(String::from("Flag")))
        } else {
            None
        }
    }

    fn validate(&self, value: Option<&str>) -> Result<(), ParseError> {
        match (self.allow_empty, value) {
            (true, _) => Ok(()),
            (false, Some(_)) => Ok(()),
            (false, None) => Err(ParseError::no_value_given("")),
        }
    }

    fn post_validate(&self, _k: Option<&ArgKey>, _args: &ParsedArg) -> Result<(), ParseError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct Arg {
    help_text: Option<String>,
    validators: Vec<Box<dyn ArgValidator>>,
}

impl ArgValidator for Arg {
    fn id(&self) -> Option<String> {
        None
    }

    fn validate(&self, value: Option<&str>) -> Result<(), ParseError> {
        for validator in &self.validators {
            validator.validate(value)?;
        }
        Ok(())
    }

    fn post_validate(&self, key: Option<&ArgKey>, args: &ParsedArg) -> Result<(), ParseError> {
        for validator in &self.validators {
            validator.post_validate(key, args)?;
        }
        Ok(())
    }

    fn help(&self) -> Option<tui::DomNode> {
        self.help_text
            .as_ref()
            .map(|text| tui::Paragraph(text.clone()))
    }
}

impl Arg {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn help(mut self, text: impl Into<String>) -> Self {
        self.help_text = Some(text.into());
        self
    }

    pub fn validate(mut self, validator: impl ArgValidator + 'static) -> Self {
        self.validators.push(Box::new(validator));
        self
    }

    pub fn n_at_least(self, min_size: u64) -> Self {
        self.validate(ArgCountValidator::at_least(min_size))
    }

    pub fn n_at_most(self, max_size: u64) -> Self {
        self.validate(ArgCountValidator::at_most(max_size))
    }

    pub fn n_equal_to(self, value: u64) -> Self {
        self.validate(ArgCountValidator::equal_to(value))
    }

    pub fn n_range(self, min_size: u64, max_size: u64) -> Self {
        self.validate(ArgCountValidator::range(min_size, max_size))
    }

    pub fn require_value(self) -> Self {
        self.validate(ArgEmptyValidator::require_value())
    }

    pub fn as_flag(self) -> Self {
        self.validate(ArgEmptyValidator::allow())
    }

    pub fn required(self) -> Self {
        self.require_value().n_equal_to(1)
    }

    pub fn optional(self) -> Self {
        self.n_range(0, 1)
    }

    pub fn len(&self) -> usize {
        self.validators.len()
    }

    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }
}
