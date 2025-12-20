use std::{fmt::Debug, iter::Peekable};

use crate::{Arg, ArgKey, ArgValidator, ParseError, ParseErrorKind, ParsedArg};

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
    pub fn params_iter(&self) -> impl Iterator<Item = &(ArgKey, Arg)> {
        self.params.iter()
    }

    fn parse_params(
        &self,
        key: &ArgKey,
        value: Option<&str>,
        args: &mut ParsedArg,
        raw_args: &mut Peekable<std::env::Args>,
    ) -> Result<bool, ParseError> {
        for (arg_key, arg) in self.params_iter() {
            if arg_key == key {
                let parse_res = match ArgValidator::validate(arg, value) {
                    Ok(_) => Ok(value.map(String::from)),
                    Err(e) => match e.kind {
                        ParseErrorKind::NoValueGiven => {
                            raw_args.next();
                            match ArgValidator::validate(arg, raw_args.peek().map(|v| v as &str)) {
                                Ok(_) => Ok(raw_args.peek().cloned()),
                                Err(e) => Err(e),
                            }
                        }
                        _ => Err(e),
                    },
                }
                .map_err(|e| e.key(key.clone()))?;
                args.add_argument(key.clone(), parse_res.unwrap_or_default());
                raw_args.next();
                return Ok(true);
            }
        }
        Ok(false)
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
                return Err(ParseError::invalid_value(format_args!(
                    "expected args instead of kwargs"
                ))
                .key(format!("arg{}", pos_id)));
            }
            ArgValidator::validate(&self.pos, Some(current_arg))
                .map_err(|e| e.key(format!("arg{}", pos_id)))?;
            args.add_positional_argument(current_arg.clone());
            ArgValidator::post_validate(&self.pos, None, args)
                .map_err(|e| e.key(format!("arg{}", pos_id)))?;
            raw_args.next();
        }
        let mut is_parser_run = true;
        while is_parser_run && let Some(current_arg) = raw_args.peek().cloned() {
            is_parser_run = false;
            if let Ok((parsed_key, parsed_value)) = ArgKey::parse_arg(&current_arg) {
                is_parser_run = self.parse_params(&parsed_key, parsed_value, args, raw_args)?;
            }
        }
        for (arg_key, arg) in self.params.iter() {
            ArgValidator::post_validate(arg, Some(arg_key), args)
                .map_err(|e| e.key(arg_key.clone()))?;
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
        parser.add_positional_argument(Arg::new().require_value());
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
        let arg_beg_id = match args.len() {
            0 => 0,
            v => v - 1,
        };
        for i in arg_beg_id..self.len() {
            self.args[i].parse(i, args, raw_args, args.len() <= i)?
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

impl Debug for ArgParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, tier) in self.iter().enumerate() {
            writeln!(f, "arg{}", id)?;
            for (k, _) in tier.params_iter() {
                writeln!(f, "{}", k)?;
            }
        }
        Ok(())
    }
}
