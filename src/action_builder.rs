use crate::{app::App, argument::ArgOptions};

pub trait ActionProvider {
    fn run(&self, app: &mut App);
}

pub struct AppAction {
    name: String,
    action: Box<dyn ActionProvider>,
    help_text: String,
}

impl AppAction {
    fn run(&self, app: &mut App) {
        self.action.run(app);
    }
}

pub struct ActionBuilder<'a> {
    actions: Vec<AppAction>,
    app: &'a mut App,
    arg_id: usize,
}
impl<'a> ActionBuilder<'a> {
    pub fn new(app: &'a mut App, help_text: impl Into<String>) -> Self {
        app.add_positional().help(help_text.into());
        let arg_id = app.parser.len();
        return Self {
            actions: Vec::new(),
            app: app,
            arg_id: arg_id,
        };
    }
    pub fn add_action(
        &mut self,
        name: impl Into<String> + PartialEq<String>,
        help_text: impl Into<String>,
        action_fn: impl ActionProvider + 'static,
    ) -> &mut Self {
        if let Some(action) = self.actions.iter_mut().find(|action| name == action.name) {
            action.help_text = help_text.into();
            action.action = Box::new(action_fn);
        } else {
            self.actions.push(AppAction {
                name: name.into(),
                action: Box::new(action_fn),
                help_text: help_text.into(),
            })
        }
        return self;
    }
    pub fn run(&mut self) {
        self.update_args();
        self.app.advanced_parse_args(false, false);
        let pos_arg = self.app.args.current_positional();
        if self.app.args.positional_argument_size() != self.arg_id {
            self.app.log_help(Some(1));
        }
        match self.actions.iter().find(|&a| pos_arg == &a.name) {
            Some(action) => action.run(&mut self.app),
            None => self.app.log_help(Some(1)),
        }
    }

    fn update_args(&mut self) {
        let arg = self.app.parser.get_mut(self.arg_id - 1).unwrap();
        let options: ArgOptions = self
            .actions
            .iter()
            .fold(ArgOptions::new(), |mut prev, cur| {
                return prev
                    .add_option_help(cur.name.clone(), cur.help_text.clone())
                    .take();
            });
        arg.arg_mut().add_validator(options);
    }
}

#[macro_export]
macro_rules! app_action {
    ($action_name:ident, $action_body:block) => {
        struct $action_name {}
        impl clrust::ActionProvider for $action_name {
            fn run(&self, app: &mut clrust::App) {
                $action_body;
            }
        }
    };
}
