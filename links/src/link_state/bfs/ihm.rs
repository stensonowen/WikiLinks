//! Custom hash map implementation for use in breadth-first search
//! This application omits some of the features of a normal hash table and must be very fast,
//!  so it makes sense to rewrite it to take advantage of certain optimizations.
//! These tables are always keyed `i32`s and grow fast enough that their capacity can always
//!  be a power of two; also, inserts and lookups need to be very fast, so we can make the
//!  hash function a simple bitmask (assuming `page_id`s are relatively randomly distributed).
//! In theory this means we can also omit bounds checking, but we'll see (I don't have any
//!  `unsafe` blocks in this project yet).
//! Because we want lots of caching and we're assuming our inputs are pretty random, we can
//!  probably get away with open addressing by just incrementing the hash.
//! We never remove from these tables, so we can save on the space / complexity by not 
//!  implementing this mechanism.
//! This shows a pretty reliable few point bump over `fnv` in lookups and like 10-15% in
//!  inserts, which is pretty promising (hasn't been tested yet though).

// Speed comparison: w/ RT=.5 and BC=6, speedup is almost 50%
// Memory comparison: 

use std::fmt::Debug;

// TODO: tweak this?
const RESIZE_THRESHOLD: f64 = 0.5; // resize when table is 1/2 full
// TODO: tweak this?
const BEGIN_CAP: usize = 6; // 2^6 = 64

// TODO: verify this?
const ENTRY_RESERVED: u32 = ::std::u32::MAX; // u32::max_value() on nightly

pub type IHMap = IHM<u32>;
pub type IHSet = IHM<()>;

/// Entry in our hash map: Instead of using `Option` or some unsafe magic, we reserve
///  one potential value as "none" (where `page_id` is `INT_MAX`). This saves a lot on space
///  (probably about 50% of what it would be) while still being fast (it's like 20%
///  faster than `Option<Entry<T>>` because of caching or something).
/// The idea is stolen from the `optional` crate, but I like my interface better.
#[derive(Debug, Clone, Copy)]
pub(super) struct Entry<T: Debug+Copy+Default> {
    key: u32,
    val: T,
}

impl<T: Debug+Copy+Default> Entry<T> {
    #[inline]
    fn is_none(&self) -> bool {
        self.key == ENTRY_RESERVED
    }
    #[inline]
    fn is_some(&self) -> bool {
        self.is_none() == false
    }
    #[inline]
    fn get(&self) -> Option<&Self> {
        // the fact that this isn't in `optional` makes me worry it's slow?
        // Am I crazy?
        if self.is_none() {
            None
        } else {
            Some(self)
        }
    }
    #[inline]
    fn none() -> Self {
        Entry {
            key: ENTRY_RESERVED,
            val: Default::default(),
        }
    }
}

/// Integer hash map: map keyed by integers for a very specific application.
/// Has some restrictions: Can't remove entries, capacity must be a power of 2, 
///  input must be relatively randomly distributed, etc.
#[derive(Debug, Clone)]
pub struct IHM<T: Debug+Copy+Default> {
    size: usize,
    cap_exp: usize, // capacity is 2**this
    data: Box<[Entry<T>]>,
}

impl<T: Debug+Copy+Default> Default for IHM<T> {
    fn default() -> Self {
        IHM::with_capacity(BEGIN_CAP)
    }
}

impl<T: Debug+Copy+Default> IHM<T> {
    #[inline]
    fn hash(&self, n: u32) -> usize {
        // n % self.capacity()
        // last `cap_exp` bytes of `n`
        (n & ((1 << self.cap_exp) - 1)) as usize
    }
    #[inline]
    fn hash_with(n: usize, cap: usize) -> usize {
        // way to `hash` without requiring a self-borrow :/
        // don't mix up the arg order :/
        (n & ((1 << cap) -1)) as usize
    }
    // uhhhhh note this is cap_exp NOT real capacity
    // prediction: I forget this and run out of mem :/
    pub fn with_capacity(cap_exp: usize) -> Self {
        // not sure what the proper way to create a big boxed array
        //  without just making it on the stack first
        //  except using a vector
        let v: Vec<Entry<T>> = vec![Entry::none(); 1<<cap_exp];
        IHM {
            size: 0,
            cap_exp,
            data: v.into_boxed_slice(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    pub fn len(&self) -> usize {
        self.size
    }
    pub fn capacity(&self) -> usize {
        1 << self.cap_exp
    }
    pub fn clear(&mut self) {
        for entry in self.data.iter_mut() {
            *entry = Entry::none();
        }
    }
    pub fn contains_key(&self, key: u32) -> bool {
        let mut addr = self.hash(key);
        loop {
            // For now let's see how well rustc optimizes this
            // but this might need to be rewritten to be faster
            // because it'll happen a lot
            // TODO examine asm
            // TODO eventually don't bounds check
            //  this leads to a few points of speedup
            //  but it also introduces the possibility of a segfault
            //  maybe until everything is stable and if there are no crashes then revisit

            /*
            let entry = unsafe { self.data.get_unchecked(addr) };
            match entry.get() {
            */
            match self.data[addr].get() {
                None => return false,
                Some(e) if e.key == key => return true,
                Some(_) => addr = Self::hash_with(addr+1, self.cap_exp),
            }
        }
    }
    fn resize(&mut self) {
        // NOTE this doesn't change in-place. It will briefly consume 50% more
        // mem than in its final state due to this redundancy
        let mut other = Self::with_capacity(self.cap_exp + 1);
        assert_eq!(other.capacity(), 2*self.capacity());
        for entry in self.data.iter() {
            if entry.is_some() {
                other.insert_elem(entry.key, entry.val);
            }
        }
        *self = other;
    }
    fn insert_elem(&mut self, key: u32, val: T) {
        // TODO maybe could add some cool simd stuff or manual unrolling here
        if self.len() as f64 / self.capacity() as f64 >= RESIZE_THRESHOLD {
            self.resize();
        }
        self.size += 1;
        let mut addr = self.hash(key);
        loop {
            let entry = &mut self.data[addr];
            //let entry = unsafe { self.data.get_unchecked_mut(addr) };
            if entry.is_none() {
                // wasn't present, insert and continue
                *entry = Entry { key, val };
                return;
            } else if entry.key == key {
                // entry already there
                return;
            } else {
                // otherwise increment addr and try again
                //addr = self.hash(addr as u32 + 1);
                //addr = (addr+1) & ((1 << self.cap_exp) - 1)
                addr = Self::hash_with(addr+1, self.cap_exp);
            }
        }
    }
}

use std::iter::FilterMap;
use std::slice::Iter;
type IterType<'a,T> = FilterMap<Iter<'a, Entry<T>>, for<'r> fn(&'r Entry<T>) -> Option<u32>>;

impl IHM<()> {
    pub fn insert(&mut self, key: u32) {
        self.insert_elem(key, ())
    }
    //pub(super) fn keys<'a>(&'a self) -> IterType<'a, ()> {
    pub(super) fn keys(&self) -> IterType<()> {
        self.data.iter().filter_map(|i| i.get().map(|e| e.key))
    }
}

impl IHM<u32> {
    pub fn insert(&mut self, key: u32, val: u32) {
        self.insert_elem(key, val)
    }
    pub fn get(&self, key: u32) -> Option<u32> {
        let mut addr = self.hash(key);
        loop {
            /*
            let entry = unsafe { self.data.get_unchecked(addr) };
            match entry.get() {
            */
            match self.data[addr].get() {
                None => return None,
                Some(e) if e.key == key => return Some(e.val),
                Some(_) => addr = (addr+1) & ((1 << self.cap_exp) - 1),
            }
        }
    }
}


