use std::collections::BTreeMap;
use std::process;

use crate::{App, Arg, ArgOptionValidator};

struct AppAction {
    help_text: String,
    callback: Box<dyn Fn(&mut App) + 'static>,
}

pub struct BuildAction<'a> {
    app: &'a mut App,
    help_text: Option<String>,
    actions: BTreeMap<String, AppAction>,
}

impl<'a> BuildAction<'a> {
    pub fn new(app: &'a mut App, help_text: Option<String>) -> Self {
        Self {
            app,
            help_text,
            actions: BTreeMap::new(),
        }
    }

    pub fn add_action<F>(
        &mut self,
        name: impl Into<String>,
        help_text: impl Into<String>,
        f: F,
    ) -> &mut Self
    where
        F: Fn(&mut App) + 'static,
    {
        let name = name.into();
        self.actions.insert(
            name,
            AppAction {
                help_text: help_text.into(),
                callback: Box::new(f),
            },
        );
        self
    }

    pub fn run(self) {
        if self.actions.is_empty() {
            return;
        }

        let BuildAction {
            app,
            help_text,
            actions,
        } = self;

        let mut argument = Arg::new();
        if let Some(help) = help_text {
            argument = argument.help(help);
        }
        let mut options = ArgOptionValidator::new();
        for (name, action) in &actions {
            options = options.option(name.clone(), Some(action.help_text.clone()));
        }
        argument = argument.validate(options).required();

        app.add_positional_argument(argument);
        let action_index = app.arg_len() - 1;

        if app.parse_args(false, true).is_err() {
            return;
        }

        let parsed = app.args();
        if parsed.contains("-h") || parsed.contains("--help") {
            app.print_help_text();
            process::exit(0);
        }

        if parsed.len() <= action_index {
            eprintln!("arg{}: expected action name", action_index);
            process::exit(1);
        }

        let action_name = parsed.arg().to_string();
        match actions.get(&action_name) {
            Some(action) => (action.callback)(app),
            None => {
                eprintln!("Unknown action '{}'", action_name);
                process::exit(1);
            }
        }
    }
}
