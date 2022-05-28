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
    use crate::utils::{serve_rpc, ClientID};

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

    // Local insert and delete the whole string
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn local_delete_all() {
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

        // Remove a whole block
        doc.delete_local(0, 3).await;
        assert_eq!(doc.to_string().await, "".to_string());
    }

    // Local insert and delete from beginning and end of a block
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn local_delete_block_beg_end() {
        let cid = 1 as ClientID;
        let mut doc = Doc::new("text".to_string(), cid);

        doc.insert_local(
            Content {
                content: "12345".to_string(),
            },
            0,
        )
        .await;
        assert_eq!(doc.to_string().await, "12345".to_string());

        // Delete part of a block from the start
        doc.delete_local(0, 3).await;
        assert_eq!(doc.to_string().await, "45".to_string());

        // Delete part of a block from then end
        doc.delete_local(2, 1).await;
        assert_eq!(doc.to_string().await, "4".to_string());
    }

    // Local insert and delete across multiple blocks
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn local_delete_across_blocks() {
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
                content: "456".to_string(),
            },
            3,
        )
        .await;
        assert_eq!(doc.to_string().await, "123456".to_string());

        // Delete part of a block from the start
        doc.delete_local(2, 2).await;
        assert_eq!(doc.to_string().await, "1256".to_string());
    }
}

#[cfg(test)]
mod zk_test {
    use crate::block::Content;
    use crate::doc::Doc;
    use crate::sync_transaction::SyncTransaction;
    use crate::utils::{serve_rpc, ClientID};
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::{thread, time};
    use tokio::sync::mpsc::channel;
    use tokio::sync::mpsc::{Receiver, Sender};
    use tokio::sync::Mutex;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn zk_register_test() {
        let client_id1 = 1;
        let client_id2 = 2;
        let doc_name = "doc";

        let (sender1, mut receiver1): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender1, mut init_receiver1): (Sender<()>, Receiver<()>) = channel(1);
        let (sender2, mut receiver2): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender2, mut init_receiver2): (Sender<()>, Receiver<()>) = channel(1);

        let doc1 = Arc::new(Mutex::new(Doc::new(doc_name.to_string(), client_id1)));
        let doc2 = Arc::new(Mutex::new(Doc::new(doc_name.to_string(), client_id2)));
        let chan1 = Arc::new(Mutex::new(HashMap::new()));
        let chan2 = Arc::new(Mutex::new(HashMap::new()));

        let txn11 = SyncTransaction::new(
            client_id1,
            doc1.clone(),
            chan1.clone(),
            "127.0.0.1:4001".to_string(),
        );
        let txn22 = SyncTransaction::new(
            client_id2,
            doc2.clone(),
            chan2.clone(),
            "127.0.0.1:4002".to_string(),
        );

        tokio::spawn(async move {
            let _ = init_receiver1.recv().await;
            let _ = init_receiver2.recv().await;
            println!("receive init signal, rpc services successfully started");
            let txn1 = SyncTransaction::new(client_id1, doc1, chan1, "127.0.0.1:4001".to_string());
            let txn2 = SyncTransaction::new(client_id2, doc2, chan2, "127.0.0.1:4002".to_string());
            let succ = txn1.register().await;
            assert_eq!(true, succ);
            let succ = txn2.register().await;
            assert_eq!(true, succ);

            // shutdown rpc service
            let _ = sender1.send(()).await;
            let _ = sender2.send(()).await;
        });

        tokio::spawn(async move {
            serve_rpc(txn11, receiver1, init_sender1).await;
        });

        serve_rpc(txn22, receiver2, init_sender2).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn zk_same_user_test() {
        let client_id1 = 1;
        let doc_name = "doc";

        let (sender1, mut receiver1): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender1, mut init_receiver1): (Sender<()>, Receiver<()>) = channel(1);
        let (sender2, mut receiver2): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender2, mut init_receiver2): (Sender<()>, Receiver<()>) = channel(1);

        let doc1 = Arc::new(Mutex::new(Doc::new(doc_name.to_string(), client_id1)));
        let chan1 = Arc::new(Mutex::new(HashMap::new()));

        let txn1 = SyncTransaction::new(
            client_id1,
            doc1.clone(),
            chan1.clone(),
            "127.0.0.1:4001".to_string(),
        );
        let txn11 = SyncTransaction::new(
            client_id1,
            doc1.clone(),
            chan1.clone(),
            "127.0.0.1:4001".to_string(),
        );

        // for the second access
        let txn22 = SyncTransaction::new(
            client_id1,
            doc1.clone(),
            chan1.clone(),
            "127.0.0.1:4001".to_string(),
        );

        tokio::spawn(async move {
            let _ = init_receiver1.recv().await;
            println!("receive init signal, rpc services successfully started");
            let succ = txn1.register().await;
            assert_eq!(true, succ);

            // shutdown rpc service
            let _ = sender1.send(()).await;
        });

        tokio::spawn(async move {
            serve_rpc(txn11, receiver1, init_sender1).await;
        });

        let wait = time::Duration::from_secs(2);
        thread::sleep(wait);

        // same user reaccess the same file
        tokio::spawn(async move {
            let _ = init_receiver2.recv().await;
            // shutdown rpc service
            let _ = sender2.send(()).await;
        });

        serve_rpc(txn22, receiver2, init_sender2).await;
    }

    // TODO: check current list of clients?
    // is it possible to make sure the order of registration?
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn zk_new_register_broadcast_test() {
        let client_id1 = 1;
        let client_id2 = 2;
        let client_id3 = 3;
        let doc_name = "doc";

        let (sender1, mut receiver1): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender1, mut init_receiver1): (Sender<()>, Receiver<()>) = channel(1);
        let (sender2, mut receiver2): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender2, mut init_receiver2): (Sender<()>, Receiver<()>) = channel(1);
        let (sender3, mut receiver3): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender3, mut init_receiver3): (Sender<()>, Receiver<()>) = channel(1);

        let doc1 = Arc::new(Mutex::new(Doc::new(doc_name.to_string(), client_id1)));
        let doc2 = Arc::new(Mutex::new(Doc::new(doc_name.to_string(), client_id2)));
        let doc3 = Arc::new(Mutex::new(Doc::new(doc_name.to_string(), client_id3)));
        let chan1 = Arc::new(Mutex::new(HashMap::new()));
        let chan2 = Arc::new(Mutex::new(HashMap::new()));
        let chan3 = Arc::new(Mutex::new(HashMap::new()));

        let txn11 = SyncTransaction::new(
            client_id1,
            doc1.clone(),
            chan1.clone(),
            "127.0.0.1:4001".to_string(),
        );
        let txn22 = SyncTransaction::new(
            client_id2,
            doc2.clone(),
            chan2.clone(),
            "127.0.0.1:4002".to_string(),
        );
        let txn33 = SyncTransaction::new(
            client_id3,
            doc3.clone(),
            chan2.clone(),
            "127.0.0.1:4003".to_string(),
        );

        tokio::spawn(async move {
            let _ = init_receiver1.recv().await;
            println!("receive init signal, rpc services successfully started");
            let txn1 = SyncTransaction::new(client_id1, doc1, chan1, "127.0.0.1:4001".to_string());
            let succ = txn1.register().await;
            assert_eq!(true, succ);

            // shutdown rpc service
            let _ = sender1.send(()).await;
        });

        tokio::spawn(async move {
            let _ = init_receiver2.recv().await;
            println!("receive init signal, rpc services successfully started");
            let txn2 = SyncTransaction::new(client_id2, doc2, chan2, "127.0.0.1:4002".to_string());
            let succ = txn2.register().await;
            assert_eq!(true, succ);

            // shutdown rpc service
            let _ = sender2.send(()).await;
        });

        tokio::spawn(async move {
            let _ = init_receiver3.recv().await;
            println!("receive init signal, rpc services successfully started");
            let txn3 = SyncTransaction::new(client_id3, doc3, chan3, "127.0.0.1:4003".to_string());
            let succ = txn3.register().await;
            assert_eq!(true, succ);

            // shutdown rpc service
            let _ = sender3.send(()).await;
        });

        tokio::spawn(async move {
            serve_rpc(txn11, receiver1, init_sender1).await;
        });

        tokio::spawn(async move {
            serve_rpc(txn22, receiver2, init_sender2).await;
        });

        serve_rpc(txn33, receiver3, init_sender3).await;
    }
}
