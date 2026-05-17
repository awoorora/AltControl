use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, OwnedDisplayHandle},
    keyboard::NamedKey,
    window::{Window, WindowAttributes, WindowId, WindowLevel},
};
use crate::OverlayCommand;
use std::sync::mpsc::Receiver;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use rdev::{EventType, Key};

// Navigation State Machine

#[derive(Debug, Clone, Copy, PartialEq)]
enum Side { Left, Right }

#[derive(Debug, Clone, Copy, PartialEq)]
enum NavState {
    WaitingForColumn,
    ColumnSelected(Key),
    CellSelected(Key, Key),
    ThirdSelected(Key, Key, Side),
}

// OverlayApp

pub struct OverlayApp {
    window: Option<Rc<Window>>,
    window_id: Option<WindowId>,
    command_rx: Receiver<OverlayCommand>,
    context: Option<Context<OwnedDisplayHandle>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    grid_width: u32,
    grid_height: u32,
    font: Option<rusttype::Font<'static>>,
    font_px: f32,
    nav_state: NavState,
    ready_for_input: bool,
}


// Key Mapping Helpers

fn char_to_rdev_key(c: char) -> Option<Key> {
    match c {
        'A' => Some(Key::KeyA), 'S' => Some(Key::KeyS), 'D' => Some(Key::KeyD),
        'F' => Some(Key::KeyF), 'G' => Some(Key::KeyG), 'H' => Some(Key::KeyH),
        'J' => Some(Key::KeyJ), 'K' => Some(Key::KeyK), 'L' => Some(Key::KeyL),
        ';' => Some(Key::SemiColon),
        'Q' => Some(Key::KeyQ), 'W' => Some(Key::KeyW), 'E' => Some(Key::KeyE),
        'R' => Some(Key::KeyR), 'T' => Some(Key::KeyT), 'Y' => Some(Key::KeyY),
        'U' => Some(Key::KeyU), 'I' => Some(Key::KeyI), 'O' => Some(Key::KeyO),
        'P' => Some(Key::KeyP),
        'Z' => Some(Key::KeyZ), 'X' => Some(Key::KeyX), 'C' => Some(Key::KeyC),
        'V' => Some(Key::KeyV), 'B' => Some(Key::KeyB), 'N' => Some(Key::KeyN),
        'M' => Some(Key::KeyM), ',' => Some(Key::Comma), '.' => Some(Key::Dot),
        '/' => Some(Key::Slash),
        _ => None,
    }
}

fn is_column_key(key: Key) -> bool {
    matches!(key,
        Key::KeyA | Key::KeyS | Key::KeyD | Key::KeyF | Key::KeyG |
        Key::KeyH | Key::KeyJ | Key::KeyK | Key::KeyL | Key::SemiColon
    )
}

fn key_side(key: Key) -> Option<Side> {
    match key {
        Key::KeyQ | Key::KeyW | Key::KeyE | Key::KeyR | Key::KeyT |
        Key::KeyA | Key::KeyS | Key::KeyD | Key::KeyF | Key::KeyG |
        Key::KeyZ | Key::KeyX | Key::KeyC | Key::KeyV | Key::KeyB => Some(Side::Left),

        Key::KeyY | Key::KeyU | Key::KeyI | Key::KeyO | Key::KeyP |
        Key::KeyH | Key::KeyJ | Key::KeyK | Key::KeyL | Key::SemiColon |
        Key::KeyN | Key::KeyM | Key::Comma | Key::Dot | Key::Slash => Some(Side::Right),

        _ => None,
    }
}

fn col_map() -> std::collections::HashMap<Key, usize> {
    let mut m = std::collections::HashMap::new();
    let keys = [
        Key::KeyA, Key::KeyS, Key::KeyD, Key::KeyF, Key::KeyG,
        Key::KeyH, Key::KeyJ, Key::KeyK, Key::KeyL, Key::SemiColon,
    ];
    for (i, k) in keys.iter().enumerate() { m.insert(*k, i); }
    m
}

fn row_map() -> std::collections::HashMap<Key, usize> {
    let mut m = std::collections::HashMap::new();
    let keys = [
        Key::KeyQ, Key::KeyW, Key::KeyE, Key::KeyR, Key::KeyT,
        Key::KeyY, Key::KeyU, Key::KeyI, Key::KeyO, Key::KeyP,
        Key::KeyA, Key::KeyS, Key::KeyD, Key::KeyF, Key::KeyG,
        Key::KeyH, Key::KeyJ, Key::KeyK, Key::KeyL, Key::SemiColon,
        Key::KeyZ, Key::KeyX, Key::KeyC, Key::KeyV, Key::KeyB,
        Key::KeyN, Key::KeyM, Key::Comma, Key::Dot, Key::Slash,
    ];
    for (i, k) in keys.iter().enumerate() { m.insert(*k, i); }
    m
}

// Text Rendering

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
        if let Some(bb) = font.glyph(c).scaled(scale)
            .positioned(rusttype::Point { x: caret, y: 0.0 })
            .pixel_bounding_box()
        {
            text_width = bb.max.x as f32;
            caret = bb.max.x as f32 + font_px * 0.1;
        }
    }

    let mut caret_x = cell_x as f32 + cell_w as f32 / 2.0 - text_width / 2.0;
    let caret_y = baseline_y;

    for c in text.chars() {
        let glyph = font.glyph(c).scaled(scale)
            .positioned(rusttype::Point { x: caret_x, y: caret_y });

        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let px = bb.min.x + x as i32;
                let py = bb.min.y + y as i32;
                if px >= 0 && px < buf_width as i32 && py >= 0 && py < buf_height as i32 {
                    let idx = (py as u32 * buf_width + px as u32) as usize;
                    if idx < buffer.len() && v > 0.3 {
                        let existing = buffer[idx];
                        let bg_r = existing & 0xFF;
                        let bg_g = (existing >> 8) & 0xFF;
                        let bg_b = (existing >> 16) & 0xFF;
                        let a = (v * 255.0) as u32;
                        let a_inv = 255 - a;
                        let r = (255 * a + bg_r * a_inv) / 255;
                        let g = (255 * a + bg_g * a_inv) / 255;
                        let b = (255 * a + bg_b * a_inv) / 255;
                        buffer[idx] = (b << 16) | (g << 8) | r;
                    }
                }
            });
            caret_x = bb.max.x as f32 + font_px * 0.1;
        } else {
            caret_x += font_px * 0.6;
        }
    }
}

fn blend_color_with_black(hex_rgb: u32, opacity: f32) -> u32 {
    let r = ((hex_rgb >> 16) & 0xFF) as f32;
    let g = ((hex_rgb >> 8) & 0xFF) as f32;
    let b = (hex_rgb & 0xFF) as f32;
    let blended_r = (r * opacity) as u8;
    let blended_g = (g * opacity) as u8;
    let blended_b = (b * opacity) as u8;
    (blended_b as u32) << 16 | (blended_g as u32) << 8 | (blended_r as u32)
}

// OverlayApp Implementation

impl OverlayApp {
    pub fn new(command_rx: Receiver<OverlayCommand>) -> Self {
        let font_bytes: &'static [u8] = include_bytes!("../assets/calibril.ttf");
        let font = rusttype::Font::try_from_bytes(font_bytes);

        Self {
            window: None,
            window_id: None,
            command_rx,
            context: None,
            surface: None,
            grid_width: 0,
            grid_height: 0,
            font,
            font_px: 25.0,
            nav_state: NavState::WaitingForColumn,
            ready_for_input: false,
        }
    }

    fn render_grid(&mut self) {
        if self.grid_width == 0 || self.grid_height == 0 { return; }
        let Some(surface) = &mut self.surface else { return; };

        if let Some(window) = &self.window {
            let size = window.inner_size();
            if let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
                let _ = surface.resize(w, h);
            }
        }

        let mut buffer = match surface.buffer_mut() {
            Ok(b) => b,
            Err(e) => { eprintln!("[overlay] buffer_mut failed: {}", e); return; }
        };

        let buf_width = buffer.width().get();
        let buf_height = buffer.height().get();
        let cell_w = self.grid_width / 10;
        let cell_h = self.grid_height / 30;

        // Clear to transparent
        for pixel in buffer.iter_mut() { *pixel = 0x00000000; }

        // Column backgrounds (20% opacity, symmetric colors)
        let column_colors = [0xf86565, 0xffc766, 0xe6ff66, 0x68ff66, 0x66f7ff];
        let opacity = 0.2;
        for col in 0..10usize {
            let color_idx = if col < 5 { col } else { 9 - col };
            let bg_pixel = blend_color_with_black(column_colors[color_idx], opacity);
            let start_x = col * cell_w as usize;
            let end_x = (col + 1) * cell_w as usize;
            for y in 0..buf_height {
                for x in start_x..end_x.min(buf_width as usize) {
                    let idx = (y * buf_width + x as u32) as usize;
                    if idx < buffer.len() { buffer[idx] = bg_pixel; }
                }
            }
        }

        // Vertical grid lines (2px, white)
        for col in 0..=10 {
            for dx in 0..2 {
                let x = col * cell_w + dx;
                if x < buf_width {
                    for y in 0..buf_height {
                        let idx = (y * buf_width + x) as usize;
                        if idx < buffer.len() { buffer[idx] = 0xFFFFFF; }
                    }
                }
            }
        }

        // Horizontal grid lines (1px, white)
        for row in 0..=30 {
            let y = row * cell_h;
            if y < buf_height {
                for x in 0..buf_width {
                    let idx = (y * buf_width + x) as usize;
                    if idx < buffer.len() { buffer[idx] = 0xFFFFFF; }
                }
            }
        }

        // Cell labels
        let col_keys = ['A','S','D','F','G','H','J','K','L',';'];
        let row_keys = [
            'Q','W','E','R','T','Y','U','I','O','P',
            'A','S','D','F','G','H','J','K','L',';',
            'Z','X','C','V','B','N','M',',','.','/'];

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

        let _ = buffer.present();
        self.ready_for_input = true;
    }

    fn handle_key_press(&mut self, key: winit::keyboard::Key) {
        if !self.ready_for_input {
            if let winit::keyboard::Key::Named(NamedKey::Escape) = key { self.cleanup(); }
            return;
        }

        match key {
            winit::keyboard::Key::Named(NamedKey::Escape) => self.cleanup(),
            winit::keyboard::Key::Named(NamedKey::Control) => {
                self.perform_click_and_close(rdev::Button::Left);
            }
            winit::keyboard::Key::Named(NamedKey::Alt) => {
                self.perform_click_and_close(rdev::Button::Right);
            }
            winit::keyboard::Key::Character(ch) => {
                if let Some(c) = ch.chars().next() {
                    self.process_nav_key(c.to_ascii_uppercase());
                }
            }
            _ => {}
        }
    }

    fn process_nav_key(&mut self, ch: char) {
        let Some(key) = char_to_rdev_key(ch) else {
            self.cleanup();
            return;
        };

        match self.nav_state {
            NavState::WaitingForColumn => {
                if !is_column_key(key) { self.cleanup(); return; }
                // Preemptively move to column center (row G = index 14)
                if let Some((x, y)) = self.get_cell_center(key, Key::KeyG) {
                    self.simulate_mouse_move(x, y);
                }
                self.nav_state = NavState::ColumnSelected(key);
            }
            NavState::ColumnSelected(col_key) => {
                if let Some((x, y)) = self.get_cell_center(col_key, key) {
                    self.simulate_mouse_move(x, y);
                }
                self.nav_state = NavState::CellSelected(col_key, key);
            }
            NavState::CellSelected(col_key, row_key) => {
                match key_side(key) {
                    Some(side) => {
                        if let Some((x, y)) = self.get_third_center(col_key, row_key, side) {
                            self.simulate_mouse_move(x, y);
                        }
                        self.nav_state = NavState::ThirdSelected(col_key, row_key, side);
                    }
                    None => self.cleanup(),
                }
            }
            NavState::ThirdSelected(_, _, _) => {
                // Ctrl/Alt clicks are handled upstream in handle_key_press.
                // Any nav key here means the user wants to cancel.
                self.cleanup();
            }
        }
    }

    fn get_cell_center(&self, col_key: Key, row_key: Key) -> Option<(f64, f64)> {
        let col_idx = *col_map().get(&col_key)?;
        let row_idx = *row_map().get(&row_key)?;
        let cell_w = self.grid_width as f32 / 10.0;
        let cell_h = self.grid_height as f32 / 30.0;
        let x = col_idx as f32 * cell_w + cell_w / 2.0;
        let y = row_idx as f32 * cell_h + cell_h / 2.0;
        Some((x as f64, y as f64))
    }

    fn get_third_center(&self, col_key: Key, row_key: Key, side: Side) -> Option<(f64, f64)> {
        let col_idx = *col_map().get(&col_key)?;
        let row_idx = *row_map().get(&row_key)?;
        let cell_w = self.grid_width as f32 / 10.0;
        let cell_h = self.grid_height as f32 / 30.0;
        let cell_x = col_idx as f32 * cell_w;
        let y = row_idx as f32 * cell_h + cell_h / 2.0;
        let x = match side {
            Side::Left  => cell_x + cell_w / 6.0,
            Side::Right => cell_x + cell_w * 5.0 / 6.0,
        };
        Some((x as f64, y as f64))
    }

    fn perform_click_and_close(&mut self, button: rdev::Button) {
        self.cleanup();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(80));
            // Release the modifier key so the OS sees a plain click, not Ctrl/Alt+Click
            let modifier = if button == rdev::Button::Left { rdev::Key::ControlLeft } else { rdev::Key::Alt };
            let _ = rdev::simulate(&rdev::EventType::KeyRelease(modifier));
            std::thread::sleep(std::time::Duration::from_millis(10));
            let _ = rdev::simulate(&rdev::EventType::ButtonPress(button));
            let _ = rdev::simulate(&rdev::EventType::ButtonRelease(button));
        });
    }

    fn simulate_mouse_move(&self, x: f64, y: f64) {
        std::thread::spawn(move || {
            let _ = rdev::simulate(&EventType::MouseMove { x, y });
        });
    }

    fn cleanup(&mut self) {
        self.nav_state = NavState::WaitingForColumn;
        self.ready_for_input = false;
        self.window = None;
        self.window_id = None;
        self.context = None;
        self.surface = None;

        if let Some(state_mutex) = crate::STATE.get() {
            if let Ok(mut s) = state_mutex.lock() {
                s.app_mode = crate::AppMode::Monitoring;
                s.keys_history.clear();
                s.keys_current.clear();
            }
        }

        if let Some(tx) = crate::OVERLAY_TX.get() {
            let _ = tx.send(crate::OverlayCommand::Hide);
        }
    }
}

// ApplicationHandler Implementation

impl ApplicationHandler for OverlayApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window_id != Some(window_id) { return; }

        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                self.cleanup();
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { logical_key, state, .. }, ..
            } if state == ElementState::Pressed => {
                self.handle_key_press(logical_key);
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    window.pre_present_notify();
                }
                self.render_grid();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        loop {
            match self.command_rx.try_recv() {
                Ok(cmd) => match cmd {
                    OverlayCommand::Show => {
                        if self.window.is_none() {
                            let attributes = WindowAttributes::default()
                                .with_title("AltCtrl Grid")
                                .with_transparent(true)
                                .with_decorations(false)
                                .with_window_level(WindowLevel::AlwaysOnTop)
                                .with_visible(true)
                                .with_active(true)
                                .with_position(winit::dpi::LogicalPosition::new(0.0, 0.0))
                                .with_inner_size(winit::dpi::LogicalSize::new(1920.0, 1080.0));

                            match event_loop.create_window(attributes) {
                                Ok(window) => {
                                    let window = Rc::new(window);
                                    let (logical_w, logical_h) = rdev::display_size()
                                        .expect("Failed to get display size");

                                    let context = Context::new(event_loop.owned_display_handle())
                                        .expect("Failed to create softbuffer context");
                                    let surface = Surface::new(&context, window.clone())
                                        .expect("Failed to create softbuffer surface");

                                    self.grid_width = logical_w as u32;
                                    self.grid_height = logical_h as u32;
                                    self.context = Some(context);
                                    self.surface = Some(surface);
                                    self.window_id = Some(window.id());
                                    self.window = Some(window);

                                    if let Some(w) = &self.window {
                                        w.focus_window();
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
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    eprintln!("[overlay] Channel disconnected");
                    break;
                }
            }
        }

        event_loop.set_control_flow(ControlFlow::Poll);
    }
}
