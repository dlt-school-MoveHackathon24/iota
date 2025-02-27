// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{fs, path::PathBuf};

use clap::Parser;
use iota_graphql_rpc::{
    commands::Command,
    config::{ConnectionConfig, Ide, ServerConfig, ServiceConfig, TxExecFullNodeConfig, Version},
    server::{builder::export_schema, graphiql_server::start_graphiql_server},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

// WARNING!!!
//
// Do not move or use similar logic to generate git revision information outside
// of a binary entry point (e.g. main.rs). Placing the below logic into a
// library can result in unnecessary builds.
const GIT_REVISION: &str = {
    if let Some(revision) = option_env!("GIT_REVISION") {
        revision
    } else {
        git_version::git_version!(
            args = ["--always", "--abbrev=40", "--dirty", "--exclude", "*"],
            fallback = "DIRTY"
        )
    }
};

// VERSION mimics what other iota binaries use for the same const
static VERSION: Version = Version {
    year: env!("CARGO_PKG_VERSION_MAJOR"),
    month: env!("CARGO_PKG_VERSION_MINOR"),
    patch: env!("CARGO_PKG_VERSION_PATCH"),
    sha: GIT_REVISION,
    full: const_str::concat!(
        env!("CARGO_PKG_VERSION_MAJOR"),
        ".",
        env!("CARGO_PKG_VERSION_MINOR"),
        ".",
        env!("CARGO_PKG_VERSION_PATCH"),
        "-",
        GIT_REVISION
    ),
};

#[tokio::main]
async fn main() {
    let cmd: Command = Command::parse();
    match cmd {
        Command::GenerateDocsExamples => {
            let mut buf: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            // we are looking to put examples content in
            // iota/docs/content/references/iota-graphql/examples.mdx
            let filename = "docs/content/references/iota-graphql/examples.mdx";
            buf.pop();
            buf.pop();
            buf.push(filename);
            let content = iota_graphql_rpc::examples::generate_examples_for_docs()
                .expect("Generating examples markdown file for docs failed");
            std::fs::write(buf, content).expect("Writing examples markdown failed");
            println!("Generated the docs example.mdx file and copied it to {filename}.");
        }
        Command::GenerateSchema { file } => {
            let out = export_schema();
            if let Some(file) = file {
                println!("Write schema to file: {:?}", file);
                std::fs::write(file, &out).unwrap();
            } else {
                println!("{}", &out);
            }
        }
        Command::GenerateExamples { file } => {
            let new_content: String = iota_graphql_rpc::examples::generate_markdown()
                .expect("Generating examples markdown failed");
            let mut buf: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            buf.push("docs");
            buf.push("examples.md");
            let file = file.unwrap_or(buf);

            std::fs::write(file.clone(), new_content).expect("Writing examples markdown failed");
            println!("Written examples to file: {:?}", file);
        }
        Command::StartServer {
            ide_title,
            db_url,
            port,
            host,
            config,
            node_rpc_url,
            prom_host,
            prom_port,
        } => {
            let connection = ConnectionConfig::new(port, host, db_url, None, prom_host, prom_port);
            let service_config = service_config(config);
            let _guard = telemetry_subscribers::TelemetryConfig::new()
                .with_env()
                .init();
            let tracker = TaskTracker::new();
            let cancellation_token = CancellationToken::new();

            println!("Starting server...");
            let server_config = ServerConfig {
                connection,
                service: service_config,
                ide: Ide::new(ide_title),
                tx_exec_full_node: TxExecFullNodeConfig::new(node_rpc_url),
                ..ServerConfig::default()
            };

            let cancellation_token_clone = cancellation_token.clone();
            let graphql_service_handle = tracker.spawn(async move {
                start_graphiql_server(&server_config, &VERSION, cancellation_token_clone)
                    .await
                    .unwrap();
            });

            // Wait for shutdown signal
            tokio::select! {
                result = graphql_service_handle => {
                    if let Err(e) = result {
                        println!("GraphQL service crashed or exited with error: {:?}", e);
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("Ctrl+C signal received.");
                },
            }

            println!("Shutting down...");

            // Send shutdown signal to application
            cancellation_token.cancel();
            tracker.close();
            tracker.wait().await;
        }
    }
}

fn service_config(path: Option<PathBuf>) -> ServiceConfig {
    let Some(path) = path else {
        return ServiceConfig::default();
    };

    let contents = fs::read_to_string(path).expect("Reading configuration");
    ServiceConfig::read(&contents).expect("Deserializing configuration")
}
