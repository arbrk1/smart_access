use super::*;

pub trait OfView<View: ?Sized>: Sized {
    type View: ?Sized;

    fn give_access<CPS, F>(self, cps: CPS, f: F) -> bool where
        CPS: Each<View=View>,
        F: FnMut(&mut Self::View) -> bool;
}


impl<View: ?Sized> OfView<View> for () {
    type View = View;
    
    fn give_access<CPS, F>(self, cps: CPS, f: F) -> bool where
        CPS: Each<View=View>,
        F: FnMut(&mut Self::View) -> bool
    {
        cps.each(f)
    }
}

impl<View: ?Sized, Prev, Index> OfView<View> for (Prev, Index) where
    Prev: OfView<View>,
    Prev::View: Of<Index>,
    Index: Clone
{
    type View = <Prev::View as Of<Index>>::View;
    
    fn give_access<CPS, F>(self, cps: CPS, mut f: F) -> bool where
        CPS: Each<View=View>,
        F: FnMut(&mut Self::View) -> bool
    {
        let (prev, index) = self;

        prev.give_access(cps, |v| { v.each_of(index.clone(), &mut f) })
    }
}


