// wasm-renderer/src/lib.rs

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
use simulation::Grid as CoreGrid; // Import your core Grid type

#[wasm_bindgen]
pub struct WasmGrid {
    inner: CoreGrid,
}

#[wasm_bindgen]
impl WasmGrid {
    /// Creates a new WasmGrid with the given dimensions.
    /// Note that this does NOT handle any canvas rendering.
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> WasmGrid {
        WasmGrid {
            inner: CoreGrid::new(width, height),
        }
    }

    /// Sets a pixel in the grid.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        self.inner.set_pixel(x, y, color);
    }

    /// Returns the grid width.
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    /// Returns the grid height.
    pub fn height(&self) -> u32 {
        self.inner.height
    }
     
    /// Retrieves the color value at position (x, y).
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.inner.width && y < self.inner.height {
            let idx = (y * self.inner.width + x) as usize;
            self.inner.pixels[idx]
        } else {
            0 // default color (black) if out-of-bounds
        }
    }
    
    /// Update the grid simulation.
    pub fn step(&mut self) {
        self.inner.step();
    }
}

#[wasm_bindgen]
pub struct Renderer {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    pixel_size: u32,
}

#[wasm_bindgen]
impl Renderer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str, pixel_size: u32) -> Renderer {
        let window = window().expect("global window does not exist");
        let document = window.document().expect("should have a document on window");
        let canvas = document
            .get_element_by_id(canvas_id)
            .expect("Canvas element not found")
            .dyn_into::<HtmlCanvasElement>()
            .expect("Failed to cast element to canvas");

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        Renderer {
            canvas,
            context,
            pixel_size,
        }
    }

    /// Render the grid by drawing each pixel on the canvas.
    pub fn render(&self, grid: &WasmGrid) {
        // Resize canvas based on grid dimensions
        self.canvas.set_width(grid.width() * self.pixel_size);
        self.canvas.set_height(grid.height() * self.pixel_size);

        // Clear canvas
        self.context.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );

        // Draw each pixel from grid.inner.pixels
        for y in 0..grid.width() {
            for x in 0..grid.height() {
                // Calculate index (ensure you use the correct logic based on your grid design)
                let idx = (y * grid.width() + x) as usize;
                // Here you would normally extract the pixel from the inner grid.
                // For illustration, assume you have a method to get a pixel:
                let color = grid.inner.get_pixel(x, y);
                // Since our WasmGrid wrapper doesn’t expose pixel data, you may want to add a method to do so.
                // For now, assume a dummy color:
                let red = ((color >> 16) & 0xFF) as u8;
                let green = ((color >> 8) & 0xFF) as u8;
                let blue = (color & 0xFF) as u8;
                let color_str = format!("rgb({}, {}, {})", red, green, blue);
                self.context.set_fill_style(&color_str.into());
                self.context.fill_rect(
                    (x * self.pixel_size) as f64,
                    (y * self.pixel_size) as f64,
                    self.pixel_size as f64,
                    self.pixel_size as f64,
                );
            }
        }
    }
}

/// Starts an animation loop that updates the grid and re-renders it.
#[wasm_bindgen]
pub fn start_animation(renderer: Renderer, grid: WasmGrid) {
    // Wrap grid and renderer in Rc<RefCell<>> so the closure can capture mutable state.
    let grid_rc = Rc::new(RefCell::new(grid));
    let renderer_rc = Rc::new(renderer);

    // Create a recursive closure using Rc<RefCell<Option<Closure<dyn FnMut()>>>>.
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    // Create the closure, cloning g inside so we don't move the outer g.
    *f.borrow_mut() = Some(Closure::wrap(Box::new({
        let g = g.clone(); // clone g for use inside the closure
        move || {
            // Update simulation state.
            grid_rc.borrow_mut().step();
            // Render the updated grid.
            renderer_rc.render(&grid_rc.borrow());

            // Schedule the next frame.
            window()
                .unwrap()
                .request_animation_frame(
                    g.borrow().as_ref().unwrap().as_ref().unchecked_ref()
                )
                .expect("should register requestAnimationFrame OK");
        }
    }) as Box<dyn FnMut()>));

    // Start the animation loop.
    window()
        .unwrap()
        .request_animation_frame(
            g.borrow().as_ref().unwrap().as_ref().unchecked_ref()
        )
        .expect("should register requestAnimationFrame OK");
}
