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