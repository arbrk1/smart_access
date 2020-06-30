
/// Constructs a pathlike type for `AT`, `DetachedPath`, and `CpsBatch`.
///
/// The invocation `path!(I1, I2, .. In)` resolves into 
/// `(..(((), I1), I2) .. In)`.
///
/// ### Usage example
///
/// ```
/// # use smart_access::{ AT, Cps, path };
/// type AccessIJ<CPS> = AT<CPS, path!(usize, usize)>;
///
/// let mut foo = vec![vec![1,2], vec![3,4]];
/// let path: AccessIJ<_> = foo.at(0).at(1);
///
/// # #[cfg(feature="batch_ct")] fn some_fn() {
/// # use smart_access::{ CpsBatch };
/// type Batch2<CPS,F,G> = CpsBatch<CPS, path!(F, G)>;
/// let mut bar = 42;
/// let batch: Batch2<_,_,_> = bar.batch_ct().add(|x,_| *x + 1).add(|x,_| 2 * *x);
/// # }
///
/// # #[cfg(feature="detach")] fn some_fn2() {
/// # use smart_access::{ DetachedPath, detached_at };
/// type DetachedIJ<T> = DetachedPath<Vec<Vec<T>>, path!(usize, usize)>;
///
/// let detached_path: DetachedIJ<f64> = detached_at(4).at(2);
/// # }
/// ```
#[macro_export]
macro_rules! path {
    // first, we reverse the path due to limitations in the macro matcher
    // (we can't use ($($tt:ty,)* $t:ty) because of an ambiguity)
    ( $($tt:ty),* ) => { 
        $crate::path!( $($tt),* ;; ) 
    };

    ( $t:ty $(,$tt:ty)* ;; $($rev:ty),* ) => { 
        $crate::path!( $($tt),* ;; $t $(,$rev)* ) 
    };

    // now we construct the reversed-reversed path type
    ( ;; $t:ty $(,$tt:ty)* ) => { 
        ($crate::path!(;; $($tt),* ), $t) 
    };

    ( ;; ) => { () };
}

