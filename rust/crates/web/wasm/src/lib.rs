use wasm_bindgen::prelude::*;

/// Adds two numbers so it can be called from JS once compiled to WebAssembly.
#[wasm_bindgen]
pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

/// Tiny example that returns a message to JS callers.
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(add(2, 2), 4);
        assert_eq!(greet("Satty"), "Hello, Satty!");
    }
}
