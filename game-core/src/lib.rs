mod components;
mod engine;
mod events;
mod pathfinding;
mod world;

use std::cell::RefCell;
use wasm_bindgen::prelude::*;

use engine::GameEngine;

thread_local! {
    static ENGINE: RefCell<Option<GameEngine>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn init() -> String {
    let mut engine = GameEngine::new();
    let snapshot = engine.get_initial_snapshot();

    ENGINE.with(|e| {
        *e.borrow_mut() = Some(engine);
    });

    snapshot
}

#[wasm_bindgen]
pub fn tick(dt: f32) {
    ENGINE.with(|e| {
        if let Some(ref mut engine) = *e.borrow_mut() {
            engine.tick(dt);
        }
    });
}

#[wasm_bindgen]
pub fn push_event(json: &str) {
    ENGINE.with(|e| {
        if let Some(ref mut engine) = *e.borrow_mut() {
            engine.push_event(json);
        }
    });
}

#[wasm_bindgen]
pub fn drain_events() -> String {
    ENGINE.with(|e| {
        if let Some(ref mut engine) = *e.borrow_mut() {
            engine.drain_events()
        } else {
            "[]".to_string()
        }
    })
}
