use super::*;
use core::marker::PhantomData;

#[derive(Debug)]
pub struct DetachPoint<T>(pub T);

#[derive(Debug, Clone)]
pub struct DetachedRoot<V: ?Sized>(PhantomData<*const V>);

impl<V: ?Sized> DetachedRoot<V> {
    pub fn new() -> Self {
        DetachedRoot(PhantomData)
    }
}


/// A helper for attached paths.
///
/// Forwards the access query to the wrapped type.
impl<T> Cps for DetachPoint<T> where
    T: Cps
{    
    type View = T::View;
    
    fn access<R, F>(self, f: F) -> Option<R> where
        F: FnOnce(&mut Self::View) -> R
    {
        self.0.access(f)
    }
}


/// A helper for detached paths.
///
/// `access` returns `None`.
impl<V: ?Sized> Cps for DetachedRoot<V> {
    type View = V;
    
    fn access<R, F>(self, _: F) -> Option<R> where
        F: FnOnce(&mut V) -> R
    {
        None
    }
}

#[must_use]
pub trait Detach {
    type Left;
    type Right;

    fn detach(self) -> (Self::Left, Self::Right);
}

impl<CPS: Cps> Detach for DetachPoint<CPS> {
    type Left = CPS;
    type Right = DetachedRoot<CPS::View>;

    fn detach(self) -> (Self::Left, Self::Right) {
        (self.0, DetachedRoot(PhantomData))
    }
}

impl<Prev, Index> Detach for AT<Prev, Index> where
    Prev: Detach
{
    type Left = Prev::Left;
    type Right = AT<Prev::Right, Index>;

    fn detach(self) -> (Self::Left, Self::Right) {
        let (left, right) = self.prev.detach();

        (left, AT { prev: right, index: self.index })
    }
}


/// A detached path. __Requires `detach` feature.__
///
/// Intended to be used as an `Attach<CPS, View=V>` bound.
///
/// Can be created by a [`detached_at`](fn.detached_at.html) function.
///
/// See examples [here](struct.AT.html) and [here](fn.detached_at.html).
///
/// __Warning!__ In the next version of the crate this trait 
/// very likely will become `Attach<View>`.
pub trait Attach<CPS: Cps> {
    type Output: Cps<View=Self::View>;
    type View: ?Sized;

    fn attach(self, cps: CPS) -> Self::Output;
}


impl<CPS: Cps> Attach<CPS> for DetachedRoot<CPS::View> {
    type Output = DetachPoint<CPS>;
    type View = CPS::View;

    fn attach(self, cps: CPS) -> Self::Output {
        DetachPoint(cps)
    }
}

impl<CPS: Cps, Prev, Index, V: ?Sized> Attach<CPS> for AT<Prev, Index> where
    Prev: Attach<CPS>,
    Prev::View: At<Index, View=V>,
{
    type Output = AT<Prev::Output, Index>;
    type View = V;

    fn attach(self, cps: CPS) -> Self::Output {
        AT { prev: self.prev.attach(cps), index: self.index }
    }
}


