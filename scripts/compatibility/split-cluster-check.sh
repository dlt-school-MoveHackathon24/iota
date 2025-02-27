#!/bin/bash
# Copyright (c) Mysten Labs, Inc.
# Modifications Copyright (c) 2024 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0

# Runs a cluster with 2 validators built at one commit, and 2 at another, and
# verifies that the cluster can produce checkpoints and reconfigure
#
# Usage:
#
# WORKING_DIR=/tmp/split-cluster-check ./scripts/compatibility/split-cluster-check.sh
#
# You can then re-run using the same WORKING_DIR to skip building the binaries
# every time. If you omit WORKING_DIR, a temp dir will be created and used.

# first arg is the released commit, defaults to `origin/mainnet`
RELEASE_COMMIT=${1:-origin/mainnet}

# second arg is the release candidate commit, defaults to origin/main
RELEASE_CANDIDATE_COMMIT=${2:-origin/main}

# Abort if git repo is dirty
if ! git diff-index --quiet HEAD --; then
  echo "Git repo is dirty, aborting"
  exit 1
fi

# if WORKING_DIR is not set, create a temp dir
if [ -z "$WORKING_DIR" ]; then
  WORKING_DIR=$(mktemp -d)
fi

echo "Using working dir $WORKING_DIR"

REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"

# check if binaries have already been built
if [ -f "$WORKING_DIR/iota-node-release" ] && [ -f "$WORKING_DIR/iota-release" ] && [ -f "$WORKING_DIR/iota-node-candidate" ]; then
  echo "Binaries already built, skipping build"
else
  echo "Building iota-node and iota at $RELEASE_COMMIT"

  # remember current commit
  CURRENT_COMMIT=$(git rev-parse HEAD)

  git checkout $RELEASE_COMMIT || exit 1
  cargo build --bin iota-node --bin iota || exit 1
  cp ./target/debug/iota-node "$WORKING_DIR/iota-node-release"
  cp ./target/debug/iota "$WORKING_DIR/iota-release"

  echo "Building iota-node at $RELEASE_CANDIDATE_COMMIT"
  git checkout $RELEASE_CANDIDATE_COMMIT || exit 1
  cargo build --bin iota-node || exit 1
  cp ./target/debug/iota-node "$WORKING_DIR/iota-node-candidate"

  echo "returning to $CURRENT_COMMIT"
  git checkout $CURRENT_COMMIT || exit 1
fi

export IOTA_CONFIG_DIR="$WORKING_DIR/config"
rm -rf "$IOTA_CONFIG_DIR"

"$WORKING_DIR/iota-release" genesis --epoch-duration-ms 20000

LOG_DIR="$WORKING_DIR/logs"

mkdir -p "$LOG_DIR"

# read all configs in the config dir to an array
CONFIGS=()
while IFS= read -r -d '' file; do
  CONFIGS+=("$file")
done < <(find "$IOTA_CONFIG_DIR" -name "127.0.0.1*.yaml" -print0)

export RUST_LOG=iota=debug,info

# 2 release nodes
"$WORKING_DIR/iota-node-release" --config-path "${CONFIGS[0]}" > "$LOG_DIR/node-0.log" 2>&1 &
"$WORKING_DIR/iota-node-release" --config-path "${CONFIGS[1]}" > "$LOG_DIR/node-1.log" 2>&1 &

# 2 candidate nodes
"$WORKING_DIR/iota-node-candidate" --config-path "${CONFIGS[2]}" > "$LOG_DIR/node-2.log" 2>&1 &
"$WORKING_DIR/iota-node-candidate" --config-path "${CONFIGS[3]}" > "$LOG_DIR/node-3.log" 2>&1 &

# and a fullnode
"$WORKING_DIR/iota-node-release" --config-path "$IOTA_CONFIG_DIR/fullnode.yaml" > "$LOG_DIR/fullnode.log" 2>&1 &

echo "sleeping for 60 seconds"
sleep 60

# kill all child processes
echo "shutting down nodes"
pkill -P $$

# ensure that "Node State has been reconfigured" is in "$LOG_DIR/fullnode.log"
if ! grep -q "Node State has been reconfigured" "$LOG_DIR/fullnode.log"; then
  echo "Could not find 'Node State has been reconfigured' in fullnode log"
  exit 1
fi

echo "Cluster reconfigured successfully"

exit 0
