mod grid_map;

use std::collections::HashSet;
use std::mem::swap;
use std::sync::{Mutex, OnceLock};
use rdev::{Event, listen, EventType, Key, display_size};
use crate::AppMode::{Monitoring, Navigating};
use crate::grid_map::GridEngine;

// TODO: REMOVE!
// Check winit's monitor methods or the dpi crate
const magic_scaling_number: f32 = 1.25;

enum AppMode{
    Monitoring,
    Navigating
}


struct AppState{
    keys_history: Vec<Key>,
    keys_current: HashSet<Key>,
    app_mode:AppMode,
    grid_engine: GridEngine
}

static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

fn get_state() -> &'static Mutex <AppState> {
    STATE.get_or_init(|| {
        let (w, h) = display_size().unwrap();

        let w = w as f32;
        let h = h as f32;
        Mutex::new(AppState{
            keys_history: Vec::new(),
            keys_current: HashSet::new(),
            app_mode:Monitoring,

            grid_engine: GridEngine::new(w * magic_scaling_number, h * magic_scaling_number)
        })
    })
}

fn handle_monitoring(event: Event, state: &mut AppState){
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
                    println!("Overlay Opened");
                    state.keys_history.clear();
                    state.keys_current.clear();
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

            println!("DEBUG: A key has been pressed key history: {:?}", state.keys_history);
            
            if state.keys_history.len() >= 2 {
                println!("DEBUG: Two keys in history, attempting to get their coords");
                let col_selector_key = state.keys_history[0];
                let row_selector_key = state.keys_history[1];

                if let Some((x, y)) = state.grid_engine.get_coords(col_selector_key, row_selector_key) {
                    std::thread::spawn(move || {
                        let actual_x = x / magic_scaling_number;
                        let actual_y = y / magic_scaling_number;
                        if let Err(e) = rdev::simulate(&EventType::MouseMove { x: actual_x as f64, y: actual_y as f64 }) {
                            eprintln!("Failed to simulate mouse move: {:?}", e);
                        } else {
                            println!("Moved to: {}, {}", x, y);
                        }
                    });
                } else {
                    println!("Invalid Grid Selection!");
                }

                println!("DEBUG: Attempt finished successfully");

                state.keys_current.clear();
                state.keys_history.clear();
                state.app_mode = Monitoring;
                println!("Overlay Closed");
            }
        }

        EventType::KeyRelease(key) => {
            state.keys_current.remove(&key);
            if key == Key::Escape {
                state.keys_history.clear();
                state.app_mode = Monitoring;
                println!("Overlay Closed");
            }
        }
        _ => {}
    }
}

fn callback(event: Event) {
    let mut state = get_state().lock().unwrap();
    match state.app_mode {
        Monitoring => {
            // Monitoring Mode
            // Listen for Alt + Ctrl
            handle_monitoring(event, &mut state);
        }
        Navigating => {
            // Navigation Mode
            // Capture keys into Vec for grid selection
            // If Escape, set is_overlay_visible = false
            handle_navigation(event, &mut state);
        }
        _ => {}
    }
}

fn main(){
    let (w, h) = display_size().unwrap();

    let w = w as f32;
    let h = h as f32;

    println!("The screen has {:.0}x{:.0} pixels", w * magic_scaling_number, h * magic_scaling_number); // Windows Scaled at 125%
    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error)
    }
}

// TODO: Alt+Ctrl might mess up stuff in the current app the user has open. Find a way to fix it, or just put up a disclaimer