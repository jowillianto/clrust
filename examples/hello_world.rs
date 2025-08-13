use clrust;

#[derive(Default)]
struct VarBuilder {}

impl clrust::ActionProvider for VarBuilder {
    fn run(&self, app: &mut clrust::App) {
        app.add_parametric_unchecked("--name").required();
        app.add_help_args();
        app.parse_args();
        println!("name: {}", app.args.first_of("--name").unwrap());
    }
}

fn main() {
    let mut app = clrust::App::new(clrust::AppIdentity::new(
        "Hello World",
        "A Hello World",
        clrust::AppVersion {
            major: 0,
            minor: 0,
            patch: 0,
        },
    ));

    clrust::ActionBuilder::new(&mut app, "HEYYY")
        .add_action("hey", "HEY HEY", VarBuilder::default())
        .run();
}
