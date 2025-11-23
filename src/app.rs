use core::fmt;
use std::iter::Peekable;

use crate::{
    AppIdentity, Arg, ArgParser, ArgValidator, ParsedArg, paragraph,
    tui::{self, DomRenderer, DomStyle, RgbColor},
};

pub struct App {
    identity: AppIdentity,
    parser: ArgParser,
    parsed: ParsedArg,
    raw_args: Peekable<std::env::Args>,
    out: tui::AnsiTerminal<std::io::Stdout>,
    err: tui::AnsiTerminal<std::io::Stderr>,
}

impl App {
    pub fn new(identity: AppIdentity) -> Self {
        let mut app = Self {
            identity,
            parser: ArgParser::new(),
            parsed: ParsedArg::new(),
            raw_args: std::env::args().peekable(),
            out: tui::AnsiTerminal::default(),
            err: tui::AnsiTerminal::default(),
        };
        app.add_help_arguments();
        app
    }

    pub fn identity(&self) -> &AppIdentity {
        &self.identity
    }

    pub fn args(&self) -> &ParsedArg {
        &self.parsed
    }

    pub fn add_argument(&mut self, key: &str, arg: Arg) {
        self.parser.add_argument(key, arg);
    }

    pub fn add_positional_argument(&mut self, arg: Arg) {
        self.parser.add_positional_argument(arg);
        self.add_help_arguments();
    }
    fn add_help_arguments(&mut self) {
        self.parser.add_argument(
            "-h",
            Arg::new()
                .help("Show the help message for the application")
                .as_flag(),
        );
        self.parser.add_argument(
            "--help",
            Arg::new()
                .help("Show the help message for the application")
                .as_flag(),
        );
    }

    pub fn arg_len(&self) -> usize {
        self.parser.len()
    }

    pub fn print_help_text(&mut self) {
        let style = DomStyle::new().fg(RgbColor::bright_green());
        let mut layout = tui::Layout::new().style(style.clone());
        layout = layout.append_child(paragraph!(
            "{} v{}",
            self.identity.name,
            self.identity.version
        ));

        if !self.identity.description.is_empty() {
            layout = layout.append_child(paragraph!("{}", &self.identity.description));
        }
        if let Some(author) = &self.identity.author {
            layout = layout.append_child(paragraph!("Written by : {}", author));
        }
        if let Some(license) = &self.identity.license {
            layout = layout.append_child(paragraph!("{}", license));
        }

        layout = layout.append_child(paragraph!(""));

        for (idx, tier) in self.parser.iter().enumerate() {
            let mut section = tui::Layout::new().style(style.clone());
            section = section.append_child(paragraph!("arg{idx}:"));

            if tier.is_empty() {
                section = section.append_child(paragraph!("  <no keyword arguments defined>"));
            } else {
                section = section.append_child(paragraph!("  Keyword Arguments:"));
                for (key, arg) in tier.params_iter() {
                    let mut entry = tui::Layout::new().style(style.clone().indent(2));
                    entry = entry.append_child(paragraph!("{}", key));
                    if let Some(node) = ArgValidator::help(arg) {
                        entry = entry.append_child(node);
                    } else {
                        entry = entry.append_child(paragraph!("<no-help>"));
                    }
                    section = section.append_child(tui::VStack(entry));
                }
            }
            layout = layout.append_child(tui::VStack(section));
            layout = layout.append_child(paragraph!(""));
        }
        let _ = self.out.render(&tui::VStack(layout));
    }

    pub fn render_err(&mut self, dom: &tui::DomNode, exit_code: i32) {
        let _ = self.err.render(dom);
        std::process::exit(exit_code);
    }

    pub fn render_err_string(&mut self, msg: impl fmt::Display, exit_code: i32) {
        self.render_err(
            &tui::VStack(
                tui::Layout::new()
                    .style(tui::DomStyle::new().fg(tui::RgbColor::bright_yellow()))
                    .append_child(paragraph!("{}", msg)),
            ),
            exit_code,
        )
    }

    pub fn render_out(&mut self, dom: &tui::DomNode) {
        let _ = self.out.render(dom);
    }

    pub fn render_out_string(&mut self, msg: impl fmt::Display) {
        self.render_out(&paragraph!("{}", msg));
    }

    pub fn parse_args(&mut self, auto_help: bool) -> &ParsedArg {
        match self
            .parser
            .incremental_parse(&mut self.parsed, &mut self.raw_args)
        {
            Ok(_) => {
                if auto_help && (self.parsed.count("-h") + self.parsed.count("--help") > 0) {
                    self.print_help_text();
                    std::process::exit(0);
                }
                &self.parsed
            }
            Err(err) => {
                let _ = self.err.render(&tui::VStack(
                    tui::Layout::default()
                        .append_child(paragraph!("{}", err.msg))
                        .style(DomStyle::new().fg(RgbColor::bright_yellow())),
                ));
                std::process::exit(1);
            }
        }
    }
}
