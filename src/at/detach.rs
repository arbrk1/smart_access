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


/// A detached path. __Requires `detach` feature.__
///
/// Intended to be used as an `Attach<CPS, View=V>` bound.
///
/// Can be created by a [`detached_at`](fn.detached_at.html) function.
///
/// See examples [here](struct.AT.html) and [here](fn.detached_at.html).
pub trait Attach {
    type List;

    type ToView: ?Sized;
    type View: ?Sized;

    fn attach_to<CPS: Cps<View=Self::ToView>>(self, cps: CPS) -> AT<CPS, Self::List>;
}


impl<ToView: ?Sized, List, View: ?Sized> Attach for 
AT<DetachedRoot<ToView>, List> where
    Self: Cps<View=View>
{
    type List = List;

    type ToView = ToView;
    type View = View;

    fn attach_to<CPS: Cps<View=ToView>>(self, cps: CPS) -> AT<CPS, Self::List>
    {
        AT { cps: cps, list: self.list }
    }
}

/* ?????????????
trait Transform<View: ?Sized> {
    type View: ?Sized;
}

impl<View: ?Sized> Transform<View> for () {
    type View = View;
}

impl<Prev, Index, U: ?Sized, V: ?Sized> Transform<U> for (Prev, Index) where
    Prev: Transform<U>,
    Prev::View: At<Index, View=V>
{
    type View = V;
}*/



/*
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
}*/

