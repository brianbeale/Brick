use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[test]
fn rust_test() {
    assert!(true);
}

#[wasm_bindgen_test]
fn web_test() {
    assert!(true);
}
