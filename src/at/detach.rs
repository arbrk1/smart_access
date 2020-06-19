use super::*;
use core::marker::PhantomData;

#[derive(Debug)]
pub struct AttachedRoot<T>(pub T);

#[derive(Debug, Clone)]
pub struct DetachedRoot<V: ?Sized>(PhantomData<*const V>);


/// A helper for attached paths.
///
/// Forwards the access query to the wrapped type.
impl<T> Cps for AttachedRoot<T> where
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


pub trait Detach {
    type Output;

    fn detach(self) -> Self::Output;
}

impl<CPS: Cps> Detach for AttachedRoot<CPS> {
    type Output = DetachedRoot<CPS::View>;

    fn detach(self) -> Self::Output {
        DetachedRoot(PhantomData)
    }
}

impl<Prev, Index> Detach for AT<Prev, Index> where
    Prev: Detach
{
    type Output = AT<Prev::Output, Index>;

    fn detach(self) -> Self::Output {
        AT { prev: self.prev.detach(), index: self.index }
    }
}


pub trait Attach<CPS: Cps> {
    type Output;

    fn attach(self, cps: CPS) -> Self::Output;
}


impl<CPS: Cps> Attach<CPS> for DetachedRoot<CPS::View> {
    type Output = AttachedRoot<CPS>;

    fn attach(self, cps: CPS) -> Self::Output {
        AttachedRoot(cps)
    }
}

impl<CPS: Cps, Prev, Index> Attach<CPS> for AT<Prev, Index> where
    Prev: Attach<CPS>
{
    type Output = AT<Prev::Output, Index>;

    fn attach(self, cps: CPS) -> Self::Output {
        AT { prev: self.prev.attach(cps), index: self.index }
    }
}


