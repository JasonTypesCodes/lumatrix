use std::{path::PathBuf, sync::{Arc, Mutex}, time::Duration};

use mlua::Lua;

use crate::{
    frame::Frame,
    lua_api::FrameProxy,
};

pub trait Module: Send {
    /// The daemon calls update() no slower than this interval.
    fn desired_interval(&self) -> Duration;

    /// Called each tick with elapsed time since last call.
    /// Returns true if the display changed and should be redrawn.
    fn update(&mut self, dt: Duration) -> bool;

    /// Render current state into frame.
    fn render(&self, frame: &mut Frame);

    /// Returns true when the module has finished and should be replaced with blank.
    fn is_done(&self) -> bool { false }
}

// ---------------------------------------------------------------------------
// BlankModule
// ---------------------------------------------------------------------------

pub struct BlankModule {
    first: bool,
}

impl BlankModule {
    pub fn new() -> Self {
        Self { first: true }
    }
}

impl Module for BlankModule {
    fn desired_interval(&self) -> Duration {
        Duration::from_millis(100)
    }

    fn update(&mut self, _dt: Duration) -> bool {
        if self.first {
            self.first = false;
            true
        } else {
            false
        }
    }

    fn render(&self, frame: &mut Frame) {
        frame.clear();
    }
}

// ---------------------------------------------------------------------------
// TestModule
// ---------------------------------------------------------------------------

/// Flashes a small plus sign in each corner on/off every 500ms.
pub struct TestModule {
    on: bool,
    elapsed: Duration,
    first: bool,
}

impl TestModule {
    pub fn new() -> Self {
        Self { on: true, elapsed: Duration::ZERO, first: true }
    }
}

impl Module for TestModule {
    fn desired_interval(&self) -> Duration {
        Duration::from_millis(100)
    }

    fn update(&mut self, dt: Duration) -> bool {
        if self.first {
            self.first = false;
            return true;
        }
        self.elapsed += dt;
        if self.elapsed >= Duration::from_millis(500) {
            self.elapsed -= Duration::from_millis(500);
            self.on = !self.on;
            true
        } else {
            false
        }
    }

    fn render(&self, frame: &mut Frame) {
        if !self.on { return; }
        // 3×3 plus sign centered one cell in from each corner
        let corners: [(usize, usize); 4] = [(1, 1), (1, 7), (32, 1), (32, 7)];
        for (r, c) in corners {
            frame.set(r,     c,     255);
            frame.set(r - 1, c,     255);
            frame.set(r + 1, c,     255);
            frame.set(r,     c - 1, 255);
            frame.set(r,     c + 1, 255);
        }
    }
}

// ---------------------------------------------------------------------------
// SweepModule
// ---------------------------------------------------------------------------

const SWEEP_TRAIL: i32 = 11; // ~1/3 of 34 rows

pub struct SweepModule {
    lead: i32,
    done: bool,
}

impl SweepModule {
    pub fn new() -> Self {
        Self { lead: -1, done: false }
    }
}

impl Module for SweepModule {
    fn desired_interval(&self) -> Duration {
        Duration::from_millis(30)
    }

    fn update(&mut self, _dt: Duration) -> bool {
        if self.done { return false; }
        self.lead += 3;
        if self.lead >= crate::frame::ROWS as i32 + SWEEP_TRAIL {
            self.done = true;
            return false;
        }
        true
    }

    fn render(&self, frame: &mut Frame) {
        let rows = crate::frame::ROWS as i32;
        let cols = crate::frame::COLS;
        // trail extends upward behind the descending lead
        for i in 0..SWEEP_TRAIL {
            let row = self.lead - i;
            if row < 0 || row >= rows { continue; }
            let frac = 1.0 - (i as f32 / SWEEP_TRAIL as f32);
            let b = (255.0 * frac) as u8;
            for col in 0..cols {
                frame.set(row as usize, col, b);
            }
        }
    }

    fn is_done(&self) -> bool {
        self.done
    }
}

// ---------------------------------------------------------------------------
// LuaModule
// ---------------------------------------------------------------------------

pub struct LuaModule {
    lua: Lua,
    desired_interval: Duration,
    pending_dt: Duration,
}

impl LuaModule {
    pub fn new(
        script: &str,
        args: &[String],
    ) -> anyhow::Result<Self> {
        let lua = Lua::new();
        let args_table = lua.create_table()?;
        for (i, arg) in args.iter().enumerate() {
            args_table.set(i + 1, arg.as_str())?;
        }
        lua.globals().set("args", args_table)?;
        lua.load(script).exec()?;

        let interval_ms: u64 = lua
            .globals()
            .get::<Option<mlua::Function>>("desired_interval_ms")?
            .and_then(|f| f.call::<u64>(()).ok())
            .unwrap_or(100);

        Ok(Self {
            lua,
            desired_interval: Duration::from_millis(interval_ms),
            pending_dt: Duration::ZERO,
        })
    }
}

impl Module for LuaModule {
    fn desired_interval(&self) -> Duration {
        self.desired_interval
    }

    fn update(&mut self, dt: Duration) -> bool {
        self.pending_dt = dt;
        true
    }

    fn render(&self, frame: &mut Frame) {
        let shared = Arc::new(Mutex::new(Frame::new()));
        let proxy = FrameProxy(Arc::clone(&shared));

        let tick_result: mlua::Result<()> = (|| {
            let tick: mlua::Function = self.lua.globals().get("tick")?;
            let proxy_ud = self.lua.create_userdata(proxy)?;
            tick.call::<bool>((self.pending_dt.as_millis() as u64, proxy_ud))?;
            Ok(())
        })();

        if let Err(e) = tick_result {
            eprintln!("lua module error: {e}");
            return;
        }

        let frame_data = {
            let guard = shared.lock().unwrap();
            guard.clone()
        };
        *frame = frame_data;
    }

    fn is_done(&self) -> bool {
        self.lua.globals()
            .get::<Option<mlua::Function>>("is_done")
            .ok()
            .flatten()
            .and_then(|f| f.call::<bool>(()).ok())
            .unwrap_or(false)
    }
}

// ---------------------------------------------------------------------------
// ModuleRegistry
// ---------------------------------------------------------------------------

pub struct ModuleRegistry {
    /// Ordered search directories. Earlier entries win on name collision.
    /// Priority: user config dir > system data dir > cwd lua/ (dev).
    search_dirs: Vec<PathBuf>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self { search_dirs: build_search_dirs() }
    }

    /// Load and instantiate a module by name, searching all directories.
    pub fn get(&self, name: &str, args: &[String]) -> Option<Box<dyn Module>> {
        let filename = format!("{}.lua", name);
        for dir in &self.search_dirs {
            let path = dir.join(&filename);
            let Ok(script) = std::fs::read_to_string(&path) else { continue };
            return match LuaModule::new(&script, args) {
                Ok(m) => Some(Box::new(m)),
                Err(e) => {
                    eprintln!("error loading '{}' from {}: {}", name, path.display(), e);
                    None
                }
            };
        }
        eprintln!(
            "module '{}' not found (searched: {})",
            name,
            self.search_dirs.iter().map(|d| d.display().to_string()).collect::<Vec<_>>().join(", ")
        );
        None
    }

    /// Load a module directly from a filesystem path.
    pub fn get_from_path(&self, path: &PathBuf, args: &[String]) -> Result<Box<dyn Module>, String> {
        let script = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read '{}': {}", path.display(), e))?;
        LuaModule::new(&script, args)
            .map(|m| Box::new(m) as Box<dyn Module>)
            .map_err(|e| format!("error loading '{}': {}", path.display(), e))
    }

    /// All module names visible across all search directories (deduped, sorted).
    pub fn names(&self) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        for dir in &self.search_dirs {
            let Ok(entries) = std::fs::read_dir(dir) else { continue };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("lua") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        seen.insert(stem.to_owned());
                    }
                }
            }
        }
        let mut names: Vec<String> = seen.into_iter().collect();
        names.sort();
        names
    }
}

/// Build the ordered list of directories to search for .lua modules.
///
/// Search order (first match wins):
///   1. ~/.config/lumatrix/modules/   — user overrides
///   2. <exe>/../share/lumatrix/modules/  — installed system scripts
///   3. <cwd>/lua/                            — development source tree
fn build_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // 1. User config dir
    let config_base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/root".to_owned()))
                .join(".config")
        });
    dirs.push(config_base.join("lumatrix").join("modules"));

    // 2. System data dir relative to the installed binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin_dir) = exe.parent() {
            dirs.push(bin_dir.join("../share/lumatrix/modules"));
        }
    }

    // 3. cwd/lua/ — works when running from the source tree with `cargo run`
    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd.join("lua"));
    }

    dirs
}
