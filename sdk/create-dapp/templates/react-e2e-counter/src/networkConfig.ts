// Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { getFullnodeUrl } from "@iota/iota-sdk/client";
import {
  DEVNET_COUNTER_PACKAGE_ID,
  MAINNET_COUNTER_PACKAGE_ID,
} from "./constants.ts";
import { createNetworkConfig } from "@iota/dapp-kit";

const { networkConfig, useNetworkVariable, useNetworkVariables } =
  createNetworkConfig({
    devnet: {
      url: getFullnodeUrl("devnet"),
      variables: {
        counterPackageId: DEVNET_COUNTER_PACKAGE_ID,
      },
    },
    mainnet: {
      url: getFullnodeUrl("mainnet"),
      variables: {
        counterPackageId: MAINNET_COUNTER_PACKAGE_ID,
      },
    },
  });

export { useNetworkVariable, useNetworkVariables, networkConfig };
