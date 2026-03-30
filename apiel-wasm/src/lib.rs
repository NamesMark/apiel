use apiel::Env;
use apiel::parse::{eval_to_val, format_val};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static ENV: RefCell<Env> = RefCell::new(Env::new());
}

#[wasm_bindgen]
pub fn eval_apl(input: &str) -> String {
    ENV.with(|env| {
        let mut env = env.borrow_mut();
        match eval_to_val(input, &mut env) {
            Ok(val) => format_val(&val),
            Err(e) => format!("ERROR: {e}"),
        }
    })
}

#[wasm_bindgen]
pub fn reset_env() {
    ENV.with(|env| {
        *env.borrow_mut() = Env::new();
    });
}
