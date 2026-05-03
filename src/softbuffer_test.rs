use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle},
    window::{Window, WindowAttributes, WindowId},
};

pub struct SoftbufferTest {
    window: Option<Rc<Window>>,
    window_id: Option<WindowId>,
    context: Option<Context<OwnedDisplayHandle>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    width: u32,
    height: u32,
}

impl SoftbufferTest {
    pub fn new() -> Self {
        Self {
            window: None,
            window_id: None,
            context: None,
            surface: None,
            width: 800,
            height: 600,
        }
    }

    fn render(&mut self) {
        if let Some(surface) = &mut self.surface {
            // Resize surface to match window
            if let Some(window) = &self.window {
                let size = window.inner_size();
                if let (Some(w), Some(h)) = (
                    NonZeroU32::new(size.width),
                    NonZeroU32::new(size.height),
                ) {
                    if let Err(e) = surface.resize(w, h) {
                        eprintln!("Failed to resize surface: {}", e);
                        return;
                    }
                }
            }

            // Get the buffer (mutable slice of pixels)
            let mut buffer = match surface.buffer_mut() {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed to get buffer: {}", e);
                    return;
                }
            };

            let width = buffer.width().get();
            let height = buffer.height().get();

            // Fill with a gradient pattern
            for y in 0..height {
                for x in 0..width {
                    // simple gradient: red increases with x, blue increases with y
                    let red = (x * 255 / width) as u8;
                    let green = 128; // constant
                    let blue = (y * 255 / height) as u8;

                    // Pack into u32 (little-endian: 0xBBGGRR)
                    let pixel = (blue as u32) | ((green as u32) << 8) | ((red as u32) << 16);

                    buffer[(y * width + x) as usize] = pixel;
                }
            }

            // Present the buffer to the window
            if let Err(e) = buffer.present() {
                eprintln!("Failed to present buffer: {}", e);
            }
        }
    }
}

impl ApplicationHandler for SoftbufferTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_title("Softbuffer Test - Gradient")
            .with_inner_size(winit::dpi::LogicalSize::new(self.width as f64, self.height as f64));

        if let Ok(window) = event_loop.create_window(attributes) {
            let window = Rc::new(window);
            self.window_id = Some(window.id());

            // Create softbuffer context with owned display handle
            let context = Context::new(event_loop.owned_display_handle())
                .expect("Failed to create context");

            // Create surface bound to the window
            let surface = Surface::new(&context, window.clone())
                .expect("Failed to create surface");

            self.context = Some(context);
            self.surface = Some(surface);
            self.window = Some(window);

            // Initial render
            self.render();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window_id != Some(window_id) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Request redraw for continuous updates
        if self.window.is_some() {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            event_loop.set_control_flow(ControlFlow::Poll);
        } else {
            event_loop.exit();
        }
    }
}

pub fn run_softbuffer_test() {
    println!("Starting softbuffer test...");
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = SoftbufferTest::new();
    event_loop.run_app(&mut app).expect("Event loop failed");
    println!("Softbuffer test closed");
}