#![deny(clippy::all)]
#![forbid(unsafe_code)]

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: usize = 320;
const HEIGHT: usize = 320;
const GRID_SIZE: usize = WIDTH * HEIGHT;
const FRAME_SIZE: usize = 4 * WIDTH as usize * HEIGHT as usize;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    grid: Box<[f32; GRID_SIZE]>
}

fn index_to_coords(i: usize) -> (i32, i32) {
    let x = (i % WIDTH as usize) as i32;
    let y = (i / WIDTH as usize) as i32;
    (x, y)
}

fn clamp(x: i32, limit: usize) -> usize {
    if x < 0 {
        0
    } else if x as usize >= limit {
        limit - 1
    } else {
        x as usize
    }
}

fn coords_to_index(x: i32, y: i32) -> usize {
    let x2 = clamp(x, WIDTH);
    let y2 = clamp(y, HEIGHT);
    y2 * WIDTH + x2
}

impl World {
    /// Create a new `World` instance with zero pixels.
    fn new() -> Self {
        let mut grid = Box::new([0f32; GRID_SIZE]);

        let radius: i32 = 100;
        let px = (WIDTH / 2) as i32;
        let py = (HEIGHT / 2) as i32;

        for (i, val) in grid.iter_mut().enumerate() {
            let (x, y) = index_to_coords(i);

            *val = if (x - px).pow(2) + (y - py).pow(2) < radius.pow(2) {
                255.
            } else {
                0.
            }
        }

        Self {
            grid
        }
    }

    /// Update the `World` internal state.
    fn update(&mut self) {
        let mut grid = Box::new([0f32; GRID_SIZE]);

        let kappa = 1.;

        for (i, val) in (*grid).iter_mut().enumerate() {
            let (x, y) = index_to_coords(i);
            let val0 = self.grid[coords_to_index(x, y)];
            let val1 = self.grid[coords_to_index(x+1, y)];
            let val2 = self.grid[coords_to_index(x, y-1)];
            let val3 = self.grid[coords_to_index(x-1, y)];
            let val4 = self.grid[coords_to_index(x, y+1)];

            *val = val0 - kappa * (val0 - (val1 + val2 + val3 + val4) / 4.);
        }

        self.grid.copy_from_slice(&*grid);
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let val = self.grid[i] as u8;
            let color = [val, val, val, 255];
            pixel.copy_from_slice(&color);
        }
    }
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Fluit")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };
    let mut world = Box::new(World::new());

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| eprintln!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            if event != Event::MainEventsCleared {
                println!("{:?}", event);
            }
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}