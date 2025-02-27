// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_types::transaction::Transaction;
pub use move_tx_generator::MoveTxGenerator;
pub use package_publish_tx_generator::PackagePublishTxGenerator;
pub use root_object_create_tx_generator::RootObjectCreateTxGenerator;

use crate::mock_account::Account;

mod move_tx_generator;
mod package_publish_tx_generator;
mod root_object_create_tx_generator;

pub(crate) trait TxGenerator: Send + Sync {
    /// Given an account that contains a sender address, a keypair for that
    /// address, and a list of gas objects owned by this address, generate a
    /// single transaction.
    fn generate_tx(&self, account: Account) -> Transaction;

    fn name(&self) -> &'static str;
}
