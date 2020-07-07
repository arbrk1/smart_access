use crate::At;
use core::ops;
use multiref::Slice;

impl<T> At<usize> for Slice<T> {
    type View = T;

    fn access_at<R, F>(&mut self, i: usize, f: F) -> Option<R> where
        F: FnOnce(&mut T) -> R
    {
        self.as_mut().access_at(i, |x| f(*x))
    }
}


impl<T> At<ops::Range<usize>> for Slice<T> {
    type View = Slice<T>;
    
    fn access_at<R, F>(&mut self, i: ops::Range<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        self.as_mut().access_at(i, |subslice| f(Slice::new_mut(subslice)))
    }
}


impl<T> At<ops::RangeFrom<usize>> for Slice<T> {
    type View = Slice<T>;
    
    fn access_at<R, F>(&mut self, i: ops::RangeFrom<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        self.as_mut().access_at(i, |subslice| f(Slice::new_mut(subslice)))
    }
}


impl<T> At<ops::RangeFull> for Slice<T> {
    type View = Slice<T>;
    
    fn access_at<R, F>(&mut self, i: ops::RangeFull, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        self.as_mut().access_at(i, |subslice| f(Slice::new_mut(subslice)))
    }
}


impl<T> At<ops::RangeInclusive<usize>> for Slice<T> {
    type View = Slice<T>;
    
    fn access_at<R, F>(&mut self, i: ops::RangeInclusive<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        self.as_mut().access_at(i, |subslice| f(Slice::new_mut(subslice)))
    }
}


impl<T> At<ops::RangeTo<usize>> for Slice<T> {
    type View = Slice<T>;
    
    fn access_at<R, F>(&mut self, i: ops::RangeTo<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        self.as_mut().access_at(i, |subslice| f(Slice::new_mut(subslice)))
    }
}


impl<T> At<ops::RangeToInclusive<usize>> for Slice<T> {
    type View = Slice<T>;
    
    fn access_at<R, F>(&mut self, i: ops::RangeToInclusive<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        self.as_mut().access_at(i, |subslice| f(Slice::new_mut(subslice)))
    }
}

