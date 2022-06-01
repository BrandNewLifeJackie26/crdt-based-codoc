mod wasm_rpc;

use crate::wasm_rpc::{
    wasm_service_client::WasmServiceClient, DeleteRequest, EndRequest, GetStringRequest,
    InsertRequest, RegisterRequest,
};
use tonic::transport::Channel;
use tonic_ws_transport::WsConnector;
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

static mut SERCH: Option<Channel> = None;

#[wasm_bindgen]
pub async fn register(client_id: u32, client_ip: String, doc_name: String) {
    const URL: &str = "ws://127.0.0.1:3001";
    let endpoint = tonic::transport::Endpoint::from_static(URL);
    let channel = endpoint
        .connect_with_connector(WsConnector::new())
        .await
        .expect("failed to connect");
    println!("Connected to {}", URL);

    unsafe {
        SERCH = Some(channel);
        if let Some(temp_ch) = &SERCH {
            let mut client = WasmServiceClient::new(temp_ch.clone());
            let request = tonic::Request::new(RegisterRequest {
                client_id: client_id,
                client_ip: client_ip,
                doc_name: doc_name,
            });

            let response = client.register(request).await.expect("RPC call failed");
            format!("{:?}", response);
        }
    }
}

#[wasm_bindgen]
pub async fn insert_update(client_id: u32, pos: u32, content: String) {
    unsafe {
        if let Some(temp_ch) = &SERCH {
            let mut client = WasmServiceClient::new(temp_ch.clone());
            let request = tonic::Request::new(InsertRequest {
                client_id: client_id,
                pos: pos,
                updates: content,
            });
            let response = client.insert(request).await.expect("RPC call failed");

            format!("{:?}", response);
        }
    }
}

#[wasm_bindgen]
pub async fn delete_update(client_id: u32, pos: u32, len: u32) {
    unsafe {
        if let Some(temp_ch) = &SERCH {
            let mut client = WasmServiceClient::new(temp_ch.clone());
            let request = tonic::Request::new(DeleteRequest {
                client_id: client_id,
                pos: pos,
                len: len,
            });
            let response = client.delete(request).await.expect("RPC call failed");

            format!("{:?}", response);
        }
    }
}

#[wasm_bindgen]
pub async fn to_string(client_id: u32) -> String {
    let mut res = String::new();
    unsafe {
        if let Some(temp_ch) = &SERCH {
            let mut client = WasmServiceClient::new(temp_ch.clone());
            let request = tonic::Request::new(GetStringRequest {
                client_id: client_id,
            });
            let response = client.get_string(request).await.expect("RPC call failed");
            res = response.into_inner().entire_doc;
            format!("{:?}", res);
        }
    }
    return res;
}

#[wasm_bindgen]
pub async fn sign_out(client_id: u32) {
    // in this function, shutdown all rpc services
    unsafe {
        if let Some(temp_ch) = &SERCH {
            let mut client = WasmServiceClient::new(temp_ch.clone());
            let request = tonic::Request::new(EndRequest {
                client_id: client_id,
            });
            let response = client.end(request).await.expect("RPC call failed");

            format!("{:?}", response);
        }
    }
}
