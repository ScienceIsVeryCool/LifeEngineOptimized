// renderer-wasm/src/utils.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

// A macro to provide `println!`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (crate::utils::log(&format_args!($($t)*).to_string()))
}

// A macro for `console.error` logging.
#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => (crate::utils::error(&format_args!($($t)*).to_string()))
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // If the feature is not enabled, we can still provide a basic panic handler
    #[cfg(not(feature = "console_error_panic_hook"))]
    std::panic::set_hook(Box::new(|info| {
        console_error!("Rust panic occurred: {:?}", info);
    }));
}