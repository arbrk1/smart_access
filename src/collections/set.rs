use core::borrow::Borrow;
use alloc::collections::BTreeSet;
use core::hash::Hash;
use crate::At;


#[cfg(feature="hashbrown")]
impl<T> At<(T,)> for hashbrown::HashSet<T> where
    T: Eq + Hash,
{
    type View = Self;

    fn access_at<R,F>(&mut self, item: (T,), f: F) -> Option<R> where
        F: FnOnce(&mut Self) -> R
    {
        self.insert(item.0);

        Some(f(self))
    }
}


#[cfg(feature="hashbrown")]
impl<T> At<(T,())> for hashbrown::HashSet<T> where
    T: Eq + Hash,
{
    type View = T;

    fn access_at<R,F>(&mut self, mut item: (T,()), f: F) -> Option<R> where
        F: FnOnce(&mut T) -> R
    {
        if let Some(v) = self.take(&item.0) {
            item.0 = v;
        }

        let result = f(&mut item.0);
        
        self.insert(item.0);

        Some(result)
    }
}


#[cfg(feature="hashbrown")]
impl<Q,T> At<&Q> for hashbrown::HashSet<T> where
    T: Borrow<Q> + Eq + Hash,
    Q: ?Sized + Eq + Hash
{
    type View = T;

    fn access_at<R,F>(&mut self, i: &Q, f: F) -> Option<R> where
        F: FnOnce(&mut T) -> R
    {
        self.take(i).map(|mut v| {
            let result = f(&mut v);

            self.insert(v);

            result
        })
    }
}


#[cfg(feature="std_hashmap")]
extern crate std;


#[cfg(feature="std_hashmap")]
impl<T> At<(T,)> for std::collections::HashSet<T> where
    T: Eq + Hash,
{
    type View = Self;

    fn access_at<R,F>(&mut self, item: (T,), f: F) -> Option<R> where
        F: FnOnce(&mut Self) -> R
    {
        self.insert(item.0);

        Some(f(self))
    }
}


#[cfg(feature="std_hashmap")]
impl<T> At<(T,())> for std::collections::HashSet<T> where
    T: Eq + Hash,
{
    type View = T;

    fn access_at<R,F>(&mut self, mut item: (T,()), f: F) -> Option<R> where
        F: FnOnce(&mut T) -> R
    {
        if let Some(v) = self.take(&item.0) {
            item.0 = v;
        }

        let result = f(&mut item.0);
        
        self.insert(item.0);

        Some(result)
    }
}


#[cfg(feature="std_hashmap")]
impl<Q,T> At<&Q> for std::collections::HashSet<T> where
    T: Borrow<Q> + Eq + Hash,
    Q: ?Sized + Eq + Hash
{
    type View = T;

    fn access_at<R,F>(&mut self, i: &Q, f: F) -> Option<R> where
        F: FnOnce(&mut T) -> R
    {
        self.take(i).map(|mut v| {
            let result = f(&mut v);

            self.insert(v);

            result
        })
    }
}






impl<T> At<(T,)> for BTreeSet<T> where
    T: Ord,
{
    type View = Self;

    fn access_at<R,F>(&mut self, item: (T,), f: F) -> Option<R> where
        F: FnOnce(&mut Self) -> R
    {
        self.insert(item.0);

        Some(f(self))
    }
}

impl<T> At<(T,())> for BTreeSet<T> where
    T: Ord,
{
    type View = T;

    fn access_at<R,F>(&mut self, mut item: (T,()), f: F) -> Option<R> where
        F: FnOnce(&mut T) -> R
    {
        if let Some(v) = self.take(&item.0) {
            item.0 = v;
        }

        let result = f(&mut item.0);
        
        self.insert(item.0);

        Some(result)
    }
}

impl<Q,T> At<&Q> for BTreeSet<T> where
    T: Borrow<Q> + Ord,
    Q: ?Sized + Ord
{
    type View = T;

    fn access_at<R,F>(&mut self, i: &Q, f: F) -> Option<R> where
        F: FnOnce(&mut T) -> R
    {
        self.take(i).map(|mut v| {
            let result = f(&mut v);

            self.insert(v);

            result
        })
    }
}


/* EDIT-ACCESSOR: WIP
impl<Q,T> At<Option<&Q>> for BTreeSet<T> where
    T: Borrow<Q> + Ord,
    Q: ?Sized + Ord
{
    type View = Option<T>;

    fn access_at<R,F>(&mut self, maybe_i: Option<&Q>, f: F) -> Option<R> where
        F: FnOnce(&mut Option<T>) -> R
    {
        maybe_i.map(|i| {
            self.take(i).map(|v| {
                let mut cell = Some(v);

                let result = f(&mut cell);

                if let Some(new_v) = cell {
                    self.insert(new_v);
                }

                result
            })
        }).flatten()
    }
}*/

