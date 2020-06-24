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
smart_access = "0.5"
```

in your `Cargo.toml`.

### Variants

The library, although being very small, includes some pluggable components.

For a bare-bones version use

```
smart_access = { version = "0.5", default-features = false }
```

But usually you'll want something more convenient.

#### Accessors for Vec, HashMap and BTreeMap

```
smart_access = { version = "0.5", default-features = false, features = ["std_collections"] }
```

#### A maximal `no_std` variant

```
smart_access = { version = "0.5", default-features = false, features = ["batch_ct", "detach"] }
```


## Versions

* `0.5.0`: The `detach` feature has been reworked. (WIP)
* `0.4.1`: Fixed some serious bugs in the `detach`-enabled version of the crate.
* `0.4.0`: Public API for using access batches as function inputs/outputs.
* `0.3.0`: Public API for using detached accessors as function inputs/outputs.
* `0.2.2`: New feature `detach` allows one to detach an accessor from the source of the data accessed.
* `0.2.1`: Now really works on `no_std`.
* `0.2.0`: Simplistic batch editing + breaking change for rt-batches + doc improvements.
* `0.1.2`: A bit more user-friendly docs.
* `0.1.1`: Only the README has been updated.
* `0.1.0`: The first iteration.

