pub mod block;
pub mod block_store;
pub mod doc;
pub mod list;
pub mod sync_transaction;
pub mod txn_rpc;
pub mod utils;
pub mod zk_conn;
pub use crate::block::Block;
pub use crate::block::BlockID;
pub use crate::block_store::BlockStore;

#[cfg(test)]
mod local_tests {
    use crate::block::Content;
    use crate::doc::Doc;
    use crate::utils::ClientID;

    // Local insert to a single doc (one letter at a time),
    // there is no need to use transaction if no sync is needed
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn local_insert_single() {
        let cid = 1 as ClientID;
        let mut doc = Doc::new("text".to_string(), cid);

        doc.insert_local(
            Content {
                content: "1".to_string(),
            },
            0,
        )
        .await;
        assert_eq!(doc.to_string().await, "1".to_string());

        doc.insert_local(
            Content {
                content: "2".to_string(),
            },
            1,
        )
        .await;
        assert_eq!(doc.to_string().await, "12".to_string());

        // Insert pos is larger than length
        doc.insert_local(
            Content {
                content: "3".to_string(),
            },
            10,
        )
        .await;
        assert_eq!(doc.to_string().await, "123".to_string());

        doc.insert_local(
            Content {
                content: "4".to_string(),
            },
            1,
        )
        .await;
        assert_eq!(doc.to_string().await, "1423".to_string());
    }

    // Local insert to a single doc, (may insert more than one letter at a time)
    // there is no need to use transaction if no sync is needed
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn local_insert_multiple() {
        let cid = 1 as ClientID;
        let mut doc = Doc::new("text".to_string(), cid);

        doc.insert_local(
            Content {
                content: "123".to_string(),
            },
            0,
        )
        .await;
        assert_eq!(doc.to_string().await, "123".to_string());

        doc.insert_local(
            Content {
                content: "45".to_string(),
            },
            1,
        )
        .await;
        assert_eq!(doc.to_string().await, "14523".to_string());

        // Insert pos is larger than length
        doc.insert_local(
            Content {
                content: "6".to_string(),
            },
            10,
        )
        .await;
        assert_eq!(doc.to_string().await, "145236".to_string());

        doc.insert_local(
            Content {
                content: "789".to_string(),
            },
            4,
        )
        .await;
        assert_eq!(doc.to_string().await, "145278936".to_string());
    }
}

#[cfg(test)]
mod zk_test {
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::doc::Doc;
    use crate::sync_transaction::SyncTransaction;
    use crate::utils::{serve_rpc, ClientID};
    use std::{thread, time};
    use tokio::sync::mpsc::channel;
    use tokio::sync::mpsc::{Receiver, Sender};
    use tokio::sync::Mutex;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn zk_register_test() {
        // init channels used for rpc and user service communication
        let (sender1, receiver1): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender1, mut init_receiver1): (Sender<()>, Receiver<()>) = channel(1);
        let (sender2, receiver2): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender2, mut init_receiver2): (Sender<()>, Receiver<()>) = channel(1);

        let (txn_rpc1, txn_service1) =
            init_txn_w_rpc("doc".to_string(), 1, "127.0.0.1:4001".to_string()).await;
        let (txn_rpc2, txn_service2) =
            init_txn_w_rpc("doc".to_string(), 2, "127.0.0.1:4002".to_string()).await;

        // start rpc services
        tokio::spawn(async move {
            serve_rpc(txn_rpc1, receiver1, init_sender1).await;
        });
        tokio::spawn(async move {
            serve_rpc(txn_rpc2, receiver2, init_sender2).await;
        });

        // start user operation
        let _ = init_receiver1.recv().await;
        let _ = init_receiver2.recv().await;
        println!("receive init signal, rpc services successfully started");
        tokio::spawn(async move {
            let succ = txn_service1.register().await;
            assert_eq!(true, succ);
        });
        tokio::spawn(async move {
            let succ = txn_service2.register().await;
            assert_eq!(true, succ);
        });

        // wait for all operations to finish
        let wait = time::Duration::from_secs(2);
        thread::sleep(wait);

        // shutdown rpc service
        let _ = sender1.send(()).await;
        let _ = sender2.send(()).await;

        // wait for rpc to shutdown
        let wait = time::Duration::from_secs(2);
        thread::sleep(wait);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn zk_new_register_broadcast_test() {
        let (sender1, receiver1): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender1, mut init_receiver1): (Sender<()>, Receiver<()>) = channel(1);
        let (sender2, receiver2): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender2, mut init_receiver2): (Sender<()>, Receiver<()>) = channel(1);
        let (sender3, receiver3): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender3, mut init_receiver3): (Sender<()>, Receiver<()>) = channel(1);

        let (txn_rpc1, txn_service1) =
            init_txn_w_rpc("doc".to_string(), 1, "127.0.0.1:4001".to_string()).await;
        let (txn_rpc2, txn_service2) =
            init_txn_w_rpc("doc".to_string(), 2, "127.0.0.1:4002".to_string()).await;
        let (txn_rpc3, txn_service3) =
            init_txn_w_rpc("doc".to_string(), 3, "127.0.0.1:4003".to_string()).await;

        // start rpc services
        tokio::spawn(async move {
            serve_rpc(txn_rpc1, receiver1, init_sender1).await;
        });
        tokio::spawn(async move {
            serve_rpc(txn_rpc2, receiver2, init_sender2).await;
        });
        tokio::spawn(async move {
            serve_rpc(txn_rpc3, receiver3, init_sender3).await;
        });

        let _ = init_receiver1.recv().await;
        let _ = init_receiver2.recv().await;
        let _ = init_receiver3.recv().await;
        println!("receive init signal, rpc services successfully started");

        tokio::spawn(async move {
            let succ = txn_service1.register().await;
            assert_eq!(true, succ);
        });
        tokio::spawn(async move {
            let succ = txn_service2.register().await;
            assert_eq!(true, succ);
        });

        // delay the op, see if it got propagated
        let wait = time::Duration::from_secs(2);
        thread::sleep(wait);

        tokio::spawn(async move {
            let succ = txn_service3.register().await;
            assert_eq!(true, succ);
        });

        // wait for all operations to finish
        let wait = time::Duration::from_secs(2);
        thread::sleep(wait);

        // shutdown rpc service
        let _ = sender1.send(()).await;
        let _ = sender2.send(()).await;
        let _ = sender3.send(()).await;

        // wait for rpc to shutdown
        let wait = time::Duration::from_secs(2);
        thread::sleep(wait);
    }

    // create a transaction (one is rpc service, one is the real service that a client use)
    pub async fn init_txn_w_rpc(
        doc_name: String,
        client_id: ClientID,
        client_ip: String,
    ) -> (SyncTransaction, SyncTransaction) {
        let doc = Arc::new(Mutex::new(Doc::new(doc_name.to_string(), client_id)));
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
        return (txn_rpc, txn_service);
    }
}
