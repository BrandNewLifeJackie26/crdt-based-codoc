use core::fmt;
use std::collections::LinkedList;

const ID_START: u64 = u64::MIN;
const ID_END: u64 = u64::MAX;

struct UID {
    // id_list: Vec<i32>,
    id: u64,
}

struct ITEM_ID {
    id: Vec<u64>,
}

struct Content {
    content: String,
}

struct Item {
    uid: UID,
    item_id: ITEM_ID,
    clock: u64,
    origin_left: ITEM_ID,
    origin_right: ITEM_ID,
    isDeleted: bool,
    content: Content,
}

// Use YATA algorithm to represent a CRDT list
struct List {
    items: LinkedList<Item>,
}

impl List {
    pub fn new(uid: UID) -> List {
        let start_item = Item {
            uid: UID { id: uid.id },
            item_id: ITEM_ID { id: vec![ID_START] },
            clock: 0,
            origin_left: ITEM_ID { id: vec![ID_START] },
            origin_right: ITEM_ID { id: vec![ID_START] },
            isDeleted: false,
            content: Content {
                content: "".to_string(),
            },
        };
        let end_item = Item {
            uid: UID { id: uid.id },
            item_id: ITEM_ID { id: vec![ID_END] },
            clock: 0,
            origin_left: ITEM_ID { id: vec![ID_END] },
            origin_right: ITEM_ID { id: vec![ID_END] },
            isDeleted: false,
            content: Content {
                content: "".to_string(),
            },
        };

        let items = LinkedList::from([start_item, end_item]);
        List { items }
    }

    pub fn insert() {}

    pub fn delete() {}
}
