use std::borrow::Borrow;
use std::collections::{HashMap, BTreeMap};
use std::hash::Hash;
use crate::At;

impl<Q,K,V> At<&Q> for HashMap<K,V> where
    K: Borrow<Q> + Eq + Hash,
    Q: ?Sized + Eq + Hash
{
    type View = V;

    fn access_at<R,F>(&mut self, i: &Q, f: F) -> Option<R> where
        F: FnOnce(&mut V) -> R
    {
        match self.get_mut(i) {
            Some(v) => Some(f(v)),
            None    => None,
        }
    }
}

impl<K,V> At<(K,V)> for HashMap<K,V> where
    K: Eq + Hash,
{
    type View = V;

    fn access_at<R,F>(&mut self, kv: (K,V), f: F) -> Option<R> where
        F: FnOnce(&mut V) -> R
    {
        Some(f(self.entry(kv.0).or_insert(kv.1)))
    }
}


impl<Q,K,V> At<&Q> for BTreeMap<K,V> where
    K: Borrow<Q> + Ord,
    Q: ?Sized + Ord
{
    type View = V;

    fn access_at<R,F>(&mut self, i: &Q, f: F) -> Option<R> where
        F: FnOnce(&mut V) -> R
    {
        match self.get_mut(i) {
            Some(v) => Some(f(v)),
            None    => None,
        }
    }
}

impl<K,V> At<(K,V)> for BTreeMap<K,V> where
    K: Ord,
{
    type View = V;

    fn access_at<R,F>(&mut self, kv: (K,V), f: F) -> Option<R> where
        F: FnOnce(&mut V) -> R
    {
        Some(f(self.entry(kv.0).or_insert(kv.1)))
    }
}

