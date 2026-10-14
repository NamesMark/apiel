use apiel::Env;
use apiel::parse::{eval_to_val, format_val};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static ENV: RefCell<Env> = RefCell::new(Env::new());
}

#[wasm_bindgen]
pub fn eval_apl(input: &str) -> String {
    // Taking the env out instead of borrowing it means a panic during eval
    // loses the variables but leaves a fresh env behind. A held borrow would
    // stay locked after the trap and fail every later call.
    let mut env = ENV.take();
    let output = match eval_to_val(input, &mut env) {
        Ok(val) => format_val(&val),
        Err(e) => format!("ERROR: {e}"),
    };
    ENV.set(env);
    output
}

#[wasm_bindgen]
pub fn reset_env() {
    ENV.with(|env| {
        *env.borrow_mut() = Env::new();
    });
}
