use crate::at::At;
use std::ops;

impl<T> At<()> for Vec<T> 
{
    type View = [T];

    fn access_at<R, F>(&mut self, _: (), f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        Some(f(self as &mut [T]))
    }
}


impl<T> At<usize> for Vec<T> 
{
    type View = T;

    fn access_at<R, F>(&mut self, i: usize, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        (self as &mut [T]).access_at(i,f)
    }
}


impl<T> At<ops::Range<usize>> for Vec<T> {
    type View = Vec<T>;
    
    fn access_at<R, F>(&mut self, i: ops::Range<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        if i.end > self.len() { return None; }
        if i.start > i.end    { return None; }

        let right_part   = self.split_off(i.end);
        let mut mid_part = self.split_off(i.start);

        let result = f(&mut mid_part);
        
        self.extend(mid_part);
        self.extend(right_part);

        Some(result)
    }
}


impl<T> At<ops::RangeFrom<usize>> for Vec<T> {
    type View = Vec<T>;
    
    fn access_at<R, F>(&mut self, i: ops::RangeFrom<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        if i.start > self.len() { return None; }

        let mut mid_part = self.split_off(i.start);

        let result = f(&mut mid_part);
        
        self.extend(mid_part);

        Some(result)
    }
}


impl<T> At<ops::RangeFull> for Vec<T> {
    type View = Vec<T>;
    
    fn access_at<R, F>(&mut self, _: ops::RangeFull, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        Some(f(self))
    }
}


impl<T> At<ops::RangeInclusive<usize>> for Vec<T> {
    type View = Vec<T>;
    
    fn access_at<R, F>(&mut self, i: ops::RangeInclusive<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        let (start, end) = (*i.start(), *i.end());

        if end >= self.len() { return None; }

        // overflow is prevented by the previous line
        if start > end+1   { return None; }

        let right_part   = self.split_off(end+1);
        let mut mid_part = self.split_off(start);

        let result = f(&mut mid_part);
        
        self.extend(mid_part);
        self.extend(right_part);

        Some(result)
    }
}


impl<T> At<ops::RangeTo<usize>> for Vec<T> {
    type View = Vec<T>;
    
    fn access_at<R, F>(&mut self, i: ops::RangeTo<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        if i.end > self.len() { return None; }

        let right_part = self.split_off(i.end);

        let result = f(self);
        
        self.extend(right_part);

        Some(result)
    }
}


impl<T> At<ops::RangeToInclusive<usize>> for Vec<T> {
    type View = Vec<T>;
    
    fn access_at<R, F>(&mut self, i: ops::RangeToInclusive<usize>, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        if i.end >= self.len() { return None; }

        let right_part = self.split_off(i.end+1);

        let result = f(self);
        
        self.extend(right_part);

        Some(result)
    }
}
