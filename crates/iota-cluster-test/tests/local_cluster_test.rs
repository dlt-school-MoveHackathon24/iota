// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_cluster_test::{config::ClusterTestOpt, ClusterTest};

#[tokio::test]
async fn cluster_test() {
    telemetry_subscribers::init_for_testing();

    ClusterTest::run(ClusterTestOpt::new_local()).await;
}

#[cfg(feature = "pg_integration")]
#[tokio::test]
async fn test_iota_cluster() {
    use iota_cluster_test::{
        cluster::{Cluster, LocalNewCluster},
        config::Env,
    };
    use iota_graphql_rpc::client::simple_client::SimpleClient;
    use reqwest::StatusCode;
    use tokio::time::sleep;
    let fullnode_rpc_port: u16 = 9020;
    let indexer_rpc_port: u16 = 9124;
    let pg_address = "postgres://postgres:postgrespw@localhost:5432/iota_indexer".to_string();
    let graphql_address = format!("127.0.0.1:{}", 8000);

    let opts = ClusterTestOpt {
        env: Env::NewLocal,
        faucet_address: None,
        fullnode_address: Some(format!("127.0.0.1:{}", fullnode_rpc_port)),
        epoch_duration_ms: Some(60000),
        indexer_address: Some(format!("127.0.0.1:{}", indexer_rpc_port)),
        pg_address: Some(pg_address),
        config_dir: None,
        graphql_address: Some(graphql_address),
        local_migration_snapshots: Default::default(),
        remote_migration_snapshots: Default::default(),
    };

    let _cluster = LocalNewCluster::start(&opts).await.unwrap();

    let grphql_url: String = format!("http://127.0.0.1:{}", 8000);

    sleep(std::time::Duration::from_secs(20)).await;

    // Try JSON RPC URL
    let query = r#"
        {
            checkpoint {
                sequenceNumber
            }
        }
    "#;
    let resp = SimpleClient::new(grphql_url)
        .execute_to_graphql(query.to_string(), true, vec![], vec![])
        .await
        .unwrap();

    assert!(resp.errors().is_empty());
    assert!(resp.http_status() == StatusCode::OK);
    let resp_body = resp.response_body().data.clone().into_json().unwrap();
    assert!(resp_body.get("checkpoint").is_some());
}
