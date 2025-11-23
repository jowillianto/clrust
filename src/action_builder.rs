use crate::{App, Arg, ArgOptionValidator};
use crate::{paragraph, tui};

pub trait ActionHandler {
    fn run(&mut self, app: &mut App);
}

struct AppAction {
    name: String,
    help_text: String,
    handler: Box<dyn ActionHandler>,
}

pub struct ActionBuilder<'a> {
    app: &'a mut App,
    help_text: Option<String>,
    actions: Vec<AppAction>,
}

impl<'a> ActionBuilder<'a> {
    pub fn new(app: &'a mut App, help_text: Option<String>) -> Self {
        Self {
            app,
            help_text,
            actions: Vec::new(),
        }
    }

    pub fn add_action(
        mut self,
        name: impl Into<String>,
        help_text: impl Into<String>,
        handler: impl ActionHandler + 'static,
    ) -> Self {
        let name = name.into();
        if let Some(action) = self.actions.iter_mut().find(|action| action.name == name) {
            action.help_text = help_text.into();
            action.handler = Box::new(handler);
        } else {
            self.actions.push(AppAction {
                name,
                help_text: help_text.into(),
                handler: Box::new(handler),
            });
        }
        self
    }

    pub fn run(self) {
        if self.actions.is_empty() {
            return;
        }

        let ActionBuilder {
            app,
            help_text,
            mut actions,
        } = self;

        let mut argument = Arg::new();
        if let Some(help) = help_text {
            argument = argument.help(help);
        }
        let mut options = ArgOptionValidator::new();
        for action in &actions {
            options = options.option(action.name.clone(), Some(action.help_text.clone()));
        }
        argument = argument.validate(options).required();

        app.add_positional_argument(argument);
        let action_index = app.arg_len() - 1;

        app.parse_args(false);

        if app.args().len() <= action_index {
            app.render_err(
                &tui::VStack(
                    tui::Layout::default()
                        .append_child(paragraph!("arg{}: expected action name", action_index))
                        .style(tui::DomStyle::new().fg(tui::RgbColor::bright_yellow())),
                ),
                1,
            );
        }

        let action_name = app.args().arg().to_string();
        match actions.iter_mut().find(|action| action.name == action_name) {
            Some(action) => action.handler.run(app),
            None => {
                app.render_err(
                    &tui::VStack(
                        tui::Layout::default()
                            .append_child(paragraph!("Unknown action '{}'", action_name))
                            .style(tui::DomStyle::new().fg(tui::RgbColor::bright_yellow())),
                    ),
                    1,
                );
            }
        }
    }
}
