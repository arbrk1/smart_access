#[cfg(feature="batch_rt")]
use super::*;

// trait used mainly for compile-time constructed call chains
// private to crate::at
pub trait RunBatch<View: ?Sized> {
    type Result;

    fn run(self, view: &mut View) -> Self::Result;
}


#[cfg(feature="batch_ct")]
impl<View: ?Sized> RunBatch<View> for () 
{
    type Result = ();

    fn run(self, _view: &mut View) -> () { () }
}

#[cfg(feature="batch_ct")]
impl<View: ?Sized, Prev, F, R> RunBatch<View> for (Prev, F) where
    Prev: RunBatch<View>,
    F: FnOnce(&mut View, Prev::Result) -> R
{
    type Result = R;

    fn run(self, view: &mut View) -> R {
        let tmp = self.0.run(view);

        self.1(view, tmp)
    }
}

#[cfg(feature="batch_rt")]
impl<View: ?Sized, R> RunBatch<View> for Vec<FnBoxRt<View, R>> {
    type Result = Option<R>;

    fn run(self, view: &mut View) -> Option<R> {
        let mut current_result = None;

        for f in self {
            current_result = Some(f(view, current_result));
        }

        current_result
    }
}

