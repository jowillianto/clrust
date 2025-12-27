use std::{
    sync::{Mutex, atomic::AtomicBool},
    thread::{self, JoinHandle, yield_now},
};

use super::prelude::{Emitter, Error};

#[derive(Default)]
pub struct StdoutEmitter;
impl Emitter for StdoutEmitter {
    fn emit(&self, v: String) -> Result<(), Error> {
        print!("{}", v);
        Ok(())
    }
}

#[derive(Default)]
pub struct EmptyEmitter;
impl Emitter for EmptyEmitter {
    fn emit(&self, _: String) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Default)]
pub struct StderrEmitter;
impl Emitter for StderrEmitter {
    fn emit(&self, v: String) -> Result<(), Error> {
        eprint!("{}", v);
        Ok(())
    }
}

pub struct FileEmitter<W: std::io::Write> {
    file: Mutex<W>,
}

impl FileEmitter<std::fs::File> {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        let file = std::fs::File::create(path)?;
        Ok(Self {
            file: Mutex::new(file),
        })
    }
}
unsafe impl<W: std::io::Write> Sync for FileEmitter<W> {}
unsafe impl<W: std::io::Write> Send for FileEmitter<W> {}

impl<W: std::io::Write> Emitter for FileEmitter<W> {
    fn emit(&self, v: String) -> Result<(), Error> {
        let mut guard = match self.file.lock() {
            Ok(v) => v,
            Err(e) => e.into_inner(),
        };
        guard.write_all(v.as_bytes())?;
        Ok(())
    }
}

/* Converts any emitter such that now they will log to a queue before emitting out */
pub struct ThreadedEmitter {
    sender: std::sync::mpsc::Sender<String>,
    thread: Option<JoinHandle<()>>,
    is_running: AtomicBool,
}

impl ThreadedEmitter {
    pub fn new(emitter: impl 'static + Emitter) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<String>();
        let is_running = AtomicBool::new(true);
        let is_running_ptr = is_running.as_ptr();
        let is_running_ref = unsafe { AtomicBool::from_ptr(is_running_ptr) };
        let handle = thread::spawn(move || {
            loop {
                match receiver.try_recv() {
                    Ok(msg) => {
                        if let Err(e) = emitter.emit(msg) {
                            eprintln!("{}", e);
                        }
                    }
                    Err(_) => match is_running_ref.load(std::sync::atomic::Ordering::Acquire) {
                        true => {
                            yield_now();
                            continue;
                        }
                        false => break,
                    },
                }
            }
        });
        Self {
            sender,
            thread: Some(handle),
            is_running,
        }
    }
}

impl Drop for ThreadedEmitter {
    fn drop(&mut self) {
        self.is_running
            .store(false, std::sync::atomic::Ordering::Release);
        if let Some(handle) = self.thread.take() {
            handle.join().unwrap();
        }
    }
}

unsafe impl Send for ThreadedEmitter {}
unsafe impl Sync for ThreadedEmitter {}

impl Emitter for ThreadedEmitter {
    fn emit(&self, v: String) -> Result<(), Error> {
        self.sender
            .send(v)
            .map_err(|e| Error::io_error(format_args!("{}", e)))
    }
}
