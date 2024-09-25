import { buildModule } from "@nomicfoundation/hardhat-ignition/modules";
import deployedAddresses from '../deployments/chain-31337/deployed_addresses.json';

const wiotaAddress = deployedAddresses["WIOTAModule#WIOTA"];

const BridgeModule = buildModule("BridgeModule", (m) => {

  const bridge = m.contract("Bridge", [wiotaAddress]);

  const wiota = m.contractAt("WIOTA", wiotaAddress);

  // Transfer ownership of WIOTA to the Bridge contract
  m.call(wiota, "transferOwnership", [bridge])

  return { bridge };
});

export default BridgeModule;
