pub mod ipc;
pub mod render;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex, atomic::AtomicU8},
};

use crate::module::{ModuleRegistry, SweepModule};

pub fn socket_path() -> PathBuf {
    let base = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
    PathBuf::from(base).join("lumatrix.sock")
}

pub async fn run_daemon(device: String, brightness: u8, min_interval_ms: u64) -> anyhow::Result<()> {
    let port = serialport::new(&device, 115_200)
        .timeout(std::time::Duration::from_millis(1000))
        .open()
        .map_err(|e| anyhow::anyhow!("cannot open {}: {}", device, e))?;

    // Give the USB CDC device a moment to finish initializing after port open.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let brightness_atom = Arc::new(AtomicU8::new(brightness));
    let registry = Arc::new(ModuleRegistry::new());

    let active_module: Arc<Mutex<Box<dyn crate::module::Module>>> =
        Arc::new(Mutex::new(Box::new(SweepModule::new())));

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    {
        let tx = shutdown_tx.clone();
        tokio::spawn(async move {
            let mut sigint = tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::interrupt()
            ).expect("failed to register SIGINT");
            let mut sigterm = tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::terminate()
            ).expect("failed to register SIGTERM");
            tokio::select! {
                _ = sigint.recv() => {},
                _ = sigterm.recv() => {},
            }
            let _ = tx.send(true);
        });
    }

    let min_interval = std::time::Duration::from_millis(min_interval_ms);

    let path = socket_path();
    let ipc_handle = {
        let active = active_module.clone();
        let brightness_atom = brightness_atom.clone();
        let registry = registry.clone();
        let rx = shutdown_rx.clone();
        let p = path.clone();
        tokio::spawn(async move {
            if let Err(e) = ipc::run_ipc_server(p, active, brightness_atom, registry, rx, min_interval).await {
                eprintln!("IPC server error: {e}");
            }
        })
    };

    let result =
        render::run_render_loop(port, brightness_atom, active_module, shutdown_rx, min_interval).await;

    let _ = shutdown_tx.send(true);
    let _ = ipc_handle.await;

    result
}
