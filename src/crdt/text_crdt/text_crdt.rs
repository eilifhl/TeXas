use std::collections::HashSet;
use uuid::{Timestamp, Uuid};

struct TextCrdt {
    replica_id: Uuid,
    clock: u64, // to make the IDs, see `ElementId` struct
    elements: Vec<TextElement>, // contains the text elements that create the text
    seen_operations: HashSet<OperationId>,
}

struct TextElement {
    id: ElementId,
    position: Position,
    value: char, // the text char itself
    deleted: bool,
}

// if replica_id = 'A'
// Insert 'H' -> clock becomes 1 -> element id = (A, 1)
// Insert 'i' -> clock becomes 2 -> element id = (A, 2)
struct ElementId {
    replica_id: Uuid,
    counter: u64,
}

struct OperationId {
    replica_id: Uuid,
    counter: u64,
}

enum TextOperation {
    Insert {
        op_id: OperationId,
        element_id: ElementId,
        position: Position,
        value: char,
    },
    Delete { // set element to tombstone
        op_id: OperationId,
        element_id: ElementId,
    },
}

// position is a lexicographically ordered path.
// a short path like [10] can be extended to [10, 500] when there is
// no free digit between two neighboring positions, e.g. [10] and [11].
struct Position(Vec<PositionComponent>);

struct PositionComponent {
    // the digit is where in the text we are.
    // If you want to insert between 10 and 20, you can choose 15:
    digit: u32,
    timestamp: Timestamp, // for tiebreaks: LWW. maybe use another datatype for it
}
