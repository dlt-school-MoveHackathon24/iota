// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{ffi::OsString, fs, path::PathBuf, time::Duration};

use anyhow::Result;
use backoff::backoff::Backoff;
use futures::StreamExt;
use iota_metrics::spawn_monitored_task;
use iota_storage::blob::Blob;
use iota_types::{
    full_checkpoint_content::CheckpointData, messages_checkpoint::CheckpointSequenceNumber,
};
use notify::{RecursiveMode, Watcher};
use object_store::{path::Path, ObjectStore};
use tap::pipe::Pipe;
use tokio::{
    sync::{mpsc, mpsc::error::TryRecvError, oneshot},
    time::timeout,
};
use tracing::{debug, error, info};

use crate::{create_remote_store_client, executor::MAX_CHECKPOINTS_IN_PROGRESS};

/// Implements a checkpoint reader that monitors a local directory.
/// Designed for setups where the indexer daemon is colocated with FN.
/// This implementation is push-based and utilizes the inotify API.
pub struct CheckpointReader {
    path: PathBuf,
    remote_store_url: Option<String>,
    remote_store_options: Vec<(String, String)>,
    current_checkpoint_number: CheckpointSequenceNumber,
    last_pruned_watermark: CheckpointSequenceNumber,
    checkpoint_sender: mpsc::Sender<CheckpointData>,
    processed_receiver: mpsc::Receiver<CheckpointSequenceNumber>,
    remote_fetcher_receiver: Option<mpsc::Receiver<Result<CheckpointData>>>,
    exit_receiver: oneshot::Receiver<()>,
    options: ReaderOptions,
}

#[derive(Clone)]
pub struct ReaderOptions {
    pub tick_interval_ms: u64,
    pub timeout_secs: u64,
    pub batch_size: usize,
}

impl Default for ReaderOptions {
    fn default() -> Self {
        Self {
            tick_interval_ms: 100,
            timeout_secs: 5,
            batch_size: 100,
        }
    }
}

impl CheckpointReader {
    /// Represents a single iteration of the reader.
    /// Reads files in a local directory, validates them, and forwards
    /// `CheckpointData` to the executor.
    async fn read_local_files(&self) -> Result<Vec<CheckpointData>> {
        let mut files = vec![];
        for entry in fs::read_dir(self.path.clone())? {
            let entry = entry?;
            let filename = entry.file_name();
            if let Some(sequence_number) = Self::checkpoint_number_from_file_path(&filename) {
                if sequence_number >= self.current_checkpoint_number {
                    files.push((sequence_number, entry.path()));
                }
            }
        }
        files.sort();
        debug!("unprocessed local files {:?}", files);
        let mut checkpoints = vec![];
        for (_, filename) in files.iter().take(MAX_CHECKPOINTS_IN_PROGRESS) {
            let checkpoint = Blob::from_bytes::<CheckpointData>(&fs::read(filename)?)?;
            checkpoints.push(checkpoint);
        }
        Ok(checkpoints)
    }

    fn exceeds_capacity(&self, checkpoint_number: CheckpointSequenceNumber) -> bool {
        (MAX_CHECKPOINTS_IN_PROGRESS as u64 + self.last_pruned_watermark) <= checkpoint_number
    }

    async fn remote_fetch_checkpoint_internal(
        store: &dyn ObjectStore,
        checkpoint_number: CheckpointSequenceNumber,
    ) -> Result<CheckpointData> {
        let path = Path::from(format!("{}.chk", checkpoint_number));
        let response = store.get(&path).await?;
        let bytes = response.bytes().await?;
        Blob::from_bytes::<CheckpointData>(&bytes)
    }

    async fn remote_fetch_checkpoint(
        store: &dyn ObjectStore,
        checkpoint_number: CheckpointSequenceNumber,
    ) -> Result<CheckpointData> {
        let mut backoff = backoff::ExponentialBackoff::default();
        backoff.max_elapsed_time = Some(Duration::from_secs(60));
        backoff.initial_interval = Duration::from_millis(100);
        backoff.current_interval = backoff.initial_interval;
        backoff.multiplier = 1.0;
        loop {
            match Self::remote_fetch_checkpoint_internal(store, checkpoint_number).await {
                Ok(data) => return Ok(data),
                Err(err) if err.to_string().contains("404") => match backoff.next_backoff() {
                    Some(duration) => tokio::time::sleep(duration).await,
                    None => return Err(err),
                },
                Err(err) => return Err(err),
            }
        }
    }

    fn start_remote_fetcher(&mut self) -> mpsc::Receiver<Result<CheckpointData>> {
        let batch_size = self.options.batch_size;
        let start_checkpoint = self.current_checkpoint_number;
        let (sender, receiver) = mpsc::channel(batch_size);
        let url = self
            .remote_store_url
            .clone()
            .expect("remote store url must be set");
        let store = create_remote_store_client(
            url,
            self.remote_store_options.clone(),
            self.options.timeout_secs,
        )
        .expect("failed to create remote store client");

        spawn_monitored_task!(async move {
            let mut checkpoint_stream = (start_checkpoint..u64::MAX)
                .map(|checkpoint_number| Self::remote_fetch_checkpoint(&store, checkpoint_number))
                .pipe(futures::stream::iter)
                .buffered(batch_size);

            while let Some(checkpoint) = checkpoint_stream.next().await {
                if sender.send(checkpoint).await.is_err() {
                    info!("remote reader dropped");
                    break;
                }
            }
        });
        receiver
    }

    fn remote_fetch(&mut self) -> Vec<CheckpointData> {
        let mut checkpoints = vec![];
        if self.remote_fetcher_receiver.is_none() {
            self.remote_fetcher_receiver = Some(self.start_remote_fetcher());
        }
        while !self.exceeds_capacity(self.current_checkpoint_number + checkpoints.len() as u64) {
            match self.remote_fetcher_receiver.as_mut().unwrap().try_recv() {
                Ok(Ok(checkpoint)) => checkpoints.push(checkpoint),
                Ok(Err(err)) => {
                    error!("remote reader transient error {:?}", err);
                    self.remote_fetcher_receiver = None;
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    error!("remote reader channel disconnect error");
                    self.remote_fetcher_receiver = None;
                    break;
                }
                Err(TryRecvError::Empty) => break,
            }
        }
        checkpoints
    }

    async fn sync(&mut self) -> Result<()> {
        let backoff = backoff::ExponentialBackoff::default();
        let mut checkpoints = backoff::future::retry(backoff, || async {
            self.read_local_files()
                .await
                .map_err(backoff::Error::transient)
        })
        .await?;

        if self.remote_store_url.is_some()
            && (checkpoints.is_empty()
                || checkpoints[0].checkpoint_summary.sequence_number
                    > self.current_checkpoint_number)
        {
            checkpoints = self.remote_fetch();
        } else {
            // cancel remote fetcher execution because local reader has made progress
            self.remote_fetcher_receiver = None;
        }

        info!(
            "Local reader. Current checkpoint number: {}, pruning watermark: {}, unprocessed checkpoints: {:?}",
            self.current_checkpoint_number,
            self.last_pruned_watermark,
            checkpoints.len(),
        );
        for checkpoint in checkpoints {
            assert_eq!(
                checkpoint.checkpoint_summary.sequence_number,
                self.current_checkpoint_number
            );
            if self.exceeds_capacity(checkpoint.checkpoint_summary.sequence_number) {
                break;
            }
            self.checkpoint_sender.send(checkpoint).await?;
            self.current_checkpoint_number += 1;
        }
        Ok(())
    }

    /// Cleans the local directory by removing all processed checkpoint files.
    fn gc_processed_files(&mut self, watermark: CheckpointSequenceNumber) -> Result<()> {
        info!("cleaning processed files, watermark is {}", watermark);
        self.last_pruned_watermark = watermark;
        for entry in fs::read_dir(self.path.clone())? {
            let entry = entry?;
            let filename = entry.file_name();
            if let Some(sequence_number) = Self::checkpoint_number_from_file_path(&filename) {
                if sequence_number < watermark {
                    fs::remove_file(entry.path())?;
                }
            }
        }
        Ok(())
    }

    fn checkpoint_number_from_file_path(file_name: &OsString) -> Option<CheckpointSequenceNumber> {
        file_name
            .to_str()
            .and_then(|s| s.rfind('.').map(|pos| &s[..pos]))
            .and_then(|s| s.parse().ok())
    }

    pub fn initialize(
        path: PathBuf,
        starting_checkpoint_number: CheckpointSequenceNumber,
        remote_store_url: Option<String>,
        remote_store_options: Vec<(String, String)>,
        options: ReaderOptions,
    ) -> (
        Self,
        mpsc::Receiver<CheckpointData>,
        mpsc::Sender<CheckpointSequenceNumber>,
        oneshot::Sender<()>,
    ) {
        let (checkpoint_sender, checkpoint_recv) = mpsc::channel(MAX_CHECKPOINTS_IN_PROGRESS);
        let (processed_sender, processed_receiver) = mpsc::channel(MAX_CHECKPOINTS_IN_PROGRESS);
        let (exit_sender, exit_receiver) = oneshot::channel();
        let reader = Self {
            path,
            remote_store_url,
            remote_store_options,
            current_checkpoint_number: starting_checkpoint_number,
            last_pruned_watermark: starting_checkpoint_number,
            checkpoint_sender,
            processed_receiver,
            remote_fetcher_receiver: None,
            options,
            exit_receiver,
        };
        (reader, checkpoint_recv, processed_sender, exit_sender)
    }

    pub async fn run(mut self) -> Result<()> {
        let (inotify_sender, mut inotify_recv) = mpsc::channel(1);
        std::fs::create_dir_all(self.path.clone()).expect("failed to create a directory");
        let mut watcher = notify::recommended_watcher(move |res| {
            if let Err(err) = res {
                eprintln!("watch error: {:?}", err);
            }
            inotify_sender
                .blocking_send(())
                .expect("Failed to send inotify update");
        })
        .expect("Failed to init inotify");

        watcher
            .watch(&self.path, RecursiveMode::NonRecursive)
            .expect("Inotify watcher failed");

        loop {
            tokio::select! {
                _ = &mut self.exit_receiver => break,
                Some(gc_checkpoint_number) = self.processed_receiver.recv() => {
                    self.gc_processed_files(gc_checkpoint_number).expect("Failed to clean the directory");
                }
                Ok(Some(_)) | Err(_) = timeout(Duration::from_millis(self.options.tick_interval_ms), inotify_recv.recv())  => {
                    self.sync().await.expect("Failed to read checkpoint files");
                }
            }
        }
        Ok(())
    }
}
