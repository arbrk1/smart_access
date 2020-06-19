#[cfg(any(feature="batch_rt", feature="batch_ct"))]
mod run_batch;  // a helper for compile-time batch execution

#[cfg(any(feature="batch_rt", feature="batch_ct"))]
use run_batch::RunBatch;


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
pub trait Cps {
    type View: ?Sized;

    /// Returns `Some(f(..))` or `None`.
    ///
    /// The rules governing the value returned are defined by an implementation.
    fn access<R, F>(self, f: F) -> Option<R> where
        Self: Sized,
        F: FnOnce(&mut Self::View) -> R;

    /// Equivalent to `self.access(|x| std::mem::replace(x, new_val))`
    fn replace(self, new_val: Self::View) -> Option<Self::View> where
        Self: Sized,
        Self::View: Sized 
    {
        self.access(|x| std::mem::replace(x, new_val))
    }

    /// Equivalent to `self.access(|_| ())`
    fn touch(self) -> Option<()> where
        Self: Sized
    {
        self.access(|_| ())
    }

    /// &#8220;Moves in the direction&#8221; of the provided index.
    ///
    /// It seems to be impossible to override `at` in a meaningful way.
    fn at<Index>(self, i: Index) -> AT<Self, Index> where
        Self: Sized,
        Self::View: At<Index>
    {
        AT { prev: self, index: i }
    }

    #[cfg(feature="batch_ct")]
    /// Constructs a [compile-time batch](struct.CpsBatch.html).
    fn batch_ct(self) -> CpsBatch<Self, ()> where
        Self: Sized,
    {
        CpsBatch { cps: self, list: () }
    }

    #[cfg(feature="batch_rt")]
    /// Constructs a [runtime batch](struct.CpsBatch.html).
    fn batch_rt<R>(self) -> CpsBatch<Self, Vec<FnBoxRt<Self::View, R>>> where
        Self: Sized,
    {
        CpsBatch { cps: self, list: Vec::new() }
    }
}


/// A builder for complex mutations.
///
/// Comes in two flavors.
///
/// ## Compile-time version
///
/// Created by method `.batch_ct()` of any [`Cps`](trait.Cps.html)-bounded value.
///
/// Efficient but can't be combined with loops (and is difficult to use in 
/// presence of conditional branches).
///
/// ### Example
///
/// ```
/// use smart_access::Cps;
///
/// let mut foo = 0;
///
/// // here we use a mutable reference as a Cps-bounded value
/// let batch = (&mut foo).batch_ct();
/// 
/// // compile-time batches are immutable because adding a new mutator changes type of the batch
/// let batch = batch
///     .add(|v, _| { *v = *v + 2; 42 })
///     .add(|v, x| { *v = *v * x; "Hello!" });
///
/// let result = batch.run();
/// 
/// assert!(result == Some("Hello!"));
/// assert!(foo == (0 + 2) * 42);
/// ```
///
///
/// ## Runtime version
///
/// Created by method `.batch_rt()`. Has _mutable_ interface. Can be combined
/// with loops but every `.add` consumes some memory.
///
/// ### Example
///
/// ```
/// use smart_access::Cps;
///
/// let mut foo = 0;
///
/// let mut batch = (&mut foo).batch_rt();
///
/// for i in 1..=10 {
///     // "move" is required if the closure uses any local variables
///     batch = batch.add(move |v, _| { *v = *v + i; i });
/// }
///
/// // Previous result can be used but it is wrapped in Option. 
/// // This Option is None only in the first mutator in a batch, 
/// // i.e. when there is no previous value.
/// batch = batch.add(|v, prev| { *v = *v * prev.unwrap(); 42 });
///
/// // "Builder" style can also be used:
/// batch = batch
///     .add(|v, prev| { *v = -*v; prev.unwrap() } )
///     .add(|v, prev| { *v = -*v; prev.unwrap() } );
///
/// // Runtime batches give a direct access to the vector of actions:
/// batch.edit().access(|vec| {
///     let f = vec.pop().unwrap();
///     vec.push(f);
/// });
///
/// let result = batch.run();
///
/// // Unlike compile-time batches all intermediate results must be of the same type.
/// assert!(result == Some(42)); 
/// assert!(foo == (1..=10).sum::<i32>() * 10);
/// ```
#[cfg(any(feature="batch_rt", feature="batch_ct"))]
#[must_use]
pub struct CpsBatch<CPS, L> {
    cps: CPS,
    list: L,
}

#[cfg(feature="batch_rt")]
type FnBoxRt<V, R> = Box<dyn FnOnce(&mut V, Option<R>) -> R>;


/// An _empty_ compile-time batch.
#[cfg(feature="batch_ct")]
impl<CPS> CpsBatch<CPS, ()> where
    CPS: Cps
{
    /// Runs an _empty_ compile-time batch. 
    ///
    /// Immediately returns `None`.
    pub fn run(self) -> Option<()> { None }

    /// Adds a new function to an _empty_ compile-time batch.
    pub fn add<F, R>(self, f: F) -> CpsBatch<CPS, ((), F)>
        where F: FnOnce(&mut CPS::View, ()) -> R
    {
        CpsBatch { cps: self.cps, list: (self.list, f) }
    }
}

/// A _nonempty_ compile-time batch.
#[cfg(feature="batch_ct")]
impl<CPS,Prev,F,R> CpsBatch<CPS, (Prev, F)> where
    CPS: Cps,
    (Prev,F): RunBatch<CPS::View, Result=R>,
{
    /// Runs a _nonempty_ compile-time batch.
    pub fn run(self) -> Option<R> {
        let list = self.list;

        self.cps.access(|v| list.run(v))
    }
    
    /// Adds a new function to a _nonempty_ compile-time batch.
    pub fn add<G, S>(self, g: G) -> CpsBatch<CPS, ((Prev, F), G)>
        where G: FnOnce(&mut CPS::View, R) -> S
    {
        CpsBatch { cps: self.cps, list: (self.list, g) }
    }

    /// Takes the last function from a _nonempty_ compile-time batch.
    ///
    /// You can use it as follows:
    ///
    /// ```
    /// # use smart_access::Cps;
    /// let mut foo = 0;
    /// let mut maybe_f = None;
    /// foo.batch_ct()
    ///     .add(|x, _| { *x += 1; })
    ///     .add(|x, _| { *x += 1; })
    ///     .pop(Some(&mut maybe_f))
    ///     .run();
    ///
    /// assert!(foo == 1);
    /// 
    /// maybe_f.unwrap()(&mut foo, ());
    /// assert!(foo == 2);
    /// ```
    pub fn pop(self, dst: Option<&mut Option<F>>) -> CpsBatch<CPS, Prev> {
        let (prev, f) = self.list;
        
        if let Some(place) = dst { *place = Some(f); }
        
        CpsBatch { cps: self.cps, list: prev }
    }

    /// Clears a _nonempty_ compile-time batch.
    pub fn clear(self) -> CpsBatch<CPS, ()> {
        CpsBatch { cps: self.cps, list: () }
    }
}


#[cfg(feature="batch_ct")]#[test]
fn test_ct_batch_editing() {
    use crate::Cps;
    let mut foo = 1;

    foo.batch_ct()
        .add(|x, _| { *x += 1; })
        .add(|x, _| { *x += 1; })
        .pop(None)
        .run();

    assert!(foo == 2);
    
    foo.batch_ct()
        .add(|x, _| { *x += 1; })
        .add(|x, _| { *x += 1; })
        .clear()
        .run();

    assert!(foo == 2);
}



/// A runtime batch.
///
/// Has two interfaces:
/// * a direct access to the underlying vector: the `.edit()` method
/// * a compile-time batch compatibility layer
#[cfg(feature="batch_rt")]
impl<CPS, R> CpsBatch<CPS, Vec<FnBoxRt<CPS::View, R>>> where
    CPS: Cps
{
    /// Runs an empty runtime batch. 
    ///
    /// Immediately returns `None` if the batch is empty.
    pub fn run(self) -> Option<R> {
        let list = self.list;

        if list.len() == 0 { return None; }

        self.cps.access(|v| list.run(v)).map(|x| x.unwrap())
    }
    
    /// Adds a new function to a runtime batch.
    pub fn add<F>(mut self, f: F) -> Self where 
        F: FnOnce(&mut CPS::View, Option<R>) -> R + 'static
    {
        self.list.push(Box::new(f));

        self
    }

    /// Takes the last function from a runtime batch.
    pub fn pop(mut self, dst: Option<&mut Option<FnBoxRt<CPS::View, R>>>) -> Self
    {
        let maybe_f = self.list.pop();

        if let Some(place) = dst { *place = maybe_f; }

        self
    }

    /// Clears a runtime batch.
    pub fn clear(mut self) -> Self {
        self.list.clear();

        self
    }

    /// A direct access to the underlying vector.
    pub fn edit(&mut self) -> &mut Vec<FnBoxRt<CPS::View, R>> {
        &mut self.list
    }
}


#[cfg(feature="batch_rt")]#[test]
fn test_rt_batch_editing() {
    use crate::Cps;
    let mut foo = 1;

    foo.batch_rt()
        .add(|x, _| { *x += 1; })
        .add(|x, _| { *x += 1; })
        .pop(None)
        .run();

    assert!(foo == 2);
    
    foo.batch_rt()
        .add(|x, _| { *x += 1; })
        .add(|x, _| { *x += 1; })
        .clear()
        .run();

    assert!(foo == 2);
    
    let mut maybe_f = None;
    foo.batch_rt()
        .add(|x, _| { *x += 1; })
        .add(|x, _| { *x += 1; })
        .pop(Some(&mut maybe_f))
        .run();
    
    assert!(foo == 3);
    
    maybe_f.unwrap()(&mut foo, None);
    assert!(foo == 4);
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
#[must_use]
pub struct AT<T, Index> { 
    prev: T, 
    index: Index,
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

