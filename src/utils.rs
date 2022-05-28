use serde::{Deserialize, Serialize};
use std::net::ToSocketAddrs;
use std::{thread, time};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{
    block::Block, sync_transaction::SyncTransaction, txn_rpc::txn_service_server::TxnServiceServer,
};
use std::{error::Error, fmt::Display};

pub type CRDTResult<T> = Result<T, Box<(dyn Error + Send + Sync)>>;

/// basic error types that can occur when running the tribbler service.
#[derive(Debug, Clone)]
pub enum CRDTError {
    RegisterUserFailed(),
    Unknown(String),
    ZKCreateZnodeFailed(String),
}

impl Display for CRDTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            CRDTError::RegisterUserFailed() => format!("cannot register user"),
            CRDTError::Unknown(x) => format!("unknown error: {}", x),
            CRDTError::ZKCreateZnodeFailed(x) => format!("failed to create path: {}", x),
        };
        write!(f, "{}", x)
    }
}

impl std::error::Error for CRDTError {}

// general type
pub type ClientID = u32;

pub type Updates = Vec<Block>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Peer {
    pub client_id: ClientID,
    pub ip_addr: String,
}

impl Peer {}

// start rpc service
pub async fn serve_rpc(txn: SyncTransaction, mut receiver: Receiver<()>, sender: Sender<()>) {
    let ip = txn.client_ip.clone();
    println!("starting rpc at {:?}", ip);
    let doc_name = txn.doc_name.clone();
    // let (sender_r, mut receiver_r): (Sender<()>, Receiver<()>) = channel(1);

    // tokio::spawn(async move {
    //     println!("background syncing process start");
    //     let _ = receiver_r.recv().await;
    //     txn.zk.background_sync(doc_name).await;
    // });

    let txn_rpc = TxnServiceServer::new(txn);
    let server = tonic::transport::Server::builder().add_service(txn_rpc);
    let resolved_addr_res = ip.to_socket_addrs().unwrap().next();

    if let Some(resolved_addr) = resolved_addr_res {
        let res = server
            .serve_with_shutdown(resolved_addr, async move {
                let _ = sender.send(()).await;
                // let _ = sender_r.send(()).await;
                receiver.recv().await;
                println!("successfully shut down txn rpc service");
            })
            .await;
        if let Err(e) = res {
            println!("failed to start rpc service {:?}", e);
        }
    } else {
        println!("cannot resolve ip address");
    }
}
