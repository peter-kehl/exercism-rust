use super::Indexer;
use core::hash::Hash;
use std::{
    collections::{hash_set, HashMap, HashSet},
    fmt,
    marker::PhantomData,
};

pub trait AbstractSet<T: core::hash::Hash + Eq + Clone>: Clone {
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

#[derive(Debug)]
pub struct HashedSet<T> {
    set: HashSet<T>,
}

impl<T: Hash + Eq + Clone> AbstractSet<T> for HashedSet<T> {
    type ITER<'a>
    where
        T: 'a,
        Self: 'a,
    = HashedSetIter<'a, T>;
    fn contains(&self, value: &T) -> bool {
        self.set.contains(value)
    }
    fn insert(&mut self, value: T) -> bool {
        self.set.insert(value)
    }
    fn remove(&mut self, value: &T) -> bool {
        self.set.remove(value)
    }
    fn iter<'a>(&'a self) -> Self::ITER<'a> {
        HashedSetIter {
            set_iter: self.set.iter(),
        }
    }

    fn new_like(&self) -> Self {
        Self {
            set: HashSet::<T>::new(),
        }
    }
}

impl<T: Hash + Eq + Clone> Clone for HashedSet<T> {
    fn clone(&self) -> Self {
        Self {
            set: self.set.clone(),
        }
    }
}

impl<T: Hash + Eq> HashedSet<T> {
    pub fn new() -> Self {
        Self {
            set: HashSet::new(),
        }
    }
}

pub struct HashedSetIter<'a, T: 'a> {
    set_iter: hash_set::Iter<'a, T>,
}
impl<'a, T: Clone> Iterator for HashedSetIter<'a, T> {
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<T> {
        self.set_iter.next().map(|value| value.clone())
    }
}

impl<T: core::hash::Hash + Eq> FromIterator<T> for HashedSet<T> {
    fn from_iter<IT>(iter: IT) -> Self
    where
        IT: IntoIterator<Item = T>,
    {
        Self {
            set: HashSet::from_iter(iter),
        }
    }
}

/// Backed by a mutable slice of booleans (not packed, but ordinary).
/// Using a mutable slice rather than a shared one, as the main purpose of AbstractSet is to
/// actively operate on it (rather than pass it/store it as immutable only).
/// Not backed by an (owned) array - that would require a const generic parameter, which would
/// enlarge the resulting binary and compile & build time.
#[derive(Debug)]
struct SliceSet<'s, T: Clone, I: Indexer<T>> {
    slice: &'s mut [bool],
    /// Stored owned, not by reference - good for CPU cache affinity.
    indexer: I,
    _items: PhantomData<T>, // so that we don't mix BoolSliceSet of various item types
}

impl<'s, T: Hash + Eq + Clone, I: Indexer<T>> AbstractSet<T> for SliceSet<'s, T, I> {
    type ITER<'a>
    where
        T: 'a,
        Self: 'a,
    = BoolSliceSetIter<'a, T, I>;
    fn contains(&self, value: &T) -> bool {
        self.slice[self.indexer.index(value)]
    }
    fn insert(&mut self, value: T) -> bool {
        let index = self.indexer.index(&value);
        let already_present = self.slice[index];
        self.slice[index] = true;
        !already_present
    }
    fn remove(&mut self, value: &T) -> bool {
        let index = self.indexer.index(&value);
        let was_present = self.slice[index];
        self.slice[index] = false;
        was_present
    }
    fn iter<'a>(&'a self) -> Self::ITER<'a> {
        BoolSliceSetIter {
            slice_enum: self.slice.iter().enumerate(),
            indexer: self.indexer.clone(),
            _items: PhantomData,
        }
    }
    fn new_like(&self) -> Self {
        unimplemented!("Cannot be implemented.");
    }
}

impl<'s, T: Hash + Eq + Clone, I: Indexer<T>> Clone for SliceSet<'s, T, I> {
    fn clone(&self) -> Self {
        unimplemented!("Cannot be supported");
    }
}
/*impl<'s, T: Hash + Eq, I: BoolSliceSetIndexer<T>> BoolSliceSet<'s, T, I> {
    fn new(slice: &'s mut [bool]) -> Self {
        Self {
            slice,
            indexer: PhantomData,
            _items: PhantomData,
        }
    }
}*/

#[derive(Clone)]
struct BoolSliceSetIter<'a, T: Clone, I: Indexer<T>> {
    slice_enum: core::iter::Enumerate<core::slice::Iter<'a, bool>>,
    /// Cloned, owned - better than a reference, good for CPU cache affinity.
    indexer: I,
    _items: PhantomData<T>,
}
impl<'a, T: Clone, I: Indexer<T>> Iterator for BoolSliceSetIter<'a, T, I> {
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<T> {
        loop {
            if let Some((index, &value_present)) = self.slice_enum.next() {
                if value_present {
                    break Some(self.indexer.value(index));
                }
            } else {
                break None;
            }
        }
    }
}

#[derive(Clone)]
struct BoolSliceCharSetIndexer {}
impl Indexer<char> for BoolSliceCharSetIndexer {
    //fn start() -> char { ' ' }
    fn index(&self, item: &char) -> usize {
        0
    }
    fn value(&self, index: usize) -> char {
        ' '
    }
}

/*impl<'a, T: core::hash::Hash + Eq> FromIterator<T> for BoolSliceSet<'a, T> {
    fn from_iter<IT>(iter: IT) -> Self
    where
        IT: IntoIterator<Item = T>,
    {
        Self {
            set: HashSet::from_iter(iter),
        }
    }
}*/
