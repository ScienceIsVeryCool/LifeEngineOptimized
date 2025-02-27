// renderer-native/src/main.rs

use pixels::{Error, Pixels, SurfaceTexture};
use simulation::{Grid, CellState};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<(), Error> {
    // Initialize the simulation grid.
    let mut grid = Grid::new(100, 100);
    grid.update_cell(10, 10, CellState::Food);
    grid.update_cell(20, 20, CellState::Wall);

    // Create a window using winit.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("LifeEngine - Native")
        .with_inner_size(LogicalSize::new(800.0, 800.0))
        .build(&event_loop)
        .unwrap();

    // Create a pixel buffer that matches the grid size.
    let width = grid.cols as u32;
    let height = grid.rows as u32;
    let surface_texture = SurfaceTexture::new(width, height, &window);
    let mut pixels = Pixels::new(width, height, surface_texture)?;

    // Run the event loop.
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(_) => {
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
    let width = grid.cols;
    let height = grid.rows;
    // Each pixel is RGBA (4 bytes).
    for row in 0..height {
        for col in 0..width {
            let i = (row * width + col) * 4;
            let cell = grid.get_cell(col, row).unwrap();
            let color = match cell.state {
                CellState::Empty => [0, 0, 0, 255],    // Black
                CellState::Food  => [0, 255, 0, 255],  // Green
                CellState::Wall  => [128, 128, 128, 255],// Gray
            };
            frame[i..i + 4].copy_from_slice(&color);
        }
    }
}
