import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";

const config: HardhatUserConfig = {
  solidity: "0.8.24",
  networks: {
    hardhat: {
      // See: https://hardhat.org/hardhat-network/docs/reference#mining-modes
      mining: {
        auto: true,
        // Produce new block every 3 minutes to resolve next issues
        // https://github.com/NomicFoundation/hardhat/issues/2053
        // https://github.com/ethers-io/ethers.js/issues/2338
        // https://github.com/ethers-io/ethers.js/discussions/4116
        interval: 3 * 60 * 1000, // should be less then 5 minutes to make event subscription work
      },
    },
  },
};

export default config;
