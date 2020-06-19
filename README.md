# Smart accessors for Rust

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
smart_access = "0.2"
```

in your `Cargo.toml`.

## Versions

* `0.2.1`: WIP
* `0.2.0`: Simplistic batch editing + breaking change for rt-batches + doc improvements.
* `0.1.2`: A bit more user-friendly docs.
* `0.1.1`: Only the README has been updated.
* `0.1.0`: The first iteration.

