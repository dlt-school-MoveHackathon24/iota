// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use config::AuthorityIdentifier;
use iota_macros::fail_point;
use store::{
    reopen,
    rocks::{open_cf, DBMap, MetricConf, ReadWriteOptions},
    Map, TypedStoreError,
};
use types::{Vote, VoteAPI, VoteInfo};

use crate::NodeStorage;

/// The storage for the last votes digests per authority
#[derive(Clone)]
pub struct VoteDigestStore {
    store: DBMap<AuthorityIdentifier, VoteInfo>,
}

impl VoteDigestStore {
    pub fn new(vote_digest_store: DBMap<AuthorityIdentifier, VoteInfo>) -> VoteDigestStore {
        Self {
            store: vote_digest_store,
        }
    }

    pub fn new_for_tests() -> VoteDigestStore {
        let rocksdb = open_cf(
            tempfile::tempdir().unwrap(),
            None,
            MetricConf::default(),
            &[NodeStorage::VOTES_CF],
        )
        .expect("Cannot open database");
        let map = reopen!(&rocksdb, NodeStorage::VOTES_CF;<AuthorityIdentifier, VoteInfo>);
        VoteDigestStore::new(map)
    }

    /// Insert the vote's basic details into the database for the corresponding
    /// header author key.
    #[allow(clippy::let_and_return)]
    pub fn write(&self, vote: &Vote) -> Result<(), TypedStoreError> {
        fail_point!("narwhal-store-before-write");

        let result = self.store.insert(&vote.origin(), &vote.into());

        fail_point!("narwhal-store-after-write");
        result
    }

    /// Read the vote info based on the provided corresponding header author key
    pub fn read(
        &self,
        header_author: &AuthorityIdentifier,
    ) -> Result<Option<VoteInfo>, TypedStoreError> {
        self.store.get(header_author)
    }
}
