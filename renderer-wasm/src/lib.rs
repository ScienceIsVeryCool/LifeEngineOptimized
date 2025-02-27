use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, window};

#[wasm_bindgen]
pub struct Grid {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl Grid {
    /// Creates a new Grid associated with a canvas element by its ID.
    /// `width` and `height` represent the grid dimensions (number of pixels).
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32, canvas_id: &str) -> Grid {
        let window = window().expect("global window does not exist");
        let document = window.document().expect("should have a document on window");
        let canvas = document
            .get_element_by_id(canvas_id)
            .expect("Canvas element not found")
            .dyn_into::<HtmlCanvasElement>()
            .expect("Failed to cast element to canvas");
        
        // Optionally, set the canvas size (this example uses grid dimensions as canvas size).
        canvas.set_width(width);
        canvas.set_height(height);
        
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        
        let pixels = vec![0; (width * height) as usize]; // initialize with black (0x000000)

        Grid { width, height, pixels, canvas, context }
    }
    
    /// Set the color of a specific pixel in the grid.
    /// Color is a 24-bit value in the form 0xRRGGBB.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        self.pixels[idx] = color;
    }
    
    /// Renders the grid on the canvas.
    /// `pixel_size` controls how large each pixel appears on the canvas.
    pub fn render(&self, pixel_size: u32) {
        // Clear the canvas
        self.context.clear_rect(
            0.0, 
            0.0, 
            self.canvas.width() as f64, 
            self.canvas.height() as f64
        );
        
        // Draw each pixel
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                let color = self.pixels[idx];
                let red = ((color >> 16) & 0xFF) as u8;
                let green = ((color >> 8) & 0xFF) as u8;
                let blue = (color & 0xFF) as u8;
                let color_str = format!("rgb({}, {}, {})", red, green, blue);
                
                self.context.set_fill_style(&color_str.into());
                self.context.fill_rect(
                    (x * pixel_size) as f64, 
                    (y * pixel_size) as f64, 
                    pixel_size as f64, 
                    pixel_size as f64
                );
            }
        }
    }
}
