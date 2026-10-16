//! Background device worker that applies lighting changes off the UI thread.
//!
//! The worker owns a single HID handle and coalesces bursts of changes, such
//! as slider drags, into one write.

use crate::backend::HidBackend;
use crate::device::{self, ConnectedDeviceInfo};
use crate::protocol::LightingConfig;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

/// How often a missing keyboard is looked for again.
const RECONNECT_INTERVAL: Duration = Duration::from_secs(2);

/// Work requested by the app. `seq` lets the app tell stale reports from fresh ones.
#[derive(Debug, Clone)]
pub enum Command {
    /// Apply an onboard lighting effect.
    Apply { seq: u64, config: LightingConfig },
    /// Upload per-key colors and switch to Custom mode.
    ApplyPerKey {
        seq: u64,
        colors: Vec<u8>,
        brightness: u8,
    },
}

impl Command {
    fn seq(&self) -> u64 {
        match self {
            Self::Apply { seq, .. } | Self::ApplyPerKey { seq, .. } => *seq,
        }
    }
}

/// Reports sent back to the app.
#[derive(Debug, Clone)]
pub enum Event {
    Connected(ConnectedDeviceInfo),
    Disconnected(String),
    /// A command finished; `config` is the state read back from the keyboard.
    Applied {
        seq: u64,
        config: LightingConfig,
    },
    Error(String),
}

/// Handle to the worker thread. Dropping it stops the thread.
pub struct DeviceWorker {
    commands: Sender<Command>,
    events: Receiver<Event>,
}

impl DeviceWorker {
    /// Starts the worker. `wake` is called after every event so a GUI can repaint.
    pub fn spawn(wake: impl Fn() + Send + 'static) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (evt_tx, evt_rx) = mpsc::channel();
        thread::Builder::new()
            .name("kreoctl-device".into())
            .spawn(move || Worker::new(evt_tx, wake).run(cmd_rx))
            .expect("failed to spawn device worker thread");
        Self {
            commands: cmd_tx,
            events: evt_rx,
        }
    }

    pub fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }

    pub fn try_recv(&self) -> Option<Event> {
        self.events.try_recv().ok()
    }
}

struct Worker<W: Fn()> {
    events: Sender<Event>,
    wake: W,
    backend: Option<HidBackend>,
    reported_missing: bool,
}

impl<W: Fn()> Worker<W> {
    fn new(events: Sender<Event>, wake: W) -> Self {
        Self {
            events,
            wake,
            backend: None,
            reported_missing: false,
        }
    }

    fn emit(&self, event: Event) {
        let _ = self.events.send(event);
        (self.wake)();
    }

    fn run(mut self, commands: Receiver<Command>) {
        let mut next_connect = Instant::now();
        loop {
            if self.backend.is_none() && Instant::now() >= next_connect {
                self.connect();
                next_connect = Instant::now() + RECONNECT_INTERVAL;
            }

            match commands.recv_timeout(RECONNECT_INTERVAL) {
                Ok(first) => {
                    // Collapse everything queued behind it: only the newest state matters.
                    let mut latest = first;
                    while let Ok(next) = commands.try_recv() {
                        latest = next;
                    }
                    self.execute(latest);
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        if let Some(backend) = self.backend.as_mut() {
            let _ = device::end_direct_mode(backend);
        }
    }

    fn connect(&mut self) {
        match device::open_device_with_info() {
            Ok((backend, info)) => {
                self.backend = Some(backend);
                self.reported_missing = false;
                self.emit(Event::Connected(info));
            }
            Err(e) => {
                if !self.reported_missing {
                    self.reported_missing = true;
                    self.emit(Event::Disconnected(e.to_string()));
                }
            }
        }
    }

    fn drop_backend(&mut self, reason: String) {
        self.backend = None;
        self.reported_missing = true;
        self.emit(Event::Disconnected(reason));
    }

    fn execute(&mut self, command: Command) {
        let seq = command.seq();
        let Some(backend) = self.backend.as_mut() else {
            self.emit(Event::Error("Keyboard not connected".into()));
            return;
        };

        let result = match &command {
            Command::Apply { config, .. } => device::set_lighting_with_backend(backend, config),
            Command::ApplyPerKey {
                colors, brightness, ..
            } => device::set_per_key_colors_with_backend(backend, colors, *brightness),
        };

        if let Err(e) = result {
            self.drop_backend(format!("Write failed: {e}"));
            return;
        }

        match device::read_current_lighting_with_backend(backend) {
            Ok(config) => self.emit(Event::Applied { seq, config }),
            Err(e) => self.drop_backend(format!("Read-back failed: {e}")),
        }
    }
}
