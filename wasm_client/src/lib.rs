mod wasm_rpc;

use tonic_ws_transport::WsConnector;
use wasm_bindgen::prelude::*;

use crate::wasm_rpc::{wasm_service_client::WasmServiceClient, RegisterRequest};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // import external javascript function
}

#[wasm_bindgen]
pub async fn register(client_id: u32, client_ip: String, doc_name: String) -> String {
    const URL: &str = "ws://127.0.0.1:3001";
    let endpoint = tonic::transport::Endpoint::from_static(URL);
    let channel = endpoint
        .connect_with_connector(WsConnector::new())
        .await
        .expect("failed to connect");
    println!("Connected to {}", URL);

    let mut client = WasmServiceClient::new(channel);

    let request = tonic::Request::new(RegisterRequest {
        client_id: client_id,
        client_ip: client_ip,
        doc_name: doc_name,
    });
    println!("REQUEST={:?}", request);

    let response = client.register(request).await.expect("RPC call failed");
    println!("RESPONSE={:?}", response);

    format!("{:?}", response)
}

#[wasm_bindgen]
pub async fn insert_update(left_origin: u32, content: String) {}

#[wasm_bindgen]
pub async fn delete_update(left_origin: u32, content: String) {}

#[wasm_bindgen]
pub async fn sign_out() {
    // in this function, shutdown all rpc services
}
