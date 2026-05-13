pub trait StateBasedCrdt<T> {
    fn merge(&mut self, other: &T);
}
