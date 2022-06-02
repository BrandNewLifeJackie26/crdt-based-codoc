use core::time;
use crdt::block::Content;
use crdt::doc::Doc;
use crdt::sync_txn::SyncTransaction;
use crdt::utils::{serve_rpc, ClientID};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic_ws_transport::WsConnection;
use wasm_rpc::wasm_service_server::{WasmService, WasmServiceServer};

pub mod wasm_rpc {
    include!("wasm_rpc.rs");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:3001";

    let listener = TcpListener::bind(addr).await?;
    let listener_stream = TcpListenerStream::new(listener);
    let incoming = listener_stream.then(|connection| async {
        match connection {
            Ok(tcp_stream) => {
                let ws_stream = tokio_tungstenite::accept_async(tcp_stream).await.unwrap();
                Ok(WsConnection::from_combined_channel(ws_stream))
            }
            Err(e) => Err(e),
        }
    });

    let wasm_server = WasmRpcServer {
        txn_service: RwLock::new(HashMap::new()),
        rpc_shutdown_sender: Mutex::new(HashMap::new()),
    };

    println!("[wasm] server listening on {}", addr);

    Server::builder()
        .add_service(WasmServiceServer::new(wasm_server))
        .serve_with_incoming(incoming)
        .await?;

    Ok(())
}

pub struct WasmRpcServer {
    txn_service: RwLock<HashMap<u32, SyncTransaction>>,
    rpc_shutdown_sender: Mutex<HashMap<u32, Sender<()>>>,
}

// implement rpc interface
impl WasmRpcServer {
    fn start_helper(
        &self,
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

        return (txn_service, txn_rpc, txn_background);
    }
}

#[async_trait::async_trait]
impl WasmService for WasmRpcServer {
    async fn register(
        &self,
        request: tonic::Request<wasm_rpc::RegisterRequest>,
    ) -> Result<tonic::Response<wasm_rpc::Response>, tonic::Status> {
        let temp_request = request.into_inner();
        let client_id = temp_request.client_id;
        let client_ip = temp_request.client_ip;
        let doc_name = temp_request.doc_name;

        let doc = Arc::new(Mutex::new(Doc::new(
            doc_name.to_string(),
            client_id.clone(),
        )));
        let (sender, receiver): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender, mut init_receiver): (Sender<()>, Receiver<()>) = channel(1);

        let (txn_service, txn_rpc, txn_bg) =
            self.start_helper(doc_name, client_id.clone(), client_ip.clone(), doc);
        tokio::spawn(async move {
            serve_rpc(txn_rpc, txn_bg, receiver, init_sender).await;
        });
        let _ = init_receiver.recv().await;
        println!("[wasm] crdt rpc service started");

        let mut temp1 = self.txn_service.write().await;
        temp1.insert(client_id.clone(), txn_service);
        let mut temp2 = self.rpc_shutdown_sender.lock().await;
        temp2.insert(client_id.clone(), sender);
        println!("[wasm] successfully started crdt rpc");

        if let Some(service) = temp1.get(&client_id.clone()) {
            let reg_succ = service.register().await;
            match reg_succ {
                true => println!("[wasm] successfully returned zk register"),
                false => println!("[wasm] failed in zk register"),
            }
        }
        return Ok(tonic::Response::new(wasm_rpc::Response { succ: true }));
    }

    async fn insert(
        &self,
        request: tonic::Request<wasm_rpc::InsertRequest>,
    ) -> Result<tonic::Response<wasm_rpc::Response>, tonic::Status> {
        let temp_request = request.into_inner();
        let client_id = temp_request.client_id;
        let pos = temp_request.pos;
        let content_str = temp_request.updates;
        println!(
            "[wasm] crdt insert request received {:?} at {:?}",
            content_str, pos
        );
        let temp = self.txn_service.read().await;
        let service = temp.get(&client_id.clone());
        if let Some(service) = service.clone() {
            let mut doc = service.doc.lock().await;
            doc.insert_local(
                Content {
                    content: content_str,
                },
                pos,
            )
            .await;
        }
        return Ok(tonic::Response::new(wasm_rpc::Response { succ: true }));
    }

    async fn delete(
        &self,
        request: tonic::Request<wasm_rpc::DeleteRequest>,
    ) -> Result<tonic::Response<wasm_rpc::Response>, tonic::Status> {
        let temp_request = request.into_inner();
        let client_id = temp_request.client_id;
        let pos = temp_request.pos;
        let len = temp_request.len;
        println!(
            "[wasm] crdt delete request received, pos={:?}, len={:?}",
            pos, len
        );

        let temp = self.txn_service.read().await;
        let service = temp.get(&client_id.clone());
        if let Some(service) = service {
            let mut doc = service.doc.lock().await;
            doc.delete_local(pos, len).await;
        }
        return Ok(tonic::Response::new(wasm_rpc::Response { succ: true }));
    }

    async fn get_string(
        &self,
        request: tonic::Request<wasm_rpc::GetStringRequest>,
    ) -> Result<tonic::Response<wasm_rpc::GetStringResponse>, tonic::Status> {
        println!("[wasm] crdt get current content request received");
        let temp_request = request.into_inner();
        let client_id = temp_request.client_id;

        let temp = self.txn_service.read().await;
        let service = temp.get(&client_id.clone());
        let mut res = String::new();
        if let Some(service) = service {
            res = service.get_content().await;
        }
        println!("[wasm] entire content is {:?}", res);
        return Ok(tonic::Response::new(wasm_rpc::GetStringResponse {
            entire_doc: res,
        }));
    }

    async fn end(
        &self,
        request: tonic::Request<wasm_rpc::EndRequest>,
    ) -> Result<tonic::Response<wasm_rpc::Response>, tonic::Status> {
        let temp_request = request.into_inner();
        let client_id = temp_request.client_id;

        // wait for all operation to finish
        let wait = time::Duration::from_secs(1);
        thread::sleep(wait);

        // shutdown rpc service
        let temp = self.rpc_shutdown_sender.lock().await;
        let copy = (*temp).clone();
        if let Some(sender) = copy.get(&client_id.clone()) {
            let _ = sender.clone().send(()).await;
        }

        // wait for rpc service to shutdown
        let wait = time::Duration::from_secs(1);
        thread::sleep(wait);
        return Ok(tonic::Response::new(wasm_rpc::Response { succ: true }));
    }
}
