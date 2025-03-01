// renderer-wasm/src/lib.rs
// At the top of renderer-wasm/src/lib.rs
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
use simulation::{Grid as CoreGrid, CellState, Organism};
use std::cell::RefCell;
use std::rc::Rc;

mod utils;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    utils::set_panic_hook();
    Ok(())
}

#[wasm_bindgen]
pub struct WasmGrid {
    inner: CoreGrid,
}

#[wasm_bindgen]
impl WasmGrid {
    /// Creates a new WasmGrid with the given dimensions.
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

    /// Sets a cell with a specific state
    pub fn set_cell(&mut self, x: u32, y: u32, state_idx: u8) {
        let state = match state_idx {
            0 => CellState::Empty,
            1 => CellState::Food,
            2 => CellState::Wall,
            3 => CellState::Mouth,
            4 => CellState::Producer,
            5 => CellState::Mover,
            6 => CellState::Killer,
            7 => CellState::Armor,
            8 => CellState::Eye,
            _ => CellState::Empty,
        };
        self.inner.set_cell(x, y, state, None);
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
        self.inner.get_pixel(x, y)
    }
    
    /// Update the grid simulation.
    pub fn step(&mut self) {
        self.inner.step();
    }
    
    /// Reset the grid
    pub fn reset(&mut self, clear_walls: bool) {
        self.inner.reset(clear_walls);
    }
    
    /// Get the number of organisms
    pub fn organism_count(&self) -> usize {
        self.inner.organisms.len()
    }
    
    /// Set the food production probability
    pub fn set_food_production_rate(&mut self, rate: f32) {
        self.inner.food_production_prob = rate;
    }
    
    /// Set the maximum number of organisms
    pub fn set_max_organisms(&mut self, max: usize) {
        self.inner.max_organisms = max;
    }
    
    /// Set the lifespan multiplier
    pub fn set_lifespan_multiplier(&mut self, multiplier: u32) {
        self.inner.lifespan_multiplier = multiplier;
    }
    
    /// Set whether organisms die instantly when hit by a killer
    pub fn set_insta_kill(&mut self, insta_kill: bool) {
        self.inner.insta_kill = insta_kill;
    }
    
    /// Add a simple organism at the specified position
    #[wasm_bindgen]
    pub fn add_organism(&mut self, x: u32, y: u32) -> bool {
        self.inner.create_basic_organism(x, y)
    }
    
    /// Add a custom organism
    #[wasm_bindgen]
    pub fn add_custom_organism(&mut self, x: u32, y: u32, organism_type: u8) -> bool {
        let mut organism = Organism::new(self.inner.next_organism_id, x, y);
        
        match organism_type {
            // Basic producer
            0 => {
                organism.add_cell(CellState::Mouth, 0, 0);
                organism.add_cell(CellState::Producer, 1, 0);
                organism.add_cell(CellState::Producer, -1, 0);
                organism.add_cell(CellState::Producer, 0, 1);
                organism.add_cell(CellState::Producer, 0, -1);
            },
            // Mobile hunter
            1 => {
                organism.add_cell(CellState::Mouth, 0, 0);
                organism.add_cell(CellState::Mover, 1, 0);
                organism.add_cell(CellState::Killer, 0, 1);
                organism.add_cell(CellState::Eye, -1, 0);
            },
            // Armored producer
            2 => {
                organism.add_cell(CellState::Mouth, 0, 0);
                organism.add_cell(CellState::Producer, 1, 0);
                organism.add_cell(CellState::Producer, -1, 0);
                organism.add_cell(CellState::Armor, 0, 1);
                organism.add_cell(CellState::Armor, 0, -1);
            },
            // Default to basic producer
            _ => {
                organism.add_cell(CellState::Mouth, 0, 0);
                organism.add_cell(CellState::Producer, 1, 0);
                organism.add_cell(CellState::Producer, -1, 0);
            }
        }
        
        self.inner.add_organism(organism)
    }
    
    /// Create the "Origin of Life" organism in the center
    #[wasm_bindgen]
    pub fn origin_of_life(&mut self) {
        self.inner.origin_of_life();
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

        // Draw each pixel from grid
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let color = grid.get_pixel(x, y);
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