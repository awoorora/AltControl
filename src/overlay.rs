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

    font: Option<rusttype::Font<'static>>,
    font_px: f32,
}

fn get_text_width(font: &rusttype::Font, text: &str, font_px: f32) -> f32 {
    let scale = rusttype::Scale::uniform(font_px);
    let mut caret = 0.0;

    for c in text.chars() {
        if let Some(glyph) = font.glyph(c).scaled(scale).positioned(rusttype::Point { x: caret, y: 0.0 }).pixel_bounding_box() {
            caret = glyph.max.x as f32 + font_px * 0.1; // Small gap between chars
        }
    }
    caret
}

fn draw_text_to_buffer(
    font: &rusttype::Font,
    font_px: f32,
    grid_width: u32,
    grid_height: u32,
    buffer: &mut [u32],
    buf_width: u32,
    buf_height: u32,
    text: &str,
    cell_x: u32,
    cell_y: u32,
) {
    let cell_w = grid_width / 10;
    let cell_h = grid_height / 30;

    let scale = rusttype::Scale::uniform(font_px);
    let v_metrics = font.v_metrics(scale);

    let text_height = v_metrics.ascent - v_metrics.descent;
    let baseline_y = cell_y as f32 + cell_h as f32 / 2.0 + text_height / 2.0;

    let mut text_width = 0.0;
    let mut caret = 0.0;
    for c in text.chars() {
        let glyph = font.glyph(c).scaled(scale).positioned(rusttype::Point { x: caret, y: 0.0 });
        if let Some(bb) = glyph.pixel_bounding_box() {
            text_width = bb.max.x as f32;
            caret = bb.max.x as f32 + font_px * 0.1; // Gap between chars
        }
    }
    let start_x = cell_x as f32 + cell_w as f32 / 2.0 - text_width / 2.0;
    let mut caret_x = start_x;
    let caret_y = baseline_y;

    for c in text.chars() {
        let glyph = font.glyph(c)
            .scaled(scale)
            .positioned(rusttype::Point { x: caret_x, y: caret_y });

        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let px_i32 = bb.min.x + x as i32;
                let py_i32 = bb.min.y + y as i32;

                if px_i32 >= 0 && px_i32 < buf_width as i32
                    && py_i32 >= 0 && py_i32 < buf_height as i32 {

                    let idx = (py_i32 as u32 * buf_width + px_i32 as u32) as usize;
                    if idx < buffer.len() {
                        // Read the existing pixel from buffer (already has column background)
                        let existing = buffer[idx];
                        let bg_r = existing & 0xFF;
                        let bg_g = (existing >> 8) & 0xFF;
                        let bg_b = (existing >> 16) & 0xFF;

                        // Foreground: white text
                        let fg_r = 255u32;
                        let fg_g = 255u32;
                        let fg_b = 255u32;

                        // Alpha coverage from glyph
                        let a = (v * 255.0) as u32;
                        let a_inv = 255 - a;

                        // Alpha blend: result = fg * a + bg * (1-a)
                        let r = (fg_r * a + bg_r * a_inv) / 255;
                        let g = (fg_g * a + bg_g * a_inv) / 255;
                        let b = (fg_b * a + bg_b * a_inv) / 255;

                        // Pack as 0x00BBGGRR
                        buffer[idx] = (b << 16) | (g << 8) | r;
                    }
                }
            });
        }
        // Advance caret using actual glyph width + small gap
        if let Some(bb) = glyph.pixel_bounding_box() {
            caret_x = bb.max.x as f32 + font_px * 0.1;
        } else {
            caret_x += font_px * 0.6; // Fallback for invisible glyphs
        }
    }
}

fn blend_color_with_black(hex_rgb: u32, opacity: f32) -> u32 {
    // Extract RGB from 0xRRGGBB format
    let r = ((hex_rgb >> 16) & 0xFF) as f32;
    let g = ((hex_rgb >> 8) & 0xFF) as f32;
    let b = (hex_rgb & 0xFF) as f32;

    // Pre-blend with black: result = color * opacity + black * (1 - opacity) = color * opacity
    let blended_r = (r * opacity) as u8;
    let blended_g = (g * opacity) as u8;
    let blended_b = (b * opacity) as u8;

    // Pack as 0x00BBGGRR (softbuffer Windows format: alpha ignored, B in bits 16-23)
    (blended_b as u32) << 16 | (blended_g as u32) << 8 | (blended_r as u32)
}

impl OverlayApp {
    pub fn new(command_rx: Receiver<OverlayCommand>) -> Self {
        let font_bytes: &'static [u8] = include_bytes!("../assets/calibril.ttf");
        let font = rusttype::Font::try_from_bytes(font_bytes);

        Self {
            window: None,
            window_id: None,
            command_rx: Some(command_rx),
            context: None,
            surface: None,
            grid_width: 1920,
            grid_height: 1080,
            font,
            font_px: 25.0, // Adjust
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

        let buf_width = buffer.width().get();
        let buf_height = buffer.height().get();

        // Clear to transparent black
        for pixel in buffer.iter_mut() {
            *pixel = 0x00000000;
        }

        // raw column backgrounds (BEFORE lines/text)
        let column_colors = [
            0xf86565, // col 0 & 9: red
            0xffc766, // col 1 & 8: orange
            0xe6ff66, // col 2 & 7: yellow
            0x68ff66, // col 3 & 6: green
            0x66f7ff, // col 4 & 5: cyan
        ];

        let cell_w = self.grid_width / 10;
        let cell_h = self.grid_height / 30;
        let opacity = 0.2; // 20%

        for col in 0..10 {
            // Symmetric color mapping: 0-9, 1-8, 2-7, 3-6, 4-5
            let color_idx = if col < 5 { col } else { 9 - col };
            let bg_pixel = blend_color_with_black(column_colors[color_idx], opacity);

            let start_x = col * cell_w as usize;
            let end_x = (col + 1) * cell_w as usize;

            for y in 0..buf_height {
                for x in start_x..end_x.min(buf_width as usize) {
                    let idx = (y * buf_width + x as u32) as usize;
                    if idx < buffer.len() {
                        buffer[idx] = bg_pixel;
                    }
                }
            }
        }

        // Draw vertical grid lines (opaque white, overwrites background)
        let line_thickness = 2;
        for col in 0..=10 {
            let base_x = col * cell_w;
            for dx in 0..line_thickness {
                let x = base_x + dx;
                if x < buf_width {
                    for y in 0..buf_height {
                        let idx = (y * buf_width + x) as usize;
                        if idx < buffer.len() {
                            buffer[idx] = 0xFFFFFF; // Opaque white
                        }
                    }
                }
            }
        }

        // Draw horizontal grid lines (opaque white)
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

        // Draw text labels (opaque white, overwrites background and lines)
        let col_keys = ['A','S','D','F','G','H','J','K','L',';'];
        let row_keys = ['Q','W','E','R','T','Y','U','I','O','P', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ';', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', ',', '.', '/'];

        for (col_idx, &col_key) in col_keys.iter().enumerate() {
            for (row_idx, &row_key) in row_keys.iter().enumerate() {
                let label = format!("{}{}", col_key, row_key);
                let cell_x = (col_idx * cell_w as usize) as u32;
                let cell_y = (row_idx * cell_h as usize) as u32;

                if let Some(font) = &self.font {
                    draw_text_to_buffer(
                        font, self.font_px, self.grid_width, self.grid_height,
                        &mut buffer, buf_width, buf_height,
                        &label, cell_x, cell_y,
                    );
                }
            }
        }

        // Present to screen
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