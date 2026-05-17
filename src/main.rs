mod overlay;

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use rdev::{Event, listen, EventType, Key};
use winit::event_loop::EventLoop;
use crate::AppMode::{Monitoring, Navigating};
use crate::overlay::OverlayApp;
use std::sync::mpsc::{self, Sender};

#[derive(Debug, Clone)]
pub enum OverlayCommand {
    Show,
    Hide,
}

static OVERLAY_TX: OnceLock<Sender<OverlayCommand>> = OnceLock::new();

#[derive(PartialEq, Debug)]
pub enum AppMode {
    Monitoring,
    Navigating,
}

pub struct AppState {
    pub keys_history: Vec<Key>,
    pub keys_current: HashSet<Key>,
    pub app_mode: AppMode,
}

pub static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

fn get_state() -> &'static Mutex<AppState> {
    STATE.get_or_init(|| {
        Mutex::new(AppState {
            keys_history: Vec::new(),
            keys_current: HashSet::new(),
            app_mode: Monitoring,
        })
    })
}

fn callback(event: Event) {
    let mut state = get_state().lock().unwrap();

    if state.app_mode == Monitoring {
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
}

fn main() {
    let (tx, rx) = mpsc::channel();
    OVERLAY_TX.set(tx).unwrap();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut overlay_app = OverlayApp::new(rx);

    // rdev must run on a background thread; winit requires the main thread
    std::thread::spawn(|| {
        if let Err(e) = listen(callback) {
            eprintln!("rdev error: {:?}", e);
        }
    });

    event_loop.run_app(&mut overlay_app).expect("Event loop failed");
}
