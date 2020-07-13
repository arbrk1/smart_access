use core::borrow::Borrow;
use alloc::collections::BTreeMap;
use core::hash::Hash;
use crate::At;

#[cfg(feature="hashbrown")]
impl<Q,K,V> At<&Q> for hashbrown::HashMap<K,V> where
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

#[cfg(feature="hashbrown")]
impl<K,V> At<(K,V)> for hashbrown::HashMap<K,V> where
    K: Eq + Hash,
{
    type View = V;

    fn access_at<R,F>(&mut self, kv: (K,V), f: F) -> Option<R> where
        F: FnOnce(&mut V) -> R
    {
        Some(f(self.entry(kv.0).or_insert(kv.1)))
    }
}

#[cfg(feature="hashbrown")]
impl<K,V,M> At<(K,V,M)> for hashbrown::HashMap<K,V> where
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


#[cfg(feature="std_hashmap")]
extern crate std;


#[cfg(feature="std_hashmap")]
impl<Q,K,V> At<&Q> for std::collections::HashMap<K,V> where
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

#[cfg(feature="std_hashmap")]
impl<K,V> At<(K,V)> for std::collections::HashMap<K,V> where
    K: Eq + Hash,
{
    type View = V;

    fn access_at<R,F>(&mut self, kv: (K,V), f: F) -> Option<R> where
        F: FnOnce(&mut V) -> R
    {
        Some(f(self.entry(kv.0).or_insert(kv.1)))
    }
}

#[cfg(feature="std_hashmap")]
impl<K,V,M> At<(K,V,M)> for std::collections::HashMap<K,V> where
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


/* EDIT-ACCESSOR: WIP
impl<Q,K,V> At<Option<&Q>> for BTreeMap<K,V> where
    K: Borrow<Q> + Ord /* FIXME: remove Clone when remove_entry stabilizes */ + Clone,
    Q: ?Sized + Ord
{
    type View = Option<V>;

    fn access_at<R,F>(&mut self, maybe_i: Option<&Q>, f: F) -> Option<R> where
        F: FnOnce(&mut Option<V>) -> R
    {
        maybe_i.map(|i| {
            if let Some( (k,_) ) = self.get_key_value(i) {
                let k = k.clone();
                let v = self.remove(i).unwrap();

                let mut cell = Some(v);
                
                let result = f(&mut cell);

                if let Some(new_v) = cell {
                    self.insert(k, new_v);
                }

                Some(result)
            } else { None }
        }).flatten()

        /* UNSTABLE (rustc v1.44)
        maybe_i.map(|i| {
            self.remove_entry(i).map(|(k,v)| {
                let mut cell = Some(v);

                let result = f(&mut cell);

                if let Some(new_v) = cell {
                    self.insert(k, new_v);
                }

                result
            })
        }).flatten() */
    }
}*/

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

