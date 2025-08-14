use crate::{
    app_identity::AppIdentity,
    arg_key::ArgKey,
    argument::{Arg, ArgValidator},
    argument_parser::ArgumentParser,
    error::ParseError,
    parsed_arg::ParsedArg,
    terminal::{Color, TerminalNode, TerminalNodes, TextEffect, TextFormat},
};
use core::fmt;

pub struct OutputFormat {
    error: TextFormat,
    help: TextFormat,
}
impl Default for OutputFormat {
    fn default() -> Self {
        return Self {
            error: TextFormat::new()
                .bg(Color::Yellow)
                .effect(TextEffect::Bold)
                .take(),
            help: TextFormat::new()
                .bg(Color::Green)
                .effect(TextEffect::Bold)
                .take(),
        };
    }
}

pub struct App {
    pub identity: AppIdentity,
    pub parser: ArgumentParser,
    pub args: ParsedArg,
    pub format: OutputFormat,
}

impl App {
    pub fn new(identity: AppIdentity) -> Self {
        let mut app = Self {
            identity: identity,
            parser: ArgumentParser::default(),
            args: ParsedArg::default(),
            format: OutputFormat::default(),
        };
        app.add_help_args();
        return app;
    }
    pub fn error_format(&mut self, f: TextFormat) -> &mut Self {
        self.format.error = f;
        return self;
    }
    pub fn help_format(&mut self, f: TextFormat) -> &mut Self {
        self.format.help = f;
        return self;
    }
    pub fn add_positional(&mut self) -> &mut Arg {
        self.parser.add_positional();
        self.add_help_args();
        return self.parser.last_mut_arg().arg_mut();
    }
    pub fn add_argument(&mut self, key: impl Into<ArgKey> + PartialEq<ArgKey>) -> &mut Arg {
        return self.parser.add_argument(key);
    }
    pub fn add_argument_unchecked(&mut self, key: impl Into<String>) -> &mut Arg {
        return self.parser.add_argument_unchecked(key);
    }
    pub fn add_help_args(&mut self) {
        self.parser
            .add_argument_unchecked("-h")
            .help("Show the help message for the application")
            .optional();
        self.parser
            .add_argument_unchecked("--help")
            .help("Show the help message for the application")
            .optional();
    }
    pub fn advanced_parse_args(&mut self, auto_help: bool, is_final: bool) {
        return match self.parser.parse_mut_args(&mut self.args) {
            Ok(args) => {
                let help_arg_count = args.count("-h") + args.count("--help");
                if auto_help && help_arg_count != 0 {
                    self.log_help(None);
                }
                if is_final && self.args.current_arg().is_some() {
                    self.log_err_and_exit(ParseError::NotEnd, Some(1));
                }
            }
            Err(e) => {
                let help_arg_count = self.args.count("-h") + self.args.count("--help");
                if auto_help && help_arg_count != 0 {
                    self.log_help(None);
                }
                self.log_err_and_exit(e, Some(1));
            }
        };
    }
    pub fn parse_args(&mut self) {
        self.advanced_parse_args(true, true);
    }
    pub fn log_help(&self, exit_code: Option<i32>) -> ! {
        let exit_code = exit_code.unwrap_or(0);
        let mut nodes = TerminalNodes::default()
            .begin_format(self.format.help.clone())
            .append_node("command: ")
            .take();
        for arg in self.args.arg_iter() {
            nodes
                .append_node(arg.arg())
                .append_node(TerminalNode::Indent(1));
        }
        nodes.new_line();
        let mut beg_id = self.args.positional_argument_size();
        for structure in self
            .parser
            .arg_iter()
            .skip(self.args.positional_argument_size() - 1)
        {
            if beg_id != self.args.positional_argument_size() {
                nodes
                    .new_line()
                    .append_node(format!("arg{}", beg_id))
                    .new_line();
                let mut sub_node = TerminalNodes::new(2);
                ArgValidator::help(structure.arg(), &mut sub_node);
                nodes.append_sub_node(sub_node);
            }
            if structure.param_len() != 0 {
                nodes.append_node("Keyword Arguments").new_line();
            }
            for (k, v) in structure.param_iter() {
                nodes.append_node(k.value()).append_node(" : ").new_line();
                let mut sub_node = TerminalNodes::new(2);
                ArgValidator::help(v, &mut sub_node);
                nodes.append_sub_node(sub_node);
            }
            beg_id += 1;
        }
        nodes.to_stdout();
        std::process::exit(exit_code);
    }

    pub fn format_err(&self, node: impl Into<TerminalNode>) -> TerminalNodes {
        return TerminalNodes::with_format(self.format.error.clone(), node, 0);
    }

    pub fn format_help(&self, node: impl Into<TerminalNode>) -> TerminalNodes {
        return TerminalNodes::with_format(self.format.help.clone(), node, 0);
    }

    pub fn log_err_and_exit(&self, e: impl fmt::Display, err_code: Option<i32>) -> ! {
        self.format_err(format!("{}", e)).to_stderr();
        std::process::exit(err_code.unwrap_or(1));
    }
}
