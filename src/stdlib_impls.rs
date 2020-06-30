//! Implementation of [`At`](../trait.At.html) for stdlib collections. 
//! __Requires `std_collections` feature.__
//!
//! The following traits are implemented:
//! * `At<usize, View=T> for Vec<T>`: simple indexing
//! * `At<range, View=Vec<T>> for Vec<T>`: subvector (its size can be changed); 
//!   __Warning:__ access is O(n); wrap vector in `&mut[..]` to get O(1) access
//! * `At<K, View=V> for <Some>Map<K,V>`: access value if it is present 
//! * `At<(K,V), View=V> for <Some>Map<K,V>`: ensure that the value is 
//!   present (using the provided default) then access it
//! * `AT<(K,V,M), View=V> for <Some>Map<K,V>`: if the value is present 
//!   then preprocess it with a mutator `M`, otherwise insert the provided `V` 
//!
//! Though in normal circumstances these implementations __do not__ panic
//! there __exists__ a possibility of panicking. For example 
//! `At<range> for Vec<T>` splits vector into (at most) three parts
//! then glues them back after the update. Every of these actions 
//! can panic on Out Of Memory.
//!
//! ### Vector accessors
//!
//! ```
//! # use smart_access::{ Cps };
//! let mut foo = vec![1,2,3];
//!
//! assert!(foo.at(0).replace(4) == Some(1));
//! assert!(foo == vec![4,2,3]);
//! 
//! assert!(foo.at(3).replace(0) == None);
//! assert!(foo == vec![4,2,3]);
//!
//! assert!(foo.at(1..=2).replace(vec![5,6,7]) == Some(vec![2,3]));
//! assert!(foo == vec![4,5,6,7]);
//! ```
//!
//!
//! ### Map accessors
//!
//! Implemented for `HashMap` and `BTreeMap`:
//!
//! * `map.at(&k).access(f)` is equivalent to `map.get_mut(&k).map(|v| f(v))`
//! * `map.at( (k,v) ).access(f)` is equivalent to `Some(f(map.entry(k).or_insert(v)))`
//! * `map.at( (k,v,m) ).access(f)` is equivalent to `Some(f(self.entry(k).and_modify(m).or_insert(v)))`
//!
//! ```
//! # use smart_access::{ Cps };
//! # use std::collections::HashMap;
//! let mut hm = HashMap::<usize, usize>::new();
//!
//! hm.at( (42, 1) ).touch();
//!
//! let mut found = false;
//! hm.at(&42).access(|x| { 
//!     assert!(*x == 1);
//!
//!     found = true;
//!     
//!     *x += 1; 
//! });
//!
//! assert!(found);
//! assert!(hm.get(&42) == Some(&2));
//!
//! let mutator = |x: &mut _| { *x = 4; };
//! hm.at( (41, 3, &mutator) ).touch();
//! hm.at( (42, 3, &mutator) ).touch();
//! 
//! assert!(hm.get(&41) == Some(&3));
//! assert!(hm.get(&42) == Some(&4));
//! ```

mod vec;
mod map;

#[test]
fn test_vec() {
    use crate::Cps;
    
    let mut foo = vec![1,2,3,4,5];

    let update = |i| move |vec: &mut Vec<i32>| {
        vec.push(i);

        vec[0]
    };
    
    assert!(foo.at(1..3).access(update(6)) == Some(2));
    assert!(foo == vec![1,2,3,6,4,5]);
    
    assert!(foo.at(2..).access(update(7)) == Some(3));
    assert!(foo == vec![1,2,3,6,4,5,7]);
    
    assert!(foo.at(..4).access(update(8)) == Some(1));
    assert!(foo == vec![1,2,3,6,8,4,5,7]);
    
    assert!(foo.at(..).access(update(9)) == Some(1));
    assert!(foo == vec![1,2,3,6,8,4,5,7,9]);
    
    assert!(foo.at(..=10).access(update(1)) == None);
    assert!(foo == vec![1,2,3,6,8,4,5,7,9]);
    
    assert!(foo.at(3..=4).access(update(0)) == Some(6));
    assert!(foo == vec![1,2,3,6,8,0,4,5,7,9]);

    assert!(foo.at(4).replace(1) == Some(8));
    assert!(foo == vec![1,2,3,6,1,0,4,5,7,9]);
}


#[test]
fn test_hash_map() {
    use std::collections::HashMap;
    use crate::Cps;


    let mut map = HashMap::<String,i32>::new();
    map.at( ("foo".to_string(), 1) ).touch();
    map.at( ("bar".to_string(), 2) ).touch();
    map.at( ("baz".to_string(), 3) ).touch();
    
    assert!(map.at("foo").replace(4) == Some(1));
    assert!(map.at("quuz").replace(5) == None);

    let mut reference_map = HashMap::<String,i32>::new();
    reference_map.entry("foo".to_string()).or_insert(4);
    reference_map.entry("bar".to_string()).or_insert(2);
    reference_map.entry("baz".to_string()).or_insert(3);

    assert!(map == reference_map);
}


#[test]
fn test_btree_map() {
    use std::collections::BTreeMap;
    use crate::Cps;


    let mut map = BTreeMap::<String,i32>::new();
    map.at( ("foo".to_string(), 1) ).touch();
    map.at( ("bar".to_string(), 2) ).touch();
    map.at( ("baz".to_string(), 3) ).touch();
    
    assert!(map.at("foo").replace(4) == Some(1));
    assert!(map.at("quuz").replace(5) == None);

    let mut reference_map = BTreeMap::<String,i32>::new();
    reference_map.entry("foo".to_string()).or_insert(4);
    reference_map.entry("bar".to_string()).or_insert(2);
    reference_map.entry("baz".to_string()).or_insert(3);

    assert!(map == reference_map);
}

