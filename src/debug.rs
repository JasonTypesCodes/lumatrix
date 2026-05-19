use std::{
    io::Write,
    time::{Duration, Instant},
};

use crate::{frame, module::ModuleRegistry};

pub fn run(module_name: &str, args: &[String], max_frames: Option<u64>, registry: &ModuleRegistry) -> anyhow::Result<()> {
    let mut module = if module_name.starts_with('/') || module_name.starts_with('.') || module_name.ends_with(".lua") {
        registry.get_from_path(&std::path::PathBuf::from(module_name), args)
            .map_err(|e| anyhow::anyhow!("{}", e))?
    } else {
        registry.get(module_name, args)
            .ok_or_else(|| anyhow::anyhow!("unknown module '{}'", module_name))?
    };

    let hw_min = std::time::Duration::from_millis(30);
    let desired = module.desired_interval();
    let interval = desired.max(hw_min);

    let target_ms = interval.as_millis();
    println!("module:   {}", module_name);
    println!("interval: {}ms  ({:.1} fps target)", target_ms, 1000.0 / target_ms as f64);
    if desired < hw_min {
        println!(
            "warning: module requests {}ms interval; capped at {}ms (hardware minimum)",
            desired.as_millis(),
            hw_min.as_millis()
        );
    }
    println!("frame:    {}×{}", frame::COLS, frame::ROWS);
    println!("Ctrl+C to stop\n");

    let result = if let Some(n) = max_frames {
        run_batch(module.as_mut(), interval, n)
    } else {
        // hide cursor
        print!("\x1B[?25l");
        let r = run_loop(module.as_mut(), interval);
        // restore cursor
        print!("\x1B[?25h\n");
        std::io::stdout().flush().ok();
        r
    };

    result
}

fn run_batch(
    module: &mut dyn crate::module::Module,
    interval: Duration,
    max_frames: u64,
) -> anyhow::Result<()> {
    let mut last_tick = Instant::now();
    let stdout = std::io::stdout();

    for frame_num in 1..=max_frames {
        std::thread::sleep(interval);

        let dt = last_tick.elapsed();
        last_tick = Instant::now();

        let redraw = module.update(dt);
        let mut frame = crate::frame::Frame::new();
        if redraw {
            module.render(&mut frame);
        }

        let bytes = frame.as_bytes();
        let mut out = stdout.lock();

        writeln!(
            out,
            "frame {:5} | dt {:4}ms | fps {:4.1} | redraw: {}",
            frame_num,
            dt.as_millis(),
            1000.0 / dt.as_millis() as f64,
            redraw
        )?;

        for row in 0..frame::ROWS {
            for col in 0..frame::COLS {
                let b = bytes[row * frame::COLS + col];
                let cell = match b {
                    0        => "  ",
                    1..=50   => "░░",
                    51..=120  => "▒▒",
                    121..=200 => "▓▓",
                    _        => "██",
                };
                out.write_all(cell.as_bytes())?;
            }
            out.write_all(b"\n")?;
        }
        out.write_all(b"\n")?;
        out.flush()?;
    }

    Ok(())
}

fn run_loop(
    module: &mut dyn crate::module::Module,
    interval: Duration,
) -> anyhow::Result<()> {
    let mut last_tick = Instant::now();
    let mut frame_num: u64 = 0;
    let mut dt_samples: Vec<u64> = Vec::with_capacity(64);

    loop {
        std::thread::sleep(interval);

        let dt = last_tick.elapsed();
        last_tick = Instant::now();
        frame_num += 1;
        dt_samples.push(dt.as_millis() as u64);
        if dt_samples.len() > 60 {
            dt_samples.remove(0);
        }
        let avg_dt: f64 = dt_samples.iter().sum::<u64>() as f64 / dt_samples.len() as f64;

        let redraw = module.update(dt);
        let mut frame = crate::frame::Frame::new();
        if redraw {
            module.render(&mut frame);
        }

        // move cursor to top-left without clearing (avoids flicker)
        print!("\x1B[H");

        let bytes = frame.as_bytes();
        let stdout = std::io::stdout();
        let mut out = stdout.lock();

        // header
        writeln!(
            out,
            "frame {:5} | dt {:4}ms | avg {:5.1}ms | fps {:4.1} | redraw: {}   ",
            frame_num,
            dt.as_millis(),
            avg_dt,
            1000.0 / avg_dt,
            redraw
        )?;

        // frame — each pixel = 2 chars wide for readability
        for row in 0..frame::ROWS {
            for col in 0..frame::COLS {
                let b = bytes[row * frame::COLS + col];
                let cell = match b {
                    0       => "  ",
                    1..=50  => "░░",
                    51..=120 => "▒▒",
                    121..=200 => "▓▓",
                    _       => "██",
                };
                out.write_all(cell.as_bytes())?;
            }
            out.write_all(b"\n")?;
        }
        out.flush()?;
    }
}
