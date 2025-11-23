use std::{
    env,
    ffi::OsStr,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use clrust::{ActionBuilder, ActionHandler, App, AppIdentity, AppVersion, Arg, ArgEmptyValidator};

#[derive(Debug, Clone)]
struct AppState {
    app_dir: PathBuf,
    data_dir: Option<PathBuf>,
    llama_model_path: Option<PathBuf>,
    llama_exe: Option<PathBuf>,
    llama_port: i32,
    llama_context_size: usize,
    llama_gpu_layers: usize,
    interrupted: Arc<AtomicBool>,
}

impl AppState {
    fn new(app_dir: PathBuf) -> Self {
        Self {
            app_dir,
            data_dir: None,
            llama_model_path: None,
            llama_exe: None,
            llama_port: 8080,
            llama_context_size: 0,
            llama_gpu_layers: 100,
            interrupted: Arc::new(AtomicBool::new(false)),
        }
    }

    fn resolved_model_path(&self) -> PathBuf {
        self.llama_model_path
            .clone()
            .unwrap_or_else(|| self.app_dir.join("models/default.gguf"))
    }

    fn resolved_data_path(&self) -> PathBuf {
        self.data_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("/data"))
    }

    fn resolved_llama_path(&self) -> PathBuf {
        self.llama_exe
            .clone()
            .unwrap_or_else(|| self.app_dir.join("llama/bin/llama-server"))
    }
}

fn spawn_process<I, S>(args: I) -> std::io::Result<std::process::Child>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut iter = args.into_iter();
    let cmd = iter
        .next()
        .map(|s| s.as_ref().to_owned())
        .expect("Command needs at least program name");
    Command::new(cmd)
        .args(iter)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
}

struct ProcessManager {
    procs: Vec<std::process::Child>,
}

impl ProcessManager {
    fn new() -> Self {
        Self { procs: Vec::new() }
    }

    fn push(&mut self, child: std::process::Child) {
        self.procs.push(child);
    }

    fn terminate(&mut self) {
        for child in &mut self.procs {
            let _ = child.kill();
        }
        self.procs.clear();
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        self.terminate();
    }
}

fn run_stack(state: &AppState, heavy: bool) -> Result<ProcessManager, String> {
    let mut procs = ProcessManager::new();

    let data_mount = format!("{}:/data", state.resolved_data_path().to_string_lossy());

    let backend = spawn_process([
        "docker",
        "run",
        "--rm",
        "-v",
        &data_mount,
        "-p",
        "8000:8000",
        "drug-search-chat-backend",
    ])
    .map_err(|e| format!("failed to start backend: {e}"))?;
    procs.push(backend);

    let frontend = spawn_process([
        "docker",
        "run",
        "--rm",
        "-p",
        "3000:3000",
        "drug-search-chat-frontend",
    ])
    .map_err(|e| format!("failed to start frontend: {e}"))?;
    procs.push(frontend);

    if heavy {
        let llama_cmd = spawn_process([
            state.resolved_llama_path().into_os_string(),
            "--host".into(),
            "0.0.0.0".into(),
            "--port".into(),
            state.llama_port.to_string().into(),
            "-m".into(),
            state.resolved_model_path().into_os_string(),
            "--no-webui".into(),
            "--context-shift".into(),
            "--ctx_size".into(),
            state.llama_context_size.to_string().into(),
            "--jinja".into(),
            "-ngl".into(),
            state.llama_gpu_layers.to_string().into(),
        ])
        .map_err(|e| format!("failed to start llama server: {e}"))?;
        procs.push(llama_cmd);
    }

    Ok(procs)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let identity = AppIdentity::new(
        "Drug Search",
        "Command Line Interface to launch the Drug Searcher stack.",
        AppVersion::new(0, 0, 0),
    )
    .author("Calici Ltd.")
    .license("Proprietary Software Licensed to Receivers");

    let mut app = App::new(identity);
    app.add_argument(
        "--data",
        Arg::new()
            .help("Path to the data folder storing user data.")
            .validate(ArgEmptyValidator::require_value())
            .optional(),
    );

    app.parse_args(false);
    let app_dir = env::current_exe()?.parent().unwrap().to_path_buf();
    let mut state = AppState::new(app_dir);
    state.data_dir = app.args().first_of("--data").cloned().map(PathBuf::from);
    attach_sigint_handler(state.interrupted.clone())?;

    ActionBuilder::new(&mut app, Some(String::from("Choose how to run the stack")))
        .add_action(
            "heavy",
            "Run with local llama server",
            HeavyAction {
                state: state.clone(),
            },
        )
        .add_action(
            "lite",
            "Use hosted APIs only",
            LiteAction {
                state: state.clone(),
            },
        )
        .run();

    Ok(())
}

fn configure_llama(app: &mut App, state: &mut AppState) {
    app.add_argument(
        "--llama",
        Arg::new()
            .help("Path to llama executable (default: bundled).")
            .require_value()
            .optional(),
    );
    app.add_argument(
        "--model",
        Arg::new()
            .help("Path to GGUF model (default: bundled).")
            .require_value()
            .optional(),
    );
    app.add_argument(
        "--port",
        Arg::new()
            .help("Port for llama server (default: 8080).")
            .require_value()
            .optional(),
    );
    app.add_argument(
        "--offload_layers",
        Arg::new()
            .help("Layers to offload to GPU (default: all).")
            .require_value()
            .optional(),
    );
    app.add_argument(
        "--context_size",
        Arg::new()
            .help("Context size for the model (default: max).")
            .require_value()
            .optional(),
    );
    app.parse_args(true);
    state.llama_exe = app.args().first_of("--llama").map(PathBuf::from);
    state.llama_model_path = app.args().first_of("--model").map(PathBuf::from);
    state.llama_port = app
        .args()
        .first_of("--port")
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    state.llama_gpu_layers = app
        .args()
        .first_of("--offload_layers")
        .and_then(|n| n.parse().ok())
        .unwrap_or(100);
    state.llama_context_size = app
        .args()
        .first_of("--context_size")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0);
}

fn wait_for_interrupt(state: &AppState) {
    while !state.interrupted.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(200));
    }
}

fn attach_sigint_handler(flag: Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error>> {
    let handler_flag = flag.clone();
    ctrlc::set_handler(move || {
        handler_flag.store(true, Ordering::SeqCst);
    })?;
    Ok(())
}

struct HeavyAction {
    state: AppState,
}

impl ActionHandler for HeavyAction {
    fn run(&mut self, app: &mut App) {
        configure_llama(app, &mut self.state);
        match run_stack(&self.state, true) {
            Ok(mut procs) => {
                wait_for_interrupt(&self.state);
                procs.terminate();
            }
            Err(e) => app.render_err_string(e, 1),
        }
    }
}

struct LiteAction {
    state: AppState,
}

impl ActionHandler for LiteAction {
    fn run(&mut self, _app: &mut App) {
        if let Ok(mut procs) = run_stack(&self.state, false) {
            wait_for_interrupt(&self.state);
            procs.terminate();
        }
    }
}
