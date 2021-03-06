pub mod crdt;
pub mod wasm;

#[cfg(test)]
mod local_tests {
    use crate::crdt::block::Content;
    use crate::crdt::doc::Doc;
    use crate::crdt::utils::ClientID;

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

        // Delete part of a block till the end
        doc.delete_local(1, 1).await;
        assert_eq!(doc.to_string().await, "4".to_string());

        // Deleting some charater exceeding the whole doc should take no effect
        doc.delete_local(2, 3).await;
        assert_eq!(doc.to_string().await, "4".to_string());

        // Deleting characters out of bound should take no effect either
        doc.insert_local(
            Content {
                content: "567".to_string(),
            },
            1,
        )
        .await;
        assert_eq!(doc.to_string().await, "4567".to_string());
        doc.delete_local(2, 10).await;
        assert_eq!(doc.to_string().await, "45".to_string());
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
mod remote_test {
    use crate::crdt::block::Block;
    use crate::crdt::block::BlockID;
    use crate::crdt::block::Content;
    use crate::crdt::doc::Doc;
    use crate::crdt::utils::ClientID;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn remote_insert_none() {
        let cid = 1 as ClientID;
        let mut doc1 = Doc::new("text".to_string(), cid);

        doc1.insert_local(
            Content {
                content: "1234567".to_string(),
            },
            0,
        )
        .await;

        let mut updates = vec![];
        let new_block: Block = Block {
            id: BlockID {
                client: 2,
                clock: 100,
            },
            left_origin: None,
            right_origin: None,
            is_deleted: false,
            content: Content {
                content: "NEW2".to_string(),
            },
        };
        updates.push(new_block);

        doc1.insert_remote(updates).await;

        assert_eq!(doc1.to_string().await, "1234567NEW2".to_string());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn remote_insert_split() {
        let cid = 1 as ClientID;
        let mut doc1 = Doc::new("text".to_string(), cid);

        doc1.insert_local(
            Content {
                content: "1234567".to_string(),
            },
            0,
        )
        .await;
        let store = doc1.block_store.clone();
        let store_lock = store.lock().await;
        let id = store_lock.kv_store.get(&1).unwrap().list[0]
            .clone()
            .lock()
            .await
            .id
            .clone();
        drop(store_lock);

        let mut updates = vec![];
        let new_block: Block = Block {
            id: BlockID {
                client: 2,
                clock: 100,
            },
            left_origin: Some(id.clone()),
            right_origin: Some(BlockID {
                client: id.client,
                clock: id.clock + 2,
            }),
            is_deleted: false,
            content: Content {
                content: "NEW2".to_string(),
            },
        };
        updates.push(new_block);

        doc1.insert_remote(updates).await;

        assert_eq!(doc1.to_string().await, "12NEW234567".to_string());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn remote_insert_split_with_squash() {
        let cid = 1 as ClientID;
        let mut doc1 = Doc::new("text".to_string(), cid);
        doc1.insert_local(
            Content {
                content: "1234567".to_string(),
            },
            0,
        )
        .await;
        doc1.insert_local(
            Content {
                content: "aabbccdd".to_string(),
            },
            7,
        )
        .await;
        assert_eq!(doc1.to_string().await, "1234567aabbccdd");

        let cid2 = 2 as ClientID;
        let mut doc2 = Doc::new("text".to_string(), cid2);
        let mut updates_1_to_2: Vec<Block> = vec![];
        let new_block: Block = Block {
            id: BlockID {
                client: cid,
                clock: 0,
            },
            left_origin: None,
            right_origin: None,
            is_deleted: false,
            content: Content {
                content: "1234567aabbccdd".to_string(),
            },
        };
        updates_1_to_2.push(new_block);
        doc2.insert_remote(updates_1_to_2).await;
        assert_eq!(doc2.to_string().await, "1234567aabbccdd");

        doc2.insert_local(
            Content {
                content: "NEW2".to_string(),
            },
            7,
        )
        .await;
        assert_eq!(doc2.to_string().await, "1234567NEW2aabbccdd");

        let mut updates_2_to_1: Vec<Block> = vec![];
        let new_block: Block = Block {
            id: BlockID {
                client: cid2,
                clock: 0,
            },
            left_origin: Some(BlockID {
                client: cid,
                clock: 0,
            }),
            right_origin: Some(BlockID {
                client: cid,
                clock: 7,
            }),
            is_deleted: false,
            content: Content {
                content: "NEW2".to_string(),
            },
        };
        updates_2_to_1.push(new_block);
        doc1.insert_remote(updates_2_to_1).await;
        assert_eq!(doc1.to_string().await, "1234567NEW2aabbccdd".to_string());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn remote_merge_conflict_insert() {
        let cid = 1 as ClientID;
        let mut doc1 = Doc::new("text".to_string(), cid);

        doc1.insert_local(
            Content {
                content: "1234567".to_string(),
            },
            0,
        )
        .await;
        let store = doc1.block_store.clone();
        let store_lock = store.lock().await;
        let left = store_lock.kv_store.get(&1).unwrap().list[0]
            .clone()
            .lock()
            .await
            .id
            .clone();
        drop(store_lock);

        let mut updates = vec![];
        let new_block: Block = Block {
            id: BlockID {
                client: 2,
                clock: 100,
            },
            left_origin: Some(left.clone()),
            right_origin: None,
            is_deleted: false,
            content: Content {
                content: "NEW2".to_string(),
            },
        };
        updates.push(new_block);
        doc1.insert_remote(updates).await;

        let mut updates = vec![];
        let new_block: Block = Block {
            id: BlockID {
                client: 14,
                clock: 21,
            },
            left_origin: Some(left),
            right_origin: None,
            is_deleted: false,
            content: Content {
                content: "FROM14".to_string(),
            },
        };
        updates.push(new_block);

        doc1.insert_remote(updates).await;

        assert_eq!(doc1.to_string().await, "1234567NEW2FROM14".to_string());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn remote_delete_reverse_order() {
        let cid = 1 as ClientID;
        let mut doc1 = Doc::new("text".to_string(), cid);

        doc1.insert_local(
            Content {
                content: "1234567".to_string(),
            },
            0,
        )
        .await;
        doc1.insert_local(
            Content {
                content: "aabbccdd".to_string(),
            },
            7,
        )
        .await;
        let left = BlockID {
            client: cid,
            clock: 0,
        };
        let right = BlockID {
            client: cid,
            clock: 7,
        };

        let mut updates = vec![];
        let new_block: Block = Block {
            id: BlockID {
                client: 14,
                clock: 21,
            },
            left_origin: Some(left.clone()),
            right_origin: Some(right.clone()),
            is_deleted: true,
            content: Content {
                content: "NEW2".to_string(),
            },
        };
        updates.push(new_block);
        doc1.delete_remote(updates).await;

        let mut updates = vec![];
        let new_block: Block = Block {
            id: BlockID {
                client: 14,
                clock: 21,
            },
            left_origin: Some(left),
            right_origin: Some(right),
            is_deleted: false,
            content: Content {
                content: "NEW2".to_string(),
            },
        };
        updates.push(new_block);

        doc1.insert_remote(updates).await;

        assert_eq!(doc1.to_string().await, "1234567aabbccdd".to_string());
    }
}

#[cfg(test)]
mod zk_test {
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::crdt::doc::Doc;
    use crate::crdt::sync_txn::SyncTransaction;
    use crate::crdt::utils::{serve_rpc, ClientID};
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

        let (txn_rpc1, txn_service1, txn_bg1) =
            init_txn_w_rpc("doc".to_string(), 1, "127.0.0.1:4001".to_string()).await;
        let (txn_rpc2, txn_service2, txn_bg2) =
            init_txn_w_rpc("doc".to_string(), 2, "127.0.0.1:4002".to_string()).await;

        // start rpc services
        tokio::spawn(async move {
            serve_rpc(txn_rpc1, txn_bg1, receiver1, init_sender1).await;
        });
        let wait = time::Duration::from_secs(2);
        thread::sleep(wait);
        tokio::spawn(async move {
            serve_rpc(txn_rpc2, txn_bg2, receiver2, init_sender2).await;
        });

        // start user operation
        let _ = init_receiver1.recv().await;
        let _ = init_receiver2.recv().await;

        println!("----------- start op --------------");
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

    // is it possible to make sure the order of registration?
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn zk_register_test_more_nodes() {
        let (sender1, receiver1): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender1, mut init_receiver1): (Sender<()>, Receiver<()>) = channel(1);
        let (sender2, receiver2): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender2, mut init_receiver2): (Sender<()>, Receiver<()>) = channel(1);
        let (sender3, receiver3): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender3, mut init_receiver3): (Sender<()>, Receiver<()>) = channel(1);
        let (sender4, receiver4): (Sender<()>, Receiver<()>) = channel(1);
        let (init_sender4, mut init_receiver4): (Sender<()>, Receiver<()>) = channel(1);

        let (txn_rpc1, txn_service1, txn_bg1) =
            init_txn_w_rpc("doc".to_string(), 1, "127.0.0.1:4001".to_string()).await;
        let (txn_rpc2, txn_service2, txn_bg2) =
            init_txn_w_rpc("doc".to_string(), 2, "127.0.0.1:4002".to_string()).await;
        let (txn_rpc3, txn_service3, txn_bg3) =
            init_txn_w_rpc("doc".to_string(), 3, "127.0.0.1:4003".to_string()).await;
        let (txn_rpc4, txn_service4, txn_bg4) =
            init_txn_w_rpc("doc".to_string(), 4, "127.0.0.1:4004".to_string()).await;

        // start 1 and 2
        tokio::spawn(async move {
            serve_rpc(txn_rpc1, txn_bg1, receiver1, init_sender1).await;
        });
        tokio::spawn(async move {
            serve_rpc(txn_rpc2, txn_bg2, receiver2, init_sender2).await;
        });

        let _ = init_receiver1.recv().await;
        let _ = init_receiver2.recv().await;
        tokio::spawn(async move {
            let succ = txn_service1.register().await;
            assert_eq!(true, succ);
        });
        tokio::spawn(async move {
            let succ = txn_service2.register().await;
            assert_eq!(true, succ);
        });

        // start 3 and 4
        tokio::spawn(async move {
            serve_rpc(txn_rpc3, txn_bg3, receiver3, init_sender3).await;
        });
        tokio::spawn(async move {
            serve_rpc(txn_rpc4, txn_bg4, receiver4, init_sender4).await;
        });

        let _ = init_receiver3.recv().await;
        let _ = init_receiver4.recv().await;

        // delay the op, see if it got propagated
        let wait = time::Duration::from_secs(2);
        thread::sleep(wait);

        // delay the op, see if it got propagated
        let wait = time::Duration::from_secs(2);
        thread::sleep(wait);

        tokio::spawn(async move {
            let succ = txn_service3.register().await;
            assert_eq!(true, succ);
        });
        tokio::spawn(async move {
            let succ = txn_service4.register().await;
            assert_eq!(true, succ);
        });

        // wait for all operations to finish
        let wait = time::Duration::from_secs(2);
        thread::sleep(wait);

        // shutdown rpc service
        let _ = sender1.send(()).await;
        let _ = sender2.send(()).await;
        let _ = sender3.send(()).await;
        let _ = sender4.send(()).await;

        // wait for rpc to shutdown
        let wait = time::Duration::from_secs(2);
        thread::sleep(wait);
    }

    // create a transaction (one is rpc service, one is the real service that a client use)
    pub async fn init_txn_w_rpc(
        doc_name: String,
        client_id: ClientID,
        client_ip: String,
    ) -> (SyncTransaction, SyncTransaction, SyncTransaction) {
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
        let txn_background = SyncTransaction::new(
            doc_name.clone(),
            client_id.clone(),
            doc.clone(),
            chan.clone(),
            client_ip.clone(),
        );
        return (txn_rpc, txn_service, txn_background);
    }
}

#[cfg(test)]
mod perf_tests {
    use crate::crdt::block::Content;
    use crate::crdt::doc::Doc;
    use crate::crdt::utils::ClientID;
    use jemalloc_ctl::{epoch, stats};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    extern crate jemallocator;

    #[global_allocator]
    static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn local_random_insert_perf_check() {
        let iterations = 10;
        for _ in 0..iterations {
            local_random_insert_perf_check_once().await;
        }
    }

    // Local insert to a single doc,
    // profiling memory v.s. insertion patterns
    // #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn local_random_insert_perf_check_once() {
        let e = epoch::mib().unwrap();
        let allocated = stats::allocated::mib().unwrap();
        let prev_mem = allocated.read().unwrap();
        println!("Total allocated memory before running: {}", prev_mem);

        let cid = 1 as ClientID;
        let mut doc = Doc::new("text".to_string(), cid);

        // A ref string and a doc
        // Randomly pick a position inside the string and insert random content
        let mut ref_string = "".to_string();
        let insertions: usize = 10;
        let max_ins_len: usize = 20;
        for _ in 0..insertions {
            let rand_ins_len = thread_rng().gen_range(0..max_ins_len);
            let rand_pos = thread_rng().gen_range(0..(ref_string.len() + 1));
            let rand_string: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(rand_ins_len)
                .map(char::from)
                .collect();

            doc.insert_local(
                Content {
                    content: rand_string.clone(),
                },
                rand_pos as u32,
            )
            .await;
            ref_string.insert_str(rand_pos, &rand_string);
            assert_eq!(doc.to_string().await, ref_string);
        }

        e.advance().unwrap();
        let post_mem = allocated.read().unwrap();
        println!("Total allocated memory: {}", post_mem);
        println!("Newly allocated memory: {}", post_mem - prev_mem);
    }
}
