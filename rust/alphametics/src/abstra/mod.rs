mod map;
pub mod set;

trait Indexer<T: Clone>: Clone {
    //fn start() -> T; // this could return a reference, but `item_for_index` can't, so let's make this one similar.
    //fn past_end() -> T;
    fn index(&self, item: &T) -> usize;
    /// Used when iterating the set.
    fn value(&self, index: usize) -> T;
}
