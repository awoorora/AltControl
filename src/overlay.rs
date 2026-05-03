use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, OwnedDisplayHandle},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId, WindowLevel},
};
use crate::OverlayCommand;
use std::sync::mpsc::Receiver;

use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;

pub struct OverlayApp {
    window: Option<Rc<Window>>,
    window_id: Option<WindowId>,
    command_rx: Option<Receiver<OverlayCommand>>,

    context: Option<Context<OwnedDisplayHandle>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,

    grid_width: u32,
    grid_height: u32,
}

impl OverlayApp {
    pub fn new(command_rx: Receiver<OverlayCommand>) -> Self {
        Self {
            window: None,
            window_id: None,
            command_rx: Some(command_rx),
            context: None,
            surface: None,
            grid_width: 1920,
            grid_height: 1080,
        }
    }

    fn render_grid(&mut self) {
        let Some(surface) = &mut self.surface else { return; };

        // Resize surface to match window
        if let Some(window) = &self.window {
            let size = window.inner_size();
            if let (Some(w), Some(h)) = (
                NonZeroU32::new(size.width),
                NonZeroU32::new(size.height),
            ) {
                let _ = surface.resize(w, h);
            }
        }

        let mut buffer = match surface.buffer_mut() {
            Ok(b) => b,
            Err(e) => { eprintln!("[overlay] buffer_mut failed: {}", e); return; }
        };

        // Cache dimensions to avoid borrow checker issues
        let buf_width = buffer.width().get();
        let buf_height = buffer.height().get();

        // Clear to transparent black
        for pixel in buffer.iter_mut() {
            *pixel = 0x00000000;
        }

        // Draw vertical lines (10 columns -> 11 lines)
        // Draw vertical lines (10 columns -> 11 lines)
        let cell_w = self.grid_width / 10; // 2400 / 10 = 240 physical pixels
        let line_thickness = 2;

        for col in 0..=10 {
            let base_x = col * cell_w;

            for dx in 0..line_thickness {
                let x = base_x + dx;
                if x < buf_width {
                    for y in 0..buf_height {
                        let idx = (y * buf_width + x) as usize;
                        if idx < buffer.len() {
                            buffer[idx] = 0xFFFFFF; // White line
                        }
                    }
                }
            }
        }

        // Draw horizontal lines (30 rows -> 31 lines)
        let cell_h = self.grid_height / 30; // 1350 / 30 = 45 physical pixels
        let line_thickness = 1;

        for row in 0..=30 {
            let base_y = row * cell_h;

            for dy in 0..line_thickness {
                let y = base_y + dy;
                if y < buf_height {
                    for x in 0..buf_width {
                        let idx = (y * buf_width + x) as usize;
                        if idx < buffer.len() {
                            buffer[idx] = 0xFFFFFF;
                        }
                    }
                }
            }
        }

        let _ = buffer.present();
    }
}

impl ApplicationHandler for OverlayApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window_id != Some(window_id) { return; }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if self.window.is_some() {
                    if let Some(window) = &self.window {
                        window.pre_present_notify();
                    }
                    self.render_grid();
                }
            }
            WindowEvent::Destroyed => {
                self.window = None;
                self.window_id = None;
                self.context = None;
                self.surface = None;
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(rx) = &self.command_rx {
            loop {
                match rx.try_recv() {
                    Ok(cmd) => match cmd {
                        OverlayCommand::Show => {
                            if self.window.is_none() {
                                let attributes = WindowAttributes::default()
                                    .with_title("Grid Overlay")
                                    .with_transparent(true)
                                    .with_decorations(false)
                                    .with_window_level(WindowLevel::AlwaysOnTop)
                                    .with_visible(true)
                                    .with_active(false)
                                    .with_position(winit::dpi::LogicalPosition::new(0.0, 0.0))
                                    .with_inner_size(winit::dpi::LogicalSize::new(1920.0, 1080.0));

                                match event_loop.create_window(attributes) {
                                    Ok(window) => {
                                        // Wrap in Rc for softbuffer
                                        let window = Rc::new(window);

                                        // // DEBUG: Print window info
                                        // println!("[DEBUG] Window created:");
                                        // println!("  - Outer position: {:?}", window.outer_position());
                                        // println!("  - Outer size: {:?}", window.outer_size());
                                        // println!("  - Inner size: {:?}", window.inner_size());
                                        // println!("  - Is visible: {:?}", window.is_visible());
                                        // println!("  - Scale factor: {}", window.scale_factor());
                                        //
                                        // // Get monitor info
                                        // if let Some(monitor) = window.current_monitor() {
                                        //     println!("  - Monitor position: {:?}", monitor.position());
                                        //     println!("  - Monitor size: {:?}", monitor.size());
                                        //     println!("  - Monitor scale factor: {}", monitor.scale_factor());
                                        // }

                                        // Create softbuffer context & surface
                                        let context = Context::new(event_loop.owned_display_handle())
                                            .expect("Failed to create context");
                                        let surface = Surface::new(&context, window.clone())
                                            .expect("Failed to create surface");

                                        self.context = Some(context);
                                        self.surface = Some(surface);
                                        self.window_id = Some(window.id());
                                        self.window = Some(window);

                                        // Initial draw
                                        if let Some(w) = &self.window {
                                            w.request_redraw();
                                        }

                                    }
                                    Err(e) => eprintln!("[overlay] Window creation failed: {}", e),
                                }
                            }
                        }
                        OverlayCommand::Hide => {
                            self.window = None;
                            self.window_id = None;
                            self.context = None;
                            self.surface = None;
                        }
                    },
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(e) => eprintln!("[overlay] Channel error: {:?}", e),
                }
            }
        }
        event_loop.set_control_flow(ControlFlow::Poll);
    }
}