use std::{
    sync::{Arc, Mutex, atomic::{AtomicU8, Ordering}},
    time::Instant,
};

use serialport::SerialPort;

use crate::{device, frame::Frame, module::{BlankModule, Module}};

pub async fn run_render_loop(
    mut port: Box<dyn SerialPort>,
    brightness: Arc<AtomicU8>,
    active_module: Arc<Mutex<Box<dyn Module>>>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    min_interval: std::time::Duration,
) -> anyhow::Result<()> {
    let mut last_tick = Instant::now();
    let mut currently_capped = false;

    loop {
        let desired = active_module.lock().unwrap().desired_interval();
        let sleep_dur = if desired < min_interval {
            if !currently_capped {
                eprintln!(
                    "warning: module requests {}ms interval; capped at {}ms (hardware minimum)",
                    desired.as_millis(),
                    min_interval.as_millis()
                );
                currently_capped = true;
            }
            min_interval
        } else {
            currently_capped = false;
            desired
        };

        tokio::select! {
            biased;
            result = shutdown_rx.changed() => {
                let _ = result;
                device::clear_display(&mut port).ok();
                break;
            }
            _ = tokio::time::sleep(sleep_dur) => {}
        }

        let dt = last_tick.elapsed();
        last_tick = Instant::now();

        let (redraw, is_done) = {
            let mut m = active_module.lock().unwrap();
            let redraw = m.update(dt);
            (redraw, m.is_done())
        };

        if redraw {
            let mut frame = Frame::new();
            active_module.lock().unwrap().render(&mut frame);
            let factor = brightness.load(Ordering::Relaxed) as f32 / 255.0;
            frame.apply_brightness(factor);
            if let Err(e) = device::send_frame(&mut port, &frame.as_bytes()) {
                eprintln!("send error: {e}");
            }
        }

        if is_done {
            // Re-check under the lock: an IPC command may have swapped the module
            // between update() and here, in which case we must not overwrite it.
            let mut m = active_module.lock().unwrap();
            if m.is_done() {
                *m = Box::new(BlankModule::new());
            }
        }
    }

    Ok(())
}
