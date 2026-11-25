//! Collection types.
//!
//! This module re-exports all collection types provided by [`alloc::collections`]
//! ([`BTreeMap`](alloc::collections::BTreeMap), [`VecDeque`](alloc::collections::VecDeque),
//! [`BinaryHeap`](alloc::collections::BinaryHeap), ...), and additionally provides
//! [`HashMap`], which is not available in `alloc`.

pub use alloc::collections::*;

mod hash_map;

pub use self::hash_map::HashMap;
