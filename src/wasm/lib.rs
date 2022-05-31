use core::time;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use crate::crdt::doc::Doc;
use crate::crdt::sync_txn::SyncTransaction;
use crate::crdt::utils::{serve_rpc, ClientID};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
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

    #[wasm_bindgen(js_name = beginSession)]
    pub fn begin_session(&mut self) {}

    #[wasm_bindgen(js_name = insertUpdate)]
    pub fn insert_update() {
        // convert position and content to blocks

        // call on doc
    }

    #[wasm_bindgen(js_name = deleteUpdate)]
    pub fn delete_update() {}
}

#[wasm_bindgen]
pub struct WasmTransaction {
    sync_txn: SyncTransaction,
    stop_rpc_sender: Sender<()>,
}

impl WasmTransaction {
    async fn start_transcation(
        doc_name: String,
        client_id: ClientID,
        client_ip: String,
        doc: Arc<Mutex<Doc>>,
    ) -> Self {
        let (sender, receiver): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender, mut init_receiver): (Sender<()>, Receiver<()>) = channel(1);

        let (txn_rpc, txn_service, txn_bg) =
            WasmTransaction::start_helper(doc_name, client_id, client_ip, doc);
        tokio::spawn(async move {
            serve_rpc(txn_rpc, txn_bg, receiver, init_sender).await;
        });
        let _ = init_receiver.recv().await;

        return WasmTransaction {
            sync_txn: txn_service,
            stop_rpc_sender: sender,
        };
    }

    fn start_helper(
        doc_name: String,
        client_id: ClientID,
        client_ip: String,
        doc: Arc<Mutex<Doc>>,
    ) -> (SyncTransaction, SyncTransaction, SyncTransaction) {
        let chan = Arc::new(Mutex::new(HashMap::new()));
        let txn_rpc = SyncTransaction::new(
            doc_name.clone(),
            client_id.clone(),
            doc.clone(),
            chan.clone(),
            client_ip.clone(),
        );
        let txn_service = SyncTransaction::new(
            doc_name.clone(),
            client_id.clone(),
            doc.clone(),
            chan.clone(),
            client_ip.clone(),
        );
        let txn_background = SyncTransaction::new(
            doc_name.clone(),
            client_id.clone(),
            doc.clone(),
            chan.clone(),
            client_ip.clone(),
        );
        return (txn_rpc, txn_service, txn_background);
    }

    async fn end_transcation(&self) {
        // wait for all operation to finish
        let wait = time::Duration::from_secs(1);
        thread::sleep(wait);

        // shutdown rpc service
        let _ = self.stop_rpc_sender.send(()).await;

        // wait for rpc service to shutdown
        let wait = time::Duration::from_secs(1);
        thread::sleep(wait);
    }
}
