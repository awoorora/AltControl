mod grid_map;

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use rdev::{Event, listen, EventType, Key, display_size};
use crate::AppMode::{Monitoring, Navigating};

#[derive(Debug)]
enum AppMode{
    Monitoring,
    Navigating
}

#[derive(Debug)]
struct AppState{
    keys_history: Vec<Key>,
    keys_current: HashSet<Key>,
    app_mode:AppMode
}

static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

fn get_state() -> &'static Mutex <AppState> {
    STATE.get_or_init(|| {
        Mutex::new(AppState{
            keys_history: Vec::new(),
            keys_current: HashSet::new(),
            app_mode:Monitoring
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

fn handle_navigation(event: Event, state: &mut AppState){
    match event.event_type {
        EventType::KeyPress(key) => {
            if state.keys_current.contains(&key) {
                return;
            }
            state.keys_current.insert(key);
        }
        EventType::KeyRelease(key) => {
            state.keys_current.clear();

            if key == Key::Escape{
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

    // TODO: remove the magic number for scaling
    // Check winit's monitor methods or the dpi crate
    println!("The screen has {:.0}x{:.0} pixels", w * 1.25, h * 1.25); // Windows Scaled at 125%
    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error)
    }
}

// TODO: Alt+Ctrl might mess up stuff in the current app the user has open. Find a way to fix it, or just put up a disclaimer
// TODO: big grid_map.rs, currently empty
// TODO: actual math based on keys pressed