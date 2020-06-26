//! # Smart accessors
//!
//! Let's begin with a few words on naming.
//!
//! What is commonly named &#8220;smart __pointer__&#8221; is usually associated 
//! with trivial access (dereference) semantics and nontrivial clone/drop semantics.
//!
//! Smart __accessors__ provided by this crate also serve the purpose of 
//! accessing some data but in a different way: they have trivial drop semantics 
//! and nontrivial access semantics.
//!
//! ### Note
//!
//! If you do not want to read a long text, just proceed to the 
//! [essential part](#cargo-features) of the documentation.
//!
//!
//! ## High level overview
//!
//! The goal of this crate is twofold:
//!
//! * to offer one possible solution to the
//! [problem](https://rust-lang.github.io/rfcs/2094-nll.html#problem-case-3-conditional-control-flow-across-functions) that 
//!   the current (rustc 1.44) borrowchecker doesn't understand
//!   functions with multiple exit points 
//!   ([Polonius](https://github.com/rust-lang/polonius)
//!   doesn't have this problem but it is not stable yet)
//! * to provide a way to do bidirectional programming (when updating 
//!   some view of data updates the data viewed to match the updated view)
//!
//! If you are aqcuainted with optics in functional languages you can 
//! think of this crate as a minimalistic &#8220;lens&#8221; (more precisely, 
//! affine traversal) library using an imperative approach.
//!
//! ### Note
//!
//! As a side-effect of the library design one can use a &#8220;build&#8221; 
//! pattern with standard `&mut` references (see below).
//!
//!
//! ## Usage examples
//!
//! This crate already implements [accessors](stdlib_impls/) for stdlib collections:
//!
//! ```
//! use smart_access::Cps;
//!
//! let mut foo = vec![vec![1,2,3], vec![4,5,6]];
//!
//! let bar = foo.at(0).at(1).replace(7);
//! assert!(foo == vec![vec![1,7,3], vec![4,5,6]]);
//! assert!(bar == Some(2));
//!
//! let baz = foo.at(2).at(1).replace(8);
//! assert!(foo == vec![vec![1,7,3], vec![4,5,6]]);
//! assert!(baz == None);  // None is returned if path doesn't make sense
//!
//! // Any mutable reference can be used as a "data location":
//! assert!(foo[0][0].replace(9) == Some(1));
//! assert!(foo == vec![vec![9,7,3], vec![4,5,6]]);
//! ```
//!
//! A somewhat more interesting example:
//!
//! ```
//! # use smart_access::Cps;
//! let mut foo = vec![1,2,3,4,5,6];
//!
//! let bar = foo.at(1..=3).replace(vec![7,8]);
//! assert!(foo == vec![1,7,8,5,6]);
//! assert!(bar == Some(vec![2,3,4]));
//! ```
//!
//! An arbitrary mutating operation can be used instead of replacement:
//!
//! ```
//! # use smart_access::Cps;
//! let mut foo = vec![1,2,3,4,5,6];
//!
//! let bar = foo.at(1..4).access(|v| { *v = vec![v.iter().sum()]; "baz" });
//! assert!(foo == vec![1,9,5,6]);
//! assert!(bar == Some("baz"));
//!
//! // And with mutable references you get a sort of the "build" part of the Builder pattern
//! foo[0].access(|x| { /* do something with the element */ });
//! ```
//!
//!
//! ## Usage guide
//!
//! To add a smart accessor to your own datatype `Data` you need to:
//!
//! * choose some type `Index`
//! * add trait [`At<Index>`](trait.At.html) to the type `Data`
//! * implement [`access_at`](trait.At.html#tymethod.access_at) method
//! * at the usage site write `use smart_access::Cps;`
//! * PROFIT!
//!
//!
//! ## Motivation (part I: lifetimes)
//!
//! Suppose you have `HashMap` but without &#8220;Entry API&#8221; 
//! (Entry API is an implementation feature: not every datastructure 
//! in the wild provides any analogue). 
//!
//! Suppose also that you want to implement something akin to
//! `|m, k, v| m.entry(k).or_insert(v)`.
//!
//! You could write
//!
//! ``` compile_fail
//! # use std::collections::HashMap;
//! // for simplicity we use usize keys in the examples below
//! fn or_insert<V>(hm: &mut HashMap<usize,V>, k: usize, v: V) -> &mut V {
//!     if let Some(v) = hm.get_mut(&k) { 
//!         return v; 
//!         // this is the first exit point but the borrow checker
//!         // doesn't distinguish between it and the second exit point
//!     }     
//!
//!     hm.insert(k, v);  // Oops: hm is already borrowed! 
//!                       // (It _MUST_ be borrowed until the exit point)
//!
//!     hm.get_mut(&k).unwrap()
//!     // the second exit point
//! }
//! ```
//!
//! but it would not compile because of limitations of the borrow checker.
//!
//! It seems there is no way to write such a function without
//! additional queries to the `HashMap` and without
//! resorting to reference-pointer-reference conversions or
//! other `unsafe` techniques.
//!
//! This crate provides a not-so-clumsy workaround:
//!
//! ```
//! use std::collections::HashMap;
//! use smart_access::{At, Cps};
//!
//! struct Ensure<K,V> { key: K, value: V }
//!
//! impl<V> At<Ensure<usize, V>> for HashMap<usize, V> 
//! {
//!     type View = V;
//!
//!     fn access_at<R, F>(&mut self, kv: Ensure<usize, V>, f: F) -> Option<R> where
//!         F: FnOnce(&mut V) -> R
//!     {
//!         if let Some(v) = self.get_mut(&kv.key) { 
//!             return Some(f(v)); 
//!             // We use so called CPS-transformation: we wrap each 
//!             // return site with a call to the provided function.
//!         } 
//!
//!         self.insert(kv.key, kv.value);
//!         Some(f(self.get_mut(&kv.key).unwrap()))
//!     }
//! }
//!
//! // now you can write or_insert (note the return type!):
//! fn or_insert<'a, V>(hm: &'a mut HashMap<usize,V>, k: usize, v: V) -> impl Cps<View=V> + 'a {
//!     hm.at(Ensure{ key: k, value: v })
//! }
//! ```
//!
//! There are some peculiarities though:
//!
//! * `&mut V` is _eager_: all code which is needed to obtain a reference 
//!   to the value is executed at the site of access
//! * `impl Cps<View=V>` is _lazy_: access is a zero-cost operation and all 
//!   the machinery needed to reach the value is run at the site of modification
//! * `&'a mut V` can be reborrowed, i.e. cloned for some subperiod of `'a`, 
//!   making it possible to modify the value referenced more than once
//! * `impl Cps<View=V>` can be used only once but has [batching](struct.CpsBatch.html). 
//!   It comes in two flavors: _compile-time batching_ 
//!   which can't be used across any control flow and _runtime batching_ which 
//!   can't be used in `no_std` contexts
//!
//! ### Note
//!
//! The forementioned accessor `Ensure { key: K, value: V }` is defined 
//! in [`stdlib_impls`](stdlib_impls/) simply as a pair `(K,V)` so 
//! for example you can write
//!
//! ```
//! # use smart_access::Cps;
//! # let mut map = std::collections::HashMap::<String,String>::new();
//! map.at( ("foo".to_string(), "bar".to_string()) ).touch();
//! ```
//!
//! instead of
//!
//! ```
//! # let mut map = std::collections::HashMap::<String,String>::new();
//! map.entry("foo".to_string()).or_insert("bar".to_string());
//! ```
//!
//!
//! ## Motivation (part II: bidirectional programming)
//!
//! We give a simple illustration: a toy example of a bidirectional vector parser.
//!
//! Not only can it parse a vector but also can print it back (note 
//! that two bidirectional parsers can be combined into a bidirectional 
//! translator from one textual representation to another).
//!
//! A combinator library greatly facilitating writing such parsers 
//! can be implemented but it is not a (current-time) goal of this crate.
//!
//! ### Note
//!
//! Some function definitions in the following code are hidden. To see them look 
//! at the full [module source](../src/smart_access/lib.rs.html).
//!
//! ```
//! // A little showcase:
//! assert!(vector_parser().bi_left((Some(vec![1,2,3]),"".into())) == "[1,2,3]".to_string());
//! assert!(vector_parser().bi_right(&mut "[1,2,3] foo".into()).0  == Some(vec![1,2,3]));
//! assert!(vector_parser().bi_right(&mut "[1,2,3,]bar".into()).0  == Some(vec![1,2,3]));
//! assert!(vector_parser().bi_right(&mut "[,]".into()).0          == None);
//! assert!(vector_parser().bi_right(&mut "[]".into()).0           == Some(vec![]));
//! assert!(vector_parser().bi_right(&mut "]1,2,3[".into()).0      == None);
//!
//! // The code:
//! use smart_access::{At, Cps};
//!
//! // a minimal set of parser combinators
//! #[derive(Clone)] struct _Number;
//! #[derive(Clone)] struct _Char(char);
//! #[derive(Clone)] struct _Many<T>(T);
//! #[derive(Clone)] struct _Optional<T>(T);
//! #[derive(Clone)] struct _Cons<Car,Cdr>(Car,Cdr);
//! #[derive(Clone)] struct _Iso<Parser,F,G>(Parser,F,G);
//!
//! fn vector_parser() -> impl Bidirectional<String, Parse<Vec<usize>>> {
//!     let grammar = 
//!         _Cons(_Char('['), 
//!         _Cons(_Many(_Cons(_Number, _Char(','))),
//!         _Cons(_Optional(_Number),
//!               _Char(']'))));
//!     
//!     let from_grammar = |(_bl, (xs, (ox, _br))): (_, (Vec<_>, (Option<_>, _)))| 
//!     {
//!         xs.into_iter().map(|(x, _comma)| x).chain(ox.into_iter()).collect()
//!     };
//!
//!     let to_grammar = |mut vec: Vec<_>| {
//!         let last = vec.pop();
//!
//!         ('[', (vec.into_iter().map(|x| (x, ',')).collect(), (last, ']')))
//!     };
//!
//!     _Iso(grammar, from_grammar, to_grammar)
//! }
//!
//! trait Bidirectional<A,B> {
//!     fn bi_left(self, b: B) -> A;
//!     fn bi_right(self, a: &mut A) -> B;
//! }
//!
//! // DO NOT USE IN PRODUCTION: efficient parsing is incompatible 
//! // with using copies of tails of the parsed string
//! type Parse<T> = (Option<T>, String);
//!
//! // a very simplistic blanket implementation
//! impl<A,B,I> Bidirectional<A,B> for I where
//!     A: At<I, View=B> + Default,
//!     B: Clone
//! {
//!     fn bi_left(self, b: B) -> A {
//!         let mut a = A::default();
//!
//!         a.at(self).access(|x| { *x = b; });
//!
//!         a
//!     }
//!
//!     fn bi_right(self, a: &mut A) -> B {
//!         a.at(self).access(|b| b.clone()).unwrap()
//!     }
//! }
//! 
//! impl At<_Number> for String {
//!     type View = Parse<usize>;
//!
//! #     fn access_at<R,F>(&mut self, _: _Number, f: F) -> Option<R> where
//! #         F: FnOnce(&mut Parse<usize>) -> R
//! #     {
//! #         let mut digits = String::new();
//! #
//! #         let mut it = self.chars();
//! #         let mut maybe_c = None;
//! #         for c in &mut it {
//! #             if c >= '0' && c <= '9' { digits.push(c); } 
//! #             else { maybe_c = Some(c); break; }
//! #         }
//! #
//! #         let rest = maybe_c.into_iter().chain(it).collect::<String>();
//! #         let mut arg = match digits.parse() {
//! #             Err(_) => (None, self.clone()),
//! #             Ok(number) => (Some(number), rest),
//! #         };
//! #
//! #         let result = f(&mut arg);
//! #         
//! #         let (maybe_number, rest) = arg;
//! #         match maybe_number {
//! #             Some(number) => { *self = number.to_string() + &rest; }
//! #             None         => { *self = rest; }
//! #         }
//! #
//! #         Some(result)
//! #     }
//!     // access_at is hidden
//! }
//!
//! impl At<_Char> for String {
//!     type View = Parse<char>;
//!
//! #     fn access_at<R,F>(&mut self, i: _Char, f: F) -> Option<R> where
//! #         F: FnOnce(&mut Parse<char>) -> R
//! #     {
//! #         let mut it = self.chars();
//! #         
//! #         let mut arg = match it.next() {
//! #             None => { (None, self.clone()) }
//! #             Some(c) => {
//! #                 if c != i.0 { (None, self.clone()) }
//! #                 else { (Some(c), it.collect::<String>()) }
//! #             }
//! #         };
//! #
//! #         let result = f(&mut arg);
//! #        
//! #         let (maybe_c, rest) = arg;
//! #         match maybe_c {
//! #             Some(c) => { *self = c.to_string() + &rest; }
//! #             None    => { *self = rest; }
//! #         }
//! #         
//! #         Some(result)
//! #     }
//!     // access_at is hidden
//! }
//! 
//! impl<V, Parser> At<_Many<Parser>> for String where
//!     String: At<Parser, View=Parse<V>>,
//!     Parser: Bidirectional<String, Parse<V>> + Clone,
//! {
//!     type View = Parse<Vec<V>>;
//!
//! #     fn access_at<R,F>(&mut self, i: _Many<Parser>, f: F) -> Option<R> where
//! #         F: FnOnce(&mut Self::View) -> R
//! #     {
//! #         let parser = &i.0;
//! #
//! #         let mut vec = Vec::<V>::new();
//! #         let mut current_string = self.clone();
//! #
//! #         loop {
//! #             match parser.clone().bi_right(&mut current_string) {
//! #                 (Some(v),s) => {
//! #                     vec.push(v);
//! #                     current_string = s;
//! #                 }
//! #
//! #                 (None,_) => { break; }
//! #             }
//! #         }
//! #
//! #         let mut arg = (Some(vec), current_string);
//! #         let result = f(&mut arg);
//! #         
//! #         let (maybe_vec, rest) = arg;
//! #         match maybe_vec {
//! #             None => { *self = rest; }
//! #             Some(vec) => {
//! #                 *self = vec.into_iter()
//! #                     .map(|x| parser.clone().bi_left((Some(x),"".into())))
//! #                     .collect::<String>() + &rest;
//! #             }
//! #         }
//! #
//! #         Some(result)
//! #     }
//!     // access_at is hidden
//! }
//!
//! impl<V, Parser> At<_Optional<Parser>> for String where
//!     String: At<Parser, View=Parse<V>>,
//!     Parser: Bidirectional<String, Parse<V>> + Clone,
//! {
//!     type View = Parse<Option<V>>;
//!
//! #     fn access_at<R,F>(&mut self, i: _Optional<Parser>, f: F) -> Option<R> where
//! #         F: FnOnce(&mut Self::View) -> R
//! #     {
//! #         let parser = i.0;
//! #
//! #         let mut arg = match parser.clone().bi_right(self) {
//! #             (maybe_value, s) => (Some(maybe_value), s),
//! #         };
//! #
//! #         let result = f(&mut arg);
//! #         
//! #         let (maybe_value, rest) = arg;
//! #         match maybe_value {
//! #             None => { *self = rest; }
//! #             Some(maybe_value) => {
//! #                 *self = parser.bi_left((maybe_value,"".into())) + &rest;
//! #             }
//! #         }
//! #
//! #         Some(result)
//! #     }
//!     // access_at is hidden
//! }
//!
//! impl<V1, V2, P1, P2> At<_Cons<P1, P2>> for String where
//!     String: At<P1, View=Parse<V1>>,
//!     String: At<P2, View=Parse<V2>>,
//!     P1: Bidirectional<String, Parse<V1>> + Clone,
//!     P2: Bidirectional<String, Parse<V2>> + Clone,
//! {
//!     type View = Parse<(V1,V2)>;
//!
//! #     fn access_at<R,F>(&mut self, i: _Cons<P1, P2>, f: F) -> Option<R> where 
//! #         F: FnOnce(&mut Self::View) -> R
//! #     {
//! #         let _Cons(p1, p2) = i;
//! #
//! #         let (maybe_v1, mut s1) = p1.clone().bi_right(self);
//! #         let (maybe_v2, s2)     = p2.clone().bi_right(&mut s1);
//! #
//! #         let mut arg = match (maybe_v1, maybe_v2) {
//! #             (Some(v1), Some(v2)) => (Some( (v1, v2) ), s2),
//! #             _ => (None, self.clone())
//! #         };
//! #
//! #         let result = f(&mut arg);
//! #
//! #         let (maybe_values, rest) = arg;
//! #         match maybe_values {
//! #             None => { *self = rest; }
//! #             Some( (v1, v2) ) => {
//! #                 *self = vec![
//! #                     p1.bi_left((Some(v1), "".into())),
//! #                     p2.bi_left((Some(v2), "".into())),
//! #                     rest
//! #                 ].into_iter().collect();
//! #             }
//! #         }
//! #
//! #         Some(result)
//! #     }
//!     // access_at is hidden
//! }
//!
//! impl<Parser, FromParser, ToParser, T, V> 
//! At<_Iso<Parser, FromParser, ToParser>> for String where
//!     String: At<Parser, View=Parse<T>>,
//!     Parser: Bidirectional<String, Parse<T>> + Clone,
//!     T: Clone,
//!     FromParser: FnOnce(T) -> V,
//!     ToParser: FnOnce(V) -> T,
//! {
//!     type View = Parse<V>;
//!
//! #     fn access_at<R,F>(&mut self, i: _Iso<Parser, FromParser, ToParser>, f: F) 
//! #         -> Option<R> where 
//! #         F: FnOnce(&mut Self::View) -> R
//! #     {
//! #         let _Iso(parser, from_parser, to_parser) = i;
//! #
//! #         let (maybe_t, rest) = parser.clone().bi_right(self);
//! #
//! #         let mut arg = (maybe_t.map(|t| from_parser(t)), rest); 
//! #         let result = f(&mut arg);
//! #
//! #         let (maybe_v, rest) = arg;
//! #         match maybe_v {
//! #             None => { *self = rest; }
//! #             Some(v) => {
//! #                 *self = parser.bi_left((Some(to_parser(v)),"".to_string())) + &rest;
//! #             }
//! #         }
//! #
//! #        Some(result)
//! #     }
//!     // access_at is hidden
//! }
//! ```
//!
//!
//! ## Connection to functional programming
//!
//! Rust type `fn(&mut V)` roughly corresponds to Haskell type `v -> v`.
//!
//! Thus Rust [`access_at`](trait.At.html#tymethod.access_at) type 
//! could be written in Haskell (after some argument-swapping) as
//!
//! ``` haskell
//! accessAt :: ix -> (v -> (v,r)) -> t -> (t, Maybe r)
//! ```
//!
//! Suppose that `access_at` always returns `Some(..)`. In such a case 
//! the Haskell type of `access_at` can be simplified to
//!
//! ``` haskell
//! accessAt :: ix -> (v -> (v,r)) -> t -> (t,r)
//! ```
//!
//! Its type is isomorphic to any of the following
//!
//! ``` haskell
//! ix -> t -> (v -> (v,r))     -> (t,r)  -- by arg-swapping
//! ix -> t -> (v->v, v->r)     -> (t,r)  -- by universal property of products
//! ix -> t -> (v->v) -> (v->r) -> (t,r)  -- by currying
//! ```
//!
//! Recall that a lens is uniquely defined by a getter and a setter:
//!
//! ``` haskell
//! type Lens t v = (t -> v, t -> v -> t)
//! ```
//!
//! This type is isomorphic to
//!
//! ``` haskell
//! type Lens t v = t -> (v, v -> t)
//! ```
//!
//! Notice that the types `(v, v->t)` and `forall r. (v->v) -> (v->r) -> (t,r)`
//! are rather similiar. Define
//!
//! ``` haskell
//! right :: (v, v->t) -> (v->v) -> (v->r) -> (t,r)
//! right (v, v_t) v_v v_r = (v_t (v_v v), v_r v)
//!
//! left :: (forall r. (v->v) -> (v->r) -> (t,r)) -> (v, v->t)
//! left f = (snd (f     id        id    ),  -- getter
//!     \v -> fst (f (\_ -> v) (\_ -> ())))  -- setter
//! ```
//!
//! Now we prove `(left . right) ~ id`:
//!
//! ``` haskell
//! left (right (v, v_t)) = (v, \x -> v_t x) ~ (v, v_t)
//! ```
//!
//! I.e. `right` is an injection: every value `lens :: Lens t v` can be 
//! presented as `left (accessAt ix)`: it suffices to define
//!
//! ``` haskell
//! accessAt ix = right lens  -- modulo aforementioned type-fu
//! ```
//!
//! In fact the full type (with `Maybe`)
//!
//! ``` haskell
//! accessAt ix :: (v -> (v,r)) -> t -> (t, Maybe r)
//! ```
//!
//! can house any lens, prism or affine traversal.
//!
//! ## Version migration guide
//!
//! ### From 0.4 to 0.5
//!
//! #### Difference #1
//!
//! The [`AT`](struct.AT.html) type changed its representation. 
//!
//! Now it has simpler and more flat structure:
//!
//! `AT<CPS, (..(((), I1), I2) .. In)>`
//!
//! instead of
//!
//! `AT<..AT<AT<CPS, I1>, I2> .. In>`
//!
//! Unfortunately, the new structure doesn't play well with type inference 
//! but this issue has been circumvented by separating the `Cps` implementation 
//! into a helper trait (this trait isn't exposed to the public API).
//!
//! Because the `AT` type isn't to be used explicitly, usually there is 
//! no need to change any code.
//!
//! Nevertheless there may exist some code which compiled on 0.4 and does not 
//! compile on 0.5.
//!
//! #### Difference #2
//!
//! Relevant only to the `detach` feature.
//!
//! Now the [`detach`](struct.At.html#method.detach) method returns not 
//! only the detached part but also the left part (an accessor with 
//! the rest of the path attached).
//!
//! A new method [`cut`](trait.Cps.html#method.cut) is provided.
//! It allows one to mark the place from which the detach starts.
//!
//! #### Difference #3
//!
//! Also about `detach`.
//!
//! The [`Attach`](trait.Attach.html) trait changed its parameter to `View` 
//! instead of `CPS`.
//!
//! Usually it's sufficient to change `CPS` to `CPS::View` in generic code.
//!
//! Interestingly, there are some cases when
//!
//! ```
//! # use smart_access::*; #[cfg(feature="detach")]
//! fn foo<CPS: Cps, Path: Attach<CPS::View>> // ...
//! # (){}
//! ```
//!
//! is not equivalent to
//!
//! ```
//! # use smart_access::*; #[cfg(feature="detach")]
//! fn foo<CPS: Cps<View=V>, Path: Attach<V>, V: ?Sized> // ...
//! # (){}
//! ```
//!
//! For example, `impl Attach<V>` works but `impl Attach<CPS::View>` doesn't.
//!
//!
//! ## Cargo features
//!
//! Currently there are following features:
//!
//! * `std`: Links to std.
//! * `std_collections`: Provides accessors for stdlib collections.
//! * `batch_rt`: Provides runtime [batching](struct.CpsBatch.html).
//! * `batch_ct`: Provides compile-time [batching](struct.CpsBatch.html). 
//!   Compatible with `no_std`.
//! * `detach`: Makes [`AT`](struct.AT.html)-paths [detachable](struct.AT.html#method.detach). 
//!   Compatible with `no_std`.
//!
//! All features are enabled by default.

#![cfg_attr(not(feature="std"), no_std)]

mod at;
pub mod core_impls;

#[cfg(feature="std_collections")]
pub mod stdlib_impls;

pub use at::{At, AT, Cps};

#[cfg(any(feature="batch_rt", feature="batch_ct"))]
mod batch;

#[cfg(any(feature="batch_rt", feature="batch_ct"))]
pub use batch::{ CpsBatch, Batch };

#[cfg(feature="batch_ct")]
pub use batch::{ BatchCt };

#[cfg(feature="batch_rt")]
pub use batch::{ BatchRt };

#[cfg(feature="detach")]
pub use at::{ Attach, detached_at };

