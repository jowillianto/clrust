use std::iter::Peekable;

use crate::{
    AppIdentity, Arg, ArgEmptyValidator, ArgParser, ArgValidator, ParseError, ParsedArg,
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
    }
    fn add_help_arguments(&mut self) {
        let help_arg = Arg::new()
            .help("Show the help message for the application")
            .validate(ArgEmptyValidator::allow());
        self.parser.add_argument("-h", help_arg);
        let long_help_arg = Arg::new()
            .help("Show the help message for the application")
            .validate(ArgEmptyValidator::allow());
        self.parser.add_argument("--help", long_help_arg);
    }

    pub fn arg_len(&self) -> usize {
        self.parser.len()
    }

    pub fn print_help_text(&mut self) {
        let mut layout = tui::Layout::new();
        layout = layout.append_child(tui::Paragraph(format!(
            "{} v{}",
            self.identity.name, self.identity.version
        )));

        if !self.identity.description.is_empty() {
            layout = layout.append_child(tui::Paragraph(self.identity.description.clone()));
        }
        if let Some(author) = &self.identity.author {
            layout = layout.append_child(tui::Paragraph(format!("Written by : {author}")));
        }
        if let Some(license) = &self.identity.license {
            layout = layout.append_child(tui::Paragraph(license.to_owned()));
        }

        layout = layout.append_child(tui::Paragraph(String::new()));

        for (idx, tier) in self.parser.iter().enumerate() {
            let mut section = tui::Layout::new();
            section = section.append_child(tui::Paragraph(format!("arg{idx}:")));

            if tier.is_empty() {
                section =
                    section.append_child(tui::Paragraph("  <no keyword arguments defined>".into()));
            } else {
                section =
                    section.append_child(tui::Paragraph(String::from("  Keyword Arguments:")));
                for (key, arg) in tier.iter() {
                    let mut entry = tui::Layout::new().style(DomStyle::default().indent(4));
                    entry = entry.append_child(tui::Paragraph(format!("{}", key)));
                    if let Some(node) = ArgValidator::help(arg) {
                        entry = entry.append_child(node);
                    } else {
                        entry = entry.append_child(tui::Paragraph(String::from("<no help>")));
                    }
                    section = section.append_child(tui::VStack(entry));
                }
            }
            layout = layout.append_child(tui::VStack(section));
            layout = layout.append_child(tui::Paragraph(String::new()));
        }
        let _ = self.out.render(&tui::VStack(layout));
    }

    pub fn parse_args(
        &mut self,
        auto_help: bool,
        auto_exit: bool,
    ) -> Result<&ParsedArg, ParseError> {
        match self
            .parser
            .incremental_parse(&mut self.parsed, &mut self.raw_args)
        {
            Ok(_) => {
                if auto_help && (self.parsed.count("-h") + self.parsed.count("--help") > 0) {
                    self.print_help_text();
                    if auto_exit {
                        std::process::exit(0);
                    }
                }
                Ok(&self.parsed)
            }
            Err(err) => {
                if auto_exit {
                    let _ = self.err.render(&tui::VStack(
                        tui::Layout::default()
                            .append_child(tui::Paragraph(err.msg))
                            .style(DomStyle::new().fg(RgbColor::bright_yellow())),
                    ));
                    std::process::exit(1);
                }
                Err(err)
            }
        }
    }
}
