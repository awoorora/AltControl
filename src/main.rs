mod grid_map;
mod overlay;
mod softbuffer_test;

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use rdev::{Event, listen, EventType, Key, display_size};
use winit::event_loop::EventLoop;
use crate::AppMode::{Monitoring, Navigating};
use crate::grid_map::GridEngine;
use crate::overlay::OverlayApp;
use std::sync::mpsc::{self, Sender, Receiver};

#[derive(Debug, Clone)]
pub enum OverlayCommand {
    Show,
    Hide,
}

// Global channel sender (rdev thread -> winit main thread)
static OVERLAY_TX: OnceLock<Sender<OverlayCommand>> = OnceLock::new();

enum AppMode {
    Monitoring,
    Navigating,
}

struct AppState {
    keys_history: Vec<Key>,
    keys_current: HashSet<Key>,
    app_mode: AppMode,
    grid_engine: GridEngine
}

static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

fn get_state() -> &'static Mutex<AppState> {
    STATE.get_or_init(|| {
        let (w, h) = display_size().unwrap();
        Mutex::new(AppState {
            keys_history: Vec::new(),
            keys_current: HashSet::new(),
            app_mode: Monitoring,
            // Use logical size for now; rdev::simulate works with these coordinates (on my machine at least)
            grid_engine: GridEngine::new(w as f32, h as f32),
        })
    })
}

fn handle_monitoring(event: Event, state: &mut AppState) {
    match event.event_type {
        EventType::KeyPress(key) => {
            state.keys_current.insert(key);
            state.keys_history.push(key);
            state.keys_history.dedup();

            if state.keys_history.len() >= 2 {
                let last = state.keys_history.last().unwrap();
                let second_to_last = state.keys_history.get(state.keys_history.len() - 2).unwrap();

                if *second_to_last == Key::Alt && *last == Key::ControlLeft {
                    state.app_mode = Navigating;
                    state.keys_history.clear();
                    state.keys_current.clear();

                    // Send Show command to overlay
                    if let Some(tx) = OVERLAY_TX.get() {
                        let _ = tx.send(OverlayCommand::Show);
                    }
                }
            }
        }
        EventType::KeyRelease(key) => {
            state.keys_current.remove(&key);
            state.keys_history.clear();
        }
        _ => {}
    }
}

fn handle_navigation(event: Event, state: &mut AppState) {
    match event.event_type {
        EventType::KeyPress(key) => {
            if state.keys_current.contains(&key) { return; }
            state.keys_current.insert(key);
            state.keys_history.push(key);

            if state.keys_history.len() >= 2 {
                let col_key = state.keys_history[0];
                let row_key = state.keys_history[1];

                if let Some((x, y)) = state.grid_engine.get_coords(col_key, row_key) {
                    // Simulate mouse move on a separate thread (rdev::simulate can block)
                    std::thread::spawn(move || {
                        if let Err(e) = rdev::simulate(&EventType::MouseMove { x: x as f64, y: y as f64 }) {
                            eprintln!("Failed to simulate mouse move: {:?}", e);
                        }
                    });
                }

                // Reset state and hide overlay
                state.keys_current.clear();
                state.keys_history.clear();
                state.app_mode = Monitoring;

                if let Some(tx) = OVERLAY_TX.get() {
                    let _ = tx.send(OverlayCommand::Hide);
                }
            }
        }
        EventType::KeyRelease(key) => {
            state.keys_current.remove(&key);
            if key == Key::Escape {
                state.keys_history.clear();
                state.app_mode = Monitoring;
                if let Some(tx) = OVERLAY_TX.get() {
                    let _ = tx.send(OverlayCommand::Hide);
                }
            }
        }
        _ => {}
    }
}

fn callback(event: Event) {
    let mut state = get_state().lock().unwrap();
    match state.app_mode {
        Monitoring => handle_monitoring(event, &mut state),
        Navigating => handle_navigation(event, &mut state),
    }
}

fn main() {
    // softbuffer_test::run_softbuffer_test();
    // return;

    // Create command channel
    let (tx, rx) = mpsc::channel();
    OVERLAY_TX.set(tx).unwrap();

    // Create winit event loop and overlay app
    // !! main thread
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut overlay_app = OverlayApp::new(rx);

    // Spawn rdev listener in background thread
    std::thread::spawn(|| {
        if let Err(e) = listen(callback) {
            eprintln!("rdev error: {:?}", e);
        }
    });

    // Run winit loop on main thread (required for window creation, winit is picky)
    event_loop.run_app(&mut overlay_app).expect("Event loop failed");
}