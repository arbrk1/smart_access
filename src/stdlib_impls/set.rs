use std::borrow::Borrow;
use std::collections::{HashSet, BTreeSet};
use std::hash::Hash;
use crate::At;


impl<T> At<(T,)> for HashSet<T> where
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


impl<T> At<(T,())> for HashSet<T> where
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


impl<Q,T> At<&Q> for HashSet<T> where
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


/* EDIT-ACCESSOR: WIP
impl<Q,T> At<Option<&Q>> for HashSet<T> where
    T: Borrow<Q> + Eq + Hash,
    Q: ?Sized + Eq + Hash
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

