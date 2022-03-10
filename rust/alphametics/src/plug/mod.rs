pub mod map;
pub mod set;

use std::fmt::{self};
use std::ops::{Add, Sub};

#[allow(unused)]
#[cfg(test)]
mod test {
    enum Slice<'a> {
        Shared(&'a [i32]),
        Mutable(&'a mut [i32]),
    }

    fn write(slice: &mut Slice) {
        match slice {
            Slice::Shared(slice) => {
                let i = slice[0];
            }
            Slice::Mutable(slice) => {
                slice[0] = 5;
            }
        }
    }

    #[test]
    fn f() {
        let shared_num = [1, 2, 3];
        let mut mut_num = [4, 5, 6];
    }
}

/// Abstract set.
pub trait Set<T: core::hash::Hash + Eq + Clone>: Clone {
    // To use with non-cloneable, have:
    // type ITER<'a>: Iterator<Item = &'a T>
    // where
    //     T: 'a,
    //     Self: 'a;
    // -- but then we can't have BoolSlice-based or any other value generation.
    /// Thanks to Shadow0133 for https://www.reddit.com/r/rust/comments/t4egmf/lifetime_generic_associated_type_bounded_by_the
    type ITER<'a>: Iterator<Item = T>
    where
        T: 'a,
        Self: 'a;

    fn contains(&self, value: &T) -> bool;
    fn insert(&mut self, value: T) -> bool;
    fn insert_all(&mut self, iter: impl Iterator<Item = T>) {
        iter.for_each(|item| {
            self.insert(item);
        });
    }
    fn remove(&mut self, value: &T) -> bool;
    fn iter<'a>(&'a self) -> Self::ITER<'a>;
    /// Return a new empty set. For range/max size-bound sets it will have same constraints or capacity.
    fn new_like(&self) -> Self;
}

pub trait Indexer<T: Clone>: Clone {
    fn index(&self, key: &T) -> usize;
    /// Used to generate an item (key) when iterating over a boolean-backed or similar set.
    fn key(&self, index: usize) -> T;
    fn new(start_key: &T) -> Self;
}

#[derive(Clone)]
struct RangeIndexer<T: Clone> {
    start_key: T,
    start_index: usize,
}
/// Default implementation for primitive unsigned/signed integers.
/// In nightly Rust as of early 2022, this works for `char`, too - `char` implements `Sub<char>`, even though that doesn't show up at https://doc.rust-lang.org/nightly/std/primitive.char.html.
/// TODO make this compile conditionally: - errornous for 32 bit and bigger integers on 16bit platforms.
impl<T: Clone + Sub<T> + Add<T>> Indexer<T> for RangeIndexer<T>
where
    T: TryInto<usize>,
    usize: TryFrom<T>,
    usize: TryFrom<<T as Sub>::Output>,
    T: TryFrom<usize>,
    <usize as TryFrom<<T as Sub>::Output>>::Error: fmt::Debug,
    <T as TryFrom<usize>>::Error: fmt::Debug,
    <T as TryInto<usize>>::Error: fmt::Debug,
{
    fn index(&self, key: &T) -> usize {
        // @TODO Consider an alternative: key.clone().try_into().expect(...) - self.start_index. Unsure about default implementation for `char`.
        // However, the current implementation would work on 16 bit platforms,
        // while using key.clone.try_into().expect(...) - self.start_index would not!
        (key.clone() - self.start_key.clone())
            .try_into()
            .expect("Item out of range.")
    }
    fn key(&self, index: usize) -> T {
        (self.start_index + index)
            .try_into()
            .expect("Index out of range.")
    }
    fn new(start_key: &T) -> Self {
        Self {
            start_key: start_key.clone(),
            start_index: start_key
                .clone()
                .try_into()
                .expect("Start index out of range."),
        }
    }
}

/*
/// As per https://doc.rust-lang.org/std/primitive.char.html#method.from_u32, any `char` can be cast to u32
/// TODO make this conditional - not compilable on 16bit platforms.
impl Indexer<char> for RangeIndexer<char> {
    fn index(&self, item: &char) -> usize {
        *item as u32 - self.start as u32
    }
    fn value(&self, index: usize) -> char {
        char::from_u32(self.start as usize + index).unwrap()
    }
}*/

fn test_char_range(indexer: &RangeIndexer<char>) {
    let clone = indexer.clone();
}

/// TODO use?
/// Implement only for types where any value has a valid (and unique) usize index.
pub trait Indexable {
    fn index(&self) -> usize;
    fn key(index: usize) -> Self;
}

/// @TODO use?
pub trait RangeIndexable {
    fn index(&self, base: &Self) -> usize;
    /// Intentionally not using &self as base, since it could be unclear.
    fn key(index: usize, base: &Self) -> Self;
}