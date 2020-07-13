//! Support for general traversals. __Requires `traversal`.__
//!
//! This module is essentially a more efficient version of 
//! [`iter_mut`](../iter_mut/) with some quirks.
//!
//! The API for traversals mimics the API for affine traversals:
//!
//! * [`Of<Index, View=V>`](trait.Of.html) corresponds to 
//!   [`At<Index, View=V>`](../trait.At.html)
//! * [`Each<View=V>`](trait.Each.html) corresponds to 
//!   [`Cps<View=V>`](../trait.Cps.html)
//!
//! Currently only the basics are implemented: the `()` accessor 
//! can be used to transform (a mutable reference to) any iterator 
//! into an `Each`-bound value:
//!
//! ```
//! use smart_access::traversal::Each;
//!
//! let mut foo = vec![vec![1, 2], vec![3, 4]];
//!
//! foo.iter_mut().of(()).each(|subvector| {
//!     subvector.iter_mut().of(()).each(|x| {
//!         *x += 1; true  // true means that the iteration must continue
//!     })
//! });
//!
//! assert!(foo == vec![vec![2, 3], vec![4, 5]]);
//!
//! foo.iter_mut().of(()).each(|subvector| {
//!     subvector.iter_mut().of(()).each(|x| {
//!         *x = 6; false  // false means that the iteration must stop
//!     })
//! });
//! 
//! assert!(foo == vec![vec![6, 3], vec![6, 5]]);
//! ```

use crate::AT;

mod internal;
use internal::OfView;



/// An analogue of the [`At`](../trait.At.html) trait.
pub trait Of<Index> where
    Index: Clone
{
    type View: ?Sized;
    
    /// Traverses the view.
    ///
    /// If `f` returns `false`, the iteration must stop.
    ///
    /// The same goes for `each_of`: usually you want to 
    /// return `true` from it. But there may be exceptions
    /// when returning `false` is more convenient.
    fn each_of<F>(&mut self, i: Index, f: F) -> bool where
        F: FnMut(&mut Self::View) -> bool;
}


impl<'a, I, T: 'a> Of<()> for I where
    I: Iterator<Item=&'a mut T>
{
    type View = T;

    fn each_of<F>(&mut self, _: (), mut f: F) -> bool where
        F: FnMut(&mut Self::View) -> bool
    {
        for x in self { 
            if !f(x) { break }
        }

        true
    }
}


/// An analogue of the [`Cps`](../trait.Cps.html) trait.
pub trait Each: Sized {
    type View: ?Sized;

    fn each<F>(self, f: F) -> bool where
        F: FnMut(&mut Self::View) -> bool;

    fn of<Index>(self, i: Index) -> AT<Self, ((), Index)> where
        Self::View: Of<Index>,
        Index: Clone
    {
        AT { cps: self, list: ((), i) } 
    }
}


impl<CPS: Each, Path> Each for AT<CPS, Path> where
    Path: OfView<CPS::View>
{
    type View = Path::View;
    
    fn each<F>(self, f: F) -> bool where 
        F: FnMut(&mut Self::View) -> bool
    {
        self.list.give_access(self.cps, f)
    }
}


impl<T: ?Sized> Each for &mut T {
    type View = T;
    
    fn each<F>(self, mut f: F) -> bool where
        F: FnMut(&mut T) -> bool
    {
        f(self)
    }
}


