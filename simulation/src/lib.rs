// simulation/src/lib.rs

/// The core Grid business logic with no WASM/browser dependencies.
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height) as usize],
        }
    }

    /// Set the color of a specific pixel.
    /// Color is a 24-bit value in the form 0xRRGGBB.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.pixels[idx] = color;
        }
    }
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.pixels[idx]
        } else {
            0x000000 // return black for invalid coordinates
        }
    }

    /// Example update: shift every row to the right by one pixel, and set the first pixel to a random color.
    pub fn step(&mut self) {
        for y in 0..self.height {
            let row_start = (y * self.width) as usize;
            // Shift row to the right, iterating backwards
            for x in (1..self.width).rev() {
                let idx = row_start + x as usize;
                let prev_idx = row_start + (x - 1) as usize;
                self.pixels[idx] = self.pixels[prev_idx];
            }
            // Fill the first pixel based on row parity:
            let new_color = if y % 2 == 0 { 0xFF0000 } else { 0x0000FF };
            self.pixels[row_start] = new_color;
        }
    }
    // You can add other simulation methods here...
}
