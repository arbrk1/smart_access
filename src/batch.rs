mod run_batch;  // a helper for compile-time batch execution
use run_batch::RunBatch;

use crate::at::Cps;


/// A builder for complex mutations. __Requires `batch_ct` or `batch_rt`.__
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
/// Compile-time batches are abstracted by the trait [`BatchCt`](trait.BatchCt.html).
///
/// If needed, the concrete type of the compile-time batch can be specified 
/// by means of the [`path`](macro.path.html) macro or fully explicitly: 
/// `CpsBatch<CPS, (..(((), F1), F2) .. Fn)>`.
///
///
/// ## Runtime version
///
/// Created by method `.batch_rt()`. Can be combined with loops but every `.add` consumes some memory.
///
/// _Almost_ compatible with compile-time version but with some quirks.
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
///
/// Runtime batches are abstracted by the trait [`BatchRt`](trait.BatchRt.html).
#[must_use]
pub struct CpsBatch<CPS, L> {
    cps: CPS,
    list: L,
}

#[cfg(feature="batch_rt")]
pub type FnBoxRt<V, R> = Box<dyn FnOnce(&mut V, Option<R>) -> R>;


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
    (Prev,F): RunBatch<CPS::View, Output=R>,
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
impl<CPS: Cps, R> CpsBatch<CPS, Vec<FnBoxRt<CPS::View, R>>> {
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



// Helpers for the Cps trait.
#[cfg(feature="batch_ct")]
pub fn new_batch_ct<CPS: Cps>(cps: CPS) -> CpsBatch<CPS, ()> {
    CpsBatch { cps: cps, list: () }
}

#[cfg(feature="batch_rt")]
pub fn new_batch_rt<CPS, V, R>(cps: CPS) -> CpsBatch<CPS, Vec<FnBoxRt<V,R>>> where 
    CPS: Cps<View=V>,
    V: ?Sized
{
    CpsBatch { cps: cps, list: Vec::new() }
}


/// An abstraction over [compile-time and runtime batches](struct.CpsBatch.html). 
/// __Requires `batch_ct` or `batch_rt`.__
///
/// The only thing which can be done with a value of `Batch`-bounded 
/// type is to [`.run()`](trait.Batch.html#tymethod.run) it.
///
/// Useful as a bound on a function return type.
///
/// If the batch returned by a function is to be edited later 
/// then consider using more precise bounds:
/// [`BatchCt`](trait.BatchCt.html) and [`BatchRt`](trait.BatchRt.html).
#[must_use]
pub trait Batch<R>: Sized {
    fn run(self) -> Option<R>;
}


#[cfg(feature="batch_ct")]
impl<CPS: Cps> Batch<()> for CpsBatch<CPS, ()> {
    fn run(self) -> Option<()> {
        self.run()
    }
}


#[cfg(feature="batch_ct")]
impl<CPS: Cps, Prev, F, R> Batch<R> for CpsBatch<CPS, (Prev, F)> where
    CPS: Cps,
    (Prev,F): RunBatch<CPS::View, Output=R>
{
    fn run(self) -> Option<R> {
        self.run()
    }
}


#[cfg(feature="batch_rt")]
impl<CPS: Cps, R> Batch<R> for CpsBatch<CPS, Vec<FnBoxRt<CPS::View, R>>> {
    fn run(self) -> Option<R> {
        self.run()
    }
}


/// A compile-time batch. __Requires `batch_ct` feature.__
///
/// See basic usage guide [here](struct.CpsBatch.html).
///
/// Allows one to write batch transformers without specifying complex
/// return types.
///
/// ```
/// use smart_access::{ BatchCt, Cps };
///
/// fn add_inc<T>(batch: impl BatchCt<i32, T>) -> impl BatchCt<i32, ()>
/// {
///     batch.add(|x, _| { *x = *x + 1; })
/// }
///
/// let mut foo = 1;
///
/// add_inc(add_inc(foo.batch_ct())).run();
/// assert!(foo == 3);
/// ```
#[cfg(feature="batch_ct")]#[must_use]
pub trait BatchCt<V: ?Sized, R>: Batch<R> {
    type CPS: Cps<View=V>;
    type List: RunBatch<V, Output=R>;

    /// Adds a new function to a compile-time batch.
    fn add<G, S>(self, g: G) -> CpsBatch<Self::CPS, (Self::List, G)> where 
        G: FnOnce(&mut V, R) -> S;

    /// Clears a compile-time batch.
    fn clear(self) -> CpsBatch<Self::CPS, ()>;

    /// [Runs](trait.Batch.html#tymethod.run) a compile-time batch.
    fn run(self) -> Option<R> {
        <Self as Batch<R>>::run(self)
    }
}


#[cfg(feature="batch_ct")]
impl<CPS: Cps> BatchCt<CPS::View, ()> for CpsBatch<CPS, ()> {
    type CPS = CPS;
    type List = ();
    
    fn add<G, S>(self, g: G) -> CpsBatch<CPS, (Self::List, G)> where 
        G: FnOnce(&mut CPS::View, ()) -> S
    {
        self.add(g)
    }

    fn clear(self) -> CpsBatch<CPS, ()> {
        self
    }
}


#[cfg(feature="batch_ct")]
impl<CPS, Prev, F, R> BatchCt<CPS::View, R> for CpsBatch<CPS, (Prev, F)> where
    CPS: Cps,
    (Prev,F): RunBatch<CPS::View, Output=R>
{
    type CPS = CPS;
    type List = (Prev, F);
    
    fn add<G, S>(self, g: G) -> CpsBatch<CPS, (Self::List, G)> where 
        G: FnOnce(&mut CPS::View, R) -> S
    {
        self.add(g)
    }

    fn clear(self) -> CpsBatch<CPS, ()> {
        self.clear()
    }
}


/// A runtime batch. __Requires `batch_rt` feature.__
///
/// See basic usage guide [here](struct.CpsBatch.html).
///
/// Allows one to write batch transformers without specifying complex
/// return types.
///
/// ```
/// use smart_access::{ BatchRt, Cps };
///
/// fn add_inc(batch: impl BatchRt<i32, ()>) -> impl BatchRt<i32, ()>
/// {
///     batch.add(|x, _| { *x = *x + 1; })
/// }
///
/// let mut foo = 1;
///
/// add_inc(add_inc(foo.batch_rt())).run();
/// assert!(foo == 3);
/// ```
#[cfg(feature="batch_rt")]#[must_use]
pub trait BatchRt<View: ?Sized, R>: Batch<R> {
    /// Adds a new function to a runtime batch.
    fn add<G>(self, g: G) -> Self
        where G: FnOnce(&mut View, Option<R>) -> R + 'static;

    /// Clears a runtime batch.
    fn clear(self) -> Self;
    
    /// Takes the last function from a runtime batch.
    fn pop(self, dst: Option<&mut Option<FnBoxRt<View, R>>>) -> Self;

    /// A direct access to the underlying vector.
    fn edit(&mut self) -> &mut Vec<FnBoxRt<View, R>>;
    
    /// [Runs](trait.Batch.html#tymethod.run) a runtime batch.
    fn run(self) -> Option<R> {
        <Self as Batch<R>>::run(self)
    }
}


#[cfg(feature="batch_rt")]
impl<CPS: Cps, R> BatchRt<CPS::View, R> for 
    CpsBatch<CPS, Vec<FnBoxRt<CPS::View, R>>> 
{
    fn add<G>(self, g: G) -> Self
        where G: FnOnce(&mut CPS::View, Option<R>) -> R + 'static 
    {
        self.add(g)
    }

    fn clear(self) -> Self {
        self.clear()
    }
    
    fn pop(self, dst: Option<&mut Option<FnBoxRt<CPS::View, R>>>) -> Self {
        self.pop(dst)
    }

    fn edit(&mut self) -> &mut Vec<FnBoxRt<CPS::View, R>> {
        self.edit()
    }

}
