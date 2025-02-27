// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::{Duration, SystemTime};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use iota_types::storage::ReadStore;
use tap::Pipe;

use crate::Result;

pub const HEALTH_PATH: &str = "/health";

#[derive(serde::Deserialize)]
pub struct Threshold {
    threshold_seconds: Option<u32>,
}

pub async fn health<S: ReadStore>(
    Query(Threshold { threshold_seconds }): Query<Threshold>,
    State(state): State<S>,
) -> Result<impl IntoResponse> {
    let summary = state.get_latest_checkpoint()?;

    // If we have a provided threshold, check that it's close to the current time
    if let Some(threshold_seconds) = threshold_seconds {
        let latest_chain_time = summary.timestamp();

        let threshold = SystemTime::now() - Duration::from_secs(threshold_seconds as u64);

        if latest_chain_time < threshold {
            return Err(anyhow::anyhow!(
                "The latest checkpoint timestamp is less than the provided threshold"
            )
            .into());
        }
    }

    StatusCode::OK.pipe(Ok)
}
