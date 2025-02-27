// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg(feature = "std")]

use iota_util_mem::{MallocSizeOf, MallocSizeOfExt};

#[test]
fn derive_vec() {
    #[derive(MallocSizeOf)]
    struct Trivia {
        v: Vec<u8>,
    }

    let t = Trivia { v: vec![0u8; 1024] };

    assert!(t.malloc_size_of() > 1000);
}

#[test]
fn derive_hashmap() {
    #[derive(MallocSizeOf, Default)]
    struct Trivia {
        hm: std::collections::HashMap<u64, Vec<u8>>,
    }

    let mut t = Trivia::default();

    t.hm.insert(1, vec![0u8; 2048]);

    assert!(t.malloc_size_of() > 2000);
}

#[test]
fn derive_ignore() {
    #[derive(MallocSizeOf, Default)]
    struct Trivia {
        hm: std::collections::HashMap<u64, Vec<u8>>,
        #[ignore_malloc_size_of = "I don't like vectors"]
        v: Vec<u8>,
    }

    let mut t = Trivia::default();

    t.hm.insert(1, vec![0u8; 2048]);
    t.v = vec![0u8; 1024];
    assert!(t.malloc_size_of() < 3000);
}

#[test]
#[cfg(all(feature = "lru", feature = "hashbrown"))]
fn derive_morecomplex() {
    #[derive(MallocSizeOf)]
    struct Trivia {
        hm: hashbrown::HashMap<u64, Vec<u8>>,
        cache: lru::LruCache<u128, Vec<u8>>,
    }

    let mut t = Trivia {
        hm: hashbrown::HashMap::new(),
        cache: lru::LruCache::unbounded(),
    };

    t.hm.insert(1, vec![0u8; 2048]);
    t.cache.put(1, vec![0u8; 2048]);
    t.cache.put(2, vec![0u8; 4096]);

    assert!(t.malloc_size_of() > 8000);
}

#[test]
fn derive_tuple() {
    #[derive(MallocSizeOf)]
    struct Trivia {
        tp1: (),
        tp2: (Vec<u8>, Vec<u8>),
    }

    let t = Trivia {
        tp1: (),
        tp2: (vec![7u8; 1024], vec![9u8; 1024]),
    };

    assert!(t.malloc_size_of() > 2000);
    assert!(t.malloc_size_of() < 3000);
}
