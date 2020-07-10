use crate::AT;

pub trait EachFrom<Index> where
    Index: Clone
{
    type View: ?Sized;

    fn each_from<F>(&mut self, i: Index, f: &mut F) where
        F: FnMut(&mut Self::View);
}


pub trait Each: Sized {
    type View: ?Sized;

    fn each<F>(self, f: F) where
        F: FnMut(&mut Self::View);

    fn from<Index>(self, i: Index) -> AT<Self, ((), Index)> where
        Self::View: EachFrom<Index>,
        Index: Clone;
}


pub trait EachView<View: ?Sized>: Sized {
    type View: ?Sized;

    fn give_access<CPS, F>(self, cps: CPS, f: &mut F) where
        CPS: Each<View=View>,
        F: FnMut(&mut Self::View);
}


impl<View: ?Sized> EachView<View> for () {
    type View = View;
    
    fn give_access<CPS, F>(self, cps: CPS, f: &mut F) where
        CPS: Each<View=View>,
        F: FnMut(&mut Self::View)
    {
        cps.each(f)
    }
}

impl<View: ?Sized, Prev, Index> EachView<View> for (Prev, Index) where
    Prev: EachView<View>,
    Prev::View: EachFrom<Index>,
    Index: Clone
{
    type View = <Prev::View as EachFrom<Index>>::View;
    
    fn give_access<CPS, F>(self, cps: CPS, f: &mut F) where
        CPS: Each<View=View>,
        F: FnMut(&mut Self::View)
    {
        let (prev, index) = self;

        prev.give_access(cps, &mut |v| { v.each_from(index.clone(), f) });
    }
}


