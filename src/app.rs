use crate::{
    app_identity::AppIdentity,
    arg_key::ArgKey,
    argument::{Arg, ArgValidator},
    argument_parser::ArgumentParser,
    parsed_arg::ParsedArg,
    terminal::{Color, TerminalNode, TerminalNodes, TextEffect, TextFormat},
};
use core::fmt;
use std::cmp::max;

pub struct OutputFormat {
    error: TextFormat,
    help: TextFormat,
}
impl Default for OutputFormat {
    fn default() -> Self {
        Self {
            error: TextFormat::new()
                .bg(Color::Yellow)
                .effect(TextEffect::Bold)
                .take(),
            help: TextFormat::new()
                .bg(Color::Green)
                .effect(TextEffect::Bold)
                .take(),
        }
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
            identity,
            parser: ArgumentParser::default(),
            args: ParsedArg::default(),
            format: OutputFormat::default(),
        };
        app.add_help_args();
        app
    }
    pub fn error_format(&mut self, f: TextFormat) -> &mut Self {
        self.format.error = f;
        self
    }
    pub fn help_format(&mut self, f: TextFormat) -> &mut Self {
        self.format.help = f;
        self
    }
    pub fn add_positional(&mut self) -> &mut Arg {
        self.parser.add_positional();
        self.add_help_args();
        self.parser.last_mut_arg().arg_mut()
    }
    pub fn add_argument(&mut self, key: impl Into<ArgKey> + PartialEq<ArgKey>) -> &mut Arg {
        self.parser.add_argument(key)
    }
    pub fn add_argument_unchecked(&mut self, key: impl Into<String>) -> &mut Arg {
        self.parser.add_argument_unchecked(key)
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
    pub fn advanced_parse_args(&mut self, auto_help: bool) {
        match self.parser.parse_mut_args(&mut self.args) {
            Ok(args) => {
                let help_arg_count = args.count("-h") + args.count("--help");
                if auto_help && help_arg_count != 0 {
                    self.log_help(None);
                }
            }
            Err(e) => {
                let help_arg_count = self.args.count("-h") + self.args.count("--help");
                if auto_help && help_arg_count != 0 {
                    self.log_help(None);
                }
                self.log_err_and_exit(e, Some(1));
            }
        }
    }
    pub fn parse_args(&mut self) {
        self.advanced_parse_args(true);
    }
    pub fn log_help(&self, exit_code: Option<i32>) -> ! {
        let exit_code = exit_code.unwrap_or(0);
        let mut nodes = TerminalNodes::default();
        // Format App Identity
        nodes
            .append_node(format!("{} v{}", self.identity.name, self.identity.version))
            .new_line();
        if !self.identity.description.is_empty() {
            nodes
                .append_node(self.identity.description.clone())
                .new_line();
        }
        if let Some(author) = &self.identity.author {
            nodes.append_node(format!("By {}", author)).new_line();
        }
        if let Some(license) = &self.identity.license {
            nodes
                .append_node(format!("License: {}", license))
                .new_line();
        }
        nodes.new_line();

        // Parsed Arguments
        for arg in self.args.arg_iter() {
            nodes
                .append_node(arg.arg())
                .append_node(TerminalNode::Indent(1));
        }
        if !self.args.is_empty() {
            nodes.new_line();
        }

        nodes.begin_format(self.format.help.clone());
        let start_id = max(0, self.args.len() as i32 - 1) as usize;
        for (cur_arg_id, structure) in self
            .parser
            .arg_iter()
            .enumerate()
            .skip(self.args.positional_argument_size() - 1)
        {
            let print_cur_pos = cur_arg_id > start_id;
            let mut sub_nodes = match print_cur_pos {
                true => TerminalNodes::new(2),
                false => TerminalNodes::new(0),
            };
            if print_cur_pos {
                nodes.append_node(format!("arg{}", cur_arg_id)).new_line();
                ArgValidator::help(structure.arg(), &mut sub_nodes);
            }
            if structure.param_len() != 0 {
                sub_nodes.append_node("Keyword Arguments: ").new_line();
            }
            for (k, v) in structure.param_iter() {
                sub_nodes.append_node(format!("{}: ", k.value())).new_line();
                let mut sub_sub_nodes: TerminalNodes = TerminalNodes::new(2);
                ArgValidator::help(v, &mut sub_sub_nodes);
                sub_nodes.append_sub_node(sub_sub_nodes);
            }
            nodes.append_sub_node(sub_nodes);
        }
        nodes.end_format();
        nodes.to_stdout();
        std::process::exit(exit_code);
    }

    pub fn format_err(&self, node: impl Into<TerminalNode>) -> TerminalNodes {
        TerminalNodes::with_format(self.format.error.clone(), node, 0)
    }

    pub fn format_help(&self, node: impl Into<TerminalNode>) -> TerminalNodes {
        TerminalNodes::with_format(self.format.help.clone(), node, 0)
    }

    pub fn log_err_and_exit(&self, e: impl fmt::Display, err_code: Option<i32>) -> ! {
        self.format_err(format!("{}", e)).to_stderr();
        std::process::exit(err_code.unwrap_or(1));
    }
}
