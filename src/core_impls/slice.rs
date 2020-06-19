use crate::at::At;
use core::ops;


impl<T> At<usize> for [T] {
    type View = T;

    fn access_at<R, F>(&mut self, i: usize, f: F) -> Option<R> where
        F: FnOnce(&mut T) -> R
    {
        match self.get_mut(i) {
            None => None,
            Some(x) => Some(f(x)),
        }
    }
}


impl<T> At<ops::Range<usize>> for [T] {
    type View = [T];
    
    fn access_at<R, F>(&mut self, i: ops::Range<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        if i.end > self.len() { return None; }
        if i.start > i.end    { return None; }

        Some(f(&mut self[i]))
    }
}


impl<T> At<ops::RangeFrom<usize>> for [T] {
    type View = [T];
    
    fn access_at<R, F>(&mut self, i: ops::RangeFrom<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        if i.start > self.len() { return None; }

        Some(f(&mut self[i]))
    }
}


impl<T> At<ops::RangeFull> for [T] {
    type View = [T];
    
    fn access_at<R, F>(&mut self, _: ops::RangeFull, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        Some(f(self))
    }
}


impl<T> At<ops::RangeInclusive<usize>> for [T] {
    type View = [T];
    
    fn access_at<R, F>(&mut self, i: ops::RangeInclusive<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        let (start, end) = (*i.start(), *i.end());

        if end >= self.len() { return None; }

        // overflow is prevented by the previous line
        if start > end+1 { return None; }

        Some(f(&mut self[i]))
    }
}


impl<T> At<ops::RangeTo<usize>> for [T] {
    type View = [T];
    
    fn access_at<R, F>(&mut self, i: ops::RangeTo<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        if i.end > self.len() { return None; }

        Some(f(&mut self[i]))
    }
}


impl<T> At<ops::RangeToInclusive<usize>> for [T] {
    type View = [T];
    
    fn access_at<R, F>(&mut self, i: ops::RangeToInclusive<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        if i.end >= self.len() { return None; }

        Some(f(&mut self[i]))
    }
}

