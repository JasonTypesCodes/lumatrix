# lumatrix

A Lua-scriptable LED matrix daemon for the Framework LED Matrix Input Module.

## Requirements

- Framework laptop with LED Matrix Input Module
- Linux
- Rust toolchain (`rustup.rs`)
- `pactl` (PulseAudio / PipeWire) for volume readout in the `system` module
- D-Bus session for desktop notification alerts in the `system` module

## Build & install

```bash
cargo build --release
cargo install --path .
```

## Daemon

Start the daemon before using any other commands:

```bash
lumatrix daemon
```

The daemon auto-detects the LED matrix. Options:

| Flag | Default | Description |
|------|---------|-------------|
| `--device <path>` | auto | Serial device, e.g. `/dev/ttyACM0`. Auto-detected if omitted. |
| `--brightness <0-255>` | `200` | Maximum brightness. A pixel value of 255 in a module maps to this level on the hardware. |
| `--min-interval-ms <ms>` | `30` | Minimum frame interval in milliseconds. 30ms is the hardware limit imposed by the LED controller. |

## Commands

All commands (except `daemon`, `devices`, and `debug`) communicate with a running daemon over a Unix socket.

### `load <name> [args…]`

Switch to a module by name. Built-in modules:

```bash
lumatrix load system       # system metrics (CPU, memory, battery, volume)
lumatrix load rain         # rain animation
lumatrix load firework     # fireworks
lumatrix load stars        # twinkling stars
lumatrix load pulse        # breathing pulse
lumatrix load hourglass    # 30-second hourglass timer
lumatrix load hourglass 60 # hourglass with a 60-second duration
```

You can also load any `.lua` file directly by path:

```bash
lumatrix load /path/to/mymodule.lua
lumatrix load ./mymodule.lua arg1 arg2
```

Any arguments after the module name are passed to the Lua script as `args[1]`, `args[2]`, etc.

### `brightness <0-255>`

Set the maximum brightness while the daemon is running:

```bash
lumatrix brightness 128    # half brightness
lumatrix brightness 255    # maximum
```

### `clear`

Turn off all LEDs immediately.

### `reset`

Run the startup sweep animation, then go blank. Useful for verifying the display is working.

### `test`

Flash a plus sign in each corner of the display, alternating on and off every 500ms. Press Ctrl+C or load another module to stop.

### `modules`

List all available module names (built-in modules plus any found in `~/.config/lumatrix/modules/`).

### `devices`

List detected LED matrix devices and their serial ports. Does not require a running daemon.

```bash
lumatrix devices
```

### `debug <name> [args…]`

Run a module in the terminal without hardware. Renders the frame as ASCII art and prints timing statistics. Accepts the same name/path/args syntax as `load`. Does not require a running daemon.

```bash
lumatrix debug rain
lumatrix debug hourglass 10
lumatrix debug /path/to/mymodule.lua
```

## Writing a module

A module is a `.lua` file. Drop it in `~/.config/lumatrix/modules/` and load it by name, or load it directly by path.

### Required function

```lua
function tick(dt_ms, frame)
    -- dt_ms: milliseconds elapsed since the last tick (integer)
    -- frame: the display object (see below)
    return true   -- true = display changed, false = skip redraw
end
```

### Optional function

```lua
function desired_interval_ms()
    return 100   -- request a tick every 100ms (default if omitted)
end
```

The daemon will call `tick` no faster than `desired_interval_ms`. The minimum enforced by the hardware is 30ms.

### Optional function: self-termination

```lua
function is_done()
    return true   -- returning true causes the daemon to unload this module
end
```

When `is_done()` returns `true` the daemon replaces the module with a blank display automatically.

### The `frame` object

| Call | Description |
|------|-------------|
| `frame:set(row, col, brightness)` | Set a single pixel. Row and col are 0-based. Brightness is 0–255. |
| `frame:fill_rect(row, col, h, w, brightness)` | Fill a rectangle. `h` = height in rows, `w` = width in columns. |
| `frame:clear()` | Set all pixels to 0. |
| `frame.ROWS` | Total rows (34). |
| `frame.COLS` | Total columns (9). |

The frame is cleared at the start of each tick. Any pixels you do not set remain off.

### The `system` global

Provides cached system metrics updated in the background:

| Call | Returns |
|------|---------|
| `system.cpu_pct()` | CPU utilization, 0.0–100.0 |
| `system.mem_pct()` | Memory utilization, 0.0–100.0 |
| `system.battery_pct()` | Battery level, 0.0–100.0 |
| `system.is_charging()` | `true` if the battery is charging |
| `system.volume_pct()` | Audio output volume, 0.0–100.0 |
| `system.disk_read_bps()` | Disk read throughput in bytes/sec |
| `system.disk_write_bps()` | Disk write throughput in bytes/sec |
| `system.net_rx_bps()` | Network receive throughput in bytes/sec |
| `system.net_tx_bps()` | Network transmit throughput in bytes/sec |
| `system.alert_remaining_ms()` | Milliseconds remaining on an active desktop notification alert, or 0 |

### The `args` global

Arguments passed on the command line after the module name are available as a 1-indexed table of strings:

```bash
lumatrix load mymodule foo 42
```

```lua
-- args[1] == "foo"
-- args[2] == "42"
local count = tonumber(args[1]) or 10
```

### Module search path

When loading by name (not by path), lumatrix searches these directories in order — first match wins:

1. `~/.config/lumatrix/modules/`
2. `<install prefix>/share/lumatrix/modules/`
3. `./lua/` (source tree, for development with `cargo run`)

## Example module

This module displays a horizontal bar that bounces back and forth across the display:

```lua
-- bouncer.lua
-- Usage: lumatrix load bouncer
--        lumatrix load bouncer 3    (bar height in rows, default 1)

local height = tonumber(args and args[1]) or 1
local row = 0
local direction = 1

function desired_interval_ms()
    return 50
end

function tick(dt_ms, frame)
    frame:fill_rect(row, 0, height, frame.COLS, 200)

    row = row + direction
    if row >= frame.ROWS - height then
        row = frame.ROWS - height
        direction = -1
    elseif row < 0 then
        row = 0
        direction = 1
    end

    return true
end
```

Save this as `~/.config/lumatrix/modules/bouncer.lua` and run:

```bash
lumatrix load bouncer
lumatrix load bouncer 3
```

## License

MIT — see [LICENSE](LICENSE).
