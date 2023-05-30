use std::{
    sync::{Arc, RwLock},
    thread::{self, sleep, JoinHandle},
    time::Duration,
};

use anyhow::Context;
use signal_hook::{
    consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM},
    iterator::{exfiltrator::SignalOnly, SignalsInfo},
};

pub type ShutdownSignal = Arc<ShutdownSignalInner>;

pub fn term_signal_hook() -> anyhow::Result<(JoinHandle<()>, ShutdownSignal)> {
    let shutdown_signal = ShutdownSignal::default();
    let signals = SignalsInfo::<SignalOnly>::new([SIGINT, SIGTERM, SIGQUIT, SIGHUP])
        .context("failed to initialize os signal stream")?;
    let handle = handle_signals(signals, shutdown_signal.clone());
    Ok((handle, shutdown_signal))
}

fn handle_signals(mut signals: SignalsInfo, shutdown_signal: ShutdownSignal) -> JoinHandle<()> {
    thread::spawn(move || {
        let signal = signals.forever().next().expect("got None signal");
        println!("\nreceived terminate signal: {signal}. shutting down...\n");
        shutdown_signal.trigger_shutdown();
        loop {
            if shutdown_signal.finished_shutdown() {
                println!("app finished, shutting down...");
                break;
            }
            sleep(Duration::from_millis(100));
        }
    })
}

#[derive(Default)]
pub struct ShutdownSignalInner {
    shutting_down: RwLock<bool>,
    finished: RwLock<bool>,
}

impl ShutdownSignalInner {
    pub fn app_should_shutdown(&self) -> bool {
        *self.shutting_down.read().unwrap()
    }

    pub fn app_finished_shutdown(&self) {
        let mut finished = self.finished.write().unwrap();
        *finished = true;
    }

    fn finished_shutdown(&self) -> bool {
        *self.finished.read().unwrap()
    }

    fn trigger_shutdown(&self) {
        let mut shutting_down = self.shutting_down.write().unwrap();
        *shutting_down = true;
    }
}

pub fn immediate_term_handle() -> anyhow::Result<JoinHandle<()>> {
    let signals = SignalsInfo::<SignalOnly>::new([SIGINT, SIGTERM, SIGQUIT, SIGHUP])
        .context("failed to initialize os signal stream")?;
    Ok(handle_signals_immediately(signals))
}

fn handle_signals_immediately(mut signals: SignalsInfo) -> JoinHandle<()> {
    thread::spawn(move || {
        let signal = signals.forever().next().expect("got None signal");
        println!("\nreceived terminate signal: {signal}. shutting down...\n");
    })
}
