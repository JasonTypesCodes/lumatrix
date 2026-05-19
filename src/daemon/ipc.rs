use std::{
    path::PathBuf,
    sync::{Arc, Mutex, atomic::{AtomicU8, Ordering}},
    time::Duration,
};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixListener,
};

use crate::module::{BlankModule, Module, ModuleRegistry, SweepModule, TestModule};

pub async fn run_ipc_server(
    socket_path: PathBuf,
    active_module: Arc<Mutex<Box<dyn Module>>>,
    brightness: Arc<AtomicU8>,
    registry: Arc<ModuleRegistry>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    min_interval: Duration,
) -> anyhow::Result<()> {
    std::fs::remove_file(&socket_path).ok();
    let listener = UnixListener::bind(&socket_path)
        .map_err(|e| anyhow::anyhow!("cannot bind socket {}: {}", socket_path.display(), e))?;

    loop {
        tokio::select! {
            biased;
            _ = shutdown_rx.changed() => break,
            result = listener.accept() => {
                let (stream, _) = result?;
                let active = active_module.clone();
                let brightness = brightness.clone();
                let registry = registry.clone();
                tokio::spawn(async move {
                    handle_connection(stream, active, brightness, registry, min_interval).await;
                });
            }
        }
    }

    std::fs::remove_file(&socket_path).ok();
    Ok(())
}

async fn handle_connection(
    stream: tokio::net::UnixStream,
    active_module: Arc<Mutex<Box<dyn Module>>>,
    brightness: Arc<AtomicU8>,
    registry: Arc<ModuleRegistry>,
    min_interval: Duration,
) {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
        return;
    }

    let response = dispatch_command(line.trim(), &active_module, &brightness, &registry, min_interval);
    let _ = writer.write_all(response.as_bytes()).await;
}

fn dispatch_command(
    cmd: &str,
    active_module: &Arc<Mutex<Box<dyn Module>>>,
    brightness: &Arc<AtomicU8>,
    registry: &Arc<ModuleRegistry>,
    min_interval: Duration,
) -> String {
    let parts: Vec<&str> = cmd.split(' ').filter(|s| !s.is_empty()).collect();
    match parts.first().copied().unwrap_or("") {
        "ping" => "pong\n".to_owned(),

        "clear" | "blank" => {
            *active_module.lock().unwrap() = Box::new(BlankModule::new());
            "ok\n".to_owned()
        }

        "reset" => {
            *active_module.lock().unwrap() = Box::new(SweepModule::new());
            "ok\n".to_owned()
        }

        "test" => {
            *active_module.lock().unwrap() = Box::new(TestModule::new());
            "ok\n".to_owned()
        }

        "load" => {
            let name = parts.get(1).copied().unwrap_or("").trim();
            let args: Vec<String> = parts.get(2..).unwrap_or(&[])
                .iter().map(|s| s.to_string()).collect();

            let result: Result<Box<dyn Module>, String> =
                if name.starts_with('/') || name.starts_with('.') || name.ends_with(".lua") {
                    registry.get_from_path(&std::path::PathBuf::from(name), &args)
                } else {
                    registry.get(name, &args).ok_or_else(|| format!(
                        "unknown module '{}' (available: {})",
                        name,
                        registry.names().join(", ")
                    ))
                };

            match result {
                Ok(m) => {
                    let interval = m.desired_interval();
                    *active_module.lock().unwrap() = m;
                    if interval < min_interval {
                        format!(
                            "ok\nwarning: module requests {}ms interval; capped at {}ms (hardware minimum)\n",
                            interval.as_millis(),
                            min_interval.as_millis()
                        )
                    } else {
                        "ok\n".to_owned()
                    }
                }
                Err(e) => format!("error: {}\n", e),
            }
        }

        "brightness" => match parts.get(1).copied().unwrap_or("").trim().parse::<u8>() {
            Ok(v) => {
                brightness.store(v, Ordering::Relaxed);
                "ok\n".to_owned()
            }
            Err(_) => "error: brightness must be 0-255\n".to_owned(),
        },

        "modules" => format!("{}\n", registry.names().join(", ")),

        other => format!("error: unknown command '{}'\n", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::ModuleRegistry;

    fn make_active() -> Arc<Mutex<Box<dyn Module>>> {
        Arc::new(Mutex::new(Box::new(BlankModule::new())))
    }

    fn make_registry() -> Arc<ModuleRegistry> {
        Arc::new(ModuleRegistry::new())
    }

    fn min_interval() -> Duration {
        Duration::from_millis(30)
    }

    #[test]
    fn test_ping() {
        let a = make_active();
        let b = Arc::new(AtomicU8::new(200));
        let r = make_registry();
        assert_eq!(dispatch_command("ping", &a, &b, &r, min_interval()), "pong\n");
    }

    #[test]
    fn test_load_valid() {
        let a = make_active();
        let b = Arc::new(AtomicU8::new(200));
        let r = make_registry();
        assert_eq!(dispatch_command("load pulse", &a, &b, &r, min_interval()), "ok\n");
    }

    #[test]
    fn test_load_unknown() {
        let a = make_active();
        let b = Arc::new(AtomicU8::new(200));
        let r = make_registry();
        let resp = dispatch_command("load bogus", &a, &b, &r, min_interval());
        assert!(resp.starts_with("error:"));
    }

    #[test]
    fn test_blank() {
        let a = make_active();
        let b = Arc::new(AtomicU8::new(200));
        let r = make_registry();
        assert_eq!(dispatch_command("blank", &a, &b, &r, min_interval()), "ok\n");
    }

    #[test]
    fn test_brightness() {
        let a = make_active();
        let b = Arc::new(AtomicU8::new(200));
        let r = make_registry();
        assert_eq!(dispatch_command("brightness 100", &a, &b, &r, min_interval()), "ok\n");
        assert_eq!(b.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_clear() {
        let a = make_active();
        let b = Arc::new(AtomicU8::new(200));
        let r = make_registry();
        assert_eq!(dispatch_command("clear", &a, &b, &r, min_interval()), "ok\n");
    }

    #[test]
    fn test_reset() {
        let a = make_active();
        let b = Arc::new(AtomicU8::new(200));
        let r = make_registry();
        assert_eq!(dispatch_command("reset", &a, &b, &r, min_interval()), "ok\n");
    }

    #[test]
    fn test_test_cmd() {
        let a = make_active();
        let b = Arc::new(AtomicU8::new(200));
        let r = make_registry();
        assert_eq!(dispatch_command("test", &a, &b, &r, min_interval()), "ok\n");
    }

    #[test]
    fn test_modules_lists_builtins() {
        let a = make_active();
        let b = Arc::new(AtomicU8::new(200));
        let r = make_registry();
        let resp = dispatch_command("modules", &a, &b, &r, min_interval());
        assert!(resp.contains("pulse"));
        assert!(resp.contains("rain"));
    }
}
