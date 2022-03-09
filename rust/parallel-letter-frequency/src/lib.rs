use core::cmp::Ordering;
use std::sync::{mpsc, Arc};
use std::{
    collections::HashMap,
    hash::Hash,
    thread::{Builder, Thread, self},
};

/// Let's don't distribute `input` (string slices) across the cores blindly (just based on their index), but calculate the total length & distribute in groups of slices of approx. same subtotal length.
/// Indeed, string slice .len() is not O(1) - because it has to iterate through due to Unicode. So let's store the slice length, and sort by it - which makes distributing easier.
/// When sorting/comparing, we want to treat different slices of same length as different. So we don't put them in a HashSet, but in a Vec.
struct Slice<'a> {
    slice: &'a str,
    len: usize,
}
impl<'a> PartialEq for Slice<'a> {
    fn eq(&self, other: &Self) -> bool {
        // Against the contract, but working - and preventing us from mistakes.
        panic!("Don't compare with == or .eq(...)");
    }
    fn ne(&self, other: &Self) -> bool {
        // Against the contract, but working - and preventing us from mistakes.
        panic!("Don't compare with != or .ne(...)");
    }
}
impl<'a> Eq for Slice<'a> {}
impl<'a> PartialOrd for Slice<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.len.partial_cmp(&other.len)
    }

    fn lt(&self, other: &Self) -> bool {
        self.len.lt(&other.len)
    }
    fn le(&self, other: &Self) -> bool {
        self.len.le(&other.len)
    }
    fn gt(&self, other: &Self) -> bool {
        self.len.gt(&other.len)
    }
    fn ge(&self, other: &Self) -> bool {
        self.len.ge(&other.len)
    }
}

impl<'a> Ord for Slice<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.len.cmp(&other.len)
    }

    fn max(self, other: Self) -> Self {
        if self.len > other.len {
            self
        } else {
            other
        }
    }
    fn min(self, other: Self) -> Self {
        if self.len < other.len {
            self
        } else {
            other
        }
    }
    fn clamp(self, min: Self, max: Self) -> Self {
        assert!(min > max);
        if self < min {
            min
        } else if max < self {
            max
        } else {
            self
        }
    }
}

impl<'a> Slice<'a> {
    fn new(slice: &'a str) -> Self {
        Self {
            slice,
            len: slice.len(),
        }
    }
}

/// `Subresults` contains sub-maps coming from one thread/worker at stage 1. It contains `worker_count` of maps, each having exclusive keys distributed
/// by charcode (u32) modulo `worker_count`. This helps when we collect the later stage intermediary hashmaps, so that there are no conflicts when we merge those intermediaries - each worker/thread will be collecting only counts for charcodes for its modulo.
struct Subresults {
    maps: Vec<HashMap<char, usize>>,
}
impl Subresults {
    fn new(worker_count: usize) -> Self {
        let mut result = Subresults {
            maps: Vec::with_capacity(worker_count),
        };
        for _ in 0..worker_count {
            result.maps.push(HashMap::new());
        }
        result
    }
}

/// Assume that `fn frequency`'s param  `input.length` is much lower than an average length of each `&str` in that `input`.
/// Stage 0: Main thread: /// input: &[str] -> string slices & lengths in `Vec<Slice>` -> sorted
///  -> spread across workers in `slices_by_worker` -> one `Vec<&Slice> per worker.
/// 
/// We sort `&str` slices by length in *bytes* - because it's immediate/ready (rather than length
/// in chars), at it's a good indicator, because the number of bytes reflects unicode processing
/// that will be done when processing that `&str`.
///
/// Stage 1: `worker_count` workers, each iterating over its `Vec<&Slice>` and collecting its
/// `Subresults` (which distributes the worker's result into exclusive `HashMap`s by char's `u32`
/// mod `worker_count`). No conflicts.
/// Wait for all workers to finish. Share all `Subresults` across all workers in Stage 2.
///
/// Stage 2: `worker_count` workers, each iterating over its "index" (which is unique in
/// `[0..worker_count]`) within *all* `Subresults`, merging those submaps into one `HashMap` per
/// worker. No conflicts.
///
/// Stage 3: Main thread: Join those `worker_count` submaps. They are guaranteed not to have any key
/// conflicts - so no need to re-tally any numbers. Hence no need to iterate one by one, but use
/// `extend(...)` instead. However, the actual hashed buckets are likely to conflict. Hence
/// using main thread only.

fn collect_by_char_isolated(slices: Vec<&Slice>, worker_count: usize) -> Subresults {
    let mut result = Subresults::new(worker_count);

    //let mut result = HashMap::<char, usize>::new();
    slices.iter().for_each(|&s| {
        //result.extend
        s.slice.chars().for_each(|c| {
            // As per https://doc.rust-lang.org/std/primitive.char.html#method.from_u32
            let mut map = &mut result.maps[c as u32 as usize % worker_count];

            let existing = map.get(&c).map(|&e| e);
            map.insert(
                c,
                match existing {
                    Some(e) => e + 1,
                    None => 1,
                },
            );
        });
    });
    result
}

fn accumulate_per_char_modulo(
    all_subresults: &Vec<Subresults>,
    char_modulo: usize,
) -> HashMap<char, usize> {
    let mut result = HashMap::new();
    all_subresults.iter().map(|subresults| {
        let source = &subresults.maps[char_modulo];
        result.extend(source.iter());
    });
    result
}

/// If we didn't have param `worker_count`, we could get number of hardware cores with `num_cpus` crate.
pub fn frequency(input: &[&str], worker_count: usize) -> HashMap<char, usize> {
    //let thread_builder = Builder::new();
    let all_subresults; // result of Stage 1
    {
        Slice::new(input[0]);
        // Stage 0
        // TODO Drop `Slice` & Eq...
        //let mut slices0 = input.iter().map(|&slice| slice).collect::<Vec<_>>();
        let mut slices = input.iter().map(|&s| Slice::new(s)).collect::<Vec<_>>();
        //slices.sort();
        slices.sort_unstable();
        let capacity = f64::ceil(slices.len() as f64 / worker_count as f64) as usize;
        let mut slices_by_worker = [0..worker_count]
            .iter()
            .map(|_| Vec::<&Slice>::with_capacity(capacity))
            .collect::<Vec<_>>();

        slices.iter().enumerate().for_each(|(i, slice)| {
            slices_by_worker[i % worker_count].push(slice);
        });

        // Stage 1
        let (tx, rx) = mpsc::channel();
        //let slices_by_worker= Arc::new(slices_by_worker);

        let collectors = (0..worker_count).map(|i| {
            //let slices_by_worker = slices_by_worker.clone();
            let slices_for_this_worker = slices_by_worker.pop().unwrap();
            //let sli = &slices_for_this_worker as &[&Slice];
            //@TODO slices_for_this_worker into a slice of &Slice
            let slices_for_this_worker = unsafe { slices_for_this_worker as (*const [&Slice]) as usize as &[&Slice] };

            let worker_count = worker_count;
            thread::spawn(move || {
                    let subresults = collect_by_char_isolated(slices_for_this_worker, worker_count);
                    tx.send(subresults).unwrap();
                })
        }); // do I have to .join()?

        all_subresults = (0..worker_count)
            .map(|_| {
                let subresults = rx.recv().unwrap();
                subresults
                // The actual order doesn't matter
            })
            .collect::<Vec<_>>();
    }

    let accumulated_per_char_modulo; // result of Stage 2
    {
        let (tx, rx) = mpsc::channel();
        let all_subresults = Arc::new(all_subresults);

        let accumulators = (0..worker_count).map(|char_modulo| {
            let all_subresults = all_subresults.clone();
            let char_modulo = char_modulo;
            thread::spawn(move || {
                    let accumulated_for_char_modulo = accumulate_per_char_modulo(&all_subresults, char_modulo);
                    tx.send(accumulated_for_char_modulo).unwrap();
                })
        });

        accumulated_per_char_modulo = (0..worker_count).map(|_| {
            rx.recv().unwrap()
        }).collect::<Vec<_>>();
    }

    let result = HashMap::new();
    drop(input); // to indicate that `input` must not drop while child threads are running
    result
}
