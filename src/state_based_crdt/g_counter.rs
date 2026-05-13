use crate::state_based_crdt::StateBasedCrdt;
use std::cmp::max;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GCounter {
    replica_id: Uuid,
    counts: HashMap<Uuid, u64>,
}

impl GCounter {
    pub fn new(replica_id: Uuid) -> Self {
        Self {
            replica_id,
            counts: HashMap::new(),
        }
    }

    pub fn increment(&mut self, amount: u64) {
        let counter = self.counts.entry(self.replica_id).or_insert(0);
        *counter += amount;
    }

    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }
}

impl StateBasedCrdt<GCounter> for GCounter {
    fn merge(&mut self, other: &GCounter) {
        for (replica_id, other_counts) in &other.counts {
            let counter = self.counts.entry(*replica_id).or_insert(0);
            *counter = max(*other_counts, *counter);
        }
    }
}
