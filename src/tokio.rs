use std::{sync::Arc, time::Duration};

use anyhow::Context;
use futures::StreamExt;
use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;
use tokio::{sync::RwLock, task::JoinHandle};

pub type ShutdownSignal = Arc<ShutdownSignalInner>;

pub fn term_signal_hook() -> anyhow::Result<(JoinHandle<()>, ShutdownSignal)> {
    let shutdown_signal: Arc<_> = ShutdownSignal::default();
    let signals = Signals::new([SIGTERM, SIGINT, SIGQUIT, SIGHUP])
        .context("failed to initialize os signal stream")?;
    let handle = handle_signals(signals, shutdown_signal.clone());
    Ok((handle, shutdown_signal))
}

fn handle_signals(mut signals: Signals, shutdown_signal: ShutdownSignal) -> JoinHandle<()> {
    tokio::spawn(async move {
        let signal = signals.next().await.expect("got None signal");
        println!("\nreceived terminate signal: {signal}\n");
        shutdown_signal.trigger_shutdown().await;
        loop {
            if shutdown_signal.finished_shutdown().await {
                println!("app finished, shutting down...");
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
}

#[derive(Default)]
pub struct ShutdownSignalInner {
    shutting_down: RwLock<bool>,
    finished: RwLock<bool>,
}

impl ShutdownSignalInner {
    pub async fn app_should_shutdown(&self) -> bool {
        *self.shutting_down.read().await
    }

    pub async fn app_finished_shutdown(&self) {
        let mut finished = self.finished.write().await;
        *finished = true;
    }

    async fn trigger_shutdown(&self) {
        let mut shutting_down = self.shutting_down.write().await;
        *shutting_down = true;
    }

    async fn finished_shutdown(&self) -> bool {
        *self.finished.read().await
    }
}

pub fn immediate_term_handle() -> anyhow::Result<JoinHandle<()>> {
    let signals = Signals::new([SIGTERM, SIGINT, SIGQUIT, SIGHUP])
        .context("failed to initialize os signal stream")?;
    Ok(handle_signals_immediately(signals))
}

fn handle_signals_immediately(mut signals: Signals) -> JoinHandle<()> {
    tokio::spawn(async move {
        let signal = signals.next().await.expect("got None signal");
        println!("\nreceived terminate signal: {signal}. shutting down...\n");
    })
}
