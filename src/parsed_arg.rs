use crate::arg_key::ArgKey;
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub struct PositionalParsedArgs {
    value: String,
    parameters: Vec<(ArgKey, String)>,
}

impl PositionalParsedArgs {
    pub fn new(value: impl Into<String>) -> Self {
        return Self {
            value: value.into(),
            parameters: Vec::new(),
        };
    }
    pub fn add_parametric(
        &mut self,
        key: impl Into<ArgKey>,
        value: impl Into<String>,
    ) -> &mut Self {
        self.parameters.push((key.into(), value.into()));
        return self;
    }
    pub fn first_of(&self, key: &(impl PartialEq<ArgKey> + ?Sized)) -> Option<&(ArgKey, String)> {
        return self.parameters.iter().find(|&arg| key == &arg.0);
    }
    pub fn filter<'a, 'b>(
        &'a self,
        key: &'b (impl PartialEq<ArgKey> + ?Sized),
    ) -> impl Iterator<Item = &'a (ArgKey, String)> {
        return self.parameters.iter().filter(move |&arg| key == &arg.0);
    }
    pub fn count(&self, key: &(impl PartialEq<ArgKey> + ?Sized)) -> usize {
        return self.filter(key).count();
    }
    pub fn contains(&self, key: &(impl PartialEq<ArgKey> + ?Sized)) -> bool {
        return self.first_of(key).is_some();
    }
    pub fn arg(&self) -> &String {
        return &self.value;
    }
    pub fn len(&self) -> usize {
        return self.parameters.len();
    }
    pub fn param_iter(&self) -> impl Iterator<Item = &(ArgKey, String)> {
        return self.parameters.iter();
    }
}

#[derive(Debug)]
pub struct ArgIter {
    it: Peekable<std::env::Args>,
}

impl ArgIter {
    pub fn new() -> Self {
        return Self::default();
    }
    pub fn arg(&mut self) -> Option<&String> {
        return self.it.peek();
    }
    pub fn next(&mut self) -> Option<&String> {
        self.it.next();
        return self.arg();
    }
}

impl Default for ArgIter {
    fn default() -> Self {
        return Self {
            it: std::env::args().peekable(),
        };
    }
}

#[derive(Debug, Default)]
pub struct ParsedArg {
    args: Vec<PositionalParsedArgs>,
    it: ArgIter,
}

impl ParsedArg {
    pub fn current_positional(&self) -> &String {
        return &self.args.last().unwrap().value;
    }
    pub fn first_of(&self, key: &(impl PartialEq<ArgKey> + ?Sized)) -> Option<&String> {
        return match self.args.last().unwrap().first_of(key) {
            Some(arg) => Some(&arg.1),
            None => None,
        };
    }
    pub fn filter<'a>(
        &'a self,
        key: &(impl PartialEq<ArgKey> + ?Sized),
    ) -> impl Iterator<Item = &'a String> {
        return self.args.last().unwrap().filter(key).map(|arg| &arg.1);
    }
    pub fn count(&self, key: &(impl PartialEq<ArgKey> + ?Sized)) -> usize {
        return self.args.last().unwrap().count(key);
    }
    pub fn contains(&self, key: &(impl PartialEq<ArgKey> + ?Sized)) -> bool {
        return self.args.last().unwrap().contains(key);
    }
    pub fn positional_argument_size(&self) -> usize {
        return self.args.len();
    }
    pub fn parametric_argument_size(&self) -> usize {
        return self.args.last().unwrap().parameters.len();
    }
    pub fn parametric_iter(&self) -> impl Iterator<Item = &(ArgKey, String)> {
        return self.args.last().unwrap().parameters.iter();
    }
    pub fn arg_iter(&self) -> impl Iterator<Item = &PositionalParsedArgs> {
        return self.args.iter();
    }

    // For use with parsing
    pub fn add_positional(&mut self, value: impl Into<String>) -> &mut Self {
        self.args.push(PositionalParsedArgs::new(value));
        return self;
    }
    pub fn add_parametric(
        &mut self,
        key: impl Into<ArgKey>,
        value: impl Into<String>,
    ) -> &mut Self {
        self.args.last_mut().unwrap().add_parametric(key, value);
        return self;
    }

    // Iterator
    pub fn current_arg(&mut self) -> Option<&String> {
        return self.it.arg();
    }
    pub fn next(&mut self) -> Option<&String> {
        return self.it.next();
    }
}
