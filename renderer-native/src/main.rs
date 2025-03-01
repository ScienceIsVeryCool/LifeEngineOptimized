// renderer-native/src/main.rs

use pixels::{Error, Pixels, SurfaceTexture};
use simulation::{Grid, CellStates};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<(), Error> {
    // Initialize the simulation grid.
    let mut grid = Grid::new(100, 100);
    
    // Create the origin of life
    grid.origin_of_life();

    // Create a window using winit.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("LifeEngine - Native")
        .with_inner_size(LogicalSize::new(800.0, 800.0))
        .build(&event_loop)
        .unwrap();

    // Create a pixel buffer that matches the grid size.
    let width = grid.width as u32;
    let height = grid.height as u32;
    let surface_texture = SurfaceTexture::new(width, height, &window);
    let mut pixels = Pixels::new(width, height, surface_texture)?;
    
    // Set simulation parameters
    grid.food_production_prob = 0.005; // 0.5% chance of food production
    grid.max_organisms = 1000;
    grid.lifespan_multiplier = 100;
    grid.insta_kill = false;

    // Run the event loop.
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(_) => {
                // Update the simulation
                grid.step();
                
                // Draw the grid
                draw_grid(pixels.get_frame(), &grid);
                
                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
        window.request_redraw();
    });
}

/// Draws the simulation grid into the frame buffer.
fn draw_grid(frame: &mut [u8], grid: &Grid) {
    // Each pixel is RGBA (4 bytes).
    for row in 0..grid.height {
        for col in 0..grid.width {
            let i = (row * grid.width + col) as usize * 4;
            let cell_color = grid.get_pixel(col, row);
            
            // Extract RGB components
            let r = ((cell_color >> 16) & 0xFF) as u8;
            let g = ((cell_color >> 8) & 0xFF) as u8;
            let b = (cell_color & 0xFF) as u8;
            
            // Set RGBA values in the frame buffer
            frame[i] = r;
            frame[i + 1] = g;
            frame[i + 2] = b;
            frame[i + 3] = 255; // Alpha always fully opaque
        }
    }
}