//! Implementation of [`At`](../trait.At.html) for core datatypes.
//!
//! The following traits are implemented:
//! * `At<usize, View=T> for [T]`: simple indexing
//! * `At<range, View=[T]> for [T]`: subslice (of fixed size)
//! * `At<(), View=T> for Option<T>`: the only meaningful sort of access
//! * `At<(), View=R> for Result<R,E>`: access to the `Ok` value
//!
//! All implementations never panic: `None` is returned instead if the 
//! index doesn't make sense. If you want panicking behaviour simply 
//! add `.unwrap()` to your access:
//!
//! ``` should_panic
//! # use smart_access::Cps;
//! let mut foo = [0,1,2];
//!
//! (&mut foo[..]).at(3).access(|x| { *x += 1; }).unwrap();
//! ```
//!
//! is the same as
//!
//! ``` should_panic
//! # use smart_access::Cps;
//! # let mut foo = [0,1,2];
//! (&mut foo[..])[3].access(|x| { *x += 1; });
//! ```

mod slice;

#[test]#[cfg(feature="alloc")]
fn test_slice() {
    use crate::Cps;
    use alloc::vec;

    let mut foo = vec![1,2,3,4,5];

    let update = |i| move |slice: &mut [i32]| {
        let old_value = slice[0];
        
        slice[0] = i;

        old_value
    };
    
    assert!((&mut foo[..]).at(1..3).access(update(6)) == Some(2));
    assert!(foo == vec![1,6,3,4,5]);
    
    assert!((&mut foo[..]).at(2..).access(update(7)) == Some(3));
    assert!(foo == vec![1,6,7,4,5]);
    
    assert!((&mut foo[..]).at(..4).access(update(8)) == Some(1));
    assert!(foo == vec![8,6,7,4,5]);
    
    assert!((&mut foo[..]).at(..).access(update(9)) == Some(8));
    assert!(foo == vec![9,6,7,4,5]);
    
    assert!((&mut foo[..]).at(..=6).access(update(1)) == None);
    assert!(foo == vec![9,6,7,4,5]);
    
    assert!((&mut foo[..]).at(3..=4).access(update(0)) == Some(4));
    assert!(foo == vec![9,6,7,0,5]);

    assert!((&mut foo[..]).at(4).replace(1) == Some(5));
    assert!(foo == vec![9,6,7,0,1]);
}


// Other implementations

use crate::At;

impl<T> At<()> for Option<T> {
    type View = T;

    fn access_at<R, F>(&mut self, _: (), f: F) -> Option<R> where
        F: FnOnce(&mut T) -> R
    {
        match self {
            Some(x) => Some(f(x)),
            None    => None,
        }
    }
}

impl<T,S> At<()> for Result<T,S> {
    type View = T;

    fn access_at<R, F>(&mut self, _: (), f: F) -> Option<R> where
        F: FnOnce(&mut T) -> R
    {
        match self {
            Ok(x)  => Some(f(x)),
            Err(_) => None,
        }
    }
}


#[test]
fn test_optional() {
    use crate::Cps;

    let mut foo: Option<i32> = Some(0);
    let mut bar: Option<i32> = None;

    assert!(foo.at(()).replace(1) == Some(0));
    assert!(foo == Some(1));
    assert!(bar.at(()).replace(2) == None);
    assert!(bar == None);

    let mut foo: Result<i32,i32> = Ok(0);
    let mut bar: Result<i32,i32> = Err(1);
    
    assert!(foo.at(()).replace(1) == Some(0));
    assert!(foo == Ok(1));
    assert!(bar.at(()).replace(2) == None);
    assert!(bar == Err(1));
}


