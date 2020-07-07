//! Support for arbitrary mutating iterators.
//! __Requires `iter_mut`.__
//!
//! Unfortunately, can't be used without `std`: memory for storing 
//! pointers is allocated internally during the access.
//!
//! This module depends on the [`multiref`](https://crates.io/crates/multiref/) 
//! crate. The [`Slice`](struct.Slice.html) type is re-exported 
//! from `multiref`. You can read [the docs](https://docs.rs/multiref/) 
//! or simply use the `At` impls for `Slice` (they are the same as 
//! for normal slices).
//!
//! ## An example
//! 
//! ```
//! use smart_access::{ Cps, iter_mut::Bounds };
//! use std::collections::BTreeMap;
//!
//! let mut map = BTreeMap::<i32, BTreeMap::<&'static str, i32>>::new();
//!
//! map.at( (1, BTreeMap::new()) ).access(|inner| {
//!     inner.insert("a", 2);
//!     inner.insert("b", 3);
//!     inner.insert("c", 4);
//! });
//! 
//! map.at( (5, BTreeMap::new()) ).access(|inner| {
//!     inner.insert("a", 6);
//!     inner.insert("b", 7);
//! });
//! 
//! map.at( (8, BTreeMap::new()) ).access(|inner| {
//!     inner.insert("a", 9);
//!     inner.insert("x", 10);
//! });
//! 
//! map.range_mut(5..).map(|(_,v)| v).at(Bounds(..)).access(|slice| {
//!     slice.as_mut()[0].at("a").replace(11);
//! });
//! ```

pub use multiref::Slice;
mod multiref_impls;

use crate::At;
use core::slice::SliceIndex;

/// A newtype-wrapper around slice bounds.
#[repr(transparent)]#[derive(Debug,Copy,Clone)]
pub struct Bounds<B>(pub B);

impl<'a, I, B, V> At<Bounds<B>> for I where
    I: Iterator<Item=&'a mut V>,
    B: SliceIndex<[&'a mut V], Output=[&'a mut V]>,
    V: 'a + ?Sized,
{
    type View = Slice<V>;

    fn access_at<R, F>(&mut self, i: Bounds<B>, f: F) -> Option<R> where
        F: FnOnce(&mut Slice<V>) -> R
    {
        let mut ref_vec = self.collect::<Vec<_>>();

        Some(f(Slice::new_mut(&mut ref_vec[i.0])))
    }
}

