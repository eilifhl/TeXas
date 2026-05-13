mod state_based_crdt;

use state_based_crdt::{GCounter, PNCounter, StateBasedCrdt};
use uuid::Uuid;

fn main() {
    // Alt dette er midlertidig!!

    // G Counter
    let mut replica_1: GCounter = GCounter::new(Uuid::new_v4());
    let mut replica_2: GCounter = GCounter::new(Uuid::new_v4());

    replica_1.increment(1);
    replica_2.increment(2);

    println!("{}", replica_1.value());
    println!("{}", replica_2.value());
    replica_1.merge(&replica_2);
    replica_2.merge(&replica_1);
    println!("{}", replica_1.value());
    println!("{}", replica_2.value());

    // PN Counter
    let mut replica_1: PNCounter = PNCounter::new(Uuid::new_v4());
    let mut replica_2: PNCounter = PNCounter::new(Uuid::new_v4());

    replica_1.increment(1);
    replica_2.increment(2);
    replica_2.decrement(1);

    println!("{}", replica_1.value());
    println!("{}", replica_2.value());
    replica_1.merge(&replica_2);
    replica_2.merge(&replica_1);
    println!("{}", replica_1.value());
    println!("{}", replica_2.value());
}
