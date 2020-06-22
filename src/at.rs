#[cfg(any(feature="batch_ct", feature="batch_rt"))]
use crate::batch::{ CpsBatch };

#[cfg(feature="batch_ct")]
use crate::batch::{ new_batch_ct };

#[cfg(feature="batch_rt")]
use crate::batch::{ new_batch_rt, FnBoxRt };

#[cfg(feature="detach")]
mod detach; // detached paths

#[cfg(feature="detach")]
use detach::{ AttachedRoot, DetachedRoot, Detach };

#[cfg(feature="detach")]
pub use detach::{ Attach };

#[cfg(not(feature="detach"))]
type AttachedRoot<T> = T;

#[cfg(not(feature="detach"))]
#[allow(non_snake_case)]
fn AttachedRoot<T>(t: T) -> T { t }


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
    ///
    /// _If you see scary `AttachedRoot<Self>` as a part of the return type 
    /// then you have enabled the `detach` feature. Without `detach` that part 
    /// is simply `Self`._
    fn at<Index>(self, i: Index) -> AT<AttachedRoot<Self>, Index> where
        Self::View: At<Index>
    {
        AT { prev: AttachedRoot(self), index: i } 
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
    /// Attaches a [detached](struct.AT.html) path.
    ///
    /// __Not intended for overriding.__
    ///
    /// _Present only on `detach`._
    fn attach<Path>(self, path: Path) -> Path::Output where
        Path: Attach<Self>
    {
        path.attach(self)
    }
}



/// A &#8220;reference&#8221; to some &#8220;location&#8221;.
///
/// With default [`Cps`](trait.Cps.html) implementors every `AT` is 
/// guaranteed to be a list of &#8220;path parts&#8221; with type
///
/// `AT<..AT<AT<AT<&mut root, I1>,I2>,I3>..In>`
///
/// Though `AT` is exposed, it's strongly recommended to use
/// [`impl Cps<View=T>`](trait.Cps.html) as a return type of your functions 
/// and [`Cps<View=T>`](trait.Cps.html) bounds on their parameters.
///
/// Enabling `detach` feature allows one to [detach](#method.detach) `AT`s from their roots. 
/// 
/// ### Note
///
/// If you pass a detached path to a function then you should use 
/// an [`Attach<CPS, View=V>`](trait.Attach.html) bound 
/// instead of a [`Cps<View=V>`](trait.Cps.html) bound.
///
/// I.e.
///
/// ```
/// # #[cfg(feature="detach")] fn test() {
/// # use smart_access::{Cps, Attach, detached_at};
/// fn replace_at<CPS, Path, V>(cps: CPS, path: Path, x: V) -> Option<V> where
///     CPS: Cps,
///     Path: Attach<CPS,View=V>
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
#[must_use]
#[cfg_attr(feature="detach", derive(Clone))]
#[derive(Debug)]
pub struct AT<T, Index> { 
    prev: T, 
    index: Index,
}


#[cfg(feature="detach")]
impl<T, I, Detached> AT<T, I> where
    AT<T, I>: Detach<Output=Detached>
{

/// Detaches the path.
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
/// let detached = foo.at(0).at(0).at(0).detach();
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
/// let path = bar.at(0).at(0).detach().at(0);
/// bar.attach(path).replace(3);
/// assert!(bar == vec![vec![vec![3]]]);
/// ```
    pub fn detach(self) -> Detached {
        <Self as Detach>::detach(self)
    }
}


/// A helper `at` method overriding the `Cps` default.
///
/// _Present only on `detach`._
#[cfg(feature="detach")]
impl<T,I> AT<T, I> where
{
    pub fn at<Index,V>(self, i: Index) -> AT<Self, Index> where
        Self: Cps<View=V>,
        V: At<Index>,
    {
        AT { prev: self, index: i } 
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
/// use smart_access::{Cps, Attach};
///
/// type Mat = Vec<Vec<f64>>;
///
/// fn mat_index<'a>(i: usize, j: usize) -> impl Attach<&'a mut Mat, View=f64> {
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
pub fn detached_at<View: ?Sized, I>(i: I) -> AT<DetachedRoot<View>, I> {
    AT {
        prev: DetachedRoot::new(),
        index: i,
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


/// `access` returns `Some` / `None` according to rules described [here](trait.At.hmtl)
impl<T, V: ?Sized, Index> Cps for AT<T, Index> where
    T: Cps<View=V>,
    V: At<Index>
{
    type View = V::View;
    
    fn access<R, F>(self, f: F) -> Option<R> where 
        F: FnOnce(&mut Self::View) -> R 
    {
        let index = self.index;

        self.prev.access(|v| { v.access_at(index, f) }).flatten()
    }
}


