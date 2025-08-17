use crate::arg_key::ArgKey;
use crate::error::ParseError;
use crate::parsed_arg::ParsedArg;
use crate::terminal::TerminalNodes;

pub trait ArgValidator {
    fn validator_id(&self) -> Option<String> {
        None
    }
    fn help(&self, _: &mut TerminalNodes) {
    }
    fn validate(&self, _: Option<&String>) -> Result<(), ParseError> {
        Ok(())
    }
    fn post_validate(&self, _: Option<&ArgKey>, _: &ParsedArg) -> Result<(), ParseError> {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ArgOption {
    value: String,
    help_text: Option<String>,
}

impl ArgOption {
    fn new(v: impl Into<String>) -> ArgOption {
        Self {
            value: v.into(),
            help_text: None,
        }
    }
    fn help(&mut self, h: impl Into<String>) -> &mut Self {
        self.help_text = Some(h.into());
        self
    }
    fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

#[derive(Debug, Default)]
pub struct ArgOptions {
    options: Vec<ArgOption>,
}

impl ArgOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_option(&mut self, v: impl Into<String> + PartialEq<String>) -> &mut Self {
        if !self.iter().any(|opt| v == opt.value) {
            self.options.push(ArgOption::new(v))
        }
        self
    }
    pub fn add_option_help(
        &mut self,
        v: impl Into<String> + PartialEq<String>,
        help: impl Into<String>,
    ) -> &mut Self {
        if let Some(opt) = self.options.iter_mut().find(|opt| v == opt.value) {
            opt.help(help.into());
        } else {
            self.options.push(ArgOption::new(v).help(help).take());
        }
        self
    }

    fn check(&self, v: &impl PartialEq<String>) -> Result<(), ParseError> {
        return match self.iter().any(|opt| v == &opt.value) {
            true => Ok(()),
            false => Err(ParseError::InvalidValue),
        };
    }

    pub fn iter(&self) -> impl Iterator<Item = &ArgOption> {
        self.options.iter()
    }
    pub fn len(&self) -> usize {
        self.options.len()
    }

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

impl ArgValidator for ArgOptions {
    fn validator_id(&self) -> Option<String> {
        Some(String::from("ArgOption"))
    }
    fn help(&self, nodes: &mut TerminalNodes) {
        if nodes.len() == 0 {
            return;
        }
        nodes.append_node("Options: ").new_line();
        for opt in self.iter() {
            if let Some(h) = &opt.help_text {
                nodes.append_node(format!("- {}: {}", &opt.value, h));
            } else {
                nodes.append_node(format!("- {}: <no-help>", &opt.value));
            }
            nodes.new_line();
        }
    }
    fn validate(&self, v: Option<&String>) -> Result<(), ParseError> {
        match v {
            None => Err(ParseError::ValueRequired),
            Some(v) => self.check(v),
        }
    }
}

#[derive(Debug, Clone)]
struct CountValidator {
    max_size: usize,
    min_size: usize,
}

impl CountValidator {
    pub fn range(min_size: usize, max_size: usize) -> Self {
        Self { min_size, max_size }
    }
    pub fn at_least(min_size: usize) -> Self {
        Self {
            min_size,
            max_size: usize::MAX,
        }
    }
    pub fn at_most(max_size: usize) -> Self {
        Self {
            min_size: 0,
            max_size,
        }
    }
    pub fn equal_to(v: usize) -> Self {
        Self {
            min_size: v,
            max_size: v,
        }
    }
    fn check(&self, count: usize) -> Result<(), ParseError> {
        if count < self.min_size || count > self.max_size {
            return Err(ParseError::TooManyOrTooLittleValue);
        }
        Ok(())
    }
}

impl Default for CountValidator {
    fn default() -> Self {
        Self {
            max_size: 1,
            min_size: 1,
        }
    }
}

impl ArgValidator for CountValidator {
    fn validator_id(&self) -> Option<String> {
        Some(String::from("CountValidator"))
    }
    fn help(&self, nodes: &mut TerminalNodes) {
        if self.min_size == self.max_size && self.min_size != 1 {
            nodes
                .append_node(format!("Arg Count: ={}", self.min_size))
                .new_line();
        } else if self.min_size == 0 && self.max_size == 1 {
            nodes.append_node("Optional");
        } else if self.min_size == 1 && self.max_size == 1 {
            nodes.append_node("Required");
        } else if self.min_size != 1 && self.max_size == usize::MAX {
            nodes.append_node(format!("Arg Count: n >= {}", self.min_size));
        } else {
            nodes
                .append_node(format!(
                    "Arg Count: {} <= n <= {}",
                    self.min_size, self.max_size
                ))
                .new_line();
        }
    }
    fn post_validate(&self, key: Option<&ArgKey>, args: &ParsedArg) -> Result<(), ParseError> {
        if let Some(key) = key {
            return self.check(args.count(key));
        }
        self.check(1)
    }
}

#[derive(Debug, Default, Clone)]
struct EmptyValidator {}

impl ArgValidator for EmptyValidator {
    fn validator_id(&self) -> Option<String> {
        Some(String::from("EmptyValidator"))
    }
    fn help(&self, nodes: &mut TerminalNodes) {
        nodes.append_node("AllowEmpty: False");
    }
    fn validate(&self, v: Option<&String>) -> Result<(), ParseError> {
        match v {
            None => Err(ParseError::ValueRequired),
            Some(_) => Ok(()),
        }
    }
    fn post_validate(&self, _: Option<&ArgKey>, _: &ParsedArg) -> Result<(), ParseError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct Arg {
    validators: Vec<Box<dyn ArgValidator>>,
    help_text: Option<String>,
}

impl Arg {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn positional() -> Self {
        Self::default().n_equal_to(1).take()
    }
    pub fn flag() -> Self {
        Self::default().take()
    }
    pub fn add_validator<T: 'static + ArgValidator>(&mut self, v: T) -> &mut Self {
        let mut validator: Box<dyn ArgValidator> = Box::new(v);
        if let Some(id) = validator.validator_id()
            && let Some(cur_validator) = self.get_mut(&id) {
                std::mem::swap(&mut validator, cur_validator);
                return self;
            }
        self.validators.push(validator);
        self
    }
    pub fn help(&mut self, h: impl Into<String>) -> &mut Self {
        self.help_text = Some(h.into());
        self
    }
    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }

    // Builder Functions
    pub fn n_at_least(&mut self, min_size: usize) -> &mut Self {
        self.add_validator(CountValidator::at_least(min_size))
    }
    pub fn n_at_most(&mut self, max_size: usize) -> &mut Self {
        self.add_validator(CountValidator::at_most(max_size))
    }
    pub fn n_equal_to(&mut self, v: usize) -> &mut Self {
        self.add_validator(CountValidator::equal_to(v))
    }
    pub fn n_range(&mut self, min_size: usize, max_size: usize) -> &mut Self {
        self.add_validator(CountValidator::range(min_size, max_size))
    }
    pub fn not_empty(&mut self) -> &mut Self {
        self.add_validator(EmptyValidator::default())
    }
    pub fn required(&mut self) -> &mut Self {
        self.not_empty().n_equal_to(1)
    }
    pub fn optional(&mut self) -> &mut Self {
        self.n_range(0, 1)
    }

    fn get_mut(&mut self, id: &impl PartialEq<String>) -> Option<&mut Box<dyn ArgValidator>> {
        self
            .validators
            .iter_mut()
            .find(|validator| {
                if let Some(validator_id) = validator.validator_id() {
                    return id == &validator_id;
                }
                false
            })
    }
}

impl ArgValidator for Arg {
    fn help(&self, nodes: &mut TerminalNodes) {
        if let Some(h) = &self.help_text {
            nodes.append_node(h).new_line();
        }
        for validator in self.validators.iter() {
            validator.help(nodes);
            nodes.new_line();
        }
    }
    fn validate(&self, v: Option<&String>) -> Result<(), ParseError> {
        for validator in self.validators.iter() {
            validator.validate(v)?;
        }
        Ok(())
    }
    fn post_validate(&self, key: Option<&ArgKey>, args: &ParsedArg) -> Result<(), ParseError> {
        for validator in self.validators.iter() {
            validator.post_validate(key, args)?;
        }
        Ok(())
    }
}
