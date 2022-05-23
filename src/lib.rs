pub mod list;

#[cfg(test)]
mod local_tests {
    use crate::list::*;

    #[test]
    fn local_insert() {
        let uid = UID { id: 1 };
        let text = List::new(uid.clone());

        let start_item_id = ItemID { id: ID_START };
        let end_item_id = ItemID { id: ID_END };

        let item_id1 = text.insert(
            start_item_id.clone(),
            end_item_id.clone(),
            Content {
                content: "1".to_string(),
            },
        );
        assert_eq!(text.to_string(), "1".to_string());

        let item_id2 = text.insert(
            start_item_id.clone(),
            item_id1.clone(),
            Content {
                content: "2".to_string(),
            },
        );
        assert_eq!(text.to_string(), "21".to_string());

        let item_id3 = text.insert(
            item_id1.clone(),
            end_item_id.clone(),
            Content {
                content: "3".to_string(),
            },
        );
        assert_eq!(text.to_string(), "213".to_string());

        let item_id4 = text.insert(
            item_id1.clone(),
            item_id3.clone(),
            Content {
                content: "4".to_string(),
            },
        );
        assert_eq!(text.to_string(), "2143".to_string());
    }
}
pub mod block;
pub mod block_store;
pub mod doc;
pub mod transaction;
pub use crate::block::Block;
pub use crate::block::BlockID;
pub use crate::block::ClientID;
pub use crate::block_store::BlockStore;
