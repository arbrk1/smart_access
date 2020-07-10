//! Support for arbitrary mutating iterators.
//! __Requires `iter_mut`.__
//!
//! Unfortunately, this module can't be used without `std`: 
//! our `Cps` values are _affine_ traversals, thus they must have 
//! all the iteration results simultaneously, which in turn requires
//! allocating memory at runtime.
//!
//! _It is planned to make the `smart_access` crate dependent only on 
//! the `alloc` crate._
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
//! // Hurrah!!! CPS without callback hell!
//! map.range_mut(5..).map(|(_,v)| v).at(Bounds(..)).at(1).at("a").replace(11);
//!
//! assert!(map.at(&8).at("a").get_clone() == Some(11));
//!
//! // And now enter batches (and another level of callbacks, 
//! // but still unlike a typical pyramidal callback hell):
//! map.range_mut(..=5).map(|(_,v)| v).at(Bounds(..)).batch_ct()
//!     .add(|x, _| x.at(0).at("c").replace(12))
//!     .add(|x, _| x.at(0).at("a").replace(13))
//!     .add(|x, _| x.at(1).at("b").replace(14))
//!     .run();
//!
//! assert!(map.at(&1).at("c").get_clone() == Some(12));
//! assert!(map.at(&1).at("a").get_clone() == Some(13));
//! assert!(map.at(&5).at("b").get_clone() == Some(14));
//! ```
//!
//! ## Usage
//!
//! Any `Iterator` (exactly `Iterator`, __not__ `IntoIterator`) 
//! has `At<Bounds<R>>` implemented for every type R of `usize`-indexed ranges.
//!
//! For example, to access the three elements of a `BTreeMap` with 
//! the smallest keys, you can write
//! ```
//! # use std::collections::BTreeMap;
//! # use smart_access::{ Cps, iter_mut::Bounds };
//! # let mut map = BTreeMap::<i32,i32>::new();
//! # let result =
//! map.iter_mut().map(|kv| kv.1).at(Bounds(..3)).access( |xs| { /* do something */ } );
//! # assert!(result == None);
//! ```

pub use multiref::Slice;
mod multiref_impls;

use crate::At;

/// A newtype-wrapper around slice bounds.
#[repr(transparent)]#[derive(Debug,Copy,Clone)]
pub struct Bounds<B>(pub B);

impl<'a, I, B, V> At<Bounds<B>> for I where
    I: Iterator<Item=&'a mut V>,
    [&'a mut V]: At<B, View = [&'a mut V]>,
    V: 'a + ?Sized,
{
    type View = Slice<V>;

    fn access_at<R, F>(&mut self, i: Bounds<B>, f: F) -> Option<R> where
        F: FnOnce(&mut Slice<V>) -> R
    {
        let mut ref_vec = self.collect::<Vec<_>>();

        (&mut ref_vec[..]).access_at(i.0, |subslice| {
            f(Slice::new_mut(subslice))
        })
    }
}

// WIP: a marker to get an iterator from some concrete collection types
// pub struct Iter; 

