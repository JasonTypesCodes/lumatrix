pub const COLS: usize = 9;
pub const ROWS: usize = 34;

#[derive(Clone)]
pub struct Frame {
    pixels: [[u8; COLS]; ROWS],
}

impl Frame {
    pub fn new() -> Self {
        Self { pixels: [[0; COLS]; ROWS] }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.pixels = [[0; COLS]; ROWS];
    }

    pub fn set(&mut self, row: usize, col: usize, brightness: u8) {
        if row < ROWS && col < COLS {
            self.pixels[row][col] = brightness;
        }
    }

    pub fn fill_rect(&mut self, row: usize, col: usize, h: usize, w: usize, brightness: u8) {
        for r in row..row.saturating_add(h).min(ROWS) {
            for c in col..col.saturating_add(w).min(COLS) {
                self.pixels[r][c] = brightness;
            }
        }
    }

    /// Scale all pixel values by a global brightness factor in [0.0, 1.0].
    pub fn apply_brightness(&mut self, factor: f32) {
        for row in self.pixels.iter_mut() {
            for px in row.iter_mut() {
                *px = (*px as f32 * factor) as u8;
            }
        }
    }

    /// Return the frame as a flat row-major byte slice (306 bytes).
    pub fn as_bytes(&self) -> Vec<u8> {
        self.pixels.iter().flat_map(|row| row.iter().copied()).collect()
    }
}
