# Meet Clark
Your friendly neighbourhood argument parser.

Clark (`clark` on crates.io) is a batteries-included toolkit for building bespoke CLI experiences.
It sits somewhere between low-level argument parsing and full blown command frameworks, giving you a
compact API for describing positional tiers, keyword arguments, interactive actions, and even quick
terminal UIs powered by ANSI-aware widgets.

## Highlights

- **Identity-first apps** – `AppIdentity` and `AppVersion` capture your CLI metadata and Clark prints
  a styled help screen (with author/license info) automatically.
- **Composable arguments** – Build `Arg`s with validators (`ArgEmptyValidator`, `ArgCountValidator`,
  or your own `ArgValidator`) and mix positional + keyword tiers as needed.
- **Incremental parsing** – Call `App::parse_args(auto_help)` whenever you finish registering a tier;
  Clark remembers what it has already parsed and only consumes the remaining args.
- **Action builder** – `ActionBuilder` prompts the user to pick an action and dispatches to the
  matching handler, letting you script multi-step workflows declaratively.
- **Mini TUI toolkit** – The `tui` module (with helpers like the `paragraph!` macro) gives you just
  enough to draw colored text, stacks, and paragraphs when printing help or errors.

## Quick Start

Add Clark to your project (the crate name is `clark`):

```bash
cargo add clark
```

Create an `App` with metadata, describe your arguments, then parse and inspect `app.args()`:

```rust
use clark::{App, AppIdentity, AppVersion, Arg, ArgEmptyValidator};

fn main() {
    let identity = AppIdentity::new(
        "Hello World",
        "Greets the provided name or defaults to world.",
        AppVersion::new(0, 1, 0),
    );
    let mut app = App::new(identity);

    app.add_argument(
        "--name",
        Arg::new()
            .help("Name to greet")
            .validate(ArgEmptyValidator::require_value())
            .optional(),
    );

    // `true` enables the auto `-h/--help` handler.
    app.parse_args(true);

    let greeting = app
        .args()
        .first_of("--name")
        .map(|name| format!("Hello, {name}!"))
        .unwrap_or_else(|| "Hello, world!".into());

    println!("{greeting}");
}
```

Run it via:

```bash
cargo run --example hello_world -- --name Clark
```

## Validating arguments

Clark models each positional tier as a `ParamTier` made of one positional `Arg` and zero or more
keyword arguments. Every `Arg` is just a list of validators:

- `ArgEmptyValidator` toggles whether the parameter behaves like a flag or requires a value.
- `ArgCountValidator` enforces how many times a key can appear (range, equality, optional, etc.).
- `ArgOptionValidator` ensures values belong to a predefined set (great for enums).
- You can always implement `ArgValidator` yourself to add domain-specific rules and richer help
  nodes (the trait lets you emit any `tui::DomNode`).

Validators participate in both immediate validation (`validate`) and post-validation
(`post_validate`) so you can reason about the final `ParsedArg` map (e.g. “at least two `--file`”).

Clark automatically registers `-h`/`--help` for each tier, prints a formatted help screen containing
all validators + descriptions, and exits gracefully if you call `App::parse_args(true)` and the user
asks for help.

## Building multi-step actions

`ActionBuilder` is a convenience for workflows that start with a positional action choice (e.g.
`myapp heavy` vs `myapp lite`). Each action implements `ActionHandler::run` and receives the mutable
`App`, so you can continue adding arguments and parsing inside the handler:

```rust
ActionBuilder::new(&mut app, Some("Choose how to run the stack".into()))
    .add_action("heavy", "Run with local llama server", HeavyAction { state: state.clone() })
    .add_action("lite", "Use hosted APIs only", LiteAction { state })
    .run();
```

The builder injects a positional argument with auto-generated help containing the available
actions. When the user chooses one, the corresponding handler can register more arguments (see
`examples/main.rs` for a longer flow that configures docker containers and spawns processes).

## Terminal UI surfaces

The `tui` module exposes a tiny DOM-like API plus ANSI-aware renderer:

- `Layout`/`VStack` let you compose blocks, indent sections, and apply foreground/background colors
  or text effects.
- `paragraph!()` quickly formats text nodes (Clark reuses it for all help/error output).
- `App::render_err[_string]` and `App::render_out[_string]` turn any DOM tree into styled output
  without forcing you to touch escape codes directly.

You can also use the renderer separately if you want to build your own screens or progress displays.

## Examples

All examples live in `examples/` and can be executed via `cargo run --example <name> -- …`.
Highlights:

| Example | Command | Demonstrates |
|---------|---------|--------------|
| `hello_world` | `cargo run --example hello_world -- --name Ada` | Minimal optional argument + auto-help |
| `echo` | `cargo run --example echo -- --echo hello` | Required keyword arguments and identity metadata |
| `csv_reader` | `cargo run --example csv_reader -- --csv data.csv --headers` | Custom IO, boolean flags, and simple error mapping |
| `main` | `cargo run --example main -- lite` | `ActionBuilder`, multi-tier parsing, process orchestration |

Reading through these files is the quickest way to see Clark idioms in action.

## License

Clark is released under the MIT License. See `LICENSE` for the full text.
