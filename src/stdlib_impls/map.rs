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
        self.get_mut(i).map(|v| f(v))
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

impl<K,V,M> At<(K,V,M)> for HashMap<K,V> where
    K: Eq + Hash,
    M: FnOnce(&mut V)
{
    type View = V;

    fn access_at<R,F>(&mut self, kvm: (K,V,M), f: F) -> Option<R> where
        F: FnOnce(&mut V) -> R
    {
        Some(f(self.entry(kvm.0).and_modify(kvm.2).or_insert(kvm.1)))
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
        self.get_mut(i).map(|v| f(v))
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

impl<K,V,M> At<(K,V,M)> for BTreeMap<K,V> where
    K: Ord,
    M: FnOnce(&mut V)
{
    type View = V;

    fn access_at<R,F>(&mut self, kvm: (K,V,M), f: F) -> Option<R> where
        F: FnOnce(&mut V) -> R
    {
        Some(f(self.entry(kvm.0).and_modify(kvm.2).or_insert(kvm.1)))
    }
}

