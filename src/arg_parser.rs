use std::iter::Peekable;

use crate::{Arg, ArgEmptyValidator, ArgKey, ArgValidator, ParseError, ParseErrorKind, ParsedArg};

pub struct ParamTier {
    pub pos: Arg,
    params: Vec<(ArgKey, Arg)>,
}

impl ParamTier {
    pub fn new(pos: Arg) -> Self {
        Self {
            pos,
            params: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &(ArgKey, Arg)> {
        self.params.iter()
    }

    pub fn parse(
        &self,
        pos_id: usize,
        args: &mut ParsedArg,
        raw_args: &mut Peekable<std::env::Args>,
        parse_positional: bool,
    ) -> Result<(), ParseError> {
        if parse_positional && let Some(current_arg) = raw_args.peek() {
            if ArgKey::is_arg_key(current_arg) {
                return Err(ParseError::invalid_value("expected args instead of kwargs")
                    .key(format!("arg{}", pos_id)));
            }
            if let Err(e) = ArgValidator::validate(&self.pos, Some(current_arg)) {
                return Err(e.key(format!("arg{}", pos_id)));
            }
            args.add_positional_argument(current_arg.clone());
            if let Err(e) = ArgValidator::post_validate(&self.pos, None, args) {
                return Err(e.key(format!("arg{}", pos_id)));
            }
            raw_args.next();
        }
        let mut is_parser_run = true;
        while is_parser_run && let Some(current_arg) = raw_args.peek().cloned() {
            is_parser_run = false;
            if let Ok((parsed_key, parsed_value)) = ArgKey::parse_arg(&current_arg) {
                for (arg_key, arg) in self.params.iter() {
                    if arg_key == &parsed_key {
                        is_parser_run = true;
                        if let Err(e) = ArgValidator::validate(arg, parsed_value) {
                            if e.kind == ParseErrorKind::NoValueGiven {
                                let next_arg = raw_args.next();
                                if let Err(e) = ArgValidator::validate(
                                    arg,
                                    next_arg.as_ref().map(|v| v as &str),
                                ) {
                                    return Err(e.key(arg_key.clone()));
                                } else {
                                    args.add_argument(
                                        arg_key.clone(),
                                        next_arg.unwrap_or_default(),
                                    );
                                }
                            } else {
                                return Err(e.key(arg_key.clone()));
                            }
                        } else {
                            args.add_argument(
                                arg_key.clone(),
                                parsed_value.map(String::from).unwrap_or_default(),
                            );
                        }
                    }
                }
                if !is_parser_run {
                    break;
                }
            }
        }
        for (arg_key, arg) in self.params.iter() {
            if let Err(e) = ArgValidator::post_validate(arg, Some(arg_key), args) {
                return Err(e.key(arg_key.clone()));
            }
        }
        Ok(())
    }
}

pub struct ArgParser {
    args: Vec<ParamTier>,
}

impl Default for ArgParser {
    fn default() -> Self {
        let mut parser = Self { args: Vec::new() };
        parser.add_positional_argument(Arg::new().validate(ArgEmptyValidator::require_value()));
        parser
    }
}

impl ArgParser {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_positional_argument(&mut self, arg: Arg) {
        self.args.push(ParamTier::new(arg));
    }

    pub fn add_argument(&mut self, k: &str, mut arg: Arg) {
        match self
            .args
            .last_mut()
            .unwrap()
            .params
            .iter_mut()
            .find(|(arg_key, _)| k == arg_key)
        {
            None => {
                self.args
                    .last_mut()
                    .unwrap()
                    .params
                    .push((ArgKey::make(k).unwrap(), arg));
            }
            Some((_, cur_arg)) => {
                std::mem::swap(cur_arg, &mut arg);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }
    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    pub fn incremental_parse(
        &self,
        args: &mut ParsedArg,
        raw_args: &mut Peekable<std::env::Args>,
    ) -> Result<(), ParseError> {
        for i in std::cmp::max(0, args.len() - 1)..self.len() {
            self.args[i].parse(i, args, raw_args, self.len() <= i)?
        }
        Ok(())
    }
    pub fn parse(&self, raw_args: &mut Peekable<std::env::Args>) -> Result<ParsedArg, ParseError> {
        let mut args = ParsedArg::new();
        self.incremental_parse(&mut args, raw_args)
            .map(move |()| args)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ParamTier> {
        self.args.iter()
    }
}
