use crate::{
    arg_key::ArgKey,
    argument::{Arg, ArgValidator},
    error::{ArgParseError, ParseError},
    parsed_arg::ParsedArg,
};

pub struct ArgStructure {
    positional: Arg,
    parameters: Vec<(ArgKey, Arg)>,
}

impl ArgStructure {
    pub fn new(arg: impl Into<Arg>) -> Self {
        Self {
            positional: arg.into(),
            parameters: Vec::new(),
        }
    }
    pub fn add_argument(&mut self, k: impl Into<ArgKey> + PartialEq<ArgKey>) -> &mut Arg {
        if !self
            .parameters
            .iter()
            .any(|(stored_key, _)| &k == stored_key)
        {
            self.parameters.push((k.into(), Arg::new()));
            &mut self.parameters.last_mut().unwrap().1
        } else {
            &mut self
                .parameters
                .iter_mut()
                .find(|(stored_key, _)| &k == stored_key)
                .unwrap()
                .1
        }
    }
    pub fn add_argument_unchecked(&mut self, k: impl Into<String>) -> &mut Arg {
        self.add_argument(ArgKey::new_unchecked(k))
    }
    pub fn param_iter(&self) -> impl Iterator<Item = &(ArgKey, Arg)> {
        self.parameters.iter()
    }
    pub fn param_len(&self) -> usize {
        self.parameters.len()
    }
    pub fn arg(&self) -> &Arg {
        &self.positional
    }
    pub fn arg_mut(&mut self) -> &mut Arg {
        &mut self.positional
    }

    fn parse_param(
        &self,
        k: &ArgKey,
        arg: &Arg,
        current_value: Option<String>,
        values: &mut ParsedArg,
    ) -> Result<String, ArgParseError> {
        match arg.validate(current_value.as_ref()) {
            Err(ParseError::ValueRequired) => arg
                .validate(values.next_arg())
                .map(|_| values.current_arg().cloned()),
            r => r.map(|_| current_value),
        }
        .map(|v| {
            values.next_arg();
            v.unwrap_or(String::from(""))
        })
        .map_err(ArgParseError::or_else(k.value()))
    }

    fn parse<'a>(
        &self,
        pos_name: impl Into<String> + Clone,
        values: &'a mut ParsedArg,
        parse_positional: bool,
    ) -> Result<&'a mut ParsedArg, ArgParseError> {
        if parse_positional && let Some(current) = values.current_arg().cloned() {
            if ArgKey::is_arg_key(&current) {
                return Err(ArgParseError::new(
                    pos_name.into(),
                    ParseError::NotPositional,
                ));
            }
            // There is no retry policy for positional argument, they are guaranteed to have a value.
            self.positional
                .validate(Some(&current))
                .map_err(ArgParseError::or_else(pos_name.clone()))?;
            values.add_positional(current);
            self.positional
                .post_validate(None, values)
                .map_err(ArgParseError::or_else(pos_name.clone()))?;
            values.next_arg();
        }
        let mut is_parser_run = true;
        while is_parser_run && let Some(current) = values.current_arg().cloned() {
            is_parser_run = false;
            if let Ok((parsed_k, parsed_v)) = ArgKey::from_cmd(&current) {
                for (k, arg) in self.param_iter() {
                    if k == &parsed_k {
                        is_parser_run = true;
                        let v = self.parse_param(k, arg, parsed_v, values)?;
                        values.add_argument(k.clone(), v);
                        break;
                    }
                }
                if !is_parser_run {
                    break;
                }
            } else {
                break;
            }
        }
        for (k, arg) in self.param_iter() {
            arg.post_validate(Some(k), values)
                .map_err(ArgParseError::or_else(k.value()))?
        }
        Ok(values)
    }
}

pub struct ArgumentParser {
    args: Vec<ArgStructure>,
}

impl ArgumentParser {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_positional(&mut self) -> &mut Arg {
        self.args.push(ArgStructure {
            positional: Arg::positional(),
            parameters: Vec::new(),
        });
        &mut self.args.last_mut().unwrap().positional
    }
    pub fn add_argument(&mut self, k: impl Into<ArgKey> + PartialEq<ArgKey>) -> &mut Arg {
        self.args.last_mut().unwrap().add_argument(k)
    }
    pub fn add_argument_unchecked(&mut self, k: impl Into<String>) -> &mut Arg {
        self.args.last_mut().unwrap().add_argument_unchecked(k)
    }
    pub fn arg_iter_mut(&mut self) -> impl Iterator<Item = &mut ArgStructure> {
        self.args.iter_mut()
    }
    pub fn arg_iter(&self) -> impl Iterator<Item = &ArgStructure> {
        self.args.iter()
    }
    pub fn last_arg(&self) -> &ArgStructure {
        self.arg_iter().last().unwrap()
    }
    pub fn len(&self) -> usize {
        self.args.len()
    }
    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }
    pub fn last_mut_arg(&mut self) -> &mut ArgStructure {
        self.arg_iter_mut().last().unwrap()
    }
    pub fn parse_args(&self) -> Result<ParsedArg, ArgParseError> {
        let mut args = ParsedArg::default();
        match self.parse_mut_args(&mut args) {
            Ok(_) => Ok(args),
            Err(e) => Err(e),
        }
    }
    pub fn parse_mut_args<'a>(
        &self,
        values: &'a mut ParsedArg,
    ) -> Result<&'a mut ParsedArg, ArgParseError> {
        let arg_id = std::cmp::max(0, values.positional_argument_size() as i32 - 1) as usize;
        for i in arg_id..self.args.len() {
            self.args[i].parse(
                i.to_string(),
                values,
                values.positional_argument_size() <= i,
            )?;
        }
        Ok(values)
    }
    pub fn get(&self, id: usize) -> Option<&ArgStructure> {
        self.args.get(id)
    }
    pub fn get_mut(&mut self, id: usize) -> Option<&mut ArgStructure> {
        self.args.get_mut(id)
    }
}

impl Default for ArgumentParser {
    fn default() -> Self {
        Self {
            args: Vec::from([ArgStructure::new(Arg::positional())]),
        }
    }
}
