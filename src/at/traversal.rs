/// General traversals. __WIP: absolutely not ready for use; anything may change!!!__

use crate::AT;

pub trait EachOf<Index> where
    Index: Clone
{
    type View: ?Sized;

    fn each_of<F>(&mut self, i: Index, f: F) where
        F: FnMut(&mut Self::View);
}


pub trait Each: Sized {
    type View: ?Sized;

    fn each<F>(self, f: F) where
        F: FnMut(&mut Self::View);

    fn of<Index>(self, i: Index) -> AT<Self, ((), Index)> where
        Self::View: EachOf<Index>,
        Index: Clone
    {
        AT { cps: self, list: ((), i) } 
    }
}


impl<CPS: Each, Path> Each for AT<CPS, Path> where
    Path: EachView<CPS::View>
{
    type View = Path::View;
    
    fn each<F>(self, f: F) where 
        F: FnMut(&mut Self::View)
    {
        self.list.give_access(self.cps, f)
    }
}


pub trait EachView<View: ?Sized>: Sized {
    type View: ?Sized;

    fn give_access<CPS, F>(self, cps: CPS, f: F) where
        CPS: Each<View=View>,
        F: FnMut(&mut Self::View);
}


impl<View: ?Sized> EachView<View> for () {
    type View = View;
    
    fn give_access<CPS, F>(self, cps: CPS, f: F) where
        CPS: Each<View=View>,
        F: FnMut(&mut Self::View)
    {
        cps.each(f)
    }
}

impl<View: ?Sized, Prev, Index> EachView<View> for (Prev, Index) where
    Prev: EachView<View>,
    Prev::View: EachOf<Index>,
    Index: Clone
{
    type View = <Prev::View as EachOf<Index>>::View;
    
    fn give_access<CPS, F>(self, cps: CPS, mut f: F) where
        CPS: Each<View=View>,
        F: FnMut(&mut Self::View)
    {
        let (prev, index) = self;

        prev.give_access(cps, |v| { v.each_of(index.clone(), &mut f) });
    }
}


