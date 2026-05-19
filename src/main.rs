mod client;
mod daemon;
mod debug;
mod device;
mod frame;
mod lua_api;
mod module;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lumatrix", about = "Framework 16 LED matrix controller")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the LED matrix daemon
    Daemon {
        /// Serial device path for the LED matrix (auto-detected if omitted)
        #[arg(long)]
        device: Option<String>,

        /// Maximum brightness (0-255); pixel value 255 maps to this level
        #[arg(long, default_value_t = 200u8)]
        brightness: u8,

        /// Minimum interval between frames in milliseconds (default 30; hardware limit)
        #[arg(long, default_value_t = 30u64, value_parser = clap::value_parser!(u64).range(1..))]
        min_interval_ms: u64,
    },

    /// List detected LED matrix devices
    Devices,

    /// Switch to a named module or a Lua script given by full path
    Load {
        name: String,
        /// Arguments passed to the module as args[1], args[2], … in Lua
        #[arg(trailing_var_arg = true, num_args = 0..)]
        args: Vec<String>,
    },

    /// Clear the display
    Clear,

    /// Run the startup sweep animation then go blank
    Reset,

    /// Flash plus signs in each corner to verify the display is working
    Test,

    /// Set maximum display brightness (0-255)
    Brightness { value: u8 },

    /// List available modules
    Modules,

    /// Run a module locally and print timing + frame to terminal (no hardware needed)
    Debug {
        name: String,
        /// Arguments passed to the module as args[1], args[2], … in Lua
        #[arg(trailing_var_arg = true, num_args = 0..)]
        args: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Daemon { device, brightness, min_interval_ms } => {
            let device = match device {
                Some(d) => d,
                None => device::find_device()
                    .ok_or_else(|| anyhow::anyhow!("no LED matrix device found; use --device to specify one"))?,
            };
            daemon::run_daemon(device, brightness, min_interval_ms).await
        }
        Command::Devices => {
            let devices = device::list_devices();
            if devices.is_empty() {
                println!("no serial devices found");
            } else {
                for d in devices {
                    println!("{:<20} {}", d.port, d.description);
                }
            }
            Ok(())
        }
        Command::Load { name, args } => {
            let mut cmd = format!("load {}", name);
            for arg in &args { cmd.push(' '); cmd.push_str(arg); }
            client::send_command(&cmd).await
        }
        Command::Clear => client::send_command("clear").await,
        Command::Reset => client::send_command("reset").await,
        Command::Test => client::send_command("test").await,
        Command::Brightness { value } => {
            client::send_command(&format!("brightness {}", value)).await
        }
        Command::Modules => client::send_command("modules").await,
        Command::Debug { name, args } => {
            let registry = module::ModuleRegistry::new();
            debug::run(&name, &args, &registry)
        }
    }
}
