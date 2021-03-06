# Smart accessors for Rust

[![crate](https://img.shields.io/crates/v/smart_access)](https://crates.io/crates/smart_access/)
[![docs](https://docs.rs/smart_access/badge.svg)](https://docs.rs/smart_access/)

## Overview

There are many sorts of &#8220;smart pointers&#8221;. They have following things in common:

* a simple protocol for accessing the data (make some bookkeeping and then give something equivalent to a raw pointer to the data)
* some nontrivial logic of ownership management


This crate provides &#8220;smart accessors&#8221;:

* they aren't concerned with questions of ownership
* they give a _bidirectional view_ of the data: updating the accessed data can 
  cause a nontrivial change of other data linked with the data being accessed
* the accessed view can be entirely virtual: it can be constructed only for the 
  duration of the access

For code examples see [the docs](https://docs.rs/smart_access/).

## Usage

Simply include 

```
smart_access = "0.7"
```

in your `Cargo.toml`.

### Variants

The library, although being very small, includes some pluggable components.

For a bare-bones version use

```
smart_access = { version = "0.7", default-features = false }
```

But usually you'll want something more convenient.

#### Accessors for Vec, HashMap and BTreeMap

```
smart_access = { version = "0.7", default-features = false, features = ["collections", "hashbrown"] }
```

#### A maximal `no_std` and no-`alloc` variant

```
smart_access = { version = "0.7", default-features = false, features = ["batch_ct", "detach", "traversal"] }
```


## Versions

* `0.7.0`: Now fully independent of `std`.
* `0.6.2`: An accessor to the `Vec`-owned slice + some doc improvements.
* `0.6.1`: Fixed iterator accessors panicking on too large ranges.
* `0.6.0`: Accessors for iterators + `get_clone` method on the `Cps` trait.
* `0.5.4`: Accessors for stdlib sets + doc improvements.
* `0.5.3`: A new sort of map accessors (wrapping `and_modify(..).or_insert(..)`).
* `0.5.2`: Added a macro for forming pathlike types.
* `0.5.1`: Some errors in the documentation fixed. A concrete type of detached paths is now public.
* `0.5.0`: A change in the presentation of the `AT` struct. The `detach` feature reworked. Docs now have a version migration guide.
* `0.4.1`: Fixed some serious bugs in the `detach`-enabled version of the crate.
* `0.4.0`: Public API for using access batches as function inputs/outputs.
* `0.3.0`: Public API for using detached accessors as function inputs/outputs.
* `0.2.2`: New feature `detach` allows one to detach an accessor from the source of the data accessed.
* `0.2.1`: Now really works on `no_std`.
* `0.2.0`: Simplistic batch editing + breaking change for rt-batches + doc improvements.
* `0.1.2`: A bit more user-friendly docs.
* `0.1.1`: Only the README has been updated.
* `0.1.0`: The first iteration.

