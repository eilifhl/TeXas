use uuid::Uuid;
use crate::crdt::text_crdt::timestamp::Timestamp;

// position is a lexicographically ordered path.
// a short path like [10] can be extended to [10, 500] when there is
// no free digit between two neighboring positions, e.g. [10] and [11].
#[derive(Clone, Debug)]
pub struct Position(Vec<PositionComponent>);

#[derive(Debug, Clone)]
pub struct PositionComponent {
    // the digit is where in the text we are.
    // If you want to insert between 10 and 20, you can choose 15:
    digit: u32,
    timestamp: Timestamp, // tie-breaker for concurrent inserts with the same digit
    replica_id: Uuid,
}
