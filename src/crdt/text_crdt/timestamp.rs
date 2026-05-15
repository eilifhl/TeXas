#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

#[test]
fn test_timestamp() {
    let mut timestamp = Timestamp::zero();

    assert_eq!(timestamp.0, 0);

    timestamp = timestamp.next();

    assert_eq!(timestamp.0, 1)
}
