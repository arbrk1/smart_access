#[cfg(any(feature="batch_ct", feature="batch_rt"))]
use crate::batch::{ CpsBatch };

#[cfg(feature="batch_ct")]
use crate::batch::{ new_batch_ct };

#[cfg(feature="batch_rt")]
use crate::batch::{ new_batch_rt, FnBoxRt };

#[cfg(feature="detach")]
mod detach; // detached paths

#[cfg(feature="detach")]
use detach::{ DetachedRoot };

#[cfg(feature="detach")]
pub use detach::{ Attach };



/// A smart access protocol.
///
/// It is intended to be used through a [`Cps`](trait.Cps.html)-bounded type.
pub trait At<Index> {
    type View: ?Sized;

    /// Accesses data at a specified index.
    ///
    /// If there is some data (or some bidirectional procedure) associated
    /// with the index then `access_at` must apply `f` to this data.
    ///
    /// If the transformation result can be placed back into `self` then
    /// it must be placed back and `access_at` must return `Some(f(data))`.
    ///
    /// Otherwise `None` __must__ be returned and `self` must stay unchanged.
    ///
    /// In essence `access_at` returns `None` if and only if `self` has
    /// not been touched.
    ///
    /// ### Note
    ///
    /// The following two cases are indistinguishable:
    /// 
    /// * a view couldn't be obtained (and thus `f` had not been called)
    /// * `f` had been called but failed to mutate the view in a meaningful way
    ///
    /// If you need to distinguish between these cases you can use some side-effect of `f`.
    fn access_at<R, F>(&mut self, i: Index, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R;
}


/// Anything that can provide (or refuse to provide) a mutable parameter 
/// for a function.
///
/// You __do not need__ to implement `Cps` for anything: it's already implemented 
/// for [`AT`](struct.AT.html) and `&mut T`, and it's sufficient for almost all 
/// purposes. Implement [`At`](trait.At.html) instead.
///
/// The main usecase for this trait is to be used as a bound on 
/// parameter and return types of functions:
/// `Cps<View=T>`-bounded type can be thought of as a 
/// lifetimeless analogue of `&mut T`.
///
/// In fact all default implementors of `Cps` have an internal lifetime 
/// parameter. If needed it can be exposed using `+ 'a` syntax in a trait 
/// bound, but in many cases one can do very well without any explicit lifetimes.
pub trait Cps: Sized {
    type View: ?Sized;

    /// Returns `Some(f(..))` or `None`.
    ///
    /// The rules governing the value returned are defined by an implementation.
    fn access<R, F>(self, f: F) -> Option<R> where
        F: FnOnce(&mut Self::View) -> R;

    /// Equivalent to `self.access(|x| std::mem::replace(x, new_val))`
    fn replace(self, new_val: Self::View) -> Option<Self::View> where
        Self::View: Sized 
    {
        self.access(|x| core::mem::replace(x, new_val))
    }

    /// Equivalent to `self.access(|_| ())`
    fn touch(self) -> Option<()> where
    {
        self.access(|_| ())
    }

    /// &#8220;Moves in the direction&#8221; of the provided index.
    ///
    /// __Not intended for overriding.__
    fn at<Index>(self, i: Index) -> AT<Self, ((), Index)> where
        Self::View: At<Index>
    {
        AT { cps: self, list: ((), i) } 
    }

    #[cfg(feature="batch_ct")]
    /// Constructs a [compile-time batch](struct.CpsBatch.html).
    ///
    /// __Not intended for overriding.__
    ///
    /// _Present only on `batch_ct`._
    fn batch_ct(self) -> CpsBatch<Self, ()> {
        new_batch_ct(self)
    }

    #[cfg(feature="batch_rt")]
    /// Constructs a [runtime batch](struct.CpsBatch.html).
    ///
    /// __Not intended for overriding.__
    ///
    /// _Present only on `batch_rt`._
    fn batch_rt<R>(self) -> CpsBatch<Self, Vec<FnBoxRt<Self::View, R>>> {
        new_batch_rt(self)
    }

    #[cfg(feature="detach")]
    /// Attaches a [detached](trait.Attach.html) path.
    ///
    /// __Not intended for overriding.__
    ///
    /// _Present only on `detach`._
    fn attach<Path, V: ?Sized>(self, path: Path) -> AT<Self, Path::List> where
        Path: Attach<Self::View, View=V>,
    {
        path.attach_to(self)
    }

    #[cfg(feature="detach")]
    /// Creates a new detach point.
    ///
    /// __Not intended for overriding.__
    ///
    /// _Present only on `detach`._
    ///
    /// ### Usage example
    ///
    /// The [`.detach()`](struct.AT.html#method.detach) method 
    /// detaches a part beginning at the closest detach point:
    ///
    /// ```
    /// # use smart_access::Cps;
    /// let mut foo = Some(Some(1));
    /// let mut bar = Some(2);
    ///
    /// //                              the detached part
    /// //                                  /------\
    /// let (left, right) = foo.at(()).cut().at(()).detach();
    ///
    /// assert!(bar.attach(right).replace(3) == Some(2));
    /// assert!(bar == Some(3));
    ///
    /// assert!(left.at(()).replace(4) == Some(1));
    /// assert!(foo == Some(Some(4)));
    /// ```
    fn cut(self) -> AT<Self, ()>
    {
        AT { cps: self, list: () }
    } 
}


/// `access` is guaranteed to return `Some(f(..))`
impl<T: ?Sized> Cps for &mut T {
    type View = T;
    
    fn access<R, F>(self, f: F) -> Option<R> where
        F: FnOnce(&mut T) -> R
    {
        Some(f(self))
    }
}




/// A &#8220;reference&#8221; to some &#8220;location&#8221;.
///
/// With default [`Cps`](trait.Cps.html) implementations
/// (and with the `detach` feature disabled) every `AT` is 
/// usually a &#8220;path component&#8221; list of type
///
/// `AT<&mut root, (..((((), I1), I2), I3) .. In)>`
///
/// But beware! Starting with version `0.5` there is a possibility to create
/// multilevel hierarchies like
///
/// `AT<AT<&mut root, ((), I1)>, ((), I2)>`
///
/// by using the [`at` of `Cps`](trait.Cps.html#method.at) instead of 
/// its [`AT`-override](#method.at).
/// 
/// Moreover, the `detach` feature is now based on such nonlinear structures.
///
/// Though `AT` is exposed, it's strongly recommended to use
/// [`impl Cps<View=T>`](trait.Cps.html) as a return type of your functions 
/// and [`Cps<View=T>`](trait.Cps.html) bounds on their parameters.
///
/// Enabling `detach` feature allows one to [detach](#method.detach) `AT`s from their roots. 
///
/// Without this feature only a single component can be detached:
///
/// ```
/// use smart_access::Cps;
///
/// let mut foo = vec![vec![1,2], vec![3,4]];
///
/// let (foo_i, j) = foo.at(0).at(0).into();
/// assert!(foo_i.at(1).replace(5) == Some(2));
/// ```
/// 
/// ### Note
///
/// _Relevant only with the `detach` feature enabled._
///
/// If you pass a detached path to a function then you should use 
/// a [`Path: Attach<CPS::View, View=V>`](trait.Attach.html) bound 
/// instead of a [`Cps<View=V>`](trait.Cps.html) bound.
///
/// I.e.
///
/// ```
/// # #[cfg(feature="detach")] fn test() {
/// # use smart_access::{Cps, Attach, detached_at};
/// fn replace_at<CPS: Cps, Path, V>(cps: CPS, path: Path, x: V) -> Option<V> where
///     Path: Attach<CPS::View, View=V>,
/// {
///     cps.attach(path).replace(x)
/// }
///
/// let mut vec = vec![1,2,3];
///
/// assert!(replace_at(&mut vec, detached_at(0), 4) == Some(1));
/// assert!(vec == vec![4,2,3]);
/// # }
/// # #[cfg(not(feature="detach"))] fn test() {}
/// # test();
/// ```
///
/// But sometimes an explicit `AT` can be useful (the example below 
/// is artificial and thus not very illuminating...): 
///
/// ```
/// # #[cfg(feature="detach")] fn test() {
/// use smart_access::*;
///
/// fn get_ij<CPS, U, V, W>(a_i: AT<CPS, ((), usize)>, j: usize) 
///     -> impl Attach<W, View=V> where 
///     CPS: Cps<View=W>,
///     W: At<usize, View=U> + ?Sized,
///     U: At<usize, View=V> + ?Sized,
///     V: ?Sized,
/// {
///     let (a,i) = a_i.into();
///     let (_, path) = a.at(i).at(j).detach();
///
///     path
/// }
///
/// let mut foo = vec![vec![1,2], vec![3,4]];
/// let path = get_ij(detached_at(1), 0);
/// 
/// assert!(foo.attach(path).replace(5) == Some(3));
/// # }
/// # #[cfg(not(feature="detach"))] fn test() {}
/// # test();
/// ```
#[must_use]
#[cfg_attr(feature="detach", derive(Clone))]
#[derive(Debug)]
pub struct AT<CPS, List> { 
    cps: CPS, 
    list: List,
}

/// `access` returns `Some` / `None` according to the rules described [here](trait.At.html)
impl<CPS: Cps, Path> Cps for AT<CPS, Path> where
    Path: AtView<CPS::View>
{
    type View = Path::View;
    
    fn access<R, F>(self, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        self.list.give_access(self.cps, f)
    }
}



impl<CPS, List> AT<CPS, List> {
    /// Override for [`at` of `Cps`](trait.Cps.html#method.at).
    ///
    /// Preserves flat structure.
    pub fn at<Index, View: ?Sized>(self, i: Index) -> AT<CPS, (List, Index)> where
        AT<CPS, List>: Cps<View=View>,
        View: At<Index>
    {
        AT { cps: self.cps, list: (self.list, i) } 
    }
}




/// `AT` can be broken apart to detach a single path component.
///
/// A more general attach/detach framework is accessible 
/// through the `detach` feature.
impl<CPS,Prev,I> From<AT<CPS,(Prev,I)>> for (AT<CPS,Prev>,I) 
{
    fn from(at: AT<CPS,(Prev,I)>) -> Self {
        let (prev, index) = at.list;

        (AT { cps: at.cps, list: prev}, index)
    }
}


#[cfg(feature="detach")]
impl<CPS: Cps, List> AT<CPS, List> {

/// Detaches the path starting from the [detach point](trait.Cps.html#method.cut).
///
/// _Present only on `detach`._
///
/// ### Usage example
///
/// ```
/// use smart_access::Cps;
///
/// let mut foo = vec![vec![vec![0]]];
/// let mut bar = vec![vec![vec![0]]];
///
/// let (_, detached) = foo.at(0).at(0).at(0).detach();
///
/// // Detached paths are cloneable (if indices are cloneable)
/// let the_same_path = detached.clone();
///
/// bar.attach(the_same_path).replace(1);
/// assert!(foo == vec![vec![vec![0]]]);
/// assert!(bar == vec![vec![vec![1]]]);
///
/// foo.attach(detached).replace(2);
/// assert!(foo == vec![vec![vec![2]]]);
/// assert!(bar == vec![vec![vec![1]]]);
/// 
/// let (_, path) = bar.at(0).at(0).detach();
/// bar.attach(path.at(0)).replace(3);
/// assert!(bar == vec![vec![vec![3]]]);
/// ```
    pub fn detach(self) -> (CPS, AT<DetachedRoot<CPS::View>, List>) {
        (self.cps, AT { cps: DetachedRoot::new(), list: self.list })
    }
}


/// Creates a detached path. __Requires `detach` feature.__
///
/// A type of the return value of `detached_at::<V>` 
/// implements [`Attach<CPS: Cps<View=V>, View=V>`](trait.Attach.html).
///
/// _Present only on `detach`._
///
/// ### Usage example
///
/// A simple case when detached paths could be helpful: creating 
/// a detached path and cloning it several times.
///
/// ```
/// use smart_access::Cps;
///
/// let reference_path = smart_access::detached_at(()).at(()).at(());
///
/// let mut items = vec![ Some(Some(Ok(1))), Some(None), Some(Some(Err(2))) ];
///
/// let sum = items.iter_mut().map(|wrapped| {
///     wrapped.attach(reference_path.clone())
///         .access(|x| *x) 
///         .into_iter() 
///         .sum::<i32>()
/// }).sum::<i32>();
///
/// assert!(sum == 1);
/// ```
///
/// A more convoluted example: a functional index combinator.
///
/// ```
/// use smart_access::{Attach, Cps};
///
/// type Mat = Vec<Vec<f64>>;
///
/// fn mat_index(i: usize, j: usize) -> impl Attach<Mat, View=f64> {
///     smart_access::detached_at(i).at(j)
/// }
///
/// let mut mat = vec![
///     vec![1., 2.],
///     vec![3., 4.]
/// ];
///
/// assert!(mat.attach(mat_index(1,1)).replace(0.) == Some(4.));
/// ```
/// 
/// But note that a more idiomatic approach would be
///
/// ```
/// use smart_access::{Cps, At};
///
/// struct Mat { numbers: Vec<Vec<f64>> };
///
/// impl At<(usize, usize)> for Mat {
///     type View = f64;
///
///     fn access_at<R,F>(&mut self, ij: (usize, usize), f: F) -> Option<R> where
///         F: FnOnce(&mut f64) -> R
///     {
///         let (i, j) = ij;
///
///         self.numbers.at(i).at(j).access(f)
///     }
/// }
///
/// let mut mat = Mat { numbers: vec![
///     vec![1., 2.],
///     vec![3., 4.]
/// ]};
///
/// assert!(mat.at( (1,1) ).replace(0.) == Some(4.));
/// ```
#[cfg(feature="detach")]
pub fn detached_at<View: ?Sized, I>(i: I) -> AT<DetachedRoot<View>, ((), I)> where
    View: At<I>
{
    AT {
        cps: DetachedRoot::new(),
        list: ((), i),
    }
}




/*
/// A [detach point](trait.Cps.html#method.cut).
///
/// Even without `detach` it is used to stop trait recursion.
impl<CPS: Cps> Cps for AT<CPS, ()> {
    type View = CPS::View;
    
    fn access<R, F>(self, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        self.cps.access(f)
    }
}


/// `access` returns `Some` / `None` according to the rules described [here](trait.At.html)
impl<CPS: Cps, Prev, Index, View: ?Sized> Cps for AT<CPS, (Prev, Index)> where
    AT<CPS, Prev>: Cps<View=View>,
    View: At<Index>
{
    type View = View::View;
    
    fn access<R, F>(self, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        let (prev, index) = self.list;
        let at = AT { cps: self.cps, list: prev };

        at.access(|v| { v.access_at(index, f) }).flatten()
    }
}*/


/// A trait which is usually needed alongside [`Attach`](trait.Attach.html) bounds.
///
/// __Update: seems to be not needed now!__
///
/// Essentially it's a type-level function mapping the `View` type of a 
/// `Cps`-bounded value `x` and a path type of the form `(..((), I1), .. In)`
/// to the `View` type of the value
///
/// `x.at(i1) .. .at(in)`
/// 
/// Technically it's a workaround for the inability of the 
/// Rust compiler to reliably infer types in presence of 
/// flexible (as in Haskell's `FlexibleContexts`) recurrent contexts.
pub trait AtView<View: ?Sized>: Sized {
    type View: ?Sized;

    fn give_access<CPS, R, F>(self, cps: CPS, f: F) -> Option<R> where
        CPS: Cps<View=View>,
        F: FnOnce(&mut Self::View) -> R;
}


impl<View: ?Sized> AtView<View> for () {
    type View = View;
    
    fn give_access<CPS, R, F>(self, cps: CPS, f: F) -> Option<R> where
        CPS: Cps<View=View>,
        F: FnOnce(&mut Self::View) -> R
    {
        cps.access(f)
    }
}

impl<View: ?Sized, Prev, Index> AtView<View> for (Prev, Index) where
    Prev: AtView<View>,
    Prev::View: At<Index>
{
    type View = <Prev::View as At<Index>>::View;
    
    fn give_access<CPS, R, F>(self, cps: CPS, f: F) -> Option<R> where
        CPS: Cps<View=View>,
        F: FnOnce(&mut Self::View) -> R
    {
        let (prev, index) = self;

        prev.give_access(cps, |v| { v.access_at(index, f) }).flatten()
    }
}



