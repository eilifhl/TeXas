use crate::state_based_crdt::StateBasedCrdt;
use std::cmp::max;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PNCounter {
    replica_id: Uuid,
    p_counts: HashMap<Uuid, u64>,
    n_counts: HashMap<Uuid, u64>,
}

impl PNCounter {
    pub fn new(replica_id: Uuid) -> Self {
        Self {
            replica_id,
            p_counts: HashMap::new(),
            n_counts: HashMap::new(),
        }
    }

    pub fn increment(&mut self, amount: u64) {
        let counter = self.p_counts.entry(self.replica_id).or_insert(0);
        *counter += amount;
    }

    pub fn decrement(&mut self, amount: u64) {
        let counter = self.n_counts.entry(self.replica_id).or_insert(0);
        *counter += amount;
    }

    pub fn value(&self) -> u64 {
        self.p_counts.values().sum::<u64>() - self.n_counts.values().sum::<u64>()
    }
}

impl StateBasedCrdt<PNCounter> for PNCounter {
    fn merge(&mut self, other: &PNCounter) {
        for (replica_id, other_counts) in &other.p_counts {
            let counter = self.p_counts.entry(*replica_id).or_insert(0);
            *counter = max(*other_counts, *counter);
        }
        for (replica_id, other_counts) in &other.n_counts {
            let counter = self.n_counts.entry(*replica_id).or_insert(0);
            *counter = max(*other_counts, *counter);
        }
    }
}
