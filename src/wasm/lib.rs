use crate::crdt::doc::Doc;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // import external javascript function
}

/** decalare wasm object */
#[wasm_bindgen]
pub struct WasmDoc(Doc);

#[wasm_bindgen]
impl WasmDoc {
    // creates a new wasm document.
    #[wasm_bindgen(constructor)]
    pub fn new(doc_name: String, client_id: u32) -> Self {
        // randomly generate a client id
        WasmDoc(Doc::new(doc_name, client_id))
    }
}
