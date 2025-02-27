// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod executor;
mod metrics;
mod progress_store;
mod reader;
#[cfg(test)]
mod tests;
mod util;
mod worker_pool;

use anyhow::Result;
use async_trait::async_trait;
pub use executor::{setup_single_workflow, IndexerExecutor, MAX_CHECKPOINTS_IN_PROGRESS};
use iota_types::{
    full_checkpoint_content::CheckpointData, messages_checkpoint::CheckpointSequenceNumber,
};
pub use metrics::DataIngestionMetrics;
pub use progress_store::{FileProgressStore, ProgressStore};
pub use reader::ReaderOptions;
pub use util::create_remote_store_client;
pub use worker_pool::WorkerPool;

#[async_trait]
pub trait Worker: Send + Sync {
    async fn process_checkpoint(&self, checkpoint: CheckpointData) -> Result<()>;
    /// Optional method. Allows controlling when workflow progress is updated in
    /// the progress store. For instance, some pipelines may benefit from
    /// aggregating checkpoints, thus skipping the saving of updates for
    /// intermediate checkpoints. The default implementation is to update
    /// the progress store for every processed checkpoint.
    async fn save_progress(
        &self,
        sequence_number: CheckpointSequenceNumber,
    ) -> Option<CheckpointSequenceNumber> {
        Some(sequence_number)
    }
}
