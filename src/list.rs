use core::fmt;
use std::collections::LinkedList;

const ID_NONE: i32 = i32::MIN;
const ID_START: i32 = i32::MIN + 1;
const ID_END: i32 = i32::MAX;

struct UID {
    // id_list: Vec<i32>,
    id: i32,
}

struct Content {
    content: String,
}

struct Item {
    uid: UID,
    left: UID,
    right: UID,
    origin: UID,
    content: Content,
}

// Use YATA algorithm to represent a CRDT list
struct List {
    items: LinkedList<Item>,
}

impl List {
    pub fn new() -> List {
        let start_item = Item {
            uid: UID { id: ID_START},
            left: UID { id: ID_NONE },
            right: UID { id: ID_END },
            origin: UID {id: ID_NONE},
            content: Content { content: "".to_string() },
        };
        let end_item = Item {
            uid: UID { id: ID_END },
            left: UID { id: ID_START },
            right: UID { id: ID_NONE },
            origin: UID {id: ID_START},
            content: Content { content: "".to_string() },
        };

        let items = LinkedList::from([start_item, end_item]);
        List { items }
    }

    pub fn insert() {

    }

    pub fn delete() {
        
    }
}