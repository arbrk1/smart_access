use super::*;
use core::marker::PhantomData;

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


/// A concrete type of [detached](struct.AT.html#method.detach) paths. __Requires `detach` feature.__
///
/// Usually [`impl Attach<V, View=W>`](trait.Attach.html) 
/// is preferable to `DetachedPath<V, List>` 
/// but sometimes the latter prevents fighting with the borrow checker 
/// (`impl Trait` returned from a function holds all the references 
/// passed to generically typed arguments of the function).
pub type DetachedPath<View, List> = AT<DetachedRoot<View>, List>;


/// A detached path. __Requires `detach` feature.__
///
/// Can be created by the [`detached_at`](fn.detached_at.html) function.
///
/// See examples [here](struct.AT.html) and [here](fn.detached_at.html).
pub trait Attach<View: ?Sized>: Sized {
    type List: AtView<View, View=Self::View>;
    type View: ?Sized;

    fn attach_to<CPS>(self, cps: CPS) -> AT<CPS, Self::List> where
        CPS: Cps<View=View>;
}

impl<ToView: ?Sized, List> Attach<ToView> for DetachedPath<ToView, List> where
    List: AtView<ToView>
{
    type List = List;
    type View = List::View;

    fn attach_to<CPS>(self, cps: CPS) -> AT<CPS, Self::List> where
        CPS: Cps<View=ToView>
    {
        AT { cps: cps, list: self.list }
    }
}


